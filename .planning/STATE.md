---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: v1 runtime hardening
status: planning
stopped_at: Consolidated foundation-complete audit and runtime-hardening roadmap
last_updated: "2026-06-12T00:00:00-05:00"
last_activity: 2026-06-12
progress:
  foundation_phases_total: 5
  foundation_phases_completed: 5
  hardening_phases_total: 6
  hardening_phases_completed: 0
  current_stage: foundation complete; runtime hardening next
---

# Project State

## Project Reference

See `.planning/PROJECT.md` and `.planning/ROADMAP.md`.

**Core value:** The instrument must let a human and agent shape a live audiovisual session together through conversation, graph structure, and direct performance control without losing legibility or human override.

**Current focus:** v1 runtime hardening.

## Current Position

Scrysynth is a late foundation prototype. The planned foundation phases have produced a canonical app-owned session model, React workspace, Tauri command layer, persistence, graph editing, scene/variation controls, agent safety scaffolding, visual-runtime scaffolding, macro controls, and MIDI/OSC binding models.

The project is not yet a complete local audiovisual instrument. Runtime execution still needs to be made real:

- Audio can launch `scsynth`, but topology loading currently marks the runtime ready without applying real synthdefs, OSC bundles, buses, groups, or node controls.
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
2. Real SuperCollider execution through OSC.
3. Real visual sidecar/protocol.
4. Hardware listener lifecycle and runtime wiring.
5. Session-aware agent orchestration.
6. Packaging, UAT, and release readiness.

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

- Install local frontend dependencies and confirm `npm test` plus `npm run build`.
- Ensure Rust stable/cargo is installed and confirm `cargo test --manifest-path src-tauri/Cargo.toml`.
- Implement or document the expected SuperCollider runtime behavior precisely.
- Decide whether `scrysynth-visual` is built in-repo as a workspace crate or supplied externally.
- Add user-facing runtime setup diagnostics for missing `scsynth`, missing visual sidecar, and unavailable hardware input.

## Session Continuity

Last consolidated: 2026-06-12.
Current td issue: `td-c20fe5`.
