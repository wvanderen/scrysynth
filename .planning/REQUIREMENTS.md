# Requirements: Scrysynth

**Defined:** 2026-06-26
**Core Value:** The instrument must let a human and agent shape a live audiovisual session together through conversation, graph structure, and direct performance control without losing legibility or human override.

## v2.0 Requirements (Studio-Grade Instrument)

Requirements for the v2.0 milestone. Each maps to a roadmap phase (numbering continues from v1.0; v2 phases start at Phase 12). Traceability is filled during roadmap creation.

### Curated Modular Node Library

- [ ] **NODES-01**: A data-driven node catalog (single source of truth) lets new node types be added without touching hardcoded compiler allowlists
- [ ] **NODES-02**: User can add oscillator, filter, envelope, LFO, and utility (VCA/mixer/noise/quantizer) nodes covering the full synthesis chain, each with rich bounded parameters mapped to SuperCollider UGens
- [ ] **NODES-03**: User can add effect nodes (delay, reverb, distortion, chorus/flanger) with characteristic + wet/dry parameters mapped to SC
- [ ] **NODES-04**: User can add a step-sequencer node with per-step gate/CV and clock transport
- [ ] **NODES-05**: Every audio node exposes CV/modulation inputs (audio-rate + control-rate ports), making patches modular, not preset-based

### Graph UX Rebuild

- [ ] **GRAPH-01**: User can drag nodes freely on the canvas; positions persist via a layout command without re-triggering audio reconciliation
- [ ] **GRAPH-02**: User can create edges by dragging between ports, with port-type validation/matching
- [ ] **GRAPH-03**: User can disconnect, reconnect, and reroute edges fluently
- [ ] **GRAPH-04**: User can multi-select nodes/edges and use keyboard shortcuts for common patching actions
- [ ] **GRAPH-05**: Ownership/freeze/reclaim/approval states are visible and actionable inside the patch surface (not only in chat)

### Visuals Behind the Grid

- [ ] **VISUAL-01**: A compositing spike validates the behind-webview render strategy (transparent overlay vs streamed texture) per-OS before full build (decision gate)
- [ ] **VISUAL-02**: A richer GPU-accelerated Bevy visual runtime replaces the minimal GPU-free sidecar
- [ ] **VISUAL-03**: The visual layer renders as an ambient layer behind the graph surface (Bespoke-Synth-style), driven by shared session abstractions (transport, macros, node signals)
- [ ] **VISUAL-04**: Visuals remain behind a separate adapter process boundary (audiovisual coherence via shared session state, not engine coupling)

### Pro-Grade Focused Shell

- [ ] **SHELL-01**: The shell is restructured around the graph as hero, with the conversation as a docked sidebar (not co-equal panes)
- [ ] **SHELL-02**: Secondary surfaces (inspector, palette, transport, settings) use progressive disclosure via collapsible menus/panels that get out of the way
- [ ] **SHELL-03**: The visual identity reaches "professional creative software" quality — iconography, semantic theming, intelligent screenspace — retiring the "card webui" feel
- [ ] **SHELL-04**: Keyboard-first actions and a command palette let common operations happen without mouse drilling

### Live Provider-Backed Agent

- [ ] **AGENT-01**: A live LLM provider is connected to the existing provider-agnostic planner boundary (streaming proposals, tool/function-calling for typed graph commands)
- [ ] **AGENT-02**: Agent proposals are validated (`validate_planner_proposal`) with confidence-threshold handling before reaching typed-command gates, preserving human override safety
- [ ] **AGENT-03**: The agent falls back to the deterministic/mock planner on provider-unavailable or invalid responses, with clear diagnostics
- [ ] **AGENT-04**: Streaming agent proposals render live previews of proposed graph edits on the rebuilt surface, with human approval

### Cross-Platform Builds

- [ ] **PLATFORM-01**: The app builds and runs for Windows (x64), with SuperCollider + visual sidecar bundled/located per-platform
- [ ] **PLATFORM-02**: The app builds and runs for Linux (x64), with a SuperCollider strategy decided (bundle vs runtime-detect; no official prebuilt binary)
- [ ] **PLATFORM-03**: The app builds for Intel and universal macOS (alongside existing Apple Silicon)
- [ ] **PLATFORM-04**: Per-platform audio-device behavior is verified (WASAPI/ASIO on Windows, JACK/PipeWire on Linux, CoreAudio on macOS)

### Notarization + Auto-Updater

- [ ] **RELEASE-01**: Builds are signed and notarized (Developer ID on macOS, Authenticode on Windows) including all bundled sidecars/binaries
- [ ] **RELEASE-02**: An auto-updater (Tauri updater plugin) checks for, downloads, and applies verified (Ed25519-signed) updates safely
- [ ] **RELEASE-03**: The release pipeline produces signed update artifacts per platform via CI

## Future Requirements

Deferred beyond v2.0. Tracked but not in the current roadmap.

- **VST/AU/LV2 plugin hosting** — preserve an adapter seam only; do not build the hosting surface in v2.
- **Arbitrary user-authored DSP / custom-code nodes** — conflicts with the explainable-primitive contract and agent safety.
- **Patch/plugin marketplace or community library** — content curation burden and identity dilution.
- **Node library expansion beyond ~16 curated nodes** — v2 ships quality over quantity; expand only after the core instrument model is proven.
- **Physical-controller GUI click-through UAT** — carried from v1; perform when hardware is available.

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Full DAW timeline / arrangement view | Pulls toward production workflow; conflicts with performance-native identity |
| Multiple competing visual modes / projection mapping / media-server | Explodes the visual runtime surface; one cohesive ambient layer is the v2 scope |
| Second swappable audio engine (e.g. WebAudio/Tone.js) | Undermines the SuperCollider-as-v1-engine decision; two engines means two truths |
| LLM-generated raw SuperCollider code | Untestable, unsafe, unexplainable; defeats the typed-command gate |
| Skeuomorphic / maximalist instrument skin | Conflicts with the "focused instrument, not dense/maximalist" design philosophy |
| Multiplayer / network collaboration | v1/v2 are explicitly local single-user |
| Unrestricted plugin ecosystem | Would expand surface area before the core instrument model is proven |
| Highly autonomous low-level DSP self-rewriting | The system favors explainable primitive-based mutation |

## Traceability

Which phases cover which requirements. v2.0 phases continue numbering from v1.0 (Phase 11) and start at Phase 12.

| Requirement | Phase | Status |
|-------------|-------|--------|
| NODES-01 | Phase 12 (Node Catalog Foundation) | Pending |
| NODES-02 | Phase 12 (Node Catalog Foundation) | Pending |
| NODES-03 | Phase 12 (Node Catalog Foundation) | Pending |
| NODES-04 | Phase 12 (Node Catalog Foundation) | Pending |
| NODES-05 | Phase 12 (Node Catalog Foundation) | Pending |
| GRAPH-01 | Phase 13 (Graph UX Rebuild) | Pending |
| GRAPH-02 | Phase 13 (Graph UX Rebuild) | Pending |
| GRAPH-03 | Phase 13 (Graph UX Rebuild) | Pending |
| GRAPH-04 | Phase 13 (Graph UX Rebuild) | Pending |
| GRAPH-05 | Phase 13 (Graph UX Rebuild) | Pending |
| VISUAL-01 | Phase 14 (Visuals Compositing Spike) | Pending |
| VISUAL-02 | Phase 15 (Visuals Behind the Grid) | Pending |
| VISUAL-03 | Phase 15 (Visuals Behind the Grid) | Pending |
| VISUAL-04 | Phase 15 (Visuals Behind the Grid) | Pending |
| SHELL-01 | Phase 16 (Focused Shell & Live Agent) | Pending |
| SHELL-02 | Phase 16 (Focused Shell & Live Agent) | Pending |
| SHELL-03 | Phase 16 (Focused Shell & Live Agent) | Pending |
| SHELL-04 | Phase 16 (Focused Shell & Live Agent) | Pending |
| AGENT-01 | Phase 16 (Focused Shell & Live Agent) | Pending |
| AGENT-02 | Phase 16 (Focused Shell & Live Agent) | Pending |
| AGENT-03 | Phase 16 (Focused Shell & Live Agent) | Pending |
| AGENT-04 | Phase 16 (Focused Shell & Live Agent) | Pending |
| PLATFORM-01 | Phase 17 (Cross-Platform Builds) | Pending |
| PLATFORM-02 | Phase 17 (Cross-Platform Builds) | Pending |
| PLATFORM-03 | Phase 17 (Cross-Platform Builds) | Pending |
| PLATFORM-04 | Phase 17 (Cross-Platform Builds) | Pending |
| RELEASE-01 | Phase 18 (Notarization & Auto-Updater) | Pending |
| RELEASE-02 | Phase 18 (Notarization & Auto-Updater) | Pending |
| RELEASE-03 | Phase 18 (Notarization & Auto-Updater) | Pending |

**Coverage:**
- v2.0 requirements: 29 total
- Mapped to phases: 29/29 ✓ (100%)
- Unmapped: 0
- Phase distribution: P12=5, P13=5, P14=1, P15=3, P16=8, P17=4, P18=3

---
*Requirements defined: 2026-06-26*
*Last updated: 2026-06-26 after v2.0 roadmap creation (traceability filled)*
