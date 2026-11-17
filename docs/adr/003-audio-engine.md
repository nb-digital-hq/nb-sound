# ADR-003: Motor de audio embebido en Rust

**Fecha:** 2026-10-29
**Estado:** Aceptado

## Contexto

El reproductor tiene que cubrir: MP3, FLAC, AAC, ALAC, Vorbis y WAV; reproducción sin cortes entre pistas; un
ecualizador de 10 bandas con 18 preselecciones; el karaoke (original e instrumental sincronizados a la muestra, con
conmutación sin clic); el DJ (dos platos con técnicas de transición), y los controles multimedia del sistema.
Depender de un reproductor externo (VLC, GStreamer) obliga a instalarlo o a empaquetar sus complementos, y
sincronizar dos reproductores por sondeo no alcanza la precisión del karaoke.

## Alternativas consideradas

1. **Rodio.** Rechazada: no sincroniza pistas a nivel de muestra ni programa eventos en un reloj.
2. **GStreamer.** Rechazada: vuelve a depender de complementos del sistema, lo contrario de la regla 1.
3. **Motor propio sobre `cpal` desde cero.** Viable pero caro.
4. **Symphonia + Kira sobre `cpal`, detrás de una interfaz propia.** Aceptada.

## Decisión

* **Decodificación:** Symphonia con `mp3`, `flac`, `aac`, `alac`, `vorbis`, `pcm`, `wav` e `isomp4`. Lo que no
  decodifica (HE-AAC, Opus, WMA, APE) va a Por revisar con el motivo "formato no compatible".
* **Mezcla:** Kira, con pistas de mezcla y efectos. **La cola y el karaoke son una fuente propia**: el reloj de Kira
  no arranca sonidos en el instante exacto (medido: se desvía a los 8,4 s), así que la fuente entrega una pista y la
  siguiente en el mismo bloque y lee original e instrumental con el mismo índice.
* **Margen y limitador:** Kira recorta a ±1,0 y algunos MP3 llegan a 1,104 tras decodificar. El motor deja margen y
  pone un limitador suave de pico al final de la pista maestra.
* **Ecualizador corregido:** las ganancias de los diez filtros se calculan para que la respuesta en cada centro sea
  la de la tabla de preselecciones; aplicada tal cual, la tabla se desvía hasta 7,2 dB. Las campanas planas no se
  calculan: el motor cuesta el 0,5 % de un núcleo en reproducción normal (`audio/banco.rs`).
* **Controles del sistema:** `mpris-server` en Linux y `souvlaki` en Windows (SMTC) y macOS
  (`MPNowPlayingInfoCenter`).
* **Sonoridad:** `ebur128` mide cada pista en segundo plano y la ganancia se aplica al decodificar.
* Todo el audio vive en el proceso de Tauri. **El webview nunca reproduce audio**, lo que además evita el defecto
  de WebKitGTK con medios servidos desde el esquema propio.

## Criterios que el motor cumple, con su prueba

| Propiedad | Criterio |
|---|---|
| Sin cortes | Al renderizar fuera de línea un álbum, entre pistas no aparece silencio que no esté en los archivos |
| Búsqueda | Error menor de 10 ms en MP3 con VBR |
| Karaoke | Deriva de 0 muestras tras 5 minutos; conmutar no produce clic (transición de 20 ms o más) |
| Ecualizador | Respuesta de las 18 preselecciones dentro de ±0,5 dB de su tabla |
| Robustez | Cero vacíos de búfer en 10 minutos con todos los núcleos al 100 % |
| Coste | Por debajo del 2 % de un núcleo en reproducción normal |

## Consecuencias

- **Positivas:** ni VLC ni sondeo; los controles del sistema no se escriben a mano; el karaoke y el DJ son exactos.
- **Negativas:** la cola sin cortes se programa en la fuente propia, no viene hecha.
- **Riesgos y cómo se vigilan:** si Kira no diera la precisión necesaria, se sustituye detrás de la interfaz del
  motor sin tocar nada más. Las pruebas del motor renderizan fuera de línea y comprueban muestra a muestra.
