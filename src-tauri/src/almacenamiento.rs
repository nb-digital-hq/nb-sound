//! Lo que NB Sound ocupa en disco, por categoría (Ajustes, Almacenamiento).
//!
//! **Por qué por categoría y no un tope total.** Con un solo tope, llenar el
//! karaoke echaría a las portadas. Cada categoría regenerable tiene su propio
//! tope y su poda; lo que no se puede rehacer (portadas originales, mashups
//! creados por el usuario) solo se muestra.
//!
//! **Estimaciones reales.** "Si pasaras toda tu biblioteca" sale de lo medido:
//! el tamaño por segundo de audio de lo que ya hay en disco por la duración de
//! la biblioteca. Para el instrumental del karaoke y para las separaciones, la
//! media de los FLAC ya guardados. Sin nada guardado aún, una cota prudente:
//! la de un FLAC estéreo de 16 bits que no comprimiera nada respecto al WAV.

use std::path::Path;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::db::BaseDeDatos;
use crate::error::{NsError, NsResult};
use crate::mashup::stems::{self, CacheStems};
use crate::paths::Rutas;

/// La preferencia del tope de las separaciones, en bytes.
pub const CLAVE_TOPE: &str = "tope_separaciones";
/// El tope más bajo que se acepta: por debajo, un mashup de 4 canciones no cabe.
pub const TOPE_MINIMO: u64 = 1 << 30;
/// WAV de 16 bits estéreo a 44,1 kHz: el techo de un instrumental en FLAC de 16
/// bits, que nunca ocupa más. Solo se usa mientras no hay ninguno con que medir.
const WAV_BYTES_SEG: f64 = 176_400.0;
/// Cuatro stems en FLAC de 16 bits, medido con el corpus: unos 12 MB por minuto.
const FLAC_BYTES_SEG: f64 = 200_000.0;
/// Segundos de audio que la cola del karaoke separa por segundo de reloj.
const SEPARACION_TIEMPO_REAL: f64 = 0.38;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub enum IdAlmacen {
    /// Portadas e imágenes de artista originales: no se pueden rehacer.
    Portadas,
    /// Miniaturas de las portadas: se rehacen solas al verlas.
    Miniaturas,
    /// Voces, batería, bajo y otros de cada canción (karaoke y mashups).
    Separaciones,
    /// El instrumental del karaoke, que también descarga el teléfono.
    Karaoke,
    /// Los mashups creados.
    Mashups,
    /// La base de datos y sus respaldos de migración.
    BaseDeDatos,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct CategoriaAlmacen {
    pub id: IdAlmacen,
    #[ts(type = "number")]
    pub bytes: u64,
    /// Lo que ocuparía con toda la biblioteca, si se puede estimar.
    #[ts(type = "number | null")]
    pub toda_la_biblioteca: Option<u64>,
    /// El tope, si la categoría tiene uno.
    #[ts(type = "number | null")]
    pub tope: Option<u64>,
    /// Si se puede vaciar desde Ajustes.
    pub liberable: bool,
    /// Lo que tardaría en rehacerse lo que hay, si se vacía.
    pub rehacer_seg: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub struct InformeAlmacen {
    pub categorias: Vec<CategoriaAlmacen>,
    /// Lo libre en el disco de la caché, si el sistema lo dice.
    #[ts(type = "number | null")]
    pub libre: Option<u64>,
    pub canciones: u32,
}

fn tamano(dir: &Path) -> u64 {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return 0;
    };
    rd.flatten()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => tamano(&e.path()),
            _ => e.metadata().map_or(0, |m| m.len()),
        })
        .sum()
}

/// Muestras por canal de una FLAC, de su STREAMINFO (36 bits).
fn muestras_flac(ruta: &Path) -> Option<u64> {
    use std::io::Read;
    let mut b = [0u8; 26];
    std::fs::File::open(ruta).ok()?.read_exact(&mut b).ok()?;
    if &b[..4] != b"fLaC" {
        return None;
    }
    // Tras "fLaC" (4) y la cabecera del bloque (4), STREAMINFO: los 36 bits
    // del total de muestras empiezan en el nibble bajo del byte 13.
    let s = &b[8..];
    let n = (u64::from(s[13] & 0x0f) << 32)
        | (u64::from(s[14]) << 24)
        | (u64::from(s[15]) << 16)
        | (u64::from(s[16]) << 8)
        | u64::from(s[17]);
    (n > 0).then_some(n)
}

/// Bytes y segundos de audio de las separaciones guardadas.
fn separaciones(raiz: &Path) -> (u64, f64) {
    let Ok(rd) = std::fs::read_dir(raiz) else {
        return (0, 0.0);
    };
    let (mut bytes, mut seg) = (0u64, 0.0f64);
    for e in rd.flatten() {
        let dir = e.path();
        if !dir.is_dir() {
            continue;
        }
        bytes += tamano(&dir);
        if let Some(n) = muestras_flac(&dir.join("voz.flac")) {
            seg += n as f64 / 44_100.0;
        }
    }
    (bytes, seg)
}

fn biblioteca(db: &BaseDeDatos) -> NsResult<(u32, f64)> {
    db.conexion()?
        .query_row(
            "SELECT count(*), coalesce(sum(duracion_seg), 0) FROM pistas WHERE estado = 'biblioteca'",
            [],
            |f| Ok((f.get::<_, i64>(0)?, f.get::<_, f64>(1)?)),
        )
        .map(|(n, s)| (u32::try_from(n).unwrap_or(u32::MAX), s))
        .map_err(|_| NsError::StorageFailure)
}

/// El tope guardado de las separaciones, o el de fábrica.
///
/// # Errors
///
/// [`NsError::StorageFailure`] si la consulta falla.
pub fn tope_guardado(db: &BaseDeDatos) -> NsResult<u64> {
    let conn = db.conexion()?;
    Ok(crate::preferencias::leer(&conn, CLAVE_TOPE)?
        .and_then(|v| v.as_u64())
        .filter(|b| *b >= TOPE_MINIMO)
        .unwrap_or(stems::TOPE_BYTES))
}

/// Bytes y segundos de los instrumentales ya preparados, para medir cuánto ocupa
/// un segundo de instrumental en esta biblioteca.
fn instrumentales(db: &BaseDeDatos) -> NsResult<Option<(f64, f64)>> {
    let conn = db.conexion()?;
    conn.query_row(
        "SELECT sum(s.bytes), sum(p.duracion_seg) FROM stems s
         JOIN pistas p ON p.id = s.pista_id
         WHERE s.estado = 'lista' AND s.bytes > 0 AND p.duracion_seg > 0",
        [],
        |f| Ok(f.get::<_, Option<f64>>(0)?.zip(f.get::<_, Option<f64>>(1)?)),
    )
    .map_err(|_| NsError::StorageFailure)
}

/// El informe de Ajustes.
///
/// # Errors
///
/// [`NsError::StorageFailure`] si la base de datos no responde.
pub fn informe(db: &BaseDeDatos, rutas: &Rutas) -> NsResult<InformeAlmacen> {
    let (canciones, segundos) = biblioteca(db)?;
    let (sep_bytes, sep_seg) = separaciones(&crate::mashup::servicio::carpeta_stems(rutas));
    let por_seg = if sep_seg > 60.0 {
        sep_bytes as f64 / sep_seg
    } else {
        FLAC_BYTES_SEG
    };
    let estimar = |b: f64| Some((b * segundos).round() as u64);
    let karaoke_por_seg = instrumentales(db)?
        .filter(|(_, seg)| *seg > 60.0)
        .map_or(WAV_BYTES_SEG, |(bytes, seg)| bytes / seg);
    let base = rutas.base_de_datos();
    let base_bytes: u64 = std::fs::read_dir(&rutas.datos)
        .map(|rd| {
            rd.flatten()
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .is_some_and(|n| n.starts_with("nb-sound.sqlite"))
                })
                .filter_map(|e| e.metadata().ok())
                .map(|m| m.len())
                .sum()
        })
        .unwrap_or_else(|_| std::fs::metadata(&base).map_or(0, |m| m.len()));
    let categoria = |id, bytes, toda, tope, liberable, rehacer| CategoriaAlmacen {
        id,
        bytes,
        toda_la_biblioteca: toda,
        tope,
        liberable,
        rehacer_seg: rehacer,
    };
    Ok(InformeAlmacen {
        categorias: vec![
            categoria(
                IdAlmacen::Portadas,
                tamano(&rutas.portadas()),
                None,
                None,
                false,
                None,
            ),
            categoria(
                IdAlmacen::Miniaturas,
                tamano(&rutas.cache.join("miniaturas")),
                None,
                None,
                true,
                None,
            ),
            categoria(
                IdAlmacen::Separaciones,
                sep_bytes,
                estimar(por_seg),
                Some(tope_guardado(db)?),
                true,
                (sep_seg > 0.0).then(|| sep_seg / SEPARACION_TIEMPO_REAL),
            ),
            categoria(
                IdAlmacen::Karaoke,
                tamano(&rutas.stems()),
                estimar(karaoke_por_seg),
                None,
                false,
                None,
            ),
            categoria(
                IdAlmacen::Mashups,
                tamano(&rutas.datos.join("mashups")),
                None,
                None,
                false,
                None,
            ),
            categoria(IdAlmacen::BaseDeDatos, base_bytes, None, None, false, None),
        ],
        libre: libre(&rutas.cache),
        canciones,
    })
}

/// Fija el tope de las separaciones y poda hasta quedar debajo.
///
/// # Errors
///
/// [`NsError::InvalidInput`] por debajo de [`TOPE_MINIMO`];
/// [`NsError::StorageFailure`].
pub fn fijar_tope(db: &BaseDeDatos, rutas: &Rutas, bytes: u64) -> NsResult<()> {
    if bytes < TOPE_MINIMO {
        return Err(NsError::InvalidInput);
    }
    crate::preferencias::guardar(db, CLAVE_TOPE, &serde_json::json!(bytes))?;
    stems::fijar_tope(bytes);
    let cache = CacheStems::compartida(crate::mashup::servicio::carpeta_stems(rutas))?;
    cache.podar(&std::collections::HashSet::new());
    Ok(())
}

/// Vacía una categoría regenerable. Devuelve lo liberado. Lo que usa una
/// generación en curso no se toca.
///
/// # Errors
///
/// [`NsError::InvalidInput`] si la categoría no es liberable;
/// [`NsError::StorageFailure`].
pub fn liberar(rutas: &Rutas, id: IdAlmacen) -> NsResult<u64> {
    match id {
        IdAlmacen::Separaciones => {
            let cache = CacheStems::abrir(crate::mashup::servicio::carpeta_stems(rutas), 0)?;
            Ok(cache.podar(&std::collections::HashSet::new()))
        }
        IdAlmacen::Miniaturas => {
            let dir = rutas.cache.join("miniaturas");
            let antes = tamano(&dir);
            if let Ok(rd) = std::fs::read_dir(&dir) {
                for e in rd.flatten() {
                    let _ = std::fs::remove_file(e.path());
                }
            }
            Ok(antes.saturating_sub(tamano(&dir)))
        }
        _ => Err(NsError::InvalidInput),
    }
}

#[cfg(unix)]
fn libre(dir: &Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    let c = std::ffi::CString::new(dir.as_os_str().as_bytes()).ok()?;
    let mut s: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: `c` es una ruta terminada en cero que vive durante la llamada y
    // `s` es una estructura del tamaño que `statvfs` escribe.
    if unsafe { libc::statvfs(c.as_ptr(), &mut s) } != 0 {
        return None;
    }
    #[allow(clippy::unnecessary_cast)]
    Some(s.f_bavail as u64 * s.f_frsize as u64)
}

#[cfg(windows)]
fn libre(dir: &Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let ancho: Vec<u16> = dir.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut disponible = 0u64;
    // SAFETY: `ancho` es una ruta terminada en cero que vive durante la llamada;
    // los punteros nulos son los totales que no se piden.
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            ancho.as_ptr(),
            &mut disponible,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    (ok != 0).then_some(disponible)
}

#[cfg(not(any(unix, windows)))]
fn libre(_: &Path) -> Option<u64> {
    None
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_informe_mide_cada_categoria_y_estima_con_la_duracion_de_la_biblioteca() {
        let dir = std::env::temp_dir().join(format!("ns-almacen-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let rutas = Rutas::bajo(&dir).unwrap();
        let db = BaseDeDatos::abrir(&rutas.base_de_datos()).unwrap();
        db.conexion()
            .unwrap()
            .execute_batch(
                "INSERT INTO pistas (id, titulo, ruta, duracion_seg) VALUES (1, 'a', 'a', 100), (2, 'b', 'b', 200);",
            )
            .unwrap();
        std::fs::create_dir_all(rutas.stems()).unwrap();
        std::fs::write(rutas.stems().join("v2-1.wav"), vec![0u8; 1000]).unwrap();
        let i = informe(&db, &rutas).unwrap();
        assert_eq!(i.canciones, 2);
        let k = i
            .categorias
            .iter()
            .find(|c| c.id == IdAlmacen::Karaoke)
            .unwrap();
        assert_eq!(k.bytes, 1000);
        assert_eq!(k.toda_la_biblioteca, Some(300 * 176_400));
        let s = i
            .categorias
            .iter()
            .find(|c| c.id == IdAlmacen::Separaciones)
            .unwrap();
        assert_eq!(s.tope, Some(stems::TOPE_BYTES));
        assert_eq!(s.toda_la_biblioteca, Some(300 * 200_000));
        assert!(i.libre.is_some_and(|l| l > 0));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn el_tope_se_guarda_rechaza_lo_absurdo_y_solo_se_libera_lo_regenerable() {
        let dir = std::env::temp_dir().join(format!("ns-almacen-tope-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let rutas = Rutas::bajo(&dir).unwrap();
        let db = BaseDeDatos::abrir(&rutas.base_de_datos()).unwrap();
        assert_eq!(fijar_tope(&db, &rutas, 1000), Err(NsError::InvalidInput));
        fijar_tope(&db, &rutas, 20 << 30).unwrap();
        assert_eq!(tope_guardado(&db).unwrap(), 20 << 30);
        assert_eq!(
            liberar(&rutas, IdAlmacen::Portadas),
            Err(NsError::InvalidInput)
        );
        let mini = rutas.cache.join("miniaturas");
        std::fs::create_dir_all(&mini).unwrap();
        std::fs::write(mini.join("1-160.jpg"), vec![0u8; 500]).unwrap();
        assert_eq!(liberar(&rutas, IdAlmacen::Miniaturas).unwrap(), 500);
        stems::fijar_tope(stems::TOPE_BYTES);
        let _ = std::fs::remove_dir_all(dir);
    }
}
