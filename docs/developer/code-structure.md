# Estructura del código

> **Estado:** Implementado.

```text
docs/                 Fuente única de verdad (español). Empieza en docs/README.md
src/                  Interfaz: SvelteKit 2 + Svelte 5, adaptador estático
  routes/             Una ruta por destino (Inicio, Biblioteca, Playlists, Tu música, DJ, A ciegas, Ajustes...)
  lib/core/           Estado de la aplicación y acceso al puente
  lib/core/ipc/       El único punto de acceso al runtime
  lib/bindings/       Tipos generados desde Rust con ts-rs. No se editan a mano
  lib/i18n/           Diccionarios en español, inglés y portugués
  lib/ui/             Componentes y vistas, por área
static/               Fuentes, marca y favicon
src-tauri/            Runtime: Rust + Tauri 2
  src/comandos.rs     La superficie del puente
  src/reproductor/    La sesión de reproducción
  src/audio/          El motor: decodificador, fuente de la cola, ecualizador, limitador
  src/db/             La base y su acceso
  src/catalogo/       Importar, identificar, etiquetar, organizar, Por revisar
  src/dj/             El DJ Privado
  src/mashup/         DJ Mashups
  src/movil/          El servidor para NB Sound Mobile
  src/a_ciegas/       El juego
  src/playlists/      Playlists inteligentes y automáticas
  src/net.rs          La única salida a internet
  src/telemetry.rs    Registro local redactado; no envía nada
  src/idioma.rs       El idioma de lo que nombra el runtime
  migrations/         SQL embebido; una migración publicada no se edita
  tests/              Pruebas de integración: contrato móvil, audio, arranque, catalogación
  tools/              Herramientas de publicación (catálogo de NB Launcher)
  vendor/nb-shared/   Exportación sellada de las bibliotecas compartidas con NB Launcher
scripts/              Guardianes, binarios, empaquetado, avisos de terceros, publicación y QA
.github/              CI, ediciones, publicación y catálogo de NB Launcher
```
