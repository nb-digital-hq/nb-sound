# Catalogación

> **Estado:** Implementado. Decisiones: [ADR-002](../adr/002-no-python.md), [ADR-007](../adr/007-information-architecture.md).

De un archivo cualquiera a una pista de la biblioteca con sus datos, su portada y su letra.

## La canalización de un archivo

1. **Validar.** Que se pueda leer y que su duración sea razonable. Lo dañado va a Por revisar.
2. **Duplicados exactos.** La huella de contenido (SHA-256 del primer y el último MB) dice si el archivo ya está.
3. **Identificar.** Con la red permitida y la clave de AcoustID, la huella de `fpcalc`; si el archivo trae un ISRC en
   sus etiquetas, se usa para buscar la grabación. Sin red, un archivo con título y artista en sus etiquetas entra tal como está, sin verificar.
4. **Normalizar.** Títulos y artistas a una forma comparable (Unicode NFKC/NFD, sin diacríticos, sin "feat." ni
   sufijos de edición), para buscar y para comparar.
5. **Buscar en MusicBrainz** por el identificador de AcoustID o por título, artista y álbum.
6. **Decidir en tres etapas.** Cada candidato se puntúa por título, artista, álbum, duración e identificadores; se
   acepta, se compara con iTunes o se desempata con IA si la persona lo activó. Lo dudoso va a Por revisar con su
   causa.
7. **Duplicados por identidad.** La misma grabación de MusicBrainz o el mismo ISRC. La copia de menos calidad se
   aparta: no se ve ni va al teléfono, y su archivo se queda donde está.
8. **Organizar.** Si viene de fuera de la carpeta de la música se **copia**; si ya está dentro, se mueve a su sitio:
   `Artista / tipo / Álbum / 01 - Canción`. Nunca se transcodifica.
9. **Etiquetar.** `lofty` escribe las etiquetas y la portada en el archivo, en su formato.
10. **Portada, letra e imagen del artista.** Cover Art Archive, iTunes, Deezer, Wikipedia o TheAudioDB para las
    imágenes; LRCLIB y lyrics.ovh para la letra, sincronizada si existe.

Lo que entra lo hace en una transacción; lo que no, va a **Por revisar** con el archivo guardado en el directorio
interno de la aplicación.

## Las fuentes de música

* **Añadir archivos o una carpeta**, o arrastrar a la ventana: se importa en una cola con su progreso, que se puede
  cancelar sin perder lo ya añadido.
* **La carpeta vigilada.** Lo que se copia en la carpeta de la música entra solo cuando termina de copiarse
  (`notify`).
* **Lo que falta, se completa solo.** Cinco minutos después de abrir, con la red permitida y a prioridad baja, cada
  canción sin portada, sin letra o con su artista sin imagen pasa por "Completar datos"; lo que no se encuentra se
  vuelve a intentar al mes (V11).

## Formatos

Se importan MP3, FLAC, M4A (AAC-LC y ALAC), OGG Vorbis y WAV. Lo que el motor no decodifica (HE-AAC, Opus, WMA, APE)
va a Por revisar con el motivo "formato no compatible".

## Por revisar

Las causas internas, cómo se agrupan y qué acción ofrece cada grupo están en
[ADR-007](../adr/007-information-architecture.md). Descartar manda el archivo a la papelera del sistema con el
nombre con que llegó (`trash`); nunca se borra del todo.

## Editar y borrar

* **Editar datos** de una canción o de un álbum escribe en la biblioteca y, si se puede, en el archivo. Los
  identificadores no cambian, así que las playlists y el teléfono la siguen reconociendo.
* **Eliminar** quita la canción de la biblioteca, de las playlists, de la cola y del teléfono en la siguiente
  sincronización; su archivo puede quedarse o ir a la papelera.
