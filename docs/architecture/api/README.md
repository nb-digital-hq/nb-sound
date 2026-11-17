# Claves públicas

> **Estado:** Implementado.

| Archivo | Para qué |
|---|---|
| `clave-de-publicacion.pem` | La clave pública Ed25519 de NB Digital, con la que se comprueba `SHA256SUMS.sig` en cada publicación ([publicación](../../developer/releases.md)) |

La clave pública del actualizador, con la que la aplicación comprueba cada paquete antes de instalarlo, está en
`src-tauri/tauri.conf.json` (`plugins.updater.pubkey`). Las claves privadas nunca entran al repositorio.
