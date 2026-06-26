# Project Research Summary

**Project:** Scrysynth
**Domain:** Graph-native desktop audiovisual instrument — v2.0 "Studio-Grade Instrument" milestone (NEW capabilities layered on a shipping v1)
**Researched:** 2026-06-26
**Confidence:** HIGH overall (one HIGH-RISK area: visuals-behind-grid compositing)

> **Scope note:** This summary covers ONLY the seven new v2.0 features. The shipped v1 (Phases 1–11) is treated as immutable integration context, not re-summarized. All four researchers grounded their findings in the **actual shipped code** (`Cargo.toml`, `package.json`, `src-tauri/src/`, `src/`, `tauri.conf.json`), not the prior aspirational STACK.md.

## Executive Summary

Scrysynth v2.0 transforms a verified v1 foundation into a studio-grade, fluently-playable audiovisual instrument. The seven target features are: (1) a curated ~12–16 node library, (2) a rebuilt graph surface (draggable nodes, flexible edges), (3) richer Bevy visuals composited **behind** the grid, (4) a focused pro shell, (5) a live LLM-backed agent (AGNT-01R carry-over), (6) cross-platform builds, and (7) full Developer ID notarization + auto-updater. Experts build this kind of milestone as **integration onto existing seams**: every v2 feature plugs into v1's typed-command pipeline, Rust-owned canonical state, and adapter boundaries — none rewrites the core mutation model. The dominant pattern across all four research files is **catalog-as-data**: refactor v1's three hardcoded allowlists (synthdef names, parameter names, runtime targets) into one Rust-owned node-catalog table that drives compiler, validation, palette, inspector, and agent context. This unblocks features 1, 2, 3, and 5 simultaneously and is the single highest-leverage v2 decision.

The recommended approach is an 8-phase sequence driven by hard dependency chains: **catalog → graph rebuild → visuals spike → visuals full → shell + agent (parallel) → cross-platform → notarization+updater**. The graph rebuild is load-bearing for v2 "feel" — agent edits, node palette, and ownership badges all become legible only on a fluent rebuilt surface. The live LLM agent is substantially de-risked by v1's provider-agnostic `PlannerProvider` boundary; it inherits all safety scaffolding and only changes *where plans come from*. Cross-platform and notarization must be last because they require stable final binaries and a frozen feature set.

The key risks cluster in three areas. (1) **Visuals-behind-grid compositing** is the single highest-uncertainty v2 item — transparent-webview + child-window stacking is per-platform and unreliable on Linux/Wayland; it MUST be settled by a one-day vertical spike before the phase is planned in detail. (2) **Linux SuperCollider bundling** has no official prebuilt binary — it is a product decision (require system install vs. bundle vs. vendor) that should be made at phase start, not discovered mid-build. (3) **The signing graph** (app + sidecar + bundled SC) must be designed, not discovered — ad-hoc signing cannot ship an updater, and notarization rejects any unsigned bundled binary. Mitigations for all three are documented in PITFALLS.md (V2-5, V2-6, V2-7) and are addressed by the suggested phase order.

## Key Findings

### Recommended Stack

v2.0 is a **deepening and completion** milestone, not a stack migration. The shipped v1 stack (Tauri 2 + Rust + React 19.1 + Vite 7 + TypeScript 5.8 + `@xyflow/react` 12.10.2 + SuperCollider 3.14 + Bevy 0.18.1 + `rosc`/`midir`/`ts-rs`/`zod`/`zustand`/`immer` + versioned-JSON persistence) is the integration baseline. Two upgrades and six additions cover all seven features. **Notable context:** the prior greenfield STACK.md (2026-04-11) was aspirational in places — it listed Radix, vanilla-extract, elkjs, `tokio`, `rusqlite`, TS 6.0, Vite 8, React 19.2, and SQLite that **did not actually ship**. v2 builds on the real code, not the older research.

**Core v2 additions / upgrades:**
- `@xyflow/react` 12.10.2 → **12.11.1** — graph rebuild uses native v12 features (custom multi-handle nodes, `onReconnect` + `reconnectEdge`, `isValidConnection`, `onBeforeDelete`); no second framework. (HIGH)
- `bevy` 0.18.1 → **0.19.0** — richer ambient visuals; same-crate second binary, one `Cargo.toml` bump. (HIGH on upgrade)
- `lucide-react` 1.21.0 — consistent iconography for the focused shell. (HIGH)
- `@radix-ui/react-*` primitives + `@vanilla-extract/css` + `@vanilla-extract/recipes` — accessible unstyled primitives + typed zero-runtime theming; clean adoption since v1 shipped plain CSS only. (MEDIUM-HIGH)
- `elkjs` 0.11.1 — auto-layout, only when explicitly in scope. (MEDIUM-HIGH)
- `async-openai` 0.41.1 + `reqwest` 0.13.4 (`rustls-tls`) — streaming + tool/function calling against any OpenAI-compatible endpoint, behind the existing `PlannerProvider` trait. (MEDIUM-HIGH)
- `tauri-plugin-updater` 2.10.1 (Rust+JS) + `tauri-plugin-process` — official updater + relaunch. (HIGH)
- `rcodesign` (CI tooling) — macOS notarization without Xcode. (HIGH)

**Critical version notes:** `TAURI_SIGNING_PRIVATE_KEY` is a real env var, never `.env` (silently ignored). Updater `pubkey` is **inline content**, never a file path. No official Windows arm64 or Linux prebuilt SuperCollider — arm64 Windows is out of scope; Linux SC bundling is a product decision.

### Expected Features

**Must have (v2.0 table stakes):**
- **Curated modular node library (~12–16 nodes)** — without it, not a modular instrument. Covers full synthesis chain (sources → filters → envelopes → LFOs → FX → sequencing → utilities); each node maps explicitly to named SuperCollider UGens with bounded CV-exposed params (3–8 per node). Reference bar: VCV Rack "Fundamental" essentials.
- **Graph UX rebuild — fluent patching** — drag, edge connect/disconnect/reconnect, multi-select, keyboard shortcuts (Cmd+K palette, Cmd+Z undo, Del, etc.), `isValidConnection` port-type validation, all routed through typed commands (ownership-aware).
- **Pro-grade focused shell** — graph as hero, persistent chat sidebar, progressive-disclosure panels, keyboard-first, retire "card webui" feel. Always-visible safety chrome is non-negotiable.
- **Live provider-backed agent (AGNT-01R)** — streaming proposals, tool-called graph edits through existing typed-command gates, deterministic fallback, ownership preserved.
- **Cross-platform builds** — Windows x86_64, universal macOS (aarch64+x86_64), Linux x86_64 (AppImage primary).
- **Developer ID notarization + auto-updater** — signed, notarized, auto-updating on all targets.

**Should have (differentiators):**
- **Visuals behind the grid** — ambient Bevy layer behind the whole patch surface (Bespoke-Synth-style); most modular synths are audio-only. Can ship incrementally (inline scopes first → full ambient layer in v2.x if compositing proves risky).
- **Ownership-aware patching** — badges/gestures embedded in the patch surface itself (not only chat), extending the v1 safety scaffold into the rebuilt graph.

**Anti-features (deliberately excluded):** VST/AU/LV2 plugin hosting; arbitrary user-authored DSP / custom code nodes; plugin/patch marketplace; full DAW timeline; multiple competing visual modes / projection mapping; second swappable audio engine (e.g. WebAudio); LLM-generated raw SuperCollider code; skeuomorphic/maximalist skin; multiplayer; node count > ~16.

### Architecture Approach

v2 plugs into v1's existing seams; **nothing shortcuts the `intent → validated command → domain event → session state → adapter projection → runtime side effects` pipeline.** The single architectural rule preserved unchanged is that Rust owns canonical state and adapters consume it. The dominant new artifact is a **Rust-owned `NodeCatalogEntry` table** (`domain/node_catalog.rs`) serialized through `ts-rs` — the single source of truth for node-type ports, parameters, synthdef mapping, validation rules, and UI metadata, eliminating the multi-place enum-update tax. Two new typed commands (`MoveNode`, `RerouteEdge`) extend the graph-edit surface so xyflow remains a controlled component over a Zustand mirror of canonical state. The visual sidecar gains `--behind` / `--headless-stream` modes (rather than a second binary). The live LLM is one new `PlannerProvider` impl in a fallback chain.

**Major components:**
1. **Node Catalog** (`domain/node_catalog.rs`, NEW) — drives compiler, synthdef planner, validation, palette, inspector, agent context via `ts-rs`.
2. **Graph command layer** (EXTENDED) — adds `MoveNode` + `RerouteEdge`; `validate_route` reads catalog ports (consistent by construction).
3. **Audio topology compiler + synthdef planner** (MODIFIED) — catalog-driven dispatch replaces hardcoded enum arms + alias table + `unreachable!()` landmines.
4. **Visual sidecar** (MODIFIED) — borderless behind-window mode + richer GPU render + async streaming protocol; frame channel separate from control bus.
5. **Live LLM provider** (`application/providers/live_llm.rs`, NEW) — one `PlannerProvider` impl + streaming channel + `validate_planner_proposal` safety step + fallback chain.
6. **Focused shell** (`App.tsx` REBUILT) — graph-hero layout, persistent chat sidebar, progressive-disclosure panels, always-visible safety chrome.
7. **Bundling + release** (MODIFIED) — per-triple sidecars + scsynth, updater config, full signing graph, CI matrix.

### Critical Pitfalls

The top risks from PITFALLS.md (full list: V2-1 through V2-16):

1. **Node library trips three hardcoded v1 allowlists (V2-1)** — `synthdef_resource` ends in `unreachable!()` (panic on audio boot path); `normalize_parameter_name` rejects params outside a 9-entry allowlist; `validate_runtime_target` string-matches a fixed set. **Avoid:** refactor to one `NodeCatalogEntry` spec table; replace `unreachable!()` with a real `Err`; add a compiler conformance test that boots real `scsynth` for every catalog entry.
2. **Draggable nodes break canonical-state invariant (V2-2, V2-14)** — without canonical positions, reload snaps layout back; without a dedicated `MoveNode` command that bypasses `AudioRuntimeManager`, every drag frame recompiles the full SuperCollider topology → dropouts and envelope re-triggers. **Avoid:** add `position: Option<(f64,f64)>` to `Node`; new `SetNodeLayout`/`MoveNode` command variant that skips `reconcile_graph_edit`; write on drag-end, not per-frame.
3. **xyflow edges bypass the typed-command layer (V2-3)** — `useEdgesState` makes the canvas show edges that don't exist in `SessionDocument.routes`; audio routes the old way; ownership/validation never consulted. **Avoid:** keep xyflow a controlled component over the Zustand mirror; wire `onConnect`/`onReconnect`/`onBeforeDelete` to typed commands; add a property test that xyflow edges == `SessionDocument.routes` after every interaction.
4. **Live LLM emits schema-valid but invariant-violating commands (V2-4)** — hallucinated node IDs, oversized batches (50+ commands), mutations on locked nodes, self-referential macro chains; v1's discrete confidence thresholds don't map to LLM's continuous values. **Avoid:** insert `validate_planner_proposal(&proposal, &session)` before mutation (unknown-ID rejection, batch cap, dry-apply topology check); re-derive the confidence → risk-tier → approval-threshold mapping; fuzz the validator.
5. **Visuals-behind-grid compositing is per-platform (V2-5, V2-11, V2-16)** — transparent webview + child-window stacking is unreliable on Linux/Wayland; the synchronous `send_and_wait` visual protocol saturates under 60fps load; `SCRYSYNTH_VISUAL_MODE=minimal` env inheritance silently degrades packaged visuals. **Avoid:** spike compositing on all three OSes before committing; convert visual adapter protocol to async/streaming; scrub env from the packaged child process.

Secondary pitfalls (V2-6 sidecar triples + SC bundling; V2-7 signing graph + `.env`/path/key gotchas; V2-8 node-ID-prefix theming; V2-9 safety-surface demotion; V2-10 chat-only authoring drift; V2-12 macOS min vs Bevy GPU; V2-13 ts-rs enum drift; V2-15 CSP null vs agent HTTP) are detailed in PITFALLS.md and mapped to specific phases below.

## Implications for Roadmap

The four researchers converged on a single dependency-respecting phase order. Suggested structure: **8 phases**, with one explicit compositing spike gated as a fast decision point and one parallelization opportunity mid-milestone.

### Phase 1: Node Catalog Foundation
**Rationale:** Everything else references real node types. The catalog must exist before the graph rebuild, the visual scene, and the agent context packet can reference node types. Highest leverage, lowest risk.
**Delivers:** `domain/node_catalog.rs` with ~12–16 entries; refactor of `synthdef_resource` / `normalize_parameter_name` / `validate_runtime_target` into catalog-driven dispatch; `catalog_id` added to `Node` with v1-schema migration; per-entry `.scsyndef` files under `resources/synthdefs/v2/`; ts-rs export + exhaustive-switch TS lint.
**Addresses:** Curated modular node library feature.
**Avoids:** V2-1 (allowlist panics), V2-13 (ts-rs drift), V2-8 (node-ID-prefix theming — taxonomy drives appearance).

### Phase 2: Graph UX Rebuild
**Rationale:** Depends on Phase 1 (custom typed-handle nodes need the catalog). The graph rebuild is load-bearing for v2 "feel" — agent edits, node palette, and ownership badges become legible only here. Flips `nodesDraggable` on.
**Delivers:** Custom `CatalogNode` + `TypedEdge` React components; `MoveNode` + `RerouteEdge` typed commands; `Node.position` canonical; `isValidConnection` (UX hint) + Rust `validate_route` (authority); xyflow as controlled component over Zustand mirror; reconnect-revert test harness; ownership-aware patching v1.
**Uses:** `@xyflow/react` 12.11.1, `elkjs` (if auto-layout in scope), `session-projections.ts`.
**Avoids:** V2-2 (positions not canonical), V2-3 (edges bypass), V2-14 (topology reapply on drag).

### Phase 3: Visuals Compositing Spike (DECISION GATE)
**Rationale:** Single highest-uncertainty v2 item. A one-day macOS proof of concept confirms or kills **Strategy A** (borderless Bevy window behind a transparent webview) before the phase is planned in detail. If A fails on macOS, default to **Strategy B** (headless Bevy → GPU readback → frame stream → canvas) everywhere.
**Delivers:** Working spike validating (1) `macOSPrivateApi:true` + transparent webview CSS lets the Bevy window show through, (2) two windows stay geometry-synced on move/resize/focus, (3) input passes through to the graph. Decision recorded for Phase 4.
**Addresses:** De-risks the visuals-behind-grid feature.
**Avoids:** V2-5 discovered late (HIGH recovery cost — "may require switching approach; significant rewrite").

### Phase 4: Visuals Behind the Grid
**Rationale:** Depends on Phase 1 (catalog-driven visual compiler) and Phase 3 (spike decision). Implements whichever compositing mode the spike validates, with the other as fallback. Couples async-protocol refactor to the rich-renderer work.
**Delivers:** `--behind` sidecar mode (Strategy A) + `--headless-stream` mode (Strategy B fallback); `SetWindowFrame` protocol message (A); `ipc::Channel<FrameChunk>` + canvas painter (B); richer Bevy render (shaders, post-FX); async fire-and-forget visual protocol replacing `send_and_wait`; `SCRYSYNTH_VISUAL_MODE` scrubbed from packaged child; visual-runtime cold-start test per platform.
**Uses:** Bevy 0.19.0, `wgpu` 29.x, existing `bevy_runtime.rs` + `bevy_sidecar.rs`.
**Avoids:** V2-5 (compositing per-platform), V2-11 (protocol saturation), V2-16 (env inheritance), V2-12 (macOS GPU requirements — validated here).

### Phase 5: Pro-Grade Focused Shell  *(parallelizable with Phase 6)*
**Rationale:** Depends on Phase 2 (new graph surface) + Phase 4 (transparent visuals surface). Rebuilding the shell first would mean rebuilding it twice. The new shell wraps the new transparent-graph surface.
**Delivers:** `FocusedShell` layout (graph hero, persistent chat sidebar, progressive-disclosure panels); **safety-surface inventory as first deliverable** (panic/reclaim, frozen indicator, runtime health, pending-action count, ownership state — always visible, never behind disclosure); lucide iconography; vanilla-extract design tokens (dark instrument, mode states, ownership/signal-type colors); command palette (Cmd+K); UAT scenario: "complete live set using only graph + performance, no chat."
**Uses:** `lucide-react` 1.21.0, `@radix-ui/react-*`, `@vanilla-extract/css` + `recipes`.
**Avoids:** V2-9 (safety surfaces demoted), V2-10 (chat-only authoring drift).

### Phase 6: Live Provider-Backed Agent — AGNT-01R  *(parallelizable with Phase 5)*
**Rationale:** Depends on Phase 1 (catalog schemas in agent context packet). Independent of the shell. Heavily de-risked by v1's `PlannerProvider` boundary — the live LLM is one new trait impl in a fallback chain; safety is fully inherited.
**Delivers:** `LiveLlmProvider: PlannerProvider` (`application/providers/live_llm.rs`) using `async-openai` streaming + tool-calling; tool schemas generated from serde command types; `send_agent_message_stream` Tauri command with `ipc::Channel<AgentStreamEvent>`; **`validate_planner_proposal` step** (unknown-ID rejection, batch cap, dry-apply topology check); **re-derived confidence → risk-tier → approval-threshold mapping**; fallback to `ParserPlannerProvider` on failure; API-key handling in Rust core (keychain/file), never in frontend or `tauri.conf.json`; fuzz test for the validator.
**Uses:** `async-openai` 0.41.1, `reqwest` 0.13.4 (`rustls-tls`), existing `tauri::async_runtime` (no new bare `tokio`).
**Avoids:** V2-4 (invariant-violating commands), V2-15 (CSP null vs agent HTTP — coordinated with Phase 5 security pass).

### Phase 7: Cross-Platform Builds
**Rationale:** Depends on Phase 4 (sidecar modes settled) and the stable feature set. Per-triple sidecar + scsynth sourcing is the real work; CI matrix is the mechanics. **The SC bundling strategy is a product decision that should be made at phase start, not discovered mid-build.**
**Delivers:** Per-triple sidecar binaries (`scrysynth-visual-{aarch64|x86_64}-apple-darwin`, `-x86_64-pc-windows-msvc.exe`, `-x86_64-unknown-linux-gnu`); SC bundling decision documented per platform (require system install on Linux + document; bundle official 3.14.1 universal on macOS + x64 on Windows); platform-specific `tauri.{os}.conf.json` merges; GitHub Actions matrix with `tauri-apps/tauri-action`; clean-OS smoke test on each target booting audio to `Ready`; `midir`/`rosc` per-OS device-enumeration UAT. Windows arm64 explicitly out of scope (no upstream SC).
**Avoids:** V2-6 (sidecar triples + SC bundling runtime failure).

### Phase 8: Developer ID Notarization + Auto-Updater
**Rationale:** LAST. Requires Phase 7 cross-platform artifacts to sign/notarize/update. Every bundled binary must be at its final location/name — doing this earlier means re-signing every time a sidecar or SC binary moves.
**Delivers:** Three explicit sub-tasks sequenced (not one): (1) acquire Developer ID Application cert + App Store Connect API key + Windows Authenticode cert; (2) sign every bundle binary (app + each sidecar triple + bundled scsynth) + notarize + staple (`rcodesign` for CI-without-Xcode); (3) `tauri-plugin-updater` + `tauri-plugin-process` with generated Ed25519 keypair, `TAURI_SIGNING_PRIVATE_KEY` in CI env (never `.env`), inline `pubkey` in config, `createUpdaterArtifacts: true`, static `latest.json` generated by `tauri-action` (never hand-edited), custom `macos-universal` target. Update UI with `ipc::Channel<DownloadEvent>` progress. Real upgrade/downgrade test against the published manifest.
**Avoids:** V2-7 (signing graph + ad-hoc signing + `.env`/path/key gotchas).

### Phase Ordering Rationale

- **Catalog (P1) before Graph rebuild (P2):** graph nodes need real typed ports before the surface that renders them is rebuilt.
- **Catalog (P1) before Visuals (P4):** the visual compiler reads catalog entries to map nodes → visual elements/params.
- **Catalog (P1) before Agent (P6):** the agent context packet must carry catalog schemas for valid tool-calling.
- **Graph rebuild (P2) before Shell (P5):** the shell composes the graph surface; rebuilding the shell first means rebuilding it twice.
- **Visuals spike (P3) before Visuals full (P4):** de-risk the single most uncertain v2 decision with a day of work, not a phase.
- **Visuals full (P4) before Shell (P5):** the shell wraps the transparent-visuals + graph surface.
- **Shell (P5) ∥ Agent (P6):** independent; parallelizable after P1 + P2 land.
- **Cross-platform (P7) before Notarization+Updater (P8):** you sign/notarize the artifacts you can build; updater needs the cross-platform matrix.

### Research Flags

**Phases likely needing deeper research during planning (`/gsd-plan-phase --research-phase`):**
- **Phase 3 (Visuals Compositing Spike):** HIGHEST uncertainty. Needs its own focused spike on webview transparency + GPU child window per-platform before the phase can be planned in detail.
- **Phase 4 (Visuals Behind the Grid):** Bevy 0.18→0.19 migration guide; headless render-to-texture + GPU readback API surface (confirm during planning); async visual protocol design.
- **Phase 6 (Live Agent):** `validate_planner_proposal` design + confidence-threshold recalibration are first-class design tasks, not generic provider wiring; tool-schema generation from serde types needs validation against OpenAI-compatible tool-calling expectations.
- **Phase 7 (Cross-Platform):** SC bundling decision is a product-level research item; Linux `.deb`/`AppImage` dependency strategy may need a dedicated spike.

**Phases with standard patterns (skip research-phase):**
- **Phase 1 (Node Catalog):** compiler already understood; it's a refactor of hardcoded knowledge into data.
- **Phase 2 (Graph UX Rebuild):** xyflow v12 is well-documented (custom nodes/handles/edges, `onReconnect`, `isValidConnection`, `onBeforeDelete` all confirmed).
- **Phase 5 (Focused Shell):** layout-only; no core/data change; conventions well-established across pro tools.
- **Phase 8 (Notarization + Updater):** Tauri docs are thorough; main risk is sequencing and signing-graph design, not unknown technology.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | **HIGH** | Versions verified against crates.io + npm registries (2026-06-26); first-party Tauri/React Flow/Bevy/SuperCollider docs; one MEDIUM-LOW area (visuals compositing) gated behind Phase 3 spike |
| Features | **HIGH** | Table-stakes/differentiators grounded in VCV Rack Fundamental, Bespoke Synth, Tauri, Anthropic tool-use docs; MEDIUM on exact final ~12–16 node cut (design judgment) and live-agent preview/approval UX |
| Architecture | **HIGH** | Integration points grounded in actual v1 codebase (file-level); catalog-as-data pattern is a refactor of existing hardcoded knowledge; MEDIUM-HIGH on visuals recommendation pending Phase 3 spike |
| Pitfalls | **HIGH** | 16 pitfalls grounded in actual v1 code (`audio/synthdefs.rs`, `runtime_manager.rs`, `agent_planner.rs`, `bevy_sidecar.rs`, `tauri.conf.json`); cross-checked against current Tauri 2.10, SC 3.14, xyflow v12 docs |

**Overall confidence:** **HIGH** — with one explicit HIGH-RISK area (visuals-behind-grid compositing) mitigated by a mandatory Phase 3 spike before Phase 4 is planned.

### Gaps to Address

- **Visuals compositing strategy (Phase 3 spike):** Cannot be resolved by research alone — requires a working macOS proof of concept. If Strategy A fails, Phase 4 defaults to Strategy B (frame stream); roadmapper should preserve budget for either path.
- **Linux SuperCollider bundling (Phase 7 product decision):** No official prebuilt binary. Three viable options (require system install + document; build from source in CI and bundle; distro-specific package dependency) — the choice should be made at Phase 7 start, not by the roadmapper. Recommend: detect system SC install on Linux at runtime + document, rather than bundling a from-source blob.
- **Live-agent confidence thresholds (Phase 6):** v1's discrete `{0.7, 0.85, 0.9}` thresholds won't map to LLM continuous values; re-derivation is empirical work that belongs in Phase 6 execution, not pre-planning.
- **macOS minimum version vs. Bevy/wgpu GPU requirements:** `tauri.conf.json` currently says `11.0`; Bevy 0.19 / wgpu 29 may require macOS 12+. Validate during Phase 4 and lock during Phase 7.
- **Exact final ~12–16 node cut:** Design judgment per product; recommend a SPEC/UI phase decide the canonical list (the category taxonomy + SC UGen mapping is HIGH-confidence; the final selection is MEDIUM).
- **Windows Authenticode cert lead time:** EV/OV cert acquisition is its own lead-time item — start early in Phase 8 (or even Phase 7) to avoid blocking release.

## Sources

### Primary (HIGH confidence)
- **Scrysynth v1 codebase (grounded):** `src-tauri/src/{audio/synthdefs.rs, audio/compiler.rs, audio/runtime_manager.rs, application/agent_planner.rs, application/agent_command.rs, visual/bevy_sidecar.rs, visual/adapter.rs, bin/scrysynth-visual.rs, domain/session.rs, application/graph_edit.rs, lib.rs}`, `src/{App.tsx, components/session/GraphViewport.tsx, components/audio/PrimitivePalette.tsx, lib/session-client.ts}`, `src-tauri/Cargo.toml`, `package.json`, `src-tauri/tauri.conf.json`
- **Tauri 2 official docs** (v2.tauri.app, updated Nov 2025–Jun 2026): window-customization, configuration reference (`macOSPrivateApi`, `transparent`, `externalBin`, `createUpdaterArtifacts`, `hardenedRuntime`), updater plugin (`pubkey`/`endpoints`, `TAURI_SIGNING_PRIVATE_KEY`, custom `macos-universal` target, `ipc::Channel` progress), macOS code signing, shell plugin
- **SuperCollider official** (supercollider.github.io): 3.14.1 macOS universal (signed+notarized) + x64-legacy, Windows x64 (Win10/11, **no arm64**), Linux source-only (gcc≥9) + distro packages
- **React Flow / xyflow v12 docs** (reactflow.dev, updated Jun 2026): custom nodes/handles/edges, `OnReconnect`/`reconnectEdge`/`IsValidConnection`/`OnBeforeDelete`, layout (dagre/elkjs) examples
- **Anthropic Claude tool-use + streaming docs** (docs.anthropic.com): function calling, `input_json_delta`, `overloaded_error`, stop reasons — canonical pattern for AGNT-01R
- **crates.io registry** (verified 2026-06-26): `async-openai` 0.41.1, `tauri-plugin-updater` 2.10.1, `reqwest` 0.13.4, `bevy` 0.19.0, `wgpu` 29.0.3, `rig-core` 0.39.0 (rejected for v2)
- **npm registry** (verified 2026-06-26): `@tauri-apps/plugin-updater` 2.10.1, `lucide-react` 1.21.0 (peer `react ^19`), `elkjs` 0.11.1, `@dagrejs/dagre` 3.0.0, `dagre` 0.8.5 (unmaintained)
- **VCV Rack Fundamental** (vcvrack.com/Fundamental): canonical curated modular library reference
- **Bespoke Synth** (github.com/BespokeSynth/BespokeSynth): cross-platform, VST hosting, MIDI/OSC, live-patchable, visuals in patch

### Secondary (MEDIUM confidence)
- **Bevy 0.15 release notes** (bevy.org/news/bevy-0-15): GPU Readback feature — basis for compositing Strategy B; Bevy 0.19.0 is current
- **Pro-tool conventions** (Ableton Live, Blender command palette/keyboard-first, Resolume, Comfy UI, TouchDesigner): established industry norms for focused-instrument shell
- `rcodesign` / `apple-codesign`: cross-platform macOS notarization without Xcode (recommended CI tooling)
- **Prior v1 research files** (`.planning/research/{STACK,FEATURES,ARCHITECTURE,PITFALLS}.md`, 2026-04-11): baseline this extends; aspirational in places (see STACK.md correction note)

---
*Research completed: 2026-06-26*
*Ready for roadmap: yes*
