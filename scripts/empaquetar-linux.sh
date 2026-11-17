#!/usr/bin/env bash
# Empaqueta NB Sound para Linux dentro de un Ubuntu 22.04.
#
# Por qué en un contenedor: compilado en Arch, el binario exige la glibc de Arch
# (2.44) y no arranca en Ubuntu 24.04 (2.39) ni en nada más antiguo. Se compila en
# la distribución más antigua que se soporta, y lo compilado ahí corre en todo lo
# posterior. Lo mismo hace el CI de Tauri.
#
# Uso: scripts/empaquetar-linux.sh        (necesita podman)
# Con TAURI_SIGNING_PRIVATE_KEY y su contraseña en el entorno, firma cada paquete
# para el actualizador. Deja el .deb, el .rpm, la AppImage y el .tar.gz en
# src-tauri/target/ubuntu-22.04/release/bundle/.
set -euo pipefail

RAIZ="$(cd "$(dirname "$0")/.." && pwd)"
IMAGEN="docker.io/library/ubuntu:22.04"

podman run --rm \
  --memory 8g \
  -v "$RAIZ":/repo:Z \
  -v nb-sound-cargo:/root/.cargo \
  -v nb-sound-rustup:/root/.rustup \
  -v nb-sound-tauri:/root/.cache/tauri \
  -w /repo \
  -e CARGO_TARGET_DIR=/repo/src-tauri/target/ubuntu-22.04 \
  -e DEBIAN_FRONTEND=noninteractive \
  -e TAURI_SIGNING_PRIVATE_KEY \
  -e TAURI_SIGNING_PRIVATE_KEY_PASSWORD \
  "$IMAGEN" bash -euo pipefail -c '
    apt-get update -qq
    apt-get install -y -qq curl ca-certificates build-essential pkg-config file xz-utils \
      libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev libasound2-dev patchelf >/dev/null
    if ! command -v node >/dev/null; then
      curl -fsSL https://deb.nodesource.com/setup_22.x | bash - >/dev/null
      apt-get install -y -qq nodejs >/dev/null
    fi
    corepack enable >/dev/null 2>&1 || npm install -g pnpm >/dev/null
    [ -x /root/.cargo/bin/cargo ] || curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.100.0 >/dev/null
    export PATH=/root/.cargo/bin:$PATH
    pnpm install --frozen-lockfile >/dev/null
    node scripts/fetch-binarios.mjs
    # Las herramientas de la AppImage (linuxdeploy, AppRun y el runtime de tipo 2)
    # quedan en el volumen nb-sound-tauri tras la primera descarga. El runtime se
    # pasa por archivo si está ahí: appimagetool lo bajaría en cada compilación.
    if [ -f /root/.cache/tauri/runtime-x86_64 ]; then export LDAI_RUNTIME_FILE=/root/.cache/tauri/runtime-x86_64; fi
    pnpm tauri build --bundles deb,rpm,appimage --config "{\"bundle\":{\"createUpdaterArtifacts\":true}}"
    # El .tar.gz es el árbol usr/ del .deb, para las distribuciones sin dpkg ni rpm
    # y para NB Launcher. Va firmado como lo demás: la aplicación lo instala ella
    # misma y sin firma se negaría.
    version=$(node -p "require(\"./package.json\").version")
    bundle=src-tauri/target/ubuntu-22.04/release/bundle
    mkdir -p "$bundle/tar"
    archivo="$bundle/tar/NB.Sound_${version}_amd64.tar.gz"
    tar -czf "$archivo" --owner=0 --group=0 -C "$(echo "$bundle"/deb/*/data)" usr
    # La lista se toma entera antes de buscar: con pipefail, un grep -q que cierra
    # la tubería al primer acierto mataría a tar y daría un falso fallo.
    contenido=$(tar -ztf "$archivo")
    for f in "usr/bin/nb-sound" "usr/bin/fpcalc" "usr/lib/NB Sound/libonnxruntime.so" \
      "usr/lib/NB Sound/LICENSE" "usr/lib/NB Sound/THIRD-PARTY-NOTICES.md" "usr/share/applications/NB Sound.desktop"; do
      grep -qxF "$f" <<<"$contenido" || { echo "Falta $f en el .tar.gz"; exit 1; }
    done
    if [ -n "${TAURI_SIGNING_PRIVATE_KEY:-}" ]; then pnpm tauri signer sign --app-version "$version" "$archivo" >/dev/null; fi
  '
