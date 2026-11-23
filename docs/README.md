# Moonlight documentation

Moonlight is a lightweight desktop web browser written in Rust, built on the Servo engine, with support for Manifest V3 Chrome extensions.

Read the documents in this order:

| # | Document | What it answers |
| --- | --- | --- |
| 1 | [Product Requirements](01-product-requirements.md) | What we are building, for whom, which features, and how success is measured |
| 2 | [Technical Architecture](02-technical-architecture.md) | ADR-001 (Servo), components, engine hooks, Cargo workspace, security |
| 3 | [Chrome Extension Specification](03-chrome-extension-spec.md) | Extension runtime, `chrome.*` API matrix by phase, install flows, test extensions |
| 4 | [Design & UI Spec](04-design-ui-spec.md) | Design principles, tokens, screens, mockups, design gaps |
| 5 | [Roadmap](05-roadmap.md) | Phases F0–F4, exit criteria, how we work (solo + AI) |
| 6 | [Engineering Handbook](06-engineering-handbook.md) | Setup, conventions, CI, testing, Definition of Done, first steps |

Architecture decision records live in `docs/adr/`. Mockup screenshots live in `docs/assets/`.
