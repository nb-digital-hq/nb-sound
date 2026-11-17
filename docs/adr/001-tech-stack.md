# ADR-001: Pila tecnológica

**Fecha:** 2026-10-29
**Estado:** Aceptado

## Contexto

Un reproductor de escritorio que reúne decodificación, catalogación, inferencia local, un servidor para el
teléfono e integración con el sistema (controles multimedia, bandeja) necesita una pila que se empaquete en un
solo instalador por plataforma y que no dependa de runtimes nativos instalados aparte. El resto de la suite NB
Digital (NB Books) ya usa Tauri, Rust y Svelte, y el objetivo es que el ecosistema se sienta como uno solo.

## Alternativas consideradas

1. **Qt/QML con un núcleo en Python.** Rechazada: varios runtimes nativos en un mismo proceso, un empaquetado frágil
   y una pila distinta a la del resto de la suite.
2. **Tauri 2 con el núcleo en Python.** Rechazada en [ADR-002](002-no-python.md): conserva el problema de
   distribución, que es el que más daño hace.
3. **Tauri 2 + SvelteKit 2 (estático) + Svelte 5 + TypeScript estricto, núcleo en Rust.** Aceptada.

## Decisión

| Capa | Elección |
|---|---|
| Ventana y sistema | Tauri 2, con `tray-icon`; `protocol-asset` acotado a la caché |
| Interfaz | SvelteKit 2 con `adapter-static`, Svelte 5 (runas), TypeScript estricto |
| Diseño | Material 3 con `m3-svelte` y Material Color Utilities ([ADR-008](008-design-system.md)) |
| Núcleo | Rust (toolchain fijado en `rust-toolchain.toml`, edición 2021), `thiserror`, `tracing`, `serde` |
| Datos | SQLite con `rusqlite` (`bundled`) y `r2d2` ([ADR-004](004-data-ownership.md)) |
| Tipos del puente | Generados con `ts-rs`, nunca escritos a mano |
| Calidad | Biome y Prettier, `rustfmt`, Clippy con `-D warnings`, `cargo-deny`, Vitest |
| Paquetes | `pnpm` 10 o posterior, Node 22 o posterior |

Complementos de Tauri, todos oficiales: `tauri-plugin-dialog` para los selectores nativos (se usa solo desde Rust:
ninguna ruta cruza el puente desde el webview) y `tauri-plugin-updater` para las actualizaciones, que solo se usa
dentro de `net.rs` ([ADR-009](009-settings-and-network.md)). `image` (solo JPEG y PNG) genera miniaturas y el color
dominante de cada portada fuera del webview.

Linux se distribuye como `.deb`, `.rpm`, AppImage y un `.tar.gz` con el árbol `usr/` para las distribuciones sin
`dpkg` ni `rpm`. El actualizador no sabe instalar ese `.tar.gz`: la aplicación lo descarga **en memoria**, verifica
su firma minisign con `minisign-verify` (la misma versión que ya trae el complemento) contra la clave pública de
`tauri.conf.json` y entrega los bytes a `tar` por la entrada estándar, elevado con `sudo -n` o `pkexec`. Nada pasa
por un archivo temporal, porque un archivo en `/tmp` que luego extrae root es una carrera para escalar privilegios.

## Consecuencias

- **Positivas:** un solo binario, sin intérprete; tipos de extremo a extremo; los guardianes y el método de la suite
  se reutilizan.
- **Negativas:** toda la lógica de catalogación e inteligencia está escrita en Rust.
- **Riesgos y cómo se vigilan:** WebKitGTK en Linux (medios servidos desde el esquema propio, desenfoques,
  animación en reposo); `check-ui` vigila esas trampas.
