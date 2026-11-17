//! El compositor (ADR-018): de un conjunto de pistas analizadas a un
//! plan de capas sobre una rejilla de compases.
//!
//! **Cómo decide.** La composición se divide en frases de 4, 8 o 16 compases.
//! En cada frase suena una base y, según convenga, una voz de otra pista y, en
//! Fiesta, una batería ajena. La base puede ser el instrumental de una pista o
//! su **mezcla original** con su propia voz: reproducir el original es una
//! forma legítima de integrar una canción, no un recurso de emergencia. Una
//! búsqueda por haces recorre las frases en orden; cada estado parcial se
//! puntúa con términos explícitos y solo sobreviven los mejores. Es
//! determinista: el azar solo desempata y sale de la semilla.
//!
//! **Tres compatibilidades distintas.** Superponer (una voz sobre la base de
//! otra pista) exige armonía compás a compás, pulsos fiables y que las dos
//! pistas no estén en fases opuestas de un patrón de dos compases. Pasar de una
//! base a otra solo exige un punto de cambio razonable: con armonía y ritmo
//! compatibles en la frontera se cruzan con intercambio de graves; si no, se
//! corta en el primer tiempo donde una sección termina. Reproducir el original
//! no exige nada más que el tempo. Una pareja incompatible para superponerse
//! sigue siendo perfectamente válida en secuencia.
//!
//! **Restricciones estrictas y preferencias graduadas.** Son estrictas las que
//! evitan defectos graves: el tempo (±8 %), la transposición (±2 semitonos),
//! dos voces a la vez, una voz sobre una mezcla con voz, una voz sobre una base
//! que choca armónicamente o en clave con alta confianza. Todo lo demás
//! (tonalidad, energía, variedad, puntos de corte) se cobra en la puntuación.
//!
//! **Por qué haces y no búsqueda exhaustiva.** El haz explora un número fijo de
//! estados por frase (anchura × candidatos), así que el coste crece linealmente
//! con la duración y con el tamaño del catálogo prefiltrado. Las decisiones
//! locales que obligarían a malas transiciones al final se evitan con una
//! presión de cobertura que crece cuando faltan canciones y quedan pocas
//! frases, y con el cierre puntuado como parte del plan.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::estiramiento::{SEMITONOS_MAX, TEMPO_MAX};
use super::estructura::{Estructura, TipoSeccion, PASOS};
use crate::dj::azar::Azar;
use crate::dj::transiciones::{camelot, distancia_camelot};
use crate::error::{NsError, NsResult};
use crate::karaoke::Stem;

/// Sube con cualquier cambio que altere el plan de una misma petición.
pub const VERSION: u32 = 3;
const NOTAS: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];
/// Puntuación mínima de una frase para aceptarla.
const UMBRAL_FRASE: f64 = 0.2;
/// Una voz no se superpone a una base cuyo croma medio choca por debajo de esto.
const UMBRAL_CROMA_VOZ: f64 = 0.6;
/// Ni compás a compás: una voz cuyo acorde coincide de media pero choca en
/// muchos compases suena fuera de sitio.
const UMBRAL_CROMA_COMPAS: f64 = 0.45;
/// Un compás con un coseno por debajo de esto choca; si chocan más de este
/// tanto de los compases, la superposición se descarta aunque la media pase.
const CHOQUE_COMPAS: f64 = 0.5;
const CHOQUES_MAX: f64 = 0.3;
/// Superponer una voz a una base exige pulsos claros en las dos: con una
/// rejilla dudosa la voz cae a destiempo.
const PULSO_CAPAS: f64 = 0.5;
/// Un fragmento canta si su voz está activa al menos esta fracción del tiempo.
const CANTA: f64 = 0.4;
/// Lo mínimo para que una canción cuente como integrada: 4 compases y 8 s.
const COMPASES_MINIMOS: usize = 4;
const SEGUNDOS_MINIMOS: f64 = 8.0;
/// Conflicto de clave por encima del cual no se superpone.
const CLAVE_MAX: f64 = 0.5;

/// Lo que quiere el usuario.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub enum Estilo {
    /// Equilibrado: frases de 8, voz sobre base, energía en arco.
    Equilibrado,
    /// Progresión con subidas, descansos y clímax; capas densas en los picos.
    Fiesta,
    /// Pocas capas, transiciones largas.
    Suave,
    /// Frases cortas y alternancia rápida: muchas canciones.
    Relampago,
}

impl Estilo {
    fn frase(self) -> usize {
        match self {
            Self::Equilibrado | Self::Fiesta => 8,
            Self::Suave => 16,
            Self::Relampago => 4,
        }
    }

    /// Energía objetivo en la posición relativa `x` de 0 a 1.
    ///
    /// Fiesta no es una rampa: sube, descansa, vuelve a subir más alto, se
    /// toma un respiro antes del clímax y resuelve. Una curva monótona se
    /// estabilizaba arriba a mitad del mashup y perdía la sensación de avance.
    fn curva(self, x: f64) -> f64 {
        match self {
            Self::Fiesta => {
                // Puntos (posición, energía) unidos por tramos suaves.
                const P: [(f64, f64); 8] = [
                    (0.0, 0.45),
                    (0.28, 0.78),
                    (0.38, 0.55),
                    (0.6, 0.86),
                    (0.68, 0.62),
                    (0.86, 0.97),
                    (0.95, 0.85),
                    (1.0, 0.6),
                ];
                let x = x.clamp(0.0, 1.0);
                let k = P
                    .windows(2)
                    .position(|w| x <= w[1].0)
                    .unwrap_or(P.len() - 2);
                let ((x0, y0), (x1, y1)) = (P[k], P[k + 1]);
                let t = ((x - x0) / (x1 - x0).max(1e-9)).clamp(0.0, 1.0);
                let s = t * t * (3.0 - 2.0 * t);
                y0 + (y1 - y0) * s
            }
            Self::Suave => 0.45 + 0.15 * (std::f64::consts::PI * x).sin(),
            Self::Equilibrado | Self::Relampago => {
                0.4 + 0.5 * (std::f64::consts::PI * x).sin().powf(0.7)
            }
        }
    }

    fn ritmo_ajeno(self) -> bool {
        matches!(self, Self::Fiesta)
    }

    /// Fracción de frases que el estilo admite con la mezcla original sin
    /// nada encima antes de empezar a cobrarlo: por debajo es un respiro, por
    /// encima el mashup se convierte en una sesión de canciones enteras.
    fn mezclas_objetivo(self) -> f64 {
        match self {
            Self::Equilibrado => 0.3,
            Self::Fiesta => 0.2,
            Self::Suave => 0.45,
            Self::Relampago => 0.35,
        }
    }

    /// Compases de anticipo de una entrada con intercambio de graves.
    fn anticipo(self) -> usize {
        match self {
            Self::Equilibrado | Self::Fiesta => 2,
            Self::Suave => 4,
            Self::Relampago => 1,
        }
    }

    /// Fundido de un cruce, en compases.
    fn cruce(self) -> f32 {
        if matches!(self, Self::Suave) {
            1.0
        } else {
            0.125
        }
    }
}

/// Una pista disponible para componer.
#[derive(Debug, Clone)]
pub struct Fuente {
    pub pista_id: i64,
    pub artista: String,
    /// Tonalidad de los rasgos (`C`, `F#`…) y si es menor.
    pub tonica: Option<usize>,
    pub menor: bool,
    pub estructura: Estructura,
    /// Si sus cuatro stems están en caché.
    pub stems: bool,
    /// Los compases `[a, b)` con stems, si solo hay un tramo separado. Con
    /// stems, solo esos compases se ofrecen como fragmento con stems.
    pub compases_stems: Option<(usize, usize)>,
}

/// De dónde sale el audio de una capa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub enum Material {
    /// La pista tal cual.
    Mezcla,
    /// Batería, bajo y otros.
    Instrumental,
    /// Bajo y otros: la base cuando la batería la pone otra pista.
    SinBateria,
    Voz,
    Bateria,
}

impl Material {
    /// Los stems que suma; vacío para la mezcla.
    #[must_use]
    pub fn stems(self) -> &'static [Stem] {
        match self {
            Self::Mezcla => &[],
            Self::Instrumental => &[Stem::Bateria, Stem::Bajo, Stem::Otros],
            Self::SinBateria => &[Stem::Bajo, Stem::Otros],
            Self::Voz => &[Stem::Voz],
            Self::Bateria => &[Stem::Bateria],
        }
    }

    #[cfg(test)]
    fn es_base(self) -> bool {
        matches!(self, Self::Mezcla | Self::Instrumental | Self::SinBateria)
    }
}

/// Cómo entra una capa.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub enum Transicion {
    /// Fundido de igual potencia de `entrada` compases.
    #[default]
    Cruce,
    /// Durante `entrada` compases suena sin graves y entrando; los graves
    /// llegan de golpe al final. Con anticipo es el intercambio de graves de
    /// un DJ: la saliente suelta los suyos en ese mismo primer tiempo.
    Filtrada,
    /// Corte limpio en el primer tiempo: para pasar entre canciones que no se
    /// superponen bien, donde una sección termina.
    Corte,
}

/// Una capa del plan: un fragmento de una pista colocado en el destino.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Capa {
    #[ts(type = "number")]
    pub pista_id: i64,
    pub material: Material,
    /// Compás de inicio en la pista de origen.
    pub origen: u32,
    pub compases: u32,
    /// Compás de inicio en el mashup.
    pub destino: u32,
    /// Razón de velocidad nominal (destino / origen).
    pub tempo: f64,
    pub semitonos: i32,
    pub ganancia_db: f32,
    /// Fundidos en compases del destino. La salida se extiende como cola más
    /// allá del final nominal.
    pub entrada: f32,
    pub salida: f32,
    #[serde(default)]
    pub transicion: Transicion,
    /// La cola de salida suena sin graves: la capa que entra ya los pone.
    #[serde(default)]
    pub graves_fuera: bool,
}

/// Lo que el compositor tuvo que ceder, para que la interfaz lo diga.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    tag = "tipo"
)]
#[ts(export, export_to = "../../src/lib/bindings/")]
pub enum Degradacion {
    /// Menos canciones de las pedidas.
    MenosCanciones { pedidas: u32, usadas: u32 },
    /// Sin stems: sin capas de voz ni batería ajena.
    SinStems,
    /// Frases en las que se prolongó la base anterior por falta de candidatos.
    FrasesProlongadas { frases: u32 },
    /// Más corto de lo pedido.
    MasCorto {
        pedidos_seg: u32,
        obtenidos_seg: u32,
    },
    /// Con esa duración no caben tantas canciones de forma reconocible (unos
    /// 8 s cada una como mínimo): se dice cuántas caben y cuánto haría falta.
    NoCaben {
        pedidas: u32,
        caben: u32,
        minimo_seg: u32,
    },
}

/// El plan completo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub version: u32,
    pub bpm: f64,
    /// Tónica de referencia (0 = do) y modo.
    pub tonica: u32,
    pub menor: bool,
    pub compases: u32,
    pub capas: Vec<Capa>,
    pub degradaciones: Vec<Degradacion>,
    /// Puntuación media por frase, de 0 a 1.
    pub calidad: f64,
}

impl Plan {
    /// Segundos de un compás.
    #[must_use]
    pub fn compas_seg(&self) -> f64 {
        4.0 * 60.0 / self.bpm
    }

    #[must_use]
    pub fn duracion_seg(&self) -> f64 {
        f64::from(self.compases) * self.compas_seg()
    }

    /// Las pistas distintas que aparecen, en orden de primera aparición.
    #[must_use]
    pub fn pistas(&self) -> Vec<i64> {
        let mut v = Vec::new();
        for c in &self.capas {
            if !v.contains(&c.pista_id) {
                v.push(c.pista_id);
            }
        }
        v
    }

    /// Las pistas que suenan de forma reconocible: como base o como voz
    /// durante al menos [`COMPASES_MINIMOS`] compases en total. Una batería
    /// ajena o un fundido no hacen reconocible una canción.
    #[must_use]
    pub fn pistas_reconocibles(&self) -> Vec<i64> {
        let mut cuenta: Vec<(i64, u32)> = Vec::new();
        for c in self
            .capas
            .iter()
            .filter(|c| c.material != Material::Bateria)
        {
            match cuenta.iter_mut().find(|(p, _)| *p == c.pista_id) {
                Some((_, n)) => *n += c.compases,
                None => cuenta.push((c.pista_id, c.compases)),
            }
        }
        cuenta
            .into_iter()
            .filter(|(_, n)| *n as usize >= COMPASES_MINIMOS)
            .map(|(p, _)| p)
            .collect()
    }
}

/// La petición.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Peticion {
    pub duracion_seg: u32,
    pub canciones: u32,
    pub estilo: Estilo,
    #[ts(type = "number")]
    pub semilla: i64,
}

/// El patrón rítmico de dos compases de un fragmento: compases pares e
/// impares desde su inicio, centrados y normalizados, y cuánto difieren.
#[derive(Debug, Clone, Copy)]
struct Par {
    par: [f32; PASOS],
    impar: [f32; PASOS],
    /// De 0 a 1: cuánto se organiza el ritmo en dos compases distintos (una
    /// clave marcada); 0 en un ritmo que repite cada compás.
    dos_compases: f64,
    /// Fiabilidad de la rejilla de la pista (pulso y compás).
    confianza: f64,
}

/// Un fragmento candidato: una frase de una pista.
#[derive(Debug, Clone)]
struct Fragmento {
    fuente: usize,
    desde: usize,
    compases: usize,
    energia: f64,
    voz: f64,
    /// Actividad vocal del primer y del último cuarto de compás.
    voz_ini: f64,
    voz_fin: f64,
    /// Si una frase cantada cruza la frontera de inicio o de final: cortar ahí
    /// corta una palabra. 0 sin canto en la frontera.
    corte_ini: f64,
    corte_fin: f64,
    /// Cuartos de compás que la voz sigue cantando tras el final (hasta 8):
    /// la cola que la deja terminar la frase.
    cola_fin: usize,
    croma: [f32; 12],
    /// Croma de los dos primeros y los dos últimos compases: la armonía en la
    /// frontera, que es la que se oye en una transición.
    croma_ini: [f32; 12],
    croma_fin: [f32; 12],
    tipo: TipoSeccion,
    /// El grupo de su sección: dos fragmentos del mismo grupo son el mismo
    /// material (un coro y su repetición) aunque estén en otro sitio.
    grupo: u32,
    /// Empieza donde empieza una sección y termina donde termina.
    ini_seccion: bool,
    fin_seccion: bool,
    /// Empieza en el arranque de la canción o acaba en su final.
    ini_cancion: bool,
    fin_cancion: bool,
    /// Si puede sonar como stems (está dentro del tramo separado).
    con_stems: bool,
    ritmo: Option<Par>,
}

/// Lo que cuesta llevar una fuente al tempo de destino y a la tonalidad de
/// referencia.
#[derive(Debug, Clone, Copy)]
struct Ajuste {
    tempo: f64,
    semitonos: i32,
    coste: f64,
}

fn camelot_de(pc: usize, menor: bool) -> Option<&'static str> {
    camelot(
        Some(NOTAS[pc % 12]),
        Some(if menor { "minor" } else { "major" }),
    )
}

/// Distancia de Camelot entre dos tonalidades, de una tabla de 24 × 24
/// calculada una vez: el compositor la consulta millones de veces.
fn distancia_tonos(pa: usize, ma: bool, pb: usize, mb: bool) -> i32 {
    static TABLA: std::sync::OnceLock<Vec<i32>> = std::sync::OnceLock::new();
    let t = TABLA.get_or_init(|| {
        let mut v = vec![99; 24 * 24];
        for a in 0..24 {
            for b in 0..24 {
                if let (Some(x), Some(y)) =
                    (camelot_de(a % 12, a >= 12), camelot_de(b % 12, b >= 12))
                {
                    v[a * 24 + b] = distancia_camelot(x, y);
                }
            }
        }
        v
    });
    t[(pa % 12 + 12 * usize::from(ma)) * 24 + pb % 12 + 12 * usize::from(mb)]
}

/// Solo el tempo excluye: lo que no cabe en ±8 % se estira con artefactos.
/// La tonalidad decide cuánto se transpone y cuánto cuesta, no si se usa.
fn ajuste(f: &Fuente, bpm: f64, tonica: usize, menor: bool) -> Option<Ajuste> {
    let src = f.estructura.bpm?;
    let tempo = bpm / src;
    if (tempo - 1.0).abs() > TEMPO_MAX {
        return None;
    }
    let destino = camelot_de(tonica, menor)?;
    let (semitonos, distancia) = match f.tonica {
        None => (0, 2),
        Some(t) => (-SEMITONOS_MAX..=SEMITONOS_MAX)
            .filter_map(|s| {
                let c = camelot_de((t as i32 + s).rem_euclid(12) as usize, f.menor)?;
                Some((s, distancia_camelot(c, destino)))
            })
            .min_by_key(|(s, d)| (*d, s.abs()))?,
    };
    let coste = (tempo - 1.0).abs() / TEMPO_MAX * 0.5
        + f64::from(semitonos.abs()) / f64::from(SEMITONOS_MAX) * 0.2
        + f64::from(distancia.min(3)) * 0.1;
    Some(Ajuste {
        tempo,
        semitonos,
        coste,
    })
}

fn rotar(c: &[f32; 12], s: i32) -> [f32; 12] {
    let mut r = [0.0f32; 12];
    for (i, v) in c.iter().enumerate() {
        r[(i as i32 + s).rem_euclid(12) as usize] = *v;
    }
    r
}

fn coseno(a: &[f32; 12], b: &[f32; 12]) -> f64 {
    a.iter().zip(b).map(|(x, y)| f64::from(x * y)).sum()
}

fn croma_medio(e: &Estructura, r: std::ops::Range<usize>) -> [f32; 12] {
    let mut c = [0.0f32; 12];
    for k in r {
        if let Some(fila) = e.croma.get(k) {
            for (a, b) in c.iter_mut().zip(fila) {
                *a += b;
            }
        }
    }
    let norma = c.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
    c.iter_mut().for_each(|x| *x /= norma);
    c
}

/// Actividad vocal de un cuarto de compás; sin cuartos medidos, la del compás.
fn cuarto(e: &Estructura, q: usize) -> Option<f64> {
    if let Some(c) = &e.voz_cuartos {
        return Some(c.get(q).copied().map_or(0.0, f64::from));
    }
    e.voz
        .as_ref()
        .map(|v| v.get(q / 4).copied().map_or(0.0, f64::from))
}

/// Si una frase cantada cruza la frontera al inicio del compás `b`.
fn corte(e: &Estructura, b: usize) -> f64 {
    if b == 0 {
        return 0.0;
    }
    match (cuarto(e, 4 * b - 1), cuarto(e, 4 * b)) {
        (Some(a), Some(c)) => a.min(c),
        // Sin stems no se sabe: un valor neutro que no prohíbe ni premia.
        _ => 0.3,
    }
}

fn centrar(p: &mut [f32; PASOS]) -> f64 {
    let m = p.iter().sum::<f32>() / PASOS as f32;
    p.iter_mut().for_each(|x| *x -= m);
    let n = p.iter().map(|x| x * x).sum::<f32>().sqrt();
    if n > 1e-6 {
        p.iter_mut().for_each(|x| *x /= n);
    }
    f64::from(n)
}

fn corr(a: &[f32; PASOS], b: &[f32; PASOS]) -> f64 {
    a.iter().zip(b).map(|(x, y)| f64::from(x * y)).sum()
}

/// El patrón de dos compases de `[desde, desde + n)`.
fn par_ritmico(e: &Estructura, desde: usize, n: usize) -> Option<Par> {
    if e.patron.len() < desde + n || n < 2 {
        return None;
    }
    let (mut par, mut impar) = ([0.0f32; PASOS], [0.0f32; PASOS]);
    for k in 0..n {
        let destino = if k % 2 == 0 { &mut par } else { &mut impar };
        for (a, b) in destino.iter_mut().zip(&e.patron[desde + k]) {
            *a += f32::from(*b) / 255.0;
        }
    }
    let (np, ni) = (centrar(&mut par), centrar(&mut impar));
    if np < 1e-3 || ni < 1e-3 {
        return None;
    }
    // Correlación entre los dos compases: 1 si son iguales. Por debajo de 0,85
    // empieza a haber una organización de dos compases; por debajo de 0,5 es
    // marcada (en la clave de son sintética, 0,2).
    let r = corr(&par, &impar);
    Some(Par {
        par,
        impar,
        dos_compases: ((0.85 - r) / 0.35).clamp(0.0, 1.0),
        confianza: (e.confianza_pulso * (0.5 + e.confianza_compas)).clamp(0.0, 1.0),
    })
}

/// La relación rítmica entre dos fragmentos alineados compás a compás.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Relacion {
    /// Parecido del groove en la mejor alineación (−1 a 1).
    pub groove: f64,
    /// De 0 a 1: certeza de que sus patrones de dos compases están en fases
    /// opuestas (una clave 3-2 contra una 2-3).
    pub conflicto: f64,
    /// De 0 a 1: los dos tienen organización de dos compases pero sus patrones
    /// no se parecen en ninguna fase: no se sabe si cruzan.
    pub incierto: f64,
}

fn relacion(a: &Par, b: &Par) -> Relacion {
    let misma = 0.5 * (corr(&a.par, &b.par) + corr(&a.impar, &b.impar));
    let cruzada = 0.5 * (corr(&a.par, &b.impar) + corr(&a.impar, &b.par));
    let dos = (a.dos_compases * b.dos_compases).sqrt();
    let conf = a.confianza.min(b.confianza);
    let conflicto = ((cruzada - misma) / 0.3).clamp(0.0, 1.0) * dos * conf;
    let incierto = dos * (1.0 - (misma.max(cruzada) / 0.5).clamp(0.0, 1.0)).max(1.0 - conf);
    Relacion {
        groove: misma.max(cruzada),
        conflicto,
        incierto: incierto.clamp(0.0, 1.0),
    }
}

fn fragmentos(fuentes: &[Fuente], frase: usize) -> Vec<Fragmento> {
    let mut v = Vec::new();
    for (i, f) in fuentes.iter().enumerate() {
        let e = &f.estructura;
        if !e.rejilla_fiable() {
            continue;
        }
        let nb = e.n_compases();
        for s in &e.secciones {
            let mut d = s.desde;
            while d + frase <= s.hasta.min(nb) {
                let con_stems = f.stems
                    && f.compases_stems
                        .is_none_or(|(a, b)| d >= a && d + frase <= b);
                let r = d..d + frase;
                let n = frase as f64;
                let energia = e.energia[r.clone()]
                    .iter()
                    .map(|x| f64::from(*x))
                    .sum::<f64>()
                    / n;
                let voz = e.voz.as_ref().map_or(0.5, |vz| {
                    vz[r.clone()].iter().map(|x| f64::from(*x)).sum::<f64>() / n
                });
                let (voz_ini, voz_fin) = (
                    cuarto(e, 4 * d).unwrap_or(0.5),
                    cuarto(e, 4 * (d + frase) - 1).unwrap_or(0.5),
                );
                let fin = d + frase;
                let cola_fin = if e.voz.is_some() {
                    (0..8)
                        .take_while(|j| cuarto(e, 4 * fin + j).unwrap_or(0.0) > 0.5)
                        .count()
                } else {
                    0
                };
                let dos = 2.min(frase);
                v.push(Fragmento {
                    fuente: i,
                    desde: d,
                    compases: frase,
                    energia,
                    voz,
                    voz_ini,
                    voz_fin,
                    corte_ini: corte(e, d),
                    corte_fin: if fin >= nb { 0.0 } else { corte(e, fin) },
                    cola_fin,
                    croma: croma_medio(e, r),
                    croma_ini: croma_medio(e, d..d + dos),
                    croma_fin: croma_medio(e, fin - dos..fin),
                    tipo: s.tipo,
                    grupo: s.grupo,
                    ini_seccion: d == s.desde,
                    fin_seccion: fin == s.hasta,
                    ini_cancion: d <= 1,
                    fin_cancion: fin + 1 >= nb,
                    con_stems,
                    ritmo: par_ritmico(e, d, frase),
                });
                d += frase;
            }
        }
    }
    v
}

/// Lo elegido para una frase.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Eleccion {
    base: usize,
    /// La base suena con su mezcla original (con su voz).
    mezcla: bool,
    voz: Option<usize>,
    ritmo: Option<usize>,
    prolongada: bool,
    /// Transposición de la base y de la voz.
    sb: i32,
    sv: i32,
    /// Cómo entra la base si cambia, y con cuántos compases de anticipo.
    trans: Transicion,
    anticipo: usize,
}

#[derive(Debug, Clone)]
struct Estado {
    elecciones: Vec<Eleccion>,
    puntos: f64,
    /// Usos de cada pista como base o voz: lo que la hace reconocible.
    usos: HashMap<usize, u32>,
    /// Cuántas veces ha sonado cada fragmento y cada (pista, grupo), como base
    /// o como voz: el material ya oído no vuelve salvo que no quede otro.
    fragmentos: HashMap<(usize, bool), u32>,
    grupos: HashMap<(usize, u32, bool), u32>,
    mezclas: u32,
}

struct Contexto<'a> {
    fuentes: &'a [Fuente],
    frag: &'a [Fragmento],
    ajustes: &'a [Option<Ajuste>],
    estilo: Estilo,
    frases: usize,
    objetivo_canciones: usize,
}

impl Contexto<'_> {
    fn x(&self, i: usize) -> f64 {
        if self.frases > 1 {
            i as f64 / (self.frases - 1) as f64
        } else {
            0.5
        }
    }

    /// Lo que cuesta volver a oír material ya usado.
    fn repetido(&self, estado: &Estado, f: usize, voz: bool, continua: bool) -> f64 {
        let fr = &self.frag[f];
        let peso = if voz { 1.2 } else { 1.0 };
        let mismo = f64::from(estado.fragmentos.get(&(f, voz)).copied().unwrap_or(0));
        let grupo = if continua {
            0.0
        } else {
            f64::from(
                estado
                    .grupos
                    .get(&(fr.fuente, fr.grupo, voz))
                    .copied()
                    .unwrap_or(0),
            )
        };
        peso * (0.6 * mismo + 0.25 * grupo)
    }

    fn sigue(&self, a: usize, b: usize) -> bool {
        let (x, y) = (&self.frag[a], &self.frag[b]);
        x.fuente == y.fuente && x.desde + x.compases == y.desde
    }

    fn tiene_tonica(&self, f: usize) -> bool {
        self.fuentes[self.frag[f].fuente].tonica.is_some()
    }

    /// Afinidad compás a compás entre la voz y la base: la media de cosenos y
    /// la fracción de compases que chocan. Una media favorable puede esconder
    /// medio fragmento fuera de sitio.
    fn croma_por_compas(&self, v: usize, b: usize, sv: i32, sb: i32) -> (f64, f64) {
        let (fv, fb) = (&self.frag[v], &self.frag[b]);
        let (ev, eb) = (
            &self.fuentes[fv.fuente].estructura,
            &self.fuentes[fb.fuente].estructura,
        );
        let n = fv.compases.min(fb.compases);
        let (mut suma, mut choques) = (0.0, 0usize);
        for k in 0..n {
            let (Some(a), Some(c)) = (ev.croma.get(fv.desde + k), eb.croma.get(fb.desde + k))
            else {
                continue;
            };
            let x = coseno(&rotar(a, sv), &rotar(c, sb));
            suma += x;
            if x < CHOQUE_COMPAS {
                choques += 1;
            }
        }
        let n = n.max(1) as f64;
        (suma / n, choques as f64 / n)
    }

    /// Transposiciones de base y voz para superponerlas. La voz se transpone
    /// lo menos posible: mover el acompañamiento degrada menos que mover una
    /// voz (formantes, vibrato, consonantes).
    fn tonos(
        &self,
        v: usize,
        b: usize,
        fija_b: Option<i32>,
        fija_v: Option<i32>,
    ) -> Option<(i32, i32)> {
        let (fv, fb) = (
            &self.fuentes[self.frag[v].fuente],
            &self.fuentes[self.frag[b].fuente],
        );
        let (tv, tb) = (fv.tonica?, fb.tonica?);
        let aj_b = self.ajustes[self.frag[b].fuente]?;
        let rango = |fija: Option<i32>| match fija {
            Some(s) => s..=s,
            None => -SEMITONOS_MAX..=SEMITONOS_MAX,
        };
        let mut mejor: Option<(f64, i32, i32)> = None;
        for sv in rango(fija_v) {
            let pv = (tv as i32 + sv).rem_euclid(12) as usize;
            for sb in rango(fija_b) {
                let pb = (tb as i32 + sb).rem_euclid(12) as usize;
                let d = distancia_tonos(pv, fv.menor, pb, fb.menor);
                if d > 1 {
                    continue;
                }
                // La tonalidad estimada puede fallar: la armonía real del
                // fragmento decide entre las transposiciones admisibles.
                let c = coseno(
                    &rotar(&self.frag[v].croma, sv),
                    &rotar(&self.frag[b].croma, sb),
                );
                let coste = 0.3 * f64::from(sv.abs()) + 0.08 * f64::from(sb.abs()) - 0.6 * c
                    + 0.04 * f64::from((sb - aj_b.semitonos).abs())
                    + 0.12 * f64::from(d);
                if mejor.is_none_or(|m| coste < m.0) {
                    mejor = Some((coste, sb, sv));
                }
            }
        }
        mejor.map(|m| (m.1, m.2))
    }

    /// Decide transposiciones y la transición de entrada de la base.
    /// Devuelve `None` si la combinación no es posible.
    fn preparar(&self, estado: &Estado, mut e: Eleccion) -> Option<Eleccion> {
        let prev = estado.elecciones.last();
        let continua_b = prev.is_some_and(|p| self.sigue(p.base, e.base) && p.mezcla == e.mezcla);
        let aj_b = self.ajustes[self.frag[e.base].fuente]?;
        let fija_b = continua_b.then(|| prev.map_or(aj_b.semitonos, |p| p.sb));
        match e.voz {
            Some(v) => {
                let (fv, b) = (&self.frag[v], &self.frag[e.base]);
                if fv.fuente == b.fuente || !fv.con_stems || !b.con_stems || e.mezcla {
                    return None;
                }
                let fija_v = prev
                    .and_then(|p| p.voz.map(|pv| (pv, p.sv)))
                    .filter(|(pv, _)| self.sigue(*pv, v))
                    .map(|(_, s)| s);
                let (sb, sv) = self.tonos(v, e.base, fija_b, fija_v)?;
                e.sb = sb;
                e.sv = sv;
            }
            None => e.sb = fija_b.unwrap_or(aj_b.semitonos),
        }
        let (trans, anticipo) = match prev {
            None => {
                let b = &self.frag[e.base];
                if b.ini_cancion {
                    (Transicion::Cruce, 0)
                } else {
                    (Transicion::Filtrada, 0)
                }
            }
            Some(_) if continua_b => (Transicion::Cruce, 0),
            Some(p) => {
                let (t, a) = self.transicion(p, &e).0;
                // Anticipar no puede volver a tocar compases ya oídos.
                let b = &self.frag[e.base];
                let oido = a > 0
                    && estado.elecciones.iter().any(|q| {
                        let f = &self.frag[q.base];
                        f.fuente == b.fuente
                            && f.desde < b.desde
                            && f.desde + f.compases > b.desde - a
                    });
                if oido {
                    (Transicion::Cruce, 0)
                } else {
                    (t, a)
                }
            }
        };
        e.trans = trans;
        e.anticipo = anticipo;
        Some(e)
    }

    /// La transición entre la base anterior y la nueva y lo que vale.
    ///
    /// Con armonía y ritmo compatibles en la frontera se cruza con intercambio
    /// de graves y anticipo (la técnica habitual de un DJ en música con pulso).
    /// Si no, se corta en el primer tiempo; y un corte suena intencionado
    /// cuando la saliente termina una sección o la entrante empieza una. Así
    /// una pareja que no se superpone se integra igual, en secuencia.
    fn transicion(&self, p: &Eleccion, e: &Eleccion) -> ((Transicion, usize), f64) {
        let (a, b) = (&self.frag[p.base], &self.frag[e.base]);
        let h = coseno(&rotar(&a.croma_fin, p.sb), &rotar(&b.croma_ini, e.sb));
        let rel = match (&a.ritmo, &b.ritmo) {
            (Some(x), Some(y)) => Some(relacion(x, y)),
            _ => None,
        };
        let conflicto = rel.map_or(0.0, |r| r.conflicto);
        let incierto = rel.map_or(0.5, |r| r.incierto);
        let estructural = f64::from(u8::from(a.fin_seccion)) * 0.5
            + f64::from(u8::from(b.ini_seccion)) * 0.3
            + if b.energia > a.energia + 0.15 || a.energia > b.energia + 0.25 {
                0.2
            } else {
                0.0
            };
        let anticipo = self.estilo.anticipo();
        let eb = &self.fuentes[b.fuente].estructura;
        // Anticipar es tocar compases anteriores al fragmento de la entrante:
        // tienen que existir (y estar separados si suena como stems) y no
        // llevar canto si suena su mezcla.
        let previo_ok = b.desde >= anticipo
            && (e.mezcla || !b.con_stems || {
                let f = &self.fuentes[b.fuente];
                f.compases_stems
                    .is_none_or(|(x, _)| b.desde - anticipo >= x)
            })
            && (!e.mezcla
                || (eb.voz.is_some()
                    && (4 * (b.desde - anticipo)..4 * b.desde)
                        .all(|q| cuarto(eb, q).unwrap_or(1.0) < 0.3)));
        if h >= 0.7 && conflicto < 0.2 && incierto < 0.6 && previo_ok {
            return ((Transicion::Filtrada, anticipo), 0.1 + (h - 0.7) * 0.3);
        }
        // Corte: aceptable en una frontera estructural; a mitad de sección y
        // con armonía que choca se cobra.
        let corte = -0.25 + estructural * 0.35 - conflicto * 0.1;
        let cruce = -(0.75 - h).max(0.0) * 0.6 - conflicto * 0.4 - incierto * 0.1;
        if corte >= cruce {
            ((Transicion::Corte, 0), corte)
        } else {
            ((Transicion::Cruce, 0), cruce)
        }
    }

    fn puntuar(&self, estado: &Estado, i: usize, e: &Eleccion) -> Option<f64> {
        let b = &self.frag[e.base];
        let fb = b.fuente;
        let aj_b = self.ajustes[fb]?;
        let x = self.x(i);
        let objetivo = self.estilo.curva(x);
        let ultima = i + 1 == self.frases;
        let mut energia = b.energia;
        let mut p = 1.0
            - (energia - objetivo).abs() * 1.2
            - aj_b.coste * 0.3
            - 0.05 * f64::from((e.sb - aj_b.semitonos).abs());
        // Cobertura: premiar pistas nuevas hasta las pedidas, con más fuerza
        // cuanto menos sitio queda para las que faltan.
        let distintas = estado.usos.len();
        let faltan = self.objetivo_canciones.saturating_sub(distintas);
        // Una frase garantiza una canción nueva (la base); la voz, solo si
        // encaja: la urgencia se mide contra lo seguro.
        let huecos = self.frases - i;
        let urgencia = (faltan as f64 / huecos.max(1) as f64).clamp(0.0, 1.0);
        let premio_nueva = 0.3 + 0.9 * urgencia;
        let nueva = !estado.usos.contains_key(&fb);
        if nueva && faltan > 0 {
            p += premio_nueva;
        }
        let prev = estado.elecciones.last();
        let continua_b = prev.is_some_and(|q| self.sigue(q.base, e.base));
        // Volver a una canción ya oída mientras faltan otras ocupa un hueco
        // que una nueva necesitaba.
        if !nueva && !continua_b && faltan > 0 {
            p -= premio_nueva * 0.5;
        }
        for (k, q) in estado.elecciones.iter().rev().take(3).enumerate() {
            if self.frag[q.base].fuente == fb {
                if k == 0 && continua_b {
                    p += 0.05;
                } else if q.base == e.base {
                    p -= 0.5;
                } else {
                    p -= 0.15;
                }
            }
        }
        p -= self.repetido(estado, e.base, false, continua_b);
        if e.mezcla {
            p -= self.repetido(estado, e.base, true, continua_b) * 0.5;
        }
        let tope = (2 * self.frases).div_ceil(self.objetivo_canciones.max(1)) as u32 + 1;
        if estado.usos.get(&fb).copied().unwrap_or(0) >= tope {
            p -= 0.3;
        }
        if let Some(q) = prev {
            let pa = self.frag[q.base].fuente;
            if pa != fb && self.fuentes[pa].artista == self.fuentes[fb].artista {
                p -= 0.1;
            }
        }
        // La transición de base.
        if let Some(q) = prev {
            if continua_b && q.mezcla == e.mezcla {
                p += 0.05;
            } else {
                p += self.transicion(q, e).1;
            }
        }
        let voz_previa = prev.and_then(|q| q.voz);
        // La voz original que se oiría al entrar en una mezcla a mitad de frase.
        if e.mezcla && !continua_b {
            p -= 0.3 * b.corte_ini;
        }
        // Apertura: mejor el arranque real de una canción o una intro, sin voz
        // encima de entrada.
        if i == 0 {
            if b.ini_cancion || b.tipo == TipoSeccion::Intro {
                p += 0.25;
            }
            if e.voz.is_some() {
                p -= 0.15;
            }
        }
        // Cierre: mejor el final real de una canción o un final de sección,
        // sin cortar una voz.
        if ultima {
            if b.fin_cancion || b.tipo == TipoSeccion::Final {
                p += 0.3;
            } else if b.fin_seccion {
                p += 0.1;
            }
            if e.mezcla {
                p -= 0.25 * b.corte_fin;
            }
        }
        if let Some(v) = e.voz {
            let fv = &self.frag[v];
            if e.mezcla
                || fv.fuente == fb
                || !self.tiene_tonica(v)
                || !self.tiene_tonica(e.base)
                || fv.voz < CANTA
                || !fv.con_stems
                || !b.con_stems
            {
                return None;
            }
            let (ev, eb) = (
                &self.fuentes[fv.fuente].estructura,
                &self.fuentes[fb].estructura,
            );
            if ev.confianza_pulso < PULSO_CAPAS || eb.confianza_pulso < PULSO_CAPAS {
                return None;
            }
            let aj_v = self.ajustes[fv.fuente]?;
            let c = coseno(&rotar(&fv.croma, e.sv), &rotar(&b.croma, e.sb));
            if c < UMBRAL_CROMA_VOZ {
                return None;
            }
            let (cc, choques) = self.croma_por_compas(v, e.base, e.sv, e.sb);
            if cc < UMBRAL_CROMA_COMPAS || choques > CHOQUES_MAX {
                return None;
            }
            // Clave: con certeza de fases opuestas, no; con duda, se cobra.
            if let (Some(rv), Some(rb)) = (&fv.ritmo, &b.ritmo) {
                let r = relacion(rv, rb);
                if r.conflicto > CLAVE_MAX {
                    return None;
                }
                p -= r.conflicto * 0.5 + r.incierto * 0.1;
            }
            p += 0.2 + (c - UMBRAL_CROMA_VOZ) * 0.5 + (cc - 0.6) * 0.4 - choques * 0.5;
            p -= (aj_v.tempo - 1.0).abs() / TEMPO_MAX * 0.15 + 0.12 * f64::from(e.sv.abs());
            let continua_v = voz_previa.is_some_and(|pv| self.sigue(pv, v));
            p -= self.repetido(estado, v, true, continua_v);
            match voz_previa {
                Some(_) if continua_v => p += 0.12,
                Some(pv) => {
                    // Cambiar de voz con la saliente a media palabra se oye
                    // como un salto; se prefiere donde una calla.
                    let pa = &self.frag[pv];
                    p -= 0.9 * pa.corte_fin + 0.4 * fv.corte_ini;
                    if pa.fuente == fv.fuente {
                        p -= 0.2;
                    }
                }
                None => {
                    p -= 0.25 * fv.corte_ini;
                    // Una voz que dejó una cola en esta frase no admite otra.
                    if let Some(q) = prev {
                        if q.voz.is_none() && q.mezcla {
                            p -= 0.2 * self.frag[q.base].corte_fin;
                        }
                    }
                }
            }
            if matches!(fv.tipo, TipoSeccion::Coro) {
                p += 0.1;
                if self.estilo == Estilo::Fiesta && objetivo > 0.85 {
                    p += 0.15;
                }
            }
            if !estado.usos.contains_key(&fv.fuente) && faltan > usize::from(nueva) {
                p += premio_nueva * 0.8;
            } else if estado.usos.contains_key(&fv.fuente)
                && !continua_v
                && faltan > usize::from(nueva)
            {
                p -= premio_nueva * 0.4;
            }
            if ultima {
                p -= 0.3 * fv.corte_fin * f64::from(u8::from(fv.cola_fin > 4));
            }
            energia += 0.1;
        } else if let Some(pv) = voz_previa {
            // Soltar una voz a media frase: si su cola la deja terminar (y la
            // base nueva no trae voz propia) apenas cuesta.
            let pa = &self.frag[pv];
            let cola_posible = pa.cola_fin <= 8 && !e.mezcla;
            p -= if cola_posible { 0.05 } else { 0.35 } * pa.corte_fin;
        }
        if e.voz.is_none() && !e.mezcla && b.con_stems {
            // Un instrumental solo respira; muchas seguidas, se pierde la voz.
            let sin_voz = estado
                .elecciones
                .iter()
                .rev()
                .take_while(|q| q.voz.is_none() && !q.mezcla)
                .count();
            p -= 0.08 * sin_voz as f64;
        }
        if e.mezcla {
            // El original es natural (nada separado ni transpuesto encima),
            // pero un mashup no es una sesión de canciones enteras.
            p += 0.08;
            let racha = estado
                .elecciones
                .iter()
                .rev()
                .take_while(|q| q.mezcla)
                .count();
            let forzada = !b.con_stems;
            p -= if forzada { 0.04 } else { 0.12 } * racha as f64;
            let proporcion = f64::from(estado.mezclas + 1) / self.frases as f64;
            let exceso = (proporcion - self.estilo.mezclas_objetivo()).max(0.0);
            if !forzada {
                p -= exceso * 0.8;
            }
        }
        if let Some(r) = e.ritmo {
            let fr = &self.frag[r];
            if e.mezcla
                || fr.fuente == fb
                || Some(fr.fuente) == e.voz.map(|v| self.frag[v].fuente)
                || !fr.con_stems
            {
                return None;
            }
            self.ajustes[fr.fuente]?;
            // Dos baterías juntas solo si su groove se parece y no cruzan la
            // clave; si no, suena una sola.
            match (&fr.ritmo, &b.ritmo) {
                (Some(x), Some(y)) => {
                    let rel = relacion(x, y);
                    if rel.groove < 0.5 || rel.conflicto > 0.2 {
                        return None;
                    }
                }
                _ => return None,
            }
            p += 0.05;
            energia += 0.1;
        }
        // Densidad: más capas solo cuando la curva pide energía; en los
        // descansos de Fiesta, menos.
        if energia > objetivo + 0.35 {
            p -= 0.2;
        }
        if self.estilo == Estilo::Fiesta {
            let capas = 1 + usize::from(e.voz.is_some()) + usize::from(e.ritmo.is_some());
            let quiere = if objetivo > 0.85 {
                3
            } else if objetivo > 0.65 {
                2
            } else {
                1
            };
            p -= 0.08 * (capas as f64 - quiere as f64).abs();
        }
        if e.prolongada {
            p -= 0.3;
        }
        Some(p.clamp(-1.0, 1.5))
    }
}

/// Cuántas canciones caben de forma reconocible en `compases`.
fn caben(compases: usize, compas_seg: f64, capas_por_frase: usize) -> usize {
    let minimo = COMPASES_MINIMOS.max((SEGUNDOS_MINIMOS / compas_seg).ceil() as usize);
    (compases / minimo) * capas_por_frase
}

/// Compone un plan.
///
/// # Errors
///
/// [`NsError::InvalidInput`] si la petición es imposible (sin fuentes con
/// rejilla fiable o sin ninguna pista compatible con la elegida como ancla).
pub fn componer(fuentes: &[Fuente], p: &Peticion) -> NsResult<Plan> {
    if p.duracion_seg < 30 || p.canciones < 2 {
        return Err(NsError::InvalidInput);
    }
    let fiables: Vec<usize> = (0..fuentes.len())
        .filter(|i| fuentes[*i].estructura.rejilla_fiable())
        .collect();
    if fiables.is_empty() {
        return Err(NsError::InvalidInput);
    }
    let mut azar = Azar::con_semilla(p.semilla);
    // El ancla fija el tempo y la tonalidad de referencia: la que más pistas
    // deja a tempo, con las de tonalidad cercana como desempate; la semilla
    // desempata entre las mejores.
    let mut anclas: Vec<(usize, f64)> = fiables
        .iter()
        .map(|&a| {
            let f = &fuentes[a];
            let bpm = f.estructura.bpm.unwrap_or(120.0);
            let (mut n, mut cerca) = (0.0, 0.0);
            for &o in &fiables {
                if let Some(aj) = ajuste(&fuentes[o], bpm, f.tonica.unwrap_or(0), f.menor) {
                    n += 1.0;
                    cerca += 1.0 - aj.coste.min(1.0);
                }
            }
            (a, n + 0.3 * cerca + azar.uniforme(0.0, 0.9))
        })
        .collect();
    anclas.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    let ancla = &fuentes[anclas[0].0];
    let bpm = ancla.estructura.bpm.unwrap_or(120.0);
    let (tonica, menor) = (ancla.tonica.unwrap_or(0), ancla.menor);
    let ajustes: Vec<Option<Ajuste>> = fuentes
        .iter()
        .map(|f| ajuste(f, bpm, tonica, menor))
        .collect();

    let con_stems = fuentes
        .iter()
        .any(|f| f.stems && ajuste(f, bpm, tonica, menor).is_some());
    let compas_seg = 4.0 * 60.0 / bpm;
    let total_compases = (f64::from(p.duracion_seg) / compas_seg).round() as usize;
    let compatibles = ajustes.iter().filter(|a| a.is_some()).count();
    let capas_por_frase = if con_stems { 2 } else { 1 };
    // La frase se acorta si hace falta para que quepan las canciones pedidas.
    let mut frase = p.estilo.frase();
    while frase > 4 && (total_compases / frase) * capas_por_frase < p.canciones as usize {
        frase /= 2;
    }
    let frases = (total_compases / frase).max(1);
    let caben_total = caben(frases * frase, compas_seg, capas_por_frase);
    let objetivo = (p.canciones as usize)
        .min(compatibles)
        .min(frases * capas_por_frase)
        .min(caben_total.max(1));

    let frag: Vec<Fragmento> = fragmentos(fuentes, frase)
        .into_iter()
        .filter(|f| ajustes[f.fuente].is_some())
        .collect();
    if frag.is_empty() {
        return Err(NsError::InvalidInput);
    }
    let ctx = Contexto {
        fuentes,
        frag: &frag,
        ajustes: &ajustes,
        estilo: p.estilo,
        frases,
        objetivo_canciones: objetivo.max(1),
    };
    let bases: Vec<usize> = (0..frag.len()).collect();
    let voces: Vec<usize> = if con_stems {
        (0..frag.len())
            .filter(|&i| frag[i].con_stems && frag[i].voz >= CANTA)
            .collect()
    } else {
        Vec::new()
    };
    let (haz, k) = if frag.len() > 400 {
        (6, 12)
    } else if frag.len() > 100 {
        (10, 16)
    } else {
        (16, 24)
    };

    let mut haz_actual = vec![Estado {
        elecciones: Vec::new(),
        puntos: 0.0,
        usos: HashMap::new(),
        fragmentos: HashMap::new(),
        grupos: HashMap::new(),
        mezclas: 0,
    }];
    for i in 0..frases {
        let objetivo_e = p.estilo.curva(ctx.x(i));
        let mejores = |lista: &[usize], k: usize, azar: &mut Azar| -> Vec<usize> {
            let mut v: Vec<(usize, f64)> = lista
                .iter()
                .map(|&f| {
                    let aj = ajustes[frag[f].fuente].map_or(1.0, |a| a.coste);
                    (
                        f,
                        (frag[f].energia - objetivo_e).abs() + aj * 0.3 + azar.uniforme(0.0, 0.15),
                    )
                })
                .collect();
            v.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
            v.into_iter().take(k).map(|x| x.0).collect()
        };
        // Candidatos ligeros (estado de origen, elección, puntos): solo los
        // que sobreviven se materializan; clonar el estado de cada candidato
        // era la mayor parte del coste.
        let mut siguientes: Vec<(usize, Eleccion, f64)> = Vec::new();
        for (ie, estado) in haz_actual.iter().enumerate() {
            let mut cand_b = mejores(&bases, k, &mut azar);
            if let Some(prev) = estado.elecciones.last() {
                let a = &frag[prev.base];
                if let Some(sig) = bases
                    .iter()
                    .copied()
                    .find(|&f| frag[f].fuente == a.fuente && frag[f].desde == a.desde + a.compases)
                {
                    cand_b.push(sig);
                }
            }
            if i == 0 || i + 1 == frases {
                // Aperturas y cierres naturales siempre se consideran.
                let extremos: Vec<usize> = bases
                    .iter()
                    .copied()
                    .filter(|&f| {
                        if i == 0 {
                            frag[f].ini_cancion
                        } else {
                            frag[f].fin_cancion
                        }
                    })
                    .collect();
                cand_b.extend(mejores(&extremos, k / 2, &mut azar));
            }
            let sin_usar: Vec<usize> = bases
                .iter()
                .copied()
                .filter(|f| !estado.usos.contains_key(&frag[*f].fuente))
                .collect();
            cand_b.extend(mejores(&sin_usar, k / 2, &mut azar));
            cand_b.sort_unstable();
            cand_b.dedup();
            let mut cand_v: Vec<Option<usize>> = vec![None];
            let voces_sin_usar: Vec<usize> = voces
                .iter()
                .copied()
                .filter(|f| !estado.usos.contains_key(&frag[*f].fuente))
                .collect();
            cand_v.extend(mejores(&voces, k / 2, &mut azar).into_iter().map(Some));
            cand_v.extend(
                mejores(&voces_sin_usar, k / 2, &mut azar)
                    .into_iter()
                    .map(Some),
            );
            if let Some(pv) = estado.elecciones.last().and_then(|x| x.voz) {
                let a = &frag[pv];
                if let Some(sig) = voces
                    .iter()
                    .copied()
                    .find(|&f| frag[f].fuente == a.fuente && frag[f].desde == a.desde + a.compases)
                {
                    cand_v.push(Some(sig));
                }
            }
            cand_v.sort_unstable();
            cand_v.dedup();
            for &b in &cand_b {
                let separada = frag[b].con_stems;
                // Con stems la base puede sonar instrumental o con su mezcla
                // original; sin ellos, solo la mezcla.
                let modos: &[bool] = if separada { &[false, true] } else { &[true] };
                for &mezcla in modos {
                    for &v in &cand_v {
                        if v.is_some() && mezcla {
                            continue;
                        }
                        let mut ritmos = vec![None];
                        if p.estilo.ritmo_ajeno() && v.is_some() && !mezcla {
                            ritmos.extend(
                                mejores(&bases, 4, &mut azar)
                                    .into_iter()
                                    .filter(|r| frag[*r].con_stems)
                                    .map(Some),
                            );
                        }
                        for r in ritmos {
                            let e = Eleccion {
                                base: b,
                                mezcla,
                                voz: v,
                                ritmo: r,
                                prolongada: false,
                                sb: 0,
                                sv: 0,
                                trans: Transicion::Cruce,
                                anticipo: 0,
                            };
                            let Some(e) = ctx.preparar(estado, e) else {
                                continue;
                            };
                            if let Some(s) = ctx.puntuar(estado, i, &e) {
                                if s >= UMBRAL_FRASE {
                                    siguientes.push((ie, e, estado.puntos + s));
                                }
                            }
                        }
                    }
                }
            }
        }
        if siguientes.is_empty() {
            // Degradación: prolongar la base anterior (o repetirla) sin capas.
            for (ie, estado) in haz_actual.iter().enumerate() {
                let Some(prev) = estado.elecciones.last() else {
                    continue;
                };
                let a = &frag[prev.base];
                let sig = bases
                    .iter()
                    .copied()
                    .find(|&f| frag[f].fuente == a.fuente && frag[f].desde == a.desde + a.compases)
                    .unwrap_or(prev.base);
                let e = Eleccion {
                    base: sig,
                    mezcla: prev.mezcla || !frag[sig].con_stems,
                    voz: None,
                    ritmo: None,
                    prolongada: true,
                    sb: prev.sb,
                    sv: 0,
                    trans: Transicion::Cruce,
                    anticipo: 0,
                };
                let e = ctx.preparar(estado, e).unwrap_or(e);
                let s = ctx.puntuar(estado, i, &e).unwrap_or(0.0);
                siguientes.push((ie, e, estado.puntos + s));
            }
            if siguientes.is_empty() {
                return Err(NsError::InvalidInput);
            }
        }
        // En la última frase se cobra lo que falta de cobertura: sin esto, el
        // haz preferiría un final cómodo con menos canciones.
        if i + 1 == frases {
            for (ie, e, puntos) in &mut siguientes {
                let usos = &haz_actual[*ie].usos;
                let mut nuevas: Vec<usize> = [Some(e.base), e.voz]
                    .into_iter()
                    .flatten()
                    .map(|f| frag[f].fuente)
                    .filter(|f| !usos.contains_key(f))
                    .collect();
                nuevas.dedup();
                let faltan = ctx
                    .objetivo_canciones
                    .saturating_sub(usos.len() + nuevas.len());
                *puntos -= 0.6 * faltan as f64;
            }
        }
        let firma = |c: &(usize, Eleccion, f64)| {
            (
                c.0,
                frag[c.1.base].fuente,
                frag[c.1.base].desde,
                c.1.voz.unwrap_or(usize::MAX),
                c.1.ritmo.unwrap_or(usize::MAX),
                c.1.mezcla,
            )
        };
        siguientes.sort_by(|a, b| b.2.total_cmp(&a.2).then_with(|| firma(a).cmp(&firma(b))));
        // Diversidad del haz: no más de dos estados con la misma última base.
        let mut vistos: HashMap<(usize, bool), u32> = HashMap::new();
        siguientes.retain(|c| {
            let n = vistos.entry((c.1.base, c.1.mezcla)).or_insert(0);
            *n += 1;
            *n <= 2
        });
        siguientes.truncate(haz);
        haz_actual = siguientes
            .into_iter()
            .map(|(ie, e, puntos)| {
                let base = &haz_actual[ie];
                let mut n = extender(base, e, puntos - base.puntos, &frag);
                n.puntos = puntos;
                n
            })
            .collect();
    }
    let mejor = haz_actual.into_iter().next().ok_or(NsError::Internal)?;
    let mut plan = construir(
        fuentes, &frag, &ajustes, &mejor, p, bpm, tonica, menor, frase, con_stems,
    );
    if (p.canciones as usize) > caben_total {
        // "No caben" ya explica que sean menos: no se dice dos veces.
        plan.degradaciones
            .retain(|d| !matches!(d, Degradacion::MenosCanciones { .. }));
        let minimo = COMPASES_MINIMOS.max((SEGUNDOS_MINIMOS / compas_seg).ceil() as usize);
        let compases_necesarios = (p.canciones as usize).div_ceil(capas_por_frase) * minimo;
        plan.degradaciones.push(Degradacion::NoCaben {
            pedidas: p.canciones,
            caben: caben_total as u32,
            minimo_seg: (compases_necesarios as f64 * compas_seg).ceil() as u32,
        });
    }
    Ok(plan)
}

fn extender(estado: &Estado, e: Eleccion, s: f64, frag: &[Fragmento]) -> Estado {
    let mut n = estado.clone();
    n.puntos += s;
    for f in [Some(e.base), e.voz].into_iter().flatten() {
        *n.usos.entry(frag[f].fuente).or_insert(0) += 1;
    }
    // La mezcla original también hace oír la voz de su fragmento.
    for (f, voz) in [
        (Some(e.base), false),
        (e.voz, true),
        (e.mezcla.then_some(e.base), true),
    ] {
        let Some(f) = f else { continue };
        *n.fragmentos.entry((f, voz)).or_insert(0) += 1;
        *n.grupos
            .entry((frag[f].fuente, frag[f].grupo, voz))
            .or_insert(0) += 1;
    }
    if e.mezcla {
        n.mezclas += 1;
    }
    // Los compases de anticipo también se han oído: el fragmento que acaba
    // donde empieza la base cuenta como usado.
    if e.anticipo > 0 {
        let b = &frag[e.base];
        if let Some(k) = frag
            .iter()
            .position(|f| f.fuente == b.fuente && f.desde + f.compases == b.desde)
        {
            *n.fragmentos.entry((k, false)).or_insert(0) += 1;
        }
    }
    n.elecciones.push(e);
    n
}

fn material_base(fuentes: &[Fuente], frag: &Fragmento, e: &Eleccion, con_stems: bool) -> Material {
    if e.mezcla || !con_stems || !fuentes[frag.fuente].stems || !frag.con_stems {
        Material::Mezcla
    } else if e.ritmo.is_some() {
        Material::SinBateria
    } else {
        Material::Instrumental
    }
}

#[allow(clippy::too_many_arguments)]
fn construir(
    fuentes: &[Fuente],
    frag: &[Fragmento],
    ajustes: &[Option<Ajuste>],
    estado: &Estado,
    p: &Peticion,
    bpm: f64,
    tonica: usize,
    menor: bool,
    frase: usize,
    con_stems: bool,
) -> Plan {
    let mut capas: Vec<Capa> = Vec::new();
    let estilo = p.estilo;
    let cruce = estilo.cruce();
    let n = estado.elecciones.len();
    let tempo_de = |f: usize| ajustes[frag[f].fuente].map_or(1.0, |a| a.tempo);
    // Índice en `capas` de la base y la voz que siguen abiertas.
    let (mut base_abierta, mut voz_abierta): (Option<usize>, Option<usize>) = (None, None);
    for (i, e) in estado.elecciones.iter().enumerate() {
        let destino = (i * frase) as u32;
        let b = &frag[e.base];
        let material = material_base(fuentes, b, e, con_stems);
        let ultima = i + 1 == n;
        // La base: se alarga si sigue la misma grabación con el mismo
        // material y tono; si no, entra con su transición.
        let sigue_base = base_abierta.is_some_and(|k| {
            let c = &capas[k];
            c.pista_id == fuentes[b.fuente].pista_id
                && c.material == material
                && c.semitonos == e.sb
                && c.origen + c.compases == b.desde as u32
                && c.destino + c.compases == destino
        });
        if sigue_base {
            let k = base_abierta.unwrap_or_default();
            capas[k].compases += frase as u32;
        } else {
            let (mut origen, mut dest, mut largo) = (b.desde as u32, destino, frase as u32);
            let (transicion, entrada) = match e.trans {
                Transicion::Filtrada if i == 0 => (Transicion::Filtrada, 1.0),
                Transicion::Filtrada => {
                    let a = e.anticipo as u32;
                    origen -= a;
                    dest -= a;
                    largo += a;
                    (Transicion::Filtrada, a as f32)
                }
                Transicion::Corte => (Transicion::Corte, 1.0 / 32.0),
                Transicion::Cruce if i == 0 => (Transicion::Cruce, 1.0 / 64.0),
                Transicion::Cruce => (Transicion::Cruce, cruce),
            };
            if let Some(k) = base_abierta {
                let c = &mut capas[k];
                match transicion {
                    Transicion::Filtrada => {
                        c.salida = if estilo == Estilo::Relampago {
                            0.5
                        } else {
                            1.0
                        };
                        c.graves_fuera = true;
                    }
                    Transicion::Corte => c.salida = 1.0 / 16.0,
                    Transicion::Cruce => c.salida = cruce,
                }
            }
            capas.push(Capa {
                pista_id: fuentes[b.fuente].pista_id,
                material,
                origen,
                compases: largo,
                destino: dest,
                tempo: tempo_de(e.base),
                semitonos: e.sb,
                ganancia_db: 0.0,
                entrada,
                salida: cruce,
                transicion,
                graves_fuera: false,
            });
            base_abierta = Some(capas.len() - 1);
        }
        if e.voz.is_some() {
            if let Some(k) = base_abierta {
                capas[k].ganancia_db = capas[k].ganancia_db.min(-1.5);
            }
        }
        // La voz.
        match e.voz {
            Some(v) => {
                let fv = &frag[v];
                let sigue_voz = voz_abierta.is_some_and(|k| {
                    let c = &capas[k];
                    c.pista_id == fuentes[fv.fuente].pista_id
                        && c.origen + c.compases == fv.desde as u32
                        && c.destino + c.compases == destino
                        && c.semitonos == e.sv
                        // Tope de 16 compases por capa: la memoria del render
                        // depende del largo de las capas.
                        && c.compases + frase as u32 <= 16
                });
                if sigue_voz {
                    let k = voz_abierta.unwrap_or_default();
                    capas[k].compases += frase as u32;
                } else {
                    let mut entrada: f32 = if estilo == Estilo::Suave { 0.5 } else { 0.0625 };
                    if let Some(k) =
                        voz_abierta.filter(|k| capas[*k].destino + capas[*k].compases == destino)
                    {
                        // Relevo: si la saliente aún canta, se funde sobre el
                        // primer medio compás de la entrante, como un cruce.
                        let anterior = estado.elecciones[i - 1].voz.map(|x| &frag[x]);
                        let canta = anterior.is_some_and(|a| a.corte_fin > 0.5 || a.voz_fin > 0.5);
                        capas[k].salida = capas[k].salida.max(if canta { 0.5 } else { 0.25 });
                        if fv.corte_ini > 0.5 || fv.voz_ini > 0.5 {
                            entrada = entrada.max(0.25);
                        }
                    }
                    capas.push(Capa {
                        pista_id: fuentes[fv.fuente].pista_id,
                        material: Material::Voz,
                        origen: fv.desde as u32,
                        compases: frase as u32,
                        destino,
                        tempo: tempo_de(v),
                        semitonos: e.sv,
                        ganancia_db: 1.0,
                        entrada,
                        salida: if estilo == Estilo::Suave { 0.5 } else { 0.0625 },
                        transicion: Transicion::Cruce,
                        graves_fuera: false,
                    });
                    voz_abierta = Some(capas.len() - 1);
                }
            }
            None => {
                // La voz que se suelta termina su frase con su cola, si nada
                // canta encima en la frase nueva.
                if let Some(k) = voz_abierta.take() {
                    if let Some(pv) = estado.elecciones[i - 1].voz {
                        let a = &frag[pv];
                        if !e.mezcla && a.corte_fin > 0.3 {
                            let cola = (a.cola_fin as f32 / 4.0 + 0.25).min(2.0);
                            capas[k].salida = capas[k].salida.max(cola);
                        }
                    }
                }
            }
        }
        if e.voz.is_none() {
            voz_abierta = None;
        }
        if let Some(r) = e.ritmo {
            let fr = &frag[r];
            capas.push(Capa {
                pista_id: fuentes[fr.fuente].pista_id,
                material: Material::Bateria,
                origen: fr.desde as u32,
                compases: frase as u32,
                destino,
                tempo: tempo_de(r),
                semitonos: 0,
                ganancia_db: -1.0,
                entrada: 0.0625,
                salida: 0.0625,
                transicion: Transicion::Cruce,
                graves_fuera: false,
            });
        }
        if ultima {
            // Cierre: un final real suena entero; si no, una salida progresiva
            // de dos compases tras el último (uno en Relámpago). La voz
            // termina su frase con su cola.
            if let Some(k) = base_abierta {
                capas[k].salida = if b.fin_cancion || estilo == Estilo::Relampago {
                    1.0
                } else {
                    2.0
                };
            }
            if let (Some(k), Some(v)) = (voz_abierta, e.voz) {
                let fv = &frag[v];
                capas[k].salida = (fv.cola_fin as f32 / 4.0 + 0.25).clamp(0.25, 2.0);
            }
        }
    }
    let usadas = {
        let parcial = Plan {
            version: VERSION,
            bpm,
            tonica: 0,
            menor: false,
            compases: 0,
            capas: capas.clone(),
            degradaciones: Vec::new(),
            calidad: 0.0,
        };
        parcial.pistas_reconocibles().len() as u32
    };
    let mut degradaciones = Vec::new();
    if !con_stems {
        degradaciones.push(Degradacion::SinStems);
    }
    if usadas < p.canciones {
        degradaciones.push(Degradacion::MenosCanciones {
            pedidas: p.canciones,
            usadas,
        });
    }
    let prolongadas = estado.elecciones.iter().filter(|e| e.prolongada).count() as u32;
    if prolongadas > 0 {
        degradaciones.push(Degradacion::FrasesProlongadas {
            frases: prolongadas,
        });
    }
    let compases = (n * frase) as u32;
    let obtenidos = (f64::from(compases) * 240.0 / bpm).round() as u32;
    if obtenidos + 30 < p.duracion_seg {
        degradaciones.push(Degradacion::MasCorto {
            pedidos_seg: p.duracion_seg,
            obtenidos_seg: obtenidos,
        });
    }
    capas.sort_by_key(|c| (c.destino, c.material as u8));
    Plan {
        version: VERSION,
        bpm,
        tonica: tonica as u32,
        menor,
        compases,
        capas,
        degradaciones,
        calidad: if n > 0 {
            (estado.puntos / n as f64).clamp(0.0, 1.0)
        } else {
            0.0
        },
    }
}

/// La tónica de una tonalidad de los rasgos.
#[must_use]
pub fn tonica(tonalidad: Option<&str>) -> Option<usize> {
    let t = tonalidad?.trim();
    NOTAS.iter().position(|n| *n == t).or(match t {
        "Db" => Some(1),
        "Eb" => Some(3),
        "Gb" => Some(6),
        "Ab" => Some(8),
        "Bb" => Some(10),
        _ => None,
    })
}

/// Para las pruebas de otros módulos: la relación rítmica entre dos tramos.
#[cfg(test)]
pub(crate) fn relacion_entre(
    a: &Estructura,
    da: usize,
    b: &Estructura,
    db: usize,
    n: usize,
) -> Option<Relacion> {
    Some(relacion(&par_ritmico(a, da, n)?, &par_ritmico(b, db, n)?))
}

#[cfg(test)]
pub(crate) mod pruebas {
    use super::*;
    use crate::mashup::estructura::{Seccion, VERSION as V_EST};

    /// Una fuente sintética con estructura fija: intro, verso, coro, verso,
    /// coro, final, de 8 compases cada uno.
    pub(crate) fn fuente(id: i64, bpm: f64, tonica: usize, menor: bool, stems: bool) -> Fuente {
        let tipos = [
            TipoSeccion::Intro,
            TipoSeccion::Verso,
            TipoSeccion::Coro,
            TipoSeccion::Verso,
            TipoSeccion::Coro,
            TipoSeccion::Final,
        ];
        let nb = tipos.len() * 8 * 2;
        let compas = 240.0 / bpm;
        let mut croma = [0.0f32; 12];
        for g in [0, 4, 7] {
            croma[(tonica + g + usize::from(menor && g == 4) * 11) % 12] = 0.577;
        }
        let energia: Vec<f32> = (0..nb)
            .map(|k| match tipos[k / 16] {
                TipoSeccion::Intro | TipoSeccion::Final => 0.3,
                TipoSeccion::Coro => 0.9,
                _ => 0.6,
            } + (id % 7) as f32 * 0.01)
            .collect();
        let voz: Vec<f32> = (0..nb)
            .map(|k| match tipos[k / 16] {
                TipoSeccion::Intro | TipoSeccion::Final => 0.0,
                _ => 0.9,
            })
            .collect();
        Fuente {
            pista_id: id,
            artista: format!("artista {}", id % 13),
            tonica: Some(tonica),
            menor,
            stems,
            compases_stems: None,
            estructura: Estructura {
                version: V_EST,
                duracion: nb as f64 * compas,
                bpm: Some(bpm),
                confianza_pulso: 0.9,
                pulsos: (0..nb * 4).map(|k| k as f64 * compas / 4.0).collect(),
                compases: (0..=nb).map(|k| k as f64 * compas).collect(),
                confianza_compas: 0.8,
                energia,
                croma: vec![croma; nb],
                voz: stems.then_some(voz),
                voz_cuartos: None,
                patron: vec![[0u8; crate::mashup::estructura::PASOS]; nb],
                secciones: tipos
                    .iter()
                    .enumerate()
                    .map(|(i, t)| Seccion {
                        desde: i * 16,
                        hasta: (i + 1) * 16,
                        tipo: *t,
                        grupo: i as u32,
                        confianza: 0.8,
                        energia: 0.5,
                    })
                    .collect(),
            },
        }
    }

    /// Una biblioteca de `n` pistas casi todas compatibles: tempos 120–126 y
    /// tonalidades vecinas de do mayor.
    pub(crate) fn biblioteca(n: usize, stems: bool) -> Vec<Fuente> {
        (0..n)
            .map(|i| {
                let tonica = [0usize, 7, 5, 9][i % 4];
                let menor = i % 4 == 3;
                fuente(i as i64 + 1, 120.0 + (i % 7) as f64, tonica, menor, stems)
            })
            .collect()
    }

    fn peticion(canciones: u32, duracion: u32, semilla: i64) -> Peticion {
        Peticion {
            duracion_seg: duracion,
            canciones,
            estilo: Estilo::Equilibrado,
            semilla,
        }
    }

    fn comprobar_invariantes(plan: &Plan, fuentes: &[Fuente]) {
        for c in &plan.capas {
            let f = fuentes
                .iter()
                .find(|f| f.pista_id == c.pista_id)
                .expect("pista inexistente");
            assert!(
                (c.origen + c.compases) as usize <= f.estructura.n_compases(),
                "fuera de la pista"
            );
            assert!(c.destino + c.compases <= plan.compases, "fuera del mashup");
            assert!((c.tempo - 1.0).abs() <= TEMPO_MAX + 1e-9);
            assert!(c.semitonos.abs() <= SEMITONOS_MAX);
            if !c.material.stems().is_empty() {
                assert!(f.stems, "stem de una pista sin stems");
            }
        }
        // Nunca dos voces a la vez ni una voz sobre una mezcla con voz.
        for k in 0..plan.compases {
            let activas: Vec<&Capa> = plan
                .capas
                .iter()
                .filter(|c| c.destino <= k && k < c.destino + c.compases)
                .collect();
            let voces = activas
                .iter()
                .filter(|c| c.material == Material::Voz)
                .count();
            assert!(voces <= 1, "compás {k}");
            if voces == 1 {
                // Una mezcla solo puede coincidir con una voz en su anticipo
                // filtrado, que no lleva canto.
                assert!(!activas.iter().any(|c| c.material == Material::Mezcla
                    && !(c.transicion == Transicion::Filtrada
                        && (k as f32) < c.destino as f32 + c.entrada)));
            }
            assert!(!activas.is_empty(), "compás {k} vacío");
        }
    }

    #[test]
    fn la_misma_semilla_da_el_mismo_plan_y_otra_da_una_variante() {
        let f = biblioteca(12, true);
        let a = componer(&f, &peticion(8, 240, 7)).unwrap();
        let b = componer(&f, &peticion(8, 240, 7)).unwrap();
        assert_eq!(a, b);
        let c = componer(&f, &peticion(8, 240, 8)).unwrap();
        assert_ne!(a.capas, c.capas);
    }

    #[test]
    fn respeta_la_duracion_las_canciones_y_los_limites_de_cada_capa() {
        for (n, canciones, dur) in [
            (5usize, 4u32, 120u32),
            (10, 8, 240),
            (20, 16, 360),
            (50, 40, 600),
        ] {
            let f = biblioteca(n, true);
            let plan = componer(&f, &peticion(canciones, dur, 1)).unwrap();
            comprobar_invariantes(&plan, &f);
            assert!((plan.duracion_seg() - f64::from(dur)).abs() <= plan.compas_seg() * 16.0);
            let usadas = plan.pistas_reconocibles().len() as u32;
            assert!(usadas >= canciones, "{n}: {usadas} de {canciones}");
            assert!(plan.calidad > 0.3, "{n}: {}", plan.calidad);
        }
    }

    #[test]
    fn cien_canciones_caben_con_frases_cortas_y_sin_explotar_el_coste() {
        let f = biblioteca(120, true);
        let inicio = std::time::Instant::now();
        let plan = componer(
            &f,
            &Peticion {
                duracion_seg: 900,
                canciones: 100,
                estilo: Estilo::Relampago,
                semilla: 3,
            },
        )
        .unwrap();
        comprobar_invariantes(&plan, &f);
        assert!(plan.pistas().len() >= 80, "{}", plan.pistas().len());
        assert!(
            inicio.elapsed().as_secs_f64() < 20.0,
            "{:?}",
            inicio.elapsed()
        );
    }

    #[test]
    fn las_pistas_incompatibles_quedan_fuera() {
        let mut f = biblioteca(6, true);
        // Una a 90 BPM y otra a 140: fuera de ±8 % incluso plegadas.
        f.push(fuente(100, 90.0, 0, false, true));
        f.push(fuente(101, 140.0, 6, false, true));
        let plan = componer(&f, &peticion(8, 240, 2)).unwrap();
        let usadas = plan.pistas();
        assert!(!usadas.contains(&100));
        assert!(!usadas.contains(&101));
        assert!(plan
            .degradaciones
            .iter()
            .any(|d| matches!(d, Degradacion::MenosCanciones { .. })));
    }

    #[test]
    fn sin_stems_no_hay_capas_de_voz_y_se_dice() {
        let f = biblioteca(8, false);
        let plan = componer(&f, &peticion(6, 180, 4)).unwrap();
        assert!(plan.capas.iter().all(|c| c.material == Material::Mezcla));
        assert!(plan.degradaciones.contains(&Degradacion::SinStems));
        comprobar_invariantes(&plan, &f);
    }

    #[test]
    fn con_stems_hay_voz_de_una_pista_sobre_la_base_de_otra() {
        let f = biblioteca(10, true);
        let plan = componer(&f, &peticion(8, 240, 5)).unwrap();
        let voces = plan
            .capas
            .iter()
            .filter(|c| c.material == Material::Voz)
            .count();
        assert!(voces >= 3, "{voces}");
        for v in plan.capas.iter().filter(|c| c.material == Material::Voz) {
            let base = plan
                .capas
                .iter()
                .find(|c| {
                    c.material != Material::Voz
                        && c.destino <= v.destino
                        && v.destino < c.destino + c.compases
                })
                .unwrap();
            assert_ne!(base.pista_id, v.pista_id);
        }
    }

    #[test]
    fn la_energia_sigue_un_arco() {
        let f = biblioteca(16, true);
        let plan = componer(&f, &peticion(10, 420, 9)).unwrap();
        let energia_en = |k: u32| -> f64 {
            plan.capas
                .iter()
                .filter(|c| {
                    c.material != Material::Voz && c.destino <= k && k < c.destino + c.compases
                })
                .map(|c| {
                    let fu = f.iter().find(|x| x.pista_id == c.pista_id).unwrap();
                    f64::from(fu.estructura.energia[(c.origen + k - c.destino) as usize])
                })
                .fold(0.0, f64::max)
        };
        let n = plan.compases;
        let medio = (n / 3..2 * n / 3).map(energia_en).sum::<f64>() / f64::from(n / 3);
        let inicio = (0..n / 6).map(energia_en).sum::<f64>() / f64::from(n / 6);
        assert!(medio > inicio, "{inicio} → {medio}");
    }

    #[test]
    fn sin_material_suficiente_falla_o_degrada_pero_no_inventa() {
        // Ninguna pista con rejilla fiable.
        let mut f = biblioteca(4, true);
        for x in &mut f {
            x.estructura.confianza_pulso = 0.1;
        }
        assert_eq!(
            componer(&f, &peticion(4, 120, 1)),
            Err(NsError::InvalidInput)
        );
        // Una sola pista compatible: el plan existe pero dice que usó menos.
        let f = vec![
            fuente(1, 120.0, 0, false, true),
            fuente(2, 90.0, 0, false, true),
        ];
        let plan = componer(&f, &peticion(4, 120, 1)).unwrap();
        assert_eq!(plan.pistas().len(), 1);
        assert!(plan
            .degradaciones
            .iter()
            .any(|d| matches!(d, Degradacion::MenosCanciones { .. })));
        // Peticiones absurdas.
        assert_eq!(
            componer(&f, &peticion(1, 120, 1)),
            Err(NsError::InvalidInput)
        );
        assert_eq!(componer(&f, &peticion(4, 5, 1)), Err(NsError::InvalidInput));
    }

    #[test]
    fn datos_de_analisis_incompletos_no_rompen_el_plan() {
        let mut f = biblioteca(8, true);
        f[0].tonica = None;
        f[1].estructura.voz = None;
        f[2].estructura.secciones.clear();
        let plan = componer(&f, &peticion(6, 180, 1)).unwrap();
        comprobar_invariantes(&plan, &f);
        // La pista sin tonalidad no lleva voz encima ni pone voz.
        assert!(!plan
            .capas
            .iter()
            .any(|c| c.pista_id == f[0].pista_id && c.material == Material::Voz));
    }

    #[test]
    fn la_tonica_se_lee_con_sostenidos_y_bemoles() {
        assert_eq!(tonica(Some("C")), Some(0));
        assert_eq!(tonica(Some("Bb")), Some(10));
        assert_eq!(tonica(Some("F#")), Some(6));
        assert_eq!(tonica(Some("H")), None);
        assert_eq!(tonica(None), None);
    }

    #[test]
    fn ningun_fragmento_suena_dos_veces_mientras_quede_material_nuevo() {
        let f = biblioteca(12, true);
        for semilla in 1..6 {
            let plan = componer(&f, &peticion(8, 300, semilla)).unwrap();
            for material in [Material::Voz, Material::Instrumental] {
                let mut vistos = std::collections::HashSet::new();
                for c in plan.capas.iter().filter(|c| c.material == material) {
                    for k in 0..c.compases {
                        assert!(
                            vistos.insert((c.pista_id, c.origen + k)),
                            "semilla {semilla}: {material:?} repite el compás {} de {}",
                            c.origen + k,
                            c.pista_id
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn una_voz_que_sigue_cantando_es_una_sola_capa_y_los_relevos_se_cruzan() {
        let f = biblioteca(12, true);
        let plan = componer(&f, &peticion(6, 300, 3)).unwrap();
        let voces: Vec<&Capa> = plan
            .capas
            .iter()
            .filter(|c| c.material == Material::Voz)
            .collect();
        assert!(!voces.is_empty());
        for par in voces.windows(2) {
            let (a, b) = (par[0], par[1]);
            assert!(
                !(a.pista_id == b.pista_id
                    && a.origen + a.compases == b.origen
                    && a.destino + a.compases == b.destino
                    && a.compases + b.compases.min(8) <= 16),
                "continuación partida en dos capas sin llegar al tope"
            );
            if a.destino + a.compases == b.destino {
                // Las voces sintéticas cantan hasta el borde: el relevo funde.
                let misma = a.pista_id == b.pista_id && a.origen + a.compases == b.origen;
                assert!(
                    misma || (a.salida >= 0.25 && b.entrada >= 0.25),
                    "{a:?} -> {b:?}"
                );
            }
        }
    }

    #[test]
    fn sin_pulso_claro_no_se_superpone_una_voz() {
        let mut f = biblioteca(8, true);
        for x in &mut f {
            x.estructura.confianza_pulso = 0.4;
        }
        let plan = componer(&f, &peticion(4, 240, 1)).unwrap();
        assert!(plan.capas.iter().all(|c| c.material != Material::Voz));
    }

    fn pet(canciones: u32, duracion: u32, estilo: Estilo, semilla: i64) -> Peticion {
        Peticion {
            duracion_seg: duracion,
            canciones,
            estilo,
            semilla,
        }
    }

    /// Las bases activas en cada compás del destino.
    fn bases_en(plan: &Plan, k: u32) -> Vec<&Capa> {
        plan.capas
            .iter()
            .filter(|c| c.material.es_base() && c.destino <= k && k < c.destino + c.compases)
            .collect()
    }

    #[test]
    fn dieciseis_canciones_de_una_biblioteca_de_ochenta() {
        // Ochenta pistas con tonalidades de toda la rueda: muchas parejas no se
        // superponen, pero todas pueden sonar en secuencia.
        let f: Vec<Fuente> = (0..80)
            .map(|i| {
                fuente(
                    i as i64 + 1,
                    118.0 + (i % 9) as f64,
                    (i * 7) % 12,
                    i % 3 == 0,
                    true,
                )
            })
            .collect();
        for estilo in [Estilo::Equilibrado, Estilo::Fiesta, Estilo::Relampago] {
            let plan = componer(&f, &pet(16, 420, estilo, 11)).unwrap();
            comprobar_invariantes(&plan, &f);
            let n = plan.pistas_reconocibles().len();
            assert!(n >= 16, "{estilo:?}: {n} de 16");
        }
    }

    /// Pistas cuyas armonías no casan entre sí: cada una un racimo de
    /// semitonos propio.
    fn incompatibles(n: usize, stems: bool) -> Vec<Fuente> {
        (0..n)
            .map(|i| {
                let mut x = fuente(
                    i as i64 + 1,
                    120.0 + (i % 5) as f64,
                    (i * 5) % 12,
                    false,
                    stems,
                );
                // Un acorde distinto en cada compás, sin relación entre pistas:
                // ninguna transposición los alinea compás a compás.
                let n = x.estructura.croma.len();
                x.estructura.croma = (0..n)
                    .map(|k| {
                        let h = (i * 7919 + k * 104_729 + i * k * 31) % 12;
                        let mut c = [0.0f32; 12];
                        for d in [0usize, 1, 6] {
                            c[(h + d) % 12] = 0.577;
                        }
                        c
                    })
                    .collect();
                x
            })
            .collect()
    }

    #[test]
    fn sin_parejas_superponibles_se_cumplen_las_canciones_en_secuencia() {
        let f = incompatibles(12, true);
        let plan = componer(&f, &pet(8, 300, Estilo::Equilibrado, 2)).unwrap();
        comprobar_invariantes(&plan, &f);
        assert!(
            plan.pistas_reconocibles().len() >= 8,
            "{:?}",
            plan.pistas_reconocibles()
        );
        // Nada se fuerza encima: ninguna voz ajena sobre armonías que chocan.
        assert!(plan.capas.iter().all(|c| c.material != Material::Voz));
        // Y no es una cadena de cruces a ciegas: hay cortes en las fronteras.
        assert!(plan.capas.iter().any(|c| c.transicion == Transicion::Corte));
    }

    #[test]
    fn la_armonia_compas_a_compas_manda_sobre_la_media() {
        // Pistas A (do, fa#, do, fa#…) y B (fa#, do, fa#, do…): el croma medio
        // es idéntico, pero compás a compás una choca con la otra siempre.
        let do_ = {
            let mut c = [0.0f32; 12];
            for g in [0, 4, 7] {
                c[g] = 0.577;
            }
            c
        };
        let fa_s = rotar(&do_, 6);
        let f: Vec<Fuente> = (0..12)
            .map(|i| {
                let mut x = fuente(i as i64 + 1, 120.0, 0, false, true);
                let n = x.estructura.croma.len();
                x.estructura.croma = (0..n)
                    .map(|k| if (k + i % 2) % 2 == 0 { do_ } else { fa_s })
                    .collect();
                x
            })
            .collect();
        let plan = componer(&f, &pet(8, 300, Estilo::Equilibrado, 4)).unwrap();
        comprobar_invariantes(&plan, &f);
        for v in plan.capas.iter().filter(|c| c.material == Material::Voz) {
            for b in bases_en(&plan, v.destino) {
                // La alternancia depende de la paridad del compás de origen.
                let fase = |c: &Capa| (c.pista_id - 1 + i64::from(c.origen)) % 2;
                assert_eq!(fase(v), fase(b), "{v:?} sobre {b:?}");
            }
        }
    }

    /// Una clave de son en 16 pasos por compás: el lado de tres (pasos 0, 6
    /// y 12) y el de dos (4 y 8), sobre un tumbao que se repite.
    fn clave(tres_dos: bool, nb: usize) -> Vec<[u8; PASOS]> {
        (0..nb)
            .map(|k| {
                let tres = (k % 2 == 0) == tres_dos;
                let mut p = [20u8; PASOS];
                for j in [3usize, 7, 11, 15] {
                    p[j] = 120;
                }
                let golpes: &[usize] = if tres { &[0, 6, 12] } else { &[4, 8] };
                for &j in golpes {
                    p[j] = 255;
                }
                p
            })
            .collect()
    }

    #[test]
    fn la_clave_opuesta_se_detecta_con_confianza_y_un_pulso_de_cuatro_no() {
        let mut a = fuente(1, 120.0, 0, false, true).estructura;
        let mut b = a.clone();
        let nb = a.n_compases();
        a.patron = clave(true, nb);
        b.patron = clave(false, nb);
        let opuesta = relacion_entre(&a, 0, &b, 0, 8).unwrap();
        assert!(opuesta.conflicto > 0.5, "{opuesta:?}");
        // Desplazada un compás, la misma pista queda en fase.
        let en_fase = relacion_entre(&a, 0, &b, 1, 8).unwrap();
        assert!(en_fase.conflicto < 0.05, "{en_fase:?}");
        let misma = relacion_entre(&a, 0, &a, 2, 8).unwrap();
        assert!(misma.conflicto < 0.05 && misma.groove > 0.9, "{misma:?}");
        // Un bombo a negras se repite cada compás: no hay fase de dos compases
        // que pueda cruzarse.
        let mut c = a.clone();
        c.patron = vec![
            {
                let mut p = [10u8; PASOS];
                for j in [0usize, 4, 8, 12] {
                    p[j] = 255;
                }
                p
            };
            nb
        ];
        let llano = relacion_entre(&a, 0, &c, 0, 8).unwrap();
        assert!(llano.conflicto < 0.05, "{llano:?}");
        // Con una rejilla dudosa, el mismo conflicto pesa mucho menos.
        b.confianza_pulso = 0.4;
        b.confianza_compas = 0.0;
        let dudosa = relacion_entre(&a, 0, &b, 0, 8).unwrap();
        assert!(dudosa.conflicto < opuesta.conflicto * 0.5, "{dudosa:?}");
        assert!(dudosa.incierto > 0.4, "{dudosa:?}");
    }

    #[test]
    fn una_voz_no_cruza_la_clave_pero_la_cancion_entra_igual() {
        let mut f = biblioteca(12, true);
        for (i, x) in f.iter_mut().enumerate() {
            let nb = x.estructura.n_compases();
            x.estructura.patron = clave(i % 2 == 0, nb);
        }
        let plan = componer(&f, &pet(8, 300, Estilo::Equilibrado, 6)).unwrap();
        comprobar_invariantes(&plan, &f);
        let orientacion = |c: &Capa| (c.pista_id - 1 + i64::from(c.origen)) % 2;
        for v in plan.capas.iter().filter(|c| c.material == Material::Voz) {
            for b in bases_en(&plan, v.destino) {
                assert_eq!(orientacion(v), orientacion(b), "{v:?} sobre {b:?}");
            }
        }
        assert!(plan.pistas_reconocibles().len() >= 8);
    }

    #[test]
    fn los_relevos_de_voz_caen_donde_la_voz_calla() {
        // Las voces respiran solo al final de cada sección de 16 compases y
        // cantan a través de la mitad: un relevo a mitad de sección corta.
        let mut f = biblioteca(12, true);
        for x in &mut f {
            let nb = x.estructura.n_compases();
            let v = x.estructura.voz.clone().unwrap();
            x.estructura.voz_cuartos = Some(
                (0..nb * 4)
                    .map(|q| {
                        let ultimo = q % 64 >= 62;
                        if ultimo {
                            0.0
                        } else {
                            v[q / 4]
                        }
                    })
                    .collect(),
            );
        }
        let mut relevos = (0, 0);
        for semilla in 1..5 {
            let plan = componer(&f, &pet(8, 300, Estilo::Equilibrado, semilla)).unwrap();
            let voces: Vec<&Capa> = plan
                .capas
                .iter()
                .filter(|c| c.material == Material::Voz)
                .collect();
            for par in voces.windows(2) {
                let (a, b) = (par[0], par[1]);
                if a.destino + a.compases == b.destino {
                    relevos.0 += 1;
                    if (a.origen + a.compases) % 16 == 0 {
                        relevos.1 += 1;
                    }
                }
            }
        }
        assert!(relevos.0 > 0);
        assert!(
            relevos.1 * 10 >= relevos.0 * 8,
            "{} de {} relevos en una pausa",
            relevos.1,
            relevos.0
        );
    }

    #[test]
    fn una_voz_que_se_suelta_a_media_frase_termina_con_su_cola() {
        let f = biblioteca(12, true);
        for semilla in 1..6 {
            let plan = componer(&f, &pet(8, 300, Estilo::Equilibrado, semilla)).unwrap();
            for v in plan.capas.iter().filter(|c| c.material == Material::Voz) {
                let fin = v.destino + v.compases;
                let sigue_voz = plan
                    .capas
                    .iter()
                    .any(|c| c.material == Material::Voz && c.destino == fin);
                let mezcla = bases_en(&plan, fin)
                    .iter()
                    .any(|c| c.material == Material::Mezcla);
                // Las voces sintéticas cantan toda la sección: cortar a mitad
                // (fin no múltiplo de 16) sin relevo exige cola.
                if fin < plan.compases && !sigue_voz && !mezcla && (v.origen + v.compases) % 16 != 0
                {
                    assert!(v.salida >= 1.0, "semilla {semilla}: {v:?}");
                }
            }
        }
    }

    #[test]
    fn abre_por_un_principio_y_cierra_sin_cortar_en_seco() {
        let f = biblioteca(12, true);
        for estilo in [Estilo::Equilibrado, Estilo::Fiesta, Estilo::Relampago] {
            let plan = componer(&f, &pet(8, 300, estilo, 3)).unwrap();
            let primera = bases_en(&plan, 0);
            assert!(
                primera
                    .iter()
                    .any(|c| c.origen == 0 || c.transicion == Transicion::Filtrada),
                "{estilo:?}: {primera:?}"
            );
            for c in plan
                .capas
                .iter()
                .filter(|c| c.destino + c.compases == plan.compases)
            {
                assert!(c.salida >= 0.25, "{estilo:?}: final en seco {c:?}");
            }
        }
    }

    #[test]
    fn las_voces_casi_nunca_se_transponen_y_la_mezcla_original_no_domina() {
        let f = biblioteca(16, true);
        let (mut voces, mut quietas, mut frases, mut mezclas) = (0, 0, 0, 0);
        for semilla in 1..6 {
            let plan = componer(&f, &pet(10, 360, Estilo::Equilibrado, semilla)).unwrap();
            for v in plan.capas.iter().filter(|c| c.material == Material::Voz) {
                voces += 1;
                quietas += usize::from(v.semitonos == 0);
            }
            for k in (0..plan.compases).step_by(8) {
                frases += 1;
                mezclas += usize::from(
                    bases_en(&plan, k)
                        .iter()
                        .all(|c| c.material == Material::Mezcla),
                );
            }
        }
        assert!(
            voces > 0 && quietas * 10 >= voces * 8,
            "{quietas} de {voces}"
        );
        assert!(
            mezclas * 2 <= frases,
            "{mezclas} de {frases} frases con el original"
        );
    }

    #[test]
    fn los_estilos_tienen_identidades_distintas() {
        let f = biblioteca(20, true);
        let eq = componer(&f, &pet(10, 360, Estilo::Equilibrado, 1)).unwrap();
        let fi = componer(&f, &pet(10, 360, Estilo::Fiesta, 1)).unwrap();
        let re = componer(&f, &pet(10, 360, Estilo::Relampago, 1)).unwrap();
        // Relámpago cambia más a menudo: más capas de base y más cortas.
        let bases = |p: &Plan| p.capas.iter().filter(|c| c.material.es_base()).count();
        assert!(bases(&re) > bases(&eq), "{} {}", bases(&re), bases(&eq));
        // Fiesta: una curva con descansos antes de subir más, no una rampa.
        let c: Vec<f64> = (0..=100)
            .map(|i| Estilo::Fiesta.curva(f64::from(i) / 100.0))
            .collect();
        let bajadas = c.windows(3).filter(|w| w[1] < w[0] && w[1] <= w[2]).count();
        assert!(bajadas >= 2, "{bajadas}");
        let pico = c.iter().copied().fold(0.0f64, f64::max);
        let pos = c.iter().position(|v| *v == pico).unwrap();
        assert!((75..=95).contains(&pos), "clímax en {pos}");
        assert_ne!(eq.capas, fi.capas);
    }

    #[test]
    fn lo_que_no_cabe_se_dice_con_lo_que_haria_falta() {
        let f = biblioteca(30, true);
        let plan = componer(&f, &pet(24, 60, Estilo::Relampago, 1)).unwrap();
        let d = plan
            .degradaciones
            .iter()
            .find_map(|d| match d {
                Degradacion::NoCaben {
                    pedidas,
                    caben,
                    minimo_seg,
                } => Some((*pedidas, *caben, *minimo_seg)),
                _ => None,
            })
            .expect("debe decir que no caben");
        assert_eq!(d.0, 24);
        assert!(d.1 < 24 && d.2 > 60, "{d:?}");
        // Lo que cuenta como usado suena de verdad.
        assert!(plan.pistas_reconocibles().len() as u32 <= d.1);
    }
}
