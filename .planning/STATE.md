---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: v1 runtime hardening
status: planning
stopped_at: Phase 8 minimal visual sidecar path verified and documented
last_updated: "2026-06-16T00:00:00-05:00"
last_activity: 2026-06-16
progress:
  foundation_phases_total: 5
  foundation_phases_completed: 5
  hardening_phases_total: 6
  hardening_phases_completed: 2
  current_stage: Phase 8 minimal visual sidecar path complete; hardware runtime wiring next
---

# Project State

## Project Reference

See `.planning/PROJECT.md` and `.planning/ROADMAP.md`.

**Core value:** The instrument must let a human and agent shape a live audiovisual session together through conversation, graph structure, and direct performance control without losing legibility or human override.

**Current focus:** v1 runtime hardening.

## Current Position

Scrysynth is a late foundation prototype. The planned foundation phases have produced a canonical app-owned session model, React workspace, Tauri command layer, persistence, graph editing, scene/variation controls, agent safety scaffolding, visual-runtime scaffolding, macro controls, and MIDI/OSC binding models.

The project is not yet a complete local audiovisual instrument. Runtime execution still needs production UAT and additional hardening:

- Audio can launch `scsynth`, apply v1 SynthDef/topology commands over OSC, produce audible output from the default source-to-output graph, apply live parameter updates, stop, panic, and restart after panic on a real local SuperCollider install.
- Visual runtime management now launches the in-repo minimal `scrysynth-visual` sidecar, handshakes over JSON lines, loads compiled scene snapshots, applies live parameter batches, stops, panics, and restarts after panic. The renderer remains minimal and GPU-free; richer Bevy output and packaged sidecar wiring are still future work.
- MIDI/OSC managers and routing code exist, but app-level listener startup/configuration and receiver wiring need production implementation.
- Agent collaboration is deterministic parser plus safety policy, not a real session-aware LLM/orchestrator.

## Completed Foundation

| Phase | Status | Result |
|-------|--------|--------|
| 1. Session Core & Recall | Complete | Canonical session schema, generated TS contracts, JSON persistence, graph/inspector workspace. |
| 2. Playable Audio Graph Foundation | Complete | Graph edit commands, audio primitive model, topology compiler, runtime lifecycle shell, transport/panic controls. |
| 3. Performance Workspace | Complete | Conversation/graph/performance view switching, scene recall, variation save/restore. |
| 4. Agent Collaboration Scaffold | Complete | Deterministic intent parser, ownership gates, approvals, action history, conversation UI. |
| 5. Visual Sync & Cross-Modal Control Scaffold | Complete | Visual adapter shell, runtime health panel, cross-domain macros, MIDI/OSC binding models. |

## Active Hardening Work

1. Local developer readiness and docs.
2. Hardware listener lifecycle and runtime wiring.
3. Session-aware agent orchestration.
4. Packaging, UAT, and release readiness.

## Completed Hardening Work

| Phase | Status | Result |
|-------|--------|--------|
| 7. Real SuperCollider Execution | Complete | Default source-to-output graph verified against local `scsynth` with audible playback, live parameter update, stop, panic, and restart-after-panic. |
| 8. Real Visual Runtime Path | Complete for minimal sidecar | `scrysynth-visual` starts as a separate process, acknowledges handshake and scene load, applies live parameter batches, stops, panics, and restarts after panic. Runtime Health now surfaces missing sidecar, booting, ready, degraded, disconnected, stopped, panicked, and restartable visual states. |

## Recent Audit Findings

- `README.md` was still the default Tauri template before this consolidation.
- `ROADMAP.md`, `STATE.md`, and `REQUIREMENTS.md` had drifted from one another.
- `ROADMAP.md` showed all five phases complete in the progress table, while some plan checkboxes remained unchecked.
- `STATE.md` still claimed Phase 01/02 execution even though later phase summaries and code exist.
- `REQUIREMENTS.md` marked several Phase 4 agent/interface requirements pending despite implemented Phase 4 scaffold work.
- Local verification in the audit shell was blocked by missing dependencies/tooling: no `node_modules`, no `cargo` on `PATH`.

## Decisions

- Treat Phases 1-5 as **foundation complete**, not as proof of release-ready runtime behavior.
- Track the next milestone as **v1 runtime hardening**.
- Keep the canonical app-owned session model as the source of truth.
- Preserve SuperCollider as the v1 audio execution target, but require real OSC/resource application before claiming playable audio.
- Preserve the separate visual runtime boundary, but require an actual sidecar/protocol before claiming visual execution.
- Keep human override, ownership, freeze/reclaim, and approval gates as non-negotiable product constraints.

## Pending Todos

- Add user-facing runtime setup diagnostics for unavailable hardware input.

## Latest Verification

2026-06-16 task `td-11fe55`:

- `npm test` passed: 3 files, 26 tests.
- `npm run build` passed.
- `cargo build --manifest-path src-tauri/Cargo.toml --bin scrysynth-visual` passed.
- Missing configured sidecar diagnostics were verified with `visual::bevy_sidecar::tests::adapter_reports_missing_configured_sidecar_with_setup_guidance`.
- Real visual sidecar lifecycle was verified with `cargo test --manifest-path src-tauri/Cargo.toml --test visual_sidecar_uat`.
- Direct sidecar protocol UAT observed `ready`, `sceneLoaded`, `parameterBatchApplied`, graceful `shutdownComplete`, panic `shutdownComplete`, and successful restart after panic.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- Evidence is recorded in `.planning/phases/08-real-visual-runtime-path/08-real-visual-runtime-path-05-UAT.md`.

2026-06-13 task `td-d38373`:

- `npm test` passed: 3 files, 22 tests.
- `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed.
- `/Applications/SuperCollider.app/Contents/Resources/scsynth` exists.
- `SCRYSYNTH_SCSYNTH_PATH=/Applications/SuperCollider.app/Contents/Resources/scsynth npm run tauri dev` launched the Tauri app after elevated localhost permission.
- Resumed real-`scsynth` UAT verified audible output, live parameter update, panic, and restart-after-panic by human confirmation.
- Resumed UAT found Stop disabled while runtime was `ready / healthy`; fixed frontend projection so active `ready` patches are stoppable and added a regression test.
- Stop retest passed by human confirmation. Phase 7 completion evidence is in `.planning/phases/07-real-supercollider-execution/07-real-supercollider-execution-07-UAT.md`.

## Session Continuity

Last consolidated: 2026-06-12.
Current td issue: `td-11fe55`.
