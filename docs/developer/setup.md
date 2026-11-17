# Puesta en marcha

> **Estado:** Aplicable.

## Requisitos

* **Node** 22 o posterior y **pnpm** 10 o posterior (`.nvmrc`, `packageManager` en `package.json`).
* **Rust** con la versión fijada en `rust-toolchain.toml` (`rustup` la instala sola).
* **Linux:** las bibliotecas de desarrollo que pide Tauri y el audio:
  `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libssl-dev libasound2-dev`.
* **Windows:** Visual Studio Build Tools con C++ y WebView2 (viene con Windows 11).
* **macOS:** Xcode Command Line Tools. Solo Apple Silicon: ONNX Runtime no publica binario para Intel.

## Primera vez

```bash
pnpm install        # dependencias y ganchos de Git (.githooks)
pnpm binarios       # fpcalc y ONNX Runtime, con su SHA-256 comprobado
pnpm build          # la interfaz compilada, que el runtime incrusta
```

## Trabajar

```bash
pnpm tauri dev      # la aplicación en modo de desarrollo
pnpm verify         # lint, tipos, pruebas de la interfaz, guardianes y presupuesto de búsqueda
cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test          # también regenera los tipos del puente en src/lib/bindings
```

Esa secuencia es la **definición de terminado**: nada se da por hecho sin ella en verde.

## Datos de desarrollo

La aplicación usa los directorios estándar del sistema ([almacenamiento](../architecture/storage.md)). Para no tocar
una biblioteca real, se puede apuntar a otros con `XDG_DATA_HOME` y `XDG_CACHE_HOME` en Linux. Las pruebas que
necesitan música real la leen, en solo lectura, de la carpeta que diga `NS_CORPUS_MUSICA`, y se saltan solas si no
está; ninguna escribe en ella.
