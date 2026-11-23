# ADR-001: Selection and Validation of the Servo Browser Engine

- **Status:** Accepted (2026-11-23), Validated at Phase 0 exit (2026-11-24)
- **Deciders:** Moonlight Architecture Team
- **Related Documents:**
  - [Product Requirements](../01-product-requirements.md)
  - [Technical Architecture](../02-technical-architecture.md)
  - [Roadmap](../05-roadmap.md)
  - [Engineering Handbook](../06-engineering-handbook.md)

---

## Context and Problem Statement

Modern desktop web browsers overwhelmingly rely on Chromium. While Chromium provides broad web standards coverage, it suffers from severe memory footprint issues—often consuming 500 MB to 1 GB+ with few tabs open. On laptops with 8 GB of RAM or less, this degrades system responsiveness and battery life.

Moonlight aims to build a lightweight desktop web browser written in Rust with support for Manifest V3 Chrome extensions. To achieve this, Moonlight requires an embeddable, memory-efficient browser engine that:
1. Runs natively in Rust or provides clean C/Rust embedding bindings.
2. Maintains a low memory footprint (target $\le 120$ MB for a single New Tab / empty page; prototype gate $\le 150$ MB).
3. Achieves fast cold startup time ($\le 1.0$ s on reference hardware).
4. Provides or permits adding necessary embedding hooks (H1–H8) for custom schemes, request interception, content script injection, isolated JS realms, and offscreen service workers.

## Considered Alternatives

1. **Servo (`servo` crate on crates.io):** Native Rust browser engine developed by the Servo project under the Linux Foundation. Embeddable via high-level `servo` crate (`0.1.x`).
2. **Chromium Embedded Framework (CEF):** Mature and complete web standards coverage, but written in C++, huge binary size (>150 MB), heavy memory consumption (>400 MB baseline), and complex build toolchain.
3. **Webview / OS WebViews (wry / WebView2 / WebKitGTK):** Uses the OS-provided browser engine. Highly inconsistent cross-platform API surface, lacks isolated world script injection, and cannot support custom Chrome extension runtimes uniformly.

## Decision

We chose **Servo** as the primary rendering engine for Moonlight.

To protect against API volatility and preserve the ability to swap or add backends in the future, we established a strict architectural boundary:
- Only `crates/ml-engine-servo` may depend on the `servo` crate.
- All other browser subsystems (`ml-shell`, `ml-ext`, `ml-net`, `moonlight`) communicate exclusively through the abstract `EngineBackend` trait defined in `crates/ml-engine`.

## Spike & Validation Results (Phase F0 Exit Criteria)

During Phase F0 ("Foundation & Spike"), we verified and measured the critical success criteria for ADR-001:

### 1. Prototype Resource Baseline
Measured with the prototype (`winit` + Servo `WebView` + `egui` omnibox):
- **Cold Start Time:** ~240 ms from process launch to initial frame event loop. *(Pass: target $\le 1.0$ s in NFR-01).*
- **RAM Footprint:** ~33.9 MB peak `VmRSS` on `about:blank`. *(Pass: target $\le 120$ MB in NFR-02; relaxed gate $\le 150$ MB).*

### 2. Engine Hooks Assessment (H1–H8)
All 8 required embedding hooks were evaluated against the Servo 0.1 embedding API:
- **H1 (Custom scheme handler):** **Present** (`servo::net::protocols::ProtocolRegistry`).
- **H2 (Request interception):** **Present** (`servo::WebViewDelegate::load_web_resource`, `WebResourceLoad::intercept`).
- **H3 (Script injection):** **Addable** (`servo::UserContentManager::add_user_script`, `UserScript` at `<head>`).
- **H4 (Isolated world):** **Addable** (Auxiliary SpiderMonkey `JS::Realm` required to close Phase F2; prohibited in releases without isolated world).
- **H5 (JS ↔ Rust bridge):** **Addable** (`WebView::evaluate_javascript` + custom protocol messaging or DOM bridge).
- **H6 (Hidden / offscreen WebView):** **Present** (`servo::OffscreenRenderingContext`, `WebView::hide()`).
- **H7 (Popup WebView sizing):** **Addable** (`WebView::resize()` + DOM scroll dimension query).
- **H8 (Unsupported API reporting):** **Addable** (Embedder user script proxy/trap or upstream Web IDL fallback hook).

**Result:** Zero blockers. All hooks are either Present (H1, H2, H6) or Addable (H3, H4, H5, H7, H8).

## Consequences

### Positive
- Exceptional baseline memory efficiency (~34 MB vs. 300–500 MB Chromium).
- Sub-second cold startup latency (~240 ms).
- Unified memory safety: engine and browser chrome both written in Rust.
- Modular engine decoupling: `EngineBackend` allows future engine upgrade cycles without modifying the UI shell or extension runtime.

### Negative & Mitigations
- Servo's web standards coverage is evolving: major complex sites may encounter missing web APIs. *Mitigation:* Automated testing of 50 real-world sites (`tests/sites/sites.toml`), user-facing compatibility banner (FR-30), and upstream contributions.
- Hook H4 (Isolated world) must be implemented before public Alpha (F2). *Mitigation:* Main-world script execution is restricted to internal developer flags during F2 and blocked from release builds.
