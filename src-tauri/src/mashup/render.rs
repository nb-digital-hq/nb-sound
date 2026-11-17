//! El renderizador fuera de línea (ADR-018).
//!
//! **Por qué fuera de línea.** Un mashup denso tiene decenas de capas, cada una
//! con su vocoder; en el hilo de audio no cabe sin cortes (ADR-003). Aquí se
//! trabaja en un hilo de fondo y el resultado es un WAV que el reproductor de
//! siempre reproduce como cualquier archivo.
//!
//! **Memoria acotada.** Las capas se procesan por orden de destino. El
//! acumulador solo cubre desde lo ya volcado hasta el final de la capa activa
//! más larga; todo lo anterior al inicio de la siguiente capa está completo y
//! se vuelca a disco. La RAM depende de las capas simultáneas (tres como mucho,
//! de 16 compases), no del número de canciones ni de la duración total.
//!
//! **Dos pasadas.** La primera mezcla y escribe flotantes en un temporal
//! mientras mide la sonoridad integrada (EBU R128); la segunda aplica la
//! ganancia que lleva al objetivo de la biblioteca (−14 LUFS), pasa el
//! limitador del motor y escribe el WAV de 16 bits. La sonoridad global no se
//! conoce hasta el final: sin la segunda pasada habría que normalizar a ciegas.
//!
//! **Cortes sin clic.** Toda capa entra y sale con un fundido de igual potencia
//! de al menos 5 ms; un cruce entre dos bases solapa la cola de la saliente con
//! la entrada de la nueva.
//!
//! **Transiciones por bandas.** Una capa se puede partir en graves (Linkwitz-
//! Riley de cuarto orden a 150 Hz) y el resto, que suman exactamente la señal.
//! Una entrada filtrada sube primero sin graves y los recibe de golpe en el
//! primer tiempo; la saliente suelta los suyos en ese mismo instante
//! (`graves_fuera`): es el intercambio de graves de un DJ, que evita dos bajos
//! a la vez sin vaciar la mezcla.
//!
//! **Voz al frente.** Las voces van a un acumulador aparte. Al volcar, se mide
//! en la banda de presencia (1 a 4 kHz) cuánto enmascara el acompañamiento a la
//! voz y, solo cuando la tapa, se atenúa esa banda del acompañamiento hasta
//! 5 dB, con ataque de 60 ms y relajación de 350 ms para que no bombee. Si la
//! voz ya se oye, la mezcla no se toca.
//!
//! **Picos.** Antes del limitador suave del motor, un limitador con 3 ms de
//! anticipo lleva los picos a −1 dBFS con una ganancia suavizada; el pico
//! verdadero (sobremuestreado, EBU R128) se mide y se guarda.
//!
//! **Final.** El audio dura el plan más la cola de las capas que terminan en
//! su último compás: un cierre que se desvanece no se trunca.

use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use ebur128::{EbuR128, Mode};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::compositor::{Capa, Material, Plan, Transicion};
use super::estiramiento::{self, Vocoder, TEMPO_MAX};
use super::estructura::Estructura;
use crate::audio::limitador::limitar;
use crate::audio::sonoridad::OBJETIVO_LUFS;
use crate::error::{NsError, NsResult};

pub const TASA: u32 = 44_100;
/// Sonoridad a la que se lleva cada capa antes de su ganancia de papel: así una
/// voz grabada baja y una base muy comprimida quedan en la misma escala.
const CAPA_LUFS: f64 = -18.0;
const FUNDIDO_MIN_SEG: f64 = 0.005;
const BLOQUE: usize = 16_384;

/// Lee audio de una pista o de sus stems.
pub trait Origen {
    /// `segundos` de `material` de la pista desde `desde`, en estéreo planar a
    /// [`TASA`]. Puede devolver menos si la pista se acaba.
    ///
    /// # Errors
    ///
    /// Los de la lectura.
    fn leer(
        &mut self,
        pista: i64,
        material: Material,
        desde: f64,
        segundos: f64,
    ) -> NsResult<(Vec<f32>, Vec<f32>)>;
}

/// Lo que se mide del resultado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Metricas {
    pub duracion_seg: f64,
    pub lufs: f64,
    pub pico_dbfs: f64,
    /// Muestras en el techo del limitador o por encima.
    pub recortes: u32,
    /// El mayor salto entre muestras consecutivas, relativo al pico local.
    pub salto_max: f64,
    /// El hueco de silencio (< −60 dBFS en 50 ms) más largo, en segundos.
    pub silencio_max_seg: f64,
    pub capas: u32,
    pub segundos_render: f64,
    /// Lo más que llegó a cubrir el acumulador en memoria: la prueba de que la
    /// RAM no crece con la duración del mashup.
    pub acumulador_max_seg: f64,
    /// Pico verdadero (sobremuestreado) del resultado, en dBTP.
    #[serde(default)]
    pub pico_verdadero_dbtp: f64,
}

/// Un biquad en forma directa II transpuesta.
#[derive(Debug, Clone, Copy, Default)]
struct Biquad {
    b: [f32; 3],
    a: [f32; 2],
    z: [f32; 2],
}

impl Biquad {
    fn paso_bajo(f: f32) -> Self {
        Self::rbj(f, true)
    }

    fn paso_alto(f: f32) -> Self {
        Self::rbj(f, false)
    }

    /// Butterworth de segundo orden (Q = 1/√2), fórmulas de Robert
    /// Bristow-Johnson.
    fn rbj(f: f32, bajo: bool) -> Self {
        let w = std::f32::consts::TAU * f / TASA as f32;
        let (sn, cs) = w.sin_cos();
        let alfa = sn / (2.0 * std::f32::consts::FRAC_1_SQRT_2);
        let a0 = 1.0 + alfa;
        let b = if bajo {
            [(1.0 - cs) / 2.0, 1.0 - cs, (1.0 - cs) / 2.0]
        } else {
            [(1.0 + cs) / 2.0, -(1.0 + cs), (1.0 + cs) / 2.0]
        };
        Self {
            b: [b[0] / a0, b[1] / a0, b[2] / a0],
            a: [-2.0 * cs / a0, (1.0 - alfa) / a0],
            z: [0.0; 2],
        }
    }

    fn procesar(&mut self, x: f32) -> f32 {
        let y = self.b[0] * x + self.z[0];
        self.z[0] = self.b[1] * x - self.a[0] * y + self.z[1];
        self.z[1] = self.b[2] * x - self.a[1] * y;
        y
    }
}

/// Graves de una señal: Linkwitz-Riley de cuarto orden (dos Butterworth en
/// cascada). El resto es la señal menos los graves, así que sumar las dos
/// bandas devuelve el original exacto.
const CORTE_GRAVES: f32 = 150.0;

fn graves(x: &[f32]) -> Vec<f32> {
    let (mut a, mut b) = (
        Biquad::paso_bajo(CORTE_GRAVES),
        Biquad::paso_bajo(CORTE_GRAVES),
    );
    x.iter().map(|v| b.procesar(a.procesar(*v))).collect()
}

/// La banda de presencia de la voz, 1 a 4 kHz.
#[derive(Debug, Clone, Copy)]
struct Banda {
    alto: Biquad,
    bajo: Biquad,
}

impl Banda {
    fn new() -> Self {
        Self {
            alto: Biquad::paso_alto(1000.0),
            bajo: Biquad::paso_bajo(4000.0),
        }
    }

    fn procesar(&mut self, x: f32) -> f32 {
        self.bajo.procesar(self.alto.procesar(x))
    }
}

/// El estado del control de enmascaramiento entre bloques.
struct Presencia {
    acomp: [Banda; 2],
    voz: Banda,
    e_acomp: f32,
    e_voz: f32,
    atenuacion: f32,
}

impl Presencia {
    fn new() -> Self {
        Self {
            acomp: [Banda::new(), Banda::new()],
            voz: Banda::new(),
            e_acomp: 0.0,
            e_voz: 0.0,
            atenuacion: 0.0,
        }
    }

    /// Suma voz y acompañamiento atenuando la banda de presencia del
    /// acompañamiento donde tapa a la voz.
    fn mezclar(&mut self, a: &mut [f32], b: &mut [f32], va: &[f32], vb: &[f32]) {
        let tasa = TASA as f32;
        let suave = 1.0 - (-1.0 / (0.01 * tasa)).exp();
        let ataque = 1.0 - (-1.0 / (0.06 * tasa)).exp();
        let relaja = 1.0 - (-1.0 / (0.35 * tasa)).exp();
        for i in 0..a.len() {
            let (x, y) = (a[i], b[i]);
            let (ba, bb) = (self.acomp[0].procesar(x), self.acomp[1].procesar(y));
            let v = 0.5 * (va[i] + vb[i]);
            let bv = self.voz.procesar(v);
            self.e_acomp += (0.5 * (ba * ba + bb * bb) - self.e_acomp) * suave;
            self.e_voz += (bv * bv - self.e_voz) * suave;
            // Cuánto supera el acompañamiento a la voz en su banda, en dB:
            // desde −3 dB empieza a taparla; a +6 dB la atenuación es plena.
            let activa = self.e_voz > 1e-7;
            let objetivo = if activa {
                let r = 10.0 * (self.e_acomp.max(1e-12) / self.e_voz.max(1e-12)).log10();
                ((r + 3.0) / 9.0).clamp(0.0, 1.0) * 0.44
            } else {
                0.0
            };
            let k = if objetivo > self.atenuacion {
                ataque
            } else {
                relaja
            };
            self.atenuacion += (objetivo - self.atenuacion) * k;
            a[i] = x - self.atenuacion * ba + va[i];
            b[i] = y - self.atenuacion * bb + vb[i];
        }
    }
}

/// Limitador de pico con anticipo. La ganancia necesaria de cada muestra se
/// lleva al mínimo de su entorno de ±`ANTICIPO` muestras y ese mínimo se
/// promedia sobre otra ventana igual: el promedio nunca supera lo necesario
/// en el pico (todos sus términos son mínimos que lo incluyen) y baja en
/// rampa, sin escalón. Sube con 80 ms de relajación. Todo en O(1) por muestra;
/// la salida va retrasada `2 × ANTICIPO` muestras (6 ms).
const ANTICIPO: usize = 132;
const TECHO_PICO: f32 = 0.891;

struct Anticipo {
    retardo: std::collections::VecDeque<[f32; 2]>,
    /// Mínimo deslizante de lo necesario: (índice, valor), creciente.
    minimos: std::collections::VecDeque<(usize, f32)>,
    /// Los últimos mínimos y su suma, para el promedio.
    medios: std::collections::VecDeque<f32>,
    suma: f64,
    n: usize,
    ganancia: f32,
}

impl Anticipo {
    fn new() -> Self {
        let mut medios = std::collections::VecDeque::with_capacity(2 * ANTICIPO + 2);
        // Antes del principio no hace falta bajar nada.
        medios.extend(std::iter::repeat_n(1.0f32, ANTICIPO));
        Self {
            retardo: std::collections::VecDeque::with_capacity(2 * ANTICIPO + 2),
            minimos: std::collections::VecDeque::new(),
            medios,
            suma: ANTICIPO as f64,
            n: 0,
            ganancia: 1.0,
        }
    }

    fn empujar(&mut self, necesaria: f32) -> Option<[f32; 2]> {
        let n = self.n;
        self.n += 1;
        while self.minimos.back().is_some_and(|(_, v)| *v >= necesaria) {
            self.minimos.pop_back();
        }
        self.minimos.push_back((n, necesaria));
        // Ventana [n − 2A, n]: su mínimo es el de la muestra n − A.
        while self
            .minimos
            .front()
            .is_some_and(|(i, _)| *i + 2 * ANTICIPO < n)
        {
            self.minimos.pop_front();
        }
        if n < ANTICIPO {
            return None;
        }
        let m = self.minimos.front().map_or(1.0, |x| x.1);
        self.medios.push_back(m);
        self.suma += f64::from(m);
        if self.medios.len() > 2 * ANTICIPO + 1 {
            self.suma -= f64::from(self.medios.pop_front().unwrap_or(1.0));
        }
        if n < 2 * ANTICIPO {
            return None;
        }
        let objetivo = (self.suma / self.medios.len() as f64) as f32;
        if objetivo < self.ganancia {
            self.ganancia = objetivo;
        } else {
            self.ganancia += (objetivo - self.ganancia) * (1.0 / (0.08 * TASA as f32));
        }
        let fr = self.retardo.pop_front()?;
        Some([fr[0] * self.ganancia, fr[1] * self.ganancia])
    }

    /// Entra un marco y sale el de hace `2 × ANTICIPO`, si lo hay.
    fn procesar(&mut self, fr: [f32; 2]) -> Option<[f32; 2]> {
        let pico = fr[0].abs().max(fr[1].abs());
        self.retardo.push_back(fr);
        self.empujar(if pico > TECHO_PICO {
            TECHO_PICO / pico
        } else {
            1.0
        })
    }

    /// Lo que queda en el retardo, al final.
    fn vaciar(&mut self) -> Vec<[f32; 2]> {
        let mut v = Vec::with_capacity(self.retardo.len());
        while !self.retardo.is_empty() {
            match self.empujar(1.0) {
                Some(fr) => v.push(fr),
                None => {
                    // Señal más corta que el retardo: sale tal cual.
                    v.extend(self.retardo.drain(..));
                }
            }
        }
        v
    }
}

/// Lo que dura el audio de un plan: el plan más la cola de las capas que
/// terminan en su último compás (como mucho cuatro compases).
#[must_use]
pub fn duracion_seg(plan: &Plan) -> f64 {
    let compas = plan.compas_seg();
    let cola = plan
        .capas
        .iter()
        .filter(|c| c.destino + c.compases >= plan.compases)
        .map(|c| f64::from(c.salida))
        .fold(0.0f64, f64::max)
        .min(4.0);
    (f64::from(plan.compases) + cola) * compas
}

/// Borra el temporal si el render no llega al final.
struct Temporal(PathBuf, bool);

impl Drop for Temporal {
    fn drop(&mut self) {
        if !self.1 {
            let _ = std::fs::remove_file(&self.0);
        }
    }
}

fn igual_potencia(x: f64) -> f32 {
    (x.clamp(0.0, 1.0) * std::f64::consts::FRAC_PI_2).sin() as f32
}

fn sonoridad(a: &[f32], b: &[f32]) -> Option<f64> {
    let mut m = EbuR128::new(2, TASA, Mode::I).ok()?;
    m.add_frames_planar_f32(&[a, b]).ok()?;
    m.loudness_global()
        .ok()
        .filter(|l| l.is_finite() && *l > -70.0)
}

/// El mapa de tiempo que lleva cada pulso real del fragmento `[s0, s1)` a su
/// pulso de la rejilla del destino, en muestras relativas al inicio de la
/// lectura. `None` si los pulsos no cuadran con los compases (un pulso perdido
/// o de más) o algún tramo pide un estirón excesivo: entonces vale el tempo
/// único del fragmento, que al menos acierta en los extremos.
fn mapa_de_pulsos(
    e: &Estructura,
    s0: f64,
    s1: f64,
    compases: usize,
    destino_seg: f64,
    tempo: f64,
) -> Option<Vec<(f64, f64)>> {
    let margen = (s1 - s0) / (8.0 * compases.max(1) as f64);
    let dentro: Vec<f64> = e
        .pulsos
        .iter()
        .copied()
        .filter(|p| *p > s0 + margen && *p < s1 - margen)
        .collect();
    if dentro.len() + 1 != 4 * compases {
        return None;
    }
    // Si los pulsos ya caen a menos de 8 ms de donde los deja el tempo único,
    // el mapa no aporta nada y el tempo único evita vocodificar lo que no
    // cambia de tempo.
    let lineal = (s1 - s0) / (4 * compases) as f64;
    if dentro
        .iter()
        .enumerate()
        .all(|(j, p)| (p - s0 - (j + 1) as f64 * lineal).abs() < 0.008)
    {
        return None;
    }
    let tasa = f64::from(TASA);
    let paso = destino_seg / (4 * compases) as f64;
    let mut mapa = Vec::with_capacity(4 * compases + 2);
    mapa.push((0.0, 0.0));
    for (j, p) in dentro.iter().enumerate() {
        mapa.push(((p - s0) * tasa, (j + 1) as f64 * paso * tasa));
    }
    mapa.push(((s1 - s0) * tasa, destino_seg * tasa));
    // La cola sigue al tempo medio del fragmento.
    mapa.push(((s1 - s0 + 60.0 * tempo) * tasa, (destino_seg + 60.0) * tasa));
    let valido = mapa.windows(2).all(|w| {
        let (de, ds) = (w[1].0 - w[0].0, w[1].1 - w[0].1);
        de > 0.0 && (de / ds - 1.0).abs() <= estiramiento::TEMPO_MAX_TRAMO
    });
    valido.then_some(mapa)
}

/// Prepara una capa: lee, transforma, iguala y funde. Devuelve su primera
/// muestra en el destino y su audio.
fn preparar(
    capa: &Capa,
    estructura: &Estructura,
    plan: &Plan,
    origen: &mut dyn Origen,
    vocoder: &mut Vocoder,
) -> NsResult<(usize, Vec<f32>, Vec<f32>)> {
    let compas = plan.compas_seg();
    let (o, n) = (capa.origen as usize, capa.compases as usize);
    let s0 = *estructura.compases.get(o).ok_or(NsError::InvalidInput)?;
    let s1 = *estructura
        .compases
        .get(o + n)
        .ok_or(NsError::InvalidInput)?;
    let destino_seg = n as f64 * compas;
    let cola_seg = f64::from(capa.salida) * compas;
    // El tempo exacto sale de los compases reales del fragmento, no del BPM
    // global: así una pista que deriva unos milisegundos sigue en la rejilla.
    let tempo = ((s1 - s0) / destino_seg).clamp(1.0 - TEMPO_MAX, 1.0 + TEMPO_MAX);
    let leer_seg = (s1 - s0) + cola_seg * tempo;
    let (a, b) = origen.leer(capa.pista_id, capa.material, s0, leer_seg)?;
    let largo = ((destino_seg + cola_seg) * f64::from(TASA)).round() as usize;
    // Una voz (o una mezcla, que la lleva) transpuesta conserva sus
    // formantes; un instrumental no lo necesita y se ahorra el coste.
    let formantes =
        matches!(capa.material, Material::Voz | Material::Mezcla) && capa.semitonos != 0;
    vocoder.conservar_formantes(formantes.then(|| 2f64.powf(f64::from(capa.semitonos) / 12.0)));
    let transformado = match mapa_de_pulsos(estructura, s0, s1, n, destino_seg, tempo) {
        Some(mapa) => estiramiento::transformar_mapa(vocoder, &a, &b, &mapa, largo, capa.semitonos),
        None => estiramiento::transformar(vocoder, &a, &b, tempo, capa.semitonos),
    };
    vocoder.conservar_formantes(None);
    let (mut a, mut b) = transformado?;
    a.resize(largo, 0.0);
    b.resize(largo, 0.0);
    // Igualar y aplicar la ganancia del papel; nunca más de ±12 dB de ajuste
    // para no levantar el ruido de un fragmento casi mudo.
    let ajuste = sonoridad(&a, &b).map_or(0.0, |l| (CAPA_LUFS - l).clamp(-12.0, 12.0));
    let g = 10f32.powf(((ajuste + f64::from(capa.ganancia_db)) / 20.0) as f32);
    let entrada =
        ((f64::from(capa.entrada) * compas).max(FUNDIDO_MIN_SEG) * f64::from(TASA)) as usize;
    let salida = (cola_seg.max(FUNDIDO_MIN_SEG) * f64::from(TASA)) as usize;
    let fin_nominal = (destino_seg * f64::from(TASA)).round() as usize;
    // La salida empieza en el final nominal y se extiende sobre la cola;
    // sin cola (último fundido mínimo) se come los últimos milisegundos.
    let inicio_salida = if cola_seg > FUNDIDO_MIN_SEG {
        fin_nominal
    } else {
        largo.saturating_sub(salida)
    };
    // Los graves cambian en un dieciseisavo de compás: rápido para que suene a
    // golpe en el primer tiempo, lento para no hacer clic.
    let rampa_graves = ((compas / 16.0) * f64::from(TASA)).max(1.0);
    let filtrada = capa.transicion == Transicion::Filtrada;
    let por_bandas = filtrada || capa.graves_fuera;
    let peso = |i: usize| -> (f32, f32) {
        let mut w = g;
        if i < entrada {
            w *= igual_potencia(i as f64 / entrada as f64);
        }
        if i >= inicio_salida {
            w *= igual_potencia(1.0 - (i - inicio_salida) as f64 / salida.max(1) as f64);
        }
        let mut wg = w;
        if filtrada {
            // Sin graves hasta el final de la entrada; luego, enteros.
            let d = i as f64 - entrada as f64;
            wg = g * (d / rampa_graves).clamp(0.0, 1.0) as f32;
            if i >= inicio_salida {
                wg *= igual_potencia(1.0 - (i - inicio_salida) as f64 / salida.max(1) as f64);
            }
        }
        if capa.graves_fuera && i >= inicio_salida {
            let d = (i - inicio_salida) as f64;
            wg = wg.min(g * (1.0 - d / rampa_graves).clamp(0.0, 1.0) as f32);
        }
        (w, wg)
    };
    if por_bandas {
        let (ga, gb) = (graves(&a), graves(&b));
        for i in 0..a.len() {
            let (w, wg) = peso(i);
            a[i] = (a[i] - ga[i]) * w + ga[i] * wg;
            b[i] = (b[i] - gb[i]) * w + gb[i] * wg;
        }
    } else {
        for (i, (x, y)) in a.iter_mut().zip(b.iter_mut()).enumerate() {
            let (w, _) = peso(i);
            *x *= w;
            *y *= w;
        }
    }
    let inicio = (f64::from(capa.destino) * compas * f64::from(TASA)).round() as usize;
    Ok((inicio, a, b))
}

/// Renderiza `plan` en `destino` (WAV estéreo de 16 bits a 44,1 kHz).
/// `avance` recibe la fracción hecha y devuelve `false` para cancelar.
///
/// # Errors
///
/// [`NsError::Cancelled`]; los de la lectura y la escritura. Nunca deja un
/// archivo a medias en `destino`.
pub fn renderizar(
    plan: &Plan,
    estructuras: &HashMap<i64, Estructura>,
    origen: &mut dyn Origen,
    destino: &Path,
    avance: &mut dyn FnMut(f64) -> bool,
) -> NsResult<Metricas> {
    let reloj = std::time::Instant::now();
    if let Some(dir) = destino.parent() {
        std::fs::create_dir_all(dir).map_err(|_| NsError::StorageFailure)?;
    }
    let crudo = Temporal(destino.with_extension("f32.parcial"), false);
    let total = (duracion_seg(plan) * f64::from(TASA)).round() as usize;
    let mut medidor = EbuR128::new(2, TASA, Mode::I).map_err(|_| NsError::Internal)?;
    let mut escritor =
        BufWriter::new(std::fs::File::create(&crudo.0).map_err(|_| NsError::StorageFailure)?);
    let mut capas: Vec<&Capa> = plan.capas.iter().collect();
    capas.sort_by_key(|c| (c.destino, c.material as u8));
    let mut vocoder = Vocoder::new();
    // Los acumuladores cubren [volcado, volcado + len): el acompañamiento y,
    // aparte, las voces, que se suman al volcar.
    let mut acc: [Vec<f32>; 4] = Default::default();
    let mut volcado = 0usize;
    let mut acumulador_max = 0usize;
    let mut presencia = Presencia::new();
    let mut entrelazado = Vec::with_capacity(BLOQUE * 2);
    let mut volcar = |hasta: usize,
                      acc: &mut [Vec<f32>; 4],
                      volcado: &mut usize,
                      medidor: &mut EbuR128,
                      escritor: &mut BufWriter<std::fs::File>|
     -> NsResult<()> {
        let hasta = hasta.min(total);
        if hasta <= *volcado {
            return Ok(());
        }
        let n = hasta - *volcado;
        for c in acc.iter_mut() {
            if c.len() < n {
                c.resize(n, 0.0);
            }
        }
        {
            let [a, b, va, vb] = acc;
            presencia.mezclar(&mut a[..n], &mut b[..n], &va[..n], &vb[..n]);
        }
        for k in (0..n).step_by(BLOQUE) {
            let m = BLOQUE.min(n - k);
            entrelazado.clear();
            for i in k..k + m {
                entrelazado.push(acc[0][i]);
                entrelazado.push(acc[1][i]);
            }
            medidor
                .add_frames_f32(&entrelazado)
                .map_err(|_| NsError::Internal)?;
            for v in &entrelazado {
                escritor
                    .write_all(&v.to_le_bytes())
                    .map_err(|_| NsError::StorageFailure)?;
            }
        }
        for c in acc.iter_mut() {
            c.drain(..n);
        }
        *volcado = hasta;
        Ok(())
    };
    for (k, capa) in capas.iter().enumerate() {
        if !avance(0.9 * k as f64 / capas.len().max(1) as f64) {
            return Err(NsError::Cancelled);
        }
        let est = estructuras
            .get(&capa.pista_id)
            .ok_or(NsError::InvalidInput)?;
        let (inicio, a, b) = preparar(capa, est, plan, origen, &mut vocoder)?;
        // Todo lo anterior al inicio de esta capa ya no recibirá nada.
        volcar(inicio, &mut acc, &mut volcado, &mut medidor, &mut escritor)?;
        let desde = inicio - volcado.min(inicio);
        let fin = (desde + a.len()).min(total.saturating_sub(volcado));
        for c in acc.iter_mut() {
            if c.len() < fin {
                c.resize(fin, 0.0);
            }
        }
        let k = if capa.material == Material::Voz { 2 } else { 0 };
        for i in desde..fin {
            acc[k][i] += a[i - desde];
            acc[k + 1][i] += b[i - desde];
        }
        acumulador_max = acumulador_max.max(acc[0].len());
    }
    volcar(total, &mut acc, &mut volcado, &mut medidor, &mut escritor)?;
    drop(volcar);
    escritor.flush().map_err(|_| NsError::StorageFailure)?;
    drop(escritor);

    // Segunda pasada: ganancia, limitador y WAV.
    let lufs_crudo = medidor
        .loudness_global()
        .ok()
        .filter(|l| l.is_finite() && *l > -70.0);
    let g = 10f32.powf((lufs_crudo.map_or(0.0, |l| OBJETIVO_LUFS - l) / 20.0) as f32);
    let parcial = Temporal(destino.with_extension("wav.parcial"), false);
    let metricas = escribir_final(&crudo.0, &parcial.0, total, g, avance)?;
    std::fs::rename(&parcial.0, destino).map_err(|_| NsError::StorageFailure)?;
    let mut parcial = parcial;
    parcial.1 = true;
    avance(1.0);
    Ok(Metricas {
        capas: plan.capas.len() as u32,
        segundos_render: reloj.elapsed().as_secs_f64(),
        acumulador_max_seg: acumulador_max as f64 / f64::from(TASA),
        ..metricas
    })
}

fn cabecera_wav(n: usize) -> NsResult<[u8; 44]> {
    let datos = u32::try_from(n * 4).map_err(|_| NsError::Internal)?;
    let mut b = [0u8; 44];
    b[0..4].copy_from_slice(b"RIFF");
    b[4..8].copy_from_slice(&(36 + datos).to_le_bytes());
    b[8..16].copy_from_slice(b"WAVEfmt ");
    b[16..20].copy_from_slice(&16u32.to_le_bytes());
    b[20..22].copy_from_slice(&1u16.to_le_bytes());
    b[22..24].copy_from_slice(&2u16.to_le_bytes());
    b[24..28].copy_from_slice(&TASA.to_le_bytes());
    b[28..32].copy_from_slice(&(TASA * 4).to_le_bytes());
    b[32..34].copy_from_slice(&4u16.to_le_bytes());
    b[34..36].copy_from_slice(&16u16.to_le_bytes());
    b[36..40].copy_from_slice(b"data");
    b[40..44].copy_from_slice(&datos.to_le_bytes());
    Ok(b)
}

fn escribir_final(
    crudo: &Path,
    destino: &Path,
    total: usize,
    g: f32,
    avance: &mut dyn FnMut(f64) -> bool,
) -> NsResult<Metricas> {
    let mut lector =
        BufReader::new(std::fs::File::open(crudo).map_err(|_| NsError::StorageFailure)?);
    lector
        .seek(SeekFrom::Start(0))
        .map_err(|_| NsError::StorageFailure)?;
    let mut f =
        BufWriter::new(std::fs::File::create(destino).map_err(|_| NsError::StorageFailure)?);
    f.write_all(&cabecera_wav(total)?)
        .map_err(|_| NsError::StorageFailure)?;
    let mut medidor =
        EbuR128::new(2, TASA, Mode::I | Mode::TRUE_PEAK).map_err(|_| NsError::Internal)?;
    let mut bytes = vec![0u8; BLOQUE * 8];
    let mut bloque = Vec::with_capacity(BLOQUE * 2);
    let mut medida = Medida::new();
    let mut anticipo = Anticipo::new();
    let mut leidas = 0;
    while leidas < total {
        let m = BLOQUE.min(total - leidas);
        lector
            .read_exact(&mut bytes[..m * 8])
            .map_err(|_| NsError::StorageFailure)?;
        bloque.clear();
        for fr in bytes[..m * 8].as_chunks::<8>().0 {
            let l = f32::from_le_bytes([fr[0], fr[1], fr[2], fr[3]]) * g;
            let r = f32::from_le_bytes([fr[4], fr[5], fr[6], fr[7]]) * g;
            if let Some([x, y]) = anticipo.procesar([l, r]) {
                bloque.push(limitar(x));
                bloque.push(limitar(y));
            }
        }
        medida.escribir(&bloque, &mut medidor, &mut f)?;
        leidas += m;
        if !avance(0.9 + 0.1 * leidas as f64 / total as f64) {
            return Err(NsError::Cancelled);
        }
    }
    bloque.clear();
    for [x, y] in anticipo.vaciar() {
        bloque.push(limitar(x));
        bloque.push(limitar(y));
    }
    medida.escribir(&bloque, &mut medidor, &mut f)?;
    let (pico, recortes, salto_max, silencio_max) = (
        medida.pico,
        medida.recortes,
        medida.salto_max,
        medida.silencio_max,
    );
    let verdadero = (0..2)
        .filter_map(|c| medidor.true_peak(c).ok())
        .fold(0.0f64, f64::max);
    f.flush().map_err(|_| NsError::StorageFailure)?;
    f.get_ref()
        .sync_all()
        .map_err(|_| NsError::StorageFailure)?;
    Ok(Metricas {
        duracion_seg: total as f64 / f64::from(TASA),
        lufs: medidor.loudness_global().unwrap_or(f64::NEG_INFINITY),
        pico_dbfs: 20.0 * f64::from(pico.max(1e-9)).log10(),
        recortes,
        salto_max,
        silencio_max_seg: silencio_max as f64 / f64::from(TASA),
        capas: 0,
        segundos_render: 0.0,
        acumulador_max_seg: 0.0,
        pico_verdadero_dbtp: 20.0 * verdadero.max(1e-9).log10(),
    })
}

/// Lo que se mide mientras se escribe el WAV final.
struct Medida {
    pico: f32,
    recortes: u32,
    salto_max: f64,
    ventana: usize,
    en_ventana: usize,
    energia_ventana: f64,
    silencio: usize,
    silencio_max: usize,
    anterior: [f32; 2],
}

impl Medida {
    fn new() -> Self {
        Self {
            pico: 0.0,
            recortes: 0,
            salto_max: 0.0,
            ventana: (0.05 * f64::from(TASA)) as usize,
            en_ventana: 0,
            energia_ventana: 0.0,
            silencio: 0,
            silencio_max: 0,
            anterior: [0.0; 2],
        }
    }

    fn escribir(
        &mut self,
        bloque: &[f32],
        medidor: &mut EbuR128,
        f: &mut BufWriter<std::fs::File>,
    ) -> NsResult<()> {
        if bloque.is_empty() {
            return Ok(());
        }
        medidor
            .add_frames_f32(bloque)
            .map_err(|_| NsError::Internal)?;
        for fr in bloque.as_chunks::<2>().0 {
            for (c, &v) in fr.iter().enumerate() {
                self.pico = self.pico.max(v.abs());
                if v.abs() >= 0.999 {
                    self.recortes += 1;
                }
                // El salto se mide contra el nivel: un clic es un salto grande
                // donde la señal es pequeña.
                let d = f64::from((v - self.anterior[c]).abs());
                if d > 0.25 {
                    self.salto_max = self.salto_max.max(d);
                }
                self.anterior[c] = v;
                let q = (v.clamp(-1.0, 1.0) * 32_767.0).round() as i16;
                f.write_all(&q.to_le_bytes())
                    .map_err(|_| NsError::StorageFailure)?;
            }
            self.energia_ventana += f64::from(fr[0] * fr[0] + fr[1] * fr[1]) * 0.5;
            self.en_ventana += 1;
            if self.en_ventana == self.ventana {
                let rms = (self.energia_ventana / self.ventana as f64).sqrt();
                if rms < 0.001 {
                    self.silencio += self.ventana;
                    self.silencio_max = self.silencio_max.max(self.silencio);
                } else {
                    self.silencio = 0;
                }
                (self.en_ventana, self.energia_ventana) = (0, 0.0);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod pruebas {
    use super::*;
    use crate::audio::decodificador::Decodificador;
    use crate::mashup::compositor::pruebas::biblioteca;
    use crate::mashup::compositor::{componer, Estilo, Peticion};
    use std::f32::consts::TAU;

    /// Un origen sintético: cada pista es un acorde propio con golpes en cada
    /// pulso a su tempo; cada stem es una parte de la mezcla.
    pub(crate) struct Sintetico {
        pub bpm: HashMap<i64, f64>,
        pub lecturas: u32,
    }

    impl Origen for Sintetico {
        fn leer(
            &mut self,
            pista: i64,
            material: Material,
            desde: f64,
            segundos: f64,
        ) -> NsResult<(Vec<f32>, Vec<f32>)> {
            self.lecturas += 1;
            let bpm = *self.bpm.get(&pista).ok_or(NsError::FileNotFound)?;
            let n = (segundos * f64::from(TASA)) as usize;
            let f0 = 110.0 * 2f32.powf((pista % 12) as f32 / 12.0);
            let v: Vec<f32> = (0..n)
                .map(|i| {
                    let t = desde + i as f64 / f64::from(TASA);
                    let fase = (t * bpm / 60.0).fract() as f32;
                    let golpe = (-fase * 40.0).exp() * (TAU * 60.0 * t as f32).sin() * 0.5;
                    let acorde = [1.0, 1.26, 1.5]
                        .iter()
                        .map(|r| (TAU * f0 * r * t as f32).sin())
                        .sum::<f32>()
                        * 0.1;
                    let voz = (TAU * f0 * 4.0 * t as f32).sin()
                        * 0.15
                        * (1.0 + (TAU * 3.0 * t as f32).sin())
                        * 0.5;
                    match material {
                        Material::Mezcla => golpe + acorde + voz,
                        Material::Instrumental => golpe + acorde,
                        Material::SinBateria => acorde,
                        Material::Bateria => golpe,
                        Material::Voz => voz,
                    }
                })
                .collect();
            Ok((v.clone(), v))
        }
    }

    pub(crate) fn dir(nombre: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ns-render-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn escenario(
        n: usize,
        canciones: u32,
        dur: u32,
        semilla: i64,
    ) -> (Plan, HashMap<i64, Estructura>, Sintetico) {
        let f = biblioteca(n, true);
        let plan = componer(
            &f,
            &Peticion {
                duracion_seg: dur,
                canciones,
                estilo: Estilo::Equilibrado,
                semilla,
            },
        )
        .unwrap();
        let est = f
            .iter()
            .map(|x| (x.pista_id, x.estructura.clone()))
            .collect();
        let bpm = f
            .iter()
            .map(|x| (x.pista_id, x.estructura.bpm.unwrap()))
            .collect();
        (plan, est, Sintetico { bpm, lecturas: 0 })
    }

    fn leer_wav(r: &Path) -> Vec<f32> {
        let mut d = Decodificador::abrir(r, 0.0).unwrap();
        let mut frames = Vec::new();
        while d.leer(&mut frames, 65_536).unwrap() > 0 {}
        frames.iter().map(|f| f.left).collect()
    }

    #[test]
    fn el_render_dura_lo_que_el_plan_y_cumple_sonoridad_picos_y_continuidad() {
        let (plan, est, mut o) = escenario(8, 6, 120, 1);
        let d = dir("basico");
        let destino = d.join("m.wav");
        let m = renderizar(&plan, &est, &mut o, &destino, &mut |_| true).unwrap();
        let x = leer_wav(&destino);
        let esperado = (duracion_seg(&plan) * f64::from(TASA)).round() as usize;
        assert_eq!(x.len(), esperado);
        // −14 LUFS ±1: la tolerancia de un medidor R128 sobre material con
        // fundidos; el limitador puede bajar algo la integrada en los picos.
        assert!((m.lufs - OBJETIVO_LUFS).abs() <= 1.0, "{}", m.lufs);
        assert_eq!(m.recortes, 0);
        assert!(m.pico_dbfs <= -0.0087, "{}", m.pico_dbfs);
        // Ningún silencio accidental de más de 50 ms (una ventana).
        assert!(m.silencio_max_seg <= 0.05, "{}", m.silencio_max_seg);
        // Ningún salto de más de 0,6 entre muestras: el golpe sintético más
        // abrupto sube 0,5 en una muestra; un corte sin fundido saltaría al nivel
        // completo de la mezcla (más de 0,8 tras normalizar).
        assert!(m.salto_max < 0.6, "{}", m.salto_max);
        assert!(!d.join("m.f32.parcial").exists() && !d.join("m.wav.parcial").exists());
    }

    #[test]
    fn la_misma_configuracion_da_el_mismo_archivo_byte_a_byte() {
        let (plan, est, mut o) = escenario(6, 4, 60, 3);
        let d = dir("repro");
        renderizar(&plan, &est, &mut o, &d.join("a.wav"), &mut |_| true).unwrap();
        renderizar(&plan, &est, &mut o, &d.join("b.wav"), &mut |_| true).unwrap();
        assert_eq!(
            std::fs::read(d.join("a.wav")).unwrap(),
            std::fs::read(d.join("b.wav")).unwrap()
        );
    }

    #[test]
    fn cancelar_no_deja_archivos_ni_temporales() {
        let (plan, est, mut o) = escenario(6, 4, 60, 3);
        let d = dir("cancelar");
        let mut llamadas = 0;
        let r = renderizar(&plan, &est, &mut o, &d.join("x.wav"), &mut |_| {
            llamadas += 1;
            llamadas < 3
        });
        assert_eq!(r, Err(NsError::Cancelled));
        assert_eq!(std::fs::read_dir(&d).unwrap().count(), 0);
    }

    #[test]
    fn un_fallo_de_lectura_no_deja_archivos_ni_temporales() {
        let (plan, est, mut o) = escenario(6, 4, 60, 3);
        o.bpm.clear();
        let d = dir("fallo");
        let r = renderizar(&plan, &est, &mut o, &d.join("x.wav"), &mut |_| true);
        assert_eq!(r, Err(NsError::FileNotFound));
        assert_eq!(std::fs::read_dir(&d).unwrap().count(), 0);
    }

    #[test]
    fn las_capas_caen_en_su_compas_y_con_su_ganancia() {
        // Una sola capa de batería en el compás 2 de un plan de 4: el golpe
        // debe sonar en los pulsos del destino y el resto debe ser silencio.
        let f = biblioteca(1, true);
        let est: HashMap<i64, Estructura> = f
            .iter()
            .map(|x| (x.pista_id, x.estructura.clone()))
            .collect();
        let bpm = f[0].estructura.bpm.unwrap();
        let plan = Plan {
            version: 1,
            bpm,
            tonica: 0,
            menor: false,
            compases: 4,
            capas: vec![Capa {
                pista_id: f[0].pista_id,
                material: Material::Bateria,
                origen: 8,
                compases: 1,
                destino: 2,
                tempo: 1.0,
                semitonos: 0,
                ganancia_db: 0.0,
                entrada: 0.0,
                salida: 0.0,
                transicion: Transicion::Cruce,
                graves_fuera: false,
            }],
            degradaciones: vec![],
            calidad: 1.0,
        };
        let mut o = Sintetico {
            bpm: [(f[0].pista_id, bpm)].into(),
            lecturas: 0,
        };
        let d = dir("compas");
        let m = renderizar(&plan, &est, &mut o, &d.join("x.wav"), &mut |_| true).unwrap();
        let x = leer_wav(&d.join("x.wav"));
        let compas = (plan.compas_seg() * f64::from(TASA)) as usize;
        let antes = x[..2 * compas - 500]
            .iter()
            .fold(0.0f32, |a, v| a.max(v.abs()));
        let despues = x[3 * compas + 500..]
            .iter()
            .fold(0.0f32, |a, v| a.max(v.abs()));
        assert!(antes == 0.0 && despues == 0.0, "{antes} {despues}");
        // Cada pulso del compás 2 tiene su golpe a menos de 10 ms de su sitio.
        let pulso = compas / 4;
        for k in 0..4 {
            let c = 2 * compas + k * pulso;
            let ventana = &x[c.saturating_sub(441)..c + 2205];
            let max_en = ventana.iter().fold(0.0f32, |a, v| a.max(v.abs()));
            assert!(max_en > 0.1, "pulso {k}: {max_en}");
        }
        assert!(m.duracion_seg > 0.0);
    }

    #[test]
    fn la_memoria_del_render_no_crece_con_la_duracion() {
        // Cada capa se lee una vez y el acumulador nunca pasa de las capas
        // simultáneas.
        let (plan, est, mut o) = escenario(20, 16, 360, 5);
        let d = dir("largo");
        let m = renderizar(&plan, &est, &mut o, &d.join("x.wav"), &mut |_| true).unwrap();
        assert_eq!(o.lecturas as usize, plan.capas.len());
        assert!(m.duracion_seg > 300.0);
        // Dos frases de 16 compases a 120 BPM son 64 s: el acumulador nunca
        // abarca más que las capas solapadas, por larga que sea la mezcla.
        assert!(m.acumulador_max_seg < 70.0, "{}", m.acumulador_max_seg);
        assert!(m.acumulador_max_seg < m.duracion_seg / 4.0);
    }

    #[test]
    fn los_pulsos_que_vacilan_dan_un_mapa_y_los_regulares_o_incompletos_no() {
        let f = biblioteca(1, true);
        let mut e = f[0].estructura.clone();
        let (s0, s1) = (e.compases[8], e.compases[10]);
        let destino = s1 - s0;
        assert!(
            mapa_de_pulsos(&e, s0, s1, 2, destino, 1.0).is_none(),
            "regular"
        );
        let primero = e.pulsos.iter().position(|p| *p > s0 + 0.01).unwrap();
        e.pulsos[primero + 2] += 0.03;
        let mapa = mapa_de_pulsos(&e, s0, s1, 2, destino, 1.0).expect("vacilante");
        assert_eq!(mapa.len(), 4 * 2 + 2);
        assert!(mapa.windows(2).all(|w| w[1].0 > w[0].0 && w[1].1 > w[0].1));
        e.pulsos.remove(primero + 4);
        assert!(
            mapa_de_pulsos(&e, s0, s1, 2, destino, 1.0).is_none(),
            "incompleto"
        );
    }

    #[test]
    fn el_limitador_con_anticipo_deja_los_picos_en_su_techo_sin_escalones() {
        let mut l = Anticipo::new();
        let n = 20_000;
        let x: Vec<f32> = (0..n)
            .map(|i| {
                // Un golpe que sube en 2 ms: la entrada no tiene escalones.
                let d = (i as f32 - 5000.0).abs();
                let golpe = 0.4 + 1.2 * (1.0 - (d / 90.0).min(1.0));
                golpe * (i as f32 * 0.05).sin()
            })
            .collect();
        let mut y = Vec::new();
        for v in &x {
            if let Some(fr) = l.procesar([*v, *v]) {
                y.push(fr[0]);
            }
        }
        y.extend(l.vaciar().iter().map(|f| f[0]));
        assert_eq!(y.len(), n);
        let pico = y.iter().fold(0.0f32, |a, v| a.max(v.abs()));
        assert!(pico <= TECHO_PICO + 1e-4, "{pico}");
        // Lejos del golpe la señal sale intacta.
        // Antes del golpe la señal sale intacta y en su sitio; mucho después,
        // la ganancia ya ha vuelto (relajación de 80 ms).
        assert!((y[100] - x[100]).abs() < 1e-6, "{} {}", y[100], x[100]);
        assert!(
            (y[n - 1] - x[n - 1]).abs() < 0.01,
            "{} {}",
            y[n - 1],
            x[n - 1]
        );
        let salto = y
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0f32, f32::max);
        assert!(salto < 0.12, "{salto}");
    }
}
