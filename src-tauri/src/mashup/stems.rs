//! La caché de stems, compartida por los mashups y el karaoke (ADR-018).
//!
//! **Por qué en caché y no en datos.** El instrumental del karaoke vive en datos
//! porque el teléfono lo descarga y el usuario lo espera siempre. Los cuatro
//! stems de un mashup solo sirven para componer y se pueden rehacer: si el
//! sistema limpia la caché, se pierde tiempo de procesador, no información.
//!
//! **Formato.** FLAC de 16 bits: sin pérdida a la profundidad a la que se
//! reproduce todo lo demás, y en torno a la cuarta parte de un WAV de 32 bits
//! flotantes. Symphonia ya lo decodifica y permite leer solo un fragmento, que
//! es lo que pide el renderizador.
//!
//! **Clave.** El hash del contenido de la pista, el modelo y la versión del
//! separador. Un archivo que cambia tiene otro hash y por tanto otra entrada; la
//! vieja deja de usarse y la recoge la poda LRU. Así no hace falta vigilar los
//! archivos para invalidar.
//!
//! **Atomicidad.** Se escribe en `<clave>.parcial/` y se renombra la carpeta
//! entera al terminar: una entrada existe completa o no existe. Las `.parcial`
//! huérfanas de un cierre a medias se borran al arrancar la caché.
//!
//! **Tramos.** Un mashup solo usa unos compases de cada canción, y separar la
//! canción entera para eso era la mayor parte de la espera. Una entrada puede
//! guardar solo un tramo (archivo `tramo` con su primera y última muestra);
//! sin ese archivo, la entrada es la canción entera. Lo que necesita la canción
//! completa (el karaoke) pide [`CacheStems::completa`] y, si solo hay un tramo,
//! separa la canción entera y la sustituye.
//!
//! **Compartida.** El karaoke y los mashups usan la misma caché: una canción se
//! separa una vez para todo. El tope es uno para toda la aplicación y lo fija el
//! usuario en Ajustes ([`fijar_tope`]); lo que usa una generación en curso queda
//! protegido ([`proteger`]) aunque otra parte del programa pode a la vez.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::SystemTime;

use crate::error::{NsError, NsResult};
use crate::karaoke::{self, Estereo, Stem, STEMS};

/// Sube cuando cambia la forma de separar o de guardar: deja inservibles las
/// entradas anteriores sin tener que borrarlas a mano.
pub const VERSION_SEPARADOR: u32 = 1;
/// El tope por defecto: unos 125 pistas con sus cuatro stems.
pub const TOPE_BYTES: u64 = 5 * 1024 * 1024 * 1024;
const MARCA_USO: &str = "uso";
const MARCA_TRAMO: &str = "tramo";
/// El tope de [`CacheStems::compartida`]: el que fija el usuario.
const TOPE_COMPARTIDO: u64 = u64::MAX;
static TOPE: AtomicU64 = AtomicU64::new(TOPE_BYTES);
static PROTEGIDAS: Mutex<Option<HashMap<String, u32>>> = Mutex::new(None);

/// Fija el tope de la caché compartida. Nunca cero: sin caché, cada mashup y
/// cada karaoke volverían a separar desde el principio.
pub fn fijar_tope(bytes: u64) {
    TOPE.store(bytes.max(1), Ordering::Relaxed);
}

/// El tope vigente de la caché compartida.
#[must_use]
pub fn tope() -> u64 {
    TOPE.load(Ordering::Relaxed)
}

/// Mientras viva, la entrada de su pista no se poda.
#[derive(Debug)]
pub struct Proteccion(String);

impl Drop for Proteccion {
    fn drop(&mut self) {
        let mut g = PROTEGIDAS.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(m) = g.as_mut() {
            if let Some(n) = m.get_mut(&self.0) {
                *n -= 1;
                if *n == 0 {
                    m.remove(&self.0);
                }
            }
        }
    }
}

/// Protege la entrada de `sha256` de la poda mientras dure lo que la usa.
#[must_use]
pub fn proteger(sha256: &str) -> Proteccion {
    let mut g = PROTEGIDAS.lock().unwrap_or_else(PoisonError::into_inner);
    *g.get_or_insert_with(HashMap::new)
        .entry(sha256.to_owned())
        .or_insert(0) += 1;
    Proteccion(sha256.to_owned())
}

fn protegida(sha256: &str) -> bool {
    PROTEGIDAS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .is_some_and(|m| m.contains_key(sha256))
}

/// Lo que guarda una entrada, en muestras a 44,1 kHz desde el inicio de la
/// canción. `hasta` es `None` si es la canción entera.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tramo {
    pub desde: u64,
    pub hasta: Option<u64>,
}

/// La caché en disco.
#[derive(Debug, Clone)]
pub struct CacheStems {
    raiz: PathBuf,
    tope: u64,
}

impl CacheStems {
    /// Abre la caché bajo `raiz` y borra lo que dejó a medias un cierre anterior.
    ///
    /// # Errors
    ///
    /// [`NsError::StorageFailure`] si no se puede crear la carpeta.
    pub fn abrir(raiz: PathBuf, tope: u64) -> NsResult<Self> {
        std::fs::create_dir_all(&raiz).map_err(|_| NsError::StorageFailure)?;
        let c = Self { raiz, tope };
        c.limpiar_parciales();
        Ok(c)
    }

    /// La caché compartida bajo `raiz`, con el tope que fije el usuario.
    ///
    /// # Errors
    ///
    /// [`NsError::StorageFailure`] si no se puede crear la carpeta.
    pub fn compartida(raiz: PathBuf) -> NsResult<Self> {
        Self::abrir(raiz, TOPE_COMPARTIDO)
    }

    fn tope_vigente(&self) -> u64 {
        if self.tope == TOPE_COMPARTIDO {
            tope()
        } else {
            self.tope
        }
    }

    /// La clave de una pista, a partir del hash de su contenido.
    #[must_use]
    pub fn clave(sha256: &str) -> String {
        format!("{sha256}-{}-v{VERSION_SEPARADOR}", karaoke::MODELO)
    }

    fn carpeta(&self, sha256: &str) -> PathBuf {
        self.raiz.join(Self::clave(sha256))
    }

    /// La ruta del stem si la entrada está completa. Marca el uso para la LRU.
    #[must_use]
    pub fn ruta(&self, sha256: &str, stem: Stem) -> Option<PathBuf> {
        let dir = self.carpeta(sha256);
        let r = dir.join(format!("{}.flac", stem.clave()));
        if !r.is_file() {
            return None;
        }
        tocar(&dir.join(MARCA_USO));
        Some(r)
    }

    /// Lo que guarda la entrada de la pista, si tiene sus cuatro stems.
    #[must_use]
    pub fn tramo(&self, sha256: &str) -> Option<Tramo> {
        let dir = self.carpeta(sha256);
        if !STEMS
            .iter()
            .all(|s| dir.join(format!("{}.flac", s.clave())).is_file())
        {
            return None;
        }
        let Ok(t) = std::fs::read_to_string(dir.join(MARCA_TRAMO)) else {
            return Some(Tramo {
                desde: 0,
                hasta: None,
            });
        };
        let mut p = t.split_whitespace().filter_map(|x| x.parse::<u64>().ok());
        match (p.next(), p.next()) {
            (Some(desde), Some(hasta)) if hasta > desde => Some(Tramo {
                desde,
                hasta: Some(hasta),
            }),
            // Una marca ilegible no se sirve: mejor separar otra vez que leer
            // desplazado.
            _ => None,
        }
    }

    /// Si la pista tiene los cuatro stems de la canción entera.
    #[must_use]
    pub fn completa(&self, sha256: &str) -> bool {
        self.tramo(sha256).is_some_and(|t| t.hasta.is_none())
    }

    /// Si la entrada cubre las muestras `[desde, hasta)`.
    #[must_use]
    pub fn cubre(&self, sha256: &str, desde: u64, hasta: u64) -> bool {
        self.tramo(sha256)
            .is_some_and(|t| t.desde <= desde && t.hasta.is_none_or(|h| hasta <= h))
    }

    /// Guarda los cuatro stems de una pista de forma atómica.
    ///
    /// # Errors
    ///
    /// [`NsError::Internal`] si falta algún stem; [`NsError::StorageFailure`].
    pub fn guardar(&self, sha256: &str, stems: &[Option<Estereo>; 4]) -> NsResult<u64> {
        self.guardar_en(sha256, stems, None)
    }

    /// Guarda los cuatro stems de un tramo que empieza en la muestra `desde`.
    ///
    /// # Errors
    ///
    /// Los de [`CacheStems::guardar`].
    pub fn guardar_tramo(
        &self,
        sha256: &str,
        stems: &[Option<Estereo>; 4],
        desde: u64,
    ) -> NsResult<u64> {
        self.guardar_en(sha256, stems, Some(desde))
    }

    fn guardar_en(
        &self,
        sha256: &str,
        stems: &[Option<Estereo>; 4],
        desde: Option<u64>,
    ) -> NsResult<u64> {
        let final_ = self.carpeta(sha256);
        let parcial = final_.with_extension("parcial");
        let _ = std::fs::remove_dir_all(&parcial);
        std::fs::create_dir_all(&parcial).map_err(|_| NsError::StorageFailure)?;
        let resultado = (|| {
            let mut bytes = 0;
            for s in STEMS {
                let (a, b) = stems[s.indice()].as_ref().ok_or(NsError::Internal)?;
                bytes += escribir_flac(&parcial.join(format!("{}.flac", s.clave())), a, b)?;
            }
            if let Some(d) = desde {
                let n = stems[0].as_ref().map_or(0, |s| s.0.len()) as u64;
                std::fs::write(parcial.join(MARCA_TRAMO), format!("{d} {}", d + n))
                    .map_err(|_| NsError::StorageFailure)?;
            }
            tocar(&parcial.join(MARCA_USO));
            let _ = std::fs::remove_dir_all(&final_);
            std::fs::rename(&parcial, &final_).map_err(|_| NsError::StorageFailure)?;
            Ok(bytes)
        })();
        if resultado.is_err() {
            let _ = std::fs::remove_dir_all(&parcial);
        }
        resultado
    }

    /// Lo que ocupa la caché.
    #[must_use]
    pub fn tamano(&self) -> u64 {
        self.entradas().iter().map(|e| e.2).sum()
    }

    /// Borra las entradas menos usadas hasta quedar bajo el tope, sin tocar las
    /// de `protegidas` (las claves de un mashup en curso). Devuelve lo liberado.
    pub fn podar(&self, protegidas: &HashSet<String>) -> u64 {
        let mut entradas = self.entradas();
        let mut total: u64 = entradas.iter().map(|e| e.2).sum();
        entradas.sort_by_key(|e| e.1);
        let mut liberado = 0;
        for (dir, _, bytes) in entradas {
            if total <= self.tope_vigente() {
                break;
            }
            let nombre = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if protegidas.iter().any(|p| Self::clave(p) == nombre)
                || nombre.split('-').next().is_some_and(protegida)
            {
                continue;
            }
            if std::fs::remove_dir_all(&dir).is_ok() {
                total -= bytes;
                liberado += bytes;
            }
        }
        liberado
    }

    fn entradas(&self) -> Vec<(PathBuf, SystemTime, u64)> {
        let Ok(rd) = std::fs::read_dir(&self.raiz) else {
            return Vec::new();
        };
        rd.flatten()
            .filter(|e| e.path().is_dir() && e.path().extension().is_none_or(|x| x != "parcial"))
            .map(|e| {
                let dir = e.path();
                let uso = std::fs::metadata(dir.join(MARCA_USO))
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                let bytes = std::fs::read_dir(&dir)
                    .map(|r| {
                        r.flatten()
                            .filter_map(|f| f.metadata().ok())
                            .map(|m| m.len())
                            .sum()
                    })
                    .unwrap_or(0);
                (dir, uso, bytes)
            })
            .collect()
    }

    fn limpiar_parciales(&self) {
        if let Ok(rd) = std::fs::read_dir(&self.raiz) {
            for e in rd.flatten() {
                if e.path().extension().is_some_and(|x| x == "parcial") {
                    let _ = std::fs::remove_dir_all(e.path());
                }
            }
        }
    }
}

fn tocar(marca: &Path) {
    // La fecha de modificación de un archivo vacío es el reloj de la LRU: no
    // depende de que el sistema de archivos guarde la fecha de acceso.
    let _ = std::fs::File::create(marca).and_then(|f| f.set_modified(SystemTime::now()));
}

/// FLAC estéreo de 16 bits a 44,1 kHz.
pub(crate) fn escribir_flac(destino: &Path, izq: &[f32], der: &[f32]) -> NsResult<u64> {
    use flacenc::component::BitRepr as _;
    use flacenc::error::Verify as _;
    let n = izq.len().min(der.len());
    let mut muestras = Vec::with_capacity(n * 2);
    let q = |v: f32| i32::from((v.clamp(-1.0, 1.0) * 32_767.0).round() as i16);
    for i in 0..n {
        muestras.push(q(izq[i]));
        muestras.push(q(der[i]));
    }
    let config = flacenc::config::Encoder::default()
        .into_verified()
        .map_err(|_| NsError::Internal)?;
    let fuente = flacenc::source::MemSource::from_samples(&muestras, 2, 16, karaoke::TASA as usize);
    let flujo = flacenc::encode_with_fixed_block_size(&config, fuente, config.block_size)
        .map_err(|_| NsError::Internal)?;
    drop(muestras);
    let mut sumidero = flacenc::bitsink::ByteSink::new();
    flujo.write(&mut sumidero).map_err(|_| NsError::Internal)?;
    let mut bytes = sumidero.as_slice().to_vec();
    // `flacenc` 0.5 apunta en STREAMINFO como tamaño mínimo de bloque el del
    // último, más corto. La especificación excluye el último bloque de ese
    // mínimo; con mínimo distinto del máximo, Symphonia y libFLAC leen el flujo
    // como de bloque variable, interpretan el número de marco como número de
    // muestra y rechazan el archivo. Se corrige el campo (bytes 8–9, tras
    // "fLaC" y la cabecera del bloque de metadatos) con el máximo (10–11).
    if bytes.len() > 12 && &bytes[..4] == b"fLaC" {
        let (min, max) = (8..10, 10..12);
        let m = [bytes[max.start], bytes[max.start + 1]];
        bytes[min].copy_from_slice(&m);
    }
    let bytes = bytes.as_slice();
    std::fs::write(destino, bytes).map_err(|_| NsError::StorageFailure)?;
    Ok(bytes.len() as u64)
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::audio::decodificador::Decodificador;

    fn dir(nombre: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ns-stems-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    fn stems_de(n: usize) -> [Option<Estereo>; 4] {
        std::array::from_fn(|k| {
            let f = 110.0 * (k + 1) as f32;
            let s: Vec<f32> = (0..n)
                .map(|i| (i as f32 * f / 44_100.0 * std::f32::consts::TAU).sin() * 0.3)
                .collect();
            Some((s.clone(), s))
        })
    }

    #[test]
    fn un_stem_guardado_se_relee_sin_perdida_mas_alla_de_16_bits() {
        let c = CacheStems::abrir(dir("relee"), TOPE_BYTES).unwrap();
        let stems = stems_de(44_100);
        c.guardar(&"a".repeat(64), &stems).unwrap();
        assert!(c.completa(&"a".repeat(64)));
        let r = c.ruta(&"a".repeat(64), Stem::Bajo).unwrap();
        let mut d = Decodificador::abrir(&r, 0.0).unwrap();
        let mut frames = Vec::new();
        while d.leer(&mut frames, 65_536).unwrap() > 0 {}
        assert_eq!(frames.len(), 44_100);
        let original = &stems[1].as_ref().unwrap().0;
        let err = frames
            .iter()
            .zip(original)
            .fold(0.0f32, |m, (f, o)| m.max((f.left - o).abs()));
        // Media unidad de cuantización de 16 bits más el redondeo de lectura.
        assert!(err <= 1.0 / 32_767.0, "{err}");
    }

    #[test]
    fn otro_hash_es_otra_entrada_y_la_vieja_no_se_sirve() {
        let c = CacheStems::abrir(dir("hash"), TOPE_BYTES).unwrap();
        c.guardar(&"a".repeat(64), &stems_de(1000)).unwrap();
        assert!(c.ruta(&"b".repeat(64), Stem::Voz).is_none());
        assert!(c.ruta(&"a".repeat(64), Stem::Voz).is_some());
        assert_ne!(
            CacheStems::clave(&"a".repeat(64)),
            CacheStems::clave(&"b".repeat(64))
        );
    }

    #[test]
    fn una_escritura_cortada_no_deja_entrada_ni_basura() {
        let raiz = dir("cortada");
        let c = CacheStems::abrir(raiz.clone(), TOPE_BYTES).unwrap();
        let mut incompletos = stems_de(1000);
        incompletos[2] = None;
        assert_eq!(
            c.guardar(&"a".repeat(64), &incompletos),
            Err(NsError::Internal)
        );
        assert!(!c.completa(&"a".repeat(64)));
        assert_eq!(std::fs::read_dir(&raiz).unwrap().count(), 0);
        // Una `.parcial` de un cierre brusco se borra al reabrir.
        std::fs::create_dir_all(raiz.join("x.parcial")).unwrap();
        CacheStems::abrir(raiz.clone(), TOPE_BYTES).unwrap();
        assert_eq!(std::fs::read_dir(&raiz).unwrap().count(), 0);
    }

    #[test]
    fn la_poda_respeta_el_tope_el_orden_de_uso_y_las_protegidas() {
        let raiz = dir("poda");
        let c = CacheStems::abrir(raiz, 0).unwrap();
        for k in ["a", "b", "c"] {
            c.guardar(&k.repeat(64), &stems_de(4000)).unwrap();
        }
        let una = c.tamano() / 3;
        let c = CacheStems { tope: una * 2, ..c };
        // "a" se usa ahora: pasa a ser la más reciente; "b" queda la más vieja.
        std::thread::sleep(std::time::Duration::from_millis(20));
        let _ = c.ruta(&"a".repeat(64), Stem::Voz);
        std::thread::sleep(std::time::Duration::from_millis(20));
        let _ = c.ruta(&"c".repeat(64), Stem::Voz);
        let protegidas: HashSet<String> = HashSet::new();
        assert!(c.podar(&protegidas) > 0);
        assert!(!c.completa(&"b".repeat(64)));
        assert!(c.completa(&"a".repeat(64)) && c.completa(&"c".repeat(64)));
        // Con tope cero, la protegida sobrevive.
        let c = CacheStems { tope: 0, ..c };
        let protegidas: HashSet<String> = std::iter::once("a".repeat(64)).collect();
        c.podar(&protegidas);
        assert!(c.completa(&"a".repeat(64)));
        assert!(!c.completa(&"c".repeat(64)));
    }

    #[test]
    fn un_tramo_se_sirve_solo_dentro_de_sus_muestras_y_no_pasa_por_completo() {
        let c = CacheStems::abrir(dir("tramo"), TOPE_BYTES).unwrap();
        let sha = "t".repeat(64);
        c.guardar_tramo(&sha, &stems_de(10_000), 44_100).unwrap();
        assert_eq!(
            c.tramo(&sha),
            Some(Tramo {
                desde: 44_100,
                hasta: Some(54_100)
            })
        );
        assert!(!c.completa(&sha), "un tramo no sirve al karaoke");
        assert!(c.cubre(&sha, 45_000, 54_000));
        assert!(!c.cubre(&sha, 40_000, 50_000));
        assert!(!c.cubre(&sha, 50_000, 60_000));
        // La canción entera sustituye al tramo.
        c.guardar(&sha, &stems_de(100_000)).unwrap();
        assert!(c.completa(&sha) && c.cubre(&sha, 0, 100_000));
    }

    #[test]
    fn la_poda_no_toca_lo_que_protege_una_generacion_en_curso() {
        let c = CacheStems::abrir(dir("protegida"), 0).unwrap();
        let (a, b) = ("p".repeat(64), "q".repeat(64));
        c.guardar(&a, &stems_de(2000)).unwrap();
        c.guardar(&b, &stems_de(2000)).unwrap();
        let guarda = proteger(&a);
        c.podar(&HashSet::new());
        assert!(c.completa(&a) && !c.completa(&b));
        drop(guarda);
        c.podar(&HashSet::new());
        assert!(!c.completa(&a));
    }
}
