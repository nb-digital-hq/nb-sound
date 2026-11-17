# ADR-008: Sistema de diseño

**Fecha:** 2026-10-29
**Estado:** Aceptado

## Contexto

NB Sound comparte con la suite la técnica de Material 3 (`m3-svelte`, tokens `--m3c-*` y Material Color Utilities)
y sus guardianes de interfaz, pero tiene su propia apariencia: un catálogo de temas con nombre, una tipografía a
elección y una marca que toma el color del tema. Un reproductor tiene además dos necesidades propias: el color de
cada portada y listas de miles de canciones que se desplazan de corrido.

## Alternativas consideradas

1. **Copiar la apariencia de otra aplicación de la suite.** Rechazada: la suite comparte método, no aspecto.
2. **Material 3 como técnica, con los temas, la tipografía y la marca de NB Sound.** Aceptada.

## Decisión

* **Material 3 como técnica**: `m3-svelte`, tokens `--m3c-*` y las mismas reglas de interfaz que la suite: sin
  colores literales, sin desenfoques en cromo permanente, sin `will-change`, nada que se anime en reposo, ningún
  `:hover` que desplace la caja e iconos SVG, nunca emojis.
* **63 temas completos** (oscuros, claros, de alto contraste y OLED), cada uno con sus colores traducidos a los
  tokens. El color por portada es una opción: con ella, la reproducción, los álbumes, los artistas y las playlists
  toman el color de su portada: Rust calcula el color dominante de cada portada una vez (`portadas.rs`, con `image`) y lo guarda, y la interfaz genera con él el esquema de Material con Material Color Utilities, que se carga a demanda.
* **Cinco tipografías** que viajan dentro de la aplicación con su licencia (Inter de fábrica, Fira Sans, IBM Plex
  Sans, Atkinson Hyperlegible y Source Sans 3), y seis tamaños de interfaz.
* **Una marca que toma el color del tema**: un SVG pintado como máscara, con el disco en `--m3c-primary` y la
  silueta en `--m3c-on-primary`. Los iconos de la aplicación se generan del mismo SVG con sus colores de origen,
  porque fuera de la ventana no hay tema.
* **Listas continuas**: una ventana virtual de filas de alto fijo, con los datos pedidos en bloques de 256 alineados
  a su tamaño. Medido en WebKitGTK: cero fotogramas largos con 100.000 filas a 12.000 px/s, con el mismo número de
  nodos que con 2.400. Mientras la lista va a más de dos filas por fotograma no se piden portadas; aparecen al frenar.
* La barra de progreso interpola la posición con `requestAnimationFrame` a partir de los eventos de estado del motor,
  solo mientras suena y la ventana es visible. El puente no lleva eventos de posición a 30 o 60 Hz.

## Consecuencias

- **Positivas:** la apariencia es de NB Sound y los guardianes son los de la suite.
- **Negativas:** cada tema tiene que completar los tokens que no define.
- **Riesgos y cómo se vigilan:** una portada con colores apagados daría un esquema gris; el sistema tonal de Material
  garantiza el contraste en cualquier caso. `check-ui` vigila las reglas de interfaz.
