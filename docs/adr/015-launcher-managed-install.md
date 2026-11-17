# ADR-015: Instalación administrada por NB Launcher

**Fecha:** 2026-11-12
**Estado:** Aceptado

## Contexto

NB Launcher puede instalar una copia de NB Sound por usuario en Linux. Hay que evitar que el actualizador propio de
NB Sound cambie los binarios a la vez que el gestor, y que NB Sound encuentre sus recursos dentro de esa copia.

## Alternativas consideradas

1. **Dos actualizadores para la misma instalación.** Rechazada: carreras y autoridad ambigua.
2. **Guardar los datos de NB Sound dentro de NB Launcher.** Rechazada: los datos son de NB Sound.
3. **Decidir la autoridad por una variable de entorno.** Insuficiente al abrir el binario fuera de NB Launcher.
4. **Una marca de instalación administrada junto al ejecutable y un bloqueo compartido.** Aceptada.

## Decisión

Una copia administrada en Linux tiene la estructura `nb.sound/<sha256>/usr/bin/nb-sound` y una marca
`.nb-launcher-managed` junto al ejecutable, con el contenido `nb-launcher-managed-v1`. NB Sound comprueba las dos
cosas antes de abrir sus datos; si la marca existe pero no es válida, no asume una instalación independiente ni
actualiza nada por su cuenta.

El proceso mantiene un bloqueo compartido sobre `nb.sound/running.lock` mientras vive; el gestor necesita el
bloqueo exclusivo para cambiar los binarios. NB Sound carga ONNX Runtime de `usr/lib/NB Sound` de su misma versión.
En una copia administrada, buscar actualizaciones responde `INSTALLATION_MANAGED` antes de cualquier petición de red,
y la interfaz dice "Esta instalación se actualiza desde NB Launcher".

## Consecuencias

- **Positivas:** NB Launcher controla sus binarios y NB Sound conserva sus datos, sus migraciones y su dominio.
  Quitar una instalación no borra la biblioteca ni las copias.
- **Negativas:** el formato existe solo en Linux.
- **Riesgos y cómo se vigilan:** las pruebas comprueban la estructura, los recursos y la exclusión mutua.
