//! El prefiltro: qué pistas merece la pena analizar a fondo.
//!
//! El análisis estructural cuesta segundos por pista y la separación, minutos.
//! Antes de pagarlos se usan los rasgos que la biblioteca ya tiene (tempo,
//! tonalidad, energía, de la tabla `rasgos`) para quedarse con un grupo
//! compatible con un ancla. Lo que no pasa este filtro nunca podría sonar junto
//! en el mashup: no tiene sentido analizarlo.

use super::estiramiento::{SEMITONOS_MAX, TEMPO_MAX};
use super::estructura::plegar;
use crate::dj::azar::Azar;
use crate::dj::planificador::Candidata;
use crate::dj::transiciones::{camelot, distancia_camelot};

const NOTAS: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Si `b` puede sonar con `a` como ancla: tempo plegado a ±8 % y tonalidad a
/// un paso de Camelot con una transposición de ±2 semitonos. Sin tonalidad, solo
/// cuenta el tempo. Con el mismo modo, la transposición siempre alcanza una
/// vecina (los tres vecinos mayores de una tonalidad cubren la rueda a ±2), así
/// que en la práctica filtra el tempo y el modo.
#[must_use]
pub fn compatibles(a: &Candidata, b: &Candidata) -> bool {
    let (Some(ta), Some(tb)) = (a.bpm, b.bpm) else {
        return false;
    };
    let (ta, tb) = (plegar(ta), plegar(tb));
    if (ta / tb - 1.0).abs() > TEMPO_MAX {
        return false;
    }
    let Some(ca) = camelot(a.tonalidad.as_deref(), a.modo.as_deref()) else {
        return true;
    };
    let Some(pb) = super::compositor::tonica(b.tonalidad.as_deref()) else {
        return true;
    };
    (-SEMITONOS_MAX..=SEMITONOS_MAX).any(|s| {
        let pc = (pb as i32 + s).rem_euclid(12) as usize;
        camelot(Some(NOTAS[pc]), b.modo.as_deref()).is_some_and(|cb| distancia_camelot(ca, cb) <= 1)
    })
}

/// Elige hasta `cupo` pistas compatibles entre sí alrededor de un ancla.
/// Determinista con la semilla. Las favoritas y las más escuchadas pesan algo
/// más: un mashup de lo que uno escucha suena más suyo.
#[must_use]
pub fn preseleccionar(candidatas: &[Candidata], cupo: usize, semilla: i64) -> Vec<i64> {
    let utiles: Vec<&Candidata> = candidatas.iter().filter(|c| c.bpm.is_some()).collect();
    if utiles.is_empty() || cupo == 0 {
        return Vec::new();
    }
    let mut azar = Azar::con_semilla(semilla);
    // No se prueba cada pista como ancla en bibliotecas enormes: 64 anclas
    // repartidas bastan para encontrar el grupo grande, y el coste queda en
    // 64 × n comparaciones baratas.
    let paso = (utiles.len() / 64).max(1);
    let desfase = (azar.real() * paso as f64) as usize;
    let mut mejor: Option<(f64, usize)> = None;
    for i in (desfase..utiles.len()).step_by(paso) {
        let n = utiles.iter().filter(|b| compatibles(utiles[i], b)).count() as f64;
        let v = n + azar.uniforme(0.0, n * 0.15 + 0.5);
        if mejor.is_none_or(|(m, _)| v > m) {
            mejor = Some((v, i));
        }
    }
    let Some((_, ancla)) = mejor else {
        return Vec::new();
    };
    let a = utiles[ancla];
    let mut grupo: Vec<(f64, i64)> = utiles
        .iter()
        .filter(|b| compatibles(a, b))
        .map(|b| {
            let tempo =
                (plegar(a.bpm.unwrap_or(120.0)) / plegar(b.bpm.unwrap_or(120.0)) - 1.0).abs();
            let gusto =
                if b.favorita { 0.3 } else { 0.0 } + (b.reproducciones as f64).ln_1p() * 0.05;
            let p = 1.0 - tempo / TEMPO_MAX * 0.5 + gusto + azar.uniforme(0.0, 0.3);
            (p, b.id)
        })
        .collect();
    grupo.sort_by(|x, y| y.0.total_cmp(&x.0).then(x.1.cmp(&y.1)));
    let mut ids: Vec<i64> = vec![a.id];
    ids.extend(grupo.into_iter().map(|g| g.1).filter(|id| *id != a.id));
    ids.truncate(cupo);
    ids
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn c(id: i64, bpm: f64, ton: &str, modo: &str) -> Candidata {
        Candidata {
            id,
            bpm: Some(bpm),
            tonalidad: Some(ton.into()),
            modo: Some(modo.into()),
            ..Candidata::default()
        }
    }

    #[test]
    fn la_compatibilidad_pliega_tempos_y_admite_transponer() {
        assert!(compatibles(
            &c(1, 120.0, "C", "major"),
            &c(2, 62.0, "G", "major")
        ));
        assert!(!compatibles(
            &c(1, 120.0, "C", "major"),
            &c(2, 100.0, "C", "major")
        ));
        // Con ±2 semitonos toda tonalidad alcanza una vecina de Camelot: fa#
        // mayor bajada uno es fa (7B), vecina de do (8B). La tonalidad no
        // excluye; decide cuánto hay que transponer, y eso lo cobra el
        // compositor.
        for t in NOTAS {
            assert!(
                compatibles(&c(1, 120.0, "C", "major"), &c(2, 120.0, t, "major")),
                "{t}"
            );
            assert!(
                compatibles(&c(1, 120.0, "A", "minor"), &c(2, 120.0, t, "minor")),
                "{t}m"
            );
        }
    }

    #[test]
    fn preselecciona_el_grupo_compatible_y_es_reproducible() {
        let mut v: Vec<Candidata> = (0..30)
            .map(|i| c(i, 120.0 + (i % 5) as f64, "C", "major"))
            .collect();
        v.extend((30..40).map(|i| c(i, 90.0, "F#", "major")));
        v.push(Candidata {
            id: 99,
            ..Candidata::default()
        });
        let a = preseleccionar(&v, 20, 3);
        assert_eq!(a.len(), 20);
        assert!(a.iter().all(|id| *id < 30));
        assert_eq!(a, preseleccionar(&v, 20, 3));
        assert!(preseleccionar(&[], 5, 1).is_empty());
    }

    #[test]
    fn escala_a_bibliotecas_grandes_sin_coste_cuadratico() {
        let v: Vec<Candidata> = (0..5000)
            .map(|i| c(i, 80.0 + (i % 80) as f64, NOTAS[(i % 12) as usize], "major"))
            .collect();
        let t = std::time::Instant::now();
        let a = preseleccionar(&v, 150, 1);
        assert!(!a.is_empty());
        assert!(t.elapsed().as_secs_f64() < 2.0, "{:?}", t.elapsed());
    }
}
