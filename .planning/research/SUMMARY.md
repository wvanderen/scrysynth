# Project Research Summary

**Project:** Scrysynth
**Domain:** Local-first graph-native audiovisual performance instrument with AI-assisted co-creation
**Researched:** 2026-04-11
**Confidence:** MEDIUM

## Executive Summary

Scrysynth should be built as a local-first desktop instrument, not a chat app, browser toy, or mini-DAW. The research is consistent on the core product shape: a Rust-owned canonical session graph inside a Tauri host, with the UI, agent, SuperCollider audio runtime, and visual runtime all operating against the same typed model. Experts in adjacent tools win by making signal flow visible, performance control immediate, recall reliable, and runtime behavior legible; AI only helps when it is constrained, explainable, and reversible.

The recommended v1 approach is opinionated: prioritize the canonical graph, dependable audio execution, session recall, scenes/macros, controller mapping, and a session-anchored conversational agent that works through a small safe primitive library. Use SuperCollider as the audio engine, but keep its node trees, buses, and runtime IDs behind an adapter. Keep visuals in a separate runtime with shared transport and scene semantics, but do not let visual depth delay the playable audio/control loop.

The biggest risks are architectural drift and live-performance trust failures. If runtime state becomes the real source of truth, if graph primitives leak engine internals, if async/runtime readiness is hand-waved, or if human/agent ownership is left implicit, the product will become hard to explain, hard to recover, and unsafe on stage. Mitigate that early with a strict command -> event -> reduce -> project pipeline, explicit ownership rules, correlation IDs, runtime health visibility, and transactional scene semantics.

## Key Findings

### Recommended Stack

The stack recommendation is strong and coherent: Tauri 2 + Rust should own the desktop control plane and canonical session model; React + TypeScript + Vite should provide the graph/chat/performance UI; SuperCollider should be the first audio runtime via a Rust adapter; SQLite should persist sessions and recovery state; and a separate visual runtime should sit behind the same adapter shape. The main through-line is that app semantics stay stable while runtimes stay replaceable.

**Core technologies:**
- `Tauri 2.10.x` + stable `Rust` — desktop shell, IPC, sidecars, permissions, and process supervision around an app-owned core
- `React 19` + `TypeScript 6` + `Vite 8` — practical UI stack for graph, inspector, conversation, and performance surfaces
- `serde` + `tokio` + `ts-rs` + `zod` — typed command/state contracts with async orchestration and runtime-boundary validation
- `@xyflow/react` + `elkjs` — graph viewport plus structured layout without inventing node-editor infrastructure
- `SuperCollider 3.14.1` + `rosc` — real-time audio execution and OSC control via a Rust-managed adapter
- `SQLite` via `rusqlite` + versioned JSON import/export — durable local persistence plus portable session interchange
- `Bevy 0.18` behind a separate adapter — viable v1 visual runtime, but with lower confidence than the audio path

Critical version/decision requirements: keep Tauri on 2.x, honor the Rust floor required by plugins, prefer `ts-rs` over RC type-bridge tooling in the foundation, and avoid Electron, CRDT sync, graph databases, frontend-owned SQL writes, or webview-native visual runtimes as the canonical path.

### Expected Features

The table stakes are clear: users will expect a visible canonical session graph, low-latency stable audio, direct performance controls, save/load with fast recall, scenes/variations, MIDI/OSC controller mapping, basic AV sync, reversible edits, and runtime feedback. The AI promise belongs in v1 too, but only as a session-anchored mutation layer over safe primitives, not as an unrestricted generator.

**Must have (table stakes):**
- Canonical live session graph with inspectable state, routes, buses, macros, scenes, and runtime status
- Reliable low-latency audio execution with panic-safe transport and direct performance controls
- Save/load sessions with fast recall, plus scene and variation recall for live set structure
- MIDI/OSC learn and hardware mapping for tactile performance
- Basic audiovisual synchronization through shared transport/events
- Session-anchored conversational agent with reversible edits, panic/stop-all, and visible mutation status

**Should have (competitive):**
- Conversation/graph/performance triad over the same object model
- Explainable agent mutations with structured diffs and provenance
- Stable primitive-based mutation library instead of arbitrary low-level rewrites
- Cross-modal macros and scenes coordinating audio, visuals, and control state
- Intent-aware approvals for high-risk actions

**Defer (v2+):**
- Fine-grained shared ownership beyond core human override rules
- Rehearsal branching, compare/merge workflows, and deeper variation management
- Projection mapping, advanced output routing, plugin marketplace, multiplayer collaboration, DAW-style arrangement, cloud-first workflows, and autonomous DSP self-rewriting

### Architecture Approach

Architecture research is the strongest input and should drive the roadmap. Scrysynth should use a Rust app core as the control plane with a canonical session store, command handlers, reducer/validator, event bus, transaction coordinator, ownership/safety policy, persistence, and observability. The UI, agent orchestrator, audio adapter, and visual adapter are all clients of that core. All durable mutations must follow one path: intent -> validated command -> domain events -> state reduction -> adapter projections -> runtime side effects -> runtime feedback.

**Major components:**
1. `Rust app core` — owns canonical session state, invariants, persistence, ownership, and scheduling
2. `Frontend UI` — renders graph, chat, transport, and runtime health; dispatches typed intents only through Tauri IPC
3. `Agent orchestrator` — proposes policy-checked commands from conversation and goals; never mutates session state directly
4. `Audio runtime adapter` — compiles canonical graph diffs into SuperCollider resources, OSC bundles, and health events
5. `Visual runtime adapter` — applies shared scene/control/transport semantics through a separate visual protocol

Architecture through-lines:
- Canonical product model first; runtime models are compiled projections
- Command/event/reducer pipeline is mandatory, not optional
- Transactional scene recall at the session layer, incremental diff application at the runtime layer
- Typed internal Rust event bus for core coordination; Tauri events/channels only at the app edge
- Ephemeral runtime state kept separate from persisted creative state
- Correlation IDs, ownership metadata, and health signals built in from day one

### Critical Pitfalls

The biggest hazards are already known and should shape scope and sequencing more than feature ambition.

1. **Runtime-owned truth** — prevent it by making the session graph canonical and keeping SC/visual IDs out of product state
2. **Engine-shaped product primitives** — prevent it by defining performer-facing primitives first, then compiling to SC groups/buses and visual structures
3. **SuperCollider ordering and bus mistakes** — prevent it with deterministic topology, private buses, validation rules, and compiled route plans
4. **Async readiness races** — prevent it with explicit adapter states, acknowledgements, and queued mutations instead of sleeps/timeouts
5. **Implicit human/agent ownership** — prevent it by encoding ownership, reclaim, veto, and confirmation rules in the domain model before rich agent behavior
6. **Tauri IPC overload and false perfect sync goals** — prevent it by separating semantic orchestration from hot telemetry/control streams and by syncing runtimes through shared semantics, not mutual blocking

## Implications for Roadmap

Based on the combined research, the roadmap should favor control-plane correctness and playable reliability over UI depth or AI breadth.

### Phase 1: Canonical Core and Safety Model
**Rationale:** Everything depends on a stable session vocabulary and mutation pipeline; this is the main dependency choke point.
**Delivers:** Session schema, commands, events, reducers, snapshots, ownership rules, safety policy, correlation IDs, and core observability.
**Addresses:** Canonical graph, session persistence, reversible edits, explainable agent groundwork.
**Avoids:** Runtime-owned truth, engine-leaking product primitives, and implicit ownership.

### Phase 2: Audio Runtime and Playable Instrument Loop
**Rationale:** The product is judged first as an instrument, so reliable audio and recovery must arrive before deep editing or rich agent features.
**Delivers:** SuperCollider sidecar/process supervision, OSC adapter, deterministic group/bus allocation, transport, panic/stop-all, macros, controller mapping, runtime health, and load/play/mutate/recover loop.
**Uses:** Tauri sidecars, Rust/Tokio orchestration, SuperCollider, `rosc`, SQLite snapshots.
**Implements:** Audio adapter, scheduler/coordinator, runtime health surfaces.

### Phase 3: UI Projection, Scenes, and Recall
**Rationale:** Once the command/event/audio loop is stable, the UI can sit cleanly over canonical state instead of inventing parallel truth.
**Delivers:** Graph read-first viewport with bounded editing, inspector, transport/performance surface, save/load flows, scene recall, variation basics, and runtime status/diff visibility.
**Addresses:** Table-stakes visibility, session recall, scenes, performance control, inspectable runtime feedback.
**Avoids:** Building the patch editor before the playable loop and under-specifying scene transitions.

### Phase 4: Session-Anchored Agent Collaboration
**Rationale:** Agent features are only safe once commands, ownership, history, and diffs are stable.
**Delivers:** Conversational agent over safe primitives, proposal/review/approve flow, explainable mutations, provenance, risk-based approvals, and human reclaim behavior.
**Addresses:** Core AI promise without sacrificing performance trust.
**Avoids:** Chat-as-primary-authoring, unbounded patch programming, and opaque undo/history.

### Phase 5: Visual Runtime and Cross-Modal Coordination
**Rationale:** Visuals should be architecturally first-class but sequenced after audio/control trust is proven.
**Delivers:** Separate visual adapter, shared transport/events, cross-modal macros/scenes, acknowledgements, degraded-mode behavior, and bounded AV sync.
**Addresses:** Basic visual runtime sync and differentiated audiovisual instrument behavior.
**Avoids:** Over-coupling audio and visuals and pretending to guarantee exact engine-frame atomicity.

### Phase 6: Performance Hardening and Advanced Coordination
**Rationale:** Transition polish, resource budgets, and deeper coordination only pay off after the foundation is reliable.
**Delivers:** Compiled scene transitions, richer macro/binding systems, resource budgets, degraded modes, subsystem restart paths, and performance profiling under load.
**Addresses:** Live reliability, larger sessions, and more expressive transitions.
**Avoids:** Collapse under scale, coarse recovery, and fragile load behavior.

### Phase Ordering Rationale

- Freeze semantics before surfaces: canonical graph, ownership, and mutation grammar come before advanced UI or agent work.
- Make it playable before making it broad: dependable audio/control loop is more important than freeform patch editing or visual depth.
- Add AI after auditability exists: commands, diffs, undo/recovery, and ownership gates are prerequisites for trustworthy co-creation.
- Keep visuals parallel in architecture but later in execution: shared abstractions should exist early, while runtime-specific depth waits until the audio path is proven.
- Reserve advanced transitions and performance polish for later phases because they depend on correct diffing, coordination, and runtime health signals.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 3:** Scene transition semantics, preload/tail/ramp policy, and variation behavior need phase-specific design research.
- **Phase 4:** Mutation grammar, approval thresholds, and undo/event-log retention should be validated once the primitive set stabilizes.
- **Phase 5:** Visual runtime contract details and engine-specific capabilities remain the least certain area.
- **Phase 6:** Resource budgets, degraded-mode policies, and cross-platform performance tuning need real-machine validation.

Phases with standard patterns (skip research-phase):
- **Phase 1:** Rust/Tauri canonical core, typed commands/events, persistence, and ownership scaffolding are well supported by the research.
- **Phase 2:** SuperCollider adapter foundation, sidecar supervision, OSC control, and runtime health patterns are well documented.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Tauri, Rust, SuperCollider, SQLite, and graph tooling recommendations are anchored in current official docs and stable ecosystem patterns. |
| Features | MEDIUM | Table stakes are well supported by adjacent tool categories; AI-specific expectations are still partly inferred from the product thesis. |
| Architecture | HIGH | The command/event/reducer/projection model is cohesive, strongly justified, and well matched to local-first runtime orchestration. |
| Pitfalls | HIGH | Risks are concrete, domain-specific, and backed by known Tauri and SuperCollider constraints. |

**Overall confidence:** MEDIUM

### Gaps to Address

- `Visual runtime contract`: keep Bevy as the current direction, but validate capability and protocol details during phase planning before locking deep visual scope.
- `Scene transition policy`: define what crossfades, quantizes, preloads, persists, and rolls back before promising sophisticated scene/variation behavior.
- `Undo/event-log retention`: decide how much event sourcing is durable versus session-local once the mutation grammar is proven.
- `Cross-platform audio device behavior`: validate bootstrap, diagnostics, and low-latency expectations on real macOS, Windows, and Linux targets early.
- `Primitive library boundaries`: keep the initial mutation vocabulary small and explicit so both UX and agent behavior stay legible.

## Sources

### Primary (HIGH confidence)
- Tauri v2 docs — app IPC, state management, sidecars, shell/capabilities, and desktop architecture constraints
- SuperCollider official docs — server architecture, order of execution, node messaging, sync/async behavior, and audio device selection
- React Flow official docs — maintained node-editor patterns for graph-native UI
- Official product docs for Ableton Live, Max, Resolume, VCV Rack, and SuperCollider — category expectations for live graph/performance tools

### Secondary (MEDIUM confidence)
- Bevy docs — current greenfield visual-runtime option with good long-term potential, but less certain than audio/runtime foundations
- TouchDesigner docs — useful for interaction-model framing, though not as fresh or directly prescriptive as the primary sources
- Package registry/crates.io version checks — current version validation for recommended dependencies

### Tertiary (LOW confidence)
- Rehearsal-memory branch workflow as a differentiator — promising, but weakly validated compared with the core live-instrument patterns
- Fine-grained shared ownership UX beyond basic human override — strategically valuable, but still needs product-specific validation

---
*Research completed: 2026-04-11*
*Ready for roadmap: yes*
