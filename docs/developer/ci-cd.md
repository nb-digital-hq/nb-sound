# Integración continua

> **Estado:** Implementado. Linux se verifica entero; Windows y macOS compilan, pasan los lints y compilan las
> pruebas, pero **no se han verificado en un equipo real**.

## Flujos

| Flujo | Cuándo | Qué hace |
|---|---|---|
| `ci.yml` | Cada push a `main`, cada pull request | Ver abajo |
| `ediciones.yml` | A mano, al preparar una versión | Paquetes firmados de Windows y macOS como artefactos ([empaquetado](packaging.md)) |
| `catalogo-launcher.yml` | Al publicar una versión, cada lunes y a mano | Genera, firma y sube el catálogo para NB Launcher ([ADR-016](../adr/016-launcher-catalog.md)) |

## `ci.yml`

| Trabajo | Sistema | Qué comprueba |
|---|---|---|
| Procedencia y convenciones | Linux, solo en pull requests | Asunto de cada commit en Conventional Commits y `Signed-off-by` |
| Contratos de workflows | Linux | `actionlint`, con versión y huella fijadas |
| `pnpm verify` | Ubuntu 22.04 | Lint, tipos, pruebas de la interfaz, guardianes y presupuesto de búsqueda |
| Runtime | Ubuntu 22.04, Windows, macOS | `cargo fmt --check` y `cargo clippy --all-targets --all-features -D warnings` en los tres; `cargo test` en Linux; `cargo test --no-run` en Windows y macOS; tipos del puente al día |
| Dependencias y licencias | Linux | `cargo deny check advisories licenses bans sources` |

## Lo que hay que preparar antes de `cargo`

* **La interfaz compilada** (`pnpm build`): el runtime la incrusta y el script de compilación de Tauri se detiene sin
  ella.
* **Los binarios incluidos** (`pnpm binarios`): `fpcalc` y ONNX Runtime son recursos del paquete.
* **La cadena de Rust** fijada en `rust-toolchain.toml`, la misma en local y en la CI.

## Trampas conocidas de las otras plataformas

* Código o importaciones que solo existen bajo `cfg(target_os = "linux")` o `cfg(unix)` quedan sin usar en las otras
  plataformas, y con `-D warnings` son un error. Se declaran con su condición (`check-plataforma`) y se comprueban
  compilando con las condiciones cambiadas.
* Archivos cuyo nombre solo difiere en mayúsculas rompen Windows y macOS.
* Las pruebas que dependen del tiempo se escriben con márgenes que aguantan un ejecutor lento; en local se reproduce
  con `taskset -c 0-3`.

## Avisos de dependencias

Un aviso nuevo de `cargo deny` se arregla actualizando la dependencia. Lo que no tiene arreglo se acepta en
`deny.toml` con su motivo y se documenta en [cadena de suministro](../audit/supply-chain.md).
