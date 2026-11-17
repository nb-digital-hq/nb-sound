## Problem

<!-- Link the issue or the ADR. A pull request without context cannot be reviewed properly. -->

Closes #

## Approach

<!-- One or two sentences about the approach, not a summary of the diff. -->

## How to verify

<!-- Manual steps, or the name of the added test. Another person must be able to check it. -->

---

## Definition of done

See [`AGENTS.md`](../blob/main/AGENTS.md), section "Verification".

- [ ] The acceptance criterion is met and can be checked by another person
- [ ] There are tests for the added logic, and one regression test per fixed defect
- [ ] `pnpm verify` plus the Rust checks pass
- [ ] If the change crosses the bridge, the generated bindings are up to date
- [ ] If it has architectural or user-facing impact, `docs/` was updated in this same pull request
- [ ] If it adds visible text, it exists in Spanish, English and Portuguese
- [ ] Commits are signed off with `git commit -s` (DCO)

## Impact

- [ ] Adds or changes dependencies <!-- justify per docs/architecture/dependency-policy.md -->
- [ ] Changes the database schema <!-- include a migration and a migration test -->
- [ ] Changes the NB Sound Mobile protocol <!-- only additive changes are accepted -->
- [ ] Affects personal data or anything that leaves the device
- [ ] None of the above
