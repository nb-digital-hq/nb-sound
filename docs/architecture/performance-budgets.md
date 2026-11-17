# Presupuestos de rendimiento

> **Estado:** Implementado. Cada presupuesto tiene una prueba o un guardián que lo mide.

| Qué | Presupuesto | Dónde se mide |
|---|---|---|
| JavaScript del arranque | 250 KB sin comprimir; cada `import()` sigue siendo un trozo aparte | `check-paquete` |
| Búsqueda en 100.000 pistas | Menos de 30 ms | `pnpm test:rendimiento` (aislada de los ensayos de DSP) |
| Desplazamiento de listas | Ningún fotograma largo (más de 25 ms) con 100.000 filas a 12.000 px/s en WebKitGTK | [Auditoría de rendimiento](../audit/performance.md) |
| Motor de audio en reproducción normal | Menos del 2 % de un núcleo | `audio::pruebas`, en `--release` |
| Vacíos de búfer con la CPU llena | Cero en 10 minutos | `qa_audio` |
| Análisis guardado por pista | Menos de 16 KB | Prueba del esquema |
| Memoria del karaoke | Menos de 3 GB de pico (medido: 1,4 GB con una canción de 4:45) | `karaoke`, prueba con el modelo real |
| Pedir una sesión al DJ | Menos de 3 s con la biblioteca entera | `dj::embeddings` |
| Escrituras concurrentes | Ocho escritores sin un solo `SQLITE_BUSY` | `db` |
| Nada se anima en reposo | Cero animaciones con la ventana quieta | `check-ui` |

## Por qué así

* **Sin comprimir**, porque es una aplicación de escritorio: los archivos se leen del disco y lo que cuesta es leerlos
  y compilarlos.
* **Las búsquedas y los órdenes ocurren en SQL**, nunca en la interfaz, y toda colección que cruza el puente está
  acotada ([contratos del puente](ipc-contracts.md)).
* **El trabajo de fondo usa la prioridad más baja** del sistema, para el procesador y para el disco: la música y la
  ventana no compiten con el karaoke, el análisis ni las copias.
