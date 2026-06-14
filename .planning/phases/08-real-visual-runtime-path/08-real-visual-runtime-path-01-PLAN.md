---
phase: 08-real-visual-runtime-path
plan: 01
type: hardening
status: planned
created: 2026-06-14
depends_on:
  - 07-real-supercollider-execution
td_epic: td-717878
requirements:
  - VISUAL-RUNTIME-01
  - VISUAL-RUNTIME-02
  - VISUAL-RUNTIME-03
  - VISUAL-RUNTIME-04
---

# Phase 8: Real Visual Runtime Path

## Goal

Visuals run through a separate process that consumes canonical scene, control, macro, scene, and runtime projections from the app instead of succeeding through lifecycle stubs.

This phase should preserve the product boundary: `SessionDocument` remains canonical, the visual process is an adapter target, and visual runtime IDs or transport details stay ephemeral.

## Current Baseline

- `VisualRuntimeManager` can start, stop, panic, and update visual health state.
- `BevySidecarAdapter` resolves `scrysynth-visual` or `SCRYSYNTH_BEVY_PATH`, launches the process, and terminates it.
- `load_scene` and `update_parameters` are stubs that return success without sending anything.
- `compile_session_to_visual_scene` derives a simple scene projection from enabled graph nodes.
- The README already documents that no visual sidecar binary is included and scene/parameter delivery is stubbed.

## Success Criteria

1. Provide or document the `scrysynth-visual` sidecar executable path and launch contract.
2. Replace visual scene-load and parameter-update stubs with a typed local protocol.
3. Drive visual scene state from canonical session nodes, scenes, macros, and runtime events.
4. Surface visual runtime errors, disconnects, reconnects, and restart behavior in the Runtime Health panel.
5. Record UAT evidence covering missing sidecar diagnostics, successful sidecar handshake, scene load, live parameter updates, stop, panic, and restart.

## Task Breakdown

### 8.1 Define the visual sidecar protocol and runtime contract

Task: `td-2ca78f`

Create a protocol document and Rust message types for app-to-sidecar and sidecar-to-app communication. The contract should cover handshake, scene load, parameter updates, runtime events, errors, ping/pong, and shutdown. The first implementation should prefer JSON lines over stdio or loopback IPC unless implementation research identifies a concrete reason to choose otherwise.

Acceptance:

- Protocol doc exists in this phase folder.
- Rust protocol types are serializable/deserializable and tested.
- The launch contract documents `scrysynth-visual`, `SCRYSYNTH_BEVY_PATH`, message framing, readiness timeout, and shutdown behavior.
- Runtime-owned fields are not added to `SessionDocument`.

### 8.2 Implement a minimal `scrysynth-visual` sidecar executable

Task: `td-14b0a8`

Add or wire a small sidecar binary that speaks the protocol and renders a simple real scene from `CompiledVisualScene`. It does not need advanced authoring, but it must be a real process that receives scene state and parameter changes.

Acceptance:

- The repository either contains the sidecar crate/binary or documents an external build path clearly enough to run locally.
- The sidecar replies to handshake and scene load messages.
- The sidecar applies parameter updates without requiring a full restart.
- Missing executable diagnostics remain actionable.
- Automated tests cover protocol handling where feasible without requiring GPU availability.

### 8.3 Replace adapter stubs with transport, handshake, and ack handling

Task: `td-1ea016`

Update `BevySidecarAdapter` so `start`, `load_scene`, `update_parameters`, `stop`, and `panic` communicate with the sidecar through the typed protocol. Startup must not mark the runtime healthy until the process has acknowledged readiness and scene load.

Acceptance:

- Adapter sends handshake and waits for bounded ready acknowledgement.
- Scene load sends the compiled scene and waits for bounded acknowledgement.
- Parameter updates send typed update batches and fail clearly when no patch is active.
- Stop and panic terminate/notify the sidecar predictably.
- Runtime failure keeps `VisualRuntimeLifecycle::Failed`, `VisualRuntimeHealth::Degraded`, and `RuntimeConnectionState::Error` aligned.

### 8.4 Drive canonical visual projection from scenes, macros, and runtime events

Task: `td-c845bc`

Improve `compile_session_to_visual_scene` and update paths so the visual sidecar receives meaningful canonical projections, not just enabled node placeholders. Scene recall, macro values, and relevant runtime events should produce visual updates.

Acceptance:

- Active scene selection influences the compiled visual scene.
- Macro values that target visual parameters produce `VisualParameterUpdate` messages.
- Graph parameter edits and scene recall reload/update the sidecar without stale success states.
- Tests cover scene compilation, macro-to-visual update mapping, and failure behavior.

### 8.5 Harden health panel behavior and document Phase 8 UAT

Task: `td-11fe55`

Make visual runtime state legible in the UI and capture manual verification evidence. The user should understand whether the sidecar is missing, booting, ready, degraded, disconnected, or recoverable.

Acceptance:

- Runtime Health panel shows actionable visual errors and active scene/renderer state.
- Restart after visual panic is verified.
- UAT doc records exact local setup, commands, expected behavior, and observed behavior.
- README and roadmap wording are updated only after real behavior is verified.

## Suggested Dependency Order

1. 8.1 protocol and launch contract.
2. 8.2 minimal sidecar.
3. 8.3 real adapter transport and acknowledgements.
4. 8.4 canonical projection and live updates.
5. 8.5 health UI, docs, and UAT evidence.

## Non-Goals

- No in-webview Three.js or React visual runtime as the canonical v1 path.
- No advanced visual graph editor.
- No projection mapping, media server, timeline, or clip launcher scope.
- No persistence of sidecar process IDs, renderer entity IDs, transport sequence IDs, or GPU resource IDs in `SessionDocument`.
- No hardware input runtime work; Phase 9 owns live MIDI/OSC listener wiring.
- No session-aware agent orchestration work; Phase 10 owns agent planning.
