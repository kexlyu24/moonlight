# Moonlight — Engineering Handbook

Last updated: 2026-11-23

Related documents: [Product Requirements](01-product-requirements.md) · [Technical Architecture](02-technical-architecture.md) · [Chrome Extension Specification](03-chrome-extension-spec.md) · [Roadmap](05-roadmap.md)

## Development environment setup

Everyone uses the same Rust toolchain (pinned in `rust-toolchain.toml`) plus Servo's per-OS build prerequisites; Servo's prerequisites change over time, so the main reference is the [embedding guide in the Servo Book](https://book.servo.org/embedding/overview.html).

| OS | Prerequisites |
| --- | --- |
| All | `rustup`, Git, Python 3, `cargo-nextest`, `cargo-deny`, Node.js LTS + `pnpm` (for `web/`) |
| Windows 10/11 | Visual Studio Build Tools (MSVC, Windows SDK), LLVM/Clang |
| Linux | From F3: Clang/LLVM, `pkg-config`, X11, Wayland, GStreamer, fontconfig, and freetype development headers (package names per distro are in the Servo Book) |
| macOS | From F3: Xcode Command Line Tools, Homebrew |

```bash
git clone https://github.com/<org>/moonlight.git
cd moonlight
cargo xtask setup        # check prerequisites, download assets, build web/
cargo run -p moonlight   # run the browser with a dev profile
cargo nextest run        # all tests
```

The first build is slow because the engine and SpiderMonkey are compiled too; enable a cache (`sccache`) locally and in CI. The dev profile lives in `./target/profile-dev/` so it never mixes with a regular Moonlight install.

## Repo, branch, commit, and CI conventions

Trunk-based development: `main` always builds and ships as Nightly; changes land through small PRs that pass CI and review.

- **Branches:** `main` + short feature branches (`feat/omnibox-suggest`, `fix/crx-zip-slip`) that live at most 1 week. Big features hide behind feature flags.
- **Commits:** Conventional Commits (`feat(ext): support chrome.alarms`). Everything in the GitHub repo is in English: commit messages, PRs, issues, README, CONTRIBUTING, ADRs, and code comments.
- **PRs:** at most ±400 changed lines. Before merging, ask the AI to review the diff, then run the self-review checklist: tests exist and pass, no `unwrap()` on untrusted-input paths, and any PR touching `ml-ext` permissions, CRX, or the updater is read manually line by line.
- **Versioning:** SemVer, 0.x until 1.0. Release channels: Nightly (every night), Beta (every 4 weeks), Stable.
- **ADRs:** architecture decisions go in `docs/adr/NNN-title.md` (context, options, decision, consequences).

**CI pipeline** (GitHub Actions; Windows only until Alpha, Linux and macOS added at the start of F3)

1. `cargo fmt --check` and `cargo clippy --workspace -- -D warnings`.
2. `cargo deny check` (dependency licenses must be compatible with MPL-2.0, security advisories, duplicate dependencies).
3. `cargo nextest run --workspace`.
4. Build `web/` + lint TypeScript.
5. Headless smoke test: open 10 core sites and 5 core extensions (a short version of the full suite).
6. RAM and startup benchmarks; fail on a regression > 10% versus `main`.
7. Nightly: full suite of 50 sites + 20 extensions, 1 hour of fuzzing per target, signed Nightly installer build.

## Testing strategy and Definition of Done

Moonlight is tested in five layers; the last two (real sites and real extensions) measure product progress and are published on a dashboard.

| Layer | What is tested | Tools | When |
| --- | --- | --- | --- |
| Unit | Manifest parser, CRX verification, DNR rule matching, permissions, DB migrations | `cargo nextest`, `proptest`, `insta` | Every PR |
| Integration | `chrome.*` APIs end to end through a headless WebView | `tests/` harness + engine pixel readback | Every PR |
| Fuzzing | Untrusted input: manifests, CRX3 headers, ZIPs, DNR rules, omnibox URLs | `cargo-fuzz` | Nightly |
| Real sites | 50 test sites: load, no fatal JS errors, main flow clickable | Automated scenarios + screenshots | Nightly |
| Real extensions | 20 test extensions (see the [Chrome Extension Specification](03-chrome-extension-spec.md)) | Per-extension scenarios | Nightly |

**Composition of the 50 test sites** (full list compiled at the start of F0 in `tests/sites/sites.toml`)

| Category | Count | Examples |
| --- | --- | --- |
| Documentation & developer | 12 | GitHub, docs.rs, MDN, Stack Overflow, crates.io |
| Search & portals | 6 | DuckDuckGo, Google Search, Wikipedia |
| News & reading | 8 | Hacker News, Kompas, Detik |
| Video & media | 6 | YouTube, Twitch, Spotify Web |
| Productivity | 8 | Gmail, Google Docs, Notion, Figma (view only) |
| Shopping & local services | 6 | Tokopedia, Shopee, Gojek web |
| Social | 4 | X, Reddit, LinkedIn |

**Definition of Done** for every feature PR:

- [ ] Meets the acceptance criteria of the related FR in the PRD
- [ ] New unit/integration tests; CI green on every OS supported in that phase
- [ ] No RAM/startup regression > 10%
- [ ] All UI strings go through Fluent (en-US)
- [ ] Usable with the keyboard; visible focus
- [ ] Documentation and the API matrix updated when adding a `chrome.*` API

## First steps checklist

Goal of the first steps: a live repo, green CI, and one Moonlight window showing a web page through Servo.

- [ ] Create the `moonlight` repo (README, LICENSE MPL-2.0, CODE_OF_CONDUCT, SECURITY.md, all in English)
- [x] Write `AGENTS.md` at the repo root with project conventions for every AI session
- [ ] Create the Cargo workspace following the structure in the [Technical Architecture](02-technical-architecture.md), empty crates with `lib.rs`
- [ ] Pin the Rust version in `rust-toolchain.toml` and the `servo` version in `Cargo.toml`
- [ ] CI: fmt, clippy, deny, nextest on Windows
- [ ] Prototype: a `winit` window + one Servo `WebView` loading a URL from a CLI argument
- [ ] Add the simplest `egui` omnibox + back/forward/reload
- [ ] Measure the baseline: prototype RAM and startup time on the reference laptop
- [ ] Compile `tests/sites/sites.toml` (50 sites) and `tests/extensions/` (20 extensions + latest manifests)
- [ ] Build a status table for hooks H1–H8 with evidence (links to Servo code or upstream issues)
- [ ] Review ADR-001 at the end of Phase 0
- [ ] Design: work through the Alpha-priority design gaps (Settings, extension details, shield breakdown) and write English copy for every mockup
