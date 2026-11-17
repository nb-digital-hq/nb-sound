//! El icono de la bandeja del sistema: reproducir o pausar, anterior,
//! siguiente, volver a la ventana y salir, sin tener que buscarla.
//!
//! Cerrar la ventana sigue cerrando la aplicación: la bandeja es un
//! atajo, no un sitio donde la música se queda sonando sin ventana.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Wry};

use crate::idioma::t;
use crate::reproductor::{Buzon, Orden};

const ID: &str = "nb-sound";

/// El menú en el idioma en uso.
fn menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    Menu::with_items(
        app,
        &[
            &MenuItem::with_id(
                app,
                "pausar",
                t("Reproducir o pausar", "Play or pause", "Tocar ou pausar"),
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(
                app,
                "anterior",
                t("Anterior", "Previous", "Anterior"),
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(
                app,
                "siguiente",
                t("Siguiente", "Next", "Próxima"),
                true,
                None::<&str>,
            )?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(
                app,
                "mostrar",
                t("Mostrar NB Sound", "Show NB Sound", "Mostrar o NB Sound"),
                true,
                None::<&str>,
            )?,
            &MenuItem::with_id(app, "salir", t("Salir", "Quit", "Sair"), true, None::<&str>)?,
        ],
    )
}

/// Vuelve a escribir el menú tras un cambio de idioma. Sin bandeja, nada.
pub fn actualizar(app: &AppHandle) {
    let Some(icono) = app.tray_by_id(ID) else {
        return;
    };
    if let Err(error) = menu(app).and_then(|m| icono.set_menu(Some(m))) {
        tracing::info!(error = %error, "no se pudo traducir el menú de la bandeja");
    }
}

/// Pone el icono. Si el escritorio no tiene bandeja, no pasa nada.
pub fn poner(app: &AppHandle, buzon: Buzon) {
    let resultado = (|| -> tauri::Result<()> {
        let menu = menu(app)?;
        let mut constructor = TrayIconBuilder::with_id(ID)
            .tooltip("NB Sound")
            .menu(&menu)
            .on_menu_event(move |app, evento| match evento.id().as_ref() {
                "pausar" => buzon.enviar(Orden::PausarReanudar),
                "anterior" => buzon.enviar(Orden::Anterior),
                "siguiente" => buzon.enviar(Orden::Siguiente),
                "mostrar" => {
                    if let Some(v) = app.get_webview_window("main") {
                        let _ = v.unminimize();
                        let _ = v.show();
                        let _ = v.set_focus();
                    }
                }
                "salir" => app.exit(0),
                _ => {}
            });
        if let Some(icono) = app.default_window_icon() {
            constructor = constructor.icon(icono.clone());
        }
        constructor.build(app)?;
        Ok(())
    })();
    if let Err(error) = resultado {
        tracing::info!(error = %error, "sin icono en la bandeja");
    }
}
