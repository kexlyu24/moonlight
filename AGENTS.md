# AGENTS.md — instructions for AI coding agents

You are working on **Moonlight**, a lightweight desktop web browser written in Rust on top of the Servo engine, with support for Manifest V3 Chrome extensions.

## Read first

Before writing any code, read the documents in `docs/` in this order:

1. `docs/01-product-requirements.md` — scope, priorities (P0/P1/P2), functional requirements (FR-xx), non-functional requirements (NFR-xx)
2. `docs/02-technical-architecture.md` — ADR-001 (Servo), components, engine hooks H1–H8, Cargo workspace layout, security principles
3. `docs/03-chrome-extension-spec.md` — extension runtime and the `chrome.*` API matrix
4. `docs/04-design-ui-spec.md` — design tokens and screens
5. `docs/05-roadmap.md` — phases F0–F4 and their exit criteria
6. `docs/06-engineering-handbook.md` — setup, conventions, CI, Definition of Done, first steps checklist

If a task conflicts with these documents, stop and ask the maintainer instead of guessing.

## Current phase

**F0 — Foundation & spike.** Work from the "First steps checklist" in `docs/06-engineering-handbook.md`. Do not start work from later phases unless the maintainer asks.

## Hard rules

- **Language:** everything in the repo is English: code, comments, commit messages, PRs, issues, docs, and UI strings.
- **Platform:** Windows 10/11 x64 only until the end of Alpha (F2). Write portable Rust anyway; avoid Windows-only APIs unless they are required.
- **Engine boundary:** only `crates/ml-engine-servo` may depend on the `servo` crate. Everything else talks to the engine through the `EngineBackend` trait in `crates/ml-engine`.
- **UI strings:** all user-facing text goes through Fluent (`crates/ml-i18n`, en-US). No hard-coded strings.
- **Extensions:** Manifest V3 only. Every `chrome.*` call is permission-checked in Rust, never only in the JS shim.
- **License:** MPL-2.0. New dependencies must have licenses compatible with MPL-2.0 (`cargo deny check` enforces this).
- **No `unwrap()` / `expect()`** on paths that handle untrusted input (manifests, CRX files, ZIPs, DNR rules, URLs, network data).

## Security-sensitive areas

Changes in these areas must be small, fully tested, and clearly flagged in the PR description so the maintainer can read them line by line:

- CRX3 verification and ZIP unpacking (`crates/ml-ext`)
- `PermissionGate` and host permission checks (`crates/ml-ext`, `crates/ml-ext-api`)
- The JS ↔ Rust bridge (hook H5)
- The updater and signature verification (`crates/ml-update`)
- Manifest and declarativeNetRequest rule parsers

## How to work

1. Take one small task at a time, on its own branch (`feat/...`, `fix/...`).
2. Write tests together with the code. Code without tests is not done.
3. Run before finishing a task:
   ```bash
   cargo fmt --all
   cargo clippy --workspace -- -D warnings
   cargo nextest run --workspace
   cargo deny check
   ```
4. Use Conventional Commits (`feat(ext): support chrome.alarms`).
5. Keep PRs under about 400 changed lines.
6. When you make an architecture decision, write an ADR in `docs/adr/NNN-title.md`.
7. When you add a `chrome.*` API, update the API matrix in `docs/03-chrome-extension-spec.md`.

## Definition of Done

A task is done only when it meets the Definition of Done in `docs/06-engineering-handbook.md`.
