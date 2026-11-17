# Pruebas

> **Estado:** Implementado.

## Las suites

| Suite | Dónde | Qué cubre | Cómo se ejecuta |
|---|---|---|---|
| Interfaz | `src/**/*.test.ts` | Estado, formato, navegación, avisos, idiomas | `pnpm test` |
| Runtime | `src-tauri/src/**` (`#[cfg(test)]`) | Cada módulo: motor de audio fuera de línea, catalogación, DJ, mashups, base, red, karaoke | `cargo test` |
| Contrato móvil | `src-tauri/tests/contrato_movil.rs` | Las respuestas doradas del protocolo de NB Sound Mobile | `cargo test` |
| Integración | `src-tauri/tests/qa_*.rs` | Audio con la CPU llena, DJ de extremo a extremo, catalogación, forma de onda, resumen de escucha, copia a dispositivos, Modo fiesta, arranque | `cargo test` |
| Rendimiento | `db::busqueda`, `mashup::rendimiento` | La búsqueda en 100.000 pistas; mediciones de mashups | `pnpm test:rendimiento`; las de mashups con `--ignored` |
| Guardianes | `scripts/check-*.mjs` | Las reglas de la [Constitución](../architecture/constitution.md) y de las normas de código | `pnpm guardianes` |
| De extremo a extremo con la aplicación real | `scripts/qa/` | La aplicación compilada sobre una copia aislada de datos, manejada por el inspector de WebKit | `scripts/qa/ui-qa.sh <escenario>` |

## La verificación completa

```bash
pnpm verify                                  # lint, tipos, interfaz, guardianes, búsqueda
cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

En la CI se ejecuta igual en Linux, limitada a los núcleos del ejecutor; en local conviene reproducirlo con
`taskset -c 0-3` para ver lo que solo falla en una máquina lenta.

## Pruebas con la aplicación real (`scripts/qa/`)

`ui-qa.sh` no compila nada: usa el binario de `src-tauri/target/release/`. Prepara una **copia aislada** de los datos
(la base copiada con `sqlite3 .backup` y la música por enlaces duros, en solo lectura), abre la aplicación dentro de
un KWin virtual con el audio a un sumidero nulo y la maneja por el inspector remoto de WebKit (`inspector.mjs`).
Cada escenario (inicio, biblioteca, catalogación, agregados, experiencias, funciones, mashups, móvil…) deja una línea
de resultado. Nunca toca los datos reales de quien la ejecuta.

## Datos de prueba

* Las pruebas de Rust usan bases en memoria o directorios temporales.
* El contrato móvil usa un escenario inventado (`tests/contrato_movil/escenario.json`).
* Las pruebas que necesitan música real la leen de `NS_CORPUS_MUSICA` y se saltan solas si no está.
* El simulador de 100.000 pistas conserva la variedad de una biblioteca real (títulos, duraciones, portadas y
  formatos distintos).
