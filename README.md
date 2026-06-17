# Scrysynth

Scrysynth is a graph-native desktop audiovisual instrument for live co-creation between a human performer and AI agents. It is a Tauri app with a Rust-owned canonical session graph, a React workspace, and adapter boundaries for audio, visuals, hardware input, and agent actions.

Current stage: foundation prototype. The session model, workspace surfaces, command handlers, tests, and adapter seams are in place. The next work is runtime hardening: making SuperCollider, the visual sidecar, hardware listeners, and agent orchestration behave as real production paths rather than scaffolds.

## What Runs Today

- Tauri 2 desktop shell with a Rust backend and React/Vite frontend.
- Canonical session graph for nodes, routes, buses, macros, scenes, variations, ownership, runtime status, pending actions, action history, and hardware bindings.
- JSON session save/open flow.
- Graph, conversation, and performance workspace views.
- Bounded graph edits, scene recall, variation save/restore, macro CRUD, and ownership controls.
- Runtime managers for audio and visuals, including health state, actionable setup/runtime diagnostics, active visual scene/renderer readouts, and panic/stop/restart controls.
- MIDI/OSC learn and routing model.

Known limitation: the architecture is still ahead of a complete audiovisual instrument. Phase 7 real SuperCollider execution has been verified against a local `scsynth` install for the default source-to-output graph, including audible playback, live parameter change, stop, panic, and restart. Phase 8 now has a verified minimal visual sidecar path: the app launches `scrysynth-visual`, handshakes over JSON lines, loads a compiled scene, applies live parameter updates, stops, panics, and restarts after panic. The renderer is intentionally minimal and GPU-free for now; richer Bevy visuals and packaged sidecar wiring remain future hardening work.

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
- Connect MIDI/OSC listeners into the app runtime, not only the testable router.
- Replace deterministic agent parsing with a real session-aware agent orchestration layer.
- Clean up packaging, docs, release checks, and manual verification.

See `.planning/ROADMAP.md` for the current roadmap and `.planning/STATE.md` for the consolidated project state.
