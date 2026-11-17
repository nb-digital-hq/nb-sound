//! Mediciones de DJ Mashups. Están ignoradas: dependen del equipo y
//! se ejecutan a mano en compilación optimizada, una a una, para que no
//! compitan entre sí:
//!
//! ```bash
//! cargo test --release --lib mashup::rendimiento -- --ignored --test-threads=1 --nocapture
//! ```
//!
//! Cada medición imprime una línea `METRICA {json}` comparable entre ejecuciones;
//! los resultados de referencia están en `docs/architecture/mashups.md`. Los
//! umbrales que comprueban no son de velocidad (dependen del equipo), sino de
//! forma: que el coste crezca como se diseñó (lineal y no combinatorio) y que la
//! memoria del render no dependa de la duración.
//!
//! Las que usan música real leen `NS_CORPUS_MUSICA` (una carpeta de MP3, en solo
//! lectura) y se saltan solas si no está.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use super::compositor::pruebas::biblioteca;
use super::compositor::{componer, Estilo, Peticion};
use super::estiramiento::{transformar, Vocoder};
use super::estructura;
use super::render::pruebas::Sintetico;
use super::render::{renderizar, TASA};

fn metrica(nombre: &str, datos: serde_json::Value) {
    println!(
        "METRICA {}",
        serde_json::json!({ "medicion": nombre, "datos": datos })
    );
}

/// El pico de memoria residente del proceso, en MB (Linux).
fn pico_mb() -> Option<f64> {
    let s = std::fs::read_to_string("/proc/self/status").ok()?;
    let kb: f64 = s
        .lines()
        .find(|l| l.starts_with("VmHWM:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()?;
    Some(kb / 1024.0)
}

fn temporal(nombre: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ns-rend-{nombre}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
#[ignore = "medición de rendimiento"]
fn el_compositor_escala_linealmente_de_5_a_100_canciones() {
    let mut tiempos = Vec::new();
    for n in [5usize, 10, 20, 50, 100] {
        let f = biblioteca(n + n / 4, true);
        let p = Peticion {
            duracion_seg: (n as u32 * 12).max(120),
            canciones: n as u32,
            estilo: if n >= 50 {
                Estilo::Relampago
            } else {
                Estilo::Equilibrado
            },
            semilla: 1,
        };
        let t = Instant::now();
        let plan = componer(&f, &p).unwrap();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        tiempos.push((n, ms));
        metrica(
            "compositor",
            serde_json::json!({
                "canciones_pedidas": n, "canciones_usadas": plan.pistas().len(),
                "capas": plan.capas.len(), "compases": plan.compases,
                "calidad": plan.calidad, "ms": ms,
            }),
        );
    }
    // De 5 a 100 canciones (×20) el tiempo no puede crecer más de ×400: una
    // búsqueda combinatoria crecería sin cota; la de haces, en torno a ×20–×60.
    let (_, t5) = tiempos[0];
    let (_, t100) = tiempos[4];
    assert!(t100 / t5.max(0.1) < 400.0, "{tiempos:?}");
}

#[test]
#[ignore = "medición de rendimiento"]
fn el_vocoder_procesa_mas_rapido_que_el_tiempo_real() {
    let x: Vec<f32> = (0..TASA as usize * 30)
        .map(|i| (i as f32 * 0.0313).sin() * 0.4 + (i as f32 * 0.0071).sin() * 0.2)
        .collect();
    let mut v = Vocoder::new();
    let t = Instant::now();
    let _ = transformar(&mut v, &x, &x, 1.05, 1).unwrap();
    let s = t.elapsed().as_secs_f64();
    metrica(
        "vocoder",
        serde_json::json!({ "segundos_audio": 30, "segundos_cpu": s, "factor_tiempo_real": 30.0 / s }),
    );
    assert!(30.0 / s > 1.0);
}

#[test]
#[ignore = "medición de rendimiento"]
fn el_render_no_gasta_mas_memoria_por_durar_mas() {
    for (n, dur) in [(5usize, 120u32), (20, 360), (50, 900)] {
        let f = biblioteca(n + 4, true);
        let plan = componer(
            &f,
            &Peticion {
                duracion_seg: dur,
                canciones: n as u32,
                estilo: Estilo::Equilibrado,
                semilla: 2,
            },
        )
        .unwrap();
        let est: HashMap<_, _> = f
            .iter()
            .map(|x| (x.pista_id, x.estructura.clone()))
            .collect();
        let bpm = f
            .iter()
            .map(|x| (x.pista_id, x.estructura.bpm.unwrap()))
            .collect();
        let mut o = Sintetico { bpm, lecturas: 0 };
        let d = temporal("render");
        let m = renderizar(&plan, &est, &mut o, &d.join("m.wav"), &mut |_| true).unwrap();
        metrica(
            "render",
            serde_json::json!({
                "canciones": n, "duracion_seg": m.duracion_seg, "capas": m.capas,
                "segundos_render": m.segundos_render,
                "factor_tiempo_real": m.duracion_seg / m.segundos_render,
                "acumulador_max_seg": m.acumulador_max_seg, "lufs": m.lufs,
                "pico_dbfs": m.pico_dbfs, "pico_rss_mb": pico_mb(),
            }),
        );
        assert!(m.acumulador_max_seg < 70.0);
        let _ = std::fs::remove_dir_all(d);
    }
}

/// Los MP3 de `NS_CORPUS_MUSICA`, en todas sus subcarpetas y en orden estable.
/// Se leen; nunca se escribe en esa carpeta.
fn corpus() -> Option<Vec<PathBuf>> {
    fn recorrer(d: &std::path::Path, v: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                recorrer(&p, v);
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("mp3")) {
                v.push(p);
            }
        }
    }
    let dir = PathBuf::from(std::env::var_os("NS_CORPUS_MUSICA")?);
    let mut v = Vec::new();
    recorrer(&dir, &mut v);
    v.sort();
    (!v.is_empty()).then_some(v)
}

#[test]
#[ignore = "necesita un corpus local"]
fn el_analisis_estructural_de_canciones_reales() {
    let Some(archivos) = corpus() else { return };
    let (mut audio, mut cpu, mut fiables) = (0.0, 0.0, 0);
    let n = archivos.len().min(20);
    for r in archivos.iter().step_by((archivos.len() / n).max(1)).take(n) {
        let t = Instant::now();
        let Ok(e) = estructura::analizar(r) else {
            continue;
        };
        cpu += t.elapsed().as_secs_f64();
        audio += e.duracion;
        fiables += usize::from(e.rejilla_fiable());
    }
    metrica(
        "estructura_corpus",
        serde_json::json!({
            "canciones": n, "minutos_audio": audio / 60.0, "segundos_cpu": cpu,
            "segundos_por_minuto": cpu / (audio / 60.0), "rejilla_fiable": fiables,
            "pico_rss_mb": pico_mb(),
        }),
    );
}

/// El modelo y ONNX Runtime reales, como en `karaoke`: `NS_MODELO_KARAOKE` y
/// `NS_ONNXRUNTIME`.
fn modelo_real() -> Option<PathBuf> {
    let modelo = PathBuf::from(std::env::var_os("NS_MODELO_KARAOKE")?);
    let ort = PathBuf::from(std::env::var_os("NS_ONNXRUNTIME")?);
    crate::onnx::fijar_ruta(&ort);
    Some(modelo)
}

/// Los hilos de la medición: `NS_HILOS` para comparar, o los de un mashup.
fn hilos() -> usize {
    std::env::var("NS_HILOS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(crate::karaoke::hilos_cola)
}

#[test]
#[ignore = "necesita el modelo de karaoke y un corpus local"]
fn la_separacion_de_cuatro_stems_con_el_modelo_real() {
    let (Some(archivos), Some(modelo)) = (corpus(), modelo_real()) else {
        return;
    };
    let (izq, der) = crate::karaoke::cargar(&archivos[0]).unwrap();
    let segundos = izq.len() as f64 / f64::from(TASA);
    let t = Instant::now();
    let stems = crate::karaoke::separar_archivo_stems_de(
        &modelo,
        &izq,
        &der,
        [true; 4],
        hilos(),
        &mut |_| true,
    )
    .unwrap();
    let cpu = t.elapsed().as_secs_f64();
    // La suma de los cuatro stems reconstruye la mezcla: HTDemucs reparte la
    // señal, no la inventa. Se mide la relación señal/residuo de la suma.
    let (mut senal, mut residuo) = (0.0f64, 0.0f64);
    for (i, x) in izq.iter().enumerate() {
        let suma: f32 = stems.iter().map(|s| s.as_ref().unwrap().0[i]).sum();
        senal += f64::from(*x).powi(2);
        residuo += f64::from(x - suma).powi(2);
    }
    let snr = 10.0 * (senal / residuo.max(1e-12)).log10();
    let d = temporal("stems");
    let cache = super::stems::CacheStems::abrir(d.clone(), super::stems::TOPE_BYTES).unwrap();
    let t = Instant::now();
    let bytes = cache.guardar(&"0".repeat(64), &stems).unwrap();
    metrica(
        "separacion_real",
        serde_json::json!({
            "segundos_audio": segundos, "segundos_cpu": cpu, "hilos": hilos(),
            "factor_tiempo_real": segundos / cpu, "flac_mb": bytes as f64 / 1e6,
            "mb_por_minuto": bytes as f64 / 1e6 / (segundos / 60.0),
            "escritura_flac_seg": t.elapsed().as_secs_f64(),
            "snr_suma_db": snr, "pico_rss_mb": pico_mb(),
        }),
    );
    // Más de 20 dB: la suma de stems es la mezcla salvo el error del modelo.
    assert!(snr > 20.0, "{snr}");
    for s in &stems {
        assert_eq!(s.as_ref().unwrap().0.len(), izq.len());
    }
    let _ = std::fs::remove_dir_all(d);
}

#[test]
#[ignore = "necesita el modelo de karaoke y un corpus local"]
fn un_mashup_real_de_canciones_del_corpus_con_htdemucs() {
    use super::persistencia::{self, PeticionMashup};
    use super::servicio::{Aviso, Mashups, Separador};
    use std::sync::{Arc, Mutex};
    let (Some(archivos), Some(modelo)) = (corpus(), modelo_real()) else {
        return;
    };
    let n: usize = std::env::var("NS_MASHUP_CANCIONES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(12);
    let dir = temporal("mashup-real");
    let rutas = crate::paths::Rutas::bajo(&dir).unwrap();
    let db = crate::db::BaseDeDatos::abrir(&rutas.base_de_datos()).unwrap();
    // Los rasgos de cada pista, con el analizador de siempre: es lo que el
    // prefiltro usa en la aplicación.
    let t = Instant::now();
    let mut insertadas = 0;
    for (i, r) in archivos
        .iter()
        .step_by((archivos.len() / n).max(1))
        .take(n)
        .enumerate()
    {
        let Ok(ra) = crate::rasgos::analizar(r) else {
            continue;
        };
        let conn = db.conexion().unwrap();
        conn.execute(
            "INSERT INTO pistas (id, titulo, artista_nombre, album_titulo, ruta) VALUES (?1, ?2, ?3, '', ?4)",
            rusqlite::params![i as i64 + 1, format!("Pista {i}"), format!("Artista {i}"), r.to_string_lossy()],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO rasgos (pista_id, version, bpm, tonalidad, modo, energia) VALUES (?1, 'v', ?2, ?3, ?4, ?5)",
            rusqlite::params![i as i64 + 1, ra.bpm, ra.tonalidad, ra.modo, ra.energia],
        )
        .unwrap();
        insertadas += 1;
    }
    let rasgos_seg = t.elapsed().as_secs_f64();
    let avisos = Arc::new(Mutex::new(Vec::new()));
    let a = Arc::clone(&avisos);
    let m = Mashups::con_separador(
        db.clone(),
        rutas.clone(),
        move |x| a.lock().unwrap().push(x),
        move || {
            let modelo = modelo.clone();
            let s: Separador = Arc::new(move |izq, der, seguir| {
                crate::karaoke::separar_archivo_stems_de(
                    &modelo,
                    izq,
                    der,
                    [true; 4],
                    crate::karaoke::hilos_cola(),
                    seguir,
                )
            });
            Some(s)
        },
    )
    .unwrap();
    let pedir = |semilla| PeticionMashup {
        duracion_seg: 120,
        canciones: 4,
        estilo: Estilo::Equilibrado,
        semilla: Some(semilla),
        pistas: vec![],
        playlist_id: None,
    };
    let esperar = |id: i64| {
        while !avisos
            .lock()
            .unwrap()
            .contains(&Aviso::Terminado { mashup_id: id })
        {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    };
    let t = Instant::now();
    let id = m.generar(pedir(1)).unwrap();
    esperar(id);
    let primera = t.elapsed().as_secs_f64();
    let v = persistencia::vista(&db, id).unwrap();
    assert_eq!(v.estado, "lista", "{:?}", v.error);
    let t = Instant::now();
    let id2 = m.generar(pedir(1)).unwrap();
    esperar(id2);
    let segunda = t.elapsed().as_secs_f64();
    let plan = v.plan.clone().unwrap();
    let met = v.metricas.clone().unwrap();
    metrica(
        "mashup_real",
        serde_json::json!({
            "pistas_candidatas": insertadas, "rasgos_seg": rasgos_seg,
            "primera_generacion_seg": primera, "con_cache_seg": segunda,
            "canciones": v.pistas.len(), "capas": plan.capas.len(),
            "voces": plan.capas.iter().filter(|c| c.material == super::compositor::Material::Voz).count(),
            "bpm": plan.bpm, "calidad": plan.calidad, "degradaciones": plan.degradaciones,
            "lufs": met.lufs, "pico_dbfs": met.pico_dbfs, "recortes": met.recortes,
            "salto_max": met.salto_max, "silencio_max_seg": met.silencio_max_seg,
            "duracion_seg": met.duracion_seg, "render_seg": met.segundos_render,
            "cache_stems_mb": dir_mb(&super::servicio::carpeta_stems(&rutas)),
            "pico_rss_mb": pico_mb(),
        }),
    );
    assert!((met.lufs + 14.0).abs() <= 1.0);
    assert_eq!(met.recortes, 0);
    assert!(
        segunda < primera / 2.0,
        "la caché no ahorró trabajo: {primera} → {segunda}"
    );
    // Se deja el WAV para escucharlo si se pide.
    if let Some(destino) = std::env::var_os("NS_MASHUP_SALIDA") {
        std::fs::copy(m.ruta(id).unwrap(), destino).unwrap();
    }
    let _ = std::fs::remove_dir_all(dir);
}

fn dir_mb(d: &std::path::Path) -> f64 {
    fn suma(d: &std::path::Path) -> u64 {
        std::fs::read_dir(d).map_or(0, |r| {
            r.flatten()
                .map(|e| {
                    let p = e.path();
                    if p.is_dir() {
                        suma(&p)
                    } else {
                        e.metadata().map_or(0, |m| m.len())
                    }
                })
                .sum()
        })
    }
    suma(d) as f64 / 1e6
}

/// Mashups con la biblioteca y la caché de stems reales, sin separar nada
/// nuevo y sin tocar los datos: `NS_DATOS_REALES` es la carpeta de datos de la
/// aplicación (se copia la base con `VACUUM INTO`) y `NS_CACHE_REALES` la de
/// caché (los stems se enlazan con enlaces duros: la poda de la copia solo
/// quita enlaces de la copia). Las canciones se leen en solo lectura.
/// `NS_MASHUP_SALIDA` (una carpeta) guarda los WAV para escucharlos.
#[test]
#[ignore = "necesita los datos y la caché reales"]
fn mashups_reales_desde_una_copia_de_los_datos() {
    use super::persistencia::{self, PeticionMashup};
    use super::servicio::{Aviso, Mashups};
    use std::sync::{Arc, Mutex};
    let (Some(datos), Some(cache)) = (
        std::env::var_os("NS_DATOS_REALES").map(PathBuf::from),
        std::env::var_os("NS_CACHE_REALES").map(PathBuf::from),
    ) else {
        return;
    };
    let dir = temporal("reales");
    let rutas = crate::paths::Rutas::bajo(&dir).unwrap();
    {
        let origen = rusqlite::Connection::open_with_flags(
            datos.join("nb-sound.sqlite"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        origen
            .execute(
                "VACUUM INTO ?1",
                [rutas.base_de_datos().to_string_lossy().to_string()],
            )
            .unwrap();
    }
    let stems = super::servicio::carpeta_stems(&rutas);
    let estado = std::process::Command::new("cp")
        .arg("-al")
        .arg(cache.join("stems-mashup"))
        .arg(&stems)
        .status()
        .unwrap();
    assert!(estado.success());
    let db = crate::db::BaseDeDatos::abrir(&rutas.base_de_datos()).unwrap();
    // Las pistas con stems en caché, y la biblioteca para completar 80.
    let conn = db.conexion().unwrap();
    let mut con_stems: Vec<(i64, String)> = Vec::new();
    for e in std::fs::read_dir(&stems).unwrap().flatten() {
        let n = e.file_name().to_string_lossy().to_string();
        let sha = n.split('-').next().unwrap_or("").to_string();
        if let Ok(f) = conn.query_row(
            "SELECT id, artista_nombre FROM pistas WHERE sha256 = ?1 AND estado = 'biblioteca'",
            [&sha],
            |f| Ok((f.get::<_, i64>(0)?, f.get::<_, String>(1)?)),
        ) {
            con_stems.push(f);
        }
    }
    con_stems.sort();
    let mut otras: Vec<i64> = conn
        .prepare("SELECT p.id FROM pistas p JOIN rasgos r ON r.pista_id = p.id WHERE p.estado = 'biblioteca' ORDER BY p.id")
        .unwrap()
        .query_map([], |f| f.get(0))
        .unwrap()
        .flatten()
        .collect();
    otras.retain(|id| !con_stems.iter().any(|(x, _)| x == id));
    drop(conn);
    let avisos = Arc::new(Mutex::new(Vec::new()));
    let a = Arc::clone(&avisos);
    let m = Mashups::con_separador(
        db.clone(),
        rutas.clone(),
        move |x| a.lock().unwrap().push(x),
        || None,
    )
    .unwrap();
    let esperar = |id: i64| loop {
        let hecho = avisos
            .lock()
            .unwrap()
            .iter()
            .any(|x| matches!(x, Aviso::Terminado { mashup_id } if *mashup_id == id));
        if hecho {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    };
    let salida = std::env::var_os("NS_MASHUP_SALIDA").map(PathBuf::from);
    let salsa: Vec<i64> = con_stems
        .iter()
        .filter(|(_, a)| a.contains("Tito Rojas"))
        .map(|x| x.0)
        .collect();
    let todas_stems: Vec<i64> = con_stems.iter().map(|x| x.0).collect();
    let mut ochenta = todas_stems.clone();
    ochenta.extend(
        otras
            .iter()
            .step_by((otras.len() / 40).max(1))
            .take(80 - todas_stems.len().min(80)),
    );
    ochenta.truncate(80);
    let casos: Vec<(&str, Vec<i64>, u32, u32, Estilo)> = vec![
        (
            "equilibrada",
            todas_stems.clone(),
            6,
            180,
            Estilo::Equilibrado,
        ),
        ("fiesta", todas_stems.clone(), 8, 240, Estilo::Fiesta),
        ("relampago", todas_stems.clone(), 12, 180, Estilo::Relampago),
        (
            "dieciseis_de_ochenta",
            ochenta,
            16,
            360,
            Estilo::Equilibrado,
        ),
        ("salsa", salsa, 4, 150, Estilo::Equilibrado),
        ("no_caben", todas_stems, 30, 60, Estilo::Relampago),
    ];
    for (nombre, pistas, canciones, duracion, estilo) in casos {
        if pistas.len() < 2 {
            continue;
        }
        let t = Instant::now();
        let id = m
            .generar(PeticionMashup {
                duracion_seg: duracion,
                canciones,
                estilo,
                semilla: Some(1),
                pistas: pistas.clone(),
                playlist_id: None,
            })
            .unwrap();
        esperar(id);
        let seg = t.elapsed().as_secs_f64();
        let v = persistencia::vista(&db, id).unwrap();
        assert_eq!(v.estado, "lista", "{nombre}: {:?}", v.error);
        let plan = v.plan.clone().unwrap();
        let met = v.metricas.clone().unwrap();
        let cuenta = |mat: super::compositor::Material| {
            plan.capas.iter().filter(|c| c.material == mat).count()
        };
        let trans = |t: super::compositor::Transicion| {
            plan.capas
                .iter()
                .filter(|c| c.material.stems().len() != 1 && c.transicion == t)
                .count()
        };
        metrica(
            "mashup_real_copia",
            serde_json::json!({
                "caso": nombre, "candidatas": pistas.len(), "pedidas": canciones,
                "reconocibles": plan.pistas_reconocibles().len(), "segundos": seg,
                "voces": cuenta(super::compositor::Material::Voz),
                "mezclas": cuenta(super::compositor::Material::Mezcla),
                "instrumentales": cuenta(super::compositor::Material::Instrumental),
                "baterias": cuenta(super::compositor::Material::Bateria),
                "filtradas": trans(super::compositor::Transicion::Filtrada),
                "cortes": trans(super::compositor::Transicion::Corte),
                "voces_transpuestas": plan.capas.iter().filter(|c| c.material == super::compositor::Material::Voz && c.semitonos != 0).count(),
                "degradaciones": plan.degradaciones, "bpm": plan.bpm,
                "lufs": met.lufs, "pico_dbfs": met.pico_dbfs, "pico_verdadero_dbtp": met.pico_verdadero_dbtp,
                "recortes": met.recortes, "salto_max": met.salto_max,
                "silencio_max_seg": met.silencio_max_seg, "duracion_seg": met.duracion_seg,
                "render_seg": met.segundos_render, "pico_rss_mb": pico_mb(),
            }),
        );
        assert!((met.lufs + 14.0).abs() <= 1.0, "{nombre}: {}", met.lufs);
        assert_eq!(met.recortes, 0, "{nombre}");
        if let Some(s) = &salida {
            std::fs::create_dir_all(s).unwrap();
            std::fs::copy(m.ruta(id).unwrap(), s.join(format!("{nombre}.wav"))).unwrap();
            std::fs::write(
                s.join(format!("{nombre}.plan.json")),
                serde_json::to_string_pretty(&plan).unwrap(),
            )
            .unwrap();
        }
    }
    let _ = std::fs::remove_dir_all(dir);
}
