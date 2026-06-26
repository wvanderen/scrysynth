# Stack Research — Scrysynth v2.0 "Studio-Grade Instrument"

**Domain:** New capabilities added to a shipping graph-native Tauri desktop audiovisual instrument
**Researched:** 2026-06-26
**Scope:** Stack additions/changes for the **seven v2.0 features only**. The v1 shipped stack is treated as immutable integration context (see "Existing shipped stack" below), not re-justified.
**Overall confidence:** MEDIUM-HIGH (one HIGH-RISK area: visuals-behind-grid compositing)

> ⚠️ **Correction to the prior STACK.md (dated 2026-04-11).** The greenfield STACK.md was aspirational in places. The **actually shipped** `package.json`/`Cargo.toml` (v1.0) differ from it: **no Radix, no vanilla-extract, no elkjs, no `tokio`, no `rusqlite`** shipped; TypeScript is `~5.8.3` (not 6.0), Vite `^7.0.4` (not 8), React `^19.1.0`. Persistence is **versioned JSON files** (`persistence/session_file.rs`), not SQLite. The visual runtime is a **second binary (`scrysynth-visual`) in the same crate**, not a separate workspace crate. This document is grounded in the real code, not the older research file.

## Existing Shipped Stack (integration context — do NOT re-add)

These are already working in v1.0 and are the integration points for v2.0 additions:

| Layer | What ships (verified in repo) | Relevant to v2.0 because… |
|------|-------------------------------|---------------------------|
| Shell | Tauri 2 (`tauri` crate 2, `@tauri-apps/cli` 2, `@tauri-apps/api` 2) + `tauri-plugin-shell`/`-dialog`/`-opener` | Updater + cross-platform work extends this |
| Core | Rust + `serde`/`serde_json`/`ts-rs`/`thiserror`/`uuid`/`tracing` (no `tokio` — uses `tauri::async_runtime`) | LLM provider runs on Tauri's async runtime |
| Audio | SuperCollider 3.14 via `rosc` (OSC); synthdefs in `audio/synthdefs.rs` + `audio/compiler.rs` | Node library maps onto synthdefs |
| MIDI | `midir` 0.10 (CoreMIDI virtual source) | Unchanged in v2.0 |
| Visual | Bevy **0.18.1** as a same-crate second binary; JSON-lines stdin/stdout protocol; `bevy_runtime.rs` (visible window mode) + `sidecar.rs` (minimal mode); bundled via `externalBin` | "Visuals behind grid" evolves this |
| Frontend | React 19.1 + Vite 7 + TypeScript 5.8 + `@xyflow/react` 12.10.2 + `zustand`/`immer` + `zod` (plain CSS, **no** Radix/vanilla-extract) | Graph UX + shell UI rebuild here |
| Persistence | Versioned JSON files via `serde` (no SQLite) | Unchanged in v2.0 |

---

## 1. Graph UX Rebuild (draggable nodes, edge connect/reconnect)

**Verdict: NO new interaction libraries.** `@xyflow/react` v12 already owns this surface natively.

| Layer | Technology | Version | Purpose | Why / integration | Confidence |
|------|------------|---------|---------|-------------------|------------|
| Node editor | `@xyflow/react` | **upgrade 12.10.2 → 12.11.1** (current, released 2026-06-22) | Custom nodes, multi-handle nodes, draggable, edge connect/reconnect | v12 has native `NodeTypes`, multi-`Handle` nodes, custom `edgeTypes` (`BaseEdge` + `getBezierPath`/`getSmoothStepPath`), **built-in edge reconnect** via `OnReconnect` type + `reconnectEdge()` util + a dedicated "Reconnect Edge" example, `isValidConnection` validation, and an "Easy Connect"/"Proximity Connect"/"Connection Limit"/"Preventing Cycles" example set. The rebuild is deeper use of the installed lib, not a new one. | HIGH |
| Auto-layout (optional) | `elkjs` | **0.11.1** (new dep) | Structured layout when the graph grows / scene recall | React Flow's own docs show elkjs + dagre layout examples. ELK handles multi-handle/bus/route layout far better than dagre once the node library and routes/buses appear. Pull in only when auto-layout is actually a v2.0 deliverable. | MEDIUM-HIGH |
| Auto-layout alt | `@dagrejs/dagre` | 3.0.0 (new dep) — only if a lighter layout suffices | Simpler layered layout | The maintained fork of dagre. **Avoid the original `dagre` (0.8.5) — it is unmaintained.** Prefer ELK unless you specifically need dagre's speed/shape. | MEDIUM |

**What NOT to add:** do **not** introduce Rete.js, BaklavaJS, LiteGraph, Flume, or a second node-editor framework. The cost/risk of migrating the existing React Flow graph off `@xyflow/react` dwarfs the rebuild effort, and v12 already covers every requested interaction. (See Anti-Recommendations.)

---

## 2. Curated Modular Node Library (~12–16 audio node types)

**Verdict: Mostly design + synthdef authoring work; minimal stack change.** The node library is a domain-content deliverable that maps onto the existing audio compiler/synthdef path, not a new dependency.

| Layer | Technology | Version | Purpose | Why / integration | Confidence |
|------|------------|---------|---------|-------------------|------------|
| Synthdef execution | SuperCollider 3.14 (existing) + existing `audio/synthdefs.rs`/`compiler.rs` | 3.14.1 (shipped) | Compile ~12–16 well-designed node types (oscillators, filters, envelopes, LFOs, FX, sequencing) to real SC synthdefs | The node library is authored as SC `SynthDef`s with rich per-node parameters; the existing OSC bridge (`rosc`) + bus/group model already compiles and runs synthdefs audibly. No new runtime. | HIGH |
| Node schema/contracts | Existing `ts-rs` (Rust→TS) + `zod` validation | shipped | Per-node parameter specs flow through the canonical graph | Keep node type definitions in the Rust domain core (source of truth) → `ts-rs` exports → React node components consume generated types. Do not define node schemas in the frontend. | HIGH |
| React node components | React 19 + Radix/vanilla-extract (see §4 if adopted) | existing + §4 | Rich per-node parameter UI (knobs, dropdowns, envelopes) | The visual richness comes from the shell UI work (§4), not a dedicated node-UI library. | MEDIUM-HIGH |

**What NOT to add:** do **not** pull in Web Audio / Tone.js / soundfont libs to "render" nodes — audio execution stays 100% SuperCollider-side by project constraint. Do not generate synthdefs dynamically from arbitrary LLM output without going through the typed command layer.

---

## 3. Visuals Behind the Grid (richer Bevy ambient layer, Bespoke-Synth-style) — 🔴 HIGHEST RISK

This is the single riskiest v2.0 item. The shipped visual runtime opens **its own Bevy window**; v2.0 wants the Bevy render to read as an **ambient layer behind the graph canvas inside the Tauri webview**. Two viable compositing strategies exist; recommend a **primary + fallback** split.

| Layer | Technology | Version | Purpose | Why / integration | Confidence |
|------|------------|---------|---------|-------------------|------------|
| Bevy (upgrade) | `bevy` | **upgrade 0.18.1 → 0.19.0** (current, crates.io 2026-06-26) | Richer GPU-native ambient visuals | Bevy moved to 0.19. Upgrade in lockstep; audit migration guide (0.18→0.19). The visual runtime is a same-crate binary, so the upgrade is one `Cargo.toml` bump + fixes. | HIGH (on upgrade) |
| GPU abstraction | `wgpu` (transitively via Bevy) | 29.0.3 (current) | Rendering foundation | Keep going through Bevy; do not depend on `wgpu` directly in the product app. | MEDIUM |
| **Strategy A — transparent overlay window** *(recommended primary, macOS + Windows)* | Tauri 2 window transparency + Bevy borderless transparent window | Tauri: `app.macOSPrivateApi: true` + window `transparent: true` | Bevy renders an always-behind, frameless, transparent window; main Tauri webview is transparent; React Flow canvas is translucent so visuals show through behind nodes/edges | Cheapest path to the "Bespoke-Synth ambient layer" feel; reuses the existing sidecar. App must track main-window position/size and keep the Bevy window z-ordered beneath it. **Caveat:** per-platform window stacking beneath a sibling window is reliable on macOS (`NSWindow.level`) and Windows (`SetWindowPos`/owner), but **unreliable on Linux/Wayland** (no global stacking control) → use Strategy B there. | MEDIUM |
| **Strategy B — Bevy GPU readback → webview canvas** *(fallback, esp. Linux/Wayland)* | Bevy render-to-texture + **GPU Readback** (stable since Bevy 0.15) → frame stream over local transport → `<canvas>` behind React Flow | Bevy 0.19 features | Bevy renders offscreen to a texture, reads frames back to CPU, streams to the webview and draws on a canvas behind the graph | True single-window compositing, no z-order fragility. Costs readback + IPC bandwidth; mitigate by rendering at viewport resolution, throttling to ~30–60 fps, and using a dedicated high-throughput channel (not the JSON-lines control protocol). | MEDIUM-LOW |
| Transport | Extend the existing JSON-lines sidecar protocol for control; add a **separate binary frame channel** (local socket / shared memory / Tauri IPC) for Strategy B | existing `tokio`-free, `tauri::async_runtime` + `serde` | Keep typed control messages on the existing protocol; ship raw frames on a distinct path so they don't starve control acks | Matches the v1 decision to keep a typed adapter protocol separate from the canonical graph model. | MEDIUM |
| Platform-native window control | `objc2-app-kit` (macOS), Win32 via `windows`/`windows-sys` (Windows) | current | Z-order/background of the overlay window (Strategy A) | Tauri's own window-customization doc uses `objc2-app-kit` to set NSWindow background; same approach sets window level. Expect small `#[cfg(target_os=…)]` glue. | MEDIUM |

**Recommendation:** Plan Strategy A as the primary experience on macOS/Windows (lowest cost, best feel), with Strategy B as the Linux/Wayland fallback and a future path to uniform single-window compositing. **This phase must carry an explicit research/spike flag** — the overlay-window z-order + transparent-webview combination across three OSes is the most likely source of v2.0 rework.

**What NOT to add:** do **not** move the canonical visuals into the webview via `three`/`@react-three/fiber`/PixiJS — that violates the project's separate-visual-runtime constraint and couples visuals to UI frame timing. Do **not** try to share a GPU texture handle directly between the Bevy sidecar and the WKWebView/WebView2 process — cross-process GPU texture sharing is not practically supported here; use readback+stream.

---

## 4. Pro-Grade Focused Shell UI/UX (iconography, theming, progressive disclosure)

The shipped frontend uses **plain CSS with no component system**. v2.0's "focused instrument, not dense/maximalist" shell is the moment to adopt a small, durable UI foundation. The prior STACK.md recommended Radix + vanilla-extract; they were never actually installed, so v2.0 is a clean adoption decision.

| Layer | Technology | Version | Purpose | Why / integration | Confidence |
|------|------------|---------|---------|-------------------|------------|
| Icon library | `lucide-react` | **1.21.0** (npm `latest`, verified 2026-06-26; peer `react ^19.0.0` ✓) | Consistent iconography across the shell, inspector, transport, palette | Lucide is the 2026 default for pro tooling UI: tree-shakeable, consistent stroke style, MIT, explicit React 19 peer support. Far better than hand-rolled SVGs or an ad-hoc icon set for a focused-instrument aesthetic. | HIGH |
| Headless UI primitives | Radix UI primitives | `@radix-ui/react-dialog` 1.1.15, `@radix-ui/react-slider` 1.3.6, `@radix-ui/react-*` as needed (new deps) | Accessible overlays, dropdowns, sliders, tabs, tooltips for inspector/transport/menus | Radix gives unstyled, accessible primitives (keyboard, focus, ARIA) without imposing a generic SaaS look — exactly right for a distinctive instrument shell that needs custom visual identity. Adopt incrementally per surface. | MEDIUM-HIGH |
| Styling/theming | `@vanilla-extract/css` (+ `@vanilla-extract/recipes` for variant components) | `@vanilla-extract/css` 1.20.1 (new dep) | Typed design tokens, themes (light/dark/performance-mode), zero-runtime CSS | Better than utility-first for an app with long-lived design tokens, multiple modes, and a strong visual identity. Build-time extraction keeps the runtime lean. Introduces a tiny Vite plugin step (Vite 7 is already in use). | MEDIUM-HIGH |
| Tooltip / disclosure | `@radix-ui/react-tooltip` + existing React state | current | Progressive-disclosure menus/panels | The "focused, not maximalist" goal is a design-system + interaction decision, not a library; Radix supplies the accessible substrate. | MEDIUM |

**What NOT to add:** do **not** adopt a full canned component kit (MUI, Chakra, Mantine, Ant) — they impose a generic SaaS design language antithetical to a distinctive pro-instrument shell and fight the "graph as hero" goal. Do **not** add Tailwind — utility-first is a poor fit for token-driven, mode-rich instrument theming and would clash with vanilla-extract.

---

## 5. Live Provider-Backed Agent (AGNT-01R)

The shipped planner has a clean, **synchronous, provider-agnostic** boundary: `trait PlannerProvider { fn plan(&request) -> Result<PlannerProviderOutput, PlannerProviderError> }`, returning `PlannerProviderOutput::Json(String)` or `Typed`. AGNT-01R is a **new `PlannerProvider` impl** that calls a live LLM. Keep the boundary intact.

| Layer | Technology | Version | Purpose | Why / integration | Confidence |
|------|------------|---------|---------|-------------------|------------|
| LLM client (primary) | `async-openai` | **0.41.1** (crates.io verified 2026-06-26; ~2M recent downloads) | Streaming chat completions + tool/function calling against any OpenAI-compatible endpoint | The dedicated, maintained Rust crate for OpenAI-compatible APIs; supports streaming and tool/function-calling. Works against hosted OpenAI **and** local/OpenRouter/Ollama/LM Studio by overriding `base_url` — preserving provider optionality the project wants. | MEDIUM-HIGH |
| HTTP client | `reqwest` | **0.13.4** (crates.io verified 2026-06-26; ~134M recent downloads) | Underlying transport (used by async-openai) or hand-rolled calls | Standard Rust HTTP client; `rustls-tls` feature avoids system OpenSSL on cross-platform builds. | HIGH |
| Async runtime | Existing `tauri::async_runtime` (no new `tokio` dep needed) | shipped | Drive async provider calls behind the synchronous planner trait | The planner trait is synchronous; bridge to async via `tauri::async_runtime::block_on` inside the provider impl, OR refactor the one call site to async. Do **not** add a bare `tokio` dependency just for the agent — Tauri already provides a runtime. | MEDIUM-HIGH |
| Tool/function calling | OpenAI-style tool schema | via async-openai | Let the LLM emit structured graph-edit intents | Parse LLM tool-call/JSON output into the existing `TypedCommand` set + `PlannerProposalWire` (already `serde`-derived). Keep the typed-command validation/ownership/freeze gates in place — the provider only *proposes*; the app still authorizes. | MEDIUM |
| Secrets | Tauri secure storage / env | existing patterns | API key handling | Never ship keys in the bundle; load at runtime from env/secure store. | HIGH |

**Recommendation:** Implement the provider with `async-openai` against an OpenAI-compatible endpoint (start with one hosted provider + one local option), keep the `PlannerProvider` trait synchronous, and add provider optionality via `base_url`. If you later want a thinner dependency surface, hand-roll with `reqwest` + `serde_json` (the OpenAI Chat Completions schema is small); avoid pulling in a heavyweight agent/RAG framework.

**What NOT to add:** do **not** adopt `rig-core` (0.39.0) or a full agent/RAG orchestration framework for v2.0 — Scrysynth already owns the session-aware, explainable, human-overridable orchestration model; a framework would compete with it. Do **not** let the LLM bypass the typed-command/ownership/approval layer.

---

## 6. Cross-Platform Builds (Windows / Linux / Intel macOS / universal macOS)

Tauri 2's bundler handles the packaging mechanics; the real work is **per-OS sourcing of the external runtimes (SuperCollider + the visual sidecar)** and a CI matrix.

| Layer | Technology | Version | Purpose | Why / integration | Confidence |
|------|------------|---------|---------|-------------------|------------|
| Bundler | Tauri 2 `bundle` + `externalBin` (target-triple-suffixed) | shipped | One bundle per OS/arch | Tauri already resolves `externalBin` by `<name>-<target-triple>{.exe}`. Add the visual sidecar for every target triple; extend `prepare-sidecar.sh` to cross-build per target. | HIGH |
| macOS universal | `lipo` of `aarch64`+`x86_64` binaries; Tauri `--target universal-apple-darwin` | current toolchain | Single universal `.app`/`.dmg` | Standard Tauri path; the updater's custom target `macos-universal` is explicitly supported. | MEDIUM-HIGH |
| SC on macOS | Official **3.14.1 universal** DMG (signed & notarized) | 3.14.1 | Extract `scsynth`/`sclang` for bundling | Official binaries exist for both arches. | HIGH |
| SC on Windows | Official **3.14.1 x64** installer (Win10/11) | 3.14.1 | Extract `scsynth.exe`/`sclang.exe` for bundling | **No official Windows arm64 build** — arm64 Windows is out of scope for SC bundling. | HIGH |
| SC on Linux | **No official prebuilt binary** — source tarball (gcc≥9) or distro packages | 3.14.1 | Linux audio runtime | 🔴 Critical pitfall: there is no official Linux SC binary. Options: (a) build SC from source in CI and bundle, (b) require a system SC install and detect it at runtime, (c) ship a distro-specific package. Recommendation: detect a system SC install at runtime on Linux and document it, rather than bundling a from-source blob into every Linux bundle. | MEDIUM (decision), HIGH (fact) |
| CI / matrix | GitHub Actions + `tauri-apps/tauri-action` | current | Build Windows/Linux/macOS(Intel+ARM+universal) artifacts + release JSON | `tauri-action` generates the updater `latest.json` per target and publishes to GitHub Releases. Run SC extraction as a per-OS setup step. | HIGH |

**What NOT to add:** do **not** assume SC is uniformly bundleable everywhere — Linux SC bundling is genuinely awkward. Do **not** target Windows arm64 for v2.0 (no upstream SC). Do **not** hand-roll release tooling when `tauri-action` already produces signed updater artifacts.

---

## 7. Full Developer ID Notarization + Auto-Updater

| Layer | Technology | Version | Purpose | Why / integration | Confidence |
|------|------------|---------|---------|-------------------|------------|
| Updater (Rust) | `tauri-plugin-updater` | **2.10.1** (crates.io verified 2026-06-26) | In-app update check + download + verify + install | Official plugin; `#[cfg(desktop)] app.plugin(UpdaterBuilder::new().build())` in `lib.rs`. | HIGH |
| Updater (JS) | `@tauri-apps/plugin-updater` | **2.10.1** (npm verified 2026-06-26) | Frontend update UX | Pairs with the JS guest bindings; expose check/download/install over a Tauri channel for progress. | HIGH |
| Relaunch | `@tauri-apps/plugin-process` (+ Rust `tauri-plugin-process`) | current 2.x | Restart after install | Required companion — updater installs, process plugin relaunches. Add to deps + capabilities. | HIGH |
| Signing keys | Tauri CLI `tauri signer generate` | ships with `@tauri-apps/cli` 2 | Ed25519 keypair to sign update bundles | `pubkey` goes in `tauri.conf.json` (`plugins.updater.pubkey`, **inlined, not a path**); private key in `TAURI_SIGNING_PRIVATE_KEY` env (`.env` files do **not** work). | HIGH |
| Update artifacts | `bundle.createUpdaterArtifacts: true` | Tauri config | Emit `.app.tar.gz.sig` / `.AppImage.sig` / `.exe.sig` / `.msi.sig` | One config flag; produces per-target signed bundles. | HIGH |
| Update endpoint | Static JSON via GitHub Releases (`tauri-action`) | — | Distribution + version negotiation | Simplest credible path; `tauri-action` writes `latest.json` with `{{target}}`/`{{arch}}` resolution. Avoids standing up a dynamic update server. | HIGH |
| macOS notarization | `xcrun notarytool` (Apple) **or** `rcodesign` (cargo-installable, cross-platform, no Xcode needed) | current | Developer ID signing + notarization + stapling | `rcodesign` is the right choice for CI without a macOS GUI/Xcode: signs + notarizes + staples from any host. Tauri reads signing identity from `bundle.macOS.signingIdentity`. Set `APPLE_*` notarization creds in CI secrets. | HIGH (rcodesign path) |
| Windows signing | EV/OV code-sign cert via `bundle.windows.certificateThumbprint` or `signCommand` | — | Authenticode-sign `.exe`/`.msi` | Tauri config supports a cert thumbprint or a custom `signCommand`. Avoid SmartScreen reputation friction by using an EV cert or accumulating reputation. | MEDIUM-HIGH |

**What NOT to add:** do **not** build a custom update verifier/protocol — Tauri's signature scheme is mandatory and cannot be disabled. Do **not** rely on `.env` for the signing key (explicitly unsupported). Do **not** ship a Developer-ID-signed/notarized build without testing the updater end-to-end against the static JSON manifest.

---

## Recommended Initial Package Set (v2.0 additions only)

```bash
# frontend (graph UX, shell UI)
npm install lucide-react @xyflow/react@^12.11.0
# optional layout (add when auto-layout is in scope)
npm install elkjs
# UI foundation (adopt incrementally per shell surface)
npm install @radix-ui/react-dialog @radix-ui/react-slider @radix-ui/react-tooltip \
            @vanilla-extract/css @vanilla-extract/recipes
# dev (vanilla-extract vite plugin)
npm install -D @vanilla-extract/vite-plugin

# updater (frontend + process for relaunch)
npm install @tauri-apps/plugin-updater @tauri-apps/plugin-process

# rust core (add to src-tauri/Cargo.toml)
cargo add async-openai reqwest --features reqwest/rustls-tls
cargo add tauri-plugin-updater --target 'cfg(any(target_os = "macos", windows, target_os = "linux"))'
cargo add tauri-plugin-process  --target 'cfg(any(target_os = "macos", windows, target_os = "linux"))'
# upgrade existing
#   bevy 0.18.1 -> 0.19.0
#   @xyflow/react 12.10.2 -> 12.11.1

# tooling (CI, not bundled)
cargo install rcodesign        # cross-platform macOS notarization (CI host)
# tauri signer keypair (one-time)
npx tauri signer generate -w ~/.tauri/scrysynth.key
```

## Alternatives Considered

| Category | Recommended | Alternative | When the alternative wins |
|----------|-------------|-------------|---------------------------|
| Graph editor | `@xyflow/react` 12 (deepen use) | Rete.js / LiteGraph / Flume | Never for v2.0 — migration cost >> rebuild value; v12 covers all requested interactions |
| Auto-layout | `elkjs` | `@dagrejs/dagre` | When you need a lighter, faster, simpler layered layout and don't need bus/route-aware layout |
| LLM client | `async-openai` | raw `reqwest` + `serde_json` | If you want the thinnest possible dependency and only need Chat Completions JSON |
| LLM client | `async-openai` | `rig-core` | If you later want a full agent/RAG framework — **not recommended for v2.0** (conflicts with the app-owned orchestration model) |
| Shell icons | `lucide-react` | `@radix-ui/react-icons` / Tabler / Phosphor | If you already standardize on one of those sets; Lucide is the lowest-friction default |
| Shell styling | `@vanilla-extract/css` | Tailwind | If the team strongly prefers utility-first — but it clashes with token-driven instrument theming |
| macOS notarization | `rcodesign` | `xcrun notarytool` | If you sign only on a macOS host with Xcode — `rcodesign` wins in CI-without-Xcode |
| Visual compositing | Strategy A (overlay window) | Strategy B (GPU readback → canvas) | Strategy B wins on Linux/Wayland and as a future uniform single-window path; A wins on cost/feel for macOS+Windows |

## What NOT to Use (v2.0 anti-recommendations)

| Avoid | Why | Use instead |
|-------|-----|-------------|
| A second node-editor framework (Rete/LiteGraph/BaklavaJS) | The graph already runs on `@xyflow/react`; migrating risks the whole foundation for capabilities v12 already has natively | Deepen use of `@xyflow/react` 12.11 |
| Original `dagre` (0.8.5) | Unmaintained | `elkjs` or `@dagrejs/dagre` 3.x |
| `three` / `@react-three/fiber` / PixiJS in the webview as the visual runtime | Violates the separate-visual-runtime constraint; couples visuals to UI frame timing | Separate Bevy sidecar (Strategy A or B) |
| Cross-process GPU texture sharing between Bevy sidecar and the webview | Not practically supported here | Bevy GPU readback + frame stream (Strategy B) |
| A canned SaaS component kit (MUI/Chakra/Mantine/Ant) | Imposes generic design language, fights "focused instrument / graph as hero" identity | Radix primitives + vanilla-extract tokens |
| Tailwind | Utility-first clashes with token-driven, mode-rich instrument theming | `@vanilla-extract/css` |
| `rig-core` / a full agent-RAG framework | Scrysynth already owns session-aware, explainable, human-overridable orchestration; a framework competes with it | `async-openai` behind the existing `PlannerProvider` trait |
| A bare `tokio` dep just for the agent | Tauri already provides an async runtime | `tauri::async_runtime` |
| Bundling a from-source SuperCollider blob into Linux bundles | No official Linux SC binary; large, fragile, distro-dependent | Detect a system SC install on Linux at runtime + document it |
| `.env` for the Tauri signing key | Explicitly unsupported by the updater plugin | `TAURI_SIGNING_PRIVATE_KEY` env var in CI |
| A custom update verifier / disabling signature checks | Tauri's signature scheme is mandatory and cannot be disabled | Official `tauri-plugin-updater` + `tauri signer generate` |

## Version Compatibility Notes

| Package | Compatible with | Notes |
|---------|-----------------|-------|
| `lucide-react@1.21.0` | React `^19.0.0` (also 16/17/18) | peer deps verified on npm 2026-06-26 |
| `@xyflow/react@12.11.1` | React 19, Vite 7, TS 5.8 | Drop-in upgrade from 12.10.2; check the v12 changelog for minor breaking props |
| `bevy@0.19.0` | `wgpu` 29.x, Rust stable | 0.18→0.19 has a migration guide; the visual sidecar is same-crate so one bump |
| `tauri-plugin-updater@2.10.1` (Rust+JS) | Tauri 2.x; Rust ≥1.77.2 | Requires `bundle.createUpdaterArtifacts:true` + inlined `pubkey` |
| `async-openai@0.41.1` | `reqwest`, Tokio/Tauri runtime | Use `rustls-tls` to avoid system OpenSSL on Windows/Linux builds |
| `elkjs@0.11.1` / `@dagrejs/dagre@3.0.0` | Browser/Vite (runs in webview) | Add only if auto-layout is a v2.0 deliverable |
| `@vanilla-extract/css@1.20.1` | Vite 7 (via `@vanilla-extract/vite-plugin`) | Zero-runtime, build-time extraction |

## Confidence Assessment

| Area | Confidence | Reason |
|------|------------|--------|
| Graph UX (§1) | **HIGH** | Native to `@xyflow/react` v12; official docs + current version verified |
| Node library (§2) | **HIGH** | Content/design work on existing SC + ts-rs path; no new stack |
| Visuals behind grid (§3) | **MEDIUM-LOW (HIGH RISK)** | Two viable strategies documented, but cross-OS window compositing is the most rework-prone area → **needs an explicit spike** |
| Shell UI (§4) | **MEDIUM-HIGH** | Versions verified; adoption is incremental, reversible |
| Live agent (§5) | **MEDIUM-HIGH** | `async-openai`/`reqwest` versions verified; clean trait to implement against |
| Cross-platform (§6) | **MEDIUM-HIGH** | Tauri mechanics solid; Linux SC bundling is the documented weak point |
| Notarization + updater (§7) | **HIGH** | Official plugin + docs verified; `rcodesign` path well-trodden |

## Sources (verified 2026-06-26)

- crates.io registry API: `async-openai` 0.41.1, `tauri-plugin-updater` 2.10.1, `rig-core` 0.39.0, `reqwest` 0.13.4, `bevy` 0.19.0, `wgpu` 29.0.3, `tauri-plugin-shell` 2.3.5, `-dialog` 2.7.1, `-opener` 2.5.4 — `max_stable_version` + recent downloads confirmed
- npm registry `dist-tags`: `@tauri-apps/plugin-updater` 2.10.1, `lucide-react` 1.21.0 (peer `react ^19`), `elkjs` 0.11.1, `@dagrejs/dagre` 3.0.0, `dagre` 0.8.5 (unmaintained)
- Tauri v2 docs: `/learn/window-customization/` (transparent windows, `objc2-app-kit` background, `data-tauri-drag-region`), `/reference/config/` (`macOSPrivateApi`, `transparent`, `externalBin` target-triple pattern, `createUpdaterArtifacts`), `/plugin/updater/` (full updater setup, signing, endpoints, custom `macos-universal` target, `tauri-action`) — last-updated dates Nov 2025–Jun 2026
- React Flow docs (`reactflow.dev`): custom nodes/handles/edges, `OnReconnect`/`reconnectEdge`, validation, layout (dagre/elkjs) examples; "What's new" confirms current 12.11.1 (2026-06-22)
- Bevy 0.15 release notes (`bevy.org/news/bevy-0-15/`): **GPU Readback** feature (render-to-texture → CPU readback) — basis for compositing Strategy B; Bevy 0.19.0 is current
- SuperCollider downloads (`supercollider.github.io/downloads`): 3.14.1 macOS universal (signed+notarized) + x64-legacy, Windows x64 (Win10/11, **no arm64**), Linux source-only (gcc≥9) + distro packages

> **Source-confidence note:** the automated `classify-confidence` seam tags generic `webfetch`/brave as LOW/MEDIUM, but the findings above were taken from **first-party official registries and project docs** (crates.io, npm, v2.tauri.app, reactflow.dev, bevyengine.org, supercollider.github.io). These are authoritative; the per-area confidence table reflects real source quality, not the generic provider tier.

---
*Stack research for: Scrysynth v2.0 "Studio-Grade Instrument" — NEW capabilities only*
*Researched: 2026-06-26*
