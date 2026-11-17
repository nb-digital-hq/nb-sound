//! La estructura temporal de una pista (ADR-018): pulsos, compases,
//! secciones, energía, croma y voz por compás.
//!
//! **Qué se reutiliza.** El espectrograma, la envolvente de ataques, el tempo y
//! el seguidor de pulsos son los de [`crate::rasgos`], los mismos que miden el
//! BPM de la biblioteca: no hay un segundo detector que pueda contradecir al
//! primero. Lo nuevo es lo que los rasgos no necesitaban: la pista entera (no
//! sus primeros 90 s), la posición de cada pulso, el compás y las secciones.
//!
//! **Incertidumbre.** Cada nivel lleva su confianza. Una pista sin pulso claro
//! no recibe una rejilla inventada con aspecto de precisa: sus compases son de
//! dos segundos y su confianza es cero, y el compositor no la alinea por pulsos.
//!
//! **Doble y mitad.** El tempo se pliega a [78, 156) BPM antes de seguir los
//! pulsos. Un mashup no distingue 70 de 140: lo que importa es que dos
//! fragmentos coincidan en la rejilla, y el pliegue hace comparables las dos
//! lecturas de la misma canción.

use std::path::Path;

use rustfft::FftPlanner;
use serde::{Deserialize, Serialize};

use crate::error::{NsError, NsResult};
use crate::rasgos::{self, HOP, N_FFT, SR};

/// Sube cuando cambia el algoritmo: la fila guardada caduca. La 2 añade la
/// actividad vocal por cuarto de compás y el patrón de ataques por compás.
pub const VERSION: u32 = 2;
/// Pasos por compás del patrón rítmico: semicorcheas en 4/4.
pub const PASOS: usize = 16;
/// Pulsos por compás. Se asume 4/4: es el compás de casi toda la música que se
/// mezcla, y un 3/4 detectado como 4/4 solo empeora la confianza del compás.
pub const PULSOS_POR_COMPAS: usize = 4;
const BPM_MIN: f64 = 78.0;
const BPM_MAX: f64 = 156.0;
/// Compases de cada mitad del núcleo de novedad.
const NUCLEO: usize = 4;
/// Compases de reserva cuando no hay pulso.
const COMPAS_SIN_PULSO: f64 = 2.0;

/// El papel de una sección. Es una estimación: va con su confianza.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TipoSeccion {
    Intro,
    Verso,
    Coro,
    Puente,
    Final,
}

/// Un tramo de compases `[desde, hasta)` homogéneo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Seccion {
    pub desde: usize,
    pub hasta: usize,
    pub tipo: TipoSeccion,
    /// Las secciones que se parecen comparten grupo: un coro y su repetición.
    pub grupo: u32,
    pub confianza: f64,
    pub energia: f32,
}

/// La estructura de una pista, en segundos y en compases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Estructura {
    pub version: u32,
    pub duracion: f64,
    /// Plegado a [78, 156).
    pub bpm: Option<f64>,
    /// De 0 a 1: regularidad de los pulsos y prominencia de sus ataques.
    pub confianza_pulso: f64,
    pub pulsos: Vec<f64>,
    /// El inicio de cada compás; el último es el final del último compás entero.
    pub compases: Vec<f64>,
    pub confianza_compas: f64,
    /// RMS por compás, relativa al compás más fuerte de la pista.
    pub energia: Vec<f32>,
    /// Croma por compás, de norma unitaria.
    pub croma: Vec<[f32; 12]>,
    /// Actividad vocal por compás (0 a 1), si se conoce el stem de voz: la
    /// fracción del compás en la que se canta.
    pub voz: Option<Vec<f32>>,
    /// La misma actividad por cuarto de compás (cuatro por compás): dice si una
    /// frase cantada cruza la frontera de un compás, que es lo que corta una
    /// palabra al cambiar de fragmento.
    #[serde(default)]
    pub voz_cuartos: Option<Vec<f32>>,
    /// Patrón de ataques por compás en [`PASOS`] pasos (0 a 255, relativo al
    /// ataque típico de la pista). De él salen el groove y la organización en
    /// dos compases (la clave en la salsa).
    #[serde(default)]
    pub patron: Vec<[u8; PASOS]>,
    pub secciones: Vec<Seccion>,
}

impl Estructura {
    /// Número de compases enteros.
    #[must_use]
    pub fn n_compases(&self) -> usize {
        self.compases.len().saturating_sub(1)
    }

    /// Si la rejilla es fiable para alinear fragmentos por pulsos.
    #[must_use]
    pub fn rejilla_fiable(&self) -> bool {
        self.bpm.is_some() && self.confianza_pulso >= 0.35
    }
}

/// Lleva un tempo al intervalo de trabajo de los mashups.
#[must_use]
pub fn plegar(mut bpm: f64) -> f64 {
    if bpm <= 0.0 || !bpm.is_finite() {
        return bpm;
    }
    while bpm < BPM_MIN {
        bpm *= 2.0;
    }
    while bpm >= BPM_MAX {
        bpm /= 2.0;
    }
    bpm
}

fn marco_a_seg(m: f64) -> f64 {
    m * HOP as f64 / SR
}

/// Lo que se saca del espectrograma marco a marco, para soltarlo pronto: el
/// espectrograma de una pista de 4 minutos ocupa unos 85 MB.
struct PorMarco {
    ataques: Vec<f64>,
    graves: Vec<f64>,
    rms: Vec<f64>,
    croma: Vec<[f64; 12]>,
}

fn por_marco(y: &[f32]) -> PorMarco {
    let mut plan = FftPlanner::<f64>::new();
    let fft = plan.plan_fft_forward(N_FFT);
    let pot = rasgos::espectrograma(y, &fft);
    let (ataques, _) = rasgos::envolventes(&pot, &rasgos::banco_mel());
    let clases = rasgos::clases_croma();
    // Graves: hasta 150 Hz, donde están el bombo y el bajo que marcan el compás.
    let tope_graves = (150.0 * N_FFT as f64 / SR) as usize;
    let mut graves = Vec::with_capacity(pot.len());
    let mut rms = Vec::with_capacity(pot.len());
    let mut croma = Vec::with_capacity(pot.len());
    for p in &pot {
        graves.push(p[1..=tope_graves].iter().sum::<f64>().sqrt());
        rms.push((p.iter().sum::<f64>() / p.len() as f64).sqrt());
        let mut c = [0.0f64; 12];
        for (v, k) in p.iter().zip(&clases) {
            if let Some(k) = k {
                c[*k] += v;
            }
        }
        croma.push(c);
    }
    PorMarco {
        ataques,
        graves,
        rms,
        croma,
    }
}

/// El máximo de `env` cerca de `m`, con interpolación parabólica: la rejilla
/// de marcos es de 23 ms y los pulsos se quieren más finos para alinear.
fn afinar(env: &[f64], m: usize) -> f64 {
    let desde = m.saturating_sub(2);
    let hasta = (m + 2).min(env.len().saturating_sub(1));
    let mut k = m.min(env.len().saturating_sub(1));
    for j in desde..=hasta {
        if env[j] > env[k] {
            k = j;
        }
    }
    if k == 0 || k + 1 >= env.len() {
        return k as f64;
    }
    let (a, b, c) = (env[k - 1], env[k], env[k + 1]);
    let d = a - 2.0 * b + c;
    if d.abs() < 1e-12 {
        k as f64
    } else {
        k as f64 + (0.5 * (a - c) / d).clamp(-0.5, 0.5)
    }
}

/// Retardo del máximo de la envolvente de `onset_strength` respecto del ataque,
/// en marcos. La envolvente es la subida entre un marco y el anterior, con el
/// desplazamiento de librosa: su máximo cae un marco (23 ms) después del golpe.
/// Medido con clics sintéticos a 96–140 BPM: 25 ms de media sin corregir, menos
/// de 7 ms con la corrección.
const RETARDO_ENVOLVENTE: f64 = 1.0;

/// Lo periódica que es la envolvente al periodo del pulso: la autocorrelación
/// normalizada en ese retardo. El seguidor de pulsos impone su periodo aunque
/// no lo haya (es programación dinámica), así que la regularidad de sus pulsos
/// no dice nada; la envolvente, sí. Clics: más de 0,6. Ruido: menos de 0,15.
fn confianza_pulso(env: &[f64], marcos: &[usize], bpm: f64) -> f64 {
    if marcos.len() < 8 || env.len() < 64 {
        return 0.0;
    }
    let media = env.iter().sum::<f64>() / env.len() as f64;
    let c: Vec<f64> = env.iter().map(|v| v - media).collect();
    let cero: f64 = c.iter().map(|v| v * v).sum();
    if cero <= 1e-12 {
        return 0.0;
    }
    let periodo = SR / HOP as f64 * 60.0 / bpm;
    let base = periodo.round() as usize;
    let r = (base.saturating_sub(1)..=base + 1)
        .filter(|l| *l > 0 && *l < c.len())
        .map(|l| c.iter().zip(&c[l..]).map(|(a, b)| a * b).sum::<f64>() / cero)
        .fold(f64::MIN, f64::max);
    ((r - 0.15) / 0.35).clamp(0.0, 1.0)
}

fn norma(c: &[f64; 12]) -> [f32; 12] {
    let n = c.iter().map(|v| v * v).sum::<f64>().sqrt();
    let mut r = [0.0f32; 12];
    if n > 1e-12 {
        for (a, b) in r.iter_mut().zip(c) {
            *a = (b / n) as f32;
        }
    }
    r
}

fn coseno(a: &[f32; 12], b: &[f32; 12]) -> f64 {
    a.iter().zip(b).map(|(x, y)| f64::from(x * y)).sum()
}

/// Analiza una señal mono a 22.050 Hz.
///
/// # Errors
///
/// [`NsError::AudioCorrupted`] si es silencio o más corta que una ventana.
pub fn de_senal(y: &[f32]) -> NsResult<Estructura> {
    if y.len() < N_FFT || !y.iter().any(|v| v.abs() > 1e-6) {
        return Err(NsError::AudioCorrupted);
    }
    let duracion = y.len() as f64 / SR;
    let pm = por_marco(y);
    let n = pm.ataques.len();

    let hay_ataques = pm.ataques.iter().any(|v| *v > 0.0);
    let bpm = hay_ataques
        .then(|| rasgos::tempo(&pm.ataques))
        .filter(|b| *b > 0.0)
        .map(plegar);
    let marcos = bpm.map_or_else(Vec::new, |b| rasgos::pulsos(&pm.ataques, b));
    let confianza_pulso = bpm.map_or(0.0, |b| confianza_pulso(&pm.ataques, &marcos, b));
    let pulsos: Vec<f64> = marcos
        .iter()
        .map(|&m| marco_a_seg(afinar(&pm.ataques, m) - RETARDO_ENVOLVENTE).max(0.0))
        .collect();
    // El tempo de `rasgos` sale de un retardo entero de la autocorrelación: a
    // 120 BPM los vecinos son 117,5 y 123,0. Para alinear fragmentos hace falta
    // más: se ajusta una recta a los pulsos afinados y su pendiente es el periodo.
    let bpm = if confianza_pulso > 0.0 && pulsos.len() >= 8 {
        let n = pulsos.len() as f64;
        let mx = (n - 1.0) / 2.0;
        let my = pulsos.iter().sum::<f64>() / n;
        let (mut sxy, mut sxx) = (0.0, 0.0);
        for (i, p) in pulsos.iter().enumerate() {
            sxy += (i as f64 - mx) * (p - my);
            sxx += (i as f64 - mx).powi(2);
        }
        let periodo = sxy / sxx;
        if periodo > 0.0 {
            bpm.map(|b| {
                let fino = 60.0 / periodo;
                // Solo se acepta si confirma el tempo grueso: un ajuste roto por
                // un salto de fase no debe mover la rejilla a otro tempo.
                if (fino / b - 1.0).abs() < 0.04 {
                    fino
                } else {
                    b
                }
            })
        } else {
            bpm
        }
    } else {
        bpm
    };

    // El compás: la fase de cuatro con más golpe grave y más cambio armónico.
    let (compases, confianza_compas) = if confianza_pulso > 0.0 && marcos.len() >= 8 {
        let subida = |m: usize| -> f64 {
            let a = pm.graves[m.min(n - 1)];
            let b = pm.graves[m.saturating_sub(2)];
            (a - b).max(0.0) + 0.25 * a
        };
        let cambio = |m: usize| -> f64 {
            let a = norma(&pm.croma[m.min(n - 1)]);
            let b = norma(&pm.croma[m.saturating_sub(4)]);
            1.0 - coseno(&a, &b)
        };
        let mut puntos = [0.0f64; PULSOS_POR_COMPAS];
        let mut cuenta = [0usize; PULSOS_POR_COMPAS];
        for (i, &m) in marcos.iter().enumerate() {
            puntos[i % PULSOS_POR_COMPAS] += subida(m) + cambio(m) * 0.5;
            cuenta[i % PULSOS_POR_COMPAS] += 1;
        }
        for (p, c) in puntos.iter_mut().zip(cuenta) {
            *p /= c.max(1) as f64;
        }
        let mut orden: Vec<usize> = (0..PULSOS_POR_COMPAS).collect();
        orden.sort_by(|a, b| puntos[*b].total_cmp(&puntos[*a]));
        let fase = orden[0];
        let conf =
            ((puntos[orden[0]] - puntos[orden[1]]) / puntos[orden[0]].max(1e-12)).clamp(0.0, 1.0);
        let inicios: Vec<f64> = pulsos
            .iter()
            .skip(fase)
            .step_by(PULSOS_POR_COMPAS)
            .copied()
            .collect();
        (inicios, conf * confianza_pulso.sqrt())
    } else {
        let k = (duracion / COMPAS_SIN_PULSO).floor() as usize;
        ((0..=k).map(|i| i as f64 * COMPAS_SIN_PULSO).collect(), 0.0)
    };

    // Rasgos por compás.
    let nb = compases.len().saturating_sub(1);
    let marco = |s: f64| ((s * SR / HOP as f64) as usize).min(n.saturating_sub(1));
    let mut energia = Vec::with_capacity(nb);
    let mut croma = Vec::with_capacity(nb);
    for k in 0..nb {
        let (a, b) = (
            marco(compases[k]),
            marco(compases[k + 1]).max(marco(compases[k]) + 1),
        );
        let tramo = a..b.min(n);
        let len = tramo.len().max(1) as f64;
        energia.push(pm.rms[tramo.clone()].iter().sum::<f64>() / len);
        let mut c = [0.0f64; 12];
        for m in tramo {
            for (x, y) in c.iter_mut().zip(&pm.croma[m]) {
                *x += y;
            }
        }
        croma.push(norma(&c));
    }
    let tope = energia.iter().fold(0.0f64, |m, v| m.max(*v)).max(1e-12);
    let energia: Vec<f32> = energia.iter().map(|v| (v / tope) as f32).collect();
    let secciones = secciones(&energia, &croma);
    let patron = if confianza_pulso > 0.0 {
        patron_ritmico(&pm.ataques, &compases)
    } else {
        Vec::new()
    };

    Ok(Estructura {
        version: VERSION,
        duracion,
        bpm,
        confianza_pulso,
        pulsos,
        compases,
        confianza_compas,
        energia,
        croma,
        voz: None,
        voz_cuartos: None,
        patron,
        secciones,
    })
}

/// El patrón de ataques de cada compás en [`PASOS`] pasos: la envolvente de
/// ataques muestreada en cada semicorchea de la rejilla (el máximo a ±1 marco,
/// porque un golpe real no cae exacto en la rejilla). Se normaliza con el
/// percentil 95 de la pista y se cuantiza a un byte: a 16 bytes por compás una
/// canción de 4 minutos ocupa unos 2 KB.
///
/// Es la representación habitual de los trabajos de groove y de clave
/// (patrones de inicio por subdivisión métrica); lo que no se hace aquí es
/// clasificar el patrón: el compositor compara patrones entre pistas.
fn patron_ritmico(ataques: &[f64], compases: &[f64]) -> Vec<[u8; PASOS]> {
    let n = ataques.len();
    if n == 0 || compases.len() < 2 {
        return Vec::new();
    }
    let valor = |s: f64| -> f64 {
        let m = (s * SR / HOP as f64 + RETARDO_ENVOLVENTE).round() as isize;
        (m - 1..=m + 1)
            .filter_map(|k| usize::try_from(k).ok().and_then(|k| ataques.get(k)))
            .fold(0.0f64, |a, b| a.max(*b))
    };
    let crudo: Vec<[f64; PASOS]> = compases
        .windows(2)
        .map(|w| {
            let mut p = [0.0f64; PASOS];
            for (j, v) in p.iter_mut().enumerate() {
                *v = valor(w[0] + (w[1] - w[0]) * j as f64 / PASOS as f64);
            }
            p
        })
        .collect();
    let mut todos: Vec<f64> = crudo.iter().flatten().copied().collect();
    todos.sort_by(f64::total_cmp);
    let p95 = todos
        .get(todos.len() * 95 / 100)
        .copied()
        .unwrap_or(0.0)
        .max(1e-9);
    crudo
        .iter()
        .map(|p| {
            let mut q = [0u8; PASOS];
            for (a, b) in q.iter_mut().zip(p) {
                *a = ((b / p95).clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            q
        })
        .collect()
}

/// Fronteras por novedad sobre la autosimilitud (Foote, 2000), con croma y
/// energía por compás, y etiquetas por repetición y energía.
fn secciones(energia: &[f32], croma: &[[f32; 12]]) -> Vec<Seccion> {
    let nb = energia.len();
    if nb == 0 {
        return Vec::new();
    }
    let sim = |i: usize, j: usize| -> f64 {
        let e = 1.0 - f64::from((energia[i] - energia[j]).abs());
        0.7 * coseno(&croma[i], &croma[j]) + 0.3 * e
    };
    let mut novedad = vec![0.0f64; nb];
    for (t, nov) in novedad.iter_mut().enumerate().skip(1) {
        let mut s = 0.0;
        let mut cuenta = 0.0;
        for a in 1..=NUCLEO {
            for b in 1..=NUCLEO {
                let (i0, i1) = (t.checked_sub(a), t + b - 1);
                if let Some(i0) = i0 {
                    if i1 < nb {
                        let j0 = t.checked_sub(b);
                        let j1 = t + a - 1;
                        // Cuadrantes del tablero: mismo lado suma, cruzado resta.
                        if let Some(j0) = j0 {
                            s += sim(i0, j0);
                            cuenta += 1.0;
                        }
                        if j1 < nb {
                            s += sim(i1, j1);
                            cuenta += 1.0;
                        }
                        s -= 2.0 * sim(i0, i1);
                        cuenta += 2.0;
                    }
                }
            }
        }
        *nov = if cuenta > 0.0 {
            (s / cuenta).max(0.0)
        } else {
            0.0
        };
    }
    let media = novedad.iter().sum::<f64>() / nb as f64;
    let desv = (novedad.iter().map(|v| (v - media).powi(2)).sum::<f64>() / nb as f64).sqrt();
    let umbral = media + 0.5 * desv;
    let mut cortes: Vec<(usize, f64)> = Vec::new();
    for t in 2..nb.saturating_sub(1) {
        let es_max = novedad[t] >= novedad[t - 1] && novedad[t] > novedad[t + 1];
        if es_max && novedad[t] > umbral && novedad[t] > 1e-3 {
            // Las frases suelen ser de 4 compases: se atrae la frontera a la
            // frase si está a un compás.
            let t = if t % 4 == 1 {
                t - 1
            } else if t % 4 == 3 {
                t + 1
            } else {
                t
            };
            if cortes.last().is_none_or(|(u, _)| t >= u + 4) && t < nb {
                cortes.push((t, novedad[t.min(nb - 1)]));
            }
        }
    }
    let max_nov = novedad.iter().fold(0.0f64, |m, v| m.max(*v)).max(1e-12);
    let mut bordes = vec![(0usize, 1.0f64)];
    bordes.extend(
        cortes
            .iter()
            .map(|(t, v)| (*t, (v / max_nov).clamp(0.0, 1.0))),
    );
    bordes.push((nb, 1.0));

    // Media de cada tramo y agrupación por parecido.
    let mut tramos: Vec<(usize, usize, f64, [f32; 12], f32)> = Vec::new();
    for w in bordes.windows(2) {
        let (d, h) = (w[0].0, w[1].0);
        if h <= d {
            continue;
        }
        let mut c = [0.0f64; 12];
        for fila in &croma[d..h] {
            for (x, y) in c.iter_mut().zip(fila) {
                *x += f64::from(*y);
            }
        }
        let e = energia[d..h].iter().sum::<f32>() / (h - d) as f32;
        tramos.push((d, h, w[0].1.min(w[1].1).max(0.3), norma(&c), e));
    }
    let mut grupos: Vec<u32> = Vec::with_capacity(tramos.len());
    let mut representantes: Vec<usize> = Vec::new();
    for (i, t) in tramos.iter().enumerate() {
        let g = representantes
            .iter()
            .position(|&r| coseno(&tramos[r].3, &t.3) > 0.92 && (tramos[r].4 - t.4).abs() < 0.2);
        match g {
            Some(g) => grupos.push(g as u32),
            None => {
                grupos.push(representantes.len() as u32);
                representantes.push(i);
            }
        }
    }
    let veces = |g: u32| grupos.iter().filter(|x| **x == g).count();
    let mut mediana: Vec<f32> = tramos.iter().map(|t| t.4).collect();
    mediana.sort_by(f32::total_cmp);
    let mediana = mediana[mediana.len() / 2];
    // El coro: el grupo repetido con más energía.
    let coro = (0..representantes.len() as u32)
        .filter(|g| veces(*g) >= 2)
        .max_by(|a, b| {
            let e = |g: u32| {
                tramos
                    .iter()
                    .zip(&grupos)
                    .filter(|(_, x)| **x == g)
                    .map(|(t, _)| t.4)
                    .sum::<f32>()
                    / veces(g) as f32
            };
            e(*a).total_cmp(&e(*b))
        });
    let ultimo = tramos.len() - 1;
    tramos
        .iter()
        .zip(&grupos)
        .enumerate()
        .map(|(i, (t, &g))| {
            let repetida = veces(g) >= 2;
            let tipo = if i == 0 && tramos.len() > 2 && t.4 < 0.8 * mediana {
                TipoSeccion::Intro
            } else if i == ultimo && tramos.len() > 2 && t.4 < 0.8 * mediana {
                TipoSeccion::Final
            } else if Some(g) == coro {
                TipoSeccion::Coro
            } else if repetida {
                TipoSeccion::Verso
            } else {
                TipoSeccion::Puente
            };
            let apoyo = if repetida { 1.0 } else { 0.6 };
            Seccion {
                desde: t.0,
                hasta: t.1,
                tipo,
                grupo: g,
                confianza: (t.2 * apoyo).clamp(0.0, 1.0),
                energia: t.4,
            }
        })
        .collect()
}

/// La actividad vocal de un stem de voz, por compás y por cuarto de compás.
#[derive(Debug, Clone, PartialEq)]
pub struct Actividad {
    pub compases: Vec<f32>,
    pub cuartos: Vec<f32>,
}

/// Marcos de la envolvente vocal: 1.024 muestras (23 ms a 44,1 kHz).
const MARCO_VOZ: usize = 1024;

/// Actividad vocal a partir del stem de voz (mono, a `tasa`).
///
/// **Por qué no basta la RMS por compás.** Un compás de canto con una
/// respiración y uno de silencio con una sílaba final tienen RMS parecidas; lo
/// que importa para cortar es si se canta justo en la frontera. Aquí se decide
/// marco a marco (23 ms) y se resume por cuarto de compás:
///
/// * **Umbral adaptativo** a la propia voz: el piso de ruido (percentil 20 de
///   los marcos) más 12 dB, sin bajar de 24 dB por debajo del percentil 95.
///   El residuo que deja HTDemucs en tramos instrumentales queda por debajo.
/// * **Histéresis** de 6 dB: un marco activo sigue activo hasta caer 6 dB por
///   debajo del umbral de entrada; una vocal sostenida que baja no se parte.
/// * **Pausas**: un silencio de menos de 250 ms (una respiración, una
///   consonante sorda) no termina la frase.
/// * **Colas**: tras la última sílaba se cuentan 150 ms más de actividad (la
///   cola natural y la reverberación de la voz), para no cortarla en seco.
///
/// El resultado por cuarto y por compás es la fracción de marcos activos.
#[must_use]
pub fn actividad_vocal(voz: &[f32], tasa: u32, compases: &[f64]) -> Actividad {
    let nb = compases.len().saturating_sub(1);
    let marcos = voz.len() / MARCO_VOZ;
    let db: Vec<f32> = (0..marcos)
        .map(|m| {
            let t = &voz[m * MARCO_VOZ..(m + 1) * MARCO_VOZ];
            let e = t.iter().map(|v| v * v).sum::<f32>() / MARCO_VOZ as f32;
            10.0 * e.max(1e-12).log10()
        })
        .collect();
    let mut orden = db.clone();
    orden.sort_by(f32::total_cmp);
    let pct = |p: usize| orden.get(orden.len() * p / 100).copied().unwrap_or(-120.0);
    let (piso, alto) = (pct(20), pct(95));
    let entrada = (piso + 12.0).max(alto - 24.0).min(alto - 3.0);
    let salida = entrada - 6.0;
    let seg_marco = MARCO_VOZ as f64 / f64::from(tasa);
    let pausa = (0.25 / seg_marco).round() as usize;
    let cola = (0.15 / seg_marco).round() as usize;
    let mut activo = vec![false; marcos];
    let mut estado = false;
    for (a, d) in activo.iter_mut().zip(&db) {
        estado = if estado { *d > salida } else { *d > entrada };
        *a = estado;
    }
    // Pausas cortas dentro de una frase: se rellenan.
    let mut i = 0;
    while i < marcos {
        if activo[i] {
            i += 1;
            continue;
        }
        let j = (i..marcos).find(|k| activo[*k]).unwrap_or(marcos);
        if i > 0 && j < marcos && j - i < pausa {
            activo[i..j].iter_mut().for_each(|a| *a = true);
        }
        i = j;
    }
    // Colas tras el final de cada frase.
    let mut extra = 0usize;
    let mut con_cola = activo.clone();
    for (k, a) in activo.iter().enumerate() {
        if *a {
            extra = cola;
        } else if extra > 0 {
            con_cola[k] = true;
            extra -= 1;
        }
    }
    let fraccion = |s0: f64, s1: f64| -> f32 {
        let m0 = ((s0.max(0.0) / seg_marco).floor() as usize).min(marcos);
        let m1 = ((s1.max(0.0) / seg_marco).ceil() as usize).min(marcos);
        if m1 <= m0 {
            return 0.0;
        }
        con_cola[m0..m1].iter().filter(|a| **a).count() as f32 / (m1 - m0) as f32
    };
    let mut cuartos = Vec::with_capacity(nb * 4);
    let mut por_compas = Vec::with_capacity(nb);
    for k in 0..nb {
        let (a, b) = (compases[k], compases[k + 1]);
        let q = (b - a) / 4.0;
        for j in 0..4 {
            cuartos.push(fraccion(a + q * j as f64, a + q * (j + 1) as f64));
        }
        por_compas.push(fraccion(a, b));
    }
    Actividad {
        compases: por_compas,
        cuartos,
    }
}

/// Analiza un archivo entero.
///
/// # Errors
///
/// Los de la decodificación y de [`de_senal`].
pub fn analizar(ruta: &Path) -> NsResult<Estructura> {
    if !ruta.is_file() {
        return Err(NsError::FileNotFound);
    }
    de_senal(&rasgos::cargar_mono(ruta, None)?)
}

#[cfg(test)]
pub(crate) mod pruebas {
    use super::*;
    use std::f32::consts::TAU;

    /// Clics de 10 ms a `bpm` con acento grave cada cuatro, y un acorde de
    /// fondo por tramos: `tramos` es (compases, frecuencias del acorde, nivel).
    pub(crate) fn sintetica(bpm: f64, tramos: &[(usize, &[f32], f32)], desfase: f64) -> Vec<f32> {
        let sr = SR as f32;
        let compases: usize = tramos.iter().map(|t| t.0).sum();
        let periodo = 60.0 / bpm;
        let n = ((desfase + compases as f64 * 4.0 * periodo) * SR) as usize;
        let mut y = vec![0.0f32; n];
        let mut inicio = desfase;
        for (cs, acorde, nivel) in tramos {
            let fin = inicio + *cs as f64 * 4.0 * periodo;
            let (a, b) = ((inicio * SR) as usize, ((fin * SR) as usize).min(n));
            for (i, v) in y.iter_mut().enumerate().take(b).skip(a) {
                let t = i as f32 / sr;
                *v += acorde.iter().map(|f| (TAU * f * t).sin()).sum::<f32>() * 0.03 * nivel;
            }
            inicio = fin;
        }
        let total = compases * 4;
        for k in 0..total {
            let t0 = desfase + k as f64 * periodo;
            let i0 = (t0 * SR) as usize;
            let acento = k % 4 == 0;
            let largo = (0.06 * SR) as usize;
            for j in 0..largo.min(n.saturating_sub(i0)) {
                let t = j as f32 / sr;
                let env = (-t * 60.0).exp();
                let mut v = 0.4 * env * (TAU * 2500.0 * t).sin();
                if acento {
                    v += 0.8 * (-t * 25.0).exp() * (TAU * 60.0 * t).sin();
                }
                y[i0 + j] += v;
            }
        }
        y
    }

    const DO: &[f32] = &[261.6, 329.6, 392.0];
    const FA_S: &[f32] = &[370.0, 466.2, 554.4];

    #[test]
    fn detecta_el_tempo_y_coloca_los_pulsos_a_menos_de_20_ms() {
        for bpm in [96.0, 120.0, 128.0, 140.0] {
            let y = sintetica(bpm, &[(16, DO, 1.0)], 0.3);
            let e = de_senal(&y).unwrap();
            let b = e.bpm.unwrap();
            assert!((b - bpm).abs() <= 1.0, "{bpm}: {b}");
            assert!(e.confianza_pulso > 0.5, "{bpm}: {}", e.confianza_pulso);
            let periodo = 60.0 / bpm;
            let mut peor = 0.0f64;
            for p in &e.pulsos {
                let k = ((p - 0.3) / periodo).round();
                peor = peor.max((p - (0.3 + k * periodo)).abs());
            }
            assert!(peor < 0.020, "{bpm}: {peor}");
        }
    }

    #[test]
    fn el_doble_y_la_mitad_se_pliegan_a_la_misma_rejilla() {
        assert_eq!(plegar(70.0), 140.0);
        assert_eq!(plegar(280.0), 140.0);
        assert_eq!(plegar(120.0), 120.0);
        let y = sintetica(70.0, &[(10, DO, 1.0)], 0.2);
        let b = de_senal(&y).unwrap().bpm.unwrap();
        assert!((b - 140.0).abs() <= 1.5, "{b}");
        assert!((BPM_MIN..BPM_MAX).contains(&b));
    }

    #[test]
    fn el_compas_empieza_en_el_acento() {
        let y = sintetica(120.0, &[(16, DO, 1.0)], 0.25);
        let e = de_senal(&y).unwrap();
        assert!(e.confianza_compas > 0.2, "{}", e.confianza_compas);
        for c in e.compases.iter().take(e.compases.len() - 1) {
            let k = ((c - 0.25) / 2.0).round();
            assert!((c - (0.25 + k * 2.0)).abs() < 0.03, "compás en {c}");
        }
        assert!(e.n_compases() >= 14);
    }

    #[test]
    fn el_silencio_y_lo_demasiado_corto_no_se_analizan() {
        assert_eq!(de_senal(&vec![0.0; 100_000]), Err(NsError::AudioCorrupted));
        assert_eq!(de_senal(&[0.3; 100]), Err(NsError::AudioCorrupted));
        assert_eq!(de_senal(&[]), Err(NsError::AudioCorrupted));
    }

    #[test]
    fn sin_pulso_claro_no_se_inventa_una_rejilla() {
        // Un acorde sostenido y ruido suave: nada de ataques periódicos.
        let mut x = 7u64;
        let y: Vec<f32> = (0..(SR as usize * 20))
            .map(|i| {
                x = x.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let r = ((x >> 40) as f32 / (1u32 << 24) as f32 - 0.5) * 0.02;
                (TAU * 220.0 * i as f32 / SR as f32).sin() * 0.2 + r
            })
            .collect();
        let e = de_senal(&y).unwrap();
        assert!(!e.rejilla_fiable(), "confianza {}", e.confianza_pulso);
        assert_eq!(e.confianza_compas, 0.0);
        // Compases de reserva de 2 s, sin pretensión de precisión.
        assert!((e.compases[1] - e.compases[0] - COMPAS_SIN_PULSO).abs() < 1e-9);
    }

    #[test]
    fn segmenta_por_cambios_de_armonia_y_energia_y_reconoce_el_coro() {
        let y = sintetica(
            120.0,
            &[
                (8, DO, 0.3),
                (16, DO, 1.0),
                (16, FA_S, 2.0),
                (16, DO, 1.0),
                (16, FA_S, 2.0),
            ],
            0.0,
        );
        let e = de_senal(&y).unwrap();
        // Las fronteras se comparan en el tiempo (2 s por compás a 120 BPM): el
        // seguidor puede empezar la rejilla un compás después del inicio.
        let fronteras: Vec<f64> = e.secciones.iter().map(|s| e.compases[s.desde]).collect();
        for esperada in [24.0f64, 40.0, 56.0] {
            assert!(
                fronteras.iter().any(|f| (f - esperada * 2.0).abs() <= 2.1),
                "{esperada} no está en {fronteras:?}"
            );
        }
        let coros: Vec<&Seccion> = e
            .secciones
            .iter()
            .filter(|s| s.tipo == TipoSeccion::Coro)
            .collect();
        assert!(coros.len() >= 2, "{:?}", e.secciones);
        assert!(coros.iter().all(|s| e.compases[s.desde] >= 46.0));
        // Los cambios de energía se ven por compás.
        let en = |t: f64| e.energia[e.compases.iter().position(|c| *c >= t).unwrap()];
        assert!(en(60.0) > en(24.0) * 1.2, "{} {}", en(60.0), en(24.0));
    }

    #[test]
    fn la_actividad_vocal_distingue_canto_de_residuo() {
        let tasa = 44_100u32;
        let compases: Vec<f64> = (0..=8).map(|k| k as f64 * 2.0).collect();
        let voz: Vec<f32> = (0..tasa as usize * 16)
            .map(|i| {
                let t = i as f32 / tasa as f32;
                let canta = (t as usize / 2).is_multiple_of(2);
                (TAU * 300.0 * t).sin() * if canta { 0.3 } else { 0.003 }
            })
            .collect();
        let a = actividad_vocal(&voz, tasa, &compases).compases;
        assert_eq!(a.len(), 8);
        for (k, v) in a.iter().enumerate() {
            if k % 2 == 0 {
                assert!(*v > 0.9, "{k}: {v}");
            } else {
                assert!(*v < 0.1, "{k}: {v}");
            }
        }
    }

    #[test]
    fn un_archivo_ilegible_o_inexistente_da_error_y_no_estructura() {
        let d = std::env::temp_dir().join(format!("ns-estructura-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let roto = d.join("roto.mp3");
        std::fs::write(&roto, b"esto no es audio").unwrap();
        assert!(analizar(&roto).is_err());
        assert_eq!(analizar(&d.join("no.mp3")), Err(NsError::FileNotFound));
    }
}
