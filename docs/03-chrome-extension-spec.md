# Moonlight — Chrome Extension Specification

Last updated: 2026-11-23

Related documents: [Product Requirements](01-product-requirements.md) · [Technical Architecture](02-technical-architecture.md)

## Compatibility strategy

Moonlight supports Manifest V3 only and builds the `chrome.*` APIs step by step, starting with the APIs popular extensions use, then labels the rest honestly.

- **MV3 only.** Manifest V2 has not worked since Chrome 139, and all MV2 extensions were removed from the Chrome Web Store on 31 August 2026. There is no reason to support it.
- **Compatible at the API level, not the Chromium code level.** Moonlight writes its own implementation in Rust, following Chrome's official documentation. Electron takes a similar approach and officially supports only part of the extension API.
- **`chrome` and `browser` namespaces.** Both point to the same object, since Chromium now also provides a `browser` namespace.
- **Compatibility labels are computed automatically.** Before installing, Moonlight reads `manifest.json` (permissions, manifest keys) and scans the extension code for `chrome.<namespace>.<method>` calls, then compares them against the API matrix below.

| Label | Condition | Display |
| --- | --- | --- |
| Full compatibility | Every permission and API used is implemented | Green |
| Partial | Some APIs are missing, but no required permission | Orange, naming the API (design example: `chrome.fontSettings not available yet`) |
| Not supported | MV2, or a required permission is missing | Red, install button disabled |

## Extension runtime architecture

Each extension gets three kinds of context that all talk to a single `ExtensionHost` in Rust; the JS shim only forwards messages, and all logic and permission checks live in Rust.

**Contexts per extension**

| Context | Form in Moonlight | Lifecycle |
| --- | --- | --- |
| Service worker (`background.service_worker`) | Hidden WebView at origin `chrome-extension://<id>/` | Woken by events; shut down after 30 s idle |
| Extension pages (popup, options, tab pages) | Regular WebView at the same origin | While open; the popup closes when focus moves |
| Content scripts | Scripts injected into web pages (hooks H3/H4) | Follows the page |

**Flow of one API call**, for example `chrome.storage.local.set()` from a popup:

1. The JS shim turns the call into a message `{ext_id, ctx_id, api, args, call_id}` and sends it over the H5 bridge.
2. `ExtensionHost` checks that `ext_id` matches the sending context (filled in by Rust, not by JS).
3. `PermissionGate` checks the `storage` permission in the manifest.
4. The namespace handler in `ml-ext-api` writes to `extension-storage/<id>.db`.
5. The result returns with its `call_id`; the shim resolves the Promise or calls the callback and sets `chrome.runtime.lastError` on failure.
6. A `storage.onChanged` event is broadcast to every live context of that extension.

**Runtime parts**

- `ManifestParser`: MV3 validation with `serde`, rejecting unknown required keys.
- `CrxVerifier`: CRX3 verification and extension ID derived from the public key.
- `PermissionGate`: API permissions, host permissions, optional permissions, and user site restrictions.
- `EventRouter`: registers listeners and wakes the service worker when an event arrives.
- `DnrEngine`: matches declarativeNetRequest rules at hook H2; counts blocks for the shield and badge.
- `ContentScriptManager`: matches `matches`/`exclude_matches` and `run_at`, then injects.
- `ActionManager`: per-tab icon, badge, title, and popup in the toolbar.

## chrome.* API matrix by phase

Alpha covers 9 API groups, enough for ad blockers, dark mode, and simple tools; 1.0 adds the APIs password managers and productivity tools need.

| API / manifest key | Phase | Scope and notes |
| --- | --- | --- |
| `runtime` | Alpha | `id`, `getURL`, `getManifest`, `sendMessage`, `connect`, `onMessage`, `onInstalled`, `onStartup`, `lastError` |
| `storage` | Alpha | `local`, `session`, `sync`, `onChanged`; `sync` behaves like `local` until a Sync feature exists |
| `i18n` + `_locales` folder | Alpha | `getMessage`, `getUILanguage`, `__MSG_*__` placeholders in the manifest |
| `action` | Alpha | Icon, badge text/color, title, popup, `onClicked` |
| `tabs` | Alpha | `query`, `get`, `create`, `update`, `remove`, `sendMessage`, `onUpdated`, `onActivated`, `onRemoved` |
| `scripting` + `content_scripts` | Alpha | `executeScript`, `insertCSS`, `registerContentScripts`; depends on hooks H3/H4; H4 is required to close F2 (see Security principles in the Technical Architecture) |
| `declarativeNetRequest` | Alpha | Static rulesets, dynamic and session rules, `getMatchedRules`, `setExtensionActionOptions`; feeds the shield |
| `permissions` + `optional_permissions` | Alpha | `request`, `contains`, `remove`, `onAdded`, `onRemoved` |
| `alarms` | Alpha | Keep firing while the service worker sleeps |
| `contextMenus` | 1.0 | Right-click menus on pages and the extension icon |
| `notifications` | 1.0 | OS notifications |
| `windows` | 1.0 | `getCurrent`, `create`, `update`, `onFocusChanged` |
| `webNavigation` | 1.0 | Per-frame navigation events |
| `commands` | 1.0 | Extension keyboard shortcuts |
| `cookies` | 1.0 | With host permissions |
| `offscreen` | 1.0 | Hidden document for DOM, clipboard, audio; hook H6 |
| `idle` | 1.0 | Password manager auto-lock |
| `webRequest` (observational) + `webRequestAuthProvider` | 1.0 | Non-blocking, per MV3 |
| `downloads` | 1.0 | |
| `history`, `bookmarks`, `topSites`, `search` | 1.0 | |
| `identity.launchWebAuthFlow` | 1.0 | OAuth sign-in through a Moonlight window |
| `sidePanel` | Post-1.0 | Needs a side panel design |
| `userScripts` | Post-1.0 | Userscript managers |
| `devtools.*` | Post-1.0 | Waits for Moonlight DevTools |
| `fontSettings` | Post-1.0 | Design example: Partial label on Dark Reader |
| `tabGroups` | Post-1.0 | Waits for the tab groups feature |
| `privacy`, `proxy` | Post-1.0 | |
| `identity.getAuthToken`, `gcm`, `enterprise.*`, `debugger`, `platformKeys` | Not planned | Depend on a Google/Chrome account or enterprise policy |

## Install, verification, and update flows

All three install paths (Chrome Web Store, .crx file, unpacked folder) end in the same pipeline: verify, compute the compatibility label, show the permission dialog, then unpack.

**From the Chrome Web Store** (as in the "Add Bitwarden?" dialog mockup)

1. The user opens `chromewebstore.google.com/detail/...`. Moonlight recognizes the domain and reads the extension ID from the URL.
2. If the page does not show an install button for non-Chrome browsers, Moonlight injects an "Add to Moonlight" button. The open source [chromium-web-store](https://github.com/NeverDecaf/chromium-web-store) project shows this pattern works in Chromium-based browsers.
3. Moonlight downloads the CRX3 package from the update endpoint Chromium uses for the Web Store (endpoint details and terms of service checked in Phase 0).
4. The verification pipeline runs (below).
5. The Moonlight dialog appears: permissions in plain language, a "Limit to specific sites" toggle, a status line "Verified CRX3 package · 4.2 MB · MV3", and Cancel / Add extension buttons.
6. After confirmation, the package is unpacked to `extensions/<id>/<version>/`, `runtime.onInstalled` fires, and the icon appears in the toolbar.

**From a .crx file:** the "Install .crx file" button on `moonlight://extensions`, or drag and drop the file onto the window, then steps 4–6.

**From an unpacked folder:** only when Developer mode is on. No signature check; the ID is derived from the folder path; a Reload button on the extension card reloads it after code changes.

**Verification pipeline**

1. Check the `Cr24` magic number and format version 3.
2. Parse the protobuf header; verify at least one developer-key signature over the signed data + ZIP contents.
3. Derive the extension ID from the public key and match it against the requested ID.
4. Unpack the ZIP with size limits and reject paths outside the target folder.
5. Validate `manifest.json`: `manifest_version` must be 3.
6. Compute the compatibility label.

**Updates:** every 5 hours, Moonlight checks for new versions of Web Store extensions. If a new version asks for extra permissions, the extension is disabled until the user approves, as in Chrome.

## Test extension set

Compatibility is measured on a fixed set of 20 extensions covering different API patterns; the 1.0 target is 15 of 20 rated Full, or Partial with no core function lost. Each extension's API list is filled in from its latest `manifest.json` at the start of Phase 0.

| # | Extension | Category | Target |
| --- | --- | --- | --- |
| 1 | uBlock Origin Lite | Content blocker | Alpha |
| 2 | Dark Reader | Page theming | Alpha |
| 3 | Google Translate | Translation | Alpha |
| 4 | Bitwarden | Password manager | 1.0 |
| 5 | 1Password | Password manager | 1.0 |
| 6 | Privacy Badger | Privacy | Alpha |
| 7 | SponsorBlock | Media | 1.0 |
| 8 | Return YouTube Dislike | Media | 1.0 |
| 9 | Vimium | Keyboard navigation | Alpha |
| 10 | JSON Formatter | Developer | Alpha |
| 11 | Wappalyzer | Developer | 1.0 |
| 12 | React Developer Tools | Developer (devtools) | Post-1.0 |
| 13 | Tampermonkey | Userscripts | Post-1.0 |
| 14 | Grammarly | Writing | 1.0 |
| 15 | LanguageTool | Writing | 1.0 |
| 16 | Notion Web Clipper | Productivity | 1.0 |
| 17 | OneTab | Tab management | Alpha |
| 18 | ColorZilla | Design | 1.0 |
| 19 | Momentum | New Tab replacement | 1.0 |
| 20 | Google Docs Offline | Productivity | Post-1.0 |

**How it is measured:** each extension has an automated scenario in `tests/extensions/<name>/` (install, open the popup, run the main action, check the result on a local test page). CI runs every scenario nightly and publishes a status table to an internal dashboard.

## Sources

- [Manifest V2 support timeline (Chrome for Developers)](https://developer.chrome.com/docs/extensions/develop/migrate/mv2-deprecation-timeline)
- [Chrome Extension Support in Electron](https://electronjs.org/docs/api/extensions)
- [Announcement of the browser namespace for Chromium extensions](https://groups.google.com/a/chromium.org/g/chromium-extensions/c/gK1Sd57p4go/m/lgP9_dhZBQAJ)
- [chromium-web-store (NeverDecaf)](https://github.com/NeverDecaf/chromium-web-store)
