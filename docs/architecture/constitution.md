# Constitución

> **Estado:** Aplicable. Se modifica únicamente mediante un ADR aceptado.

La filosofía marca una dirección; **la Constitución dicta leyes**. Ningún cambio se integra en `main` si rompe una
de estas reglas. Cada regla dice qué la hace cumplir: una regla sin mecanismo es una intención, y las intenciones no
protegen a nadie.

---

## 1. Si hay que salir de la aplicación para instalar algo, la aplicación está mal hecha

Una aplicación que depende del entorno al que llega (el reproductor del sistema, un intérprete, un gestor de
paquetes para los modelos, una pantalla entera para diagnosticar qué falta) traslada su problema a quien la usa.
NB Sound no lo hace.

* Todo lo que la aplicación necesita para reproducir, catalogar, sincronizar y editar **viaja dentro del
  instalador**: el motor de audio, los decodificadores, la huella acústica (`fpcalc`) y el runtime de inferencia
  (ONNX Runtime para CPU).
* Lo que es grande y opcional (los modelos del karaoke, del análisis musical y de los pedidos al DJ, y la
  aceleración por GPU) es un **agregado**: se instala **con un clic desde Ajustes**, se verifica por su huella, se
  puede quitar, y nunca pide una terminal, un gestor de paquetes ni un intérprete del sistema
  ([ADR-006](../adr/006-add-ons.md)).
* Ninguna pantalla le explica a la persona qué le falta a su sistema. Si algo falta, es un defecto del empaquetado.

**Mecanismo:** la CI de Linux compila los instaladores con todo lo anterior dentro, y `check-plataforma` rechaza
código que busque ejecutables en el `PATH`.

## 2. Un solo producto, sin modos

No existe un "modo experto" ni un "modo simple". La aplicación es sencilla por defecto y profunda bajo demanda.
Lo que solo entiende quien programa (causas internas de rechazo, colas, rutas auxiliares, versiones de bibliotecas)
**no se muestra**: se registra.

**Mecanismo:** [ADR-007](../adr/007-information-architecture.md) fija los destinos; un destino nuevo exige un ADR.

## 3. Lo del usuario es intocable

* La biblioteca, las playlists, el historial, los favoritos, los teléfonos emparejados, los instrumentales y los
  mashups **sobreviven a cualquier actualización** sin pérdida.
* NB Sound **nunca escribe** en la carpeta de la música salvo por una acción explícita (importar y organizar,
  editar los datos de una canción, llevar la música a otra carpeta).
* Desinstalar no se lleva la biblioteca.

**Mecanismo:** cada migración del esquema tiene su prueba, se aplica con un respaldo previo de la base, y la copia
de seguridad guarda todo menos el audio ([ADR-004](../adr/004-data-ownership.md)).

## 4. NB Sound Mobile no se rompe

NB Sound es compatible **byte a byte** con el contrato que NB Sound Mobile espera (protocolo `1`): mismas rutas,
mismos campos, mismo certificado y la misma huella, los mismos `device_token` y los mismos identificadores de pista.
Solo se admiten cambios **aditivos**, que el cliente ignora por diseño.

**Mecanismo:** las respuestas doradas del contrato se reproducen contra el servidor de NB Sound en cada
`cargo test` ([ADR-005](../adr/005-mobile-contract.md), [contrato móvil](mobile-contract.md)).

## 5. Nada sale del dispositivo sin consentimiento

AcoustID, MusicBrainz, Cover Art Archive, iTunes, LRCLIB, lyrics.ovh, Deezer, Wikipedia, TheAudioDB, el
desempate por IA, los agregados y las actualizaciones son salidas de red. Cada una pasa por una única puerta que
comprueba el interruptor general y el consentimiento por servicio. Los registros no contienen rutas, títulos,
artistas ni claves. No hay telemetría de ninguna clase.

**Mecanismo:** `check-network` rechaza cualquier cliente HTTP fuera de `net.rs`; `check-registro` rechaza campos de
registro sin redactar ([ADR-009](../adr/009-settings-and-network.md)).

## 6. Un fallo local tiene consecuencias locales

Si falla el karaoke, la música sigue sonando. Si falla la identificación, la pista entra igual y queda en Por
revisar. Si falla el servidor móvil, el escritorio no se entera. Ningún trabajo pesado comparte hilo con el audio ni
con la ventana.

**Mecanismo:** el audio corre en su propio hilo de tiempo real; ningún comando del puente es síncrono
(`check-comandos`); una prueba de carga del motor con todos los núcleos ocupados
([ADR-003](../adr/003-audio-engine.md)).

## 7. Un ecosistema, no aplicaciones parecidas

NB Sound comparte con el resto de la suite NB Digital la pila, la estructura de carpetas, el contrato de errores,
el sistema de diseño, los guardianes y la forma de trabajar. Quien sepa moverse en una aplicación se mueve en otra
sin leer nada.

**Mecanismo:** [ADR-001](../adr/001-tech-stack.md); los guardianes son los mismos que en NB Books.

## 8. La documentación es parte del producto

`docs/` es la fuente única de verdad. Si el código y `docs/` discrepan, hay un defecto. Un cambio con impacto en la
arquitectura o en el uso no está terminado hasta que su documento lo refleja, en el mismo cambio.

**Mecanismo:** `check-docs` comprueba enlaces, duplicados, estructura y la etiqueta de estado de cada documento.
