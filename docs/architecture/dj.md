# DJ Privado

> **Estado:** Implementado.

Una sesión mezclada a partir de una frase ("algo tranquilo para trabajar de noche, sin reguetón"), con las
canciones de la biblioteca y transiciones de DJ.

## De la frase a la intención (`dj/intencion.rs`, `dj/ontologia.rs`)

* La frase se normaliza (minúsculas, sin diacríticos) y se buscan **conceptos** de una ontología perceptual: géneros,
  ánimos, energía, voces, momentos y estilos de transición. Cada concepto tiene alias en español, inglés y portugués
  y se traduce a votos sobre 24 ejes (energía, bailabilidad, melancolía, presencia vocal…).
* Las **negaciones** ("sin", "no", "without", "sem") excluyen lo que viene detrás hasta un conector ("pero", "but",
  "com").
* Con el agregado "Pedidos al DJ en tus palabras", la frase se lleva además a un vector con `all-MiniLM-L6-v2` y se
  compara con el de cada canción; esos vectores se calculan una vez y se guardan junto al agregado.
* La curva de energía de la sesión (subida progresiva, olas, bajada, pico) sale de la frase o de la duración.

## De la intención a la sesión (`dj/planificador.rs`, `dj/constructor.rs`)

Cada canción candidata se lleva a los ejes del DJ (con el análisis musical si está, o con los rasgos básicos si no),
se puntúa contra la intención y la fase de la curva en que caería, y se encadena teniendo en cuenta la compatibilidad
armónica (rueda de Camelot), el tempo y la variedad de artistas. Con la misma semilla, la sesión es la misma: el azar
es un MT19937 propio (`dj/azar.rs`).

## La mezcla (`dj/mezcla.rs`, `dj/ejecutor.rs`)

Dos platos en el motor de audio, cada uno con sus bandas. Cinco técnicas, elegidas por la compatibilidad de las dos
canciones: **corte en el beat**, **fundido largo**, **ecualización** (quitar graves de la saliente), **barrido de
filtros** y **capas** (con los stems del karaoke si están). Los fundidos son de igual potencia y sin clics ni huecos;
las mezclas con filtros bajan algo de sonoridad a mitad de la transición, y compensarlo empujaría la suma contra el
limitador.

## Lo que queda

La sesión se guarda con sus canciones y sus eventos; "Me gusta" y "No me gusta" ajustan las preferencias del DJ;
"Guardar como playlist" la convierte en una playlist normal. Escuchar dentro de una sesión cuenta en el historial.
