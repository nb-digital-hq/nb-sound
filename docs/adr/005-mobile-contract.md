# ADR-005: El contrato con NB Sound Mobile queda congelado

**Fecha:** 2026-10-29
**Estado:** Aceptado

## Contexto

NB Sound Mobile (Flutter, Android) sincroniza la biblioteca y controla el escritorio. Su protocolo, versión `1`,
está fijado por lo que el teléfono ya implementa:

* HTTPS y WSS con un certificado autofirmado persistente; el teléfono fija su huella SHA-256 en el primer
  emparejamiento.
* Anuncio mDNS `_nbsound._tcp.local.` y un código QR con un token efímero de un solo uso (300 s).
* `POST /api/v1/pair` emite un `device_token` persistente; el resto usa `Authorization: Bearer`.
* Rutas: `ping`, `pair`, `manifest` (delta paginado por `sync_version`, con tumbas), `seleccion`,
  `track/{id}/audio` y `track/{id}/stems` con `Range`, `track/{id}/lyrics`, `asset/{tipo}/{id}`, `history`.
* WebSocket `/api/v1/control`: estado del reproductor, órdenes y la cola.
* El cliente ignora campos desconocidos, así que solo se admiten cambios aditivos.

## Alternativas consideradas

1. **Diseñar un protocolo nuevo y cambiar la app móvil.** Rechazada: el teléfono funciona y cambiarlo no aporta nada
   a quien lo usa.
2. **Servidor compatible byte a byte, verificado por respuestas grabadas.** Aceptada.

## Decisión

1. El servidor está escrito en Rust (`axum`, `rustls` con `tokio-rustls`, `mdns-sd`), **compatible byte a byte**
   con el protocolo `1`: mismas rutas, códigos, nombres de campo, paginación y reglas de autoridad de la mezcla.
2. **Una identidad que no cambia**: el certificado (ECDSA P-256 creado con `rcgen`, sin caducidad) y su clave se
   crean una vez y se guardan, así que los teléfonos emparejados se reconectan sin volver a escanear el código.
3. **Respuestas doradas**: cada ruta y cada mensaje del WebSocket tiene su respuesta grabada. Un arnés las reproduce
   contra el servidor de NB Sound en cada `cargo test`; una diferencia que no sea un campo añadido rompe la prueba.
4. Un campo nuevo que necesite NB Sound es aditivo y se documenta en el [contrato móvil](../architecture/mobile-contract.md).
5. Si una mejora del escritorio exigiera romper el protocolo, se escribe un ADR que lo justifique y se sube a la
   versión `2`, manteniendo la `1` mientras exista un cliente que la use.

## Consecuencias

- **Positivas:** el teléfono funciona sin cambios.
- **Negativas:** el protocolo mezcla nombres de campo en español y en inglés, que no se pueden cambiar.
- **Riesgos y cómo se vigilan:** comportamiento implícito (cabeceras, formato de errores, cierre del WebSocket) que
  no esté escrito; las respuestas doradas lo capturan porque se grabaron de un servidor real.
