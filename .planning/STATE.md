---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: v1 runtime hardening
status: planning
stopped_at: Consolidated foundation-complete audit and runtime-hardening roadmap
last_updated: "2026-06-14T00:00:00-05:00"
last_activity: 2026-06-14
progress:
  foundation_phases_total: 5
  foundation_phases_completed: 5
  hardening_phases_total: 6
  hardening_phases_completed: 1
  current_stage: Phase 7 real SuperCollider execution complete; visual runtime hardening next
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
- Visual runtime management exists, but no `scrysynth-visual` sidecar is included and scene/parameter updates are stubs.
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
2. Real visual sidecar/protocol.
3. Hardware listener lifecycle and runtime wiring.
4. Session-aware agent orchestration.
5. Packaging, UAT, and release readiness.

## Completed Hardening Work

| Phase | Status | Result |
|-------|--------|--------|
| 7. Real SuperCollider Execution | Complete | Default source-to-output graph verified against local `scsynth` with audible playback, live parameter update, stop, panic, and restart-after-panic. |

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

- Decide whether `scrysynth-visual` is built in-repo as a workspace crate or supplied externally.
- Add user-facing runtime setup diagnostics for missing `scsynth`, missing visual sidecar, and unavailable hardware input.

## Latest Verification

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
Current td issue: `td-d38373`.
