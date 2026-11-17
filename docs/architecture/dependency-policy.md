# Política de dependencias

> **Estado:** Aplicable. La licencia se comprueba sola (`cargo deny`, `generate-notices`); el coste y el
> mantenimiento se justifican en el cambio que añade la dependencia.

## Reglas

1. **¿Cabe en menos de cien líneas propias?** Entonces no se añade la dependencia.
2. **Licencia admitida.** Solo las de la lista de `deny.toml` (permisivas y de código abierto). Una dependencia con
   otra licencia no entra; `cargo deny check` lo rechaza en la CI.
3. **Mantenida.** Con actividad en los últimos doce meses y sin avisos de seguridad abiertos.
4. **Rust puro cuando se pueda.** Una dependencia en C o C++ añade una cadena de compilación por plataforma en la CI.
5. **Sin nada en el `PATH`.** Ninguna dependencia busca ejecutables del sistema (Constitución, regla 1).
6. **Versión fijada** en `Cargo.lock` y `pnpm-lock.yaml`; las actualizaciones se hacen a mano y se verifican en local
   con la suite completa ([cadena de suministro](../audit/supply-chain.md)).
7. **Cada dependencia directa nueva necesita un ADR o una entrada en uno existente** que diga para qué está.

## Dependencias directas del runtime

| Para qué | Dependencias | ADR |
|---|---|---|
| Ventana, sistema y actualizaciones | `tauri`, `tauri-plugin-dialog`, `tauri-plugin-updater`, `minisign-verify` | [001](../adr/001-tech-stack.md) |
| Errores, registro y serialización | `thiserror`, `tracing`, `tracing-subscriber`, `tracing-appender`, `serde`, `serde_json`, `ts-rs` | [001](../adr/001-tech-stack.md) |
| Datos | `rusqlite`, `r2d2`, `r2d2_sqlite`, `sha2`, `zip` | [004](../adr/004-data-ownership.md) |
| Audio | `symphonia`, `kira`, `ebur128`, `rubato`, `rustfft`, `flacenc` | [003](../adr/003-audio-engine.md), [018](../adr/018-dj-mashups.md) |
| Controles del sistema | `mpris-server` (Linux), `souvlaki` (Windows y macOS) | [003](../adr/003-audio-engine.md) |
| Catalogación | `lofty`, `unicode-normalization`, `regex`, `notify`, `trash`, `image` | [002](../adr/002-no-python.md) |
| Inferencia | `ort` | [002](../adr/002-no-python.md) |
| Red | `ureq` (solo en `net.rs`) | [009](../adr/009-settings-and-network.md) |
| Servidor del teléfono y Modo fiesta | `axum`, `tokio`, `tokio-rustls`, `futures-util`, `mdns-sd`, `rcgen`, `qrcode` | [005](../adr/005-mobile-contract.md), [012](../adr/012-party-mode.md) |
| Secretos | `keyring` | [009](../adr/009-settings-and-network.md) |
| Prioridad de los hilos de fondo | `libc` (Unix), `windows-sys` (Windows) | [002](../adr/002-no-python.md) |
| NB Launcher | `nb-protocol`, `nb-capability` (exportación sellada) | [013](../adr/013-launcher-shared-frequency.md) |

## Dependencias de la interfaz

| Para qué | Dependencias |
|---|---|
| Material 3 | `m3-svelte`, `@ktibow/material-color-utilities-nightly` |
| Puente | `@tauri-apps/api` |
| Construcción y calidad (desarrollo) | `@sveltejs/kit`, `svelte`, `vite`, `typescript`, `svelte-check`, `@biomejs/biome`, `prettier`, `vitest`, `@tauri-apps/cli` |

La lista completa, con versiones y licencias, está en [THIRD-PARTY-NOTICES.md](../../THIRD-PARTY-NOTICES.md), que
genera `scripts/generate-notices.mjs`.
