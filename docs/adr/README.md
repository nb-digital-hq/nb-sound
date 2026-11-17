# Registro de decisiones arquitectónicas (ADR)

> **Estado:** Aplicable.

Un ADR documenta una decisión estructural: su contexto, las alternativas evaluadas, lo decidido y sus consecuencias.
Su valor no está en la decisión, sino en conservar **por qué** se tomó.

## Reglas

* Los ADR son **inmutables**. Una decisión que cambia no se edita: se escribe un ADR nuevo que supera al anterior, y
  el anterior se marca como superado con un enlace al nuevo.
* Un ADR sin alternativas rechazadas es un comunicado, no una decisión.
* Si una decisión será difícil de revertir dentro de seis meses, necesita un ADR.

## Índice

| ADR | Título | Estado |
|---|---|---|
| [001](001-tech-stack.md) | Pila tecnológica | Aceptado |
| [002](002-no-python.md) | Ningún intérprete de Python en tiempo de ejecución | Aceptado |
| [003](003-audio-engine.md) | Motor de audio embebido en Rust | Aceptado |
| [004](004-data-ownership.md) | Rust es dueño de los datos | Aceptado |
| [005](005-mobile-contract.md) | El contrato con NB Sound Mobile queda congelado | Aceptado |
| [006](006-add-ons.md) | Agregados que se instalan con un clic | Aceptado |
| [007](007-information-architecture.md) | Arquitectura de información | Aceptado |
| [008](008-design-system.md) | Sistema de diseño | Aceptado |
| [009](009-settings-and-network.md) | Configuración mínima, secretos en el llavero y una sola puerta de red | Aceptado |
| [010](010-licensing-model.md) | Modelo de licencias | Aceptado |
| [011](011-internationalization.md) | Tres idiomas, con cambio al momento | Aceptado |
| [012](012-party-mode.md) | Modelo de confianza y aislamiento del Modo fiesta | Aceptado |
| [013](013-launcher-shared-frequency.md) | La frecuencia de las copias como capacidad compartida con NB Launcher | Aceptado |
| [014](014-launcher-link-renewal.md) | Renovación privada del enlace con NB Launcher | Aceptado |
| [015](015-launcher-managed-install.md) | Instalación administrada por NB Launcher | Aceptado |
| [016](016-launcher-catalog.md) | Catálogo firmado para NB Launcher | Aceptado |
| [017](017-launcher-persistent-authorization.md) | Autorización persistente y local de NB Launcher | Aceptado |
| [018](018-dj-mashups.md) | DJ Mashups, un compositor automático fuera de línea | Aceptado |

Plantilla: [000-template.md](000-template.md).
