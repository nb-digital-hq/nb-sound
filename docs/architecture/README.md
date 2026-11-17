# Arquitectura

> **Estado:** Implementado.

Cómo está construido NB Sound y qué contratos rigen cada parte. Las decisiones y su porqué están en los
[ADR](../adr/README.md); las reglas que no se negocian, en la [Constitución](constitution.md).

| Documento | Qué cubre |
|---|---|
| [Constitución](constitution.md) | Las ocho reglas y el mecanismo que hace cumplir cada una |
| [Visión general del sistema](system-overview.md) | Las piezas, cómo se hablan y dónde vive cada responsabilidad |
| [Motor de audio](audio-engine.md) | Decodificación, mezcla, cola sin cortes, karaoke, ecualizador, sonoridad y controles del sistema |
| [Almacenamiento](storage.md) | La base SQLite, el esquema por migraciones, las rutas en disco y la copia de seguridad |
| [Contratos del puente](ipc-contracts.md) | Comandos, eventos, tipos generados y el contrato de errores |
| [Catalogación](catalog.md) | De un archivo a una pista con sus datos: validar, identificar, etiquetar, organizar, Por revisar |
| [Contrato móvil](mobile-contract.md) | El servidor para NB Sound Mobile y las respuestas doradas |
| [DJ Privado](dj.md) | Intención, planificación y mezcla de una sesión |
| [DJ Mashups](mashups.md) | Análisis estructural, separación, composición y render de un mashup |
| [Integración con NB Launcher](launcher-integration.md) | Frecuencia compartida, autorización, instalación administrada y catálogo |
| [Red](network.md) | La puerta única, los servicios, sus anfitriones y el consentimiento |
| [Internacionalización](i18n.md) | Los diccionarios, el cambio al momento y lo que nombra el runtime |
| [Presupuestos de rendimiento](performance-budgets.md) | Lo que se mide y el límite de cada cosa |
| [Política de dependencias](dependency-policy.md) | Qué hace falta para añadir una dependencia |
| [Glosario](glossary.md) | Los términos del proyecto |
