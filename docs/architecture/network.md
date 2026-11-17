# Red

> **Estado:** Implementado. Decisión: [ADR-009](../adr/009-settings-and-network.md).

## La puerta única

Toda salida a internet pasa por `src-tauri/src/net.rs`. Cada petición declara su servicio; la puerta comprueba:

1. el **interruptor general** "Usar servicios en línea" (apagado: no sale nada, sea cual sea el ajuste de cada
   servicio);
2. el **consentimiento del servicio**, uno por servicio en Ajustes, Privacidad;
3. que el **anfitrión** sea uno de los permitidos para ese servicio.

`check-network` rechaza cualquier cliente HTTP (`ureq`, `reqwest`, sockets) fuera de `net.rs`. El agente identifica
a la aplicación por su nombre, su versión y la dirección del repositorio, sin ningún dato de la persona.

## Servicios

| Servicio | Anfitriones | Qué recibe | Para qué |
|---|---|---|---|
| AcoustID | `api.acoustid.org` | La huella acústica y la duración, nunca el audio | Reconocer una canción |
| MusicBrainz | `musicbrainz.org` | Título, artista y álbum, o el identificador de AcoustID | Completar los datos |
| Cover Art Archive | `coverartarchive.org`, `*.archive.org` | El identificador del álbum | La portada |
| iTunes | `itunes.apple.com`, `*.mzstatic.com` | Título y artista | Confirmar la canción y portadas |
| LRCLIB | `lrclib.net` | Título, artista, álbum y duración | Letra sincronizada |
| lyrics.ovh | `api.lyrics.ovh` | Título y artista | Letra, si LRCLIB no la tiene |
| Deezer | `api.deezer.com`, `*.dzcdn.net` | Nombre del artista o del álbum | Imágenes |
| Wikipedia | `*.wikipedia.org`, `*.wikimedia.org` | Nombre del artista | Su foto |
| TheAudioDB | `www.theaudiodb.com`, `r2.theaudiodb.com` | Nombre del artista y del álbum | Portadas e imágenes de artista |
| Proveedor de IA | `api.anthropic.com`, `api.openai.com` | Los datos de la canción y de los candidatos | Desempatar, si se activa |
| Agregados | `github.com`, `*.githubusercontent.com` | La petición del archivo | Descargar un agregado al pulsar Instalar |
| Actualizaciones | `github.com`, `*.githubusercontent.com` | La versión instalada y el sistema | Buscar una versión nueva al pulsarlo |

Pulsar Instalar en un agregado o "Buscar actualizaciones" cuenta como consentimiento de ese servicio, siempre que el
interruptor general esté encendido.

## Lo que no sale nunca

* Telemetría, analítica, informes de fallos o uso: no existen.
* El audio de ninguna canción.
* Rutas, nombres de archivo, el nombre de la persona o su historial.

## Red local

El servidor para NB Sound Mobile (HTTPS y WSS con certificado propio, anunciado por mDNS) y el Modo fiesta (HTTP en
un puerto propio, solo mientras está encendido) escuchan en la red local; no salen a internet. Ajustes, Permisos,
comprueba si el cortafuegos del sistema los deja pasar y pide permiso con la ventana del sistema si hace falta.
