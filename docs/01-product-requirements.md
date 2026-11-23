# Moonlight — Product Requirements Document

Last updated: 2026-11-23

Related documents: [Technical Architecture](02-technical-architecture.md) · [Chrome Extension Specification](03-chrome-extension-spec.md) · [Design & UI Spec](04-design-ui-spec.md) · [Roadmap](05-roadmap.md) · [Engineering Handbook](06-engineering-handbook.md)

## Summary and vision

Moonlight is a desktop web browser written in Rust that stays light on memory and runs Manifest V3 Chrome extensions from the Chrome Web Store. Release order: Nightly 0.1, Alpha 0.2, Beta 0.5, then 1.0, with no fixed dates (see the [Roadmap](05-roadmap.md)).

**Vision:** a browser that feels as light as a text editor, still runs the extensions people rely on, and is honest about what it does not support yet.

Four product promises, taken from the designs:

1. **Light, and visibly so.** RAM use is shown on the New Tab page and per extension instead of being hidden.
2. **Built-in privacy.** The shield in the omnibox shows how many requests were blocked on the active tab.
3. **Familiar Chrome extensions.** Install from the Chrome Web Store, a .crx file, or an unpacked folder.
4. **Honest about compatibility.** A banner on pages that use unsupported features, and a Full or Partial compatibility label on every extension.

Identity: a purple crescent moon, dark theme by default, English-only UI, DuckDuckGo as the default search engine.

## Problem, opportunity, and positioning

People who want a lightweight browser usually have to give up their Chrome extensions; Moonlight closes that gap with a Rust engine plus its own extension compatibility layer.

**Problem**

- Chromium-based browsers use a lot of memory, which hurts on laptops with 8 GB or less.
- Lightweight alternative browsers usually cannot run Chrome extensions, so users lose their password manager, ad blocker, and work tools.
- When a site or extension fails in an alternative browser, users are not told why.

**Current opportunity**

- Servo is now available as a library: version 0.1.0 was published on crates.io on 13 April 2026 with an embedding API (`ServoBuilder`, `WebView`). Servo ships a new release every month, so web compatibility keeps improving.
- Chrome disabled Manifest V2 from Chrome 139, and all MV2 extensions were removed from the Chrome Web Store on 31 August 2026. Moonlight only needs MV3, which shrinks the API surface to build.

**Positioning:** Moonlight is a Rust-native browser for technical users and people on low-spec machines who prefer light and transparent over 100% compatibility. Moonlight does not compete with Chrome on every site; it competes on speed, memory, privacy, and honesty.

## Target users and personas

The primary v1 user is a developer or power user; the default shortcuts in the design (GitHub, docs.rs, Stack Overflow, MDN) already point that way.

| Persona | Who | Needs | What makes them leave |
| --- | --- | --- | --- |
| **Raka, Rust/web developer** (primary) | 26, 16 GB laptop, 30+ documentation tabs open | Light tabs, DevTools, dev extensions (JSON viewer, React DevTools), Bitwarden | Work sites breaking without explanation, no DevTools |
| **Sinta, privacy-conscious user** | 31, creative professional | Built-in tracker blocking, uBlock Origin Lite, cookie control | Silent telemetry, unclear extension permissions |
| **Dimas, student on a low-spec laptop** | 20, 4–8 GB laptop | Low RAM use, YouTube and Google Docs working, Google Translate | Lecture videos or documents that will not open |

Dimas is the riskiest persona: his needs depend on heavy sites the engine may not support yet. Validate this in the Phase 0 spike before making it a 1.0 target.

## Goals, non-goals, and success metrics

The 1.0 release succeeds if Raka can use Moonlight every day: his work sites run, his main extensions run, and RAM use is clearly lower than Chrome.

**Goals**

1. **G1 — Daily driver:** sites on the Moonlight test list (50 sites, see the [Engineering Handbook](06-engineering-handbook.md)) can be used for their main task.
2. **G2 — Extensions:** popular MV3 extensions install and work without modification.
3. **G3 — Light:** memory footprint and startup time well below Chromium browsers for the same workload.
4. **G4 — Transparent:** every incompatibility is labeled and can be reported.

**Non-goals for v1**

- Building our own engine from scratch (Moonlight uses an existing engine; see ADR-001 in the [Technical Architecture](02-technical-architecture.md)).
- Mobile (Android/iOS).
- Manifest V2 extensions.
- A Moonlight extension store.
- Cross-device sync and multiple profiles (planned after 1.0).
- DRM content (Widevine), so some streaming services will not work.
- UI languages other than English.

**1.0 success metrics** (numbers are initial hypotheses, locked after the Phase 0 spike)

| Metric | 1.0 target | How measured |
| --- | --- | --- |
| Test sites passing the smoke test | ≥ 90% of 50 sites | Automated suite: page loads, no fatal JS errors, main flow is clickable |
| Test extensions rated Full, or Partial with core features working | ≥ 15 of 20 | Extension compatibility suite ([Chrome Extension Specification](03-chrome-extension-spec.md)) |
| Total RAM, 1 New Tab + the 4 extensions in the designs | ≤ 180 MB | Per-commit benchmark in CI on a reference machine |
| Cold start until New Tab is interactive | ≤ 1.0 s | Reference laptop: 4 cores, 8 GB, SSD |
| Crash-free sessions | ≥ 99% | Opt-in crash reports |
| Beta tester day-7 retention | ≥ 40% | Opt-in, anonymous telemetry |

## Feature scope and priorities

Alpha (P0) contains 19 features that make Moonlight usable day to day; 1.0 (P1) rounds it out into a general-purpose browser; P2 is deferred until after 1.0.

| Area | Feature | Priority | Notes |
| --- | --- | --- | --- |
| Tabs & navigation | Open, close, switch, and drag to reorder tabs | P0 · Alpha | |
| Tabs & navigation | Pin and mute tabs | P1 · 1.0 | |
| Tabs & navigation | Omnibox: URL, DuckDuckGo search, suggestions from history and bookmarks | P0 · Alpha | Search engine can be changed in Settings |
| Tabs & navigation | Back, forward, reload, zoom, find in page | P0 · Alpha | |
| Tabs & navigation | Private window | P1 · 1.0 | Temporary profile, no history |
| Tabs & navigation | Restore session after closing or a crash | P1 · 1.0 | |
| Tabs & navigation | Tab groups | P2 · Post-1.0 | |
| Built-in pages | New Tab: search, shortcuts, stats for trackers blocked, RAM, and load time saved | P0 · Alpha | Matches the New Tab mockup |
| Built-in pages | Settings: General, Privacy, Appearance, Extensions, Downloads, About | P0 · Alpha | Matches the mockup sidebar |
| Built-in pages | History | P0 · Alpha | |
| Built-in pages | Bookmarks and bookmarks bar | P0 · Alpha | Star button in the toolbar |
| Built-in pages | Downloads panel and page | P0 · Alpha | |
| Built-in pages | Error pages: offline, invalid certificate, crashed tab | P0 · Alpha | |
| Extensions | Load unpacked folder (Developer mode) | P0 · Alpha | |
| Extensions | Install from the Chrome Web Store with a permission dialog | P0 · Alpha | Matches the Bitwarden dialog mockup |
| Extensions | Install .crx files with CRX3 verification | P0 · Alpha | |
| Extensions | Toolbar icon, badge count, and popup | P0 · Alpha | |
| Extensions | Extension card: version, RAM, compatibility label, enable toggle | P0 · Alpha | |
| Extensions | Limit extension access to specific sites | P1 · 1.0 | Toggle in the install dialog |
| Extensions | Automatic extension updates | P1 · 1.0 | |
| Privacy & security | Built-in tracker blocker (EasyList + EasyPrivacy), global and per-site toggle, on by default | P0 · Alpha | `adblock` crate (adblock-rust), MPL-2.0 |
| Privacy & security | Omnibox shield: requests blocked on the active tab | P0 · Alpha | "14 blocked" in the mockup |
| Privacy & security | Clear browsing data | P0 · Alpha | |
| Privacy & security | Block third-party cookies | P1 · 1.0 | |
| Privacy & security | HTTPS-only mode | P1 · 1.0 | |
| Privacy & security | Site permissions: camera, microphone, location, notifications | P1 · 1.0 | |
| Privacy & security | Site info and certificate | P1 · 1.0 | Click the lock icon |
| Compatibility | "Unsupported feature" banner with a Report button | P0 · Alpha | Matches the yellow banner mockup |
| Compatibility | Send compatibility reports (opt-in) | P1 · 1.0 | |
| Other | DevTools through remote debugging | P1 · 1.0 | |
| Other | Print and save as PDF | P1 · 1.0 | Depends on engine support |
| Other | PDF viewer | P1 · 1.0 | |
| Other | Import bookmarks, history, and passwords from Chrome and Firefox | P1 · 1.0 | |
| Other | App auto-update | P1 · 1.0 | |
| Other | English-only UI | P0 · Alpha | All strings go through Fluent so other languages stay possible later |
| Other | Light theme | P1 · 1.0 | Dark theme is the P0 default |
| Other | Accessibility: screen reader, full keyboard navigation | P1 · 1.0 | |
| Other | Built-in password manager | P2 · Post-1.0 | v1 relies on Bitwarden |
| Other | Sync and multiple profiles | P2 · Post-1.0 | |

## Functional requirements

Each requirement has an FR-xx ID so it can be referenced in issues and PRs; acceptance criteria are written to be testable automatically where possible.

### Tabs and omnibox

- **FR-01** As a user, I can open, close, and reorder tabs to keep my work organized.
    - Ctrl+T, Ctrl+W, Ctrl+Tab, and Ctrl+Shift+T (reopen closed tab) work.
    - Switching tabs renders in under 50 ms on the reference machine.
- **FR-02** As a user, I type a URL or keywords in the omnibox and go straight to my destination.
    - Input that is not a valid URL goes to the default search engine (DuckDuckGo).
    - Suggestions from history and bookmarks appear within 50 ms of typing.
    - Lock icon + bold domain; the path is dimmed (as in the mockup).
- **FR-03** The omnibox shield shows how many requests were blocked on the active tab, counting the built-in blocker plus extensions' declarativeNetRequest rules; clicking it opens a per-domain breakdown and a blocker toggle for that site. The global toggle lives in Settings > Privacy (on by default).

### New Tab

- **FR-10** The New Tab page shows the logo, a search box, up to 8 shortcuts plus an Add button, and three stats: trackers blocked this week, RAM in use now, and load time saved.
    - Stats are computed locally; no data leaves the device.
    - "Load time saved" = blocked requests × the average load time of similar requests (scripts, images, iframes) measured on that device; until there is enough data, use 50 ms per request. Shown with "~" and explained on the About page.

### Extensions

- **FR-20** On `moonlight://extensions`, the user sees a card per extension: name, version, RAM, description, compatibility label, site access scope, Details link, and enable toggle.
- **FR-21** On a Chrome Web Store page, the user presses install and sees a Moonlight dialog listing the permissions, a "Limit to specific sites" toggle, package status (verified CRX3, size, MV3), and Cancel / Add extension.
    - Packages with an invalid signature are rejected with a clear message.
    - MV2 extensions are rejected with an explanation that only MV3 is supported.
- **FR-22** The compatibility label is computed before installing: Full, Partial (naming the missing APIs), or Not supported.
- **FR-23** The extension icon in the toolbar shows a badge; clicking it opens the extension's own HTML popup, rendered by the Moonlight engine.
- **FR-24** "Load unpacked" and "Install .crx file" buttons are available; Load unpacked is only enabled when Developer mode is on.

### Site compatibility

- **FR-30** When a page uses a web API the engine does not support yet, a yellow banner appears: "Some features on this site may not be supported by the Moonlight engine yet." with a Report button and a close button.
    - After it is closed, the banner does not reappear for the same domain for 7 days.
    - Report opens a form with the URL, Moonlight version, and the list of failing APIs; the user reviews it before sending.

### Settings and data

- **FR-40** The Settings page has six sections matching the mockup sidebar: General, Privacy, Appearance, Extensions, Downloads, About.
- **FR-41** History, bookmarks, and downloads are stored per profile and can be cleared by time range.
- **FR-42** Downloads show progress, pause/resume, open folder, and warn about executable files.

## Non-functional requirements

Security and privacy have the strictest bar; performance targets are hypotheses locked after the Phase 0 benchmarks.

| ID | Category | Requirement | Target |
| --- | --- | --- | --- |
| NFR-01 | Performance | Cold start until New Tab is interactive | ≤ 1.0 s |
| NFR-02 | Performance | RAM: 1 New Tab, no extensions | ≤ 120 MB |
| NFR-03 | Performance | Extra RAM per idle extension (service worker asleep) | ≤ 5 MB median |
| NFR-04 | Performance | Input latency in the omnibox and tab strip | ≤ 16 ms per frame (60 fps) |
| NFR-05 | Stability | A crash in one tab or one extension does not close the window | Required from Beta |
| NFR-06 | Security | Extensions can only use the APIs and hosts listed in their permissions | Enforced in Rust, not in JS |
| NFR-07 | Security | .crx packages are verified against their CRX3 signature before unpacking | 100% of installs |
| NFR-08 | Security | Update binaries are signed and verified | 100% of updates |
| NFR-09 | Security | Engine security patches (Servo/SpiderMonkey) are re-released | ≤ 14 days after upstream |
| NFR-10 | Privacy | No telemetry without opt-in; crash reports are opt-in | Off by default |
| NFR-11 | Privacy | New Tab stats are computed and stored locally | No outbound requests |
| NFR-12 | Accessibility | Browser UI meets WCAG 2.2 AA (contrast, focus, labels) | Audit before 1.0 |
| NFR-13 | Platform | Alpha: Windows 10/11 x64. From Beta: + Linux x64 (Wayland + X11) and macOS | All three at 1.0 |
| NFR-14 | Size | Installer | ≤ 120 MB |
| NFR-15 | Localization | All UI strings go through Fluent files (en-US) | 0 hard-coded strings |

RAM gates: 150 MB for 1 New Tab is the relaxed gate for the F0/F1 prototype; 120 MB (NFR-02) and 180 MB with 4 extensions are the 1.0 targets; the 96 MB in the mockup is only an illustration.

## Assumptions, risks, and decisions

The biggest risk is the engine's web compatibility; the whole plan depends on the results of the Phase 0 spike.

**Assumptions**

- Built solo with AI assistance (vibe coding), with no fixed schedule.
- Servo is the main engine (ADR-001 accepted on 23 November 2026, because it is native Rust and lightweight).
- Open source project under the MPL-2.0 license, the same as Servo.

**Risks**

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Major sites (YouTube, Google Docs, banking) do not work in Servo yet | Dimas and part of Raka's needs fail | Phase 0 spike measures 50 sites; compatibility banner; `EngineBackend` interface keeps a CEF backend possible |
| The engine lacks hooks extensions need (isolated world, network interception, service worker) | Core extensions cannot run | Hook list in the [Technical Architecture](02-technical-architecture.md) verified in the spike; upstream contributions to Servo |
| The Chrome Web Store hides the install button for non-Chrome browsers, or its terms of service restrict it | Install from CWS fails | Moonlight injects its own install button; check CWS terms; .crx and unpacked stay available |
| Sandboxing and site isolation are weaker than Chromium | Security bugs have a bigger impact | Written threat model, external audit before 1.0, honest messaging on the About page |
| The upstream engine changes its API (0.x versions) | Rework on every upgrade | Pinned versions, a single adapter crate, scheduled monthly upgrades |
| Solo developer vs. the scope of a full browser | Progress slows | Strict P0/P1/P2 priorities; P2 features can be cut |

**Decisions and open questions**

- [x] Engine: Servo, decided 23 November 2026 (ADR-001)
- [x] Platforms: Alpha is Windows only; Linux and macOS arrive in Beta
- [x] License: MPL-2.0, decided 23 November 2026
- [x] UI language: English only; no other languages planned
- [x] Built-in blocker: yes, with global and per-site toggles, on by default, decided 23 November 2026
- [x] "Load time saved" formula: see FR-10, decided 23 November 2026
- [x] Team: solo + AI; roadmap based on milestones with no dates, decided 23 November 2026
- [ ] Chrome Web Store terms of service: confirm that installing from CWS in Moonlight is allowed

## Sources

- [Servo 0.1.0 on crates.io (Simon Willison, 13 Apr 2026)](https://simonwillison.net/2026/Apr/13/servo-crate-exploration/)
- [Servo 0.1.0 ships on crates.io (byteiota)](https://byteiota.com/servo-0-1-0-ships-on-crates-io-embeddable-rust-browser/)
- [Manifest V2 support timeline (Chrome for Developers)](https://developer.chrome.com/docs/extensions/develop/migrate/mv2-deprecation-timeline)
- [adblock crate (lib.rs)](https://lib.rs/crates/adblock)
