# Motor de audio

> **Estado:** Implementado. Decisión: [ADR-003](../adr/003-audio-engine.md).

Todo el audio de NB Sound vive en el proceso de Rust. El webview nunca reproduce nada: pide órdenes al reproductor
por el puente y recibe su estado por eventos.

## Recorrido de una canción

1. **Decodificar.** Symphonia abre el archivo (MP3, FLAC, AAC-LC, ALAC, Vorbis, WAV) y entrega muestras en coma
   flotante; si la tasa no es la de salida, `rubato` remuestrea. La búsqueda descarta los marcos sobrantes para
   caer en la muestra pedida (error menor de 10 ms en MP3 con VBR).
2. **Sonoridad.** Si "Igualar el volumen entre canciones" está activo, la ganancia EBU R128 medida para la pista se
   aplica al decodificar. La medición (`medicion.rs`) corre en segundo plano, a prioridad baja, y deja también la
   forma de onda que dibuja la barra de progreso.
3. **La fuente de la cola.** Una fuente propia de Kira entrega la pista y la siguiente en el mismo bloque: no hay
   silencio entre canciones que no esté en los archivos. Con el karaoke activo, lee original e instrumental con el
   mismo índice y conmuta con una transición de 20 ms, sin clic y sin deriva.
4. **Ecualizador.** Diez campanas en la pista maestra con las 18 preselecciones de libVLC, corregidas para que la
   respuesta en cada centro sea la de la tabla. Mientras están planas no se calculan (`audio/banco.rs`).
5. **Limitador.** Kira recorta a ±1,0; el motor deja margen y un limitador suave de pico al final.
6. **Salida.** `cpal` sobre el dispositivo por defecto del sistema.

## La sesión de reproducción

`reproductor/` mantiene la cola, el aleatorio, la repetición, el volumen y la posición, y lo guarda en la base cada
vez que cambia y al cerrar: al volver a abrir, todo está como se dejó, en pausa en el segundo exacto. Una escucha
cuenta desde los 30 segundos, o la mitad en canciones de menos de un minuto.

El DJ Privado usa dos platos con sus propias bandas de ecualización para mezclar ([DJ](dj.md)); un mashup suena como
un WAV más; el fragmento de A ciegas suena sin decir qué es. En los tres casos la cola de la persona espera en pausa
y vuelve al terminar.

## Controles del sistema

* **Linux:** MPRIS (`mpris-server`), con reproducir, pausar, anterior, siguiente, aleatorio, repetición, volumen y
  metadatos con portada.
* **Windows:** System Media Transport Controls (`souvlaki`).
* **macOS:** `MPNowPlayingInfoCenter` (`souvlaki`).
* **Bandeja:** reproducir o pausar, anterior, siguiente, mostrar la ventana y salir, en el idioma de la interfaz.

Las teclas multimedia llegan por esos controles; los atajos de teclado son de la ventana.

## Lo que se mide

| Propiedad | Límite | Prueba |
|---|---|---|
| Coste en reproducción normal | Menos del 2 % de un núcleo (medido: 0,5 %) | `audio::pruebas`, en `--release` |
| Vacíos de búfer con todos los núcleos al 100 % | Cero en 10 minutos | `qa_audio` |
| Deriva del karaoke | 0 muestras tras 5 minutos | `audio::pruebas` |
| Ecualizador | ±0,5 dB de la tabla | `audio::pruebas` |
