# AGENTS.md: operating contract for automated contributors

This file is the working contract of the repository for any coding agent (Claude, Gemini, Codex, Cursor, Copilot or
others) and for humans who want the same rules in one place. It applies **without anyone having to repeat it**.
Tool-specific configuration is never versioned in this repository and must never duplicate or contradict this file.

The project documentation is written in Spanish; this file, the root community files and the issue templates are in
English. Code identifiers and comments are in Spanish.

---

## 1. The rule that governs all others

**`docs/` is the single source of truth.** If the code and `docs/` disagree, there is a defect: either the code is
fixed or the document is changed through the decision process. A contradiction is never left alive.

A change with architectural or user-facing impact **is not finished** until the affected document reflects it, with
its status label up to date, in the same change (Constitution, rule 8).

## 2. Before writing code, in this order

| Step | Document | What to look for |
|---|---|---|
| 1 | [Constitution](docs/architecture/constitution.md) | The eight rules. Nothing may break them |
| 2 | [ADR index](docs/adr/README.md) | Why things are the way they are |
| 3 | [Architecture](docs/architecture/README.md) | Contracts of the module you are about to touch |
| 4 | [Product philosophy](docs/design/product-philosophy.md) | Mandatory when the change touches product or interface |

If the work depends on a decision that has not been taken, **do not start**: raise the decision first.

## 3. Repository architecture

```text
docs/              Single source of truth (Spanish). Start at docs/README.md
  adr/             Architecture Decision Records (000-template.md, 001-...)
  architecture/    Constitution, system overview, audio engine, storage, IPC, catalogue, mobile contract,
                   DJ, mashups, NB Launcher, network, i18n, budgets, dependency policy
  audit/           Privacy, threat model, supply chain, performance
  design/          Product philosophy, UI conventions, branding
  developer/       Setup, code structure, standards, testing, CI, packaging, releases
  institution/     Relationship with the institutional offering
  legal/           Licensing, model licences, notices, trademark, data protection, security policy
  user/            End-user guide
  assets/          Branding, screenshots and video
src/               Interface: SvelteKit 2 + Svelte 5, static adapter
  lib/core/ipc/    The only access point to the runtime
  lib/bindings/    Types generated from Rust with ts-rs. Never edited by hand
  lib/i18n/        Spanish, English and Portuguese dictionaries
  lib/ui/          Components and views
src-tauri/         Runtime: Rust + Tauri 2
  src/audio/       The engine: decoder, gapless source, equaliser, limiter
  src/reproductor/ The playback session
  src/db/          Persistence; migrations/ holds the embedded SQL
  src/catalogo/    Import, identification, tagging, organisation, review queue
  src/dj/          Private DJ
  src/mashup/      DJ Mashups
  src/movil/       Server for NB Sound Mobile (frozen protocol 1)
  src/net.rs       **The only network exit.** Nothing leaves through any other path
  src/telemetry.rs Local logging with mandatory redaction. No remote telemetry exists
  src/secrets.rs   Secrets go to the system keyring, never to settings, logs or the database
  vendor/nb-shared Sealed export of the libraries shared with NB Launcher. Never edited by hand
  tests/           Integration tests, including the mobile-contract golden responses
scripts/           Guardians, native binaries, packaging, notices, provenance and real-app QA
```

## 4. Commands

| Command | Purpose |
|---|---|
| `pnpm install` | Dependencies and Git hooks (`.githooks`) |
| `pnpm binarios` | `fpcalc` and ONNX Runtime, with their SHA-256 checked |
| `pnpm tauri dev` | Run the desktop application |
| `pnpm verify` | Lint, types, interface tests, guardians and the search budget |
| `pnpm guardianes` | Only the guardians |
| `pnpm build` | Build the interface the runtime embeds (required before `cargo`) |
| `scripts/empaquetar-linux.sh` | Linux packages inside Ubuntu 22.04 |

## 5. Non-negotiable rules

| Rule | Mechanism |
|---|---|
| Nothing to install outside the application; optional pieces are one-click add-ons | Packages built with everything inside; `check-plataforma` rejects lookups in `PATH` |
| No Python interpreter at runtime | `check-sin-python` |
| One product, no modes and no diagnostic views | [ADR-007](docs/adr/007-information-architecture.md) |
| No error travels as a string | `check-error-contract` |
| Bridge types are generated | `ts-rs` + `check-bindings` |
| No component calls the bridge outside `#lib/core/ipc` | Biome `noRestrictedImports` |
| No bridge command is synchronous or runs on the window thread | `check-comandos` |
| Every collection crossing the bridge is bounded | `check-listas` |
| Every external path is canonicalised and bounded | `check-rutas` |
| Every write transaction opens `BEGIN IMMEDIATE` | `check-transacciones` |
| The webview never plays audio; audio belongs to the Rust engine | `check-ui`, [ADR-003](docs/adr/003-audio-engine.md) |
| Nothing leaves the device outside `net.rs` or without consent | `check-network` |
| No log contains paths, titles, artists or keys | `check-registro` |
| Secrets go to the system keyring | `secrets.rs` with `keyring` |
| No visible text outside the dictionaries; every key in the three languages | `check-i18n`, `Forma<Textos>` in `pnpm check` |
| No regression of the mobile contract | Golden responses in `cargo test` |
| No native binary or model enters the repository | `.gitignore` and pre-commit hook |
| No literal colours, blur on permanent chrome, `will-change` or animation at rest | `check-ui` |
| No platform-bound code without declaring it | `check-plataforma` |
| No new dependency without justification and an admitted licence | [dependency policy](docs/architecture/dependency-policy.md), `cargo deny` |

## 6. Privacy and zero telemetry

* **No telemetry, analytics, crash reporting or usage tracking** of any kind, including "anonymous" or opt-out
  variants.
* **Every outgoing connection goes through `src-tauri/src/net.rs`**, which checks the global switch, the per-service
  consent and the allowed hosts. A new reason to contact the network is a new service with its own consent text.
* **Secrets go to the system keyring**, never to settings, logs or the database.
* **User content never reaches logs**: paths, titles, artists, albums and keys are redacted or measured.
* **No account or sign-in** is required by any feature.
* A real music library used for testing is read through `NS_CORPUS_MUSICA`, read-only; nothing derived from it
  enters the repository or the logs, and tests that need it skip themselves when it is unset.

## 7. Verification

Run before declaring anything finished, without being asked:

```bash
pnpm verify
cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings   # warnings are errors
cargo test
```

`cargo` needs the built interface (`pnpm build`) and the native binaries (`pnpm binarios`) first. Code that only
exists under `cfg(target_os = ...)` must not become dead code on the other platforms: with `-D warnings` it is an
error in their CI.

Definition of done:

1. The acceptance criterion is met and another person can check it.
2. Tests for the added logic, and one regression test per fixed defect, named after the property they verify.
3. Verification is green.
4. If the change crosses the bridge, the generated bindings are current.
5. If it has architectural or user-facing impact, `docs/` is updated in the same change.
6. If it adds visible text, it exists in Spanish, English and Portuguese.

## 8. Keeping `docs/` in sync

* Each fact has one owning document; others link to it.
* Every document declares its status on its first lines (**Implementado**, **Aceptado**, **Aplicable**).
* The structure of `docs/` is fixed: `adr`, `architecture`, `audit`, `design`, `developer`, `institution`, `legal`,
  `user`, `assets`.
* `node scripts/check-docs.mjs` rejects broken links, empty or duplicated files, emoji, machine-specific absolute
  paths, missing titles, missing status and broken tables.
* Documentation describes the product as it is. Development history, personal names and internal anecdotes do not
  belong in it; the reasoning behind a decision does, in impersonal form.

## 9. Language and style

* Spanish for code identifiers, comments, documentation in `docs/` and commit messages; English for the root
  community files and this file. Professional register, **no emoji** anywhere.
* Comments explain **why**, not what. When a decision is not obvious, the comment says what was discarded and why.

## 10. Commits

```bash
git commit -s -m "feat(reproductor): cola sin cortes entre pistas"
```

* Conventional Commits; the `commit-msg` hook rejects anything else.
* `-s` is mandatory: Developer Certificate of Origin.
* The body explains **why** and what was verified, not the list of files.

## 11. What to do without asking

* Run verification; fix formatting, lints and failing tests.
* Write tests for added logic and a regression test for each fixed defect.
* Update the affected document and its status label.

## 12. What requires asking

* Approving or changing an ADR, or contradicting the Constitution.
* Adding a dependency or changing a licence.
* Any change to NB Sound Mobile or to its protocol.
* Publishing, pushing to a remote, or any outward-facing action.
* Any change that sends data off the device.

## 13. One agent on the working tree

**Two agentic tools must never write to this repository at the same time.** If modifications appear that you do not
recognise, compare with `HEAD` (`git status --short`, `git show HEAD:<path>`) and restore from it; never trust the
copy on disk.
