//! Escuchar un mashup (ADR-018): el WAV renderizado suena por el
//! reproductor de siempre, con la barra de abajo.
//!
//! Funciona como el fragmento a ciegas: la cola del usuario espera en pausa y
//! vuelve al acabar o al pedir cualquier otra cosa. A diferencia del fragmento,
//! el mashup se puede pausar y recorrer: es una pieza que se escucha entera.
//! No es una pista de la biblioteca: no cuenta como escucha ni va al historial.

use super::{Orden, Servicio};
use crate::audio::PistaEnCola;
use crate::cola::Estado;

pub(super) struct MashupSonando {
    id: i64,
    ficha: u64,
    duracion: f64,
    cola_seg: f64,
    pausado: bool,
}

impl<B: kira::backend::Backend> Servicio<B> {
    /// Toca el mashup `id` desde `desde_seg`.
    pub(super) fn tocar_mashup(
        &mut self,
        id: i64,
        ruta: std::path::PathBuf,
        duracion: f64,
        desde_seg: f64,
    ) {
        let cola_seg = match self.mashup.take() {
            Some(m) => m.cola_seg,
            None => {
                self.cortar_fragmento();
                self.salir_dj();
                let pos = self.motor.posicion();
                let s = if self.actual.as_ref().is_some_and(|s| s.ficha == pos.ficha) {
                    pos.segundos
                } else {
                    self.cola_seg_restaurada
                };
                if self.cola.estado() == Estado::Reproduciendo {
                    let _ = self.cola.pausar_reanudar();
                }
                s
            }
        };
        self.actual = None;
        self.prevista = None;
        self.karaoke = false;
        let ficha = self.siguiente_ficha;
        self.siguiente_ficha += 1;
        self.motor.reproducir(
            PistaEnCola {
                ficha,
                ruta,
                instrumental: None,
                // Ya está normalizado a −14 LUFS al renderizar.
                ganancia_db: 0.0,
            },
            desde_seg.clamp(0.0, duracion),
        );
        self.motor.siguiente(None);
        self.mashup = Some(MashupSonando {
            id,
            ficha,
            duracion,
            cola_seg,
            pausado: false,
        });
    }

    /// Devuelve la cola y deja de tocar el mashup.
    pub(super) fn cortar_mashup(&mut self) {
        if let Some(m) = self.mashup.take() {
            self.motor.detener();
            self.devolver_cola(m.cola_seg);
        }
    }

    /// Al terminar el archivo, la cola vuelve.
    pub(super) fn tic_mashup(&mut self, terminada: bool) -> bool {
        if self.mashup.is_some() && terminada {
            self.cortar_mashup();
            return true;
        }
        false
    }

    /// Lo que la barra pide mientras suena un mashup.
    pub(super) fn interceptar_mashup(&mut self, orden: &Orden) -> bool {
        let Some(m) = self.mashup.as_mut() else {
            return false;
        };
        match orden {
            Orden::PausarReanudar => {
                if m.pausado {
                    self.motor.reanudar();
                } else {
                    self.motor.pausar();
                }
                m.pausado = !m.pausado;
                true
            }
            Orden::Buscar(s) => {
                let s = s.clamp(0.0, m.duracion);
                self.motor.buscar(s);
                true
            }
            Orden::DetenerMashup => {
                self.cortar_mashup();
                true
            }
            // No hay ni siguiente ni anterior ni karaoke en un mashup.
            Orden::Siguiente | Orden::Anterior | Orden::Karaoke(_) => true,
            Orden::Mashup { .. }
            | Orden::Volumen(_)
            | Orden::Ecualizador(_)
            | Orden::Normalizar(_) => false,
            _ => {
                self.cortar_mashup();
                false
            }
        }
    }

    pub(super) fn instantanea_mashup(&self, i: &mut super::Instantanea) {
        let Some(m) = &self.mashup else {
            return;
        };
        let pos = self.motor.posicion();
        i.pista_id = None;
        i.mashup_id = Some(m.id);
        i.estado = if m.pausado {
            "pausado"
        } else {
            "reproduciendo"
        }
        .to_owned();
        i.duracion_seg = m.duracion;
        i.posicion_seg = if pos.ficha == m.ficha {
            pos.segundos.clamp(0.0, m.duracion)
        } else {
            0.0
        };
        i.karaoke_disponible = false;
        i.karaoke_activo = false;
    }

    pub(super) fn cola_seg_mashup(&self) -> Option<f64> {
        self.mashup.as_ref().map(|m| m.cola_seg)
    }
}
