# Security Policy

## Reporting a vulnerability

**Do not open a public issue.** A public issue is a full disclosure before a fix exists.

Use GitHub's private vulnerability reporting: in this repository, **Security**, **Report a vulnerability**. You can
also send the report by email to **nbdigitalsas@gmail.com** with a subject that starts exactly like this:

```text
[NB Sound][Seguridad] <one-line summary>
```

The mailbox is shared by every NB Digital project and is filtered by that prefix; a report without it is very likely
not to be read.

Please include:

1. A description of the problem and the affected component.
2. Steps to reproduce, or a proof of concept.
3. The estimated impact (confidentiality, integrity, availability, privacy).
4. The version (Settings, About) or commit hash.

## What to expect

| Step | Commitment |
|---|---|
| Acknowledgement | Within 7 days |
| Public disclosure | The reporter may disclose 90 days after acknowledgement, fixed or not |
| Assessment and severity | Communicated to the reporter |
| Fix | Developed privately |
| Publication | Fixed release and security advisory, with credit if desired |

The full procedure is in [`docs/legal/security-policy.md`](docs/legal/security-policy.md) (Spanish).

## Scope

**In scope:**

* The desktop application: runtime (Rust), interface (Svelte) and the bridge between them.
* The network gate (`src-tauri/src/net.rs`) and any path that could send data off the device without consent.
* The local servers: the NB Sound Mobile server (HTTPS, WSS, mDNS) and the Party Mode server.
* The NB Launcher link and its persistent authorisations.
* Parsing of untrusted input: audio files, tags, cover images, responses from online services, backups.
* The updater, the add-on downloads, and the build, packaging, signing and release infrastructure.

**Out of scope:** third-party services the application connects to, operating-system software, and deployments
modified by the user.

The threat model, including the limits the project declares it does **not** cover, is in
[`docs/audit/threat-model.md`](docs/audit/threat-model.md).

## Cryptographic scope

| Use | Mechanism |
|---|---|
| Release integrity | `SHA256SUMS` over every artefact, signed with **Ed25519** (`SHA256SUMS.sig`) |
| Updates | Every package signed with minisign; the application verifies against its compiled public key |
| Add-ons | URL, size and SHA-256 compiled into the binary; verified before installing |
| Mobile link | TLS with a persistent self-signed certificate pinned by the phone; per-device tokens stored as hashes |
| Stored credentials | Operating-system keyring; never in settings, database or logs |
| Data at rest | Not encrypted by the application: it relies on the operating system's disk encryption |

How to verify a release is described in [`docs/developer/releases.md`](docs/developer/releases.md).

## Supported versions

Security fixes are released for the latest published version.
