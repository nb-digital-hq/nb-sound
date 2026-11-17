# ADR-010: Modelo de licencias

**Fecha:** 2026-11-17
**Estado:** Aceptado

## Contexto

NB Digital sigue un modelo dual en toda su suite: **las aplicaciones para personas son de código abierto bajo la
Apache License 2.0**, y **los módulos y servicios de gobernanza para instituciones se licencian comercialmente** por
suscripción. Las marcas "NB", "NB Digital" y "NB Sound" y sus logotipos quedan fuera de las dos licencias. NB Books ya
se publica así.

NB Sound tiene dos componentes cuya licencia condiciona la decisión:

* **El reconocimiento por Shazam** se apoyaba en una adaptación de `songrec`, que es GPL-3.0, y en una API de Shazam
  que no es pública.
* **Los modelos del agregado de análisis musical** (MusiCNN y sus cabezas, del Music Technology Group de la
  Universitat Pompeu Fabra) se licencian con Creative Commons para uso no comercial.

El código lo escribieron sus autores con la declaración de origen de cada commit, sin aportaciones de terceros que
pudieran oponerse a cambiar la licencia.

## Alternativas consideradas

1. **Conservar la GPL-3.0 con el reconocimiento por Shazam.** Rechazada: rompe el modelo de la suite y el riesgo del
   servicio, una API no pública, existiría con cualquier licencia.
2. **Reescribir el reconocimiento por Shazam o separarlo como programa GPL.** Rechazada: no sería una reescritura
   independiente y la API seguiría sin ser pública.
3. **Sustituir Shazam por un servicio de pago con la clave de cada persona.** Rechazada: NB Sound no depende de
   servicios de pago.
4. **Apache-2.0 sin el reconocimiento por Shazam, y los modelos del MTG como agregado con su propia licencia.**
   Aceptada.

## Decisión

* **El código de NB Sound y su documentación se publican bajo la Apache License 2.0** (`LICENSE`, `NOTICE`).
* **El reconocimiento por Shazam se retira.** La identificación usa AcoustID y MusicBrainz, el ISRC de las etiquetas
  cuando lo hay, e iTunes para confirmar; lo que no se reconoce entra con sus etiquetas o queda en Por revisar.
* **Los modelos del MTG se distribuyen aparte, como agregado opcional**, con su licencia no comercial y su atribución
  dentro del propio archivo y en la [documentación](../legal/model-licenses.md). No viajan en el repositorio ni en el
  instalador: se descargan solo si la persona los instala. El código Apache que los usa (el preprocesado y la
  llamada a ONNX Runtime) no contiene los modelos ni deriva de ellos.
* **Ninguna oferta comercial de NB Digital incluye esos modelos.** Si un módulo institucional necesitara el análisis
  musical, haría falta una licencia comercial del MTG o sustituir los modelos.
* **Dependencias:** solo licencias permisivas y la MPL-2.0, cuyo copyleft es por archivo. `cargo deny` y
  `generate-notices` rechazan cualquier otra.
* **Procedencia:** declaración de origen (DCO) en cada commit, sin acuerdo de cesión; cada contribuidor conserva su
  copyright.

## Consecuencias

- **Positivas:** la misma licencia que el resto de la suite; sin código GPL ni servicios no públicos.
- **Negativas:** sin Shazam se reconocen menos canciones sin etiquetas, sobre todo sin la clave de AcoustID.
- **Riesgos y cómo se vigilan:** que el MTG cambie sus condiciones; afecta a un agregado, no a la aplicación.
  `cargo deny check licenses` corre en cada CI.
