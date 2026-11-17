# Licencias de los modelos

> **Estado:** Aplicable.

Los modelos no viajan en el instalador: son [agregados](../adr/006-add-ons.md) que la persona instala con un clic, y
cada uno conserva su licencia. Instalar un agregado es aceptar las condiciones de los componentes que lleva.

| Agregado | Componente | Licencia | Origen |
|---|---|---|---|
| Karaoke | `htdemucs` (Meta), exportado a ONNX en fp16 | MIT | `facebookresearch/demucs` |
| Análisis musical | MusiCNN y sus cabezas de clasificación (Music Technology Group, Universitat Pompeu Fabra), exportados a ONNX | Creative Commons BY-NC, **uso no comercial** | Modelos de Essentia |
| Pedidos al DJ en tus palabras | `all-MiniLM-L6-v2` y su tokenizador, exportados a ONNX | Apache-2.0 | `sentence-transformers` |
| Aceleración por GPU (Linux, NVIDIA) | ONNX Runtime con CUDA | MIT | Microsoft |
| Aceleración por GPU (Linux, NVIDIA) | CUDA Runtime, cuBLAS, cuFFT, cuRAND y cuDNN | Licencias de NVIDIA, que permiten redistribuir estas bibliotecas | NVIDIA |

## El agregado de análisis musical

Los modelos del MTG se licencian para uso no comercial. Cambiar su formato a ONNX es una modificación técnica que la
propia licencia permite y no produce una obra derivada. El agregado se distribuye con su atribución y su licencia, y
quien lo instala acepta usarlo sin fines comerciales; que NB Sound sea gratuito no basta para que un uso concreto lo
sea. Sin este agregado la aplicación funciona entera: el DJ y las listas usan los rasgos básicos de cada canción.

Ninguna oferta comercial de NB Digital incluye estos modelos.
