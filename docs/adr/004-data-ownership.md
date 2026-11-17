# ADR-004: Rust es dueño de los datos

**Fecha:** 2026-10-29
**Estado:** Aceptado

## Contexto

La base guarda la biblioteca, las playlists, el historial, el karaoke, el DJ, los mashups, los teléfonos
emparejados y el estado de la sincronización. Las listas virtualizadas, la cola, las listas inteligentes y las
estadísticas no pueden repartirse entre procesos, y lo derivado nunca debe pesar más que lo que describe: guardar
las salidas crudas de los modelos de análisis, fotograma a fotograma, ocupa más de 1 MB por pista.

## Alternativas consideradas

* **Un ORM o acceso desde el webview.** Rechazada: el dueño de los datos tiene que ser uno solo y las reglas de
  transacción no se pueden imponer desde la interfaz.
* **Guardar las salidas crudas del análisis.** Rechazada: multiplica la base por veinte sin que nadie las lea.

## Decisión

1. **Rust es el único dueño del esquema y del acceso**: `rusqlite` con `r2d2`, WAL, `busy_timeout`, claves foráneas
   activas y migraciones versionadas en `src-tauri/migrations/`. Antes de aplicar una migración se guarda un
   respaldo de la base.
2. **Toda transacción de escritura se abre con `BEGIN IMMEDIATE`.** Bajo WAL, una transacción diferida que lee y
   luego escribe recibe `SQLITE_BUSY` al instante, sin respetar `busy_timeout`. Lo vigila `check-transacciones`.
3. **Un esquema normalizado** y con índices medidos.
4. **El análisis se guarda resumido**: embeddings agregados como `BLOB` de `f32` y probabilidades por clase. Menos de
   16 KB de análisis por pista, vigilado por una prueba.
5. **Identificadores estables.** Los `id` de pista, álbum, artista y playlist no cambian nunca, porque NB Sound
   Mobile los guarda en su base local ([ADR-005](005-mobile-contract.md)). Lo mismo para `sync_version`, las tumbas
   y los `device_token`.
6. **Lo operativo no se guarda**: índices de manifiestos o colas de trabajos terminados se recalculan.
7. **Rutas relativas a la carpeta de la música**: mover la carpeta entera no rompe nada, y la aplicación la busca
   sola si cambia de sitio.

## Consecuencias

- **Positivas:** una base pequeña; escrituras sin `SQLITE_BUSY`; el teléfono recibe siempre los mismos
  identificadores.
- **Negativas:** cada cambio de esquema es una migración con su prueba.
- **Riesgos y cómo se vigilan:** una migración mal escrita. Las pruebas de esquema y el respaldo previo a cada
  migración la cubren; una base de una versión más nueva no se abre con una más antigua.
