# ADR-013: La frecuencia de las copias como capacidad compartida con NB Launcher

**Fecha:** 2026-11-11
**Estado:** Aceptado

## Contexto

NB Launcher, el centro de mando de la suite, tiene que poder consultar y cambiar cada cuánto NB Sound hace copias
de seguridad, sin dejar de ser NB Sound el único dueño de ese ajuste: la validación, la ejecución de las copias, la
carpeta, la retención y el formato son suyos. Las bibliotecas de protocolo y de capacidades de NB Launcher
(`nb-protocol`, `nb-capability`, Apache-2.0) ya resuelven revisiones, observación de cambios y conflictos.

## Alternativas consideradas

1. **Que NB Launcher lea y escriba la base de NB Sound.** Rechazada: duplica reglas y salta al dueño.
2. **Copiar la lógica de la frecuencia a NB Launcher.** Rechazada: dos implementaciones del mismo dominio.
3. **Depender de las bibliotecas por una ruta a la carpeta de NB Launcher.** Rechazada: no existe en un clon limpio
   ni en la CI.
4. **Una exportación sellada de las bibliotecas dentro del repositorio.** Aceptada.

## Decisión

* **Propiedad.** NB Sound expone solo la frecuencia (0, 1, 7 o 30 días). Una escritura compara la revisión, valida
  con la lógica de NB Sound y confirma la preferencia y su contador en una misma transacción `BEGIN IMMEDIATE`
  (migración V12, tabla `revision_de_recursos`). La época de autoridad vive en memoria y se genera al abrir.
* **Bibliotecas compartidas.** Una exportación sellada en `src-tauri/vendor/nb-shared`, con su fuente, su manifiesto
  de huellas, su licencia y un verificador propio (`check-shared`). No se edita a mano; se actualiza regenerándola
  desde su origen.
* **Interfaz y conflictos.** La pantalla envía la revisión que muestra y recibe aplicado, sin cambios o conflicto.
  Ante un conflicto adopta el estado confirmado y explica que cambió desde otro control, sin reintentar una edición
  antigua. Los cambios se publican después del commit y la pantalla abierta los recibe sin recargar.
* **Permisos.** El acceso requiere autorización para esta capacidad concreta ([ADR-017](017-launcher-persistent-authorization.md));
  conceder la frecuencia no da acceso a ninguna otra preferencia, archivo, reproducción ni restauración.
* **Autonomía.** NB Sound arranca y funciona sin NB Launcher; perder el enlace solo detiene el enlace. Las
  credenciales llegan por un canal privado, nunca por argumentos, entorno, disco ni registro.

## Consecuencias

- **Positivas:** una sola lógica, cambios en los dos sentidos, conflictos explícitos y un clon de NB Sound que compila
  solo.
- **Negativas:** una migración más y una exportación fijada que se actualiza a propósito.
- **Riesgos y cómo se vigilan:** escrituras sin revisión, avisos antes de persistir, deriva de la exportación; cada
  uno tiene su prueba, y `check-shared` comprueba el inventario y las huellas.
