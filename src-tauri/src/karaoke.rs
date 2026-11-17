//! El karaoke: separar la voz con `htdemucs` en ONNX (ADR-002).
//!
//! El algoritmo es el de `_separar_por_segmentos`, el mismo que se midió contra
//! PyTorch: la mezcla se normaliza por su media y su desviación, se recorre en
//! segmentos de 7,8 s (la entrada fija del grafo exportado) con un 25 % de
//! solapamiento y ventana triangular, y se deshace la normalización. El
//! instrumental es la mezcla menos la voz. Solo se acumula la voz:
//! los otros tres stems no se usan y ocuparían otro tanto de memoria.
//!
//! **Memoria.** Medido: ONNX Runtime llegaba a 9 GB con su configuración
//! por defecto. Aquí la sesión se crea una vez por pista, sin el *arena* del
//! proveedor de CPU, sin el patrón de memoria y sin optimizar el grafo (lo que
//! más pesaba: ver [`sesion`]), con un cuarto de los núcleos: el audio corre en su
//! propio hilo de tiempo real y la ventana no se nota.
//!
//! **Formato.** El instrumental se guarda en FLAC de 16 bits, el mismo codificador
//! que la caché de stems de los mashups. Sin `ffmpeg` (ADR-006) no hay codificador
//! de MP3; un WAV ocupaba unas cuatro veces lo que el MP3 de origen, y el FLAC, sin
//! pérdida, más o menos la mitad que el WAV. NB Sound Mobile lo recibe y lo
//! reproduce tal cual.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use rubato::{FftFixedIn, Resampler};
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use ts_rs::TS;

use crate::agregados::{self, IdAgregado};
use crate::audio::decodificador::Decodificador;
use crate::db::{self, BaseDeDatos};
use crate::error::{NsError, NsResult};
use crate::paths::Rutas;
use crate::reproductor::persistencia;

/// La tasa del modelo.
pub const TASA: u32 = 44_100;
/// 7,8 s a 44,1 kHz: la entrada fija del grafo exportado.
pub const SEGMENTO: usize = 343_980;
/// El modelo dentro del agregado.
pub const ARCHIVO_MODELO: &str = "htdemucs.onnx";
/// Lo que se apunta en `stems.modelo`.
pub const MODELO: &str = "htdemucs-onnx-fp16";
/// El índice de la voz entre los cuatro stems (batería, bajo, otros, voz).
const VOZ: usize = 3;

// --- El algoritmo ---------------------------------------------------------------------

/// La ventana triangular de demucs, normalizada a 1 en su máximo.
fn ventana() -> Vec<f32> {
    let mitad = SEGMENTO / 2;
    let mut w: Vec<f32> = (1..=mitad)
        .chain((1..=SEGMENTO - mitad).rev())
        .map(|v| v as f32)
        .collect();
    let max = w.iter().fold(0.0f32, |m, &v| m.max(v));
    for v in &mut w {
        *v /= max;
    }
    w
}

/// El orden de los stems a la salida del modelo.
pub const STEMS: [Stem; 4] = [Stem::Bateria, Stem::Bajo, Stem::Otros, Stem::Voz];

/// Uno de los cuatro componentes de HTDemucs (ADR-018).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stem {
    Bateria,
    Bajo,
    Otros,
    Voz,
}

impl Stem {
    /// Su posición en la salida del modelo.
    #[must_use]
    pub fn indice(self) -> usize {
        match self {
            Self::Bateria => 0,
            Self::Bajo => 1,
            Self::Otros => 2,
            Self::Voz => VOZ,
        }
    }

    /// El nombre estable con que se guarda en caché y en la base.
    #[must_use]
    pub fn clave(self) -> &'static str {
        match self {
            Self::Bateria => "bateria",
            Self::Bajo => "bajo",
            Self::Otros => "otros",
            Self::Voz => "voz",
        }
    }

    /// El inverso de [`Stem::clave`].
    #[must_use]
    pub fn desde_clave(c: &str) -> Option<Self> {
        STEMS.into_iter().find(|s| s.clave() == c)
    }
}

/// Un stem estéreo planar ya desnormalizado.
pub type Estereo = (Vec<f32>, Vec<f32>);

/// Separa de una mezcla estéreo planar los stems marcados en `pedidos`, en el
/// orden de [`STEMS`]; los no pedidos vuelven como `None` y no ocupan memoria.
/// Es el único recorrido de HTDemucs del programa: el karaoke y los mashups
/// lo comparten (ADR-018) para no tener dos implementaciones que diverjan.
///
/// # Errors
///
/// Los de `modelo`; [`NsError::Cancelled`] si `seguir` lo pide;
/// [`NsError::Internal`] si el modelo devuelve otra forma;
/// [`NsError::AudioCorrupted`] si la mezcla está vacía.
pub fn separar_stems(
    izq: &[f32],
    der: &[f32],
    pedidos: [bool; 4],
    modelo: &mut dyn FnMut(&[f32]) -> NsResult<Vec<f32>>,
    seguir: &mut dyn FnMut(f64) -> bool,
) -> NsResult<[Option<Estereo>; 4]> {
    let largo = izq.len().min(der.len());
    if largo == 0 {
        return Err(NsError::AudioCorrupted);
    }
    // La referencia es la media de los dos canales, como `mezcla.mean(0)`.
    let n = largo as f64;
    let media = (0..largo)
        .map(|i| f64::from(izq[i] + der[i]) * 0.5)
        .sum::<f64>()
        / n;
    // `torch.std` es la muestral.
    let varianza = (0..largo)
        .map(|i| (f64::from(izq[i] + der[i]) * 0.5 - media).powi(2))
        .sum::<f64>()
        / (n - 1.0).max(1.0);
    let desv = varianza.sqrt() + 1e-8;
    let (media, desv) = (media as f32, desv as f32);
    let peso = ventana();
    let paso = ((0.75 * SEGMENTO as f64) as usize).max(1);
    let mut acum: [Option<[Vec<f32>; 2]>; 4] =
        std::array::from_fn(|k| pedidos[k].then(|| [vec![0.0f32; largo], vec![0.0f32; largo]]));
    let mut total = vec![0.0f32; largo];
    let mut trozo = vec![0.0f32; 2 * SEGMENTO];
    let inicios: Vec<usize> = (0..largo).step_by(paso).collect();
    for (k, &inicio) in inicios.iter().enumerate() {
        if !seguir(k as f64 / inicios.len() as f64) {
            return Err(NsError::Cancelled);
        }
        let m = SEGMENTO.min(largo - inicio);
        trozo.iter_mut().for_each(|v| *v = 0.0);
        for t in 0..m {
            trozo[t] = (izq[inicio + t] - media) / desv;
            trozo[SEGMENTO + t] = (der[inicio + t] - media) / desv;
        }
        let stems = modelo(&trozo)?;
        if stems.len() != 4 * 2 * SEGMENTO {
            tracing::error!(valores = stems.len(), "el modelo devolvió otra forma");
            return Err(NsError::Internal);
        }
        for (s, canales) in acum.iter_mut().enumerate() {
            let Some(canales) = canales else { continue };
            for (c, v) in canales.iter_mut().enumerate() {
                let base = (s * 2 + c) * SEGMENTO;
                for t in 0..m {
                    v[inicio + t] += stems[base + t] * peso[t];
                }
            }
        }
        for t in 0..m {
            total[inicio + t] += peso[t];
        }
    }
    seguir(1.0);
    // Se deshace la normalización en el sitio, sin copiar la pista.
    Ok(acum.map(|c| {
        c.map(|[mut a, mut b]| {
            for v in [&mut a, &mut b] {
                for (i, x) in v.iter_mut().enumerate() {
                    *x = (*x / total[i].max(1e-8)) * desv + media;
                }
            }
            (a, b)
        })
    }))
}

/// Separa la voz de una mezcla estéreo planar y devuelve el instrumental.
/// `modelo` recibe un segmento `[1, 2, SEGMENTO]` aplanado (izquierda y luego
/// derecha) y devuelve los cuatro stems `[1, 4, 2, SEGMENTO]` aplanados.
/// `seguir` recibe la fracción hecha y devuelve `false` para cancelar.
///
/// # Errors
///
/// Los de [`separar_stems`].
pub fn separar(
    izq: &[f32],
    der: &[f32],
    modelo: &mut dyn FnMut(&[f32]) -> NsResult<Vec<f32>>,
    seguir: &mut dyn FnMut(f64) -> bool,
) -> NsResult<(Vec<f32>, Vec<f32>)> {
    let mut pedidos = [false; 4];
    pedidos[VOZ] = true;
    let [_, _, _, voz] = separar_stems(izq, der, pedidos, modelo, seguir)?;
    let (mut a, mut b) = voz.ok_or(NsError::Internal)?;
    // El instrumental es la mezcla menos la voz, con la misma aritmética de
    // siempre: el resultado del karaoke no cambia en un solo bit.
    for (v, mezcla) in [(&mut a, izq), (&mut b, der)] {
        for (x, m) in v.iter_mut().zip(mezcla) {
            *x = m - *x;
        }
    }
    Ok((a, b))
}

// --- Audio de entrada y salida ---------------------------------------------------------

/// La pista entera en estéreo planar a 44,1 kHz.
pub(crate) fn cargar(ruta: &Path) -> NsResult<(Vec<f32>, Vec<f32>)> {
    let mut d = Decodificador::abrir(ruta, 0.0)?;
    let origen = d.tasa;
    let mut frames = Vec::new();
    while d.leer(&mut frames, 65_536)? > 0 {}
    let izq: Vec<f32> = frames.iter().map(|f| f.left).collect();
    let der: Vec<f32> = frames.iter().map(|f| f.right).collect();
    drop(frames);
    if origen == TASA {
        return Ok((izq, der));
    }
    remuestrear(&izq, &der, origen)
}

/// Lleva el audio a la tasa del modelo.
pub(crate) fn remuestrear(izq: &[f32], der: &[f32], origen: u32) -> NsResult<(Vec<f32>, Vec<f32>)> {
    let trozo = 4096;
    let mut rs = FftFixedIn::<f32>::new(origen as usize, TASA as usize, trozo, 2, 2)
        .map_err(|_| NsError::Internal)?;
    let esperado = (izq.len() as f64 * f64::from(TASA) / f64::from(origen)).round() as usize;
    let (mut a, mut b) = (Vec::with_capacity(esperado), Vec::with_capacity(esperado));
    let mut i = 0;
    while i + trozo <= izq.len() {
        let out = rs
            .process(&[&izq[i..i + trozo], &der[i..i + trozo]], None)
            .map_err(|_| NsError::Internal)?;
        a.extend_from_slice(&out[0]);
        b.extend_from_slice(&out[1]);
        i += trozo;
    }
    // Si el audio mide un número exacto de trozos no queda resto, y el
    // remuestreador rechaza un trozo parcial vacío: "Cooler Than Me" (48 kHz,
    // 2.670 trozos justos) fallaba siempre con INTERNAL.
    if i < izq.len() {
        let resto = rs
            .process_partial(Some(&[&izq[i..], &der[i..]]), None)
            .map_err(|_| NsError::Internal)?;
        a.extend_from_slice(&resto[0]);
        b.extend_from_slice(&resto[1]);
    }
    let cola = rs
        .process_partial::<&[f32]>(None, None)
        .map_err(|_| NsError::Internal)?;
    a.extend_from_slice(&cola[0]);
    b.extend_from_slice(&cola[1]);
    let retardo = rs.output_delay();
    Ok((
        a.into_iter().skip(retardo).take(esperado).collect(),
        b.into_iter().skip(retardo).take(esperado).collect(),
    ))
}

/// WAV PCM de 16 bits, estéreo, a 44,1 kHz. Solo lo usan las pruebas, para
/// fabricar audio de entrada sin depender de archivos del repositorio.
#[cfg(test)]
pub(crate) fn escribir_wav(destino: &Path, izq: &[f32], der: &[f32]) -> NsResult<u64> {
    use std::io::Write as _;
    let n = izq.len().min(der.len());
    let datos = u32::try_from(n * 4).map_err(|_| NsError::Internal)?;
    let mut b = Vec::with_capacity(44 + n * 4);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + datos).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&TASA.to_le_bytes());
    b.extend_from_slice(&(TASA * 4).to_le_bytes());
    b.extend_from_slice(&4u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&datos.to_le_bytes());
    let muestra = |v: f32| ((v.clamp(-1.0, 1.0) * 32_767.0).round() as i16).to_le_bytes();
    for i in 0..n {
        b.extend_from_slice(&muestra(izq[i]));
        b.extend_from_slice(&muestra(der[i]));
    }
    if let Some(dir) = destino.parent() {
        std::fs::create_dir_all(dir).map_err(|_| NsError::StorageFailure)?;
    }
    let temporal = destino.with_extension("wav.parcial");
    let mut f = std::fs::File::create(&temporal).map_err(|_| NsError::StorageFailure)?;
    f.write_all(&b).map_err(|_| NsError::StorageFailure)?;
    f.sync_all().map_err(|_| NsError::StorageFailure)?;
    std::fs::rename(&temporal, destino).map_err(|_| NsError::StorageFailure)?;
    Ok(b.len() as u64)
}

/// El instrumental en FLAC, escrito en un temporal y colocado al terminar: una
/// separación cortada a la mitad nunca deja un archivo que parezca bueno.
pub(crate) fn escribir_instrumental(destino: &Path, izq: &[f32], der: &[f32]) -> NsResult<u64> {
    if let Some(dir) = destino.parent() {
        std::fs::create_dir_all(dir).map_err(|_| NsError::StorageFailure)?;
    }
    let temporal = destino.with_extension("flac.parcial");
    let bytes = crate::mashup::stems::escribir_flac(&temporal, izq, der)?;
    std::fs::rename(&temporal, destino).map_err(|_| NsError::StorageFailure)?;
    Ok(bytes)
}

// --- El modelo -------------------------------------------------------------------------

/// El modelo del agregado, si está instalado.
#[must_use]
pub fn modelo_instalado(rutas: &Rutas) -> Option<PathBuf> {
    agregados::instalado(rutas, IdAgregado::Karaoke)?;
    let m = agregados::carpeta(rutas, IdAgregado::Karaoke).join(ARCHIVO_MODELO);
    m.is_file().then_some(m)
}

/// Hilos de la cola del karaoke: un cuarto de los núcleos. Con la mitad y el
/// giro de ONNX Runtime activo, preparar una canción tenía cuatro núcleos al
/// 97 % en un equipo de ocho, y todo lo demás (la interfaz incluida) iba más
/// lento. Tarda algo más; no se nota al usar el equipo.
#[must_use]
pub fn hilos_cola() -> usize {
    crate::onnx::hilos_de_fondo(4)
}

fn sesion(modelo: &Path, hilos: usize) -> NsResult<ort::session::Session> {
    let entorno = crate::onnx::entorno()?;
    // Los hilos los decide quien llama ([`hilos_cola`] en la aplicación; las
    // mediciones prueban otros). Medido el 2026-11-15 con una canción de 164 s:
    // 2 hilos, 0,38 veces el tiempo real; 4 hilos, 0,37. No escala con los
    // núcleos, así que un mashup tampoco pide más: lo que acelera es separar
    // menos audio (los tramos de `mashup::stems`).
    let fallo = |_| {
        tracing::error!("no se pudo preparar el modelo de karaoke");
        NsError::ComponentUnavailable
    };
    ort::session::Session::builder(entorno)
        .map_err(|_| NsError::ComponentUnavailable)?
        .with_execution_providers(crate::onnx::proveedores(
            ort::ep::CPU::default().with_arena_allocator(false).build(),
        ))
        .map_err(fallo)?
        .with_memory_pattern(false)
        .map_err(fallo)?
        .with_intra_threads(hilos)
        .map_err(fallo)?
        .with_intra_op_spinning(false)
        .map_err(fallo)?
        .with_inter_op_spinning(false)
        .map_err(fallo)?
        // Sin optimizar el grafo (medido el 2026-11-09 con 60 s de una canción
        // real): con `All` o `Level1` el pico era de 6,6 GB, porque el plegado de
        // constantes materializa tensores enormes de htdemucs; sin optimizar baja a
        // 1,3 GB y el instrumental es el mismo (diferencia máxima de 1 en 32.767,
        // 102 dB de relación señal/ruido). Cuesta en torno a un 15 % más de tiempo.
        // Ni limitar los *arenas* de malloc ni un solo hilo bajaban el pico.
        .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Disable)
        .map_err(fallo)?
        .commit_from_file(modelo)
        .map_err(|_| NsError::ComponentUnavailable)
}

/// Separa una pista y escribe su instrumental.
///
/// # Errors
///
/// Los de la decodificación, del modelo y de la escritura; [`NsError::Cancelled`].
pub fn separar_archivo(
    modelo: &Path,
    audio: &Path,
    destino: &Path,
    seguir: &mut dyn FnMut(f64) -> bool,
) -> NsResult<u64> {
    let (izq, der) = cargar(audio)?;
    let mut s = sesion(modelo, hilos_cola())?;
    let (a, b) = separar(&izq, &der, &mut |trozo| inferir(&mut s, trozo), seguir)?;
    escribir_instrumental(destino, &a, &b)
}

/// Prepara el instrumental de la cola del karaoke desde la caché de stems
/// compartida con los mashups: si la canción entera ya está separada, no se
/// pasa el modelo; si no, se separan los cuatro stems (el modelo los calcula
/// todos de todos modos), se guardan en la caché y el instrumental sale de
/// ellos. Así una canción se separa una sola vez para el karaoke y los mashups.
///
/// Separado aquí, el instrumental es idéntico bit a bit al de siempre. Leído de
/// la caché, la voz viene cuantizada a 16 bits y el instrumental difiere en
/// menos de una unidad de 16 bits: lo mismo que pierde el FLAC de 16 bits que se escribe.
///
/// # Errors
///
/// Los de la decodificación, del modelo y de la escritura; [`NsError::Cancelled`].
pub fn preparar_instrumental(
    rutas: &Rutas,
    modelo: &Path,
    audio: &Path,
    sha: Option<&str>,
    destino: &Path,
    seguir: &mut dyn FnMut(f64) -> bool,
) -> NsResult<u64> {
    use crate::mashup::stems::{self, CacheStems};
    let (izq, der) = cargar(audio)?;
    let sha = match sha {
        Some(s) => s.to_owned(),
        None => crate::mashup::servicio::sha_de(audio)?,
    };
    let cache = CacheStems::compartida(crate::mashup::servicio::carpeta_stems(rutas))?;
    let _p = stems::proteger(&sha);
    let de_cache = cache
        .completa(&sha)
        .then(|| cache.ruta(&sha, Stem::Voz))
        .flatten()
        .and_then(|r| crate::mashup::servicio::leer_todo(&r).ok())
        .filter(|v| v.0.len() == izq.len());
    let (mut a, mut b) = if let Some(v) = de_cache {
        seguir(1.0);
        v
    } else {
        let mut s = sesion(modelo, hilos_cola())?;
        let mut todos = separar_stems(
            &izq,
            &der,
            [true; 4],
            &mut |trozo| inferir(&mut s, trozo),
            seguir,
        )?;
        drop(s);
        // Si la caché no se puede escribir, el karaoke sale igual.
        if cache.guardar(&sha, &todos).is_err() {
            tracing::warn!("no se pudieron guardar los stems de una pista");
        }
        todos[VOZ].take().ok_or(NsError::Internal)?
    };
    for (v, mezcla) in [(&mut a, &izq), (&mut b, &der)] {
        for (x, m) in v.iter_mut().zip(mezcla.iter()) {
            *x = m - *x;
        }
    }
    let bytes = escribir_instrumental(destino, &a, &b)?;
    cache.podar(&std::collections::HashSet::new());
    Ok(bytes)
}

/// Separa una pista y devuelve los stems pedidos en memoria, para que quien los
/// pida (la caché de mashups) decida cómo guardarlos.
///
/// # Errors
///
/// Los de la decodificación y del modelo; [`NsError::Cancelled`].
pub fn separar_archivo_stems(
    modelo: &Path,
    audio: &Path,
    pedidos: [bool; 4],
    seguir: &mut dyn FnMut(f64) -> bool,
) -> NsResult<[Option<Estereo>; 4]> {
    let (izq, der) = cargar(audio)?;
    separar_archivo_stems_de(modelo, &izq, &der, pedidos, hilos_cola(), seguir)
}

/// Como [`separar_archivo_stems`], con la mezcla ya cargada (quien la pide la
/// necesita también para otra cosa y no la decodifica dos veces).
///
/// # Errors
///
/// Los del modelo; [`NsError::Cancelled`].
pub fn separar_archivo_stems_de(
    modelo: &Path,
    izq: &[f32],
    der: &[f32],
    pedidos: [bool; 4],
    hilos: usize,
    seguir: &mut dyn FnMut(f64) -> bool,
) -> NsResult<[Option<Estereo>; 4]> {
    let mut s = sesion(modelo, hilos)?;
    separar_stems(
        izq,
        der,
        pedidos,
        &mut |trozo| inferir(&mut s, trozo),
        seguir,
    )
}

fn inferir(s: &mut ort::session::Session, trozo: &[f32]) -> NsResult<Vec<f32>> {
    let entrada = ort::value::Tensor::from_array(([1usize, 2, SEGMENTO], trozo.to_vec()))
        .map_err(|_| NsError::Internal)?;
    let salidas = s
        .run(ort::inputs![entrada])
        .map_err(|_| NsError::Internal)?;
    let (_, datos) = salidas[0]
        .try_extract_tensor::<f32>()
        .map_err(|_| NsError::Internal)?;
    Ok(datos.to_vec())
}

/// Diez segundos con contenido en todo el espectro (`_generar_audio_sintetico`):
/// grave, medio y agudo modulados, con un poco de ruido para que el
/// modelo no tenga un silencio fácil, y el canal derecho 30 muestras detrás.
fn audio_de_prueba() -> (Vec<f32>, Vec<f32>) {
    use std::f64::consts::TAU;
    let n = TASA as usize * 10;
    let mut x = 42u64;
    let mut ruido = move || {
        // Un generador lineal congruente basta: solo tiene que no ser silencio.
        x = x.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        ((x >> 40) as f64 / f64::from(1u32 << 24) - 0.5) * 0.04
    };
    let izq: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f64 / f64::from(TASA);
            let base = 0.30 * (TAU * 80.0 * t).sin()
                + 0.25 * (TAU * 440.0 * t).sin()
                + 0.20 * (TAU * 3000.0 * t).sin();
            (base * (0.5 + 0.5 * (TAU * 0.5 * t).sin()) + ruido()) as f32
        })
        .collect();
    let mut der = izq.clone();
    der.rotate_right(30);
    (izq, der)
}

/// Lo que tarda `modelo` en separar diez segundos, sin contar la carga.
///
/// # Errors
///
/// Los del modelo.
pub fn medir_con(
    modelo: &mut dyn FnMut(&[f32]) -> NsResult<Vec<f32>>,
) -> NsResult<std::time::Duration> {
    let (izq, der) = audio_de_prueba();
    let inicio = std::time::Instant::now();
    separar(&izq, &der, modelo, &mut |_| true)?;
    Ok(inicio.elapsed())
}

/// [`medir_con`] con el modelo del agregado.
///
/// # Errors
///
/// [`NsError::ComponentUnavailable`] si el modelo no se puede cargar; los del modelo.
pub fn medir(modelo: &Path) -> NsResult<std::time::Duration> {
    let mut s = sesion(modelo, hilos_cola())?;
    medir_con(&mut |trozo| inferir(&mut s, trozo))
}

// --- La cola en segundo plano ----------------------------------------------------------

/// Lo que ve la interfaz: qué pista se está preparando y cuánto lleva.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Preparando {
    #[ts(type = "number")]
    pub pista: i64,
    pub fraccion: f64,
}

/// Pide preparar una pista: la pone en la cola si no tiene ya fila. Si estaba
/// fallida, vuelve a intentarlo.
///
/// # Errors
///
/// [`NsError::StorageFailure`] si la escritura falla.
pub fn pedir(db: &BaseDeDatos, pistas: &[i64]) -> NsResult<()> {
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    for p in pistas {
        tx.execute(
            "INSERT INTO stems (pista_id, estado) VALUES (?1, 'pendiente')
             ON CONFLICT (pista_id) DO UPDATE SET estado = 'pendiente', error = NULL,
                 intentos = 0, solicitada_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE stems.estado = 'fallida'",
            [p],
        )
        .map_err(|_| NsError::StorageFailure)?;
    }
    tx.commit().map_err(|_| NsError::StorageFailure)
}

/// "Preparar mis favoritas": las favoritas que no tienen instrumental
/// ni lo esperan van a la cola. Devuelve cuántas entraron.
///
/// # Errors
///
/// [`NsError::StorageFailure`] si la escritura falla.
pub fn pedir_favoritas(db: &BaseDeDatos) -> NsResult<u32> {
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    let nuevas = tx
        .execute(
            "INSERT INTO stems (pista_id, estado)
             SELECT p.id, 'pendiente' FROM pistas p
             WHERE p.favorita = 1 AND p.estado = 'biblioteca'
               AND NOT EXISTS (SELECT 1 FROM stems s WHERE s.pista_id = p.id)",
            [],
        )
        .map_err(|_| NsError::StorageFailure)?;
    let reintentadas = tx
        .execute(
            "UPDATE stems SET estado = 'pendiente', error = NULL, intentos = 0
             WHERE estado = 'fallida'
               AND pista_id IN (SELECT id FROM pistas WHERE favorita = 1 AND estado = 'biblioteca')",
            [],
        )
        .map_err(|_| NsError::StorageFailure)?;
    tx.commit().map_err(|_| NsError::StorageFailure)?;
    Ok(u32::try_from(nuevas + reintentadas).unwrap_or(u32::MAX))
}

/// "Preparar una playlist": como las favoritas, pero con las pistas de una
/// playlist (manual, inteligente o automática). Lo que ya está listo o en cola
/// no cuenta; lo fallido vuelve a intentarse. Devuelve cuántas entraron.
///
/// # Errors
///
/// Los de leer la playlist; [`NsError::StorageFailure`] si la escritura falla.
pub fn pedir_playlist(db: &BaseDeDatos, playlist: i64) -> NsResult<u32> {
    let ids = crate::playlists::ids(db, playlist)?;
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    let mut n = 0usize;
    for p in ids {
        n += tx
            .execute(
                "INSERT INTO stems (pista_id, estado) VALUES (?1, 'pendiente')
                 ON CONFLICT (pista_id) DO UPDATE SET estado = 'pendiente', error = NULL,
                     intentos = 0, solicitada_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                 WHERE stems.estado = 'fallida'",
                [p],
            )
            .map_err(|_| NsError::StorageFailure)?;
    }
    tx.commit().map_err(|_| NsError::StorageFailure)?;
    Ok(u32::try_from(n).unwrap_or(u32::MAX))
}

/// La próxima pendiente, por orden de petición, con su archivo y su huella.
fn siguiente(db: &BaseDeDatos) -> NsResult<Option<(i64, PathBuf, Option<String>)>> {
    let conn = db.conexion()?;
    let musica = persistencia::carpeta_musica(&conn)?;
    let fila: Option<(i64, String, Option<String>)> = conn
        .query_row(
            "SELECT s.pista_id, p.ruta, p.sha256 FROM stems s JOIN pistas p ON p.id = s.pista_id
             WHERE s.estado = 'pendiente' ORDER BY s.solicitada_en, s.pista_id LIMIT 1",
            [],
            |f| Ok((f.get(0)?, f.get(1)?, f.get(2)?)),
        )
        .optional()
        .map_err(|_| NsError::StorageFailure)?;
    Ok(fila.map(|(id, ruta, sha)| {
        let r = PathBuf::from(ruta);
        let r = match &musica {
            Some(m) if r.is_relative() => m.join(r),
            _ => r,
        };
        (id, r, sha)
    }))
}

fn marcar(db: &BaseDeDatos, sql: &str, p: impl rusqlite::Params) -> NsResult<()> {
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    tx.execute(sql, p).map_err(|_| NsError::StorageFailure)?;
    tx.commit().map_err(|_| NsError::StorageFailure)
}

/// Lo que se cortó con la aplicación cerrada vuelve a la cola.
///
/// # Errors
///
/// [`NsError::StorageFailure`] si la escritura falla.
pub fn retomar(db: &BaseDeDatos) -> NsResult<()> {
    marcar(
        db,
        "UPDATE stems SET estado = 'pendiente' WHERE estado = 'separando'",
        [],
    )?;
    // Lo que falló por no tener el modelo se vuelve a intentar: si sigue sin
    // estar, la cola no corre y no se gasta nada.
    marcar(
        db,
        "UPDATE stems SET estado = 'pendiente', error = NULL, intentos = 0
         WHERE estado = 'fallida' AND error = 'COMPONENT_UNAVAILABLE'",
        [],
    )
}

/// Tras una actualización, todo lo que había fallado se vuelve a intentar desde
/// cero: la versión nueva puede traer el arreglo (como el del remuestreo).
///
/// # Errors
///
/// [`NsError::StorageFailure`] si la escritura falla.
pub fn reintentar_fallidas(db: &BaseDeDatos) -> NsResult<()> {
    marcar(
        db,
        "UPDATE stems SET estado = 'pendiente', error = NULL, intentos = 0,
             solicitada_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE estado = 'fallida'",
        [],
    )
}

/// Veces que se intenta una canción antes de darla por fallida.
pub const MAX_INTENTOS: u32 = 3;

/// Si un fallo puede ser pasajero. Un archivo que no está o no se puede leer no
/// mejora por insistir; lo demás (el modelo, la memoria, el disco) sí puede.
fn pasajero(error: &NsError) -> bool {
    // Sin el componente (ONNX Runtime o el modelo) fallarían todas una detrás de
    // otra en un instante: la cola se para, y `rehabilitar_sin_componente` las
    // devuelve en cuanto vuelve a estar.
    !matches!(
        error,
        NsError::FileNotFound
            | NsError::AudioCorrupted
            | NsError::FormatUnsupported
            | NsError::ComponentUnavailable
    )
}

/// Lo que falló por no tener el componente vuelve a la cola: se llama cada vez
/// que la cola despierta con el modelo instalado.
fn rehabilitar_sin_componente(db: &BaseDeDatos) -> NsResult<()> {
    marcar(
        db,
        "UPDATE stems SET estado = 'pendiente', error = NULL
         WHERE estado = 'fallida' AND error = 'COMPONENT_UNAVAILABLE'",
        [],
    )
}

/// Separa una pendiente. Devuelve `false` si no había ninguna.
fn una(
    db: &BaseDeDatos,
    rutas: &Rutas,
    modelo: &Path,
    estado: &Mutex<Option<Preparando>>,
    avisar: &dyn Fn(Option<i64>),
) -> NsResult<bool> {
    let Some((pista, audio, sha)) = siguiente(db)? else {
        return Ok(false);
    };
    marcar(
        db,
        "UPDATE stems SET estado = 'separando', intentos = intentos + 1 WHERE pista_id = ?1",
        [pista],
    )?;
    let relativa = format!("v3-{pista}.flac");
    let destino = rutas.stems().join(&relativa);
    let mut ultimo = -1.0;
    let _turno = turno();
    let resultado =
        preparar_instrumental(rutas, modelo, &audio, sha.as_deref(), &destino, &mut |f| {
            if f - ultimo >= 0.01 {
                ultimo = f;
                *estado.lock().unwrap_or_else(PoisonError::into_inner) =
                    Some(Preparando { pista, fraccion: f });
                avisar(None);
            }
            true
        });
    *estado.lock().unwrap_or_else(PoisonError::into_inner) = None;
    let lista = resultado.is_ok();
    match resultado {
        Ok(bytes) => marcar(
            db,
            "UPDATE stems SET estado = 'lista', instrumental = ?2, bytes = ?3, modelo = ?4,
                 sha256 = ?5, error = NULL,
                 terminada_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE pista_id = ?1",
            params![pista, relativa, bytes as i64, MODELO, sha],
        )?,
        Err(error) => {
            tracing::warn!(
                codigo = error.code(),
                "no se pudo preparar el karaoke de una pista"
            );
            // Un fallo pasajero vuelve al final de la cola hasta `MAX_INTENTOS`
            // veces; después, o si no es pasajero, queda fallida y la consola lo
            // dice.
            marcar(
                db,
                "UPDATE stems SET error = ?2,
                     estado = CASE WHEN ?3 AND intentos < ?4 THEN 'pendiente' ELSE 'fallida' END,
                     solicitada_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                 WHERE pista_id = ?1",
                params![pista, error.code(), pasajero(&error), MAX_INTENTOS],
            )?;
        }
    }
    avisar(lista.then_some(pista));
    Ok(true)
}

static TURNO: Mutex<()> = Mutex::new(());

/// Una sola sesión de HTDemucs a la vez en todo el programa: cada una llega a
/// 1,3 GB, y el karaoke y los mashups (ADR-018) no deben sumarlas.
pub fn turno() -> std::sync::MutexGuard<'static, ()> {
    TURNO.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Completa el karaoke de una pista que lo esperaba con una separación hecha
/// para otra cosa (los mashups): el instrumental es la mezcla menos la voz, el
/// mismo cálculo que [`separar`], y no se vuelve a pasar el modelo.
/// Solo actúa si la pista tiene su fila en `pendiente`; devuelve si la completó.
///
/// # Errors
///
/// [`NsError::StorageFailure`] si falla la escritura.
pub fn completar_desde_stems(
    db: &BaseDeDatos,
    rutas: &Rutas,
    pista: i64,
    sha: Option<&str>,
    mezcla: (&[f32], &[f32]),
    voz: (&[f32], &[f32]),
) -> NsResult<bool> {
    let pendiente: bool = db
        .conexion()?
        .query_row(
            "SELECT count(*) FROM stems WHERE pista_id = ?1 AND estado = 'pendiente'",
            [pista],
            |f| f.get::<_, i64>(0),
        )
        .map_err(|_| NsError::StorageFailure)?
        > 0;
    if !pendiente {
        return Ok(false);
    }
    let resta =
        |m: &[f32], v: &[f32]| -> Vec<f32> { m.iter().zip(v).map(|(a, b)| a - b).collect() };
    let (a, b) = (resta(mezcla.0, voz.0), resta(mezcla.1, voz.1));
    let relativa = format!("v3-{pista}.flac");
    let bytes = escribir_instrumental(&rutas.stems().join(&relativa), &a, &b)?;
    marcar(
        db,
        "UPDATE stems SET estado = 'lista', instrumental = ?2, bytes = ?3, modelo = ?4,
             sha256 = ?5, error = NULL,
             terminada_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE pista_id = ?1 AND estado = 'pendiente'",
        params![pista, relativa, bytes as i64, MODELO, sha],
    )?;
    Ok(true)
}

/// El servicio: la cola y lo que se está preparando. Clonable.
#[derive(Clone)]
pub struct Karaoke {
    tx: Sender<()>,
    estado: Arc<Mutex<Option<Preparando>>>,
}

impl Karaoke {
    /// Arranca el hilo. Sin el agregado instalado espera: se le despierta al
    /// pedir una pista o al instalarlo. `avisar` recibe cada cambio y, cuando una
    /// pista queda lista, su id.
    pub fn arrancar(
        db: BaseDeDatos,
        rutas: Rutas,
        avisar: impl Fn(Option<i64>) + Send + 'static,
    ) -> Self {
        let (tx, rx): (Sender<()>, Receiver<()>) = mpsc::channel();
        let estado = Arc::new(Mutex::new(None));
        let e = Arc::clone(&estado);
        let lanzado = std::thread::Builder::new()
            .name("karaoke".into())
            .spawn(move || {
                crate::prioridad::bajar_hilo_actual();
                let _ = retomar(&db);
                // Cada vez que la cola despierta (al arrancar, al pedir algo, al
                // instalar el agregado o cada cinco minutos), lo que falló por no
                // tener el componente vuelve a intentarse; entre medias no, para no
                // girar en vacío si sigue sin estar.
                let mut despierta = true;
                loop {
                    // Hasta que el arranque fije dónde está ONNX Runtime no se toca
                    // la cola: antes, todo lo pendiente fallaba en ese instante.
                    if !crate::onnx::ruta_fijada() {
                        std::thread::sleep(Duration::from_millis(250));
                        continue;
                    }
                    let modelo = modelo_instalado(&rutas);
                    if despierta && modelo.is_some() {
                        let _ = rehabilitar_sin_componente(&db);
                    }
                    despierta = false;
                    if let Some(modelo) = modelo {
                        match una(&db, &rutas, &modelo, &e, &avisar) {
                            Ok(true) => continue,
                            Ok(false) => {}
                            Err(error) => {
                                tracing::error!(codigo = error.code(), "falló la cola del karaoke")
                            }
                        }
                    }
                    match rx.recv_timeout(Duration::from_secs(300)) {
                        Ok(()) | Err(RecvTimeoutError::Timeout) => despierta = true,
                        Err(RecvTimeoutError::Disconnected) => return,
                    }
                }
            });
        if lanzado.is_err() {
            tracing::error!("no se pudo lanzar el karaoke");
        }
        Self { tx, estado }
    }

    /// Despierta la cola: hay algo nuevo que preparar o se instaló el agregado.
    pub fn despertar(&self) {
        let _ = self.tx.send(());
    }

    #[must_use]
    pub fn preparando(&self) -> Option<Preparando> {
        *self.estado.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sin_componente_la_cola_se_para_y_vuelve_al_despertar() {
        use crate::db::BaseDeDatos;
        use crate::error::NsError;
        // Sin ONNX Runtime todas fallarían en un instante: no se reintenta en el
        // momento (la cola se para), pero vuelve al despertar.
        assert!(!super::pasajero(&NsError::ComponentUnavailable));
        assert!(super::pasajero(&NsError::Internal));
        let db = BaseDeDatos::en_memoria().unwrap();
        db.conexion()
            .unwrap()
            .execute_batch(
                "INSERT INTO pistas (id, titulo, ruta) VALUES (1, 'uno', 'a.mp3'), (2, 'dos', 'b.mp3');
                 INSERT INTO stems (pista_id, estado, error) VALUES
                     (1, 'fallida', 'COMPONENT_UNAVAILABLE'), (2, 'fallida', 'FILE_NOT_FOUND');",
            )
            .unwrap();
        super::rehabilitar_sin_componente(&db).unwrap();
        let estados: Vec<String> = db
            .conexion()
            .unwrap()
            .prepare("SELECT estado FROM stems ORDER BY pista_id")
            .unwrap()
            .query_map([], |f| f.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(estados, ["pendiente", "fallida"]);
    }

    #[test]
    fn el_remuestreo_acepta_audio_de_trozos_justos_y_con_resto() {
        // "Cooler Than Me" (48 kHz) mide 2.670 trozos de 4.096 justos y fallaba
        // siempre con INTERNAL, volviendo a la cola en cada intento.
        for n in [4_096 * 40, 4_096 * 40 + 1_234] {
            let izq: Vec<f32> = (0..n).map(|i| ((i as f32) * 0.01).sin() * 0.5).collect();
            let der = izq.clone();
            let (a, b) = super::remuestrear(&izq, &der, 48_000).expect("remuestrea");
            let esperado = (n as f64 * 44_100.0 / 48_000.0).round() as usize;
            assert_eq!((a.len(), b.len()), (esperado, esperado), "{n} muestras");
        }
    }

    use super::*;

    /// Un modelo de mentira que dice que todo es voz: copia la entrada en los
    /// cuatro stems.
    fn todo_voz(trozo: &[f32]) -> NsResult<Vec<f32>> {
        Ok(trozo.repeat(4))
    }

    fn senal(n: usize, f: f32, fase: f32) -> Vec<f32> {
        (0..n)
            .map(|i| (i as f32 * f / TASA as f32 * std::f32::consts::TAU + fase).sin() * 0.4 + 0.01)
            .collect()
    }

    #[test]
    fn si_todo_es_voz_el_instrumental_es_silencio_en_toda_la_pista() {
        // Más de dos segmentos y un trozo final corto: todas las muestras quedan
        // cubiertas por la ventana y se deshace bien la normalización.
        let n = SEGMENTO * 2 + 12_345;
        let (izq, der) = (senal(n, 440.0, 0.0), senal(n, 330.0, 1.0));
        let mut avance = Vec::new();
        let (a, b) = separar(&izq, &der, &mut todo_voz, &mut |f| {
            avance.push(f);
            true
        })
        .unwrap();
        assert_eq!((a.len(), b.len()), (n, n));
        let max = a.iter().chain(&b).fold(0.0f32, |m, v| m.max(v.abs()));
        assert!(max < 1e-4, "{max}");
        assert_eq!(avance.last(), Some(&1.0));
        assert!(avance.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn si_no_hay_voz_el_instrumental_es_la_mezcla() {
        let n = SEGMENTO + 1000;
        let (izq, der) = (senal(n, 220.0, 0.0), senal(n, 110.0, 0.5));
        let mut sin_voz = |t: &[f32]| -> NsResult<Vec<f32>> {
            let mut v = t.repeat(4);
            // La voz, a la mezcla normalizada de media cero: tras deshacer la
            // normalización vale la media, y el instrumental es la mezcla menos
            // ella.
            for x in &mut v[VOZ * 2 * SEGMENTO..] {
                *x = 0.0;
            }
            Ok(v)
        };
        let (a, _) = separar(&izq, &der, &mut sin_voz, &mut |_| true).unwrap();
        let media = (0..n).map(|i| (izq[i] + der[i]) * 0.5).sum::<f32>() / n as f32;
        for i in (0..n).step_by(997) {
            assert!((a[i] - (izq[i] - media)).abs() < 1e-4, "{i}");
        }
    }

    /// Un modelo de mentira que reparte la entrada entre los cuatro stems con
    /// pesos distintos: así cada stem es distinguible.
    fn repartido(t: &[f32]) -> NsResult<Vec<f32>> {
        let mut v = Vec::with_capacity(t.len() * 4);
        for k in 0..4 {
            v.extend(t.iter().map(|x| x * (0.1 + 0.2 * k as f32)));
        }
        Ok(v)
    }

    /// El karaoke de antes de ADR-018, tal cual, como referencia de regresión.
    fn instrumental_anterior(izq: &[f32], der: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let largo = izq.len();
        let n = largo as f64;
        let media = (0..largo)
            .map(|i| f64::from(izq[i] + der[i]) * 0.5)
            .sum::<f64>()
            / n;
        let var = (0..largo)
            .map(|i| (f64::from(izq[i] + der[i]) * 0.5 - media).powi(2))
            .sum::<f64>()
            / (n - 1.0).max(1.0);
        let desv = var.sqrt() + 1e-8;
        let (media, desv) = (media as f32, desv as f32);
        let peso = ventana();
        let paso = ((0.75 * SEGMENTO as f64) as usize).max(1);
        let mut voz = [vec![0.0f32; largo], vec![0.0f32; largo]];
        let mut total = vec![0.0f32; largo];
        let mut trozo = vec![0.0f32; 2 * SEGMENTO];
        for inicio in (0..largo).step_by(paso) {
            let m = SEGMENTO.min(largo - inicio);
            trozo.iter_mut().for_each(|v| *v = 0.0);
            for t in 0..m {
                trozo[t] = (izq[inicio + t] - media) / desv;
                trozo[SEGMENTO + t] = (der[inicio + t] - media) / desv;
            }
            let stems = repartido(&trozo).unwrap();
            for (c, v) in voz.iter_mut().enumerate() {
                let base = (VOZ * 2 + c) * SEGMENTO;
                for t in 0..m {
                    v[inicio + t] += stems[base + t] * peso[t];
                }
            }
            for t in 0..m {
                total[inicio + t] += peso[t];
            }
        }
        let f = |mezcla: &[f32], v: &[f32]| -> Vec<f32> {
            (0..largo)
                .map(|i| mezcla[i] - ((v[i] / total[i].max(1e-8)) * desv + media))
                .collect()
        };
        (f(izq, &voz[0]), f(der, &voz[1]))
    }

    #[test]
    fn el_instrumental_del_karaoke_es_identico_bit_a_bit_tras_generalizar() {
        let n = SEGMENTO * 2 + 777;
        let (izq, der) = (senal(n, 440.0, 0.0), senal(n, 330.0, 1.0));
        let nuevo = separar(&izq, &der, &mut repartido, &mut |_| true).unwrap();
        let viejo = instrumental_anterior(&izq, &der);
        assert!(nuevo
            .0
            .iter()
            .zip(&viejo.0)
            .all(|(a, b)| a.to_bits() == b.to_bits()));
        assert!(nuevo
            .1
            .iter()
            .zip(&viejo.1)
            .all(|(a, b)| a.to_bits() == b.to_bits()));
    }

    #[test]
    fn los_stems_conservan_duracion_alineacion_y_solo_ocupan_los_pedidos() {
        let n = SEGMENTO + 4321;
        let (izq, der) = (senal(n, 220.0, 0.0), senal(n, 110.0, 0.5));
        let r = separar_stems(
            &izq,
            &der,
            [true, false, true, true],
            &mut repartido,
            &mut |_| true,
        )
        .unwrap();
        assert!(r[1].is_none());
        let media = (0..n).map(|i| (izq[i] + der[i]) * 0.5).sum::<f32>() / n as f32;
        for (k, s) in r.iter().enumerate() {
            let Some((a, b)) = s else { continue };
            assert_eq!((a.len(), b.len()), (n, n));
            // El stem k es la mezcla centrada escalada por su peso: misma fase,
            // ningún desplazamiento temporal.
            let g = 0.1 + 0.2 * k as f32;
            for i in (0..n).step_by(1009) {
                let esperado = (izq[i] - media) * g + media;
                assert!((a[i] - esperado).abs() < 1e-4, "stem {k} muestra {i}");
            }
        }
    }

    #[test]
    fn las_claves_de_los_stems_son_estables_y_reversibles() {
        for s in STEMS {
            assert_eq!(Stem::desde_clave(s.clave()), Some(s));
            assert_eq!(STEMS[s.indice()], s);
        }
        assert_eq!(Stem::desde_clave("nada"), None);
    }

    #[test]
    fn se_puede_cancelar_y_un_modelo_con_otra_forma_se_rechaza() {
        let n = SEGMENTO * 3;
        let (izq, der) = (senal(n, 220.0, 0.0), senal(n, 220.0, 0.0));
        let mut llamadas = 0;
        assert_eq!(
            separar(&izq, &der, &mut todo_voz, &mut |_| {
                llamadas += 1;
                llamadas < 2
            }),
            Err(NsError::Cancelled)
        );
        let mut corto = |t: &[f32]| -> NsResult<Vec<f32>> { Ok(t.to_vec()) };
        assert_eq!(
            separar(&izq, &der, &mut corto, &mut |_| true),
            Err(NsError::Internal)
        );
        assert_eq!(
            separar(&[], &[], &mut todo_voz, &mut |_| true),
            Err(NsError::AudioCorrupted)
        );
    }

    #[test]
    fn el_wav_escrito_lo_lee_el_reproductor() {
        let d = std::env::temp_dir().join(format!("ns-karaoke-wav-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let ruta = d.join("x.wav");
        let s = senal(TASA as usize, 440.0, 0.0);
        let bytes = escribir_wav(&ruta, &s, &s).unwrap();
        assert_eq!(bytes, 44 + 4 * TASA as u64);
        let (a, b) = cargar(&ruta).unwrap();
        assert_eq!(a.len(), TASA as usize);
        assert!((a[1234] - s[1234]).abs() < 1e-3 && (b[99] - s[99]).abs() < 1e-3);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn el_instrumental_es_flac_que_lee_el_reproductor_y_ocupa_menos_que_un_wav() {
        let d = std::env::temp_dir().join(format!("ns-karaoke-flac-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let ruta = d.join("v3-1.flac");
        let s = senal(TASA as usize, 440.0, 0.0);
        let bytes = escribir_instrumental(&ruta, &s, &s).unwrap();
        assert!(bytes < 44 + 4 * TASA as u64, "{bytes}");
        assert!(!ruta.with_extension("flac.parcial").exists());
        assert_eq!(&std::fs::read(&ruta).unwrap()[..4], b"fLaC");
        let (a, b) = cargar(&ruta).unwrap();
        assert_eq!(a.len(), TASA as usize);
        assert!((a[1234] - s[1234]).abs() < 1e-3 && (b[99] - s[99]).abs() < 1e-3);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn la_cola_va_por_orden_reintenta_lo_fallido_y_retoma_lo_cortado() {
        let db = BaseDeDatos::en_memoria().unwrap();
        db.conexion()
            .unwrap()
            .execute_batch(
                "INSERT INTO pistas (id, titulo, ruta) VALUES (1, 'a', 'a.mp3'), (2, 'b', 'b.mp3'), (3, 'c', 'c.mp3');
                 INSERT INTO stems (pista_id, estado, error) VALUES (3, 'fallida', 'AUDIO_CORRUPTED');
                 INSERT INTO stems (pista_id, estado) VALUES (2, 'separando');",
            )
            .unwrap();
        pedir(&db, &[1, 3]).unwrap();
        retomar(&db).unwrap();
        let estados: String = db
            .conexion()
            .unwrap()
            .query_row(
                "SELECT group_concat(pista_id || estado, ',') FROM (SELECT * FROM stems ORDER BY pista_id)",
                [],
                |f| f.get(0),
            )
            .unwrap();
        assert_eq!(estados, "1pendiente,2pendiente,3pendiente");
        // Una lista no vuelve a la cola por pedirla otra vez.
        db.conexion()
            .unwrap()
            .execute(
                "UPDATE stems SET estado = 'lista', instrumental = 'x.wav' WHERE pista_id = 1",
                [],
            )
            .unwrap();
        pedir(&db, &[1]).unwrap();
        assert!(siguiente(&db).unwrap().is_some_and(|(id, _, _)| id != 1));
    }

    /// Con el modelo real, si alguien lo deja en `NS_MODELO_KARAOKE`: no está en el
    /// repositorio (`AGENTS.md`), así que de ordinario se salta sola.
    #[test]
    fn con_el_modelo_real_separa_una_pista_sin_pasar_de_3_gb() {
        let (Some(modelo), Some(audio), Some(ort)) = (
            std::env::var_os("NS_MODELO_KARAOKE").map(PathBuf::from),
            std::env::var_os("NS_AUDIO_KARAOKE").map(PathBuf::from),
            std::env::var_os("NS_ONNXRUNTIME").map(PathBuf::from),
        ) else {
            return;
        };
        crate::onnx::fijar_ruta(&ort);
        let d = std::env::temp_dir().join(format!("ns-karaoke-real-{}", std::process::id()));
        let bytes = separar_archivo(&modelo, &audio, &d.join("i.wav"), &mut |_| true).unwrap();
        assert!(bytes > 44);
        let _ = std::fs::remove_dir_all(&d);
        // El criterio: por debajo de 3 GB. Con el grafo optimizado eran
        // 6,6 GB. Se mide el pico del proceso, así que se ejecuta sola.
        #[cfg(target_os = "linux")]
        {
            let estado = std::fs::read_to_string("/proc/self/status").unwrap();
            let pico_kb: u64 = estado
                .lines()
                .find_map(|l| l.strip_prefix("VmHWM:"))
                .and_then(|v| v.trim().trim_end_matches("kB").trim().parse().ok())
                .unwrap();
            assert!(pico_kb < 3 * 1024 * 1024, "pico de {pico_kb} kB");
        }
    }

    #[test]
    fn preparar_las_favoritas_pone_en_cola_solo_lo_que_falta() {
        let db = BaseDeDatos::en_memoria().unwrap();
        db.conexion()
            .unwrap()
            .execute_batch(
                "INSERT INTO pistas (id, titulo, ruta, favorita) VALUES
                     (1, 'a', 'a.mp3', 1), (2, 'b', 'b.mp3', 1), (3, 'c', 'c.mp3', 1), (4, 'd', 'd.mp3', 0);
                 INSERT INTO stems (pista_id, estado, instrumental) VALUES (1, 'lista', 'x.wav');
                 INSERT INTO stems (pista_id, estado) VALUES (2, 'fallida');",
            )
            .unwrap();
        assert_eq!(pedir_favoritas(&db).unwrap(), 2);
        assert_eq!(
            pedir_favoritas(&db).unwrap(),
            0,
            "lo que ya espera no se cuenta"
        );
        let pendientes: String = db
            .conexion()
            .unwrap()
            .query_row(
                "SELECT group_concat(pista_id) FROM (SELECT pista_id FROM stems WHERE estado = 'pendiente' ORDER BY pista_id)",
                [],
                |f| f.get(0),
            )
            .unwrap();
        assert_eq!(pendientes, "2,3");
    }

    #[test]
    fn preparar_una_playlist_pone_en_cola_solo_sus_pistas_que_faltan() {
        let db = BaseDeDatos::en_memoria().unwrap();
        db.conexion()
            .unwrap()
            .execute_batch(
                "INSERT INTO pistas (id, titulo, ruta) VALUES
                     (1, 'a', 'a.mp3'), (2, 'b', 'b.mp3'), (3, 'c', 'c.mp3'), (4, 'd', 'd.mp3');
                 INSERT INTO stems (pista_id, estado, instrumental) VALUES (1, 'lista', 'x.wav');
                 INSERT INTO stems (pista_id, estado) VALUES (2, 'fallida');",
            )
            .unwrap();
        let pl = crate::playlists::crear(&db, "Para cantar", None).unwrap();
        db.conexion()
            .unwrap()
            .execute(
                "INSERT INTO playlist_pistas (playlist_id, pista_id, posicion) VALUES (?1, 1, 0), (?1, 2, 1), (?1, 3, 2)",
                [pl],
            )
            .unwrap();
        assert_eq!(pedir_playlist(&db, pl).unwrap(), 2);
        assert_eq!(
            pedir_playlist(&db, pl).unwrap(),
            0,
            "lo que ya espera no se cuenta"
        );
        let pendientes: String = db
            .conexion()
            .unwrap()
            .query_row(
                "SELECT group_concat(pista_id) FROM (SELECT pista_id FROM stems WHERE estado = 'pendiente' ORDER BY pista_id)",
                [],
                |f| f.get(0),
            )
            .unwrap();
        assert_eq!(pendientes, "2,3", "la pista 4 no está en la playlist");
    }

    #[test]
    fn el_karaoke_reutiliza_la_separacion_guardada_sin_pasar_el_modelo() {
        use crate::mashup::stems::CacheStems;
        let dir = std::env::temp_dir().join(format!("ns-karaoke-cache-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let rutas = Rutas::bajo(&dir).unwrap();
        let n = 44_100;
        let mezcla: Vec<f32> = (0..n).map(|i| ((i as f32) * 0.01).sin() * 0.5).collect();
        let voz: Vec<f32> = mezcla.iter().map(|x| x * 0.25).collect();
        let audio = dir.join("c.wav");
        escribir_wav(&audio, &mezcla, &mezcla).unwrap();
        let sha = "k".repeat(64);
        let cache = CacheStems::compartida(crate::mashup::servicio::carpeta_stems(&rutas)).unwrap();
        let otros = Some((vec![0.0; n], vec![0.0; n]));
        cache
            .guardar(
                &sha,
                &[
                    otros.clone(),
                    otros.clone(),
                    otros,
                    Some((voz.clone(), voz.clone())),
                ],
            )
            .unwrap();
        // El modelo no existe: si se intentara cargar, fallaría.
        let destino = dir.join("i.wav");
        preparar_instrumental(
            &rutas,
            &dir.join("no-hay.onnx"),
            &audio,
            Some(&sha),
            &destino,
            &mut |_| true,
        )
        .unwrap();
        let (a, _) = cargar(&destino).unwrap();
        let (m, _) = cargar(&audio).unwrap();
        let err = a
            .iter()
            .zip(&m)
            .zip(&voz)
            .fold(0.0f32, |e, ((x, mm), v)| e.max((x - (mm - v)).abs()));
        assert!(err < 3.0 / 32_767.0, "{err}");
        let _ = std::fs::remove_dir_all(dir);
    }
}
