# Normas de código

> **Estado:** Aplicable. Lo que se puede comprobar lo comprueba un guardián o el lint; el resto se revisa.

## Idioma y estilo

* **Español** en los identificadores, los comentarios, las pruebas y la documentación de `docs/`. Los archivos de
  comunidad de la raíz, en inglés.
* **Sin emojis** en el código, la documentación ni los commits (`check-docs`).
* Formato automático: Biome y Prettier en la interfaz, `rustfmt` en Rust. `pnpm lint:fix` y `cargo fmt --all`.

## Comentarios

* Explican **por qué**, no qué. Cuando una decisión no es obvia, dicen qué se descartó y por qué.
* Un comentario que cuenta lo que hace la línea siguiente sobra.
* Una medición que justifica una constante se cita con su cifra ("1,4 GB de pico con una canción de 4:45").

## Errores

* Ningún error cruza el puente como cadena: `NsError` con un código estable ([contratos del puente](../architecture/ipc-contracts.md)).
* Ningún `unwrap` ni `expect` fuera de las pruebas y de invariantes demostradas; Clippy lo vigila con `-D warnings`.
* Un fallo de un servicio de fondo se registra y se reintenta; no tira la aplicación (Constitución, regla 6).

## Registro

* `tracing` con campos estructurados. Ningún campo lleva una ruta, un título, un artista, un álbum ni una clave:
  `telemetry::redact_path`, `redact_text` o una medida (`check-registro`).

## Datos

* Toda escritura abre `BEGIN IMMEDIATE` (`db::transaccion_de_escritura`, `check-transacciones`).
* Las búsquedas y los órdenes ocurren en SQL. Toda colección que cruza el puente está acotada (`check-listas`).
* Un cambio de esquema es una migración nueva con su prueba; una migración publicada no se edita.

## Interfaz

* Solo `#lib/core/ipc` llama al puente.
* Ningún texto en el marcado: diccionarios en tres idiomas ([i18n](../architecture/i18n.md)).
* Las reglas de diseño de [convenciones de interfaz](../design/ui-conventions.md).

## Plataformas

* El código atado a una plataforma se declara con `#[cfg(...)]` (`check-plataforma`). Lo que solo existe bajo una
  condición no puede quedar sin usar en las otras: con `-D warnings` sería un error en la CI de esa plataforma.

## Pruebas

* Se nombran por la propiedad que verifican, no por la función que llaman
  (`una_cola_suena_entera_y_cada_pista_queda_en_el_historial`).
* Cada lógica nueva lleva su prueba; cada defecto corregido, una de regresión.
* Ninguna prueba escribe fuera de un directorio temporal ni depende de la red.
