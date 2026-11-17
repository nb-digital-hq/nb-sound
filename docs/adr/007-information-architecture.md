# ADR-007: Arquitectura de información

**Fecha:** 2026-10-29
**Estado:** Aceptado

## Contexto

Un reproductor que crece por acumulación acaba con decenas de vistas, las más grandes justo donde menos deberían
pesar (configuración, importación, diagnóstico), y con un interruptor de "modo avanzado" que admite que la
aplicación son dos productos. Tampoco tiene sentido pedir que se configuren varias carpetas de trabajo ni exponer
cien variables de configuración.

## Alternativas consideradas

1. **Una vista por función** (importar, preparar karaoke, sincronizar, estado del sistema). Rechazada: obliga a
   aprender dónde vive cada cosa y convierte procesos de fondo en pantallas.
2. **Pocos destinos fijos; lo demás donde se usa.** Aceptada.

## Decisión

### Destinos de la barra lateral

| Grupo | Destino | Qué contiene |
|---|---|---|
| Explorar | **Inicio** | Retomar, el álbum del día, lo último que sonó, tops, carruseles; la bienvenida si la biblioteca está vacía |
| Tu música | **Biblioteca** | Álbumes, Artistas y Pistas en pestañas; filtros de favoritas, sin verificar y listas para cantar; álbum y artista dentro |
| Tu música | **Playlists** | Manuales, inteligentes y las creadas para ti, en una sola lista |
| Tu música | **Tu música** | Identidad musical, actividad, tops y el resumen local de escucha |
| Experiencias | **DJ** | DJ Privado y DJ Mashups en la misma pestaña ([ADR-018](018-dj-mashups.md)) |
| Experiencias | **A ciegas** | El juego para redescubrir la biblioteca |
| Al pie | **Ajustes** | Ver abajo |

**Por revisar** aparece al pie **solo cuando hay archivos esperando**, con su número, y desaparece al vaciarse.

**Buscar** no es un destino: es un campo permanente en la barra superior (Ctrl+K). **El reproductor** es la barra
inferior permanente, con la reproducción ampliada, la cola y el minirreproductor. **Karaoke**, **Dispositivos** y
**Añadir música** son consolas laterales que se abren desde la barra superior.

### Lo que no existe

| No existe | Dónde está su función |
|---|---|
| Modo avanzado o modo simple | Hay una sola aplicación, sencilla por defecto |
| Una vista de importación | Arrastrar a la ventana, "Añadir música" o la carpeta vigilada; una tarjeta discreta dice el progreso |
| Una vista para preparar el karaoke | La consola de karaoke y el micrófono del reproductor |
| Una vista de estado del sistema | Ajustes, Agregados ([ADR-006](006-add-ons.md)) |
| Una vista de sincronización | La consola de Dispositivos |
| Carpetas de cuarentena o revisión que configurar | Una sola **carpeta de tu música**; lo demás va a los directorios estándar del sistema |
| Causas internas, colas, versiones | Solo en el registro |

### Ajustes

Secciones, en este orden: **Biblioteca**, **Almacenamiento**, **Reproducción**, **Apariencia** (incluido el
idioma), **Agregados**, **Permisos**, **Privacidad y servicios en línea** (incluido el acceso de NB Launcher),
**Inteligencia artificial** y **Acerca de**.

### Por revisar

Lo que no entra solo **no se esconde ni se pierde**: queda visible con su motivo en lenguaje llano y la persona
decide. Las causas internas se agrupan así:

| Causa interna | Grupo | Acción a la vista |
|---|---|---|
| `candidatos_ambiguos` | ¿Es la versión correcta? | Es esta, junto a cada candidata |
| `puntaje_intermedio`, `duracion_marginal` | Sin confirmar | Importar |
| `sin_candidatos`, `metadata_insuficiente`, `puntaje_bajo` | No las encontramos | Importar tal como está |
| Duplicado | Ya las tienes | Descartar, diciendo con qué canción coincide |
| `escritura_fallida`, `validacion_post_escritura` | No se pudieron guardar | Reintentar |
| `duracion_invalida`, audio ilegible | Archivos dañados | Descartar |

Cada grupo tiene su acción para todas, y lo que es en lote se confirma antes. **Importar tal como está** mete la
pista en la misma biblioteca con las etiquetas que trae y la marca **sin verificar**: una insignia, un filtro en
Biblioteca y "Completar datos" siempre a mano. El teléfono la recibe como cualquier otra, con el campo aditivo
`verificada`. Mientras espera, el archivo se conserva en el directorio interno de la aplicación; descartar lo manda a
la papelera del sistema, nunca lo borra.

## Consecuencias

- **Positivas:** seis destinos fijos y una consola por tarea; nada que parezca de desarrollo.
- **Negativas:** quien quisiera la tabla de causas internas no la ve: está en el registro.
- **Riesgos y cómo se vigilan:** que Por revisar se convierta en una vista técnica con otro nombre; cada fila es la
  canción como se reconoce, con su motivo dicho una vez en la cabecera del grupo.
