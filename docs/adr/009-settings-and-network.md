# ADR-009: Configuración mínima, secretos en el llavero y una sola puerta de red

**Fecha:** 2026-10-29
**Estado:** Aceptado

## Contexto

Configurar con un archivo de cien variables y guardar en él, en texto plano, claves de servicios es lo contrario de
lo que pide la [Constitución](../architecture/constitution.md). Y salir a la red desde muchos clientes distintos hace
imposible garantizar el consentimiento.

## Alternativas consideradas

1. **Un archivo de configuración con variables y claves.** Rechazada por lo anterior.
2. **Preferencias en la base, secretos en el llavero y una única puerta de red.** Aceptada.

## Decisión

1. **Tres clases de valor:**
   * **Constantes de producto** (umbrales, tamaños de lote, reintentos, tiempos de espera): constantes en Rust con su
     motivo en un comentario. No son configurables.
   * **Preferencias**: solo las que aparecen en Ajustes, guardadas en la tabla `preferencias`. La interfaz solo puede
     escribir las claves de una lista cerrada, cada una con su validación.
   * **Secretos**: en el llavero del sistema operativo con `keyring`. Nunca en archivos, registros ni respuestas del
     puente. Si el sistema no tiene llavero, no se guarda ninguna clave.
2. **Claves de la aplicación**: la clave de cliente de TheAudioDB es pública y viaja en el binario. AcoustID pide una
   clave gratuita por persona, que se guarda en el llavero; sin ella no se reconoce una canción por su sonido. El
   desempate por IA usa la cuenta y la clave de la propia persona.
3. **Una sola puerta de red** (`net.rs`): toda salida declara su servicio; la puerta comprueba el interruptor general
   y el del servicio, y el anfitrión permitido para ese servicio. Servicios: AcoustID, MusicBrainz, Cover Art
   Archive, iTunes, LRCLIB, lyrics.ovh, Deezer, Wikipedia, TheAudioDB, el proveedor de IA, los agregados y
   las actualizaciones. Con el interruptor general apagado no sale nada.
4. **Desempate por IA**: opcional y apagado de fábrica. Solo se envían los datos de la canción y de los candidatos,
   nunca el audio.
5. **Actualizaciones**: solo pueden ir a `github.com` y `*.githubusercontent.com`. Pulsar "Buscar actualizaciones"
   cuenta como consentimiento. `tauri-plugin-updater` hace la consulta y la descarga con su propio cliente, y se
   acepta con tres límites: se usa solo dentro de `net.rs` (`check-network` rechaza cualquier otro uso); el
   manifiesto y la URL de descarga pasan antes por la autorización de la puerta, y lo que llega no se instala si su
   firma no corresponde a la clave pública de la aplicación.

## Consecuencias

- **Positivas:** ninguna clave en texto plano; un solo sitio que auditar para la privacidad.
- **Negativas:** desaparece el ajuste fino por variables; una constante mal elegida se cambia en el código con su
  prueba.
- **Riesgos y cómo se vigilan:** que un servicio cambie su API; afecta a una de varias fuentes, no al producto, y
  NB Sound solo usa servicios con API pública. Se descartó el reconocimiento por Shazam: su API no es pública.
