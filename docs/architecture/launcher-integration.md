# Integración con NB Launcher

> **Estado:** Implementado. Verificado en Linux; en Windows y macOS se compila y se prueba en la CI sin ejecutarse
> en un equipo real. Decisiones: [ADR-013](../adr/013-launcher-shared-frequency.md) a
> [ADR-017](../adr/017-launcher-persistent-authorization.md).

NB Launcher es el instalador y centro de mando de la suite NB Digital. NB Sound funciona igual sin él; con él,
comparte una capacidad concreta y se deja instalar y actualizar.

## Lo que comparte: la frecuencia de las copias

NB Launcher puede leer y cambiar cada cuánto NB Sound hace copias de seguridad, y recibir los cambios. NB Sound
sigue siendo el dueño: valida, guarda la preferencia y su revisión en una sola transacción y resuelve los conflictos
diciendo que el valor cambió desde otro control. Ninguna otra preferencia, archivo, reproducción ni restauración es
accesible por ese enlace.

## El enlace

* **Iniciado por NB Launcher.** Las credenciales llegan por un canal privado heredado del supervisor, nunca por
  argumentos, entorno ni disco. El enlace caduca a los 15 minutos y se renueva por ese mismo canal
  ([ADR-014](../adr/014-launcher-link-renewal.md)).
* **NB Sound abierto por separado (Linux).** NB Sound publica un anuncio local con su aplicación, su PID, su protocolo
  y su punto de conexión, sin rutas ni datos de la biblioteca. NB Launcher demuestra su identidad Ed25519 con un
  desafío de un solo uso y NB Sound pide consentimiento en su propia ventana.
* **Autorizaciones recordadas** en `autorizaciones-launcher.sqlite3`, fuera de la biblioteca y de la copia de
  seguridad. Se revocan en Ajustes, Privacidad; revocar cierra al momento las sesiones de esa concesión.
* **Una sola instancia (Linux).** Una segunda apertura activa la ventana existente en lugar de abrir otra autoridad.

## La instalación administrada (Linux)

Una copia instalada por NB Launcher lleva la marca `.nb-launcher-managed` junto al ejecutable y mantiene un bloqueo
compartido mientras corre; el actualizador propio de NB Sound se aparta y Ajustes dice que esa instalación se
actualiza desde NB Launcher ([ADR-015](../adr/015-launcher-managed-install.md)).

## El catálogo firmado

Cada publicación lleva `launcher-catalog.json` y su firma, con los siete artefactos de la versión, su tamaño, su huella
y su caducidad ([ADR-016](../adr/016-launcher-catalog.md)). Lo produce `src-tauri/tools/publicar_catalogo.rs` y lo
firma la CI con la clave editorial ([publicación](../developer/releases.md)).

## Bibliotecas compartidas

`nb-protocol` y `nb-capability` (Apache-2.0) llegan como una exportación sellada en `src-tauri/vendor/nb-shared`, con
su manifiesto de huellas. `check-shared` comprueba el inventario sin necesitar el repositorio de NB Launcher.
