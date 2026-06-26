# Feature Research

**Domain:** Graph-native desktop co-creative human/AI audiovisual instrument — **v2.0 "Studio-Grade Instrument" new features** layered on a shipping v1
**Project:** Scrysynth
**Researched:** 2026-06-26
**Confidence:** HIGH (table stakes/differentiators grounded in VCV Rack Fundamental, Bespoke Synth, Tauri, Anthropic tool-use docs; MEDIUM where based on established pro-tool conventions)

## Scope Of This Document

This research covers ONLY the **new v2.0 features**. The shipped v1 features (canonical session graph, conversation/graph/performance view switching, scene recall, variation save/restore, cross-domain macros, agent safety scaffold, real SuperCollider audio execution, minimal GPU-free visual sidecar, CoreMIDI + OSC learn, session-aware agent orchestration with deterministic planner) are treated as **dependency context**, not re-researched as table stakes. The v1 feature analysis is preserved in the prior `FEATURES.md` (now superseded by this document for v2 planning).

The seven active v2.0 requirements (from `PROJECT.md`) drive this research:
1. Pro-grade focused shell
2. Graph UX rebuild
3. Curated modular node library (~12–16 nodes)
4. Visuals behind the grid (richer Bevy runtime)
5. Live provider-backed agent orchestration (AGNT-01R carry-over)
6. Cross-platform builds (Windows / Linux / Intel / universal macOS)
7. Full Developer ID notarization + auto-updater

## Feature Landscape

### Table Stakes (Users Expect These In A Credible v2)

A v1 that proved the foundation does not get credit for these again — a studio-grade modular instrument that ships without them feels unfinished next to Bespoke Synth, VCV Rack, and Ableton.

| Feature | Why Expected | Complexity | Dependencies (on shipped v1) | Confidence |
|---------|--------------|------------|------------------------------|------------|
| **Curated modular node library (~12–16 nodes)** | Every modular synth ships a coherent primitive set. VCV Rack's curated "Fundamental" library and Bespoke's built-in modules are the reference bar. A patch tool with only a generic "synth" node is not a modular instrument. | HIGH | SuperCollider adapter + OSC bundles; canonical graph; node inspector; synthdef generation path | HIGH |
| **Graph UX rebuild — fluent patching** | Bespoke, VCV Rack, Comfy UI, Blender shader nodes all support drag, edge connect/disconnect/reconnect, multi-select, and keyboard shortcuts. Patching friction is the #1 thing that breaks a "studio-grade" feel. | MEDIUM | `@xyflow/react` graph view; canonical graph; node inspector; ownership/freeze model (must gate edits) | HIGH |
| **Pro-grade focused shell** | The milestone explicitly mandates retiring the "card webui" feel. Pro instruments (Ableton, Blender, Resolume) use progressive disclosure, a hero canvas, docked secondary panels, and keyboard-first actions. | MEDIUM | Conversation/graph/performance views; vanilla-extract design tokens; panel/layout components | MEDIUM (conventions) |
| **Live provider-backed agent (AGNT-01R)** | v1 shipped the provider-agnostic planner boundary specifically to land this. A "co-creative AI instrument" whose AI is deterministic-only does not satisfy the product promise. | MEDIUM | Provider-agnostic planner boundary (AGNT-02R/03R); typed-command gates; ownership/approval scaffold; conversation history | HIGH (API patterns) |
| **Cross-platform builds** | Bespoke, VCV Rack, Ableton, and SuperCollider itself all ship Windows/macOS/Linux. An Apple-Silicon-only audio instrument in 2026 is not "studio-grade." | MEDIUM-HIGH | Tauri packaging (currently ad-hoc, aarch64-only); sidecar bundling (`scrysynth-visual` + SuperCollider via `externalBin`); release pipeline | HIGH |
| **Developer ID notarization + auto-updater** | v1 deferred this explicitly. A "studio-grade" release a performer installs once and updates safely requires signed, notarized, auto-updating builds. | MEDIUM | Tauri updater plugin; release pipeline; signing infrastructure | HIGH (Tauri docs) |

### Differentiators (Scrysynth-Specific Strengths)

These are not expected of a generic modular synth — they are where Scrysynth earns its identity as a co-creative AV instrument rather than "VCV Rack with a chat box."

| Feature | Value Proposition | Complexity | Dependencies (on shipped v1) | Confidence |
|---------|-------------------|------------|------------------------------|------------|
| **Visuals behind the grid (richer Bevy runtime)** | Bespoke renders oscilloscopes/spectrum inline; Scrysynth's differentiator is an *ambient visual layer behind the entire patch surface* driven by shared session abstractions. This is the visual half of "audiovisual instrument" — most modular synths are audio-only or bolt visuals on as an afterthought. | HIGH | Minimal Bevy sidecar; visual adapter transport; canonical graph + transport + macros; compositing behind the webview or a separate render window | MEDIUM |
| **Live agent editing the graph via tool/function calls** | Bespoke/VCV Rack have no AI agent. Streaming tool-use proposals that map to typed graph commands (create node, set param, connect, recall scene) — with human approval and ownership semantics preserved — is genuinely novel. | MEDIUM-HIGH | Planner boundary; typed-command gates; ownership/approval scaffold; the rebuilt graph surface (so agent edits render fluently) | HIGH (API patterns) |
| **Ownership-aware graph UX** | Patching surfaces in other tools have no concept of "this node is owned by the agent" or "frozen / reclaimable." Surfacing ownership badges, pending-action approvals, and reclaim gestures *inside the patch surface itself* (not only in chat) extends the v1 safety scaffold into the rebuilt graph. | MEDIUM | Ownership model; approval scaffold; the rebuilt graph surface | MEDIUM |

### Anti-Features (Commonly Requested, Problematic For v2)

These appear in adjacent tools or feature requests but should be deliberately excluded from v2.0 to protect scope and the "focused instrument" identity.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **VST / AU / LV2 plugin hosting** | Bespoke hosts VST2/VST3/LV2; users will ask for it. | Massive surface: plugin sandboxing, licensing, per-platform SDK procurement, crash isolation, agent-action complexity. Directly conflicts with the "small curated primitive set" thesis and the existing Out-of-Scope "unrestricted plugin ecosystem." | Curated ~16-node library mapped to SuperCollider primitives. Preserve a future adapter seam, don't build it. |
| **Arbitrary user-authored DSP / custom code nodes** | Power users want to write their own SC code or Python (Bespoke has Python livecoding). | Destroys explainability, agent safety, and the validated-primitive contract. Conflicts with the existing constraint "favor stable primitives over arbitrary internal mutation." | Bounded, validated node library. Open a "recipe/patch template" path instead of raw code. |
| **Plugin/patch marketplace or community library** | "Why can't I download more nodes/patches?" | Content curation burden, identity dilution, IP/reliability review — the existing anti-feature "huge starter content marketplace" still applies. | Ship a focused set of exemplary starter patches/scenes/mappings. |
| **Full DAW timeline / arrangement view** | "Can I sequence a whole song?" | Pulls toward production workflow; conflicts with the "performance-native, not studio replacement" Out-of-Scope boundary. | Step-sequencer node + scenes + variations for live structure. |
| **Multiple competing visual modes / projection mapping** | "Add shaders, video playback, mapping, DMX." | Resolume-class media-server scope; explodes the visual runtime surface. Existing anti-feature "deep projection mapping / media-server scope" still applies. | One cohesive ambient visual layer (scope/spectrum/reactive/generative) behind the grid, through the single Bevy adapter. |
| **Second swappable audio engine (e.g. add a WebAudio path)** | "Why not also support Tone.js for easy patches?" | Undermines the SuperCollider-as-v1-engine decision and the separate-runtime boundary. Two audio engines means two truths. | SuperCollider is the v2 audio engine; keep the adapter boundary. |
| **LLM-generated raw SuperCollider code** | "Let the agent write SynthDefs directly." | Untestable, unsafe, unexplainable; defeats the typed-command gate and the "explainable primitive-based mutation" decision. | Agent acts through validated node + graph-operation tools only. |
| **Skeuomorphic / maximalist instrument skin** | "Make it look like a real modular rack." | Directly conflicts with the v2 "focused instrument, not dense/maximalist" design philosophy and "retire card webui" mandate. | Instrument-grade flat UI with semantic color and clear iconography; graph as hero. |
| **Multiplayer / network collaboration** | "Let two performers co-patch." | Still explicitly Out-of-Scope (v1/v2 are local single-user). Sync/conflict/permissions/latency problems. | Local single-user; design state model so it could be revisited post-v2. |
| **Node count > ~16 in the curated library** | "More choice is better." | Milestone explicitly says "quality over quantity." Too many primitives weaken both UX (palette bloat) and agent reliability. | ~12–16 deeply-designed nodes; expand post-v2 only after the core instrument model is proven. |

## Deep-Dive: Expected Behavior Per New Feature

### 1. Curated Modular Node Library (~12–16 nodes)

**Reference bar:** VCV Rack "Fundamental" (the curated free library) and Bespoke's built-in modules. VCV Fundamental ships ~40 modules, but the *essentials core* a user actually needs to make music is ~12–16. SuperCollider provides the UGen primitives underneath.

**What distinguishes a credible "curated essentials" set (~12–16 nodes):**
- Covers the **full synthesis chain**: source → shape → amplitude → effect → mix → visualize → sequence/control.
- Every node has **rich but bounded parameters (typically 3–8)** — not a single "freq" knob, not a 30-knob panel.
- Each node maps **explicitly to named SuperCollider UGens** (e.g. VCO → `Saw`/`Pulse`/`SinOsc`/`LFTri`; filter → `MoogFF`/`LPF`/`HPF`/`BPF`; ADSR → `EnvGen.kr(Env.adsr(...))`; reverb → `FreeVerb`/`GVerb`; delay → `CombC`/`DelayC`).
- Parameters expose **CV/modulation inputs** (audio-rate and control-rate), not just static knobs — this is what makes it a *modular* library, not a preset browser.
- Follows **established signal conventions** so patches are intuitive: pitch on a 1V/octave-equivalent (frequency) input, gates/triggers as distinct port types, unipolar vs bipolar modulation ranges documented.

**Recommended canonical categories (mapped to SC):**

| Category | Example Nodes | Typical Parameters | SC Primitives |
|----------|---------------|--------------------|---------------|
| Oscillators (sources) | Basic VCO (sine/tri/saw/sqr + FM + sync); FM/PM oscillator | freq, waveform, FM amount, sync, pulse width | `SinOsc`, `LFTri`, `Saw`, `Pulse`, `VarSaw`, `Blip`, `PMOsc` |
| Wavetable | Wavetable VCO | freq, position, FM | `Osc`/`VOsc`/`COsc` with buffer |
| Filters | Multi-mode (LP/HP/BP/Notch); Moog-style ladder | cutoff, resonance, drive, mode, key-track | `LPF`, `HPF`, `BPF`, `BRF`, `MoogFF`, `RLPF` |
| Envelopes | ADSR EG | attack, decay, sustain, release, retrigger, gate | `EnvGen.kr(Env.adsr(...))`, `Linen` |
| LFOs | LFO (sine/tri/saw/sqr/random) | rate, shape, depth, sync, offset | `LFTri`, `LFSaw`, `LFPulse`, `SinOsc` (sub-audio), `LFNoise0/1/2` |
| Effects | Delay, Reverb, Distortion/Drive, Chorus/Flanger | time/size, feedback/damp, wet/dry, tone, depth, rate | `CombC`/`CombL`, `DelayC`, `FreeVerb`, `GVerb`, `Decimator`, `tanh`/`Clip`, chorus via delay+LFO |
| Sequencing | Step sequencer (8–16 steps) | steps, gates/accents per step, CV per step, run/reset, clock | Demand-rate UGens / pattern scheduling |
| Utilities | VCA / mixer, Noise, Quantizer, Scope/Viz, Sample&Hold | gain, scale, scale/key, time/division | `*` (multiply), `WhiteNoise`/`PinkNoise`, quantize fn, `SampleAndHold` |

**Table-stakes node count reality:** ~12–16 = roughly: 2 oscillators + 2 filters + 1 ADSR + 1 LFO + 4 FX (delay/reverb/distortion/chorus) + 1 step-sequencer + 4 utilities (VCA-mix / noise / quantizer / scope) = **15 nodes**. This is the credible "essentials" bar.

**Confidence:** HIGH on category taxonomy and SC mappings (well-documented, stable). MEDIUM on exact final cut (design judgment per product).

### 2. Graph UX Interactions (Patching Surface Rebuild)

**Reference bar:** React Flow / xyflow (the chosen v1 library) plus conventions from Bespoke, VCV Rack, Comfy UI, Blender shader nodes, TouchDesigner.

**Expected interactions in a fluent patching surface:**

| Interaction | Expected Behavior | Implementation Note |
|-------------|-------------------|---------------------|
| Node dragging | Drag node by body; moves smoothly; optional snap-to-grid | `nodesDraggable`, snap grid |
| Edge connect | Drag from an output Handle to an input Handle; ghost edge follows cursor | `onConnect` |
| Edge disconnect / reconnect | Drag from an existing edge endpoint off a port, or onto another valid port | edge update handlers |
| Edge validation (port-type matching) | Audio ports connect only to audio; control/gate ports only to compatible types; invalid drags show "not allowed" cursor, no silent mis-connect | `isValidConnection` predicate + color-coded Handles |
| Multi-select | Drag-select box; shift/cmd-click to add; select-all | `selectionKeyCode`, `multiSelectionKeyCode` |
| Keyboard shortcuts | Delete (Del/Backspace), Copy/Paste (Cmd/Ctrl+C/V), Duplicate (Cmd+D), Undo/Redo (Cmd+Z / Shift+Cmd+Z), Fit-view (Cmd+1 / F), Search/Add palette (Cmd+P or Cmd+K) | standard keymap |
| Pan / zoom | Wheel zoom (cursor-centered), space-drag pan, fit-view, minimap | built-in viewport |
| Bulk operations | Move/delete/connect across multi-selected nodes | operates through typed commands (so ownership/approval/undo apply) |

**Critical v2 requirement — ownership-aware patching:** every connect/disconnect/move must route through the existing typed-command layer so that freeze, reclaim, ownership badges, and approval workflows apply *inside the patch surface*, not only in chat. The rebuilt UX is the chance to make agent/human shared control legible on the canvas itself.

**Confidence:** HIGH on interaction conventions (xyflow API + pro-editor norms are well-established). MEDIUM on the exact keymap (product judgment).

### 3. Visuals Behind The Grid (Richer Bevy Runtime)

**Reference bar:** Bespoke Synth renders oscilloscopes, spectrum, and level displays inline with the patch. Scrysynth's differentiator is a *coherent ambient visual layer behind the whole patch surface*, not scattered scopes.

**Typical visual modes (pick a focused subset — do NOT ship all maximally):**

| Mode | What It Shows | Driver |
|------|---------------|--------|
| Oscilloscope/scope | Live waveform of a chosen signal | audio node output |
| Spectrum analyzer | FFT frequency content | audio bus |
| Reactive geometry | Shapes/forms responding to amplitude, envelopes, macros | session macros / transport |
| Generative/ambient | Particle systems, flow fields, post-processing tied to scene state | scene + transport |
| Embedded metering | VU / activity indicators in the canvas background | runtime telemetry |

**Expected behavior of the ambient layer:**
- Z-order: sits **behind** the graph (the patch stays fully legible on top), low opacity or blurred, does not compete for attention.
- Driven by **shared session abstractions** (macros, transport, scenes, audio telemetry) — NOT a second hand-authored visual patch. This is what keeps audio and visuals coherent through one session model.
- **Bespoke-style inline scopes** (a visual mode tied to a specific node) is a complementary, smaller-scope option if a full ambient layer proves too large.
- Architectural decision needed: render **behind the webview** (compositing) vs. a **separate render window**. The v1 sidecar already ships as a separate process; richer rendering likely needs either transparent-window compositing or a dedicated visible render window. Flag for phase-specific research.

**Confidence:** MEDIUM — the visual modes and Bespoke precedent are clear; the exact compositing architecture behind a Tauri webview needs phase-specific spike.

### 4. Pro-Grade Focused Shell

**Reference bar:** Ableton Live (single-session hero, docked panels, keyboard transport), Blender (progressive disclosure, command palette, keyboard-first), VS Code (command palette Cmd+K/P, theming), Resolume (performance-native density). Contrast: "card webui" (marketing-flavored, big touch targets, centered layouts, hover flourishes).

**Defining patterns of a focused instrument shell:**

| Pattern | Expected Behavior |
|---------|-------------------|
| Graph as hero | The patch canvas is the dominant surface; panels are secondary and dockable. |
| Progressive disclosure | Inspector/transport/chat collapse or auto-hide; only relevant controls surface per context. |
| Command palette (Cmd+K / Cmd+P) | Fuzzy-search every action: add node, recall scene, toggle panel, run macro, freeze/reclaim. |
| Keyboard-first actions | Single-key shortcuts in focus modes (Blender-style); transport, navigation, patching all keyboard-reachable. |
| Consistent iconography | Custom icon set with a coherent visual language; semantic color for signal types and ownership state. |
| Theming | Dark instrument aesthetic; tokens for mode states, ownership colors, signal-type colors. |
| Intelligent screenspace | Panels adapt to context; explicit focus modes (graph / chat / performance). |
| Anti-card-webui | No big bouncy cards, no marketing layout, no generic SaaS density. Instrument-grade, legible, dense where density helps. |

**Confidence:** MEDIUM — conventions are well-established across pro tools; exact visual identity is a design judgment for a UI-SPEC phase.

### 5. Live Provider-Backed Agent (AGNT-01R)

**Reference bar:** Anthropic Claude tool-use + streaming API (the canonical pattern; OpenAI function-calling is analogous). v1 already proved the provider-agnostic planner boundary with deterministic/mock proposals.

**Expected behavior of a live agent in a creative instrument:**

| Concern | Expected Pattern |
|---------|------------------|
| Tool/function calling for graph edits | Define one tool per node type + graph operation (`create_oscillator`, `set_param`, `connect_nodes`, `recall_scene`, `run_macro`), each with a JSON `input_schema`. Agent emits `tool_use`; app maps each to a typed command through the existing gate. |
| Streaming proposals | Stream text deltas to the chat in real-time; parse `input_json_delta` (partial tool args) to show a **live preview of proposed edits** before approval. |
| Provider integration | Anthropic TS SDK (or Rust HTTP); keep it behind the v1 provider-agnostic boundary so OpenAI/others can swap in. |
| Latency / fallback | Stream for responsiveness; on `overloaded_error` / timeout / rate-limit, fall back to the deterministic/mock planner so the session never deadlocks mid-performance. Cap turns per request. |
| Safety preservation | Ownership badges, freeze/reclaim, high-risk-action approval, and action history all still apply — the live provider changes *where plans come from*, not the safety contract. |
| Explainability | Each tool call → structured action schema → graph diff → conversation link (already modeled in v1). |
| Cost/budget awareness | Tool-use adds tokens (tool schemas + system prompt); bound context window per request (v1 already does bounded session context). |

**MUST-NOT:** do NOT let the agent emit raw SuperCollider code. Tool calls map to validated node/graph operations only. This preserves the "explainable primitive-based mutation" decision.

**Confidence:** HIGH on API patterns (current Anthropic docs). MEDIUM on UX of live preview/approval (product judgment).

### 6. Cross-Platform + Distribution

**Reference bar:** Tauri v2 updater + signing docs; Bespoke/VCV Rack cross-platform releases; Apple notarization + Windows code-signing norms.

**Expected for studio-grade v2 distribution:**

| Concern | Expected Behavior |
|---------|-------------------|
| Targets | Windows (x86_64), macOS (universal aarch64+x86_64 — v1 was aarch64-only), Linux (x86_64; AppImage as primary, .deb/.rpm optional). |
| Auto-updater | Tauri `tauri-plugin-updater`: mandatory update signing via a generated Ed25519-style keypair; public key embedded in `tauri.conf.json`; private key in CI env (`TAURI_SIGNING_PRIVATE_KEY`). Endpoint serves a static JSON (per `{{target}}-{{arch}}`) or a dynamic server (200+JSON / 204 no-update). Per-platform artifacts: Linux `.AppImage` + `.sig`; macOS `.app.tar.gz` + `.sig`; Windows MSI/NSIS `.exe` + `.sig`. |
| macOS notarization | Developer ID Application cert; `notarytool` submission; `stapler`; hardened runtime; universal binary. Replaces v1's ad-hoc signing. |
| Windows signing | Code-signing cert (OV or EV); signed MSI/NSIS installers; avoids SmartScreen warnings. |
| Bundled runtimes per target | SuperCollider `scsynth` + the Bevy visual sidecar must be bundled **per platform/arch** via Tauri `externalBin` (sidecar) with target triples. |
| Audio device APIs | SC handles I/O, but device discovery differs: macOS CoreAudio (v1 verified), Windows WASAPI/ASIO, Linux ALSA/JACK/PipeWire. Device-enumeration UAT must run on each platform. |

**Confidence:** HIGH on Tauri updater/signing mechanics (current official docs). MEDIUM-HIGH on per-platform audio device behavior (needs per-target UAT).

## Feature Dependencies

```text
[shipped v1: canonical session graph + SC adapter + typed-command gates + ownership scaffold]
   │
   ├──requires──> Curated modular node library (maps to SC UGens via existing synthdef path)
   │                  └──enhances──> Live provider agent (tools = node/graph operations)
   │
   ├──requires──> Graph UX rebuild (@xyflow/react already in v1)
   │                  └──requires──> Ownership-aware patching (freeze/reclaim/approval route through commands)
   │                  └──enhances──> Curated node library (palette + inspector consume node taxonomy)
   │                  └──enhances──> Live provider agent (agent edits render fluently on the rebuilt surface)
   │
   ├──requires──> Visuals behind the grid
   │     └──requires──> shipped v1 minimal Bevy sidecar + visual adapter transport
   │     └──requires──> shared session abstractions (transport, macros, scenes, audio telemetry)
   │     └──spike-needed──> compositing-behind-webview vs separate render window
   │
   ├──requires──> Live provider agent (AGNT-01R)
   │     └──requires──> shipped v1 provider-agnostic planner boundary + approval scaffold
   │     └──enhances──> Curated node library (agent tool vocabulary)
   │
   └──requires──> Cross-platform builds + notarization + auto-updater
         └──requires──> shipped v1 Tauri packaging + sidecar bundling
         └──requires──> per-platform SC + Bevy sidecar binaries
         └──conflicts-with──> ad-hoc-signing assumptions in v1 release path
```

### Dependency Notes

- **Curated node library requires the v1 SC adapter and synthdef path:** nodes are domain objects that compile to SuperCollider; the execution boundary already exists. Adding nodes is primarily taxonomy + DSP-mapping + inspector work, not new runtime plumbing.
- **Graph UX rebuild is the load-bearing dependency for the *feel* of v2:** the live agent's edits, the curated nodes' palette/inspector, and the ownership scaffolding all become legible only on a fluent rebuilt surface. Sequence the rebuild before (or tightly interleaved with) the node library and live agent.
- **Live provider agent requires the v1 planner boundary (verified):** the safety contract (typed commands, ownership, approval) is unchanged — only the plan *source* moves from deterministic to live. This de-risks AGNT-01R substantially.
- **Visuals behind the grid has an unresolved architectural spike:** behind-webview compositing vs separate render window. This is the single most likely phase to need research flags.
- **Cross-platform + notarization + updater conflict with v1's ad-hoc-signing release path:** the v1 release pipeline (aarch64-only, ad-hoc) must be replaced, not extended. Treat as a distinct workstream with its own CI matrix.

## MVP Definition For v2.0

### Launch With (v2.0 Must-Haves)

- [ ] **Curated modular node library (~12–16 nodes)** — without it, this is not a modular instrument. Map explicitly to SuperCollider UGens; cover the full synthesis chain.
- [ ] **Graph UX rebuild — fluent patching** — drag/connect/reconnect, edge validation, multi-select, keyboard shortcuts, all routed through typed commands (ownership-aware).
- [ ] **Pro-grade focused shell** — graph as hero, command palette, progressive disclosure, keyboard-first, retire card-webui feel.
- [ ] **Live provider-backed agent (AGNT-01R)** — stream proposals, tool-call graph edits through existing gates, deterministic fallback.
- [ ] **Cross-platform builds** — Windows, Linux, universal macOS.
- [ ] **Developer ID notarization + auto-updater** — signed, notarized, auto-updating on all targets.

### Add After Validation (v2.x)

- [ ] **Visuals behind the grid (full ambient layer)** — high-complexity; if behind-webview compositing proves risky, ship inline Bespoke-style scopes first, full ambient layer in v2.x.
- [ ] **Ownership-aware patching polish** — badges/gestures embedded deeper in the patch surface (basic version ships with the graph rebuild).
- [ ] **Wavetable / FM oscillator node** — extends the curated library after the essentials set is proven.

### Future Consideration (v3+)

- [ ] **VST/AU/LV3 plugin hosting** — large separate workstream; preserve the adapter seam, do not build.
- [ ] **Custom user-authored DSP nodes / Python livecoding** — conflicts with explainable-primitive thesis; defer.
- [ ] **Patch/plugin marketplace** — defer per existing anti-feature.
- [ ] **Multiplayer / network collaboration** — still Out-of-Scope; revisit only after v2 instrument model is proven.
- [ ] **Projection mapping / media-server-class visual output** — defer per existing anti-feature.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority | Why |
|---------|------------|---------------------|----------|-----|
| Graph UX rebuild | HIGH | MEDIUM | P1 | Load-bearing for v2 feel; gates node library + agent legibility |
| Curated node library (~12–16) | HIGH | HIGH | P1 | Without it, not a modular instrument |
| Pro-grade focused shell | HIGH | MEDIUM | P1 | Explicit v2 identity mandate |
| Live provider agent (AGNT-01R) | HIGH | MEDIUM | P1 | Completes the co-creation promise; v1 boundary de-risks it |
| Cross-platform builds | HIGH | MEDIUM-HIGH | P1 | Studio-grade release requirement |
| Notarization + auto-updater | MEDIUM | MEDIUM | P1 | Required for a real installed-and-updated release |
| Visuals behind the grid | MEDIUM-HIGH | HIGH | P2 | Differentiator; can ship incrementally (scopes → ambient) |
| Ownership-aware patching polish | MEDIUM | MEDIUM | P2 | Enhances the P1 graph rebuild |

**Priority key:** P1 = must-have for v2.0 launch; P2 = should-have, add when possible / ship incrementally.

## Competitor / Reference Feature Analysis

| Feature | Bespoke Synth | VCV Rack (Fundamental) | SuperCollider (raw) | Scrysynth v2 Plan |
|---------|---------------|------------------------|---------------------|-------------------|
| Curated node library | Large built-in set | ~40 curated Fundamental modules | UGens only (no UI) | ~12–16 deeply-designed nodes mapped to SC UGens |
| Graph patching UX | Freeform 2D canvas | Eurorack-style modules + cables | Code only | `@xyflow/react` rebuilt surface, ownership-aware |
| Visuals in/behind patch | Inline scopes/spectrum | Scope/Viz modules | n/a | Ambient layer behind whole grid (differentiator) |
| Live AI agent | None | None | None | Live provider, tool-called graph edits, human-approval (differentiator) |
| Plugin hosting | VST2/VST3/LV2 | Plugin ecosystem | Quarks | **Deferred** — curated primitives only |
| Cross-platform | Win/Mac/Linux | Win/Mac/Linux | Win/Mac/Linux | Win/universal-macOS/Linux (new in v2) |
| Auto-update | Manual / nightly | Manual | Manual | Tauri signed auto-updater (new in v2) |
| Performance control | MIDI/OSC mapping | MIDI/OSC mapping | MIDI/OSC | v1 already; v2 keeps + refines |

## Complexity Notes For Requirements Definition

| Area | Notes |
|------|-------|
| Curated node library | The hard part is *curation* — choosing ~12–16 nodes that compose into real music — and mapping each cleanly to SC UGens with bounded, CV-exposed params. Not a DSP-engineering problem (SC provides the DSP). |
| Graph UX rebuild | xyflow provides the interaction primitives; the work is (a) our node taxonomy + port-type system + `isValidConnection`, (b) routing every gesture through typed commands so ownership/undo apply, (c) the keyboard map + command palette. |
| Pro-grade shell | Mostly UX/IxD + design-system work, not heavy engineering. The risk is under-investing in it and shipping a "card webui v2." |
| Live provider agent | Heavily de-risked by the v1 planner boundary. Main work: provider SDK wiring, streaming + partial-JSON preview, fallback policy, and UAT of latency/approval under live conditions. |
| Visuals behind the grid | Highest technical uncertainty (compositing architecture). Recommend a spike phase before committing to behind-webview vs separate window. |
| Cross-platform + distribution | Bureaucratic not algorithmic: per-target CI matrix, signing certs, notarization, per-platform sidecar binaries, audio-device UAT on Windows/Linux. Plan buffer for per-platform audio quirks (WASAPI/ASIO, JACK/PipeWire). |
| Sequencing | The graph UX rebuild should land before (or interleaved with) the node library and live agent — it is the surface those features render on. |

## Sources

- Project context: `/Users/eggfam/dev/scrysynth/.planning/PROJECT.md` (HIGH)
- Prior v1 feature research (superseded for v2 planning): `.planning/research/FEATURES.md` v1 (HIGH)
- VCV Rack Fundamental — official module manual (canonical curated modular library; module list + parameters): https://vcvrack.com/Fundamental (HIGH)
- Bespoke Synth — README + project page (cross-platform, VST hosting, MIDI/OSC, live-patchable, visuals in patch): https://github.com/BespokeSynth/BespokeSynth (HIGH)
- SuperCollider — official site/docs (UGen primitives for oscillators/filters/envelopes/effects): https://supercollider.github.io/ and https://docs.supercollider.online (HIGH)
- Tauri v2 Updater plugin — official docs (signing, endpoints, per-platform artifacts, permissions): https://v2.tauri.app/plugin/updater/ (HIGH)
- Anthropic Claude — tool use overview + streaming docs (function calling, `input_json_delta`, `overloaded_error`, stop reasons): https://docs.anthropic.com/en/docs/build-with-claude/tool-use/overview and /streaming (HIGH)
- React Flow / xyflow — interaction model (onConnect, isValidConnection, Handle, selection keys, keyboard map): https://reactflow.dev/ and https://xyflow.com/ (MEDIUM-HIGH; specific sub-pages rotated, API stable)
- Pro-tool conventions (Ableton Live, Blender command palette/keyboard-first, Resolume, Comfy UI, TouchDesigner): established industry norms (MEDIUM)

## Confidence Notes

- **HIGH** on: curated node taxonomy + SC UGen mapping (VCV Fundamental + SuperCollider docs); Tauri updater/signing mechanics (current official docs); Anthropic tool-use/streaming API patterns (current official docs); cross-platform expectations (Bespoke/VCV precedent); table-stakes vs anti-feature classification.
- **MEDIUM** on: visuals-behind-grid compositing architecture (needs a spike — behind-webview vs separate window); pro-shell exact visual identity (design judgment for a UI-SPEC phase); live-agent preview/approval UX (product judgment); the exact final ~12–16 node cut (design judgment).
- **LOW**: none. No recommendation is made without either a current official source or an established industry convention.
