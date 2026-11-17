# ADR-006: Agregados que se instalan con un clic

**Fecha:** 2026-10-29
**Estado:** Aceptado

## Contexto

Una aplicación que usa el entorno al que llega acaba con una pantalla de "estado del sistema" que enseña versiones
de bibliotecas y comandos de reparación para el gestor de paquetes, y con un instalador que busca un intérprete y
ejecuta otro instalador. NB Sound no quiere nada de eso. A la vez, los modelos del karaoke y del análisis pesan
cientos de MB y no todo el mundo los quiere.

## Alternativas consideradas

1. **Todo dentro del instalador.** Rechazada: multiplica el tamaño de la descarga para todos por funciones que
   muchos no usan.
2. **Descargas desde la aplicación, verificadas y gestionadas por ella.** Aceptada.

## Decisión

**Qué viaja dentro del instalador** (nunca es un agregado): el motor de audio, la huella acústica (`fpcalc`, que trae
su propio decodificador), ONNX Runtime para CPU, el servidor para el teléfono, las fuentes y los iconos. Con eso la
aplicación reproduce, importa, identifica, etiqueta, descarga portadas y letras, y sincroniza con el teléfono.

`ffmpeg` no viaja: la compilación completa pesa unos 165 MB por ejecutable y lo que haría (convertir lo que no es
MP3) sería un defecto, porque un FLAC convertido pierde calidad para siempre. **La importación nunca transcodifica**:
cada archivo se queda en su formato, y lo vigila una prueba.

**Qué es un agregado** (grande y opcional):

| Agregado | Contenido | Habilita |
|---|---|---|
| Karaoke | `htdemucs` en ONNX (fp16) | Cantar cualquier canción, instrumentales para el teléfono, capas de voz e instrumentos en el DJ y en los mashups |
| Análisis musical | MusiCNN y sus cabezas de clasificación en ONNX | Ánimo, bailabilidad, estilo; listas inteligentes por ánimo, el DJ y las búsquedas por intención |
| Pedidos al DJ en tus palabras | `all-MiniLM-L6-v2` en ONNX | Entender lo que se pide al DJ con frases libres |
| Aceleración por GPU | ONNX Runtime con CUDA y sus bibliotecas (solo Linux con NVIDIA) | Karaoke y análisis varias veces más rápidos |

**Cómo se comporta:**

* Viven en **Ajustes, Agregados**: una tarjeta por agregado con qué hace, cuánto ocupa y un solo botón. Mientras se
  descarga se ve el progreso y se puede seguir usando la aplicación. Lo que no existe para el equipo no se muestra.
* Se ofrecen **en el sitio donde hacen falta**: pulsar el micrófono sin el agregado de karaoke ofrece instalarlo ahí.
* El catálogo se compila dentro del binario: URL, tamaño y SHA-256 fijados. Se descarga a un temporal, se verifica
  la huella antes de moverlo a su sitio y la instalación es atómica. Una descarga interrumpida se reanuda.
* Los archivos se publican en la publicación 1.0.0 del repositorio de NB Sound en GitHub.
* Se guardan en el directorio de datos de la aplicación, nunca en el sistema; quitar uno libera su espacio al
  momento.
* La descarga pasa por la puerta de red con su consentimiento ([ADR-009](009-settings-and-network.md)).
* Si el equipo tiene una NVIDIA con su controlador, Ajustes recomienda la aceleración con su motivo, sin instalarla.

## Consecuencias

- **Positivas:** instalar el karaoke es un clic de 146 MB; nada pide una terminal.
- **Negativas:** los archivos tienen que estar en un sitio público con enlace estable.
- **Riesgos y cómo se vigilan:** las licencias de los modelos ([licencias de los modelos](../legal/model-licenses.md)).
  Una prueba comprueba que cada URL del catálogo apunta a las publicaciones del repositorio.
