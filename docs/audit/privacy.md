# Privacidad

> **Estado:** Implementado. Lo hacen cumplir `net.rs` con `check-network`, `telemetry.rs` con `check-registro` y
> `secrets.rs`.

## Principios

* **Sin cuenta.** Ninguna función pide iniciar sesión.
* **Sin telemetría.** No hay analítica, informes de fallos ni medición de uso, tampoco "anónimos" ni desactivables.
* **Local por defecto.** La biblioteca, las playlists, el historial, los instrumentales y los mashups viven en el
  equipo. Nada se sube a ningún servidor de NB Digital: no existe ninguno al que subirlo.
* **Cada salida, con permiso.** Un interruptor general y uno por servicio, cada uno diciendo qué recibe
  ([red](../architecture/network.md)).

## Qué datos trata y dónde

| Dato | Dónde | Sale del equipo |
|---|---|---|
| Archivos de música | La carpeta de la música | Nunca a internet; al teléfono emparejado si se sincroniza |
| Biblioteca, playlists, historial, favoritas | La base local | Al teléfono emparejado, en la red local |
| Nombre y foto de perfil | La base y `perfil/` | El nombre, al teléfono; la foto, nunca |
| Claves de servicios (AcoustID, IA) | El llavero del sistema | Solo al servicio al que pertenecen |
| Huella acústica | En memoria al identificar | A AcoustID, si está permitido |
| Títulos, artistas, álbumes | La base | A los servicios de datos, portadas y letras que estén permitidos |
| Registro de la aplicación | `registros/`, redactado | Nunca |

## El registro

`telemetry.rs` escribe archivos de registro locales y no envía nada. Ningún campo de registro puede contener una
ruta, un título, un artista, un álbum ni una clave: se redactan (la ruta conserva solo su extensión; el texto, solo
su longitud) o se miden. `check-registro` rechaza una línea de registro con un campo sin redactar, y cada línea se
vuelve a redactar al escribirse en el archivo.

## La red local

* **NB Sound Mobile**: solo los teléfonos emparejados con el código QR, con un `device_token` del que el equipo solo
  guarda la huella. Quitar un teléfono le cierra el acceso al momento.
* **Modo fiesta**: solo mientras está encendido; los invitados ven los títulos que buscan y lo que suena, nunca el
  audio, el historial ni rutas ([ADR-012](../adr/012-party-mode.md)).

## Lo que la persona controla

* Apagar todos los servicios en línea con un solo interruptor.
* Apagar cada servicio por separado.
* Ver y borrar cada teléfono emparejado y cada autorización de NB Launcher.
* Crear una copia de seguridad, restaurarla o borrar la aplicación y sus datos: desinstalar no borra la carpeta de
  la música.
