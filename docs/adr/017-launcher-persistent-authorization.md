# ADR-017: Autorización persistente y local de NB Launcher

**Fecha:** 2026-11-13
**Estado:** Aceptado

## Contexto

El arranque heredado solo enlaza un NB Sound iniciado por NB Launcher. Una instancia abierta por separado necesita
comprobar la identidad de quien pide, pedir consentimiento y recordar lo concedido. El nombre, la ruta o el PID de
un proceso no demuestran continuidad entre sesiones.

## Alternativas consideradas

1. **Pedir permiso en cada sesión.** Rechazada: se eligió recordar con revocación persistente.
2. **Reutilizar la clave editorial de las publicaciones.** Rechazada: su autoridad es otra.
3. **Guardar la clave privada en un archivo si falla el llavero.** Rechazada.
4. **Guardar las concesiones en la base de la biblioteca.** Rechazada: restaurar una copia antigua devolvería
   permisos revocados.
5. **Un registro propio de claves públicas, fuera de la biblioteca.** Aceptada.

## Decisión

* NB Launcher mantiene una identidad Ed25519 por instalación en el llavero del sistema. NB Sound comprueba desafíos
  de un solo uso, ligados al recurso, las operaciones y el PID que da el transporte local, con `ed25519-dalek`; no
  hay criptografía propia.
* NB Sound guarda un registro SQLite propio, `autorizaciones-launcher.sqlite3`, fuera de la base y fuera de la copia
  de seguridad: claves públicas, ámbitos, revisiones y tumbas, sin credenciales de sesión. Todas sus escrituras usan
  `BEGIN IMMEDIATE`. Restaurar una biblioteca no cambia el registro; si falta, se vuelve a denegar.
* El consentimiento se pide en NB Sound después de comprobar la posesión de la clave, con una espera acotada de 45
  segundos y el foco en Cancelar. Aceptar compara la revisión vista al abrir la solicitud con la actual en la misma
  transacción, así que una revocación mientras el diálogo estaba abierto impide que una respuesta antigua vuelva a
  autorizar. Denegar no guarda nada.
* Revocar, en Ajustes, Privacidad, deja una tumba y cierra todas las sesiones de esa concesión después del commit.
* Una sola instancia: se adquiere antes de abrir la biblioteca, y una segunda apertura pide activar la existente.
* La adhesión de una instancia abierta por separado y la instancia única están habilitadas en Linux; en Windows y
  macOS NB Sound conserva el enlace heredado y el arranque de siempre.

## Consecuencias

- **Positivas:** un consentimiento recordado, revocable y que no sobrevive a una revocación por ninguna vía.
- **Negativas:** la adhesión se limita a Linux.
- **Riesgos y cómo se vigilan:** el llavero y los permisos locales no aíslan de un atacante con el mismo usuario o
  administrador. Las pruebas cubren consentimiento, rechazo, continuidad, revocación con sesiones activas,
  restauración sin recuperar concesiones, fallo de almacenamiento e instancia única.
