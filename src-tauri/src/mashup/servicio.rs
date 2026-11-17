//! La orquestación de una generación (ADR-018).
//!
//! Una sola generación a la vez, en un hilo de prioridad baja (`prioridad.rs`):
//! el reproductor y la interfaz no la notan. La separación comparte el turno de
//! HTDemucs con la cola del karaoke ([`karaoke::turno`]), así que nunca hay dos
//! sesiones del modelo en memoria.
//!
//! **Orden de las etapas, de lo barato a lo caro.**
//! 1. Prefiltro con los rasgos guardados ([`seleccion`]).
//! 2. Estructura de las preseleccionadas (o la guardada, si sigue valiendo).
//! 3. Un primer plan sin stems: dice qué pistas va a usar el mashup.
//! 4. Stems solo de esas pistas (o los de la caché).
//! 5. El plan definitivo con voz y capas, y el render.
//!
//! Así se separan del orden de las canciones pedidas, no de la biblioteca.
//!
//! **Progreso.** Cada etapa informa de lo que lleva de verdad (pistas hechas,
//! fracción de la separación en curso, fracción del render). Los avisos se
//! limitan a uno por punto porcentual para no saturar el puente.

use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use rusqlite::params;
use serde::Serialize;
use sha2::{Digest, Sha256};
use ts_rs::TS;

use super::compositor::{self, Fuente, Material, Peticion, Plan};
use super::estructura::{self, Estructura};
use super::persistencia::{self, PeticionMashup, PistaDeMashup};
use super::render::{self, Origen, TASA};
use super::seleccion;
use super::stems::{self, CacheStems, Tramo};
use crate::audio::decodificador::Decodificador;
use crate::db::BaseDeDatos;
use crate::error::{NsError, NsResult};
use crate::karaoke::{self, STEMS};
use crate::paths::Rutas;
use crate::reproductor::persistencia as rp;

/// Las etapas que ve el usuario.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub enum Etapa {
    Eligiendo,
    Analizando,
    Separando,
    Componiendo,
    Mezclando,
}

/// El progreso de la generación en curso.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Progreso {
    #[ts(type = "number")]
    pub mashup_id: i64,
    pub etapa: Etapa,
    /// Unidades hechas y totales de la etapa (pistas, o 0 y 0 si no aplica).
    pub hechas: u32,
    pub total: u32,
    /// De 0 a 1, la generación entera.
    pub fraccion: f64,
}

/// Lo que el servicio cuenta hacia fuera.
#[derive(Debug, Clone, PartialEq)]
pub enum Aviso {
    Progreso(Progreso),
    /// Terminó, bien o mal: la interfaz vuelve a leer el mashup.
    Terminado {
        mashup_id: i64,
    },
}

type Avisador = Arc<dyn Fn(Aviso) + Send + Sync>;

/// Separa una mezcla cargada en sus cuatro stems. En la aplicación es
/// HTDemucs ([`karaoke::separar_archivo_stems_de`]); las pruebas le pasan
/// [`karaoke::separar_stems`] con una inferencia controlada, de modo que todo
/// el recorrido (turno, caché, actividad vocal, karaoke) se ejecuta igual.
pub type Separador = Arc<
    dyn Fn(&[f32], &[f32], &mut dyn FnMut(f64) -> bool) -> NsResult<[Option<karaoke::Estereo>; 4]>
        + Send
        + Sync,
>;

struct EnCurso {
    id: i64,
    cancelar: Arc<AtomicBool>,
    progreso: Arc<Mutex<Option<Progreso>>>,
}

/// El servicio. Clonable: todas sus partes se comparten.
#[derive(Clone)]
pub struct Mashups {
    db: BaseDeDatos,
    rutas: Rutas,
    cache: CacheStems,
    en_curso: Arc<Mutex<Option<EnCurso>>>,
    avisar: Avisador,
    /// El separador, si está disponible ahora (el agregado se instala y se
    /// quita en caliente).
    separador: Arc<dyn Fn() -> Option<Separador> + Send + Sync>,
}

/// La carpeta de los mashups renderizados: en datos, porque son del usuario.
#[must_use]
pub fn carpeta(rutas: &Rutas) -> PathBuf {
    rutas.datos.join("mashups")
}

/// La caché de stems: en la caché del sistema, porque se puede rehacer.
#[must_use]
pub fn carpeta_stems(rutas: &Rutas) -> PathBuf {
    rutas.cache.join("stems-mashup")
}

impl Mashups {
    /// Arranca el servicio: da por fallidas las generaciones que cortó un cierre
    /// y borra lo que dejaron a medias.
    ///
    /// # Errors
    ///
    /// [`NsError::StorageFailure`].
    pub fn nuevo(
        db: BaseDeDatos,
        rutas: Rutas,
        avisar: impl Fn(Aviso) + Send + Sync + 'static,
    ) -> NsResult<Self> {
        let r = rutas.clone();
        Self::con_separador(db, rutas, avisar, move || {
            let modelo = karaoke::modelo_instalado(&r)?;
            let s: Separador = Arc::new(move |izq, der, seguir| {
                karaoke::separar_archivo_stems_de(
                    &modelo,
                    izq,
                    der,
                    [true; 4],
                    karaoke::hilos_cola(),
                    seguir,
                )
            });
            Some(s)
        })
    }

    /// Como [`Mashups::nuevo`], con otro separador.
    ///
    /// # Errors
    ///
    /// [`NsError::StorageFailure`].
    pub fn con_separador(
        db: BaseDeDatos,
        rutas: Rutas,
        avisar: impl Fn(Aviso) + Send + Sync + 'static,
        separador: impl Fn() -> Option<Separador> + Send + Sync + 'static,
    ) -> NsResult<Self> {
        let cache = CacheStems::compartida(carpeta_stems(&rutas))?;
        persistencia::recuperar_interrumpidos(&db)?;
        limpiar_huerfanos(&db, &carpeta(&rutas));
        Ok(Self {
            db,
            rutas,
            cache,
            en_curso: Arc::new(Mutex::new(None)),
            avisar: Arc::new(avisar),
            separador: Arc::new(separador),
        })
    }

    /// Si hay separador: sin él, el mashup se compone con mezclas enteras.
    #[must_use]
    pub fn con_stems(&self) -> bool {
        (self.separador)().is_some()
    }

    #[must_use]
    pub fn progreso(&self) -> Option<Progreso> {
        let g = self.en_curso.lock().unwrap_or_else(PoisonError::into_inner);
        g.as_ref().and_then(|e| {
            e.progreso
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone()
        })
    }

    /// Pide que pare la generación en curso, si es `id`.
    pub fn cancelar(&self, id: i64) {
        if let Some(e) = self
            .en_curso
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
        {
            if e.id == id {
                e.cancelar.store(true, Ordering::Relaxed);
            }
        }
    }

    /// Empieza una generación y devuelve su id sin esperar.
    ///
    /// # Errors
    ///
    /// [`NsError::InvalidInput`] si ya hay una en curso o la petición no tiene
    /// sentido; [`NsError::StorageFailure`].
    pub fn generar(&self, mut p: PeticionMashup) -> NsResult<i64> {
        if !(60..=3600).contains(&p.duracion_seg)
            || !(2..=120).contains(&p.canciones)
            || p.pistas.len() > 500
        {
            return Err(NsError::InvalidInput);
        }
        let mut g = self.en_curso.lock().unwrap_or_else(PoisonError::into_inner);
        if g.is_some() {
            return Err(NsError::InvalidInput);
        }
        let semilla = p.semilla.unwrap_or_else(|| {
            let mut a = crate::dj::azar::Azar::sin_semilla();
            (a.real() * 1e15) as i64
        });
        p.semilla = Some(semilla);
        let titulo = self.titulo_siguiente()?;
        let id = persistencia::crear(&self.db, &titulo, &p, semilla, compositor::VERSION)?;
        let cancelar = Arc::new(AtomicBool::new(false));
        let progreso = Arc::new(Mutex::new(None));
        *g = Some(EnCurso {
            id,
            cancelar: Arc::clone(&cancelar),
            progreso: Arc::clone(&progreso),
        });
        drop(g);
        let yo = self.clone();
        let lanzado = std::thread::Builder::new()
            .name("mashup".into())
            .spawn(move || {
                crate::prioridad::bajar_hilo_actual();
                let mut t = Trabajo {
                    s: &yo,
                    id,
                    cancelar: &cancelar,
                    progreso: &progreso,
                    ultimo: -1.0,
                };
                let r = t.ejecutar(&p, semilla);
                if let Err(e) = &r {
                    tracing::warn!(codigo = e.code(), "no se pudo generar un mashup");
                    let _ = persistencia::fallar(&yo.db, id, e);
                }
                *yo.en_curso.lock().unwrap_or_else(PoisonError::into_inner) = None;
                (yo.avisar)(Aviso::Terminado { mashup_id: id });
            });
        if lanzado.is_err() {
            *self.en_curso.lock().unwrap_or_else(PoisonError::into_inner) = None;
            persistencia::fallar(&self.db, id, &NsError::Internal)?;
            return Err(NsError::Internal);
        }
        Ok(id)
    }

    /// Una variante: la misma petición con otra semilla.
    ///
    /// # Errors
    ///
    /// Los de [`Mashups::generar`] y de leer el original.
    pub fn variante(&self, id: i64) -> NsResult<i64> {
        let v = persistencia::vista(&self.db, id)?;
        let mut p = v.peticion;
        p.semilla = Some(
            v.semilla
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407)
                .abs(),
        );
        self.generar(p)
    }

    fn titulo_siguiente(&self) -> NsResult<String> {
        let n: i64 = self
            .db
            .conexion()?
            .query_row("SELECT count(*) FROM mashups", [], |f| f.get(0))
            .map_err(|_| NsError::StorageFailure)?;
        Ok(format!("Mashup {}", n + 1))
    }

    /// La ruta del audio de un mashup listo.
    ///
    /// # Errors
    ///
    /// [`NsError::FileNotFound`] si no está listo o el archivo falta.
    pub fn ruta(&self, id: i64) -> NsResult<PathBuf> {
        let a = persistencia::archivo(&self.db, id)?.ok_or(NsError::FileNotFound)?;
        let r = carpeta(&self.rutas).join(a);
        if r.is_file() {
            Ok(r)
        } else {
            Err(NsError::FileNotFound)
        }
    }

    /// Copia el audio a `destino`. Nunca escribe sobre la música del usuario:
    /// quien llama elige la ruta con el diálogo del sistema.
    ///
    /// # Errors
    ///
    /// [`NsError::FileNotFound`]; [`NsError::StorageFailure`].
    pub fn exportar(&self, id: i64, destino: &Path) -> NsResult<u64> {
        let origen = self.ruta(id)?;
        let parcial = destino.with_extension("wav.parcial");
        let r = std::fs::copy(&origen, &parcial)
            .and_then(|b| std::fs::rename(&parcial, destino).map(|()| b))
            .map_err(|_| NsError::StorageFailure);
        if r.is_err() {
            let _ = std::fs::remove_file(&parcial);
        }
        r
    }

    /// Borra un mashup y su audio. Si se estaba generando, lo cancela antes.
    ///
    /// # Errors
    ///
    /// [`NsError::StorageFailure`].
    pub fn eliminar(&self, id: i64) -> NsResult<()> {
        self.cancelar(id);
        if let Some(a) = persistencia::eliminar(&self.db, id)? {
            let _ = std::fs::remove_file(carpeta(&self.rutas).join(a));
        }
        Ok(())
    }
}

/// Borra de la carpeta de mashups lo que la base no conoce: temporales de una
/// generación cortada.
fn limpiar_huerfanos(db: &BaseDeDatos, dir: &Path) {
    let Ok(conocidos) = persistencia::archivos(db) else {
        return;
    };
    let conocidos: HashSet<String> = conocidos.into_iter().collect();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let nombre = e.file_name().to_string_lossy().into_owned();
            if !conocidos.contains(&nombre) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
}

/// Lo que hace falta de una pista para componer.
struct Pista {
    id: i64,
    ruta: PathBuf,
    sha: String,
    titulo: String,
    artista: String,
    tonica: Option<usize>,
    menor: bool,
}

pub(crate) fn sha_de(ruta: &Path) -> NsResult<String> {
    let mut f = std::fs::File::open(ruta).map_err(|_| NsError::FileNotFound)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = f.read(&mut buf).map_err(|_| NsError::StorageFailure)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Ruta, hash, título, artista, tonalidad y modo de una pista.
type FilaPista = (
    String,
    Option<String>,
    String,
    String,
    Option<String>,
    Option<String>,
);

struct Trabajo<'a> {
    s: &'a Mashups,
    id: i64,
    cancelar: &'a AtomicBool,
    progreso: &'a Mutex<Option<Progreso>>,
    ultimo: f64,
}

/// Pesos de cada etapa en la fracción total. La separación domina cuando hay
/// que hacerla; si todo está en caché, las etapas pasan en un instante.
const PESOS: [(Etapa, f64, f64); 5] = [
    (Etapa::Eligiendo, 0.0, 0.02),
    (Etapa::Analizando, 0.02, 0.25),
    (Etapa::Separando, 0.25, 0.70),
    (Etapa::Componiendo, 0.70, 0.75),
    (Etapa::Mezclando, 0.75, 1.0),
];

/// Publica el progreso: siempre en el estado compartido, y como aviso solo al
/// cambiar de etapa o de unidad, o cada punto porcentual.
#[allow(clippy::too_many_arguments)]
fn emitir(
    s: &Mashups,
    id: i64,
    progreso: &Mutex<Option<Progreso>>,
    ultimo: &mut f64,
    etapa: Etapa,
    hechas: u32,
    total: u32,
    dentro: f64,
) {
    let (_, a, b) = PESOS
        .iter()
        .find(|p| p.0 == etapa)
        .copied()
        .unwrap_or((etapa, 0.0, 1.0));
    let fraccion = a + (b - a) * dentro.clamp(0.0, 1.0);
    let p = Progreso {
        mashup_id: id,
        etapa,
        hechas,
        total,
        fraccion,
    };
    let mut g = progreso.lock().unwrap_or_else(PoisonError::into_inner);
    let cambio = g
        .as_ref()
        .is_none_or(|v| v.etapa != etapa || v.hechas != hechas);
    *g = Some(p.clone());
    drop(g);
    if cambio || fraccion - *ultimo >= 0.01 {
        *ultimo = fraccion;
        (s.avisar)(Aviso::Progreso(p));
    }
}

impl Trabajo<'_> {
    fn seguir(&self) -> NsResult<()> {
        if self.cancelar.load(Ordering::Relaxed) {
            Err(NsError::Cancelled)
        } else {
            Ok(())
        }
    }

    fn informar(&mut self, etapa: Etapa, hechas: u32, total: u32, dentro: f64) {
        emitir(
            self.s,
            self.id,
            self.progreso,
            &mut self.ultimo,
            etapa,
            hechas,
            total,
            dentro,
        );
    }

    fn pistas(&self, ids: &[i64]) -> NsResult<Vec<Pista>> {
        let conn = self.s.db.conexion()?;
        let musica = rp::carpeta_musica(&conn)?;
        let mut v = Vec::new();
        for &id in ids {
            let fila: Option<FilaPista> = conn
                .query_row(
                    "SELECT p.ruta, p.sha256, p.titulo, p.artista_nombre, r.tonalidad, r.modo
                     FROM pistas p LEFT JOIN rasgos r ON r.pista_id = p.id
                     WHERE p.id = ?1 AND p.estado = 'biblioteca'",
                    params![id],
                    |f| {
                        Ok((
                            f.get(0)?,
                            f.get(1)?,
                            f.get(2)?,
                            f.get(3)?,
                            f.get(4)?,
                            f.get(5)?,
                        ))
                    },
                )
                .map(Some)
                .or_else(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => Ok(None),
                    _ => Err(NsError::StorageFailure),
                })?;
            // Una pista que ya no está se salta: el compositor trabaja con las que hay.
            let Some((ruta, sha, titulo, artista, ton, modo)) = fila else {
                continue;
            };
            let r = PathBuf::from(ruta);
            let ruta = match &musica {
                Some(m) if r.is_relative() => m.join(r),
                _ => r,
            };
            if !ruta.is_file() {
                continue;
            }
            let sha = match sha {
                Some(s) if s.len() == 64 => s,
                _ => match sha_de(&ruta) {
                    Ok(s) => s,
                    Err(_) => continue,
                },
            };
            v.push(Pista {
                id,
                ruta,
                sha,
                titulo,
                artista,
                tonica: compositor::tonica(ton.as_deref()),
                menor: modo.is_some_and(|m| m.to_lowercase().contains("min")),
            });
        }
        Ok(v)
    }

    fn ejecutar(&mut self, p: &PeticionMashup, semilla: i64) -> NsResult<()> {
        // 1. Elegir.
        self.informar(Etapa::Eligiendo, 0, 0, 0.0);
        let cupo = (p.canciones as usize * 3 / 2 + 4).min(160);
        let reserva = if !p.pistas.is_empty() || p.playlist_id.is_some() {
            let mut ids = p.pistas.clone();
            if let Some(pl) = p.playlist_id {
                // La lista de la playlist, sea manual, inteligente o automática.
                ids.extend(crate::playlists::ids(&self.s.db, pl)?);
            }
            let todas = crate::dj::planificador::cargar(&self.s.db, 100_000, &[], false)?;
            let pedidas: HashSet<i64> = ids.into_iter().collect();
            todas
                .into_iter()
                .filter(|c| pedidas.contains(&c.id))
                .collect::<Vec<_>>()
        } else {
            crate::dj::planificador::cargar(&self.s.db, 100_000, &[], true)?
        };
        let peticion = Peticion {
            duracion_seg: p.duracion_seg,
            canciones: p.canciones,
            estilo: p.estilo,
            semilla,
        };
        let separador = (self.s.separador)();
        let fuente =
            |pista: &Pista, e: &Estructura, stems: bool, tramo: Option<(usize, usize)>| Fuente {
                pista_id: pista.id,
                artista: pista.artista.clone(),
                tonica: pista.tonica,
                menor: pista.menor,
                estructura: e.clone(),
                stems,
                compases_stems: tramo,
            };
        // Se analiza un cupo de candidatas; si el plan previo no llega a las
        // canciones pedidas, el cupo se amplía una vez (la preselección es
        // determinista, así que el cupo mayor contiene al menor) y solo se
        // analiza lo nuevo.
        let mut pistas: Vec<Pista> = Vec::new();
        let mut estructuras: HashMap<i64, Estructura> = HashMap::new();
        let mut cupos = vec![cupo];
        let ampliado = (cupo * 2).min(160).min(reserva.len());
        if ampliado > cupo {
            cupos.push(ampliado);
        }
        let mut previo: Option<Plan> = None;
        for (ronda, &c) in cupos.iter().enumerate() {
            let candidatas = seleccion::preseleccionar(&reserva, c, semilla);
            if candidatas.len() < 2 {
                return Err(NsError::InvalidInput);
            }
            let nuevas: Vec<i64> = candidatas
                .into_iter()
                .filter(|id| !pistas.iter().any(|x| x.id == *id))
                .collect();
            if ronda > 0 && nuevas.is_empty() {
                break;
            }
            let nuevas = self.pistas(&nuevas)?;
            self.seguir()?;

            // 2. Estructura.
            let total = nuevas.len() as u32;
            for (k, pista) in nuevas.iter().enumerate() {
                self.seguir()?;
                self.informar(
                    Etapa::Analizando,
                    k as u32,
                    total,
                    k as f64 / f64::from(total.max(1)),
                );
                let e = match persistencia::leer_estructura(&self.s.db, pista.id, &pista.sha)? {
                    Some((e, _)) => e,
                    None => match estructura::analizar(&pista.ruta) {
                        Ok(e) => {
                            persistencia::guardar_estructura(&self.s.db, pista.id, &pista.sha, &e)?;
                            e
                        }
                        // Un archivo ilegible no tumba el mashup: se queda fuera.
                        Err(_) => continue,
                    },
                };
                estructuras.insert(pista.id, e);
            }
            pistas.extend(nuevas);

            // 3. Un plan previo dice qué pistas hacen falta y cuáles necesitan
            // stems. Si hay separador, supone stems para todas (sin conocer su
            // voz); lo que el plan use solo como mezcla original no se separa.
            // Cuesta milisegundos: cuenta como el final del análisis.
            self.informar(Etapa::Analizando, total, total, 1.0);
            let fuentes: Vec<Fuente> = pistas
                .iter()
                .filter_map(|x| {
                    let e = estructuras.get(&x.id)?;
                    if separador.is_some() {
                        return Some(fuente(x, e, true, None));
                    }
                    let tramo = compases_cubiertos(self.s.cache.tramo(&x.sha), &e.compases);
                    Some(fuente(x, e, tramo.is_some(), tramo))
                })
                .collect();
            let plan = compositor::componer(&fuentes, &peticion)?;
            self.seguir()?;
            let suficientes = plan.pistas_reconocibles().len() >= p.canciones as usize;
            previo = Some(plan);
            if suficientes {
                break;
            }
        }
        let previo = previo.ok_or(NsError::InvalidInput)?;

        // 4. Stems de las que se usan, solo del tramo que se usa.
        let usadas: Vec<&Pista> = {
            let ids = previo.pistas();
            pistas.iter().filter(|x| ids.contains(&x.id)).collect()
        };
        // Mientras dure la generación, la poda de otra parte (el karaoke) no
        // borra lo que este mashup va a leer.
        let _protegidas: Vec<_> = usadas.iter().map(|x| stems::proteger(&x.sha)).collect();
        let mut con_stems: HashMap<i64, (usize, usize)> = HashMap::new();
        // Sin separador aún sirve lo que ya está en la caché.
        {
            let separador = separador.as_ref();
            let n = usadas.len() as u32;
            for (k, pista) in usadas.iter().enumerate() {
                self.seguir()?;
                self.informar(Etapa::Separando, k as u32, n, k as f64 / f64::from(n));
                let Some(e) = estructuras.get(&pista.id) else {
                    continue;
                };
                let nb = e.n_compases();
                let Some((ca, cb)) = compases_necesarios(&previo, pista.id, nb) else {
                    continue;
                };
                // Dos segundos de contexto a cada lado: el modelo separa peor en
                // los bordes de lo que recibe.
                let muestra = |s: f64| (s.max(0.0) * f64::from(TASA)).round() as u64;
                let tramo = (muestra(e.compases[ca] - 2.0), muestra(e.compases[cb] + 2.0));
                if separador.is_none() && !self.s.cache.cubre(&pista.sha, tramo.0, tramo.1) {
                    continue;
                }
                match self.separar(pista, separador, tramo, k as u32, n) {
                    Ok(voz) => {
                        let t = self.s.cache.tramo(&pista.sha);
                        let disponibles = compases_cubiertos(t, &e.compases);
                        let entera = t.is_some_and(|t| t.hasta.is_none());
                        if let (Some(e), Some(voz)) = (estructuras.get_mut(&pista.id), voz) {
                            e.voz = Some(voz.compases);
                            e.voz_cuartos = Some(voz.cuartos);
                            // Una actividad vocal de un tramo tiene ceros fuera
                            // de él: no se guarda como la de la canción.
                            if entera {
                                persistencia::guardar_estructura(
                                    &self.s.db, pista.id, &pista.sha, e,
                                )?;
                            }
                        }
                        if let Some(d) = disponibles {
                            con_stems.insert(pista.id, d);
                        }
                    }
                    Err(NsError::Cancelled) => return Err(NsError::Cancelled),
                    // Si una separación falla, esa pista entra como mezcla.
                    Err(e) => tracing::warn!(
                        codigo = e.code(),
                        "no se pudo separar una pista para un mashup"
                    ),
                }
            }
        }

        // 5. El plan definitivo y el audio.
        self.informar(Etapa::Componiendo, 0, 0, 0.5);
        // Siempre se recompone: el previo suponía stems que quizá no hay, y el
        // definitivo conoce la voz real de lo separado.
        let plan: Plan = {
            let fuentes: Vec<Fuente> = usadas
                .iter()
                .filter_map(|x| {
                    let tramo = con_stems.get(&x.id).copied();
                    estructuras
                        .get(&x.id)
                        .map(|e| fuente(x, e, tramo.is_some(), tramo))
                })
                .collect();
            compositor::componer(&fuentes, &peticion)?
        };
        self.seguir()?;
        let protegidas: HashSet<String> = usadas.iter().map(|x| x.sha.clone()).collect();
        let mut origen = OrigenReal {
            pistas: pistas
                .iter()
                .map(|x| (x.id, (x.ruta.clone(), x.sha.clone())))
                .collect(),
            cache: self.s.cache.clone(),
        };
        let archivo = format!("{}.wav", self.id);
        let destino = carpeta(&self.s.rutas).join(&archivo);
        let (yo, id, progreso, cancelar) = (self.s, self.id, self.progreso, self.cancelar);
        let mut ultimo = self.ultimo;
        let metricas = render::renderizar(&plan, &estructuras, &mut origen, &destino, &mut |f| {
            emitir(yo, id, progreso, &mut ultimo, Etapa::Mezclando, 0, 0, f);
            !cancelar.load(Ordering::Relaxed)
        })?;
        self.ultimo = ultimo;
        self.informar(Etapa::Mezclando, 1, 1, 1.0);
        let bytes = std::fs::metadata(&destino).map(|m| m.len()).unwrap_or(0);
        let por_id: HashMap<i64, &Pista> = pistas.iter().map(|x| (x.id, x)).collect();
        let lista: Vec<PistaDeMashup> = plan
            .pistas()
            .iter()
            .filter_map(|id| por_id.get(id))
            .map(|x| PistaDeMashup {
                pista_id: Some(x.id),
                titulo: x.titulo.clone(),
                artista: x.artista.clone(),
            })
            .collect();
        if let Err(e) = persistencia::terminar(
            &self.s.db, self.id, &plan, &archivo, bytes, &metricas, &lista,
        ) {
            let _ = std::fs::remove_file(&destino);
            return Err(e);
        }
        // Con el mashup a salvo, la caché vuelve a su tope sin tocar lo usado.
        self.s.cache.podar(&protegidas);
        Ok(())
    }

    /// Asegura en la caché los stems del tramo `[desde, hasta)` (muestras) de una
    /// pista y devuelve su actividad vocal por compás, si hubo que medirla.
    ///
    /// Solo se separa el tramo que el plan previo usa, con margen: un mashup de
    /// unos minutos toma pocos compases de cada canción, y separar la canción
    /// entera era la mayor parte de la espera. Si el tramo pedido se acerca a la
    /// canción entera, se separa entera: sirve también al karaoke.
    fn separar(
        &mut self,
        pista: &Pista,
        separador: Option<&Separador>,
        tramo: (u64, u64),
        k: u32,
        n: u32,
    ) -> NsResult<Option<estructura::Actividad>> {
        let con_voz = persistencia::leer_estructura(&self.s.db, pista.id, &pista.sha)?
            .is_some_and(|(_, v)| v);
        let e = persistencia::leer_estructura(&self.s.db, pista.id, &pista.sha)?
            .ok_or(NsError::Internal)?
            .0;
        if self.s.cache.cubre(&pista.sha, tramo.0, tramo.1) {
            if con_voz {
                return Ok(None);
            }
            // Stems en caché pero estructura sin voz: se mide del FLAC.
            let ruta = self
                .s
                .cache
                .ruta(&pista.sha, karaoke::Stem::Voz)
                .ok_or(NsError::Internal)?;
            let desde = self.s.cache.tramo(&pista.sha).map_or(0, |t| t.desde);
            let (a, b) = leer_todo(&ruta)?;
            let mono: Vec<f32> = a.iter().zip(&b).map(|(x, y)| 0.5 * (x + y)).collect();
            return Ok(Some(actividad_desde(&mono, desde, &e.compases)));
        }
        let separador = separador.ok_or(NsError::Internal)?;
        let (izq, der) = karaoke::cargar(&pista.ruta)?;
        let largo = izq.len() as u64;
        // Lo que ya había de esta pista se conserva: el tramo nuevo es la
        // envolvente del viejo y del pedido.
        let (mut a, mut b) = (tramo.0.min(largo), tramo.1.min(largo));
        if let Some(t) = self.s.cache.tramo(&pista.sha) {
            a = a.min(t.desde);
            b = b.max(t.hasta.unwrap_or(largo).min(largo));
        }
        let entera = largo == 0 || (b - a) as f64 >= 0.85 * largo as f64;
        if entera {
            (a, b) = (0, largo);
        }
        let (ia, ib) = (a as usize, b as usize);
        let stems = {
            let _turno = karaoke::turno();
            self.seguir()?;
            let (yo, id, progreso, cancelar) = (self.s, self.id, self.progreso, self.cancelar);
            let mut ultimo = self.ultimo;
            let r = separador(&izq[ia..ib], &der[ia..ib], &mut |f| {
                // La separación es el 90 % del trabajo de cada pista.
                let dentro = (f64::from(k) + 0.9 * f) / f64::from(n);
                emitir(
                    yo,
                    id,
                    progreso,
                    &mut ultimo,
                    Etapa::Separando,
                    k,
                    n,
                    dentro,
                );
                !cancelar.load(Ordering::Relaxed)
            });
            self.ultimo = ultimo;
            r?
        };
        self.informar(Etapa::Separando, k, n, (f64::from(k) + 0.95) / f64::from(n));
        let voz = stems[karaoke::Stem::Voz.indice()]
            .as_ref()
            .ok_or(NsError::Internal)?;
        if entera {
            // La cola del karaoke esperaba esta pista: se completa sin volver a
            // pasar el modelo.
            let _ = karaoke::completar_desde_stems(
                &self.s.db,
                &self.s.rutas,
                pista.id,
                Some(&pista.sha),
                (&izq, &der),
                (&voz.0, &voz.1),
            );
        }
        let mono: Vec<f32> = voz
            .0
            .iter()
            .zip(&voz.1)
            .map(|(x, y)| 0.5 * (x + y))
            .collect();
        let actividad = actividad_desde(&mono, a, &e.compases);
        drop(mono);
        if entera {
            self.s.cache.guardar(&pista.sha, &stems)?;
        } else {
            self.s.cache.guardar_tramo(&pista.sha, &stems, a)?;
        }
        debug_assert_eq!(STEMS.len(), 4);
        Ok(Some(actividad))
    }
}

/// La actividad vocal por compás de una voz que empieza en la muestra `desde`
/// de la canción. Los compases fuera del tramo quedan a cero: sin stems ahí, el
/// compositor no puede usarlos como voz.
fn actividad_desde(mono: &[f32], desde: u64, compases: &[f64]) -> estructura::Actividad {
    let d = desde as f64 / f64::from(TASA);
    let locales: Vec<f64> = compases.iter().map(|c| c - d).collect();
    let mut v = estructura::actividad_vocal(mono, TASA, &locales);
    let fin = mono.len() as f64 / f64::from(TASA);
    for (k, x) in v.compases.iter_mut().enumerate() {
        if locales[k] < 0.0 || locales[k + 1] > fin + 1e-6 {
            *x = 0.0;
            for q in v.cuartos.iter_mut().skip(4 * k).take(4) {
                *q = 0.0;
            }
        }
    }
    v
}

/// Los compases enteros que cubre una entrada de la caché.
fn compases_cubiertos(t: Option<Tramo>, compases: &[f64]) -> Option<(usize, usize)> {
    let t = t?;
    let nb = compases.len().saturating_sub(1);
    let Some(hasta) = t.hasta else {
        return Some((0, nb));
    };
    let (d, h) = (
        t.desde as f64 / f64::from(TASA),
        hasta as f64 / f64::from(TASA),
    );
    let a = compases.iter().position(|c| *c >= d - 1e-6)?;
    let b = compases.iter().rposition(|c| *c <= h + 1e-6)?;
    (b > a).then_some((a, b.min(nb)))
}

/// Los compases `[a, b)` de una pista que el plan previo usa, con una frase
/// de margen a cada lado (una frase de 8; con media, el plan definitivo tuvo que prolongar frases
/// y solo ahorraba un 8 %) para que el plan definitivo tenga
/// dónde elegir, y la cola de los fundidos.
fn compases_necesarios(plan: &Plan, pista: i64, n_compases: usize) -> Option<(usize, usize)> {
    const MARGEN: usize = 8;
    // Solo las capas que suenan como stems: una mezcla original no se separa.
    let capas = plan
        .capas
        .iter()
        .filter(|c| c.pista_id == pista && c.material != Material::Mezcla);
    let (mut a, mut b) = (usize::MAX, 0usize);
    for c in capas {
        a = a.min(c.origen as usize);
        b = b.max((c.origen + c.compases) as usize);
    }
    (a <= b).then(|| (a.saturating_sub(MARGEN), (b + MARGEN + 2).min(n_compases)))
}

pub(crate) fn leer_todo(ruta: &Path) -> NsResult<(Vec<f32>, Vec<f32>)> {
    let mut d = Decodificador::abrir(ruta, 0.0)?;
    let mut frames = Vec::new();
    while d.leer(&mut frames, 65_536)? > 0 {}
    Ok((
        frames.iter().map(|f| f.left).collect(),
        frames.iter().map(|f| f.right).collect(),
    ))
}

/// Lee fragmentos de las pistas originales (en solo lectura) y de la caché.
pub struct OrigenReal {
    pistas: HashMap<i64, (PathBuf, String)>,
    cache: CacheStems,
}

fn fragmento(ruta: &Path, desde: f64, segundos: f64) -> NsResult<(Vec<f32>, Vec<f32>)> {
    let mut d = Decodificador::abrir(ruta, 0.0)?;
    let tasa = d.tasa;
    d.buscar_frame((desde.max(0.0) * f64::from(tasa)).round() as u64)?;
    let n = (segundos * f64::from(tasa)).round() as usize;
    let mut frames = Vec::with_capacity(n);
    while frames.len() < n {
        let falta = n - frames.len();
        if d.leer(&mut frames, falta.min(65_536))? == 0 {
            break;
        }
    }
    let a: Vec<f32> = frames.iter().map(|f| f.left).collect();
    let b: Vec<f32> = frames.iter().map(|f| f.right).collect();
    if tasa == TASA || a.is_empty() {
        Ok((a, b))
    } else {
        karaoke::remuestrear(&a, &b, tasa)
    }
}

impl Origen for OrigenReal {
    fn leer(
        &mut self,
        pista: i64,
        material: Material,
        desde: f64,
        segundos: f64,
    ) -> NsResult<(Vec<f32>, Vec<f32>)> {
        let (ruta, sha) = self.pistas.get(&pista).ok_or(NsError::FileNotFound)?;
        if material == Material::Mezcla {
            return fragmento(ruta, desde, segundos);
        }
        // Un tramo empieza a mitad de la canción: se lee desde su inicio.
        let inicio = self
            .cache
            .tramo(sha)
            .ok_or(NsError::ComponentUnavailable)?
            .desde as f64
            / f64::from(TASA);
        if desde + 1e-3 < inicio {
            return Err(NsError::ComponentUnavailable);
        }
        let mut suma: Option<(Vec<f32>, Vec<f32>)> = None;
        for s in material.stems() {
            let r = self
                .cache
                .ruta(sha, *s)
                .ok_or(NsError::ComponentUnavailable)?;
            let (a, b) = fragmento(&r, (desde - inicio).max(0.0), segundos)?;
            suma = Some(match suma {
                None => (a, b),
                Some((mut x, mut y)) => {
                    let n = x.len().max(a.len());
                    x.resize(n, 0.0);
                    y.resize(n, 0.0);
                    for (i, v) in a.iter().enumerate() {
                        x[i] += v;
                    }
                    for (i, v) in b.iter().enumerate() {
                        y[i] += v;
                    }
                    (x, y)
                }
            });
        }
        suma.ok_or(NsError::Internal)
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use std::f32::consts::TAU;
    use std::sync::atomic::AtomicU32;
    use std::time::{Duration, Instant};

    /// Una canción sintética a 44,1 kHz: batería en cada pulso con acento en el
    /// compás, un acorde de su tonalidad, un bajo y una "voz" (un tono agudo que
    /// canta en los tramos de verso y estribillo). Intro de 4 compases sin voz.
    fn cancion(bpm: f64, tonica: f32, compases: usize, semilla: u32) -> (Vec<f32>, Vec<f32>) {
        let sr = 44_100.0f64;
        let periodo = 60.0 / bpm;
        let n = (compases as f64 * 4.0 * periodo * sr) as usize;
        let f0 = 110.0 * 2f32.powf(tonica / 12.0);
        let mut x = vec![0.0f32; n];
        for (i, v) in x.iter_mut().enumerate() {
            let t = i as f64 / sr;
            let pulso = (t / periodo).floor() as usize;
            let fase = (t / periodo).fract() as f32;
            let compas = pulso / 4;
            let golpe = (-fase * 30.0).exp() * if pulso.is_multiple_of(4) { 0.5 } else { 0.25 };
            let bombo = golpe * (TAU * 55.0 * t as f32).sin();
            let caja =
                golpe * 0.3 * (TAU * 1800.0 * t as f32 * (1.0 + 0.001 * semilla as f32)).sin();
            let acorde = [1.0, 1.26, 1.5]
                .iter()
                .map(|r| (TAU * f0 * 2.0 * r * t as f32).sin())
                .sum::<f32>()
                * 0.05;
            let bajo = (TAU * f0 * 0.5 * t as f32).sin() * 0.12;
            let canta = compas >= 4 && (compas / 4) % 2 == 1;
            let voz = if canta {
                (TAU * f0 * 4.0 * t as f32).sin() * 0.1 * (0.6 + 0.4 * (TAU * 5.0 * t as f32).sin())
            } else {
                0.0
            };
            *v = bombo + caja + acorde + bajo + voz;
        }
        (x.clone(), x)
    }

    struct Escenario {
        _dir: PathBuf,
        rutas: Rutas,
        db: BaseDeDatos,
        archivos: Vec<PathBuf>,
    }

    fn escenario(nombre: &str, canciones: usize) -> Escenario {
        let dir = std::env::temp_dir().join(format!("ns-mashup-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let rutas = Rutas::bajo(&dir).unwrap();
        let db = BaseDeDatos::abrir(&rutas.base_de_datos()).unwrap();
        let musica = dir.join("musica");
        std::fs::create_dir_all(&musica).unwrap();
        let notas = ["C", "G", "F", "A", "D", "E"];
        let mut archivos = Vec::new();
        for i in 0..canciones {
            let bpm = 118.0 + (i % 5) as f64;
            let ton = [0.0, 7.0, 5.0, 9.0, 2.0, 4.0][i % 6];
            let (a, b) = cancion(bpm, ton, 24, i as u32);
            let ruta = musica.join(format!("c{i}.wav"));
            karaoke::escribir_wav(&ruta, &a, &b).unwrap();
            let conn = db.conexion().unwrap();
            conn.execute(
                "INSERT INTO pistas (id, titulo, artista_nombre, album_titulo, ruta, duracion_seg)
                 VALUES (?1, ?2, ?3, 'Disco', ?4, ?5)",
                params![
                    i as i64 + 1,
                    format!("Canción {i}"),
                    format!("Artista {}", i % 3),
                    ruta.to_string_lossy(),
                    a.len() as f64 / 44_100.0
                ],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO rasgos (pista_id, version, bpm, tonalidad, modo, energia) VALUES (?1, 'v', ?2, ?3, 'major', 0.6)",
                params![i as i64 + 1, bpm, notas[i % 6]],
            )
            .unwrap();
            archivos.push(ruta);
        }
        Escenario {
            _dir: dir,
            rutas,
            db,
            archivos,
        }
    }

    /// La inferencia controlada: reparte la mezcla en cuatro partes fijas. No
    /// separa de verdad (eso lo prueba `karaoke` con el modelo real), pero
    /// recorre el mismo `separar_stems` con su forma, normalización y ventanas.
    fn separador(llamadas: Arc<AtomicU32>) -> impl Fn() -> Option<Separador> + Send + Sync {
        move || {
            let llamadas = Arc::clone(&llamadas);
            let s: Separador = Arc::new(move |izq, der, seguir| {
                llamadas.fetch_add(1, Ordering::Relaxed);
                karaoke::separar_stems(
                    izq,
                    der,
                    [true; 4],
                    &mut |t: &[f32]| {
                        let mut v = Vec::with_capacity(t.len() * 4);
                        for k in 0..4 {
                            v.extend(t.iter().map(|x| x * [0.3, 0.3, 0.2, 0.2][k]));
                        }
                        Ok(v)
                    },
                    seguir,
                )
            });
            Some(s)
        }
    }

    fn esperar(avisos: &Arc<Mutex<Vec<Aviso>>>, id: i64, limite: Duration) {
        let t = Instant::now();
        while t.elapsed() < limite {
            if avisos
                .lock()
                .unwrap()
                .contains(&Aviso::Terminado { mashup_id: id })
            {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!("la generación no terminó");
    }

    fn sha(r: &Path) -> String {
        sha_de(r).unwrap()
    }

    fn peticion(semilla: i64) -> PeticionMashup {
        PeticionMashup {
            duracion_seg: 60,
            canciones: 3,
            estilo: compositor::Estilo::Equilibrado,
            semilla: Some(semilla),
            pistas: vec![],
            playlist_id: None,
        }
    }

    #[test]
    fn genera_un_mashup_real_con_stems_lo_guarda_y_lo_repite_desde_la_cache() {
        let e = escenario("integral", 5);
        let antes: Vec<String> = e.archivos.iter().map(|r| sha(r)).collect();
        let avisos = Arc::new(Mutex::new(Vec::new()));
        let llamadas = Arc::new(AtomicU32::new(0));
        let a = Arc::clone(&avisos);
        let m = Mashups::con_separador(
            e.db.clone(),
            e.rutas.clone(),
            move |x| a.lock().unwrap().push(x),
            separador(Arc::clone(&llamadas)),
        )
        .unwrap();
        assert!(m.con_stems());
        let id = m.generar(peticion(11)).unwrap();
        // Una segunda petición mientras la primera trabaja se rechaza.
        assert_eq!(m.generar(peticion(12)), Err(NsError::InvalidInput));
        esperar(&avisos, id, Duration::from_secs(900));
        let v = persistencia::vista(&e.db, id).unwrap();
        assert_eq!(v.estado, "lista", "{:?}", v.error);
        let plan = v.plan.clone().unwrap();
        // El separador de prueba devuelve como "voz" una fracción de la
        // mezcla, que canta sin pausas: el compositor puede no superponerla
        // (cortaría frases), pero los stems tienen que llegar al plan.
        assert!(
            plan.capas.iter().any(|c| !c.material.stems().is_empty()),
            "{:?}",
            plan.capas
        );
        assert!(v.pistas.len() >= 2);
        let met = v.metricas.clone().unwrap();
        assert!((met.lufs + 14.0).abs() <= 1.0, "{}", met.lufs);
        assert_eq!(met.recortes, 0);
        assert!(met.silencio_max_seg <= 0.05, "{}", met.silencio_max_seg);
        let ruta = m.ruta(id).unwrap();
        let mut d = Decodificador::abrir(&ruta, 0.0).unwrap();
        let mut frames = Vec::new();
        while d.leer(&mut frames, 65_536).unwrap() > 0 {}
        assert!((frames.len() as f64 / 44_100.0 - render::duracion_seg(&plan)).abs() < 0.01);
        // El progreso pasó por las etapas en orden y nunca retrocedió.
        let progreso: Vec<Progreso> = avisos
            .lock()
            .unwrap()
            .iter()
            .filter_map(|a| match a {
                Aviso::Progreso(p) => Some(p.clone()),
                Aviso::Terminado { .. } => None,
            })
            .collect();
        assert!(progreso
            .windows(2)
            .all(|w| w[1].fraccion >= w[0].fraccion - 1e-9));
        for etapa in [Etapa::Analizando, Etapa::Separando, Etapa::Mezclando] {
            assert!(progreso.iter().any(|p| p.etapa == etapa), "{etapa:?}");
        }
        let separaciones = llamadas.load(Ordering::Relaxed);
        assert!(separaciones >= 2);
        assert!(m.progreso().is_none());

        // La misma petición otra vez: nada se vuelve a separar ni a analizar, y
        // el audio sale idéntico.
        let id2 = m.generar(peticion(11)).unwrap();
        esperar(&avisos, id2, Duration::from_secs(600));
        assert_eq!(llamadas.load(Ordering::Relaxed), separaciones);
        assert_eq!(
            std::fs::read(&ruta).unwrap(),
            std::fs::read(m.ruta(id2).unwrap()).unwrap()
        );

        // Una variante es otra semilla y se guarda aparte.
        let id3 = m.variante(id).unwrap();
        esperar(&avisos, id3, Duration::from_secs(600));
        let v3 = persistencia::vista(&e.db, id3).unwrap();
        assert_eq!(v3.estado, "lista");
        assert_ne!(v3.semilla, v.semilla);

        // Exportar copia el archivo sin tocar el original del mashup.
        let fuera = e.rutas.datos.join("exportado.wav");
        let bytes = m.exportar(id, &fuera).unwrap();
        assert_eq!(bytes, std::fs::metadata(&ruta).unwrap().len());

        // Ningún archivo de música cambió.
        let despues: Vec<String> = e.archivos.iter().map(|r| sha(r)).collect();
        assert_eq!(antes, despues);

        // Borrar quita el audio.
        m.eliminar(id).unwrap();
        assert!(!ruta.exists());
        assert_eq!(m.ruta(id), Err(NsError::FileNotFound));
    }

    #[test]
    fn cancelar_deja_el_mashup_cancelado_sin_audio_ni_temporales() {
        let e = escenario("cancelar", 4);
        let avisos = Arc::new(Mutex::new(Vec::new()));
        let a = Arc::clone(&avisos);
        let m = Mashups::con_separador(
            e.db.clone(),
            e.rutas.clone(),
            move |x| a.lock().unwrap().push(x),
            separador(Arc::new(AtomicU32::new(0))),
        )
        .unwrap();
        let id = m.generar(peticion(3)).unwrap();
        m.cancelar(id);
        esperar(&avisos, id, Duration::from_secs(600));
        assert_eq!(persistencia::vista(&e.db, id).unwrap().estado, "cancelada");
        let dir = carpeta(&e.rutas);
        let restos = std::fs::read_dir(&dir).map(|r| r.count()).unwrap_or(0);
        assert_eq!(restos, 0);
        // Queda libre para otra.
        assert!(m.progreso().is_none());
        let otra = m.generar(peticion(4)).unwrap();
        m.cancelar(otra);
        esperar(&avisos, otra, Duration::from_secs(600));
    }

    #[test]
    fn sin_separador_se_compone_con_mezclas_y_se_dice() {
        let e = escenario("sinstems", 4);
        let avisos = Arc::new(Mutex::new(Vec::new()));
        let a = Arc::clone(&avisos);
        let m = Mashups::con_separador(
            e.db.clone(),
            e.rutas.clone(),
            move |x| a.lock().unwrap().push(x),
            || None,
        )
        .unwrap();
        assert!(!m.con_stems());
        let id = m.generar(peticion(5)).unwrap();
        esperar(&avisos, id, Duration::from_secs(600));
        let v = persistencia::vista(&e.db, id).unwrap();
        assert_eq!(v.estado, "lista", "{:?}", v.error);
        let plan = v.plan.unwrap();
        assert!(plan.capas.iter().all(|c| c.material == Material::Mezcla));
        assert!(plan
            .degradaciones
            .contains(&compositor::Degradacion::SinStems));
    }

    #[test]
    fn una_generacion_cortada_por_un_cierre_se_da_por_fallida_al_abrir() {
        let e = escenario("cierre", 2);
        let id = persistencia::crear(&e.db, "M", &peticion(1), 1, 1).unwrap();
        let dir = carpeta(&e.rutas);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.f32.parcial")), b"x").unwrap();
        let _m = Mashups::con_separador(e.db.clone(), e.rutas.clone(), |_| {}, || None).unwrap();
        assert_eq!(persistencia::vista(&e.db, id).unwrap().estado, "fallida");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
    }

    #[test]
    fn las_peticiones_absurdas_se_rechazan_sin_crear_nada() {
        let e = escenario("absurdas", 2);
        let m = Mashups::con_separador(e.db.clone(), e.rutas.clone(), |_| {}, || None).unwrap();
        let mut p = peticion(1);
        p.duracion_seg = 5;
        assert_eq!(m.generar(p), Err(NsError::InvalidInput));
        let mut p = peticion(1);
        p.canciones = 1000;
        assert_eq!(m.generar(p), Err(NsError::InvalidInput));
        assert!(persistencia::listar(&e.db, "", 10).unwrap().is_empty());
    }
}
