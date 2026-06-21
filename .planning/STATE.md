---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: v1 runtime hardening
status: hardening
stopped_at: Phase 10 session-aware agent orchestration planned
last_updated: "2026-06-21T00:00:00-05:00"
last_activity: 2026-06-21
progress:
  foundation_phases_total: 5
  foundation_phases_completed: 5
  hardening_phases_total: 6
  hardening_phases_completed: 3
  current_stage: Phase 10 agent orchestration is planned and ready for implementation
---

# Project State

## Project Reference

See `.planning/PROJECT.md` and `.planning/ROADMAP.md`.

**Core value:** The instrument must let a human and agent shape a live audiovisual session together through conversation, graph structure, and direct performance control without losing legibility or human override.

**Current focus:** v1 runtime hardening.

## Current Position

Scrysynth is a late foundation prototype in v1 runtime hardening. The planned foundation phases have produced a canonical app-owned session model, React workspace, Tauri command layer, persistence, graph editing, scene/variation controls, agent safety scaffolding, visual-runtime scaffolding, macro controls, and MIDI/OSC binding models.

The project is not yet a complete local audiovisual instrument. Runtime execution still needs production UAT and additional hardening:

- Audio can launch `scsynth`, apply v1 SynthDef/topology commands over OSC, produce audible output from the default source-to-output graph, apply live parameter updates, stop, panic, and restart after panic on a real local SuperCollider install.
- Visual runtime management now launches the in-repo minimal `scrysynth-visual` sidecar, handshakes over JSON lines, loads compiled scene snapshots, applies live parameter batches, stops, panics, and restarts after panic. The renderer remains minimal and GPU-free; richer Bevy output and packaged sidecar wiring are still future work.
- MIDI/OSC listener startup/configuration, learn, and post-learn routing are verified through the app-owned runtime path with a CoreMIDI virtual source and local OSC sender. GUI click-through hardware UAT remains release polish.
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
2. Session-aware agent orchestration. Phase plan: `.planning/phases/10-session-aware-agent-orchestration/10-session-aware-agent-orchestration-01-PLAN.md`.
3. Packaging, UAT, and release readiness.

## Completed Hardening Work

| Phase | Status | Result |
|-------|--------|--------|
| 7. Real SuperCollider Execution | Complete | Default source-to-output graph verified against local `scsynth` with audible playback, live parameter update, stop, panic, and restart-after-panic. |
| 8. Real Visual Runtime Path | Complete for minimal sidecar | `scrysynth-visual` starts as a separate process, acknowledges handshake and scene load, applies live parameter batches, stops, panics, and restarts after panic. Runtime Health now surfaces missing sidecar, booting, ready, degraded, disconnected, stopped, panicked, and restartable visual states. |
| 9. Hardware Input Runtime Wiring | Complete for app-runtime path | CoreMIDI virtual MIDI and local OSC sender verified live learn plus post-learn routing for macro, scene recall, transport play, transport stop, and panic. Panic stopped an active SuperCollider patch and left audio `Idle/PanicRecovered`. |

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

- Implement Phase 10 session-aware agent orchestration from epic `td-cebec0`.
- Run a physical-controller GUI click-through pass when hardware is available.

## Latest Verification

2026-06-21 planning:

- Phase 10 session-aware agent orchestration was planned in `.planning/phases/10-session-aware-agent-orchestration/10-session-aware-agent-orchestration-01-PLAN.md`.
- `td-cebec0` was created as the Phase 10 epic with child tasks `td-665cc6`, `td-0e888f`, `td-84b783`, `td-a842b4`, `td-4a9cfe`, and `td-ab3c29`.
- Phase 10 scope keeps all agent mutations behind typed commands, ownership gates, risk tiers, approvals, freeze/reclaim controls, and action history while adding a bounded session context packet and provider-agnostic planner above that safety boundary.

2026-06-19 task `td-8062dd`:

- Phase 9 UAT evidence recorded in `.planning/phases/09-hardware-input-runtime-wiring/09-hardware-input-runtime-wiring-06-UAT.md`.
- Scratch UAT runner used CoreMIDI virtual source `Scrysynth Phase 9 UAT Virtual MIDI`, local OSC sender `127.0.0.1:55970`, and `/Applications/SuperCollider.app/Contents/Resources/scsynth`.
- MIDI learn verified CC channel 0 controller 7 for macro `energy` and CC channel 0 controller 8 for transport stop.
- OSC learn verified `/scrysynth/uat/energy` for macro `energy`, `/scrysynth/uat/scene` for scene recall, `/scrysynth/uat/play` for transport play, and `/scrysynth/uat/panic` for panic.
- Post-learn routing verified macro values `0.252` from MIDI and `0.420` from OSC, scene recall with active visual scene and macro override `0.650`, transport play with audio `Ready/Healthy` and active patch `patch-v1-be42ba9bd41741b2`, transport stop to `Idle/Unknown`, and panic to `Idle/PanicRecovered` with panic recovery count `1`.
- Frontend polling was updated so a newly captured binding clears backend learn mode automatically before live routing continues.
- Verification passed: `npm test` (4 files, 38 tests), `npm run build`, and `cargo test --manifest-path src-tauri/Cargo.toml` outside the sandbox. The first sandboxed Rust run failed only because OSC tests could not bind localhost UDP sockets.
- `npm run tauri dev` launched Vite and `target/debug/scrysynth`, but GUI click-through was not claimed because the dev-launched process exposed no reliable accessibility window in this environment.

2026-06-17 planning:

- Phase 9 hardware input runtime wiring was planned in `.planning/phases/09-hardware-input-runtime-wiring/09-hardware-input-runtime-wiring-01-PLAN.md`.
- `td-dcaf9a` was created as the Phase 9 epic with child tasks `td-fca6b3`, `td-c18ac3`, `td-499618`, `td-eb6bc2`, `td-962e1b`, and `td-8062dd`.

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
Current Phase 10 epic: `td-cebec0`.
Current planning task: `td-543912`.
