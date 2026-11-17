# Rendimiento

> **Estado:** Implementado. Los presupuestos y la prueba que mide cada uno están en
> [presupuestos de rendimiento](../architecture/performance-budgets.md).

## Equipo de medición

Linux x86-64, WebKitGTK 2.54, CPU sin GPU para la inferencia, compilación optimizada salvo que se diga lo contrario.

## Listas largas en WebKitGTK

Una lista con la forma de la biblioteca (portada, título, artista, álbum y duración, filas de 56 px) alimentada con
datos y portadas distintos por fila, desplazada 8 s a velocidad constante. Un fotograma largo pasa de 25 ms.

| Filas | Velocidad | Mediana | p95 | Fotogramas largos | Nodos |
|---|---|---|---|---|---|
| 2.397 | 12.000 px/s | 17 ms | 18 ms | 0 de 481 | 268 |
| 100.000 | 3.000 px/s | 17 ms | 18 ms | 0 de 480 | 268 |
| 100.000 | 12.000 px/s | 17 ms | 18 ms | 0 de 481 | 268 |
| 100.000 | 30.000 px/s | 17 ms | 18 ms | 1 de 480 | 268 |

El número de nodos no depende del total. Lo que costaba fotogramas eran las portadas nuevas a alta velocidad: en
WebKitGTK cada imagen nueva pasa por el proceso de red. Mientras la lista va a más de dos filas por fotograma no se
piden portadas y aparecen al frenar 120 ms.

## Motor de audio

* Reproducción normal: el motor (decodificar, mezclar, efectos) usa el 0,5 % de un núcleo; las campanas del
  ecualizador planas no se calculan. El hilo de salida ronda el 4 %, que es ALSA y PipeWire, fuera del motor.
* Cero vacíos de búfer en 10 minutos con todos los núcleos ocupados.

## Karaoke

* `htdemucs` en CPU separa a unas 0,4 veces el tiempo real. Más hilos no aceleran (2 y 4 hilos: 0,38× y 0,37×).
* Pico de memoria: 1,4 GB con una canción de 4:45, sin optimizar el grafo de ONNX Runtime (con el grafo optimizado
  llegaba a 6,7 GB y daba el mismo instrumental, un 30 % más rápido).

## DJ y mashups

* Pedir una sesión: los vectores de texto de cada canción se calculan una vez y se guardan; sin ellos, 73 s con una
  biblioteca de unas 2.400 canciones, con ellos por debajo de 3 s.
* Mashups: las mediciones de análisis, composición, separación y render están en
  [la arquitectura de los mashups](../architecture/mashups.md), sección 9.

## Búsqueda

Con 100.000 pistas simuladas (títulos, duraciones, portadas y formatos distintos), una búsqueda responde en menos de
30 ms (`pnpm test:rendimiento`).
