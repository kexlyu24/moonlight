# Security Policy

## Supported Versions

Moonlight is currently under active pre-release development. Security updates and bug fixes are applied to the `main` branch.

| Channel / Version | Supported | Notes |
| ----------------- | --------- | ----- |
| `main` (Nightly)  | Yes       | Active development branch |
| Pre-release < 0.1 | Yes       | Current phase (F0 / F1) |

## Reporting a Vulnerability

We take the security of Moonlight seriously. If you discover a vulnerability or security flaw, please report it privately rather than opening a public issue.

To report a vulnerability:
1. Send an email to `[INSERT CONTACT EMAIL]` with the subject line `[SECURITY] Vulnerability Report: <Brief Description>`.
2. Include detailed information about:
   * The type of issue (e.g., memory corruption, sandbox escape, permission bypass, parser crash).
   * Steps to reproduce the issue, including proof-of-concept manifests, scripts, or files if applicable.
   * Affected components, crates, or commit hash.
   * Any potential impact or attack vectors.

Maintainers will acknowledge receipt of your report, provide an initial assessment, and keep you informed as a patch is developed and verified. Please allow reasonable time for investigation before disclosing any details publicly.

## Security-Sensitive Areas

The following components handle untrusted input and enforce security boundaries; modifications require rigorous testing and review:

* **CRX3 Verification & Extraction (`crates/ml-ext`):** Protobuf header parsing, RSA/Ed25519 signature checks, public-key ID derivation, and zip slip / directory traversal prevention.
* **Permission Gate (`crates/ml-ext`, `crates/ml-ext-api`):** Rust-enforced checks for API permissions, host permissions, and user site restrictions.
* **JS ↔ Rust Bridge (Hook H5):** Context-validated asynchronous IPC between web contexts and the browser backend.
* **Auto-Updater (`crates/ml-update`):** Cryptographic verification of update packages via Ed25519 signatures.
* **Rule and Manifest Parsers:** Strict input handling without panics (`unwrap()` / `expect()`) for `manifest.json` and declarativeNetRequest rulesets.

## Upstream Security Patches

Per NFR-09, we aim to re-release engine security patches (Servo / SpiderMonkey) within ≤ 14 days of upstream releases.
