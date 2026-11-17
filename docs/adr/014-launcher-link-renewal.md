# ADR-014: Renovación privada del enlace con NB Launcher

**Fecha:** 2026-11-12
**Estado:** Aceptado

## Contexto

El enlace entre NB Launcher y NB Sound caduca a los 15 minutos por seguridad, y la conexión autorizada se perdía
aunque la persona siguiera usando las dos aplicaciones.

## Alternativas consideradas

1. **Quitar la caducidad.** Rechazada: las credenciales servirían hasta cerrar el proceso.
2. **Renovar con la credencial de un cliente.** Rechazada: amplía la autoridad de ese cliente.
3. **Reiniciar NB Sound para renovar.** Rechazada: interrumpe la música.
4. **Renovar solo por el canal privado heredado del supervisor.** Aceptada.

## Decisión

El saludo del protocolo en su versión 2 anuncia la renovación; la versión 1 sigue funcionando para clientes
anteriores. Tras el arranque se aceptan mensajes `{"method":"renew"}` solo por la entrada estándar privada: cada uno
completo y válido extiende la vigencia 900 segundos. El fin de la entrada, un mensaje inválido o la falta de
renovación cierran el enlace y sus sesiones **sin cerrar NB Sound**. Ningún cliente del puerto puede renovar ni
ampliar permisos.

## Consecuencias

- **Positivas:** NB Launcher mantiene un enlace consentido sin reiniciar NB Sound.
- **Negativas:** una suspensión que agote la vigencia exige una conexión nueva.
- **Riesgos y cómo se vigilan:** los límites de tiempo se prueban con reloj controlado; un mensaje incompleto no
  extiende el plazo.
