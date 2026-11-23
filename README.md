# Moonlight

Moonlight is a lightweight desktop web browser written in Rust on top of the Servo engine, with support for Manifest V3 Chrome extensions.

## Status

Moonlight is currently in **Phase F0 — Foundation & spike**. See the [Roadmap](docs/05-roadmap.md) for milestones and phase exit criteria.

## Product Promises

1. **Light, and visibly so:** RAM usage is shown in real-time on the New Tab page and per extension instead of being hidden.
2. **Built-in privacy:** Built-in tracker blocker powered by the `adblock` engine (EasyList + EasyPrivacy filters) with an omnibox shield counter and per-site controls.
3. **Familiar Chrome extensions:** Install Manifest V3 extensions from the Chrome Web Store, `.crx` packages, or unpacked folders.
4. **Honest about compatibility:** Clear compatibility ratings (Full, Partial, or Not supported) on every extension and an informative banner on web pages using unsupported engine features.

## Architecture & Tech Stack

* **Rendering Engine:** Servo (`servo` crate), abstracted behind the `EngineBackend` trait so engine dependencies stay isolated in `crates/ml-engine-servo`.
* **Browser Chrome (UI):** Built in Rust using `winit`, `wgpu`, and `egui`.
* **Internal Pages:** `moonlight://` pages (New Tab, Settings, Extensions, History, Downloads) authored in HTML/CSS/TypeScript and served by the engine.
* **Storage:** SQLite (`rusqlite`) per profile for history, bookmarks, settings, and extension storage.
* **Modular Workspace:** Clean crate boundaries (`ml-shell`, `ml-engine`, `ml-net`, `ml-ext`, `ml-ext-api`, `ml-storage`, `ml-pages`, `ml-i18n`, `ml-update`).

## Getting Started

### Prerequisites

* Rust toolchain pinned via `rust-toolchain.toml`
* Git, Python 3, `cargo-nextest`, `cargo-deny`
* Node.js LTS and `pnpm` (for `web/`)
* Platform build dependencies for Servo (see [Engineering Handbook](docs/06-engineering-handbook.md))

### Basic Commands (Planned)

The following commands are planned once the Cargo workspace is established in Phase F0:

```bash
cargo xtask setup        # verify prerequisites, download assets, build web/ (planned)
cargo run -p moonlight   # run browser with development profile (planned)
cargo nextest run        # run automated test suite (planned)
```

## Documentation

Full architectural and product specifications are maintained in the [`docs/`](docs/README.md) directory:

1. [Product Requirements](docs/01-product-requirements.md)
2. [Technical Architecture](docs/02-technical-architecture.md)
3. [Chrome Extension Specification](docs/03-chrome-extension-spec.md)
4. [Design & UI Spec](docs/04-design-ui-spec.md)
5. [Roadmap](docs/05-roadmap.md)
6. [Engineering Handbook](docs/06-engineering-handbook.md)

Guidelines for AI coding agents and contributors are documented in [AGENTS.md](AGENTS.md).

## License

Moonlight is open-source software licensed under the [Mozilla Public License Version 2.0 (MPL-2.0)](LICENSE).
