# Contratos del puente

> **Estado:** Implementado.

La interfaz y el runtime se hablan por el puente de Tauri: comandos que la interfaz invoca y eventos que el runtime
emite. Estas son sus reglas, todas comprobadas por un guardián.

## Reglas

1. **Un solo punto de acceso.** Solo `src/lib/core/ipc/` importa la API de Tauri; cualquier otro componente que lo
   intente falla el lint (`noRestrictedImports` en `biome.json`).
2. **Los tipos se generan.** Todo tipo que cruza el puente se declara en Rust con `ts-rs` y se escribe en
   `src/lib/bindings/` al ejecutar `cargo test`. `check-bindings` regenera y compara: un tipo que cambió sin
   regenerarse rompe la verificación.
3. **Ningún comando es síncrono.** Todos son `async` y lo pesado se hace en un hilo de trabajo
   (`en_segundo_plano`); ninguno corre en el hilo de la ventana (`check-comandos`). Las excepciones (abrir ventanas,
   selectores nativos) están declaradas.
4. **Toda colección está acotada.** Un comando que devuelve una lista lleva un `limit` o declara por qué su tamaño
   no depende de la biblioteca (`Cota: …` en su documentación); lo vigila `check-listas`.
5. **Toda ruta que llega de fuera se canonicaliza** y se acota a su ámbito (`security::resolve_within`,
   `check-rutas`). Los selectores de archivos se abren desde Rust: ninguna ruta elegida cruza el puente desde el
   webview.
6. **Cada comando tiene interfaz.** Un comando registrado que nadie llama desde la interfaz es un defecto
   (`check-puente`).
7. **Ningún error viaja como cadena** (abajo).

## El contrato de errores

`NsError` (`src-tauri/src/error.rs`) tiene un código canónico estable por variante, que la interfaz traduce en su
idioma. Renombrar un código es un cambio incompatible. `check-error-contract` rechaza cualquier `Result<_, String>`
en la superficie pública.

| Código | Significado |
|---|---|
| `FILE_NOT_FOUND` | El archivo pedido no existe |
| `PATH_NOT_ALLOWED` | La ruta, o la clave de preferencia, está fuera de lo permitido |
| `FORMAT_UNSUPPORTED` | Un formato que el motor no decodifica |
| `AUDIO_CORRUPTED` | El audio no se puede leer |
| `STORAGE_FAILURE` | La base o el disco fallaron |
| `SCHEMA_MISMATCH` | La base es de una versión más nueva |
| `COMPONENT_UNAVAILABLE` | Falta un componente (un modelo, ONNX Runtime) |
| `INSTALLATION_MANAGED` | La instalación la actualiza NB Launcher |
| `NETWORK_DISABLED` | Los servicios en línea están apagados |
| `CONSENT_REQUIRED` | El servicio no tiene consentimiento |
| `NETWORK_FAILURE` | La red falló |
| `INTEGRITY_MISMATCH` | Una descarga no coincide con su huella o su firma |
| `ADDON_REQUIRED` | Hace falta un agregado |
| `CANCELLED` | La persona canceló |
| `INVALID_INPUT` | Un valor fuera de rango |
| `NAME_TAKEN` | Ya existe una playlist con ese nombre |
| `INTERNAL` | Un fallo que no debería ocurrir |

## Eventos

| Evento | Qué avisa |
|---|---|
| `reproductor:instantanea` | El estado del reproductor cambió (canción, cola, pausa, volumen) |
| `reproductor:aviso` | Algo que decir: un archivo que falta, una pista que ya no está |
| `biblioteca:cambio` | La biblioteca cambió por algo que no inició la interfaz |
| `importacion:progreso`, `importacion:arrastre` | El avance de una importación; archivos soltados en la ventana |
| `carpeta:avance` | El avance de llevar la música a otra carpeta |
| `karaoke:cambio` | La cola de preparación del karaoke cambió |
| `agregados:cambio` | El estado o el progreso de un agregado |
| `dj:cambio` | La sesión del DJ cambió |
| `mashup:progreso`, `mashup:terminado` | El avance y el final de un mashup |
| `movil:cambio` | Los teléfonos o su conexión cambiaron |
| `fiesta:cambio` | Las sugerencias del Modo fiesta |
| `copia:frecuencia` | La frecuencia de las copias cambió desde otro control |
| `launcher:cambio` | Una solicitud o un cambio de autorización de NB Launcher |

La posición de la canción no viaja por eventos: la interfaz la interpola con `requestAnimationFrame` a partir del
último estado ([ADR-008](../adr/008-design-system.md)).
