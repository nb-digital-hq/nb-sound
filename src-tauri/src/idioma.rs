//! El idioma de lo que el runtime nombra por su cuenta.
//!
//! Casi todo el texto visible vive en los diccionarios de la interfaz, pero
//! unas pocas cosas las nombra el runtime: la bandeja del sistema, los filtros de
//! los diálogos nativos, las listas inteligentes que crea, las preselecciones del
//! ecualizador, la página del Modo fiesta y las explicaciones del DJ. Deben hablar
//! el mismo idioma que la interfaz, y cambiar con ella sin reiniciar.
//!
//! El idioma es un valor del proceso y no una consulta a la base porque se lee
//! desde sitios sin conexión a mano (el menú de la bandeja, el servidor de la
//! fiesta) y en caminos calientes; la preferencia `idioma` lo fija al arrancar y
//! cada vez que la interfaz la cambia.

use std::sync::atomic::{AtomicU8, Ordering};

/// Los idiomas que se hablan. El orden es el de la interfaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Idioma {
    Es,
    En,
    Pt,
}

impl Idioma {
    /// El código que guarda la preferencia (`es`, `en`, `pt`).
    #[must_use]
    pub fn desde_codigo(codigo: &str) -> Option<Self> {
        match codigo {
            "es" => Some(Self::Es),
            "en" => Some(Self::En),
            "pt" => Some(Self::Pt),
            _ => None,
        }
    }

    #[must_use]
    pub fn codigo(self) -> &'static str {
        match self {
            Self::Es => "es",
            Self::En => "en",
            Self::Pt => "pt",
        }
    }
}

static ACTUAL: AtomicU8 = AtomicU8::new(0);

// En las pruebas el idioma es de cada hilo: las pruebas corren en paralelo, y una
// que cambiara el idioma de todo el proceso haría fallar a otra que comprueba un
// nombre en español mientras tanto.
#[cfg(test)]
thread_local! {
    static EN_PRUEBA: std::cell::Cell<Option<Idioma>> = const { std::cell::Cell::new(None) };
}

/// El idioma en uso.
#[must_use]
pub fn actual() -> Idioma {
    #[cfg(test)]
    if let Some(i) = EN_PRUEBA.with(std::cell::Cell::get) {
        return i;
    }
    match ACTUAL.load(Ordering::Relaxed) {
        1 => Idioma::En,
        2 => Idioma::Pt,
        _ => Idioma::Es,
    }
}

/// Cambia el idioma del runtime.
pub fn fijar(idioma: Idioma) {
    #[cfg(test)]
    EN_PRUEBA.with(|c| c.set(Some(idioma)));
    #[cfg(not(test))]
    fijar_proceso(idioma);
}

#[cfg_attr(test, allow(dead_code))]
fn fijar_proceso(idioma: Idioma) {
    let v = match idioma {
        Idioma::Es => 0,
        Idioma::En => 1,
        Idioma::Pt => 2,
    };
    ACTUAL.store(v, Ordering::Relaxed);
}

/// Elige entre las tres versiones de un texto según el idioma en uso.
#[must_use]
pub fn t<'a>(es: &'a str, en: &'a str, pt: &'a str) -> &'a str {
    match actual() {
        Idioma::Es => es,
        Idioma::En => en,
        Idioma::Pt => pt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cada_texto_sale_en_el_idioma_fijado() {
        fijar(Idioma::En);
        assert_eq!(t("Salir", "Quit", "Sair"), "Quit");
        fijar(Idioma::Pt);
        assert_eq!(t("Salir", "Quit", "Sair"), "Sair");
        fijar(Idioma::Es);
        assert_eq!(t("Salir", "Quit", "Sair"), "Salir");
    }

    #[test]
    fn un_codigo_desconocido_no_cambia_el_idioma() {
        assert_eq!(Idioma::desde_codigo("fr"), None);
        assert_eq!(Idioma::desde_codigo("pt"), Some(Idioma::Pt));
        for i in [Idioma::Es, Idioma::En, Idioma::Pt] {
            assert_eq!(Idioma::desde_codigo(i.codigo()), Some(i));
        }
    }
}
