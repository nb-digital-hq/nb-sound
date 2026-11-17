# Cadena de suministro

> **Estado:** Implementado. Licencias y avisos de las dependencias (`cargo deny`, `generate-notices`), binarios
> incluidos con huella fijada (`scripts/fetch-binarios.mjs`), agregados con huella compilada (`agregados.rs`),
> actualizaciones firmadas (`net.rs`) y publicaciones con `SHA256SUMS` firmado (`scripts/procedencia.mjs`).

Una aplicación con actualizador es un vector de distribución privilegiado: comprometer su construcción permitiría
entregar código arbitrario, firmado como legítimo, a todas las instalaciones.

## 1. Activos críticos

| Activo | Impacto si se compromete |
|---|---|
| Clave de firma del actualizador (minisign) | Actualizaciones aceptadas como oficiales |
| Clave de publicación de NB Digital (Ed25519) | `SHA256SUMS.sig` aceptado como oficial |
| Cuenta con permisos de publicación en `nb-digital-hq/nb-sound` | Alteración del código, de los flujos y de las publicaciones |

## 2. Claves

* **Ninguna clave privada entra en el repositorio.** El gancho de pre-commit y `.gitignore` rechazan claves y
  binarios; lo único que se versiona son las claves públicas: la del actualizador en `src-tauri/tauri.conf.json` y la
  de publicación en `docs/architecture/api/clave-de-publicacion.pem`.
* La clave privada del actualizador vive en los secretos del repositorio (`TAURI_SIGNING_PRIVATE_KEY` y su
  contraseña) y en la máquina de quien publica; solo la usa el flujo de publicación.
* La clave de publicación es la de NB Digital, un archivo PKCS#8 con permisos `0600` custodiado fuera del
  repositorio. Ningún flujo de integración continua la usa: las sumas se firman en local.
* La clave pública del actualizador viaja compilada en la aplicación: una actualización sin su firma no se instala.

## 3. Dependencias

* **Licencias:** `cargo deny check licenses` contra la lista de `deny.toml`, y `scripts/generate-notices.mjs`, que
  escribe [THIRD-PARTY-NOTICES.md](../../THIRD-PARTY-NOTICES.md) a partir de `cargo metadata` y de las dependencias de
  producción de npm, y falla si alguna licencia no está admitida.
* **Avisos de seguridad:** `cargo deny check advisories` (RustSec) en la CI y `pnpm audit`; Dependabot vigila los
  avisos de GitHub y **no abre propuestas de versión**: cada actualización se hace a mano y pasa la verificación
  completa.
* **Archivos de bloqueo versionados**; ninguna dependencia flotante. `deny.toml` declara `yanked = "deny"` y
  `unknown-registry = "deny"`.
* Las bibliotecas compartidas con NB Launcher llegan como exportación sellada con su manifiesto de huellas
  (`check-shared`).

### 3.1 Avisos conocidos sin acción posible

| Aviso | Dónde | Por qué no aplica o no se puede cerrar |
|---|---|---|
| `libsqlite3-sys` (RUSTSEC-2026-0459) | `rusqlite` 0.40 | El arreglo pide `rusqlite` 0.41 y `r2d2_sqlite` 0.35, el último publicado, sigue atado a 0.40 |
| `proc-macro-error` sin mantenimiento (RUSTSEC-2024-0370) | Árbol de Tauri | No es dependencia directa; no hay versión a la que saltar |
| `zvariant`, `zvariant_derive`, `zbus_names` sin mantenimiento | `zbus`, vía `keyring` | Se fundieron en `zbus` 6; `keyring` aún depende de `zbus` 5 |
| `paste` sin mantenimiento (RUSTSEC-2024-0436) | `lofty` | Macro de compilación que no llega al binario |
| `chrono` sin mantenimiento (RUSTSEC-2026-0543) | `trash` | Solo lee las fechas de la papelera del sistema; no es una vulnerabilidad y `trash` no ofrece versión sin él |
| `glib` < 0.20 (GitHub, moderada) | Pila de GTK 3 de Tauri | `gtk` 0.18 y `webkit2gtk` fijan `glib` a 0.18; NB Sound no llama a `glib` |
| `cookie`, `deepmerge` y `@sveltejs/kit` (GitHub, bajas) | Herramientas de construcción de la interfaz | No viajan en el producto: la aplicación sirve un `build/` estático, sin servidor ni cookies |

No se fuerzan versiones por debajo de un marco de trabajo (`[patch]` en Cargo, `overrides` en pnpm): cambiaría un
riesgo conocido por uno desconocido.

## 4. Binarios incluidos

`fpcalc` (Chromaprint) y ONNX Runtime para CPU se descargan de sus publicaciones oficiales con su **SHA-256 fijado**
en `scripts/binarios.json` y viajan dentro del instalador. Nunca entran al repositorio. Si la huella no coincide, la
descarga se rechaza. ONNX Runtime sale del paquete oficial de Microsoft en NuGet (`Microsoft.ML.OnnxRuntime`), que
trae las bibliotecas de las tres plataformas; la de Linux es idéntica byte a byte a la de las releases de GitHub.

## 5. Agregados

Cada agregado se publica como archivo de la publicación 1.0.0 de `nb-digital-hq/nb-sound`. Su URL, su tamaño y su
SHA-256 están compilados en el binario (`agregados.rs`): la aplicación descarga, verifica la huella y solo entonces
instala.

## 6. Procedencia de las publicaciones

Cada publicación incluye los instaladores de las tres plataformas, los artefactos firmados del actualizador,
`latest.json`, el catálogo de NB Launcher con su firma, `THIRD-PARTY-NOTICES.md` y:

* `SHA256SUMS`, con la huella de todos los archivos;
* `SHA256SUMS.sig`, su firma **Ed25519** con la clave de publicación de NB Digital.

Cómo verificarlo está en [publicación](../developer/releases.md).
