# Roadmap: Scrysynth

## Milestones

- ✅ **v1.0 Runtime Hardening** — Phases 1–11 (shipped 2026-06-26) — `[archive](milestones/v1.0-ROADMAP.md)`
- 🚧 **v2.0 Studio-Grade Instrument** — Phases 12–18 (in planning)

## v1.0 Phases

<details>
<summary>✅ v1.0 Runtime Hardening (Phases 1–11) — SHIPPED 2026-06-26</summary>

**Foundation (Phases 1–5):**

- [x] Phase 1: Session Core & Recall (6 plans) — completed 2026-04-11
- [x] Phase 2: Playable Audio Graph Foundation (3 plans) — completed 2026-04-11
- [x] Phase 3: Performance Workspace (3 plans) — completed 2026-04-11
- [x] Phase 4: Agent Collaboration Scaffold (3 plans) — completed 2026-04-11
- [x] Phase 5: Visual Sync & Cross-Modal Control Scaffold (3 plans) — completed 2026-04-11

**Runtime Hardening (Phases 6–11):**

- [x] Phase 6: Local Developer Readiness — folded into Phase 11 (DEV-01/02 verified there)
- [x] Phase 7: Real SuperCollider Execution — audible playback + panic + restart verified against local `scsynth`
- [x] Phase 8: Real Visual Runtime Path — minimal `scrysynth-visual` sidecar, JSON-lines protocol, full lifecycle verified
- [x] Phase 9: Hardware Input Runtime Wiring — CoreMIDI + local OSC learn and post-learn routing verified
- [x] Phase 10: Session-Aware Agent Orchestration (1 plan) — bounded context, provider-agnostic planner, typed-command gates, freeze/reclaim verified
- [x] Phase 11: Release Readiness (2 plans) — ad-hoc-signed Apple-Silicon `.app` + `.dmg`, 9/9 packaged-app UAT scenarios passed

**Requirement outcome:** 36/37 v1 requirements verified. `AGNT-01R` (live provider-backed agent) deferred beyond v1 — now in v2.0 as AGENT-01..04.

</details>

## v2.0 Studio-Grade Instrument (In Planning)

**Milestone Goal:** Transform Scrysynth from a verified v1 foundation into a studio-grade, fluently-playable audiovisual instrument — deep modular nodes, a rebuilt graph surface, visuals behind the grid, and a focused pro shell — while landing the v1 carry-overs (live provider-backed agent, cross-platform builds, full Developer ID notarization + auto-updater).

**Granularity:** coarse (compression tolerance applied — the parallelizable Pro Shell and Live Agent are combined into Phase 16; the Visuals Compositing Spike is kept as a standalone Phase 14 decision gate per research HIGH-RISK flag; Cross-Platform and Notarization remain separate because RELEASE needs final stable per-platform binaries from PLATFORM).

**Coverage:** 29/29 v2.0 requirements mapped. No orphans, no duplicates.

### Phase Checklist

- [ ] **Phase 12: Node Catalog Foundation** — Single-source-of-truth node catalog replacing v1 hardcoded allowlists; full synthesis chain (osc/filter/env/LFO/FX/sequencer/utility).
- [ ] **Phase 13: Graph UX Rebuild** — Fluent patching: drag, edge connect/reconnect, multi-select, ownership-aware — all through typed commands; positions canonical.
- [ ] **Phase 14: Visuals Compositing Spike** — One-day macOS PoC validating the behind-webview compositing strategy (DECISION GATE before Phase 15).
- [ ] **Phase 15: Visuals Behind the Grid** — Richer Bevy runtime as ambient layer behind the graph surface, behind a separate adapter boundary.
- [ ] **Phase 16: Focused Shell & Live Agent** — Graph-hero shell + live LLM provider with safety inheritance (parallelizable internally as two plan tracks).
- [ ] **Phase 17: Cross-Platform Builds** — Windows / Linux / Intel / universal macOS with per-platform runtime sourcing and audio-device verification.
- [ ] **Phase 18: Notarization & Auto-Updater** — Signed + notarized bundles + Ed25519-verified auto-update on every target via CI.

### Phase Details

### Phase 12: Node Catalog Foundation
**Goal**: A data-driven node catalog (single source of truth) covers the full synthesis chain — oscillators, filters, envelopes, LFOs, utilities, effects, and a step sequencer — each mapped to SuperCollider UGens, so new node types can be added without touching v1's hardcoded compiler allowlists.
**Depends on**: v1.0 shipped foundation (no v2 phase dependencies; builds on the canonical `SessionDocument` + topology compiler).
**Requirements**: NODES-01, NODES-02, NODES-03, NODES-04, NODES-05
**Success Criteria** (what must be TRUE):
  1. A performer can add oscillator, filter, envelope, LFO, utility, effect, and step-sequencer nodes to a patch and hear them shape audio through SuperCollider.
  2. Every catalog node exposes CV/modulation inputs (audio-rate + control-rate ports) alongside its primary parameters, so patches are modular (signal flows between nodes) rather than preset-based.
  3. Adding a new node type requires editing only the catalog — no changes to v1's hardcoded `synthdef_resource` / `normalize_parameter_name` / `validate_runtime_target` allowlists (and the v1 `unreachable!()` panic path is replaced with a real `Err`).
  4. The catalog drives compiler dispatch, route validation, palette, inspector, and `ts-rs` schema export from one Rust table, verified by a conformance test that boots real `scsynth` for every entry.
**Plans**: 2 plans (coarse granularity — catalog foundation + compiler refactor in P01; sequencer runtime + frontend consumption + real-scsynth conformance test in P02, Wave 2)

Plans:
- [ ] 12-01-PLAN.md — Catalog module + domain reshape + v2 SynthDef authoring + replace 5 hardcoded spots + control-bus allocation + v1 rejection (Wave 1)
- [ ] 12-02-PLAN.md — App-driven step sequencer controller + frontend catalog consumption + conformance test that boots real scsynth per entry (Wave 2, depends on 12-01)

**UI hint**: yes

### Phase 13: Graph UX Rebuild
**Goal**: The patch surface becomes a fluent, draggable, edge-reconnectable graph where every interaction flows through typed commands and persists in canonical state — load-bearing for the v2 "feel" and the surface that agent proposals, the node palette, and ownership badges render onto.
**Depends on**: Phase 12 (custom typed-handle nodes need the catalog).
**Requirements**: GRAPH-01, GRAPH-02, GRAPH-03, GRAPH-04, GRAPH-05
**Success Criteria** (what must be TRUE):
  1. User can freely drag nodes around the canvas; positions persist across reload via a dedicated `SetNodeLayout`/`MoveNode` command that bypasses audio reconciliation, so dragging does not re-trigger topology recompilation or envelope re-triggers.
  2. User can create, disconnect, and reconnect edges by dragging between typed ports, with port-type mismatches rejected both at the canvas (`isValidConnection` UX hint) and at the Rust authority (`validate_route`).
  3. User can multi-select nodes/edges and use keyboard shortcuts for common patching actions (add, delete, undo, copy).
  4. Ownership, freeze, reclaim, and pending-approval states are visible and actionable directly on nodes/edges in the patch surface — not only in chat.
  5. xyflow canvas state always equals canonical `SessionDocument` state after every interaction (no orphan edges, no untracked moves) — verified by a property test.
**Plans**: TBD
**UI hint**: yes

### Phase 14: Visuals Compositing Spike
**Goal**: A one-day macOS proof of concept confirms or kills the borderless Bevy-window-behind-transparent-webview compositing strategy (Strategy A) before Phase 15 is planned in detail — settling the single highest-uncertainty v2 item with a day of work, not a phase.
**Depends on**: Nothing in v2 (standalone decision-gate spike); informs Phase 15 planning.
**Requirements**: VISUAL-01
**Success Criteria** (what must be TRUE):
  1. A borderless Bevy window renders visibly behind a transparent Tauri webview on macOS (`macOSPrivateApi:true` + transparent webview CSS).
  2. The two windows stay geometry-synced on move, resize, and focus.
  3. Mouse/keyboard input passes through the transparent webview to the graph surface underneath.
  4. A decision is recorded (Strategy A valid on macOS → use everywhere with per-OS validation in Phase 15; or A fails → default to Strategy B headless-render-to-texture frame stream everywhere) with evidence, unblocking Phase 15 planning.
**Plans**: TBD

### Phase 15: Visuals Behind the Grid
**Goal**: A richer GPU-accelerated Bevy runtime renders as an ambient visual layer behind the entire graph surface (Bespoke-Synth-style), driven by shared session abstractions, behind a separate adapter process boundary — with the compositing strategy chosen in Phase 14 and the alternate as fallback.
**Depends on**: Phase 12 (catalog-driven visual compiler), Phase 14 (spike decision recorded).
**Requirements**: VISUAL-02, VISUAL-03, VISUAL-04
**Success Criteria** (what must be TRUE):
  1. The ambient visual layer visibly renders behind the entire patch surface during a live session, responding to transport, macros, and node signals.
  2. A richer GPU-accelerated Bevy runtime replaces the v1 minimal GPU-free sidecar (shaders/post-FX that move with the music); the v1 synchronous `send_and_wait` protocol is replaced by an async fire-and-forget streaming protocol.
  3. Visuals run in a separate adapter process — coherence comes from shared session state (transport, macros, node signals), not engine coupling; the visual adapter never owns canonical truth.
  4. The compositing strategy chosen in Phase 14 works on every target OS, with the alternate strategy available as a fallback; visual-runtime cold-start is verified per platform.
**Plans**: TBD
**UI hint**: yes

### Phase 16: Focused Shell & Live Agent
**Goal**: The app shell is restructured around the graph as hero with a persistent chat sidebar, progressive-disclosure panels, keyboard-first operation, and professional creative-software visual identity; the agent is connected to a live LLM provider with full safety inheritance and live proposal previews rendered on the rebuilt surface. (Pro Shell and Live Agent are independent and run as parallel plan tracks inside this phase.)
**Depends on**: Phase 12 (catalog schemas in agent context packet), Phase 13 (rebuilt graph surface for shell + agent previews), Phase 15 (transparent visuals surface that the shell wraps).
**Requirements**: SHELL-01, SHELL-02, SHELL-03, SHELL-04, AGENT-01, AGENT-02, AGENT-03, AGENT-04
**Success Criteria** (what must be TRUE):
  1. The shell shows the graph as hero with the conversation docked as a persistent sidebar (not co-equal panes); secondary surfaces (inspector, palette, transport, settings) collapse out of the way via progressive disclosure.
  2. Always-visible safety chrome (panic/reclaim, frozen indicator, runtime health, pending-action count, ownership state) remains on screen regardless of panel disclosure — never demoted behind a menu.
  3. Keyboard-first actions and a command palette (Cmd+K) let a performer complete a live set using only graph + performance surfaces, without reaching for the chat.
  4. The visual identity reaches "professional creative software" quality — iconography, semantic theming, intelligent screenspace — retiring the "card webui" feel.
  5. The agent connects to a live LLM provider; streaming proposals appear as live graph-edit previews the human can approve or reject, preserving override safety at every step.
  6. Invalid, oversized, or invariant-violating proposals are rejected by `validate_planner_proposal` before reaching typed-command gates; the agent falls back to the deterministic/mock planner with clear diagnostics on provider-unavailable or invalid responses.
**Plans**: TBD
**UI hint**: yes

### Phase 17: Cross-Platform Builds
**Goal**: Scrysynth builds, launches, and produces audio on Windows x64, Linux x64, and Intel/universal macOS, with per-platform SuperCollider + visual-sidecar sourcing decided and per-platform audio-device behavior verified.
**Depends on**: Phase 15 (sidecar modes settled), Phase 16 (stable feature set).
**Requirements**: PLATFORM-01, PLATFORM-02, PLATFORM-03, PLATFORM-04
**Success Criteria** (what must be TRUE):
  1. The app builds and runs on Windows x64 with SuperCollider and the visual sidecar bundled or located per-platform; audio reaches `Ready` on a clean Windows install.
  2. The app builds and runs on Linux x64 with a documented SuperCollider strategy (bundle vs. runtime-detect vs. distro package; no official prebuilt binary) decided at phase start, not discovered mid-build.
  3. The app builds for Intel macOS and universal macOS alongside the existing Apple Silicon target.
  4. Per-platform audio-device behavior is verified (WASAPI/ASIO on Windows, JACK/PipeWire on Linux, CoreAudio on macOS), including `midir`/`rosc` device-enumeration UAT per OS.
**Plans**: TBD

### Phase 18: Notarization & Auto-Updater
**Goal**: Builds are signed, notarized, and auto-updating on every target platform via a CI pipeline that produces verified (Ed25519-signed) update artifacts — closing the v1 carry-over that shipped ad-hoc-signed Apple-Silicon-only with no updater.
**Depends on**: Phase 17 (final stable per-platform binaries at their final locations/names).
**Requirements**: RELEASE-01, RELEASE-02, RELEASE-03
**Success Criteria** (what must be TRUE):
  1. Every bundle binary (app + each sidecar triple + bundled scsynth) is signed and notarized (Developer ID on macOS, Authenticode on Windows) — no unsigned binary anywhere in the bundle.
  2. The Tauri updater plugin checks for, downloads, and applies Ed25519-signed updates safely, with download progress surfaced to the user via `ipc::Channel<DownloadEvent>`.
  3. The release pipeline produces signed update artifacts (including a `latest.json` manifest generated by `tauri-action`, never hand-edited) per platform via CI; a real upgrade/downgrade test against the published manifest passes.
**Plans**: TBD

## Progress

**Execution Order:** Phases execute in numeric order. v2.0 phases run 12 → 13 → 14 → 15 → 16 → 17 → 18. Phase 14 (spike) is a standalone decision gate that may run in parallel with 12/13; Phase 16 runs two parallel plan tracks internally (Shell + Agent).

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Session Core & Recall | v1.0 | 6/6 | Complete | 2026-04-11 |
| 2. Playable Audio Graph Foundation | v1.0 | 3/3 | Complete | 2026-04-11 |
| 3. Performance Workspace | v1.0 | 3/3 | Complete | 2026-04-11 |
| 4. Agent Collaboration Scaffold | v1.0 | 3/3 | Complete | 2026-04-11 |
| 5. Visual Sync & Cross-Modal Scaffold | v1.0 | 3/3 | Complete | 2026-04-11 |
| 6. Local Developer Readiness | v1.0 | — (folded into 11) | Complete | 2026-06-26 |
| 7. Real SuperCollider Execution | v1.0 | 1/1 | Complete | 2026-06-13 |
| 8. Real Visual Runtime Path | v1.0 | 1/1 | Complete | 2026-06-16 |
| 9. Hardware Input Runtime Wiring | v1.0 | 1/1 | Complete | 2026-06-19 |
| 10. Session-Aware Agent Orchestration | v1.0 | 1/1 | Complete | 2026-06-21 |
| 11. Release Readiness | v1.0 | 2/2 | Complete | 2026-06-26 |
| 12. Node Catalog Foundation | v2.0 | 0/2 | Not started | - |
| 13. Graph UX Rebuild | v2.0 | 0/TBD | Not started | - |
| 14. Visuals Compositing Spike | v2.0 | 0/TBD | Not started | - |
| 15. Visuals Behind the Grid | v2.0 | 0/TBD | Not started | - |
| 16. Focused Shell & Live Agent | v2.0 | 0/TBD | Not started | - |
| 17. Cross-Platform Builds | v2.0 | 0/TBD | Not started | - |
| 18. Notarization & Auto-Updater | v2.0 | 0/TBD | Not started | - |

## Backlog

Carry-over candidates beyond v2.0 (tracked, not committed):

- Physical-controller GUI click-through UAT when hardware is available.
- VST/AU/LV2 plugin hosting (adapter seam only in v2).
- Node library expansion beyond ~16 curated nodes (after the core instrument model is proven).
- Multiplayer collaboration (later milestone).

## Deferred Beyond v2

- Multiplayer collaboration.
- Full DAW-style timeline/arrangement.
- Deep projection mapping or media-server workflows.
- General plugin marketplace.
- Arbitrary user-authored DSP / custom-code nodes (conflicts with explainable-primitive contract).
- LLM-generated raw SuperCollider code (defeats the typed-command gate).
- Skeuomorphic / maximalist instrument skin (conflicts with focused-instrument design philosophy).
- Second swappable audio engine (e.g. WebAudio/Tone.js).

---

_For the full v1.0 milestone record, see `.planning/milestones/v1.0-ROADMAP.md` and `.planning/milestones/v1.0-REQUIREMENTS.md`. For shipped-state summary, see `.planning/MILESTONES.md`._
