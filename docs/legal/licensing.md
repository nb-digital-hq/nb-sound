# Licencias

> **Estado:** Aplicable. Decisión: [ADR-010](../adr/010-licensing-model.md).
> Este documento recoge criterios de ingeniería; no constituye asesoría legal.

## Decisión vigente

| Elemento | Decisión |
|---|---|
| Licencia del código | **Apache License 2.0** |
| Licencia de la documentación | Apache License 2.0 |
| Modelo de NB Digital | Aplicaciones para personas de código abierto; módulos y servicios institucionales con licencia comercial, separados |
| Procedencia de las contribuciones | **DCO** (`Signed-off-by`), sin acuerdo de cesión |
| Aviso de copyright | `Copyright 2026 The NB Sound Authors`; cada contribuidor conserva el suyo |
| Marcas | "NB", "NB Digital" y "NB Sound", fuera de la licencia; ver [marcas](trademark.md) |

`LICENSE` y `NOTICE` están en la raíz del repositorio y viajan en cada paquete.

## Qué significa

* **Para quien usa NB Sound:** puede usarlo, modificarlo, redistribuirlo e incorporarlo en productos propios,
  incluidos los comerciales, conservando el aviso de copyright, la licencia, `NOTICE` y la indicación de los cambios.
  No obtiene derecho a usar el nombre ni el logotipo.
* **Para quien contribuye:** conserva el copyright de su aportación y la licencia bajo Apache 2.0, declarando su
  origen con `git commit -s`.
* **Para una institución:** Apache 2.0 está aprobada por la Open Source Initiative, incluye concesión de patentes y es
  habitual en contratación pública y privada.

## La frontera con lo comercial

| Componente | Licencia |
|---|---|
| NB Sound: aplicación, motor, catalogación, DJ, mashups, servidor para el teléfono | Apache 2.0 |
| Agregados descargables | La de cada modelo ([licencias de los modelos](model-licenses.md)) |
| Módulos y servicios institucionales de NB Digital | Licencia comercial, fuera de este repositorio |

Ningún componente de NB Sound depende de un producto comercial. Un cambio que introduzca esa dependencia se
rechaza.

## Consecuencias operativas

* **Dependencias:** solo licencias permisivas y la MPL-2.0; la GPL y la AGPL están excluidas
  ([política de dependencias](../architecture/dependency-policy.md)).
* **Ejecutables de terceros:** `fpcalc` (LGPL-2.1) viaja como programa aparte, con su licencia y su origen en los
  avisos.
* **Atribución:** [avisos de terceros](third-party-notices.md) y `NOTICE`.
