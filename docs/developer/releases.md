# Publicación

> **Estado:** Implementado.

## Versionado

Versionado semántico. La versión se cambia a la vez en `package.json`, `src-tauri/Cargo.toml`,
`src-tauri/tauri.conf.json`, `CHANGELOG.md` y `CITATION.cff`. Hay un solo canal: las publicaciones de
`nb-digital-hq/nb-sound` en GitHub, que el actualizador integrado consulta.

## Lista previa

1. La verificación completa en verde ([pruebas](testing.md)) y la CI de `main` en verde en los tres sistemas.
2. Los paquetes de Linux construidos en local y la aplicación instalada desde ellos probada sobre datos de prueba.
3. El flujo **Ediciones** terminado para Windows y macOS, y sus artefactos descargados.
4. `CHANGELOG.md` con la sección de la versión, de donde salen las notas.

## Qué lleva una publicación

* Los paquetes de las tres plataformas, cada uno con su `.sig` minisign ligada a la versión
  (`tauri signer sign --app-version`): el actualizador rechaza un manifiesto que anuncie otra versión.
* `latest.json`, el manifiesto del actualizador.
* `NOTAS.md`, la sección de la versión en `CHANGELOG.md`.
* `nb-sound_<versión>.cdx.json`, el listado de componentes con sus licencias en CycloneDX 1.5
  (`node scripts/generate-notices.mjs --cdx`). Dice de qué commit salió.
* `launcher-catalog.json` y su `.sig`, el catálogo para NB Launcher ([ADR-016](../adr/016-launcher-catalog.md)).
* Los agregados de esa versión (`karaoke-1.zip`, `analisis-1.zip`, `texto-dj-1.zip`, `aceleracion-linux-1.zip`).
* `THIRD-PARTY-NOTICES.md`.
* `SHA256SUMS` con la huella de todo lo anterior y `SHA256SUMS.sig`, su firma Ed25519 con la clave de publicación de
  NB Digital.

Con todo lo anterior en una carpeta, `scripts/procedencia.mjs` escribe `SHA256SUMS`, lo firma con `openssl` y
comprueba la firma contra la clave pública del repositorio (`docs/architecture/api/clave-de-publicacion.pem`), no
contra la que acaba de usar. GitHub sustituye los espacios de los nombres por puntos: los archivos entran en la carpeta
ya con el nombre publicado (`NB.Sound_<versión>_amd64.deb`), que es el que esperan NB Launcher y el actualizador.

El catálogo de NB Launcher no lo monta nadie a mano: el flujo **Catálogo de Launcher** lo genera y lo firma al
publicarse la versión, y lo renueva cada semana.

## Verificar una descarga

```bash
# 1. La firma de las sumas, con la clave pública de NB Digital
openssl pkeyutl -verify -pubin -inkey clave-de-publicacion.pem -rawin \
  -in SHA256SUMS -sigfile SHA256SUMS.sig

# 2. La huella de lo descargado
sha256sum --check --ignore-missing SHA256SUMS
```

La clave pública está en el repositorio, en `docs/architecture/api/clave-de-publicacion.pem`.

## Soporte

Recibe correcciones la última versión publicada. Una migración de esquema no se revierte: ante un fallo grave se
publica una versión de corrección hacia adelante y sus notas explican qué tiene que hacer quien actualizó.
