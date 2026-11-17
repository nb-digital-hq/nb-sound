//! Tempo y tono independientes (ADR-018).
//!
//! **Algoritmo.** Vocoder de fase con bloqueo de fase de identidad (Laroche y
//! Dolson, 1999): cada pico espectral avanza su fase con su frecuencia
//! instantánea y los bins de su región conservan la relación de fase con él.
//! Sin el bloqueo, el vocoder clásico suena "fasioso", sobre todo en voces.
//! Los transitorios (golpes de batería, consonantes) reinician la fase a la
//! del análisis: el ataque vuelve a ser coherente en lugar de emborronarse.
//!
//! **Por qué no remuestrear.** Remuestrear cambia tempo y tono a la vez: un 6 %
//! más rápido es un semitono más agudo. El vocoder separa las dos cosas. El
//! cambio de tono es estirar por `2^(s/12)` y remuestrear de vuelta con `rubato`
//! (sinc), en una sola pasada del vocoder aunque también cambie el tempo.
//!
//! **Por qué no WSOLA.** WSOLA conserva mejor los transitorios a costa de
//! repetir o saltar periodos, que en voces se oye como tartamudeo; con el
//! reinicio de fase el vocoder cubre los dos casos con un solo código.
//!
//! **Formantes.** Transponer estirando y remuestreando desplaza todo el
//! espectro, también la envolvente que da el timbre (los formantes): una voz
//! subida dos semitonos suena un 12 % "más pequeña", el efecto de ardilla. Con
//! [`Vocoder::conservar_formantes`] cada marco se corrige antes de sintetizar:
//! se estima la envolvente por cepstro (logaritmo del módulo, transformada
//! inversa y liftrado de los primeros 60 coeficientes, unos 1,4 ms) y se
//! multiplica el módulo por `E(r·f) / E(f)`, de modo que tras remuestrear por
//! `r` los armónicos se mueven y la envolvente se queda donde estaba. Es la
//! corrección clásica por envolvente cepstral (Röbel y Rodet, 2005, en su forma
//! más simple, sin el refinamiento iterativo de la "true envelope"); cuesta dos
//! FFT más por marco y solo se activa en voces y mezclas transpuestas. La
//! corrección se acota a ±12 dB para no levantar ruido donde la envolvente es
//! muy baja.
//!
//! **Límites.** Fuera de ±8 % de tempo y ±2 semitonos los artefactos se oyen;
//! el compositor no propone transformaciones mayores ([`TEMPO_MAX`],
//! [`SEMITONOS_MAX`]).

use std::f32::consts::{PI, TAU};
use std::sync::Arc;

use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use rustfft::num_complex::Complex32;
use rustfft::{Fft, FftPlanner};

use crate::error::{NsError, NsResult};

/// Cambio relativo de tempo máximo que se acepta.
pub const TEMPO_MAX: f64 = 0.08;
/// Transposición máxima en semitonos.
pub const SEMITONOS_MAX: i32 = 2;
/// Desvío de tempo que se tolera en un solo tramo de un mapa: los pulsos reales
/// vacilan más que el tempo medio, y un tramo corto estirado de más se oye menos
/// que un fragmento entero.
pub const TEMPO_MAX_TRAMO: f64 = 0.15;
/// 46 ms a 44,1 kHz: resuelve bien el bajo sin emborronar demasiado los golpes.
const N: usize = 2048;
/// Solapamiento de 4: el mínimo con el que la ventana de Hann suma constante.
const HS: usize = N / 4;

/// El vocoder con sus planes y buffers, para no asignar dentro del bucle.
pub struct Vocoder {
    ida: Arc<dyn Fft<f32>>,
    vuelta: Arc<dyn Fft<f32>>,
    ventana: Vec<f32>,
    buf: Vec<Complex32>,
    mag: Vec<f32>,
    fase: Vec<f32>,
    fase_ant: Vec<f32>,
    salida_ant: Vec<f32>,
    flujo_ant: Vec<f32>,
    picos: Vec<usize>,
    /// La razón de transposición cuya envolvente se compensa, si se pide.
    formantes: Option<f32>,
    cepstro: Vec<Complex32>,
    envolvente: Vec<f32>,
}

/// Coeficientes del cepstro que se conservan: la envolvente sin los armónicos
/// de voces de hasta unos 700 Hz de fundamental.
const LIFTER: usize = 60;
/// Corrección máxima de la envolvente, en neperios (±12 dB).
const CORRECCION_MAX: f32 = 1.38;

impl Default for Vocoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Vocoder {
    #[must_use]
    pub fn new() -> Self {
        let mut plan = FftPlanner::<f32>::new();
        let bins = N / 2 + 1;
        Self {
            ida: plan.plan_fft_forward(N),
            vuelta: plan.plan_fft_inverse(N),
            ventana: (0..N)
                .map(|i| 0.5 - 0.5 * (TAU * i as f32 / N as f32).cos())
                .collect(),
            buf: vec![Complex32::new(0.0, 0.0); N],
            mag: vec![0.0; bins],
            fase: vec![0.0; bins],
            fase_ant: vec![0.0; bins],
            salida_ant: vec![0.0; bins],
            flujo_ant: vec![0.0; bins],
            picos: Vec::with_capacity(bins),
            formantes: None,
            cepstro: vec![Complex32::new(0.0, 0.0); N],
            envolvente: vec![0.0; bins],
        }
    }

    /// Activa (con la razón de transposición `r = 2^(s/12)`) o desactiva la
    /// conservación de formantes para las siguientes transformaciones.
    pub fn conservar_formantes(&mut self, r: Option<f64>) {
        self.formantes = r.filter(|r| (r - 1.0).abs() > 1e-6).map(|r| r as f32);
    }

    /// Multiplica el módulo del marco actual por `E(r·f) / E(f)`.
    fn corregir_envolvente(&mut self, r: f32) {
        let bins = N / 2 + 1;
        for b in 0..bins {
            let l = self.mag[b].max(1e-9).ln();
            self.cepstro[b] = Complex32::new(l, 0.0);
            if b > 0 && b < N / 2 {
                self.cepstro[N - b] = Complex32::new(l, 0.0);
            }
        }
        self.vuelta.process(&mut self.cepstro);
        for (n, c) in self.cepstro.iter_mut().enumerate() {
            let q = n.min(N - n);
            *c = if q < LIFTER {
                Complex32::new(c.re / N as f32, 0.0)
            } else {
                Complex32::new(0.0, 0.0)
            };
        }
        self.ida.process(&mut self.cepstro);
        for b in 0..bins {
            self.envolvente[b] = self.cepstro[b].re;
        }
        for b in 0..bins {
            let x = b as f32 * r;
            let i = (x.floor() as usize).min(bins - 1);
            let j = (i + 1).min(bins - 1);
            let fr = (x - i as f32).clamp(0.0, 1.0);
            let destino = self.envolvente[i] * (1.0 - fr) + self.envolvente[j] * fr;
            let d = (destino - self.envolvente[b]).clamp(-CORRECCION_MAX, CORRECCION_MAX);
            self.mag[b] *= d.exp();
        }
    }

    /// Los marcos que son transitorio, según el flujo espectral de `x` (mono o
    /// la suma de los canales): se decide una vez y se aplica igual a los dos
    /// canales, para que el reinicio no descoloque la imagen estéreo.
    #[must_use]
    pub fn transitorios(&mut self, x: &[f32], factor: f64) -> Vec<bool> {
        self.transitorios_en(x, &posiciones(x.len(), factor))
    }

    /// Como [`Vocoder::transitorios`], sobre posiciones de análisis dadas.
    fn transitorios_en(&mut self, x: &[f32], posiciones: &[usize]) -> Vec<bool> {
        let mut flujos = Vec::with_capacity(posiciones.len());
        self.flujo_ant.iter_mut().for_each(|v| *v = 0.0);
        for &p in posiciones {
            self.analizar(x, p);
            // Flujo relativo: la energía nueva respecto de la total del marco. En
            // absoluto, las fluctuaciones de un tono sostenido pasaban por golpes,
            // y cada reinicio de fase sobre un tono es un cambio de tono.
            let (mut f, mut total) = (0.0f32, 1e-9f32);
            for (m, a) in self.mag.iter().zip(&self.flujo_ant) {
                f += (m - a).max(0.0);
                total += m;
            }
            self.flujo_ant.copy_from_slice(&self.mag);
            flujos.push(f / total);
        }
        // Un transitorio es un flujo que dobla la mediana local, es máximo local
        // y trae al menos un 20 % de energía nueva.
        (0..flujos.len())
            .map(|k| {
                let desde = k.saturating_sub(8);
                let hasta = (k + 8).min(flujos.len());
                let mut v: Vec<f32> = flujos[desde..hasta].to_vec();
                v.sort_by(f32::total_cmp);
                let med = v[v.len() / 2];
                let local = flujos[k] >= flujos[k.saturating_sub(1)]
                    && flujos[k] >= *flujos.get(k + 1).unwrap_or(&0.0);
                k > 0 && local && flujos[k] > 2.0 * med && flujos[k] > 0.2
            })
            .collect()
    }

    fn analizar(&mut self, x: &[f32], pos: usize) {
        for (i, b) in self.buf.iter_mut().enumerate() {
            let v = x.get(pos + i).copied().unwrap_or(0.0);
            *b = Complex32::new(v * self.ventana[i], 0.0);
        }
        self.ida.process(&mut self.buf);
        for k in 0..=N / 2 {
            self.mag[k] = self.buf[k].norm();
            self.fase[k] = self.buf[k].arg();
        }
    }

    /// Estira `x` por `factor` (salida / entrada) sin cambiar el tono.
    /// `transitorios` viene de [`Vocoder::transitorios`] con el mismo factor.
    #[must_use]
    pub fn estirar(&mut self, x: &[f32], factor: f64, transitorios: &[bool]) -> Vec<f32> {
        let largo = (x.len() as f64 * factor).round() as usize;
        if (factor - 1.0).abs() < 1e-6 && self.formantes.is_none() {
            return x.to_vec();
        }
        self.estirar_en(x, &posiciones(x.len(), factor), largo, transitorios)
    }

    /// Estira con posiciones de análisis arbitrarias (una por salto de síntesis)
    /// hasta `largo` muestras de salida.
    fn estirar_en(
        &mut self,
        x: &[f32],
        posiciones: &[usize],
        largo: usize,
        transitorios: &[bool],
    ) -> Vec<f32> {
        let mut y = vec![0.0f32; largo + N];
        let mut peso = vec![0.0f32; largo + N];
        let bins = N / 2 + 1;
        for (k, &pos) in posiciones.iter().enumerate() {
            self.analizar(x, pos);
            let reinicio = k == 0 || transitorios.get(k).copied().unwrap_or(false);
            let ha = if k == 0 { 0 } else { pos - posiciones[k - 1] };
            if reinicio {
                self.salida_ant.copy_from_slice(&self.fase);
            } else {
                // Picos: máximos sobre ±2 bins.
                self.picos.clear();
                for b in 2..bins - 2 {
                    let m = self.mag[b];
                    if m > self.mag[b - 1]
                        && m >= self.mag[b + 1]
                        && m > self.mag[b - 2]
                        && m >= self.mag[b + 2]
                    {
                        self.picos.push(b);
                    }
                }
                let mut nuevas = std::mem::take(&mut self.salida_ant);
                // Fase de cada pico: avance con su frecuencia instantánea.
                let mut avance_pico = vec![0.0f32; 0];
                avance_pico.reserve(self.picos.len());
                for &p in &self.picos {
                    let omega = TAU * p as f32 / N as f32;
                    let delta = principal(self.fase[p] - self.fase_ant[p] - omega * ha as f32);
                    let inst = omega + delta / ha.max(1) as f32;
                    avance_pico.push(nuevas[p] + inst * HS as f32);
                }
                // Cada bin sigue al pico más cercano (frontera en el mínimo entre picos).
                let mut j = 0;
                let mut salida = vec![0.0f32; bins];
                for (b, s) in salida.iter_mut().enumerate() {
                    if self.picos.is_empty() {
                        let omega = TAU * b as f32 / N as f32;
                        *s = nuevas[b] + omega * HS as f32;
                        continue;
                    }
                    while j + 1 < self.picos.len() && self.picos[j + 1] <= b {
                        j += 1;
                    }
                    let p = if j + 1 < self.picos.len() {
                        let (a, c) = (self.picos[j], self.picos[j + 1]);
                        if b.abs_diff(a) <= b.abs_diff(c) {
                            j
                        } else {
                            j + 1
                        }
                    } else {
                        j
                    };
                    let pico = self.picos[p];
                    *s = avance_pico[p] + (self.fase[b] - self.fase[pico]);
                }
                nuevas.copy_from_slice(&salida);
                self.salida_ant = nuevas;
            }
            self.fase_ant.copy_from_slice(&self.fase);
            if let Some(r) = self.formantes {
                self.corregir_envolvente(r);
            }
            // Síntesis.
            for b in 0..bins {
                let c = Complex32::from_polar(self.mag[b], self.salida_ant[b]);
                self.buf[b] = c;
                if b > 0 && b < N / 2 {
                    self.buf[N - b] = c.conj();
                }
            }
            self.vuelta.process(&mut self.buf);
            let destino = k * HS;
            for i in 0..N {
                let w = self.ventana[i];
                if let (Some(o), Some(p)) = (y.get_mut(destino + i), peso.get_mut(destino + i)) {
                    *o += self.buf[i].re / N as f32 * w;
                    *p += w * w;
                }
            }
        }
        for (o, p) in y.iter_mut().zip(&peso) {
            if *p > 1e-3 {
                *o /= p;
            }
        }
        y.truncate(largo);
        y
    }
}

fn principal(f: f32) -> f32 {
    (f + PI).rem_euclid(TAU) - PI
}

/// Las posiciones de análisis: una cada `HS / factor` muestras.
fn posiciones(entrada: usize, factor: f64) -> Vec<usize> {
    let salida = (entrada as f64 * factor).round() as usize;
    let marcos = salida / HS + 1;
    (0..marcos)
        .map(|k| ((k * HS) as f64 / factor).round() as usize)
        .collect()
}

/// Remuestrea por `razon` (salida / entrada) con sinc de alta calidad.
fn remuestrear(canales: &[Vec<f32>], razon: f64) -> NsResult<Vec<Vec<f32>>> {
    if (razon - 1.0).abs() < 1e-9 {
        return Ok(canales.to_vec());
    }
    let n = canales.first().map_or(0, Vec::len);
    let trozo = 4096;
    let params = SincInterpolationParameters {
        sinc_len: 128,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Cubic,
        oversampling_factor: 128,
        window: WindowFunction::BlackmanHarris2,
    };
    let mut rs = SincFixedIn::<f32>::new(razon, 1.0, params, trozo, canales.len())
        .map_err(|_| NsError::Internal)?;
    let esperado = (n as f64 * razon).round() as usize;
    let mut fuera: Vec<Vec<f32>> = vec![Vec::with_capacity(esperado + trozo); canales.len()];
    let mut i = 0;
    while i + trozo <= n {
        let entrada: Vec<&[f32]> = canales.iter().map(|c| &c[i..i + trozo]).collect();
        let out = rs.process(&entrada, None).map_err(|_| NsError::Internal)?;
        for (f, o) in fuera.iter_mut().zip(out) {
            f.extend_from_slice(&o);
        }
        i += trozo;
    }
    if i < n {
        let entrada: Vec<&[f32]> = canales.iter().map(|c| &c[i..]).collect();
        let out = rs
            .process_partial(Some(&entrada), None)
            .map_err(|_| NsError::Internal)?;
        for (f, o) in fuera.iter_mut().zip(out) {
            f.extend_from_slice(&o);
        }
    }
    let cola = rs
        .process_partial::<&[f32]>(None, None)
        .map_err(|_| NsError::Internal)?;
    for (f, o) in fuera.iter_mut().zip(cola) {
        f.extend_from_slice(&o);
    }
    let retardo = rs.output_delay();
    Ok(fuera
        .into_iter()
        .map(|f| {
            let mut v: Vec<f32> = f.into_iter().skip(retardo).take(esperado).collect();
            v.resize(esperado, 0.0);
            v
        })
        .collect())
}

/// Cambia tempo y tono de un fragmento estéreo. `tempo` es la razón de
/// velocidad (1,05 = un 5 % más rápido); la duración sale dividida por ella.
///
/// # Errors
///
/// [`NsError::InvalidInput`] fuera de los límites; [`NsError::Internal`] si el
/// remuestreador falla.
pub fn transformar(
    vocoder: &mut Vocoder,
    izq: &[f32],
    der: &[f32],
    tempo: f64,
    semitonos: i32,
) -> NsResult<(Vec<f32>, Vec<f32>)> {
    if !(tempo.is_finite() && (tempo - 1.0).abs() <= TEMPO_MAX + 1e-9)
        || semitonos.abs() > SEMITONOS_MAX
    {
        return Err(NsError::InvalidInput);
    }
    if (tempo - 1.0).abs() < 1e-6 && semitonos == 0 {
        return Ok((izq.to_vec(), der.to_vec()));
    }
    let r = 2f64.powf(f64::from(semitonos) / 12.0);
    // Estirar por r/tempo y remuestrear por 1/r: el tono sube r y la duración
    // queda dividida por tempo.
    let factor = r / tempo;
    let mono: Vec<f32> = izq.iter().zip(der).map(|(a, b)| 0.5 * (a + b)).collect();
    let tr = vocoder.transitorios(&mono, factor);
    drop(mono);
    let a = vocoder.estirar(izq, factor, &tr);
    let b = vocoder.estirar(der, factor, &tr);
    let mut c = remuestrear(&[a, b], 1.0 / r)?;
    let b = c.pop().unwrap_or_default();
    let a = c.pop().unwrap_or_default();
    Ok((a, b))
}

/// Las posiciones de análisis de un mapa de tiempo a trozos: una por salto de
/// síntesis, interpolando entre anclas `(entrada, salida)` en muestras y
/// prolongando el último tramo más allá de la última ancla.
fn posiciones_mapa(mapa: &[(f64, f64)], largo: usize) -> Vec<usize> {
    let mut j = 0;
    (0..=largo / HS)
        .map(|k| {
            let t = (k * HS) as f64;
            while j + 2 < mapa.len() && mapa[j + 1].1 <= t {
                j += 1;
            }
            let ((e0, s0), (e1, s1)) = (mapa[j], mapa[j + 1]);
            let p = e0 + (t - s0) * (e1 - e0) / (s1 - s0).max(1.0);
            p.max(0.0).round() as usize
        })
        .collect()
}

/// Como [`transformar`], pero el tempo sigue un mapa a trozos: cada ancla
/// `(entrada, salida)` en muestras fija dónde cae un instante del original en
/// el resultado. Así un fragmento cuyos pulsos reales vacilan queda clavado a
/// la rejilla del destino pulso a pulso, no solo en sus extremos: con un tempo
/// único, una batería tocada en vivo se desliza decenas de milisegundos a
/// mitad de frase y la voz de encima deja de caer con la base.
///
/// # Errors
///
/// [`NsError::InvalidInput`] con menos de dos anclas, anclas no crecientes, un
/// tramo fuera de los límites de tempo o una transposición excesiva.
pub fn transformar_mapa(
    vocoder: &mut Vocoder,
    izq: &[f32],
    der: &[f32],
    mapa: &[(f64, f64)],
    largo: usize,
    semitonos: i32,
) -> NsResult<(Vec<f32>, Vec<f32>)> {
    let tramos_validos = mapa.len() >= 2
        && mapa.windows(2).all(|w| {
            let (de, ds) = (w[1].0 - w[0].0, w[1].1 - w[0].1);
            de > 0.0 && ds > 0.0 && (de / ds - 1.0).abs() <= TEMPO_MAX_TRAMO
        });
    if !tramos_validos || semitonos.abs() > SEMITONOS_MAX {
        return Err(NsError::InvalidInput);
    }
    let r = 2f64.powf(f64::from(semitonos) / 12.0);
    // El vocoder trabaja en el dominio estirado por r; el remuestreo final
    // lo devuelve a `largo`.
    let escalado: Vec<(f64, f64)> = mapa.iter().map(|&(e, s)| (e, s * r)).collect();
    let largo_r = (largo as f64 * r).round() as usize;
    let pos = posiciones_mapa(&escalado, largo_r);
    let mono: Vec<f32> = izq.iter().zip(der).map(|(a, b)| 0.5 * (a + b)).collect();
    let tr = vocoder.transitorios_en(&mono, &pos);
    drop(mono);
    let a = vocoder.estirar_en(izq, &pos, largo_r, &tr);
    let b = vocoder.estirar_en(der, &pos, largo_r, &tr);
    let mut c = remuestrear(&[a, b], 1.0 / r)?;
    let mut b = c.pop().unwrap_or_default();
    let mut a = c.pop().unwrap_or_default();
    a.resize(largo, 0.0);
    b.resize(largo, 0.0);
    Ok((a, b))
}

#[cfg(test)]
pub(crate) mod pruebas {
    use super::*;

    const TASA: f32 = 44_100.0;

    fn seno(f: f32, segundos: f32) -> Vec<f32> {
        (0..(TASA * segundos) as usize)
            .map(|i| (TAU * f * i as f32 / TASA).sin() * 0.5)
            .collect()
    }

    /// Frecuencia dominante por FFT con interpolación parabólica, sobre el
    /// centro de la señal (los bordes del vocoder no cuentan).
    pub(crate) fn frecuencia(x: &[f32]) -> f32 {
        let n = 32_768usize;
        let ini = (x.len().saturating_sub(n)) / 2;
        let mut plan = FftPlanner::<f32>::new();
        let fft = plan.plan_fft_forward(n);
        let mut buf: Vec<Complex32> = (0..n)
            .map(|i| {
                let w = 0.5 - 0.5 * (TAU * i as f32 / n as f32).cos();
                Complex32::new(x.get(ini + i).copied().unwrap_or(0.0) * w, 0.0)
            })
            .collect();
        fft.process(&mut buf);
        let m: Vec<f32> = buf[..n / 2]
            .iter()
            .map(|c| c.norm().max(1e-12).ln())
            .collect();
        let k = (1..n / 2 - 1)
            .max_by(|a, b| m[*a].total_cmp(&m[*b]))
            .unwrap();
        let d = (m[k - 1] - m[k + 1]) / (2.0 * (m[k - 1] - 2.0 * m[k] + m[k + 1]));
        (k as f32 + d) * TASA / n as f32
    }

    fn cents(a: f32, b: f32) -> f32 {
        1200.0 * (a / b).log2()
    }

    #[test]
    fn estirar_conserva_el_tono_dentro_de_5_cents() {
        let mut v = Vocoder::new();
        for f in [110.0, 440.0, 1760.0] {
            let x = seno(f, 3.0);
            for tempo in [0.92, 0.96, 1.04, 1.08] {
                let (a, _) = transformar(&mut v, &x, &x, tempo, 0).unwrap();
                let medida = frecuencia(&a);
                assert!(cents(medida, f).abs() < 5.0, "{f} Hz a {tempo}: {medida}");
            }
        }
    }

    #[test]
    fn la_duracion_sale_dividida_por_el_tempo_a_la_muestra() {
        let mut v = Vocoder::new();
        let x = seno(440.0, 2.0);
        for tempo in [0.93, 1.0, 1.07] {
            for s in [-2, 0, 2] {
                let (a, b) = transformar(&mut v, &x, &x, tempo, s).unwrap();
                let esperado = (x.len() as f64 / tempo).round() as usize;
                assert!(
                    a.len().abs_diff(esperado) <= 1,
                    "{tempo} {s}: {} vs {esperado}",
                    a.len()
                );
                assert_eq!(a.len(), b.len());
            }
        }
    }

    #[test]
    fn transponer_cambia_el_tono_lo_pedido_sin_cambiar_la_duracion() {
        let mut v = Vocoder::new();
        let x = seno(440.0, 3.0);
        for s in [-2, -1, 1, 2] {
            let (a, _) = transformar(&mut v, &x, &x, 1.0, s).unwrap();
            let esperada = 440.0 * 2f32.powf(s as f32 / 12.0);
            assert!(cents(frecuencia(&a), esperada).abs() < 5.0, "{s}");
            assert!(a.len().abs_diff(x.len()) <= 1);
        }
    }

    /// Una vocal sintética: armónicos de `f0` con la envolvente de dos
    /// formantes fijos (700 y 1.200 Hz), como una /a/.
    fn vocal(f0: f32, segundos: f32) -> Vec<f32> {
        let formante = |f: f32| -> f32 {
            let r = |c: f32, a: f32| 1.0 / (1.0 + ((f - c) / 120.0).powi(2)) * a;
            r(700.0, 1.0) + r(1200.0, 0.6) + 0.02
        };
        let armonicos: Vec<(f32, f32)> = (1..40)
            .map(|h| h as f32 * f0)
            .filter(|f| *f < 5000.0)
            .map(|f| (f, formante(f)))
            .collect();
        (0..(TASA * segundos) as usize)
            .map(|i| {
                let t = i as f32 / TASA;
                armonicos
                    .iter()
                    .map(|(f, a)| a * (TAU * f * t).sin())
                    .sum::<f32>()
                    * 0.1
            })
            .collect()
    }

    /// Centroide espectral entre 300 y 2.500 Hz: dónde está la masa de los
    /// formantes, sin depender de qué armónico cae justo en el pico.
    fn centroide_formantes(x: &[f32]) -> f32 {
        let n = 16_384usize;
        let ini = (x.len().saturating_sub(n)) / 2;
        let mut plan = FftPlanner::<f32>::new();
        let fft = plan.plan_fft_forward(n);
        let mut buf: Vec<Complex32> = (0..n)
            .map(|i| {
                let w = 0.5 - 0.5 * (TAU * i as f32 / n as f32).cos();
                Complex32::new(x.get(ini + i).copied().unwrap_or(0.0) * w, 0.0)
            })
            .collect();
        fft.process(&mut buf);
        let (mut s, mut w) = (0.0f32, 0.0f32);
        for (k, c) in buf[..n / 2].iter().enumerate() {
            let f = k as f32 * TASA / n as f32;
            if (300.0..2500.0).contains(&f) {
                let p = c.norm_sqr();
                s += f * p;
                w += p;
            }
        }
        s / w.max(1e-12)
    }

    #[test]
    fn transponer_una_voz_conserva_sus_formantes_y_cambia_su_tono() {
        let x = vocal(160.0, 3.0);
        let original = centroide_formantes(&x);
        let mut v = Vocoder::new();
        let (sin, _) = transformar(&mut v, &x, &x, 1.0, 2).unwrap();
        v.conservar_formantes(Some(2f64.powf(2.0 / 12.0)));
        let (con, _) = transformar(&mut v, &x, &x, 1.0, 2).unwrap();
        v.conservar_formantes(None);
        let desplazado = centroide_formantes(&sin) / original - 1.0;
        let conservado = centroide_formantes(&con) / original - 1.0;
        // Sin corrección la envolvente sube con el tono (un 12 % a +2
        // semitonos); con ella se queda cerca de donde estaba.
        assert!(desplazado > 0.07, "sin corrección: {desplazado}");
        assert!(
            conservado.abs() < 0.05 && conservado.abs() < desplazado / 2.0,
            "con corrección: {conservado} (sin ella {desplazado})"
        );
        // El tono sí sube: el primer armónico pasa de 160 a 179,6 Hz.
        let f0 = |y: &[f32]| {
            // El pico más alto entre 150 y 215 Hz: el primer armónico.
            let n = 32_768usize;
            let ini = (y.len().saturating_sub(n)) / 2;
            let mut plan = FftPlanner::<f32>::new();
            let fft = plan.plan_fft_forward(n);
            let mut buf: Vec<Complex32> = (0..n)
                .map(|i| {
                    let w = 0.5 - 0.5 * (TAU * i as f32 / n as f32).cos();
                    Complex32::new(y.get(ini + i).copied().unwrap_or(0.0) * w, 0.0)
                })
                .collect();
            fft.process(&mut buf);
            let bin = |f: f32| (f * n as f32 / TASA) as usize;
            let k = (bin(150.0)..bin(215.0))
                .max_by(|a, b| buf[*a].norm().total_cmp(&buf[*b].norm()))
                .unwrap();
            k as f32 * TASA / n as f32
        };
        let esperada = 160.0 * 2f32.powf(2.0 / 12.0);
        assert!(cents(f0(&con), esperada).abs() < 20.0, "{}", f0(&con));
        assert!(con.len().abs_diff(x.len()) <= 1);
    }

    #[test]
    fn la_identidad_no_toca_la_senal_y_lo_desmesurado_se_rechaza() {
        let mut v = Vocoder::new();
        let x = seno(300.0, 0.5);
        assert_eq!(transformar(&mut v, &x, &x, 1.0, 0).unwrap().0, x);
        assert_eq!(
            transformar(&mut v, &x, &x, 1.2, 0),
            Err(NsError::InvalidInput)
        );
        assert_eq!(
            transformar(&mut v, &x, &x, 1.0, 5),
            Err(NsError::InvalidInput)
        );
        assert_eq!(
            transformar(&mut v, &x, &x, f64::NAN, 0),
            Err(NsError::InvalidInput)
        );
    }

    #[test]
    fn los_golpes_siguen_nitidos_y_en_su_sitio_tras_estirar() {
        // Golpes de 5 ms cada 0,5 s sobre un tono suave.
        let n = (TASA * 4.0) as usize;
        let mut x: Vec<f32> = (0..n)
            .map(|i| (TAU * 220.0 * i as f32 / TASA).sin() * 0.05)
            .collect();
        let golpes: Vec<usize> = (1..8).map(|k| k * (TASA * 0.5) as usize).collect();
        for &g in &golpes {
            for j in 0..220 {
                x[g + j] += (-(j as f32) / 40.0).exp() * if j % 2 == 0 { 0.9 } else { -0.9 };
            }
        }
        let mut v = Vocoder::new();
        let tempo = 0.94;
        let (a, _) = transformar(&mut v, &x, &x, tempo, 0).unwrap();
        for &g in &golpes {
            let centro = (g as f64 / tempo) as usize;
            let ventana = (TASA * 0.01) as usize;
            let (lo, hi) = (
                centro.saturating_sub(ventana),
                (centro + ventana).min(a.len()),
            );
            let pico = a[lo..hi].iter().fold(0.0f32, |m, v| m.max(v.abs()));
            // Sin reinicio de fase el golpe se reparte en 46 ms y su pico cae por
            // debajo de 0,3; con él conserva más de la mitad del original.
            assert!(pico > 0.45, "golpe en {g}: pico {pico}");
        }
    }

    #[test]
    fn un_mapa_de_pulsos_lleva_cada_golpe_a_su_sitio_de_la_rejilla() {
        // Golpes que vacilan ±30 ms alrededor de cada 0,5 s; el mapa los lleva a
        // una rejilla exacta de 0,52 s.
        let n = (TASA * 4.5) as usize;
        let mut x: Vec<f32> = (0..n)
            .map(|i| (TAU * 220.0 * i as f32 / TASA).sin() * 0.05)
            .collect();
        let vacilacion = [0.0, 0.03, -0.02, 0.025, -0.03, 0.01, -0.015, 0.02];
        let reales: Vec<f64> = (1..8).map(|k| k as f64 * 0.5 + vacilacion[k]).collect();
        for &t in &reales {
            let g = (t * f64::from(TASA)) as usize;
            for j in 0..220 {
                x[g + j] += (-(j as f32) / 40.0).exp() * if j % 2 == 0 { 0.9 } else { -0.9 };
            }
        }
        let tasa = f64::from(TASA);
        let mut mapa = vec![(0.0, 0.0)];
        mapa.extend(
            reales
                .iter()
                .enumerate()
                .map(|(k, t)| (t * tasa, (k + 1) as f64 * 0.52 * tasa)),
        );
        mapa.push((4.0 * tasa, 4.16 * tasa));
        let largo = (4.16 * tasa) as usize;
        let mut v = Vocoder::new();
        for s in [0, 1] {
            let (a, b) = transformar_mapa(&mut v, &x, &x, &mapa, largo, s).unwrap();
            assert_eq!((a.len(), b.len()), (largo, largo));
            for k in 1..8 {
                let centro = (k as f64 * 0.52 * tasa) as usize;
                let ventana = (TASA * 0.01) as usize;
                let pico = a[centro - ventana..centro + ventana]
                    .iter()
                    .fold(0.0f32, |m, v| m.max(v.abs()));
                assert!(
                    pico > if s == 0 { 0.3 } else { 0.2 },
                    "semitonos {s}, golpe {k}: {pico}"
                );
            }
        }
        // Un tramo desmesurado se rechaza en lugar de deformar el audio.
        let malo = [(0.0, 0.0), (tasa, 1.5 * tasa)];
        assert_eq!(
            transformar_mapa(&mut v, &x, &x, &malo, largo, 0),
            Err(NsError::InvalidInput)
        );
    }
}
