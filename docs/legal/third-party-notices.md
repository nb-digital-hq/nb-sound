# Avisos de terceros

> **Estado:** Implementado.

[THIRD-PARTY-NOTICES.md](../../THIRD-PARTY-NOTICES.md), en la raíz del repositorio, enumera cada componente que
viaja en la aplicación con su versión, su licencia y su origen: las dependencias de Rust, las de producción de la
interfaz, los binarios incluidos (`fpcalc`, ONNX Runtime), las fuentes y los modelos de los agregados. Viaja también
dentro de cada paquete y en cada publicación.

Lo genera `scripts/generate-notices.mjs` a partir de `cargo metadata` y del árbol de dependencias de producción de
pnpm; falla si encuentra una licencia que la [política de dependencias](../architecture/dependency-policy.md) no
admite. Se regenera al cambiar cualquier dependencia.

Los textos completos de las licencias de las fuentes están en `static/fuentes/LICENCIAS.md`.
