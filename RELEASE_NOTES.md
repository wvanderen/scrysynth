# Release Notes — Scrysynth v1.0.0

**Released:** 2026-06-26
**Target:** macOS on Apple Silicon (`aarch64-apple-darwin`)
**Version:** 1.0.0
**Signing:** ad-hoc (not Developer ID / not notarized)

Scrysynth is a graph-native desktop audiovisual instrument for live co-creation between a human performer and AI agents. This is the first packaged release: a Tauri app with a Rust-owned canonical session graph, a React workspace, SuperCollider audio execution, a minimal visual sidecar, app-owned MIDI/OSC hardware binding, and session-aware agent collaboration with human override.

## What's in this release

The behaviors below were each verified against the packaged `scrysynth.app` in the Phase 11 consolidated UAT (`.planning/phases/11-release-readiness/11-release-readiness-03-UAT.md`):

- **Session save/open** — JSON save/open restores graph, macros, and scenes.
- **Graph editing** — bounded add/remove/re-route of v1 primitives with live inspector + graph sync.
- **Audio playback via SuperCollider** — default source-to-output graph produces audible output; live parameter changes are heard during playback.
- **Scene / variation recall** — scene recall and variation save/restore within a session.
- **Cross-domain macros** — one macro drives audio and visual targets together.
- **MIDI/OSC learn + routing (app runtime path)** — learn and post-learn routing for macros, scene recall, transport play/stop, and panic, via a CoreMIDI virtual source and a local OSC sender.
- **Agent approval / rejection** — high-risk pending actions produced by the in-app planner (local parser provider routed through the planner gates), with approve/reject and freeze/reclaim.
- **Minimal visual sidecar** — bundled `scrysynth-visual` reaches Ready, loads compiled scenes, applies live parameter batches, and cycles Stop/Panic/Start.
- **Panic recovery (audio + visual)** — panic stops sound immediately and leaves a restartable state; restart recovers both the audio runtime and the visual sidecar.

## System requirements

- **macOS on Apple Silicon** (`aarch64-apple-darwin`). v1 does not ship Intel, universal, Windows, or Linux builds.
- **SuperCollider 3.14.x** installed separately — Scrysynth does not bundle `scsynth`. Install from https://supercollider.github.io/downloads. On macOS the app auto-detects `/Applications/SuperCollider.app/Contents/Resources/scsynth`; otherwise set `SCRYSYNTH_SCSYNTH_PATH` to your `scsynth` path. Audio playback and panic-recovery require this.
- **Ad-hoc signing** — v1 is ad-hoc signed (full Developer ID + notarization is deferred). The first launch requires the Gatekeeper right-click → Open workflow described below.

## Known limitations & deferred work

The following are intentionally out of scope for v1 and are tracked as follow-on work — they are **not** shipped behaviors:

- **Richer Bevy-rendered visuals / a visible render window** — v1 ships the minimal GPU-free sidecar only.
- **Live provider-backed agent orchestration (remote LLM)** — only the deterministic/local-parser planner path is wired. A live provider remains future hardening (AGNT-01R).
- **Physical-controller GUI click-through UAT** — hardware learn is verified via virtual MIDI + local OSC, not physical gear.
- **Windows, Linux, Intel, and universal macOS builds** — v1 is Apple Silicon only.
- **Full Developer ID signing + notarization + auto-update** — v1 is ad-hoc signed.
- **Multiplayer / multi-user sessions** — v1 is local and single-user.
- **Explicit multi-bus / grouped audio routing stress** — the default source-to-output graph (which routes through the master bus) is verified; a dedicated multi-bus routing stress was not a separate release scenario.

## How to install

1. Download `scrysynth_1.0.0_aarch64.dmg` for Apple Silicon.
2. Open the `.dmg` and drag `scrysynth.app` to **Applications**.
3. **First launch (ad-hoc signing):** double-clicking may be blocked by Gatekeeper because v1 is not Developer ID / notarized. Instead **right-click** `scrysynth.app` → **Open** → confirm the **"Open Anyway"** prompt. This is required only once; macOS remembers the clearance.
4. Install SuperCollider 3.14.x separately and (if needed) point `SCRYSYNTH_SCSYNTH_PATH` at your `scsynth`.
5. Launch `scrysynth`. To start visuals, open the Runtime workspace tab and click the Visual Runtime card's **Start** button (the sidecar does not auto-start). The Runtime Health panel surfaces setup errors with actionable messages.

## Verification

- **Consolidated packaged-app UAT (nine scenario areas):** `.planning/phases/11-release-readiness/11-release-readiness-03-UAT.md`
- **Release build + ad-hoc sign + secret-leak check + first-run smoke:** `.planning/phases/11-release-readiness/11-release-readiness-02-BUILD-EVIDENCE.md`
- **Per-runtime developer-mode evidence:** Phases 7 (audio), 8 (visual), 9 (hardware), 10 (agent) under `.planning/phases/`.

## Not included

This release does **not** include: notarization tickets, an auto-updater, Intel/Windows/Linux builds, a richer Bevy render window, or a live remote LLM provider. Claims of support are limited to the behaviors verified in the consolidated UAT above.
