#!/usr/bin/env bash
# Arma el agregado de aceleración para Linux con NVIDIA :
# ONNX Runtime con el proveedor CUDA y las bibliotecas de CUDA 12 y cuDNN 9 que
# necesita, sacadas de las ruedas oficiales de NVIDIA en PyPI (solo se descomprimen;
# no se usa Python). Con el controlador de NVIDIA instalado, no hace falta nada más.
#
# Uso: scripts/agregados/aceleracion-linux.sh <carpeta de trabajo>
# Deja <carpeta>/aceleracion-linux-1.zip y su SHA-256 y tamaño en la salida, que es
# lo que se copia al catálogo de `agregados.rs`.
set -euo pipefail

TRABAJO=${1:?carpeta de trabajo}
ORT_VERSION=1.31.1
# El paquete oficial de NuGet: el repositorio de GitHub dejó de servir las releases.
ORT="microsoft.ml.onnxruntime.gpu.linux.$ORT_VERSION"
RUEDAS=(nvidia-cuda-runtime-cu12 nvidia-cublas-cu12 nvidia-cufft-cu12 nvidia-curand-cu12 nvidia-cudnn-cu12)

mkdir -p "$TRABAJO/descargas" "$TRABAJO/contenido"
cd "$TRABAJO"
rm -rf contenido/*

if [[ ! -f "descargas/$ORT.nupkg" ]]; then
  curl -fL --retry 3 -o "descargas/$ORT.nupkg" \
    "https://api.nuget.org/v3-flatcontainer/microsoft.ml.onnxruntime.gpu.linux/$ORT_VERSION/$ORT.nupkg"
fi
unzip -oq "descargas/$ORT.nupkg" 'runtimes/linux-x64/native/*' -d "descargas/$ORT"
# Solo las bibliotecas, con el nombre que carga NB Sound para la principal.
nativo="descargas/$ORT/runtimes/linux-x64/native"
cp -L "$nativo/libonnxruntime.so" contenido/libonnxruntime.so
cp -L "$nativo/libonnxruntime_providers_shared.so" contenido/
cp -L "$nativo/libonnxruntime_providers_cuda.so" contenido/

for rueda in "${RUEDAS[@]}"; do
  url=$(curl -fs "https://pypi.org/pypi/$rueda/json" |
    jq -r '[.urls[] | select(.filename | test("manylinux.*x86_64"))][0].url')
  archivo="descargas/$(basename "$url")"
  [[ -f $archivo ]] || curl -fL --retry 3 -o "$archivo" "$url"
  unzip -oq "$archivo" 'nvidia/*/lib/*.so*' -d descargas/ruedas
done
find descargas/ruedas -name '*.so*' -type f -exec cp {} contenido/ \;

cat >contenido/LEEME.txt <<'TXT'
NB Sound - agregado de aceleración (Linux, NVIDIA)
ONNX Runtime (MIT) con el proveedor CUDA, y CUDA Runtime, cuBLAS, cuFFT, cuRAND y cuDNN
de NVIDIA, redistribuidos según el acuerdo de licencia de CUDA (EULA de NVIDIA,
"Attachment A", bibliotecas redistribuibles) y el de cuDNN. Necesita el controlador
de NVIDIA del sistema.
TXT

rm -f aceleracion-linux-1.zip
(cd contenido && zip -q -9 -r ../aceleracion-linux-1.zip .)
du -sh contenido
printf 'TAMANO %s\nSHA256 %s\n' "$(stat -c %s aceleracion-linux-1.zip)" \
  "$(sha256sum aceleracion-linux-1.zip | cut -d' ' -f1)"
