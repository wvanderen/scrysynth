# Domain Pitfalls

**Domain:** Graph-native, AI-assisted audiovisual performance instrument
**Researched:** 2026-04-11

## Critical Pitfalls

### Pitfall 1: Letting the runtimes become the real source of truth
**What goes wrong:** The SuperCollider node tree, visual engine state, or ad hoc UI state starts carrying meaning that does not exist in the canonical session graph.
**Why it happens:** Teams prototype quickly by "just asking scsynth what exists" or letting the visual runtime own scene/layout/material state because it already has a running graph.
**Consequences:** Session recall becomes lossy, agent actions stop being explainable, undo/redo becomes unreliable, and audio/visual parity breaks during live mutation.
**Warning signs:** Runtime-specific IDs appear in product-level APIs; graph save/load cannot recreate a session exactly; chat actions work only when the engine is already running; UI panels disagree about what is active.
**Prevention:** Make `Session`, `Node`, `Route`, `Bus`, `Macro`, `Scene`, `Variation`, and ownership state app-native first. Treat SuperCollider and the visual engine as adapters that consume compiled plans and emit telemetry/events back into app state. Never make engine objects canonical identifiers.
**Project phase:** Phase 1 - canonical graph/domain model and adapter contract.
**Detection:** Cold-load the same saved session twice and verify identical graph state, compiled runtime plan, and visible control ownership.

### Pitfall 2: Designing graph primitives around engine internals instead of performance semantics
**What goes wrong:** The graph becomes a thin GUI over synth defs, OSC messages, or visual engine nodes instead of a playable instrument.
**Why it happens:** SuperCollider is flexible, so it is tempting to expose low-level buses, groups, node order, and raw parameter names directly as the authoring model.
**Consequences:** Agent mutation becomes unsafe and unreadable, users need engine knowledge to perform, and every runtime swap or primitive revision becomes a breaking migration.
**Warning signs:** Most nodes map 1:1 to engine objects; users must understand add actions or bus indices; prompts need raw DSP details to do useful work; "simple" scene edits require many low-level operations.
**Prevention:** Define a small, stable primitive layer for sources, processors, modulators, routes, scenes, macros, and ownership. Compile those primitives down to SuperCollider groups/buses and visual-runtime structures. Keep low-level engine details behind adapter boundaries.
**Project phase:** Phase 1 - product model; Phase 2 - audio compiler/runtime adapter.
**Detection:** Ask whether a performer can understand an agent mutation diff without knowing SuperCollider node order or visual-engine internals.

### Pitfall 3: Ignoring SuperCollider execution order and bus topology
**What goes wrong:** Effects, modulators, or feedback paths read the wrong signal or stale data because synths are created in the wrong order or share buses carelessly.
**Why it happens:** scsynth evaluates nodes in tree order; `In.ar`, `InFeedback`, groups, add actions, and bus reuse have strict semantics that do not forgive casual dynamic patching.
**Consequences:** Silent FX chains, one-block-late feedback, unstable modulation, intermittent "it worked last time" routing bugs, and live-set distrust.
**Warning signs:** Effects randomly stop hearing sources after graph edits; hot-swapping routes changes sound unexpectedly; feedback patches differ between reloads; routing correctness depends on creation timing.
**Prevention:** Compile graph routes into explicit group topology and private buses. Reserve deterministic regions for source, modulation, FX, transfer, and feedback. Never rely on incidental creation order. Build validation rules that reject impossible or ambiguous routing before commands reach scsynth.
**Project phase:** Phase 2 - audio runtime foundation.
**Detection:** Automated tests that diff the compiled node/group/bus plan for common graph edits; runtime telemetry for group placement, bus allocation, and feedback usage.

### Pitfall 4: Treating asynchronous engine operations as if they were instant
**What goes wrong:** The app issues dependent actions before synth defs, buffers, sidecars, or visual resources are actually ready.
**Why it happens:** Tauri commands, sidecar startup, SuperCollider buffer/synthdef loading, and engine initialization all have async boundaries, but instrument UX encourages immediate interaction.
**Consequences:** Missing synth defs, failed scene recalls, first-trigger glitches, half-applied agent edits, and brittle startup/shutdown behavior.
**Warning signs:** First play after launch is flaky; resource-heavy patches fail only sometimes; retrying the same action immediately works; startup logic depends on sleeps/timeouts.
**Prevention:** Model readiness explicitly per adapter and per asset. Use completion messages/acknowledgements instead of timed delays. Queue graph mutations against declared engine states like `booting`, `ready`, `draining`, and `failed`.
**Project phase:** Phase 2 - runtime lifecycle and command pipeline.
**Detection:** Remove all startup sleeps in tests; require positive acknowledgements for SynthDef loads, buffer readiness, and visual scene activation.

### Pitfall 5: Using Tauri IPC like a high-rate realtime transport
**What goes wrong:** The UI thread, Rust backend, or runtime bridge gets flooded with parameter updates, meters, waveform data, or visual telemetry.
**Why it happens:** Tauri commands/events are great for app orchestration, but not for sample-accurate or very high-frequency control traffic; JSON-heavy chatter adds overhead and jitter.
**Consequences:** UI jank, control lag, dropped events, unstable automation feel, and agent actions that arrive too late to be musical.
**Warning signs:** Metering or macro moves make the UI stutter; latency rises with graph size; identical sessions feel worse on lower-end laptops; frontend receives large bursts of tiny events.
**Prevention:** Keep Tauri for coarse orchestration, state sync, and user intent. Use dedicated low-latency channels between the app core and runtimes for dense control streams, and aggregate/throttle telemetry before it reaches the webview. The canonical graph should update at semantic boundaries, not every control sample.
**Project phase:** Phase 2 - transport architecture; Phase 5 - performance hardening.
**Detection:** Measure end-to-end latency for macro gesture -> audio change and runtime telemetry -> UI update under load, not just idle conditions.

### Pitfall 6: Failing to define human/agent ownership at the control surface level
**What goes wrong:** The AI can mutate the graph or parameters in ways that collide with the performer mid-gesture, or the human cannot quickly reclaim control.
**Why it happens:** Ownership is treated as a future UX concern instead of a first-class system rule spanning chat, graph edits, macros, scenes, and emergency stop behavior.
**Consequences:** Trust collapse during performance, accidental "fighting the instrument," unreadable histories, and pressure to disable the agent entirely.
**Warning signs:** Agent and human can write the same target with no arbitration; there is no visible "who owns this now" state; override is implemented as social convention instead of enforced logic.
**Prevention:** Encode ownership in the session model. Every mutable target should support explicit states like human-owned, agent-suggested, shared, and locked. Define reclaim, veto, timeout, and confirmation rules before implementing rich agent mutation.
**Project phase:** Phase 1 - domain model; Phase 4 - agent collaboration loop.
**Detection:** Simulate concurrent edits from performer and agent and verify deterministic outcomes plus visible provenance.

### Pitfall 7: Over-coupling audio and visuals for "perfect sync"
**What goes wrong:** The visual runtime is forced to mirror audio-engine structure too closely, or audio waits on graphics work.
**Why it happens:** AV instruments invite a single-graph fantasy where one runtime owns timing, structure, and execution for both media.
**Consequences:** Visual engine choice becomes constrained, audio reliability is endangered by graphics load, and both sides become harder to evolve independently.
**Warning signs:** Visual nodes reuse audio IDs as execution truth; graphics frame drops perturb audio behavior; adding a visual feature requires audio-graph changes with no musical reason.
**Prevention:** Synchronize through shared session abstractions, transport cues, and scene/macro events, not by making one engine host the other. Audio should remain authoritative for sonic timing; visuals should subscribe to semantic state plus timing cues and degrade gracefully if rendering load spikes.
**Project phase:** Phase 3 - visual runtime integration.
**Detection:** Verify that visuals can restart or lag briefly without corrupting audio state or session truth.

## Moderate Pitfalls

### Pitfall 8: Building the patch editor before the playable instrument loop
**What goes wrong:** Time goes into rich graph editing, custom node UIs, and layout behavior before the core "load session -> play -> mutate -> recover" loop works.
**Why it happens:** Graph-native products make visual editing feel like the product, but live trust comes from runtime behavior and control safety first.
**Prevention:** Ship a constrained graph editor early: inspectable, minimally editable, and strongly typed. Prioritize audio foundation, scene recall, macros, and visible mutation history before deep freeform patch authoring.
**Project phase:** Phase 1 through Phase 2.
**Warning signs:** Weeks of node-dragging UX with no reliable scene transition demo; every roadmap conversation centers on canvas affordances instead of instrument behavior.

### Pitfall 9: Underestimating scene transition semantics
**What goes wrong:** Scene/variation changes pop, click, orphan resources, or leave visuals and audio in mismatched states.
**Why it happens:** "Scene" sounds like a saved snapshot, but live performance transitions need rules for ramping, quantization, tail handling, ownership carryover, and resource warmup.
**Prevention:** Define transition semantics explicitly: what crossfades, what hard-cuts, what persists, what preloads, and what ownership survives scene changes. Treat scene change as a compiled transition plan, not a blind state overwrite.
**Project phase:** Phase 3 - scene system; Phase 5 - performance polish.
**Warning signs:** Scene recall works only when stopped; transitions are described as "we'll just diff the graph" with no policy model.

### Pitfall 10: No resource budgeting for greenfield runtime primitives
**What goes wrong:** Primitive count, polyphony, bus count, buffers, GPU load, or telemetry volume grows until sessions collapse unpredictably.
**Why it happens:** Greenfield systems often assume the first elegant abstraction will remain cheap at scale.
**Prevention:** Put budgets in the compiler and session validator: max voices per primitive, max active routes, buffer pool limits, visual node budgets, and degraded-mode behavior when limits are hit. SuperCollider server options such as bus channels, buffers, max nodes, block size, and real-time memory should be part of product configuration, not hidden defaults.
**Project phase:** Phase 2 - runtime config; Phase 5 - scaling/performance.
**Warning signs:** Large sessions fail only on some machines; adding one more node causes sudden instability; runtime config remains mostly default values with no profiling basis.

### Pitfall 11: Cross-platform audio assumptions leaking into product design
**What goes wrong:** Linux/JACK, macOS device pairing, or Windows API/driver differences break low-latency expectations or device selection UX.
**Why it happens:** Desktop Tauri feels cross-platform at the shell layer, but SuperCollider audio behavior is still platform- and driver-specific.
**Prevention:** Design device management as a first-class subsystem with explicit backend diagnostics, not a settings afterthought. Test on at least one real machine per target OS early. Expose actionable messages for sample-rate mismatch, unavailable devices, and unsupported low-latency modes.
**Project phase:** Phase 2 - audio bootstrap and settings.
**Warning signs:** Audio boot works only on the dev machine; device selection copy assumes one API model; QA happens after core runtime decisions are baked in.

## Minor Pitfalls

### Pitfall 12: Missing provenance for agent-driven edits
**What goes wrong:** The performer cannot tell what the agent changed, why it changed, or how to undo only that change.
**Prevention:** Record semantic mutation history with author, target, reason, confidence, and inverse operation. Show it in graph and conversation views using the same event source.
**Project phase:** Phase 4 - agent UX and trust.
**Warning signs:** Undo is global and opaque; chat claims differ from actual graph changes.

### Pitfall 13: Making emergency recovery too coarse
**What goes wrong:** The only safe recovery path is killing the whole session or fully muting output.
**Prevention:** Support layered recovery: mute route, bypass effect chain, suspend agent writes, freeze scene transitions, restart a single runtime adapter, and full panic only as the last resort.
**Project phase:** Phase 2 - runtime controls; Phase 5 - live safety polish.
**Warning signs:** Operator docs say "if it gets weird, reboot"; a visual-runtime fault requires tearing down audio.

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Phase 1 - Canonical session graph | Runtime state sneaks into product model | Freeze app-native IDs and compile adapters from graph -> runtime, never reverse |
| Phase 1 - Ownership rules | Human/agent arbitration left implicit | Define reclaim, veto, confirmation, and timeout rules in the schema |
| Phase 2 - SC runtime adapter | Order-of-execution and bus bugs | Compile deterministic groups/buses and test route diffs |
| Phase 2 - Engine lifecycle | Async boot/load races | Require readiness acknowledgements, not timers |
| Phase 2 - Desktop shell transport | IPC overload from dense telemetry/control | Separate orchestration traffic from realtime control streams |
| Phase 3 - Visual runtime integration | Audio and visuals become mutually blocking | Share semantics and cues, not execution ownership |
| Phase 3 - Scene system | Blind graph diff causes bad transitions | Compile transitions with policy for tails, ramps, preload, and ownership |
| Phase 4 - Agent mutation | Agent becomes an unbounded patch programmer | Restrict mutations to stable primitives and policy-checked actions |
| Phase 5 - Live reliability | No graceful degradation under load | Add budgets, degraded modes, subsystem restart paths, and panic layers |

## Sources

- Tauri docs - Calling Rust from the Frontend (commands async behavior, events, channels), last updated 2025-11-19: https://tauri.app/develop/calling-rust/ - HIGH
- Tauri docs - State Management (shared state, mutex use, async caveats), last updated 2025-05-07: https://tauri.app/develop/state-management/ - HIGH
- Tauri docs - Embedding External Binaries / sidecars (bundling, permissions, process spawning), last updated 2026-01-07: https://tauri.app/develop/sidecar/ - HIGH
- Tauri docs - Capabilities (permission boundaries, window/webview scope), last updated 2025-08-01: https://tauri.app/security/capabilities/ - HIGH
- SuperCollider docs - Order of execution (groups, add actions, feedback, bus semantics): https://doc.sccode.org/Guides/Order-of-execution.html - HIGH
- SuperCollider docs - Server Architecture (nodes, groups, buses, buffers, server options): https://doc.sccode.org/Reference/Server-Architecture.html - MEDIUM, because the page notes some parts are outdated but the core server model remains authoritative
- SuperCollider docs - Synchronous and Asynchronous Execution (completion messages, async resource loading): https://doc.sccode.org/Guides/Sync-Async.html - HIGH
- SuperCollider docs - Audio device selection (platform-specific device/driver behavior): https://doc.sccode.org/Reference/AudioDeviceSelection.html - HIGH
