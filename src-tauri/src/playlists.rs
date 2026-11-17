//! Playlists: las manuales, que el usuario crea y ordena, y las que ya
//! vienen hechas.
//!
//! La clasificación (`clasificacion_sync_playlist`) es la que envía el contrato
//! móvil, y el teléfono agrupa con ella. "Me gusta" es una playlist de verdad con
//! sus filas: el móvil la recibe así. Se mantiene sola al marcar o desmarcar
//! favoritas.

use rusqlite::{Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::biblioteca::PaginaPistas;
use crate::db::{self, BaseDeDatos};
use crate::error::{NsError, NsResult};
use crate::idioma::t;

pub mod inteligentes;

pub use inteligentes::{
    crear_inteligente, editar_inteligente, regenerar_automaticas, OrdenInteligente,
    ReglaInteligente,
};

/// Nombre y descripción, en el idioma en uso, de una playlist que crea NB Sound.
///
/// Las que crea NB Sound se nombran solas, así que su nombre tiene que seguir al
/// idioma de la interfaz. Se identifican por su `clave_auto`, que es estable: el
/// nombre guardado es solo la última traducción. `None` para una clave que no es
/// de las que NB Sound nombra.
#[must_use]
pub fn nombre_automatico(clave_auto: &str, artista: Option<&str>) -> Option<(String, String)> {
    let par = |n: (&str, &str, &str), d: (&str, &str, &str)| {
        Some((t(n.0, n.1, n.2).to_owned(), t(d.0, d.1, d.2).to_owned()))
    };
    let analisis = (
        "Selección local creada con el análisis de audio.",
        "Local selection built from audio analysis.",
        "Seleção local criada com a análise de áudio.",
    );
    match clave_auto {
        CLAVE_ME_GUSTA => par(
            ("Me gusta", "Liked", "Curtidas"),
            (
                "Canciones marcadas como favoritas.",
                "Songs marked as favorites.",
                "Músicas marcadas como favoritas.",
            ),
        ),
        "auto:favoritas_sonando" => par(
            (
                "Favoritas que siguen sonando",
                "Favorites still playing",
                "Favoritas que continuam tocando",
            ),
            (
                "Favoritas que también tienen historial de escucha local.",
                "Favorites that also have local listening history.",
                "Favoritas que também têm histórico de escuta local.",
            ),
        ),
        "auto:redescubrir" => par(
            ("Para redescubrir", "To rediscover", "Para redescobrir"),
            (
                "Canciones de tu biblioteca con señales locales para volver.",
                "Songs in your library with local signals to come back to.",
                "Músicas da sua biblioteca com sinais locais para voltar.",
            ),
        ),
        "auto:recientes_biblioteca" => par(
            (
                "Recientes de tu biblioteca",
                "Recent in your library",
                "Recentes da sua biblioteca",
            ),
            (
                "Últimas canciones añadidas a tu biblioteca local.",
                "Latest songs added to your local library.",
                "Últimas músicas adicionadas à sua biblioteca local.",
            ),
        ),
        "auto:descubrir:canciones" => par(
            (
                "Canciones por descubrir",
                "Songs to discover",
                "Músicas para descobrir",
            ),
            (
                "Canciones de tu biblioteca que todavía casi no han sonado.",
                "Songs in your library that have barely played yet.",
                "Músicas da sua biblioteca que ainda quase não tocaram.",
            ),
        ),
        "auto:recientes_sin_escuchar" => par(
            (
                "Recientes sin escuchar",
                "Recent and unplayed",
                "Recentes não ouvidas",
            ),
            (
                "Últimas canciones añadidas que aún no tienen reproducciones.",
                "Latest songs added that have no plays yet.",
                "Últimas músicas adicionadas que ainda não têm reproduções.",
            ),
        ),
        "auto:favoritas_olvidadas" => par(
            (
                "Favoritas que no escuchas hace rato",
                "Favorites you have not played in a while",
                "Favoritas que você não ouve há tempos",
            ),
            (
                "Favoritas que llevan tiempo sin sonar en esta biblioteca.",
                "Favorites that have not played for a while in this library.",
                "Favoritas que não tocam há tempos nesta biblioteca.",
            ),
        ),
        "auto:top_canciones" => par(
            (
                "Tu top canciones",
                "Your top songs",
                "Suas músicas mais tocadas",
            ),
            (
                "Canciones con más reproducciones registradas localmente.",
                "Songs with the most plays recorded locally.",
                "Músicas com mais reproduções registradas localmente.",
            ),
        ),
        "auto:artist_mix_frecuentes" => par(
            (
                "Mix de tus artistas frecuentes",
                "Mix of your regular artists",
                "Mix dos seus artistas frequentes",
            ),
            (
                "Canciones combinadas de los artistas que más suenan en tu biblioteca.",
                "Songs combined from the artists that play most in your library.",
                "Músicas combinadas dos artistas que mais tocam na sua biblioteca.",
            ),
        ),
        "auto:album_mix_frecuentes" => par(
            (
                "Mix de álbumes frecuentes",
                "Mix of frequent albums",
                "Mix de álbuns frequentes",
            ),
            (
                "Canciones de álbumes con más actividad local.",
                "Songs from the albums with the most local activity.",
                "Músicas dos álbuns com mais atividade local.",
            ),
        ),
        "auto:albumes_para_volver" => par(
            (
                "Vuelve a esos álbumes",
                "Back to those albums",
                "Volte a esses álbuns",
            ),
            (
                "Canciones de álbumes con actividad suficiente para retomarlos.",
                "Songs from albums with enough activity to pick them up again.",
                "Músicas de álbuns com atividade suficiente para retomá-los.",
            ),
        ),
        "auto:mood:workout" => par(
            ("Para entrenar", "For working out", "Para treinar"),
            analisis,
        ),
        "auto:mood:party" => par(("Para fiesta", "For a party", "Para festa"), analisis),
        "auto:mood:focus" => par(
            ("Para concentrarme", "To focus", "Para me concentrar"),
            analisis,
        ),
        "auto:mood:chill" => par(
            ("Algo tranquilo", "Something calm", "Algo tranquilo"),
            analisis,
        ),
        "auto:mood:night" => par(
            (
                "Algo de noche",
                "Something for the night",
                "Algo para a noite",
            ),
            analisis,
        ),
        "auto:mood:fast" => par(("Rápidas", "Fast ones", "Rápidas"), analisis),
        "auto:mood:sad_energy" => par(
            (
                "Tristes pero con energía",
                "Sad but energetic",
                "Tristes mas com energia",
            ),
            analisis,
        ),
        _ => {
            let nombre = artista?.trim();
            if !clave_auto.starts_with("auto:this_is:") || nombre.is_empty() {
                return None;
            }
            let d = match crate::idioma::actual() {
                crate::idioma::Idioma::Es => {
                    format!("Canciones de {nombre} en tu biblioteca local.")
                }
                crate::idioma::Idioma::En => format!("Songs by {nombre} in your local library."),
                crate::idioma::Idioma::Pt => {
                    format!("Músicas de {nombre} na sua biblioteca local.")
                }
            };
            Some((format!("This is {nombre}"), d))
        }
    }
}

/// Una automática tal como está guardada: id, clave, regla, nombre y descripción.
type FilaAutomatica = (i64, String, Option<String>, String, Option<String>);

/// Renombra las playlists que nombra NB Sound al idioma en uso, salvo las que el
/// usuario editó. El disparador de sincronización las marca para el teléfono.
/// Devuelve cuántas cambiaron.
///
/// # Errors
///
/// [`NsError::StorageFailure`] si la lectura o la escritura fallan.
pub fn traducir_automaticas(db: &BaseDeDatos) -> NsResult<usize> {
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    let filas: Vec<FilaAutomatica> = {
        let mut s = tx
            .prepare(
                "SELECT id, clave_auto, regla, nombre, descripcion FROM playlists
                 WHERE clave_auto IS NOT NULL AND editada = 0",
            )
            .map_err(|_| NsError::StorageFailure)?;
        let filas = s
            .query_map([], |f| {
                Ok((f.get(0)?, f.get(1)?, f.get(2)?, f.get(3)?, f.get(4)?))
            })
            .map_err(|_| NsError::StorageFailure)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| NsError::StorageFailure)?;
        filas
    };
    let mut cambiadas = 0;
    for (id, clave, regla, nombre, descripcion) in filas {
        let artista = regla
            .as_deref()
            .and_then(|r| serde_json::from_str::<serde_json::Value>(r).ok())
            .and_then(|v| {
                v.get("artista_nombre")
                    .and_then(|n| n.as_str())
                    .map(str::to_owned)
            });
        let Some((nuevo, desc)) = nombre_automatico(&clave, artista.as_deref()) else {
            continue;
        };
        if nuevo == nombre && Some(&desc) == descripcion.as_ref() {
            continue;
        }
        tx.execute(
            "UPDATE playlists SET nombre = ?2, descripcion = ?3,
                 actualizada_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?1",
            rusqlite::params![id, nuevo, desc],
        )
        .map_err(|_| NsError::StorageFailure)?;
        cambiadas += 1;
    }
    tx.commit().map_err(|_| NsError::StorageFailure)?;
    Ok(cambiadas)
}

/// La `clave_auto` de "Me gusta" (`PLAYLIST_FAVORITOS_AUTO_KEY`).
pub const CLAVE_ME_GUSTA: &str = "system:favoritos";
/// Tope del nombre, el.
const MAX_NOMBRE: usize = 120;
/// Cuántas portadas lleva el mosaico de una playlist.
const PORTADAS_DEL_MOSAICO: i64 = 4;

fn bd<T>(r: rusqlite::Result<T>) -> NsResult<T> {
    r.map_err(|error| {
        tracing::error!(error = %error, "falló una consulta de playlists");
        NsError::StorageFailure
    })
}

/// Cómo se agrupa una playlist (también en el móvil).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub enum Categoria {
    MeGusta,
    Creada,
    Inteligente,
    ThisIs,
    Sistema,
}

/// La categoría y la etiqueta.
pub fn clasificar(tipo: &str, subtipo: Option<&str>, origen: &str) -> (Categoria, &'static str) {
    let subtipo = subtipo.filter(|s| !s.is_empty()).unwrap_or(match tipo {
        "manual" => "usuario",
        "sistema" => "sistema",
        _ => tipo,
    });
    let manual = t("Manual", "Manual", "Manual");
    let para_ti = t("Para ti", "For you", "Para você");
    let etiqueta = match subtipo {
        "favoritos" => t("Me gusta", "Liked", "Curtidas"),
        "usuario" => manual,
        "this_is" => "This is...",
        "mood" | "inteligente" => t("Inteligente", "Smart", "Inteligente"),
        "top_canciones" => "Top",
        "top_artistas" | "top_albumes" | "album_mix" | "artist_mix" => "Mix",
        "recientes" | "descubrimiento_local" => para_ti,
        _ if tipo == "automatica" || origen == "generado" => para_ti,
        _ if tipo == "sistema" => t("Sistema", "System", "Sistema"),
        _ => manual,
    };
    let categoria = match subtipo {
        "favoritos" => Categoria::MeGusta,
        "this_is" => Categoria::ThisIs,
        _ if tipo == "manual" && origen == "usuario" => Categoria::Creada,
        _ if tipo == "automatica" || origen == "generado" => Categoria::Inteligente,
        _ if tipo == "sistema" => Categoria::Sistema,
        _ => Categoria::Creada,
    };
    (categoria, etiqueta)
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct PlaylistResumen {
    #[ts(type = "number")]
    pub id: i64,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub categoria: Categoria,
    pub etiqueta: String,
    pub subtipo: Option<String>,
    /// Solo las manuales creadas por el usuario se reordenan y editan pista a pista.
    pub editable: bool,
    /// Las creadas por el usuario (manuales o inteligentes) se pueden borrar.
    pub borrable: bool,
    /// Si es una lista inteligente del usuario, su regla declarativa.
    pub regla_inteligente: Option<ReglaInteligente>,
    pub anclada: bool,
    pub num_pistas: u32,
    pub duracion_seg: f64,
    /// Hasta cuatro pistas con portada, para el mosaico.
    #[ts(type = "number[]")]
    pub mosaico: Vec<i64>,
    pub creada_en: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub enum OrdenPlaylists {
    Recientes,
    Antiguas,
    NombreAsc,
    NombreDesc,
    MasCanciones,
    MenosCanciones,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ConsultaPlaylists {
    pub texto: Option<String>,
    pub categoria: Option<Categoria>,
    pub orden: OrdenPlaylists,
}

/// Tope de playlists que se listan: las del usuario son decenas, y las automáticas
/// las genera la propia aplicación con un número fijo.
pub const MAX_PLAYLISTS: usize = 1_000;

fn resumen(conn: &Connection, f: &rusqlite::Row<'_>) -> rusqlite::Result<PlaylistResumen> {
    let id: i64 = f.get(0)?;
    let tipo: String = f.get(3)?;
    let subtipo: Option<String> = f.get(4)?;
    let origen: String = f.get(5)?;
    let regla_raw: Option<String> = f.get(10)?;
    let (categoria, etiqueta) = clasificar(&tipo, subtipo.as_deref(), &origen);
    let mut s = conn.prepare_cached(
        "SELECT pp.pista_id FROM playlist_pistas pp JOIN pistas p ON p.id = pp.pista_id
         LEFT JOIN albumes a ON a.id = p.album_id
         WHERE pp.playlist_id = ?1 AND coalesce(p.portada_id, a.portada_id) IS NOT NULL
         GROUP BY coalesce(p.portada_id, a.portada_id)
         ORDER BY min(pp.posicion) LIMIT ?2",
    )?;
    let mosaico = s
        .query_map(rusqlite::params![id, PORTADAS_DEL_MOSAICO], |f| f.get(0))?
        .collect::<rusqlite::Result<Vec<i64>>>()?;
    let regla_inteligente = if subtipo.as_deref() == Some("inteligente") && origen == "usuario" {
        regla_raw
            .as_deref()
            .and_then(|j| serde_json::from_str::<ReglaInteligente>(j).ok())
    } else {
        None
    };
    Ok(PlaylistResumen {
        id,
        nombre: f.get(1)?,
        descripcion: f.get(2)?,
        categoria,
        etiqueta: etiqueta.to_owned(),
        subtipo,
        editable: tipo == "manual" && origen == "usuario",
        borrable: origen == "usuario",
        regla_inteligente,
        anclada: f.get(6)?,
        num_pistas: f.get::<_, i64>(7)? as u32,
        duracion_seg: f.get(8)?,
        mosaico,
        creada_en: f.get(9)?,
    })
}

const COLUMNAS: &str =
    "pl.id, pl.nombre, pl.descripcion, pl.tipo, pl.subtipo, pl.origen, pl.anclada,
    (SELECT count(*) FROM playlist_pistas pp JOIN pistas p ON p.id = pp.pista_id
     WHERE pp.playlist_id = pl.id AND p.estado = 'biblioteca'),
    (SELECT total(p.duracion_seg) FROM playlist_pistas pp JOIN pistas p ON p.id = pp.pista_id
     WHERE pp.playlist_id = pl.id AND p.estado = 'biblioteca'),
    pl.creada_en, pl.regla";

/// Las playlists visibles, con las ancladas delante.
///
/// # Errors
///
/// [`NsError::StorageFailure`] si la consulta falla.
pub fn listar(db: &BaseDeDatos, c: &ConsultaPlaylists) -> NsResult<Vec<PlaylistResumen>> {
    inteligentes::refrescar_inteligentes(db)?;
    let conn = db.conexion()?;
    let orden = match c.orden {
        OrdenPlaylists::Recientes => "pl.creada_en DESC, pl.id DESC",
        OrdenPlaylists::Antiguas => "pl.creada_en, pl.id",
        OrdenPlaylists::NombreAsc => "pl.nombre COLLATE NOCASE, pl.id",
        OrdenPlaylists::NombreDesc => "pl.nombre COLLATE NOCASE DESC, pl.id",
        OrdenPlaylists::MasCanciones => "8 DESC, pl.id",
        OrdenPlaylists::MenosCanciones => "8, pl.id",
    };
    let sql = format!(
        "SELECT {COLUMNAS} FROM playlists pl WHERE pl.visible = 1
         ORDER BY pl.anclada DESC, {orden} LIMIT {MAX_PLAYLISTS}"
    );
    let mut s = bd(conn.prepare(&sql))?;
    let todas =
        bd(bd(s.query_map([], |f| resumen(&conn, f)))?.collect::<rusqlite::Result<Vec<_>>>())?;
    let texto = c
        .texto
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(normalizar);
    Ok(todas
        .into_iter()
        .filter(|p| c.categoria.is_none_or(|cat| p.categoria == cat))
        .filter(|p| {
            texto.as_ref().is_none_or(|t| {
                normalizar(&p.nombre).contains(t.as_str())
                    || normalizar(&p.etiqueta).contains(t.as_str())
            })
        })
        .collect())
}

/// Minúsculas y sin tildes, para buscar por nombre como en el resto.
fn normalizar(texto: &str) -> String {
    texto
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            otro => otro,
        })
        .collect()
}

/// Una playlist por su `id`.
///
/// # Errors
///
/// [`NsError::StorageFailure`] si la consulta falla.
pub fn una(db: &BaseDeDatos, id: i64) -> NsResult<Option<PlaylistResumen>> {
    inteligentes::refrescar_inteligente_si_aplica(db, id)?;
    let conn = db.conexion()?;
    bd(conn
        .query_row(
            &format!("SELECT {COLUMNAS} FROM playlists pl WHERE pl.id = ?1"),
            [id],
            |f| resumen(&conn, f),
        )
        .optional())
}

/// Un tramo de las canciones de una playlist, en su orden.
///
/// # Errors
///
/// [`NsError::StorageFailure`] si la consulta falla.
pub fn pistas(db: &BaseDeDatos, id: i64, desde: u32, limite: u32) -> NsResult<PaginaPistas> {
    let ids = ids(db, id)?;
    let total = ids.len() as u32;
    let desde = (desde as usize).min(ids.len());
    let hasta = (desde + limite.min(crate::biblioteca::MAX_TRAMO) as usize).min(ids.len());
    Ok(PaginaPistas {
        total,
        pistas: crate::biblioteca::resumenes(db, &ids[desde..hasta])?,
    })
}

/// Los `id` de las canciones visibles de una playlist, en orden.
///
/// # Errors
///
/// [`NsError::StorageFailure`] si la consulta falla.
pub fn ids(db: &BaseDeDatos, id: i64) -> NsResult<Vec<i64>> {
    inteligentes::refrescar_inteligente_si_aplica(db, id)?;
    let conn = db.conexion()?;
    let mut s = bd(conn.prepare_cached(
        "SELECT pp.pista_id FROM playlist_pistas pp JOIN pistas p ON p.id = pp.pista_id
         WHERE pp.playlist_id = ?1 AND p.estado = 'biblioteca' ORDER BY pp.posicion, pp.pista_id",
    ))?;
    let ids = bd(bd(s.query_map([id], |f| f.get(0)))?.collect::<rusqlite::Result<Vec<i64>>>())?;
    Ok(ids)
}

fn validar_nombre(tx: &Transaction<'_>, nombre: &str, excluir: Option<i64>) -> NsResult<String> {
    // Los espacios repetidos cuentan como uno.
    let limpio = nombre.split_whitespace().collect::<Vec<_>>().join(" ");
    if limpio.is_empty() || limpio.chars().count() > MAX_NOMBRE {
        return Err(NsError::InvalidInput);
    }
    // En Rust y no con `lower()` de SQLite, que solo sabe de ASCII: "RÁPIDAS" y
    // "rápidas" son el mismo nombre.
    let mut s =
        bd(tx.prepare_cached("SELECT nombre FROM playlists WHERE visible = 1 AND id IS NOT ?1"))?;
    let buscado = limpio.to_lowercase();
    let repetida = bd(bd(s.query_map([excluir], |f| f.get::<_, String>(0)))?
        .collect::<rusqlite::Result<Vec<_>>>())?
    .into_iter()
    .find(|n| n.to_lowercase() == buscado);
    if repetida.is_some() {
        return Err(NsError::NameTaken);
    }
    Ok(limpio)
}

fn exigir_editable(tx: &Transaction<'_>, id: i64) -> NsResult<()> {
    let editable: Option<bool> = bd(tx
        .query_row(
            "SELECT tipo = 'manual' AND origen = 'usuario' FROM playlists WHERE id = ?1",
            [id],
            |f| f.get(0),
        )
        .optional())?;
    match editable {
        Some(true) => Ok(()),
        Some(false) => Err(NsError::PathNotAllowed),
        None => Err(NsError::FileNotFound),
    }
}

/// Crea una playlist manual vacía y devuelve su `id`.
///
/// # Errors
///
/// [`NsError::InvalidInput`] si el nombre está vacío o es demasiado largo;
/// [`NsError::NameTaken`] si ya hay una con ese nombre.
pub fn crear(db: &BaseDeDatos, nombre: &str, descripcion: Option<&str>) -> NsResult<i64> {
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    let nombre = validar_nombre(&tx, nombre, None)?;
    bd(tx.execute(
        "INSERT INTO playlists (nombre, descripcion, tipo, subtipo, origen)
         VALUES (?1, ?2, 'manual', 'usuario', 'usuario')",
        rusqlite::params![nombre, descripcion.map(str::trim).filter(|d| !d.is_empty())],
    ))?;
    let id = tx.last_insert_rowid();
    bd(tx.commit())?;
    Ok(id)
}

/// # Errors
///
/// Los de [`crear`], y [`NsError::PathNotAllowed`] si no es una playlist manual.
pub fn renombrar(
    db: &BaseDeDatos,
    id: i64,
    nombre: &str,
    descripcion: Option<&str>,
) -> NsResult<()> {
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    exigir_editable(&tx, id)?;
    let nombre = validar_nombre(&tx, nombre, Some(id))?;
    bd(tx.execute(
        "UPDATE playlists SET nombre = ?2, descripcion = ?3,
             actualizada_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?1",
        rusqlite::params![
            id,
            nombre,
            descripcion.map(str::trim).filter(|d| !d.is_empty())
        ],
    ))?;
    bd(tx.commit())
}

/// Borra una playlist creada por el usuario (manual o inteligente). El disparador de
/// V5 deja la tumba para el móvil.
///
/// # Errors
///
/// [`NsError::PathNotAllowed`] si no es una playlist del usuario.
pub fn borrar(db: &BaseDeDatos, id: i64) -> NsResult<()> {
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    let borrable: Option<bool> = bd(tx
        .query_row(
            "SELECT origen = 'usuario' FROM playlists WHERE id = ?1",
            [id],
            |f| f.get(0),
        )
        .optional())?;
    match borrable {
        Some(true) => {}
        Some(false) => return Err(NsError::PathNotAllowed),
        None => return Err(NsError::FileNotFound),
    }
    bd(tx.execute("DELETE FROM playlists WHERE id = ?1", [id]))?;
    bd(tx.commit())
}

/// Las playlists manuales que ya tienen la canción, para que "Añadir a playlist" las marque y deje quitarla.
///
/// # Errors
///
/// [`NsError::StorageFailure`] si la consulta falla.
pub fn con_pista(db: &BaseDeDatos, pista: i64) -> NsResult<Vec<i64>> {
    let conn = db.conexion()?;
    let mut s = bd(conn.prepare_cached(
        "SELECT pp.playlist_id FROM playlist_pistas pp JOIN playlists p ON p.id = pp.playlist_id
         WHERE pp.pista_id = ?1 AND p.tipo = 'manual' AND p.origen = 'usuario'",
    ))?;
    let ids = bd(bd(s.query_map([pista], |f| f.get(0)))?.collect::<rusqlite::Result<Vec<i64>>>())?;
    Ok(ids)
}

/// Duplica cualquier playlist como una manual del usuario, con las mismas
/// canciones en el mismo orden. Se llama como la
/// original con "(copia)", y "(copia 2)", "(copia 3)"… si ese ya existe. Una
/// automática o inteligente se copia tal como está ahora: la copia ya no cambia
/// sola, que es para lo que se duplica. Devuelve el `id` de la copia.
///
/// # Errors
///
/// [`NsError::FileNotFound`] si no existe.
pub fn duplicar(db: &BaseDeDatos, id: i64) -> NsResult<i64> {
    let orden = ids(db, id)?;
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    let (nombre, descripcion): (String, Option<String>) = bd(tx
        .query_row(
            "SELECT nombre, descripcion FROM playlists WHERE id = ?1",
            [id],
            |f| Ok((f.get(0)?, f.get(1)?)),
        )
        .optional())?
    .ok_or(NsError::FileNotFound)?;
    let mut n = 1;
    let nombre = loop {
        let candidato = if n == 1 {
            format!("{nombre} (copia)")
        } else {
            format!("{nombre} (copia {n})")
        };
        match validar_nombre(&tx, &candidato, None) {
            Err(NsError::NameTaken) => n += 1,
            otro => break otro?,
        }
    };
    bd(tx.execute(
        "INSERT INTO playlists (nombre, descripcion, tipo, subtipo, origen)
         VALUES (?1, ?2, 'manual', 'usuario', 'usuario')",
        rusqlite::params![nombre, descripcion],
    ))?;
    let copia = tx.last_insert_rowid();
    for (posicion, pista) in orden.iter().enumerate() {
        bd(tx.execute(
            "INSERT INTO playlist_pistas (playlist_id, pista_id, posicion) VALUES (?1, ?2, ?3)",
            rusqlite::params![copia, pista, posicion as i64],
        ))?;
    }
    bd(tx.commit())?;
    Ok(copia)
}

/// Quita todas las canciones de una playlist manual.
///
/// # Errors
///
/// [`NsError::PathNotAllowed`] si no es una playlist manual.
pub fn vaciar(db: &BaseDeDatos, id: i64) -> NsResult<()> {
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    exigir_editable(&tx, id)?;
    bd(tx.execute("DELETE FROM playlist_pistas WHERE playlist_id = ?1", [id]))?;
    bd(tx.execute(
        "UPDATE playlists SET actualizada_en = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
        [id],
    ))?;
    bd(tx.commit())
}

/// Añade canciones al final, sin repetir las que ya estaban. Devuelve cuántas
/// entraron.
///
/// # Errors
///
/// [`NsError::PathNotAllowed`] si no es una playlist manual.
pub fn agregar(db: &BaseDeDatos, id: i64, pistas: &[i64]) -> NsResult<u32> {
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    exigir_editable(&tx, id)?;
    let mut siguiente: i64 = bd(tx.query_row(
        "SELECT coalesce(max(posicion) + 1, 0) FROM playlist_pistas WHERE playlist_id = ?1",
        [id],
        |f| f.get(0),
    ))?;
    let mut entraron = 0;
    for pista in pistas.iter().take(crate::biblioteca::MAX_RESUMENES * 10) {
        let n = bd(tx.execute(
            "INSERT OR IGNORE INTO playlist_pistas (playlist_id, pista_id, posicion)
             SELECT ?1, id, ?3 FROM pistas WHERE id = ?2",
            rusqlite::params![id, pista, siguiente],
        ))?;
        if n > 0 {
            siguiente += 1;
            entraron += 1;
        }
    }
    bd(tx.commit())?;
    Ok(entraron)
}

/// Quita una canción.
///
/// # Errors
///
/// [`NsError::PathNotAllowed`] si no es una playlist manual.
pub fn quitar(db: &BaseDeDatos, id: i64, pista: i64) -> NsResult<()> {
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    exigir_editable(&tx, id)?;
    bd(tx.execute(
        "DELETE FROM playlist_pistas WHERE playlist_id = ?1 AND pista_id = ?2",
        [id, pista],
    ))?;
    bd(tx.commit())
}

/// Mueve la canción de la posición `desde` a la `hasta` (posiciones en la lista
/// visible) y renumera, para que las posiciones queden seguidas.
///
/// # Errors
///
/// [`NsError::PathNotAllowed`] si no es una playlist manual.
pub fn mover(db: &BaseDeDatos, id: i64, desde: usize, hasta: usize) -> NsResult<()> {
    let mut orden = ids(db, id)?;
    if desde >= orden.len() || hasta >= orden.len() || desde == hasta {
        return Ok(());
    }
    let pista = orden.remove(desde);
    orden.insert(hasta, pista);
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    exigir_editable(&tx, id)?;
    for (i, p) in orden.iter().enumerate() {
        bd(tx.execute(
            "UPDATE playlist_pistas SET posicion = ?3
             WHERE playlist_id = ?1 AND pista_id = ?2 AND posicion <> ?3",
            rusqlite::params![id, p, i as i64],
        ))?;
    }
    bd(tx.commit())
}

/// Ancla o desancla cualquier playlist: las ancladas van delante.
///
/// # Errors
///
/// [`NsError::StorageFailure`] si no se puede escribir.
pub fn anclar(db: &BaseDeDatos, id: i64, anclada: bool) -> NsResult<()> {
    let mut conn = db.conexion()?;
    let tx = db::transaccion_de_escritura(&mut conn)?;
    bd(tx.execute(
        "UPDATE playlists SET anclada = ?2,
             anclada_en = CASE WHEN ?2 THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now') END
         WHERE id = ?1",
        rusqlite::params![id, anclada],
    ))?;
    bd(tx.commit())
}

/// Mantiene "Me gusta" al marcar o desmarcar una favorita, dentro de la misma
/// transacción. La crea si no existe, con los valores. La recién marcada
/// va primera (lo último escuchado o marcado, arriba).
///
/// # Errors
///
/// [`NsError::StorageFailure`] si no se puede escribir.
pub fn reflejar_favorita(tx: &Transaction<'_>, pista: i64, favorita: bool) -> NsResult<()> {
    let id: Option<i64> = bd(tx
        .query_row(
            "SELECT id FROM playlists WHERE clave_auto = ?1",
            [CLAVE_ME_GUSTA],
            |f| f.get(0),
        )
        .optional())?;
    let id = match id {
        Some(id) => id,
        None => {
            let (nombre, descripcion) = nombre_automatico(CLAVE_ME_GUSTA, None).unwrap_or_default();
            bd(tx.execute(
                "INSERT INTO playlists (nombre, descripcion, tipo, subtipo, origen, regla,
                                        clave_auto, anclada, auto_actualizable)
                 VALUES (?2, ?3, 'sistema',
                         'favoritos', 'sistema', '{\"tipo\":\"favoritos\"}', ?1, 1, 1)",
                rusqlite::params![CLAVE_ME_GUSTA, nombre, descripcion],
            ))?;
            let id = tx.last_insert_rowid();
            // Nace con todas las favoritas que ya hubiera, no solo con esta: pudo
            // haberlas marcado el teléfono antes de que existiera.
            bd(tx.execute(
                "INSERT INTO playlist_pistas (playlist_id, pista_id, posicion)
                 SELECT ?1, id, row_number() OVER (ORDER BY favorita_actualizada_en DESC, id) - 1
                 FROM pistas WHERE favorita = 1 AND estado = 'biblioteca'",
                [id],
            ))?;
            id
        }
    };
    if favorita {
        let ya: bool = bd(tx.query_row(
            "SELECT count(*) > 0 FROM playlist_pistas WHERE playlist_id = ?1 AND pista_id = ?2",
            [id, pista],
            |f| f.get(0),
        ))?;
        if !ya {
            bd(tx.execute(
                "UPDATE playlist_pistas SET posicion = posicion + 1 WHERE playlist_id = ?1",
                [id],
            ))?;
            bd(tx.execute(
                "INSERT INTO playlist_pistas (playlist_id, pista_id, posicion) VALUES (?1, ?2, 0)",
                [id, pista],
            ))?;
        }
    } else {
        bd(tx.execute(
            "DELETE FROM playlist_pistas WHERE playlist_id = ?1 AND pista_id = ?2",
            [id, pista],
        ))?;
    }
    Ok(())
}

/// Como [`reflejar_favorita`], pero sin crear "Me gusta" si no existe: el merge
/// del teléfono no tiene por qué crear playlists en el equipo. Cuando se cree, nacerá
/// con todas las favoritas.
///
/// # Errors
///
/// [`NsError::StorageFailure`] si no se puede escribir.
pub fn reflejar_favorita_si_existe(
    tx: &Transaction<'_>,
    pista: i64,
    favorita: bool,
) -> NsResult<()> {
    let existe: bool = bd(tx.query_row(
        "SELECT count(*) > 0 FROM playlists WHERE clave_auto = ?1",
        [CLAVE_ME_GUSTA],
        |f| f.get(0),
    ))?;
    if existe {
        reflejar_favorita(tx, pista, favorita)?;
    }
    Ok(())
}

#[cfg(test)]
mod pruebas;
