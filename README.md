# Scrysynth

Scrysynth is a graph-native desktop audiovisual instrument for live co-creation between a human performer and AI agents. It is a Tauri app with a Rust-owned canonical session graph, a React workspace, and adapter boundaries for audio, visuals, hardware input, and agent actions.

Current stage: foundation prototype in v1 runtime hardening. The session model, workspace surfaces, command handlers, tests, and adapter seams are in place. SuperCollider execution, the minimal visual sidecar path, the app-owned MIDI/OSC hardware runtime path, and deterministic/mock session-aware agent orchestration have been verified locally; the next major runtime gap is live provider-backed agent orchestration plus packaging/release hardening.

## What Runs Today

- Tauri 2 desktop shell with a Rust backend and React/Vite frontend.
- Canonical session graph for nodes, routes, buses, macros, scenes, variations, ownership, runtime status, pending actions, action history, and hardware bindings.
- JSON session save/open flow.
- Graph, conversation, and performance workspace views.
- Bounded graph edits, scene recall, variation save/restore, macro CRUD, and ownership controls.
- Runtime managers for audio and visuals, including health state, actionable setup/runtime diagnostics, active visual scene/renderer readouts, and panic/stop/restart controls.
- MIDI/OSC listener settings, learn, and routing through the app-owned hardware runtime path.

Known limitation: the architecture is still ahead of a complete audiovisual instrument. Phase 7 real SuperCollider execution has been verified against a local `scsynth` install for the default source-to-output graph, including audible playback, live parameter change, stop, panic, and restart. Phase 8 has a verified minimal visual sidecar path: the app launches `scrysynth-visual`, handshakes over JSON lines, loads a compiled scene, applies live parameter updates, stops, panics, and restarts after panic. Phase 9 has verified virtual MIDI and local OSC learn/routing through the app runtime for macros, scene recall, transport play/stop, and panic. Phase 10 has verified deterministic/mock session-aware planner orchestration through bounded context packets, typed proposal normalization, approval/rejection, freeze/reclaim, and diagnostics. The renderer remains intentionally minimal; richer Bevy visuals, packaged sidecar wiring, GUI click-through hardware UAT, live provider-backed agent orchestration, and release packaging remain future hardening work.

## Local Requirements

Install these before running the app locally:

- Node.js 20.19.0 or newer in the 20.x line, or Node.js 22.12.0 or newer, plus npm. This matches the Vite 7 engine range in `package-lock.json`.
- Rust stable with `cargo` on `PATH`.
- Tauri system prerequisites for your OS. On macOS this usually means Xcode Command Line Tools.
- SuperCollider with `scsynth` available on `PATH`, or set `SCRYSYNTH_SCSYNTH_PATH` to the full `scsynth` executable path.

Optional for the runtime paths:

- The in-repo visual runtime executable named `scrysynth-visual` on `PATH`, or set `SCRYSYNTH_BEVY_PATH` to its full path.
- MIDI hardware or a virtual MIDI source for hardware learn testing.
- An OSC sender if testing OSC learn/routing.

This repository currently does not include `node_modules`. If `npm test` or `npm run build` reports missing `vitest`, `tsc`, or `vite`, install frontend dependencies first.

## Setup

```sh
npm install
```

Confirm Rust is available:

```sh
cargo --version
```

Confirm SuperCollider is available, or configure an explicit path:

```sh
which scsynth
export SCRYSYNTH_SCSYNTH_PATH="/path/to/scsynth"
```

If audio startup fails, the Runtime Health panel reports the specific setup or server stage. Missing `scsynth` messages include `SCRYSYNTH_SCSYNTH_PATH`; on macOS the app also checks `/Applications/SuperCollider.app/Contents/Resources/scsynth` before reporting that the executable is missing. OSC `/sync`, SynthDef load, topology apply, and panic recovery failures are shown as audio runtime details so setup issues can be fixed without reading backend logs first.

If visual startup fails, the Runtime Health panel reports the visual lifecycle, connection state, active scene, renderer, telemetry label, and actionable sidecar errors. Missing sidecar messages include `SCRYSYNTH_BEVY_PATH`; panic leaves the visual runtime in a restartable `panicked` state so Start can relaunch the sidecar and reload the active scene.

## Development

Run the frontend only:

```sh
npm run dev
```

Run the Tauri desktop app:

```sh
npm run tauri dev
```

Build the frontend:

```sh
npm run build
```

Run frontend tests:

```sh
npm test
```

Run Rust tests:

```sh
cargo test --manifest-path src-tauri/Cargo.toml
```

Build the minimal visual sidecar:

```sh
cargo build --manifest-path src-tauri/Cargo.toml --bin scrysynth-visual
export SCRYSYNTH_BEVY_PATH="$PWD/src-tauri/target/debug/scrysynth-visual"
```

The sidecar reads Phase 8 JSON-lines messages on stdin and writes JSON-lines replies on stdout. It is intentionally minimal and GPU-free for now: handshake returns renderer readiness, scene load stores a `CompiledVisualScene` snapshot, parameter batches update that live scene state without restart, and graceful or panic shutdown requests return shutdown acknowledgements.

Phase 8 visual UAT evidence is tracked in `.planning/phases/08-real-visual-runtime-path/08-real-visual-runtime-path-05-UAT.md`. The verified path covers missing sidecar diagnostics, real sidecar handshake, scene load, live parameter update, stop, panic, and restart after panic.

## Manual Hardware UAT

Phase 9 hardware input UAT evidence is tracked in `.planning/phases/09-hardware-input-runtime-wiring/09-hardware-input-runtime-wiring-06-UAT.md`.

The verified pass used a CoreMIDI virtual source named `Scrysynth Phase 9 UAT Virtual MIDI`, a local OSC sender on `127.0.0.1`, and `/Applications/SuperCollider.app/Contents/Resources/scsynth`. It proved live learn and post-learn routing for MIDI macro and transport stop, OSC macro, OSC scene recall, OSC transport play, and OSC panic. Panic was verified while the SuperCollider runtime had an active patch and returned audio to `Idle/PanicRecovered`.

The pass exercised the app runtime command path behind the Tauri workspace, not a separate GUI click-through automation pass.

## Manual Agent Orchestration UAT

Phase 10 agent orchestration UAT evidence is tracked in `.planning/phases/10-session-aware-agent-orchestration/10-session-aware-agent-orchestration-06-UAT.md`.

The verified pass used deterministic/mock planner fixtures and the local parser provider, not a live remote LLM provider. It proved bounded session context packets, realistic planner-shaped proposals, typed command normalization, parameter validation, scene recall, high-risk approval/rejection, freeze behavior, reclaim behavior, provider-unavailable diagnostics, invalid-response diagnostics, and frontend rendering of proposal review and runtime diagnostic states.

## Manual Audio UAT

Phase 7 completion requires a real local SuperCollider check, not just automated tests:

```sh
export SCRYSYNTH_SCSYNTH_PATH="/Applications/SuperCollider.app/Contents/Resources/scsynth"
npm run tauri dev
```

In the desktop app, start the audio runtime from the transport or Runtime Health panel, verify audible output from the default graph or a minimal source-to-output graph, adjust a live parameter from the inspector or macro path, stop audio, trigger panic, then confirm the runtime can start again after panic.

Current evidence is tracked in `.planning/phases/07-real-supercollider-execution/07-real-supercollider-execution-07-UAT.md`. The resumed real-`scsynth` pass verified audible output, live parameter control, stop, panic, and restart after panic. A Stop-button projection defect found during UAT was fixed and retested successfully.

## Project Status

The planned foundation phases are complete in the planning history:

1. Session Core & Recall
2. Playable Audio Graph foundation
3. Performance Workspace
4. Agent Collaboration scaffold
5. Visual Sync & Cross-Modal Control scaffold

The project is not release-ready yet. The next milestone is v1 runtime hardening:

- Extend the verified SuperCollider path beyond the default graph as new primitives and routing workflows are hardened.
- Extend the verified minimal visual sidecar into a richer Bevy-rendered runtime and package it as a Tauri sidecar.
- Connect the verified session-aware agent orchestration boundary to a live provider-backed planner without weakening typed command, approval, freeze, and reclaim gates.
- Clean up packaging, docs, release checks, and manual verification.

See `.planning/ROADMAP.md` for the current roadmap and `.planning/STATE.md` for the consolidated project state.
