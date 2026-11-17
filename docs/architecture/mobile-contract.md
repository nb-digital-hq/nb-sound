# Contrato con NB Sound Mobile

> **Estado:** Implementado. Decisión: [ADR-005](../adr/005-mobile-contract.md).

## La fuente de verdad

1. **Las respuestas doradas**, grabadas en `src-tauri/tests/contrato_movil/respuestas.json`. Mandan sobre cualquier
   documento.
2. La descripción del protocolo `1` que mantiene NB Sound Mobile, que dice lo mismo con palabras.

Si los dos discrepan, gana la grabación: es lo que el teléfono recibe de verdad.

## Cómo se comprueba

* El escenario (`escenario.json`) son datos inventados: dos artistas, dos álbumes, cuatro pistas (una con
  instrumental y letras, una borrada), dos playlists (una oculta), un perfil y el estado de un reproductor.
* `src-tauri/tests/contrato_movil.rs` siembra ese escenario en la base de NB Sound, arranca el servidor y hace las
  mismas peticiones que el teléfono, con un cliente que confía en el certificado por su huella. Reglas:
  * mismo código de estado;
  * mismas cabeceras de las cinco que se graban (`content-type`, `content-length`, `content-range`,
    `accept-ranges`, `x-nb-sound-hash`);
  * mismo cuerpo, con dos tolerancias: los marcadores `<...>` aceptan cualquier valor del mismo tipo, y un campo
    **añadido** se admite. Un campo que falta, que cambia de tipo o de valor, rompe la prueba;
  * en el WebSocket, la misma secuencia de mensajes;
  * el `content-length` tiene que salir exacto al quitar lo añadido: es lo que prueba que el JSON se escribe igual
    que la implementación de referencia (separadores, `\uXXXX` para lo que no es ASCII, reales en su forma más
    corta).

Una segunda prueba cubre lo que la grabación no recorre: el estado y la cola que el equipo empuja cuando cambian, y
que quitar un teléfono cierra su control remoto y deja su token sin valor.

## Detalles del protocolo que fija la grabación

* **Playlists:** además de `id`, `nombre`, `tipo`, `auto_key`, `sync_version` y `pista_ids`, se envían
  `descripcion`, `subtipo`, `origen`, `categoria`, `etiqueta`, `es_favoritos`, `es_anclada`, `num_pistas` y
  `cover_url`. El nombre de las playlists que crea NB Sound sigue al idioma de la interfaz; el teléfono las agrupa
  por `auto_key`, que no cambia.
* **Errores:** sin autenticación, `401 {"error": "no_autorizado"}`. Un id de pista no numérico da
  `404 {"error": "no_encontrado"}`; un tipo de recurso desconocido, también.
* **Acuse de las órdenes:** `{"ok": true, "encolado": true}`, incluso para una acción desconocida; el mensaje de error
  solo aparece con JSON inválido.

## Campos que añade NB Sound

| Campo | Dónde | Para qué |
|---|---|---|
| `verificada` | cada pista del manifiesto | `false` si se importó tal como estaba desde Por revisar. Cambiarla sube la versión de la pista |

## Diferencias deliberadas con la grabación

| Qué | Grabación | NB Sound | Por qué |
|---|---|---|---|
| `cover_url` de las playlists | Solo con un mosaico ya generado | Siempre que alguna pista tiene portada; el mosaico se compone al pedirlo | El teléfono siempre tiene imagen |
| `indice` del mensaje `cola` | `-1` cuando el índice era 0 | El índice | Era un defecto |

Y en el mismo sentido (no guardar ni enviar lo que no sirve): `perfil.foto` va vacío (la ruta del PC no le sirve al
teléfono); una escucha de una pista que el equipo no tiene no se apunta; `queue` responde con la cola solo a quien la
pidió; `stop` pausa; quitar un teléfono cierra al momento su control remoto; el `device_token` no se guarda, solo su
SHA-256.

## La huella del audio

`hash_sha256` y `x-nb-sound-hash` no son el SHA-256 del archivo entero: son el de sus primeros 512 KB y sus últimos
512 KB, la regla con la que el teléfono comprueba la descarga. Hasta 1 MB coincide con el SHA-256 del archivo.

## El instrumental

`GET /api/v1/track/{id}/stems` sirve el instrumental del karaoke en FLAC (`audio/flac`), con `Range`. NB Sound
Mobile lo reproduce tal cual.
