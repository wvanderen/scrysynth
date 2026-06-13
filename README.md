# Scrysynth

Scrysynth is a graph-native desktop audiovisual instrument for live co-creation between a human performer and AI agents. It is a Tauri app with a Rust-owned canonical session graph, a React workspace, and adapter boundaries for audio, visuals, hardware input, and agent actions.

Current stage: foundation prototype. The session model, workspace surfaces, command handlers, tests, and adapter seams are in place. The next work is runtime hardening: making SuperCollider, the visual sidecar, hardware listeners, and agent orchestration behave as real production paths rather than scaffolds.

## What Runs Today

- Tauri 2 desktop shell with a Rust backend and React/Vite frontend.
- Canonical session graph for nodes, routes, buses, macros, scenes, variations, ownership, runtime status, pending actions, action history, and hardware bindings.
- JSON session save/open flow.
- Graph, conversation, and performance workspace views.
- Bounded graph edits, scene recall, variation save/restore, macro CRUD, and ownership controls.
- Runtime manager shells for audio and visuals, including health state and panic/stop controls.
- MIDI/OSC learn and routing model.

Known limitation: the architecture is ahead of the actual runtime execution. `scsynth` can be launched if installed, but topology loading does not yet send real synth definitions or OSC node/bus commands. The Bevy visual sidecar is expected as `scrysynth-visual`, but no sidecar binary is included yet and scene/parameter updates are stubs.

## Local Requirements

Install these before running the app locally:

- Node.js 20.19.0 or newer in the 20.x line, or Node.js 22.12.0 or newer, plus npm. This matches the Vite 7 engine range in `package-lock.json`.
- Rust stable with `cargo` on `PATH`.
- Tauri system prerequisites for your OS. On macOS this usually means Xcode Command Line Tools.
- SuperCollider with `scsynth` available on `PATH`, or set `SCRYSYNTH_SCSYNTH_PATH` to the full `scsynth` executable path.

Optional for the runtime paths:

- A future visual runtime executable named `scrysynth-visual` on `PATH`, or set `SCRYSYNTH_BEVY_PATH`.
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

## Project Status

The planned foundation phases are complete in the planning history:

1. Session Core & Recall
2. Playable Audio Graph foundation
3. Performance Workspace
4. Agent Collaboration scaffold
5. Visual Sync & Cross-Modal Control scaffold

The project is not release-ready yet. The next milestone is v1 runtime hardening:

- Implement real SuperCollider resource application through OSC.
- Build or wire the visual sidecar and protocol.
- Connect MIDI/OSC listeners into the app runtime, not only the testable router.
- Replace deterministic agent parsing with a real session-aware agent orchestration layer.
- Clean up packaging, docs, release checks, and manual verification.

See `.planning/ROADMAP.md` for the current roadmap and `.planning/STATE.md` for the consolidated project state.
