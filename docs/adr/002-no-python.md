# ADR-002: Ningún intérprete de Python en tiempo de ejecución

**Fecha:** 2026-10-29
**Estado:** Aceptado

## Contexto

La catalogación y la inteligencia musical suelen escribirse sobre Python (librosa, Essentia, PyTorch). Para un
reproductor de escritorio eso significa un segundo runtime de varios GB, un canal de IPC con contratos en varios
lenguajes y una segunda superficie de empaquetado. Además, Essentia no publica paquetes para Windows: con Python,
el análisis profundo no llegaría a esa plataforma.

Los hechos medidos que sostienen la decisión:

1. Demucs v4 (`htdemucs`) existe exportado a ONNX con paridad verificada: 166 MB en fp16, diferencia con PyTorch por
   debajo de 7·10⁻⁴ y la STFT dentro del grafo.
2. Los modelos de Essentia se usan solo como redes de inferencia; lo que hay que portar es el preprocesado de los
   espectrogramas. Su paridad en Rust se comprobó en 407 pistas (diferencia absoluta por clase menor de 0,01).
3. Los rasgos básicos (tempo, energía, brillo, croma) son DSP estándar: el tempo en Rust coincide con la referencia,
   o con su doble o su mitad, en más del 95 % de 2.397 pistas reales.

## Alternativas consideradas

1. **Python del sistema.** Rechazada por la [Constitución](../architecture/constitution.md), regla 1.
2. **Python embebido como agregado descargable.** Rechazada: no resuelve Essentia en Windows y añade un segundo
   runtime con varios GB.
3. **Todo en Rust, con la inferencia en ONNX Runtime a través de `ort`.** Aceptada.

## Decisión

* NB Sound no ejecuta Python. La catalogación, la inteligencia y la sincronización están escritas en Rust.
* La inferencia (karaoke, análisis, comprensión de los pedidos al DJ) corre en ONNX Runtime vía `ort`. El runtime de
  CPU viaja en el instalador; los modelos y la aceleración por GPU son agregados ([ADR-006](006-add-ons.md)).
* Los modelos se convierten a ONNX una sola vez, fuera de la aplicación, y se publican como agregados con su huella.

Dependencias que lo hacen posible: `lofty` (etiquetas), `unicode-normalization` y `regex` (comparar nombres),
`notify` (vigilar la carpeta de música), `trash` (descartar a la papelera del sistema), `zip` (copias de
seguridad), `rustfft` y `rubato` (DSP y remuestreo), y `libc` en Unix y `windows-sys` en Windows para bajar la
prioridad de procesador y de disco de los hilos de fondo.

## Consecuencias

- **Positivas:** un solo runtime y un solo lenguaje en el núcleo; el análisis llega a Windows; el karaoke pasa de
  varios GB de PyTorch a 166 MB.
- **Negativas:** toda la lógica está escrita en Rust y se mide contra referencias.
- **Riesgos y cómo se vigilan:** que un modelo nuevo exija operaciones que ONNX Runtime no tenga. `check-sin-python`
  rechaza cualquier archivo o llamada a un intérprete.
