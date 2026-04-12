# Phase 2 Research: Playable Audio Graph

**Phase:** 2 - Playable Audio Graph
**Researched:** 2026-04-11
**Status:** Ready for planning

## Research Answer

Phase 2 should turn the canonical session graph into a bounded live audio instrument without making SuperCollider the source of truth. The safest path is to expand the performer-facing audio primitive model in Rust first, compile that canonical graph into a deterministic SuperCollider topology through a supervised adapter, then support incremental live updates and a panic-safe recovery path.

## Prescriptive Decisions

1. Expand the canonical session model with a small, explicit v1 audio primitive set and bounded graph-edit commands before integrating deeper runtime behavior.
2. Keep SuperCollider concepts behind the adapter boundary: canonical session data should describe performer-facing nodes, routes, buses, parameters, and runtime intent, not raw SC internals.
3. Make runtime lifecycle and health explicit in app-owned state so the UI can show booting, ready, degraded, and panic-recovered conditions.
4. Compile the canonical graph into deterministic group and bus ordering so playback and recovery stay predictable across reloads and live edits.
5. Treat Tauri IPC as command/control only; do not depend on it for realtime audio-rate messaging.

## Recommended Phase Shape

### Canonical Audio Model

Extend the session contract with supported v1 primitives and mutation affordances:

- source nodes with bounded synth options and typed parameters
- gain or mixer stages
- effect or processing nodes with explicit enable or bypass state
- buses and grouped routing targets
- runtime-facing transport and health state

The model should support `SESS-04` by letting users create, remove, enable, disable, and reroute supported graph primitives without editing runtime-only details.

### SuperCollider Adapter Boundary

Build a Rust-owned adapter that:

- starts and supervises the SuperCollider process
- allocates groups and buses deterministically
- compiles the canonical graph into runtime instructions
- tracks adapter lifecycle and error state
- exposes panic-safe stop-all recovery

This should satisfy the first playable path plus `AUD-01` and `AUD-04` without leaking SC node IDs into persisted session state.

### Live Mutation Strategy

Support incremental graph changes rather than full session rebuilds wherever safe:

- parameter updates during playback
- route or bus changes validated before apply
- node enable or bypass toggles
- incremental runtime diff application with fallback to safe rebuild when necessary

This is the core of `AUD-02` and `AUD-03`: users hear changes live while the system maintains a known-good recovery path.

## Proposed Plan Breakdown

### Plan 1: Canonical Audio Graph Expansion

- add supported v1 primitives and richer runtime status to the canonical model
- define bounded graph-edit commands and validation rules
- prevent illegal routes and unsupported cycles

### Plan 2: SuperCollider Runtime Foundation

- add process supervision and adapter lifecycle management
- compile canonical graphs into deterministic SC topology
- implement transport controls and panic-safe stop-all

### Plan 3: Live Updates And Minimal Playback UI

- apply safe parameter and routing diffs during playback
- expose minimal controls to edit and hear the graph live
- prove recovery behavior when runtime state becomes invalid or disconnected

## Decisions To Lock Before Planning

1. Keep the v1 primitive set intentionally small: a limited set of sources, processors, buses, and master output.
2. Keep the Phase 2 edit surface structured rather than fully freeform; the goal is reliable sound, not unconstrained graph design.
3. Decide whether SuperCollider is local-install-first or bundled-sidecar-first for the first working path.
4. Keep transport controls focused on play, stop, and panic; defer broader controller or macro gesture scope unless needed to satisfy the phase goal.

## Pitfalls To Avoid In This Phase

1. Do not let raw SuperCollider topology become the canonical session model.
2. Do not allow arbitrary graph edits that the runtime cannot validate or recover from safely.
3. Do not treat panic as a UI-only concern; it must drive backend runtime teardown or mute behavior that returns the app to a known safe state.
4. Do not rebuild the entire runtime for every small parameter change if an incremental safe update path exists.
5. Do not expand the primitive set faster than the adapter and validation layer can guarantee.

## Verification Focus

- Rust tests prove canonical graph validation and graph-to-runtime compilation stability.
- Adapter tests prove lifecycle transitions, deterministic allocation, and panic-safe stop-all behavior.
- Playback smoke tests prove users can hear supported source-to-output paths.
- UI verification proves users can edit supported parameters and routes during playback and see runtime health changes.

## Planning Implications

- Plan 1 should stay mostly domain and validation focused.
- Plan 2 should stay mostly backend runtime orchestration focused.
- Plan 3 should add the minimum UI and incremental update path needed to hear and control the live graph.
