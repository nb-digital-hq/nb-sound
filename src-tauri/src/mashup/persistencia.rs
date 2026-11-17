//! Lo que los mashups guardan en la base (V13).
//!
//! La definición (petición, semilla, plan) es lo que el usuario conserva; el
//! WAV es un derivado. Por eso el plan se guarda entero: con él y las pistas se
//! puede volver a renderizar, y se puede explicar qué hizo el compositor.

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::compositor::{Estilo, Plan};
use super::estructura::{Estructura, VERSION as VERSION_ESTRUCTURA};
use super::render::Metricas;
use crate::db::{self, BaseDeDatos};
use crate::error::{NsError, NsResult};

fn bd<T>(r: rusqlite::Result<T>) -> NsResult<T> {
    r.map_err(|error| {
        tracing::error!(error = %error, "falló una consulta de mashups");
        NsError::StorageFailure
    })
}

/// La petición tal como la guarda la base.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct PeticionMashup {
    pub duracion_seg: u32,
    pub canciones: u32,
    pub estilo: Estilo,
    /// Sin semilla, se elige una; una variante reutiliza la petición con otra.
    #[ts(type = "number | null")]
    pub semilla: Option<i64>,
    /// Pistas concretas; vacío para que el compositor elija de la biblioteca.
    #[ts(type = "number[]")]
    pub pistas: Vec<i64>,
    #[ts(type = "number | null")]
    pub playlist_id: Option<i64>,
}

/// La estructura guardada de una pista, si sigue valiendo para ese archivo y
/// esta versión del algoritmo.
///
/// # Errors
///
/// [`NsError::StorageFailure`].
pub fn leer_estructura(
    db: &BaseDeDatos,
    pista: i64,
    sha: &str,
) -> NsResult<Option<(Estructura, bool)>> {
    let conn = db.conexion()?;
    let fila: Option<(String, bool)> = bd(conn
        .query_row(
            "SELECT datos, con_voz FROM estructura_pistas WHERE pista_id = ?1 AND sha256 = ?2 AND version = ?3",
            params![pista, sha, VERSION_ESTRUCTURA],
            |f| Ok((f.get(0)?, f.get(1)?)),
        )
        .optional())?;
    Ok(fila.and_then(|(d, v)| serde_json::from_str(&d).ok().map(|e| (e, v))))
}

/// Guarda la estructura de una pista.
///
/// # Errors
///
/// [`NsError::StorageFailure`].
pub fn guardar_estructura(db: &BaseDeDatos, pista: i64, sha: &str, e: &Estructura) -> NsResult<()> {
    let datos = serde_json::to_string(e).map_err(|_| NsError::Internal)?;
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    bd(tx.execute(
        "INSERT INTO estructura_pistas (pista_id, sha256, version, datos, con_voz) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (pista_id) DO UPDATE SET sha256 = ?2, version = ?3, datos = ?4, con_voz = ?5,
             calculada_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
        params![pista, sha, VERSION_ESTRUCTURA, datos, e.voz.is_some()],
    ))?;
    bd(tx.commit())
}

/// Crea un mashup en `generando`.
///
/// # Errors
///
/// [`NsError::StorageFailure`].
pub fn crear(
    db: &BaseDeDatos,
    titulo: &str,
    p: &PeticionMashup,
    semilla: i64,
    version: u32,
) -> NsResult<i64> {
    let peticion = serde_json::to_string(p).map_err(|_| NsError::Internal)?;
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    bd(tx.execute(
        "INSERT INTO mashups (titulo, peticion, semilla, version) VALUES (?1, ?2, ?3, ?4)",
        params![titulo, peticion, semilla, version],
    ))?;
    let id = tx.last_insert_rowid();
    bd(tx.commit())?;
    Ok(id)
}

/// Una pista del mashup con lo que se enseña de ella.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct PistaDeMashup {
    #[ts(type = "number | null")]
    pub pista_id: Option<i64>,
    pub titulo: String,
    pub artista: String,
}

/// Marca un mashup como listo, con su plan, su archivo y sus pistas. Todo en
/// una transacción: un mashup listo sin sus pistas no debe existir ni un
/// instante, y agrupar los argumentos en un tipo solo serviría a esta llamada.
///
/// # Errors
///
/// [`NsError::StorageFailure`].
#[allow(clippy::too_many_arguments)]
pub fn terminar(
    db: &BaseDeDatos,
    id: i64,
    plan: &Plan,
    archivo: &str,
    bytes: u64,
    metricas: &Metricas,
    pistas: &[PistaDeMashup],
) -> NsResult<()> {
    let plan_json = serde_json::to_string(plan).map_err(|_| NsError::Internal)?;
    let met = serde_json::to_string(metricas).map_err(|_| NsError::Internal)?;
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    bd(tx.execute(
        "UPDATE mashups SET estado = 'lista', plan = ?2, archivo = ?3, duracion_seg = ?4, bytes = ?5,
             metricas = ?6, error = NULL, terminado_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?1",
        params![id, plan_json, archivo, metricas.duracion_seg, bytes as i64, met],
    ))?;
    bd(tx.execute("DELETE FROM mashup_pistas WHERE mashup_id = ?1", [id]))?;
    for (i, p) in pistas.iter().enumerate() {
        bd(tx.execute(
            "INSERT INTO mashup_pistas (mashup_id, orden, pista_id, titulo, artista) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, i as i64, p.pista_id, p.titulo, p.artista],
        ))?;
    }
    bd(tx.commit())
}

/// Marca un mashup como fallido o cancelado, con el código del error.
///
/// # Errors
///
/// [`NsError::StorageFailure`].
pub fn fallar(db: &BaseDeDatos, id: i64, error: &NsError) -> NsResult<()> {
    let estado = if *error == NsError::Cancelled {
        "cancelada"
    } else {
        "fallida"
    };
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    bd(tx.execute(
        "UPDATE mashups SET estado = ?2, error = ?3, terminado_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?1 AND estado = 'generando'",
        params![id, estado, error.code()],
    ))?;
    bd(tx.commit())
}

/// Lo que quedó `generando` con la aplicación cerrada no va a terminar: se
/// marca fallido. Devuelve sus ids para borrar sus temporales.
///
/// # Errors
///
/// [`NsError::StorageFailure`].
pub fn recuperar_interrumpidos(db: &BaseDeDatos) -> NsResult<Vec<i64>> {
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    let ids: Vec<i64> = {
        let mut s = bd(tx.prepare("SELECT id FROM mashups WHERE estado = 'generando'"))?;
        let v = bd(s.query_map([], |f| f.get(0)))?;
        bd(v.collect::<rusqlite::Result<Vec<i64>>>())?
    };
    bd(tx.execute(
        "UPDATE mashups SET estado = 'fallida', error = 'INTERNAL',
             terminado_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE estado = 'generando'",
        [],
    ))?;
    bd(tx.commit())?;
    Ok(ids)
}

/// Una fila del historial de mashups.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct FilaMashup {
    #[ts(type = "number")]
    pub id: i64,
    pub titulo: String,
    pub estado: String,
    pub creado_en: String,
    pub duracion_seg: f64,
    pub canciones: u32,
}

/// Los mashups, del más reciente al más antiguo, filtrados por texto (título,
/// títulos y artistas de sus pistas), sin tildes ni mayúsculas.
///
/// # Errors
///
/// [`NsError::StorageFailure`].
pub fn listar(db: &BaseDeDatos, texto: &str, limite: u32) -> NsResult<Vec<FilaMashup>> {
    let conn = db.conexion()?;
    let mut s = bd(conn.prepare(
        "SELECT m.id, m.titulo, m.estado, m.creado_en, coalesce(m.duracion_seg, 0),
                (SELECT count(*) FROM mashup_pistas mp WHERE mp.mashup_id = m.id),
                m.titulo || ' ' || coalesce((SELECT group_concat(mp.titulo || ' ' || mp.artista, ' ')
                                             FROM mashup_pistas mp WHERE mp.mashup_id = m.id), '')
         FROM mashups m ORDER BY m.creado_en DESC, m.id DESC",
    ))?;
    let aguja = plegar(texto.trim());
    let filas = bd(s.query_map([], |f| {
        Ok((
            FilaMashup {
                id: f.get(0)?,
                titulo: f.get(1)?,
                estado: f.get(2)?,
                creado_en: f.get(3)?,
                duracion_seg: f.get(4)?,
                canciones: f.get::<_, i64>(5)? as u32,
            },
            f.get::<_, String>(6)?,
        ))
    }))?;
    let mut v = Vec::new();
    for fila in filas {
        let (fila, buscable) = bd(fila)?;
        if aguja.is_empty() || plegar(&buscable).contains(&aguja) {
            v.push(fila);
            if v.len() >= limite as usize {
                break;
            }
        }
    }
    Ok(v)
}

fn plegar(t: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    t.nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .collect()
}

/// Un mashup entero, para su vista.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct VistaMashup {
    #[ts(type = "number")]
    pub id: i64,
    pub titulo: String,
    pub estado: String,
    pub creado_en: String,
    pub peticion: PeticionMashup,
    #[ts(type = "number")]
    pub semilla: i64,
    pub plan: Option<Plan>,
    pub pistas: Vec<PistaDeMashup>,
    pub metricas: Option<Metricas>,
    pub error: Option<String>,
    pub duracion_seg: f64,
}

/// El detalle de un mashup.
///
/// # Errors
///
/// [`NsError::InvalidInput`] si no existe; [`NsError::StorageFailure`].
pub fn vista(db: &BaseDeDatos, id: i64) -> NsResult<VistaMashup> {
    let mut conn = db.conexion()?;
    let tx = db::lectura_coherente(&mut conn)?;
    let fila = bd(tx
        .query_row(
            "SELECT titulo, estado, creado_en, peticion, semilla, plan, metricas, error, coalesce(duracion_seg, 0)
             FROM mashups WHERE id = ?1",
            [id],
            |f| {
                Ok((
                    f.get::<_, String>(0)?,
                    f.get::<_, String>(1)?,
                    f.get::<_, String>(2)?,
                    f.get::<_, String>(3)?,
                    f.get::<_, i64>(4)?,
                    f.get::<_, Option<String>>(5)?,
                    f.get::<_, Option<String>>(6)?,
                    f.get::<_, Option<String>>(7)?,
                    f.get::<_, f64>(8)?,
                ))
            },
        )
        .optional())?
    .ok_or(NsError::InvalidInput)?;
    let pistas = {
        let mut s = bd(tx.prepare(
            "SELECT pista_id, titulo, artista FROM mashup_pistas WHERE mashup_id = ?1 ORDER BY orden",
        ))?;
        let v = bd(s.query_map([id], |f| {
            Ok(PistaDeMashup {
                pista_id: f.get(0)?,
                titulo: f.get(1)?,
                artista: f.get(2)?,
            })
        }))?;
        bd(v.collect::<rusqlite::Result<Vec<_>>>())?
    };
    Ok(VistaMashup {
        id,
        titulo: fila.0,
        estado: fila.1,
        creado_en: fila.2,
        peticion: serde_json::from_str(&fila.3).map_err(|_| NsError::Internal)?,
        semilla: fila.4,
        plan: fila.5.and_then(|p| serde_json::from_str(&p).ok()),
        pistas,
        metricas: fila.6.and_then(|m| serde_json::from_str(&m).ok()),
        error: fila.7,
        duracion_seg: fila.8,
    })
}

/// El archivo de un mashup listo, relativo a la carpeta de mashups.
///
/// # Errors
///
/// [`NsError::StorageFailure`].
pub fn archivo(db: &BaseDeDatos, id: i64) -> NsResult<Option<String>> {
    let conn = db.conexion()?;
    Ok(bd(conn
        .query_row(
            "SELECT archivo FROM mashups WHERE id = ?1 AND estado = 'lista'",
            [id],
            |f| f.get::<_, Option<String>>(0),
        )
        .optional())?
    .flatten())
}

/// Borra un mashup; devuelve su archivo para borrarlo del disco.
///
/// # Errors
///
/// [`NsError::StorageFailure`].
pub fn eliminar(db: &BaseDeDatos, id: i64) -> NsResult<Option<String>> {
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    let archivo: Option<String> = bd(tx
        .query_row("SELECT archivo FROM mashups WHERE id = ?1", [id], |f| {
            f.get(0)
        })
        .optional())?
    .flatten();
    bd(tx.execute("DELETE FROM mashups WHERE id = ?1", [id]))?;
    bd(tx.commit())?;
    Ok(archivo)
}

/// Los archivos que la base conoce: lo que no esté aquí en la carpeta de
/// mashups es basura de una generación que no terminó.
///
/// # Errors
///
/// [`NsError::StorageFailure`].
pub fn archivos(db: &BaseDeDatos) -> NsResult<Vec<String>> {
    let conn = db.conexion()?;
    let mut s = bd(conn.prepare("SELECT archivo FROM mashups WHERE archivo IS NOT NULL"))?;
    let v = bd(s.query_map([], |f| f.get(0)))?;
    bd(v.collect())
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::mashup::compositor::pruebas::biblioteca;
    use crate::mashup::compositor::{componer, Peticion};

    fn base() -> BaseDeDatos {
        BaseDeDatos::en_memoria().unwrap()
    }

    fn pista(db: &BaseDeDatos, id: i64) {
        let conn = db.conexion().unwrap();
        conn.execute(
            "INSERT INTO pistas (id, titulo, artista_nombre, album_titulo, ruta, sha256)
             VALUES (?1, ?2, 'Artista', 'Disco', ?3, ?4)",
            params![
                id,
                format!("Canción {id}"),
                format!("{id}.mp3"),
                "a".repeat(64)
            ],
        )
        .unwrap();
    }

    fn peticion() -> PeticionMashup {
        PeticionMashup {
            duracion_seg: 120,
            canciones: 4,
            estilo: Estilo::Equilibrado,
            semilla: Some(1),
            pistas: vec![],
            playlist_id: None,
        }
    }

    #[test]
    fn la_estructura_caduca_si_cambia_el_archivo() {
        let db = base();
        pista(&db, 1);
        let e = biblioteca(1, true).remove(0).estructura;
        guardar_estructura(&db, 1, &"a".repeat(64), &e).unwrap();
        let (leida, con_voz) = leer_estructura(&db, 1, &"a".repeat(64)).unwrap().unwrap();
        assert_eq!(leida, e);
        assert!(con_voz);
        assert!(leer_estructura(&db, 1, &"b".repeat(64)).unwrap().is_none());
    }

    #[test]
    fn un_mashup_completo_sobrevive_entero_a_reabrir_la_base() {
        let d = std::env::temp_dir().join(format!("ns-mashup-bd-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let ruta = d.join("b.sqlite");
        let f = biblioteca(6, true);
        let plan = componer(
            &f,
            &Peticion {
                duracion_seg: 120,
                canciones: 4,
                estilo: Estilo::Equilibrado,
                semilla: 1,
            },
        )
        .unwrap();
        let met = Metricas {
            duracion_seg: plan.duracion_seg(),
            lufs: -14.0,
            pico_dbfs: -1.0,
            recortes: 0,
            salto_max: 0.1,
            silencio_max_seg: 0.0,
            capas: plan.capas.len() as u32,
            segundos_render: 1.0,
            acumulador_max_seg: 1.0,
            pico_verdadero_dbtp: -1.0,
        };
        let id = {
            let db = BaseDeDatos::abrir(&ruta).unwrap();
            pista(&db, 1);
            let id = crear(&db, "Mashup 1", &peticion(), 1, 1).unwrap();
            let pistas = vec![PistaDeMashup {
                pista_id: Some(1),
                titulo: "Canción 1".into(),
                artista: "Artista".into(),
            }];
            terminar(&db, id, &plan, "1.wav", 10, &met, &pistas).unwrap();
            id
        };
        let db = BaseDeDatos::abrir(&ruta).unwrap();
        let v = vista(&db, id).unwrap();
        assert_eq!(v.estado, "lista");
        assert_eq!(v.plan.as_ref(), Some(&plan));
        assert_eq!(v.metricas, Some(met));
        assert_eq!(v.peticion, peticion());
        assert_eq!(archivo(&db, id).unwrap().as_deref(), Some("1.wav"));
    }

    #[test]
    fn lo_interrumpido_se_da_por_fallido_y_lo_cancelado_se_distingue() {
        let db = base();
        let a = crear(&db, "A", &peticion(), 1, 1).unwrap();
        let b = crear(&db, "B", &peticion(), 2, 1).unwrap();
        fallar(&db, b, &NsError::Cancelled).unwrap();
        assert_eq!(recuperar_interrumpidos(&db).unwrap(), vec![a]);
        assert_eq!(vista(&db, a).unwrap().estado, "fallida");
        assert_eq!(vista(&db, b).unwrap().estado, "cancelada");
        // Un mashup ya terminado no vuelve a fallar.
        fallar(&db, b, &NsError::Internal).unwrap();
        assert_eq!(vista(&db, b).unwrap().estado, "cancelada");
    }

    #[test]
    fn el_listado_busca_sin_tildes_y_borrar_devuelve_el_archivo() {
        let db = base();
        let a = crear(&db, "Mashup de verano", &peticion(), 1, 1).unwrap();
        let _b = crear(&db, "Otro", &peticion(), 2, 1).unwrap();
        assert_eq!(listar(&db, "", 10).unwrap().len(), 2);
        let l = listar(&db, "VERANÓ", 10).unwrap();
        assert_eq!(l.len(), 1);
        assert_eq!(l[0].id, a);
        assert_eq!(eliminar(&db, a).unwrap(), None);
        assert_eq!(listar(&db, "", 10).unwrap().len(), 1);
    }

    #[test]
    fn borrar_una_pista_no_borra_el_mashup_que_la_usaba() {
        let db = base();
        pista(&db, 1);
        let f = biblioteca(6, true);
        let plan = componer(
            &f,
            &Peticion {
                duracion_seg: 120,
                canciones: 4,
                estilo: Estilo::Equilibrado,
                semilla: 1,
            },
        )
        .unwrap();
        let met = Metricas {
            duracion_seg: 1.0,
            lufs: -14.0,
            pico_dbfs: -1.0,
            recortes: 0,
            salto_max: 0.0,
            silencio_max_seg: 0.0,
            capas: 1,
            segundos_render: 1.0,
            acumulador_max_seg: 1.0,
            pico_verdadero_dbtp: -1.0,
        };
        let id = crear(&db, "M", &peticion(), 1, 1).unwrap();
        let p = vec![PistaDeMashup {
            pista_id: Some(1),
            titulo: "Canción 1".into(),
            artista: "Artista".into(),
        }];
        terminar(&db, id, &plan, "m.wav", 1, &met, &p).unwrap();
        db.conexion()
            .unwrap()
            .execute("DELETE FROM pistas WHERE id = 1", [])
            .unwrap();
        let v = vista(&db, id).unwrap();
        assert_eq!(v.pistas[0].pista_id, None);
        assert_eq!(v.pistas[0].titulo, "Canción 1");
    }
}
