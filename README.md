# Scrysynth

Scrysynth is a graph-native desktop audiovisual instrument for live co-creation between a human performer and AI agents. It is a Tauri app with a Rust-owned canonical session graph, a React workspace, and adapter boundaries for audio, visuals, hardware input, and agent actions.

**Current stage: v1 (1.0.0)** — packaged and evaluated as a desktop instrument on Apple Silicon macOS. The supported behaviors below were each verified against the packaged app; see `RELEASE_NOTES.md` for the release and `.planning/phases/11-release-readiness/11-release-readiness-03-UAT.md` for the consolidated verification evidence.

## Supported in v1

Each row was re-verified against the packaged `scrysynth.app` in the Phase 11 consolidated UAT (scenario numbers refer to that doc).

| Behavior | Notes | Verified in |
|----------|-------|-------------|
| Session save/open | JSON save/open restores graph, macros, scenes | UAT scenario 1; Phase 1 |
| Graph editing | Bounded add/remove/re-route of v1 primitives, live inspector + graph sync | UAT scenario 2; Phase 2 |
| Audio playback via SuperCollider | Default source-to-output graph, audible output, live parameter changes heard | UAT scenario 3; Phase 7 (`07-...-07-UAT.md`) |
| Scene / variation recall | Scene recall + variation save/restore within a session | UAT scenario 4; Phase 3 |
| Cross-domain macros | One macro drives audio + visual targets together | UAT scenario 5; Phase 5 |
| MIDI/OSC learn + routing (app runtime path) | Learn and post-learn routing for macros, scene recall, transport, panic — via CoreMIDI virtual source + local OSC sender | UAT scenario 6; Phase 9 (`09-...-06-UAT.md`) |
| Agent approval / rejection | High-risk pending actions from the in-app planner (local parser provider through the planner gates), approve/reject + freeze/reclaim | UAT scenario 7; Phase 10 (`10-...-06-UAT.md`) |
| Minimal visual sidecar | Bundled `scrysynth-visual` reaches Ready, loads compiled scenes, applies live parameter batches, Stop/Panic/Start cycle | UAT scenario 8; Phase 8 (`08-...-05-UAT.md`) |
| Panic recovery (audio + visual) | Panic stops sound immediately and leaves a restartable state; restart recovers — for both audio and the visual sidecar | UAT scenario 9; Phases 7 + 8 |

## Not supported in v1 (deferred)

These are intentionally out of scope for v1 and are tracked as follow-on work, not as shipped behavior:

- Richer Bevy-rendered visuals / a visible render window — v1 ships the minimal GPU-free sidecar only.
- Live provider-backed agent orchestration (remote LLM) — only the deterministic/local-parser planner path is wired; AGNT-01R remains open.
- Physical-controller GUI click-through UAT — hardware learn is verified via virtual MIDI + local OSC, not physical gear.
- Windows, Linux, and Intel/universal macOS builds — v1 is Apple Silicon (`aarch64-apple-darwin`) only.
- Full Developer ID signing + notarization + auto-update — v1 is ad-hoc signed; first launch uses the Gatekeeper right-click → Open workflow.
- Multiplayer / multi-user sessions — v1 is local and single-user.

See `.planning/ROADMAP.md` ("Deferred Beyond v1") and `RELEASE_NOTES.md` for the full deferred list.

## Install

### Packaged app (end users)

1. Download the `.dmg` for Apple Silicon (`scrysynth_1.0.0_aarch64.dmg`).
2. Open the `.dmg` and drag `scrysynth.app` to **Applications**.
3. **First launch (ad-hoc signing):** because v1 is ad-hoc signed (not Developer ID / notarized), double-clicking may be blocked by Gatekeeper. Instead **right-click** `scrysynth.app` → **Open** → confirm the **"Open Anyway"** prompt. This is only required once; macOS remembers the clearance afterward.
4. **Audio requires SuperCollider.** Install SuperCollider 3.14.x separately from https://supercollider.github.io/downloads. Scrysynth does not bundle `scsynth`. On macOS the app auto-detects `/Applications/SuperCollider.app/Contents/Resources/scsynth`; if your install is elsewhere, set `SCRYSYNTH_SCSYNTH_PATH` to the full `scsynth` path.
5. Launch `scrysynth`. The Runtime Health panel surfaces setup errors (missing `scsynth`, missing sidecar, OSC bind failures, panic states) with actionable messages.

### Developer install

```sh
npm install
cargo --version                  # Rust stable required
which scsynth || export SCRYSYNTH_SCSYNTH_PATH="/path/to/scsynth"
```

Local requirements:

- Node.js 20.19.0+ (20.x line) or 22.12.0+, plus npm (matches the Vite engine range).
- Rust stable with `cargo` on `PATH`.
- Tauri macOS prerequisites (Xcode Command Line Tools).
- SuperCollider with `scsynth` on `PATH`, or `SCRYSYNTH_SCSYNTH_PATH` set.

Optional runtime extras:

- The visual sidecar executable `scrysynth-visual` on `PATH`, or `SCRYSYNTH_BEVY_PATH` set (only needed for dev mode; the packaged app bundles it).
- MIDI hardware or a virtual MIDI source, and an OSC sender, for hardware-learn testing.

This repository does not ship `node_modules` — run `npm install` before `npm test` or `npm run build`.

## Development

```sh
npm run dev                # frontend only
npm run tauri dev          # full Tauri desktop app (dev mode)
npm run build              # frontend production build
npm test                   # frontend tests (Vitest)
cargo test --manifest-path src-tauri/Cargo.toml   # Rust tests
```

Build the minimal visual sidecar for dev mode:

```sh
cargo build --manifest-path src-tauri/Cargo.toml --bin scrysynth-visual
export SCRYSYNTH_BEVY_PATH="$PWD/src-tauri/target/debug/scrysynth-visual"
```

The sidecar speaks JSON-lines over stdio: handshake reports renderer readiness, scene load stores a `CompiledVisualScene` snapshot, parameter batches update live scene state without restart, and graceful/panic shutdowns return acknowledgements. It is intentionally minimal and GPU-free in v1.

## Build from source (packaged release)

```sh
npm install
npm run tauri build
```

`beforeBuildCommand` runs `npm run build && ./scripts/prepare-sidecar.sh`, which builds the release sidecar (`cargo build --release --bin scrysynth-visual`) and copies it to the target-triple-suffixed path the Tauri bundler expects. The bundler then compiles the main binary, assembles the `.app`, ad-hoc signs it, and produces the `.dmg`. Artifacts land under `src-tauri/target/release/bundle/`.

## Manual UAT evidence

The packaged-app verification for v1 is consolidated in `.planning/phases/11-release-readiness/11-release-readiness-03-UAT.md` (nine scenario areas), with the build + ad-hoc-sign + first-run-smoke record in `.planning/phases/11-release-readiness/11-release-readiness-02-BUILD-EVIDENCE.md`. The Phase 7–10 developer-mode UAT docs (referenced above) contain the deeper per-runtime evidence.

## Project status

See `.planning/ROADMAP.md` for the roadmap and `.planning/STATE.md` for consolidated state. The v1 runtime-hardening milestone (Phases 7–11) is complete; follow-on work (live provider-backed agent, richer Bevy visuals, cross-platform, full notarization) is tracked as deferred beyond v1.
