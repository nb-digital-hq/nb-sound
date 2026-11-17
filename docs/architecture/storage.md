# Almacenamiento

> **Estado:** Implementado. Decisión: [ADR-004](../adr/004-data-ownership.md).

## Dónde vive cada cosa

| Qué | Linux | Windows | macOS |
|---|---|---|---|
| Datos | `$XDG_DATA_HOME/nb-sound` (`~/.local/share/nb-sound`) | `%APPDATA%\NB Sound` | `~/Library/Application Support/NB Sound` |
| Caché | `$XDG_CACHE_HOME/nb-sound` (`~/.cache/nb-sound`) | `%LOCALAPPDATA%\NB Sound\Cache` | `~/Library/Caches/NB Sound` |

Dentro de los datos:

| Ruta | Contenido | ¿Se puede rehacer? |
|---|---|---|
| `nb-sound.sqlite` | La base: biblioteca, playlists, historial, preferencias, sincronización, DJ, mashups | No |
| `nb-sound.sqlite.v<N>.respaldo` | La base antes de aplicar la migración `N` | — |
| `portadas/` | Portadas e imágenes de artista originales | No: algunas vienen de servicios que quizá ya no las den |
| `stems/` | Instrumentales del karaoke, en FLAC | Sí, con el agregado |
| `mashups/` | Los mashups creados | No: son de la persona |
| `agregados/` | Los modelos instalados | Sí, se descargan otra vez |
| `por-revisar/` | Los archivos que esperan una decisión | No |
| `movil/` | El certificado y la clave del servidor móvil | No: cambiarlos obliga a emparejar otra vez |
| `perfil/` | La foto de perfil | No |
| `registros/` | El registro local, redactado | — |
| `autorizaciones-launcher.sqlite3` | Las autorizaciones de NB Launcher ([ADR-017](../adr/017-launcher-persistent-authorization.md)) | No; queda fuera de la copia a propósito |

En la caché: las miniaturas de las portadas y los stems de los mashups (`stems-mashup/`), con su propio tope. Ajustes,
Almacenamiento, enseña lo que ocupa cada categoría y deja liberar lo que se puede rehacer.

## La base

* SQLite con `rusqlite` (`bundled`) y un pool `r2d2`; WAL, `busy_timeout` y claves foráneas activas.
* Toda escritura abre `BEGIN IMMEDIATE` (`db::transaccion_de_escritura`, `check-transacciones`).
* Las tablas son `STRICT`. Las búsquedas usan FTS5 (títulos, artistas, álbumes y letras).
* Las rutas de las canciones se guardan **relativas a la carpeta de la música**: mover la carpeta entera no rompe
  nada, y si al abrir ya no tiene la música, NB Sound la busca en la misma ruta bajo la carpeta personal actual y en
  la carpeta de música del sistema, y la adopta solo si encuentra la gran mayoría de las canciones.

## Migraciones

Embebidas en el binario desde `src-tauri/migrations/` y aplicadas en orden al abrir, cada una con un respaldo previo.
Una base de una versión más nueva no se abre con una más antigua.

| Versión | Contenido |
|---|---|
| V1 | Instalación: el identificador persistente de la instalación |
| V2 | Catálogo: portadas, artistas, álbumes, pistas, letras, importaciones, Por revisar y correcciones |
| V3 | Uso: playlists, historial, cola, estado del reproductor y preferencias |
| V4 | Análisis: rasgos básicos, análisis profundo y etiquetas de ambiente |
| V5 | Sincronización con el teléfono: versión, tumbas, dispositivos y transferencias de instrumentales |
| V6 | Karaoke y DJ: instrumentales, sesiones, sus pistas, eventos, preferencias y la sesión en curso |
| V7 | Búsqueda de texto completo (FTS5) |
| V8 | Sonoridad y el campo `verificada` |
| V9 | La cola de importación pendiente |
| V10 | Forma de onda |
| V11 | Lo que se completa en segundo plano |
| V12 | Revisión de recursos compartidos con NB Launcher |
| V13 | DJ Mashups: estructura de las pistas, mashups y sus canciones |

## La copia de seguridad

Un ZIP `.nbsound-backup` con la base (sacada con `VACUUM INTO`, una copia coherente sin bloquear), las portadas, la
foto de perfil y un `manifest.json` con la suma SHA-256 de cada entrada y la versión del esquema. No lleva el audio,
que es de la persona, ni las claves, que viven en el llavero, ni la identidad del servidor móvil.

Restaurar no reemplaza la base en caliente: la copia se valida entera (sumas, `integrity_check`, que su esquema no
sea más nuevo) y se aplica en el siguiente arranque, antes de abrir la base; la base anterior se aparta por si hay
que volver atrás. Las copias automáticas (cada día, semana o mes) guardan las tres últimas.
