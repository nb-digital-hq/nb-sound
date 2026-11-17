//! De las etiquetas crudas, AcoustID y el nombre del archivo a unos datos listos
//! para buscar.
//!
//! Primero las etiquetas; AcoustID sube la confianza; y el nombre del archivo es el
//! último recurso, sin títulos genéricos.

use super::etiquetas::Cruda;
use super::modelo::{CausaCuarentena, DeAcoustid, Fuente, Normalizada};
use super::texto::{
    extraer_anio_del_texto, extraer_featuring_del_titulo, limpiar_numero_pista,
    limpiar_version_titulo, normalizar_artista, normalizar_base, normalizar_isrc,
    normalizar_titulo, para_comparacion, parsear_nombre_archivo, separar_artista_principal,
};

const MIN_ARTISTA: usize = 2;
const MIN_TITULO: usize = 1;
const MIN_ARTISTA_NOMBRE_ARCHIVO: usize = 3;

const TITULOS_GENERICOS: [&str; 19] = [
    "track",
    "audio",
    "music",
    "song",
    "untitled",
    "sin titulo",
    "unknown",
    "pista",
    "tema",
    "cancion",
    "new recording",
    "recording",
    "new track",
    "vocal",
    "instrumental",
    "demo",
    "draft",
    "test",
    "sample",
];

fn largo(s: &str) -> usize {
    s.chars().count()
}

fn desde_etiquetas(c: &Cruda, n: &mut Normalizada) {
    let titulo = normalizar_base(c.titulo.as_deref().unwrap_or_default());
    let (sin_feat, feat_titulo) = extraer_featuring_del_titulo(&titulo);
    n.titulo = normalizar_titulo(&sin_feat);
    n.titulo_para_match = para_comparacion(&limpiar_version_titulo(&n.titulo));
    let artista = normalizar_base(c.artista.as_deref().unwrap_or_default());
    let (principal, feat_artista) = separar_artista_principal(&artista);
    n.artista = normalizar_artista(&principal);
    n.artista_para_match = para_comparacion(&n.artista);
    n.featuring = feat_titulo.or(feat_artista);
    n.album = normalizar_base(c.album.as_deref().unwrap_or_default());
    n.album_para_match = para_comparacion(&n.album);
    n.numero = limpiar_numero_pista(c.numero.as_deref().unwrap_or_default());
    n.anio = extraer_anio_del_texto(c.anio.as_deref().unwrap_or_default());
    // El ISRC de las etiquetas, si lo trae: identifica la grabación sin preguntar a
    // nadie, y MusicBrainz lo busca antes que el texto.
    n.isrc = c.isrc.as_deref().and_then(normalizar_isrc);
    let (t, a) = (
        largo(&n.titulo_para_match) >= MIN_TITULO,
        largo(&n.artista_para_match) >= MIN_ARTISTA,
    );
    n.confianza = match (t, a) {
        (true, true) => 0.40,
        (true, false) | (false, true) => 0.20,
        (false, false) => 0.0,
    };
}

fn fusionar(n: &mut Normalizada, acoustid: Option<&DeAcoustid>) {
    if let Some(a) = acoustid.filter(|a| !a.grabaciones.is_empty()) {
        let m = a.mejor();
        let b = if m >= 0.90 {
            0.25
        } else if m >= 0.75 {
            0.15
        } else {
            0.05
        };
        n.confianza = f64::min(1.0, n.confianza + b);
    }
}

fn desde_nombre(nombre: &str, n: &mut Normalizada, tiene_artista: bool, tiene_titulo: bool) {
    let inferido = parsear_nombre_archivo(nombre);
    if let (false, Some(t)) = (tiene_titulo, inferido.titulo.as_deref()) {
        let t = normalizar_titulo(t);
        let tn = para_comparacion(&t);
        let generico = TITULOS_GENERICOS.contains(&tn.as_str()) || largo(&tn) <= 2;
        if !generico {
            n.titulo_para_match = para_comparacion(&limpiar_version_titulo(&t));
            n.titulo = t;
            n.fuente_titulo = Fuente::NombreArchivo;
        }
    }
    if let (false, Some(a)) = (tiene_artista, inferido.artista.as_deref()) {
        let a = normalizar_artista(a);
        let an = para_comparacion(&a);
        if largo(&an) >= MIN_ARTISTA_NOMBRE_ARCHIVO {
            n.artista = a;
            n.artista_para_match = an;
            n.fuente_artista = Fuente::NombreArchivo;
        }
    }
    if n.numero.is_none() {
        n.numero = inferido.numero.and_then(|x| x.parse().ok());
    }
}

/// `normalizar_metadata`: los datos, o la cuarentena si no queda ni título ni
/// artista. Con uno de los dos basta para buscar.
///
/// # Errors
///
/// [`CausaCuarentena::MetadataInsuficiente`] sin título ni artista utilizables.
pub fn normalizar(
    cruda: &Cruda,
    duracion_seg: f64,
    nombre_archivo: &str,
    acoustid: Option<&DeAcoustid>,
) -> Result<Normalizada, CausaCuarentena> {
    let mut n = Normalizada {
        duracion_seg: Some(duracion_seg).filter(|d| *d > 0.0),
        ..Normalizada::default()
    };
    desde_etiquetas(cruda, &mut n);
    fusionar(&mut n, acoustid);
    let tiene = |n: &Normalizada| {
        (
            largo(&n.artista_para_match) >= MIN_ARTISTA,
            largo(&n.titulo_para_match) >= MIN_TITULO,
        )
    };
    let (a, t) = tiene(&n);
    if !a || !t {
        desde_nombre(nombre_archivo, &mut n, a, t);
    }
    match tiene(&n) {
        (false, false) => Err(CausaCuarentena::MetadataInsuficiente),
        _ => Ok(n),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cruda(titulo: &str, artista: &str) -> Cruda {
        Cruda {
            titulo: Some(titulo.to_owned()).filter(|s| !s.is_empty()),
            artista: Some(artista.to_owned()).filter(|s| !s.is_empty()),
            numero: Some("3/12".into()),
            anio: Some("2017-05-01".into()),
            ..Cruda::default()
        }
    }

    #[test]
    fn las_etiquetas_dan_titulo_artista_y_featuring() {
        let n = normalizar(
            &cruda(
                "Despacito (feat. Daddy Yankee) [Official Video]",
                "Luis Fonsi",
            ),
            228.0,
            "x.mp3",
            None,
        )
        .unwrap();
        assert_eq!(n.titulo, "Despacito");
        assert_eq!(n.titulo_para_match, "despacito");
        assert_eq!(
            (n.artista.as_str(), n.featuring.as_deref()),
            ("Luis Fonsi", Some("Daddy Yankee"))
        );
        assert_eq!((n.numero, n.anio), (Some(3), Some(2017)));
        assert_eq!(n.confianza, 0.40);
        assert_eq!(n.fuente_titulo, Fuente::TagLocal);
    }

    #[test]
    fn acoustid_suma_segun_su_confianza() {
        let a = DeAcoustid {
            grabaciones: vec!["r".into()],
            confianzas: vec![0.93],
        };
        let n = normalizar(&cruda("Song", "Artist"), 200.0, "x.mp3", Some(&a)).unwrap();
        assert!((n.confianza - 0.65).abs() < 1e-9);
    }

    #[test]
    fn el_nombre_del_archivo_es_el_ultimo_recurso() {
        let n = normalizar(
            &Cruda::default(),
            200.0,
            "Maná - 03 - Clavado en un bar.mp3",
            None,
        )
        .unwrap();
        assert_eq!(
            (n.artista.as_str(), n.titulo.as_str(), n.numero),
            ("Maná", "Clavado en un bar", Some(3))
        );
        assert_eq!(n.fuente_artista, Fuente::NombreArchivo);
        assert_eq!(
            normalizar(&Cruda::default(), 200.0, "track.mp3", None),
            Err(CausaCuarentena::MetadataInsuficiente),
            "un título genérico sin artista no sirve"
        );
    }
}
