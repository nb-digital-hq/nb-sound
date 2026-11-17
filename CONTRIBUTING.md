# Contributing to NB Sound

Thank you for your interest. This file is the entry point; the developer guide, in Spanish, is in
[`docs/developer/`](docs/developer/README.md). If you work with an AI coding agent, point it to [AGENTS.md](AGENTS.md):
it contains the same rules in a machine-friendly form.

## Before writing code

1. Read the [product philosophy](docs/design/product-philosophy.md) and the
   [Constitution](docs/architecture/constitution.md). A technically correct contribution that contradicts them will
   be declined.
2. Look for an existing issue. If there is none, open one before implementing.
3. Changes with architectural impact need an [ADR](docs/adr/README.md) before code.

## Licence and provenance

Your contribution is licensed under the licence of the project ([LICENSE](LICENSE)) and **you keep your copyright**:
there is no copyright assignment and no contributor licence agreement.

The only requirement is the Developer Certificate of Origin (DCO): every commit carries a `Signed-off-by` line.

```bash
git commit -s -m "fix(biblioteca): corregir el orden de la cuadrícula al importar"
```

If you forgot to sign off, `git rebase --signoff origin/main` fixes the commits on your branch.

## Setting up

```bash
pnpm install        # dependencies and Git hooks
pnpm binarios       # fpcalc and ONNX Runtime, checked against their SHA-256
pnpm build          # the interface the runtime embeds
pnpm tauri dev      # run the application
```

Requirements are in [docs/developer/setup.md](docs/developer/setup.md).

## Workflow

1. Fork the repository and create a branch: `feat/short-description` or `fix/short-description`.
2. Make atomic commits following [Conventional Commits](docs/developer/commit-convention.md).
3. Run the full verification before opening the pull request:

   ```bash
   pnpm verify
   cd src-tauri
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test
   ```

4. Open the pull request against `main` and complete the template.

## What is not accepted

* Third-party code without a compatible licence or without attribution.
* Telemetry, analytics, advertising or any data transfer without explicit consent.
* Visible text that is not in the dictionaries, or that is missing in one of the three languages.
* Changes to the NB Sound Mobile protocol that are not purely additive.
* Copyrighted music or covers used as test material.
* New dependencies without a justification of cost, licence and maintenance
  ([dependency policy](docs/architecture/dependency-policy.md)).

## Conduct and security

All interactions are governed by the [Code of Conduct](CODE_OF_CONDUCT.md). **Do not report vulnerabilities as public
issues**; see [SECURITY.md](SECURITY.md).
