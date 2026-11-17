# ADR-016: Catálogo firmado para NB Launcher

**Fecha:** 2026-11-12
**Estado:** Aceptado

## Contexto

NB Launcher verifica los paquetes de NB Sound, pero necesita descubrir una versión nueva sin actualizarse él mismo, y
el `latest.json` del actualizador no firma todos sus campos. Hay que autenticar propietario, versión, plataforma,
tamaño, huella y contrato de cada artefacto.

## Alternativas consideradas

1. **Confiar en HTTPS o en el manifiesto existente.** Rechazada: no autentica todos los datos.
2. **Entregar la clave privada a NB Launcher.** Rechazada.
3. **Una clave nueva para el mismo propietario.** Rechazada: duplica distribución y rotación sin separar autoridad.
4. **Un catálogo propio, firmado con la clave editorial que ya firma los paquetes.** Aceptada.

## Decisión

Cada publicación lleva `launcher-catalog.json` y su firma minisign. El esquema 1 identifica solo `nb.sound`, su
repositorio, una versión estable, la emisión, la caducidad (14 días) y sus siete artefactos. El productor
(`src-tauri/tools/publicar_catalogo.rs`, un ejemplo de Cargo fuera del instalador) comprueba las siete firmas y
tamaños sobre los bytes reales antes de generar los datos. La CI firma el resultado con el secreto editorial y
verifica la firma antes de subirlo; una renovación semanal refresca la caducidad sin tocar los paquetes.

## Consecuencias

- **Positivas:** NB Launcher descubre versiones nuevas con todos sus datos autenticados.
- **Negativas:** la renovación depende de que el flujo programado siga activo.
- **Riesgos y cómo se vigilan:** las pruebas del productor rechazan paquetes alterados, firmas ajenas, inventarios
  incompletos y versiones inválidas. El consumidor se protege contra la reversión de versión.
