---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: v1 runtime hardening
current_phase: 0
status: v1 shipped — awaiting next milestone
stopped_at: "v1.0 milestone archived 2026-06-26; packaged .app UAT-verified on Apple Silicon"
last_updated: "2026-06-26T18:33:26-05:00"
last_activity: 2026-06-26
last_activity_desc: Milestone v1.0 (v1 Runtime Hardening) completed and archived
progress:
  total_phases: 6
  completed_phases: 6
  total_plans: 6
  completed_plans: 6
  percent: 100
---

# Project State

## Project Reference

See `.planning/PROJECT.md` and `.planning/ROADMAP.md`.

**Core value:** The instrument must let a human and agent shape a live audiovisual session together through conversation, graph structure, and direct performance control without losing legibility or human override.

**Current focus:** v1.0 shipped — planning next milestone (run `/gsd-new-milestone`)

## Current Position

Scrysynth **v1.0 (1.0.0) shipped** on macOS Apple Silicon — ad-hoc-signed `scrysynth.app` + `.dmg`. The v1 Runtime Hardening milestone (Phases 7–11, with Phase 6 dev-readiness folded into Phase 11) is complete and archived to `.planning/milestones/v1.0-ROADMAP.md`. All milestone work was re-verified end-to-end against the packaged app in the consolidated Phase 11 UAT (9/9 scenario areas passed).

Phase: — (milestone complete)
Plan: —
Status: v1 shipped — awaiting next milestone
Last activity: 2026-06-26 — Milestone v1.0 archived

Deferred beyond v1 (tracked for a future milestone, not active v1 work): richer Bevy-rendered visuals / visible render window, live provider-backed agent orchestration (AGNT-01R), physical-controller GUI click-through UAT, Windows/Linux/Intel/universal builds, full Developer ID notarization + auto-update, and multiplayer. See PROJECT.md "Active — Next Milestone Candidates" and the v1.0 archive for the full deferred list.

## Completed Foundation

| Phase | Status | Result |
|-------|--------|--------|
| 1. Session Core & Recall | Complete | Canonical session schema, generated TS contracts, JSON persistence, graph/inspector workspace. |
| 2. Playable Audio Graph Foundation | Complete | Graph edit commands, audio primitive model, topology compiler, runtime lifecycle shell, transport/panic controls. |
| 3. Performance Workspace | Complete | Conversation/graph/performance view switching, scene recall, variation save/restore. |
| 4. Agent Collaboration Scaffold | Complete | Deterministic intent parser, ownership gates, approvals, action history, conversation UI. |
| 5. Visual Sync & Cross-Modal Control Scaffold | Complete | Visual adapter shell, runtime health panel, cross-domain macros, MIDI/OSC binding models. |

## Active Hardening Work

None — the v1 runtime-hardening milestone is complete.

Deferred beyond v1 (tracked for a future milestone, not active v1 work):

1. Live provider-backed agent orchestration on top of the verified planner boundary (AGNT-01R).
2. Richer Bevy-rendered visuals and a visible render window.
3. Physical-controller GUI click-through UAT.
4. Windows / Linux / Intel / universal builds.
5. Full Developer ID notarization + auto-update.

## Completed Hardening Work

| Phase | Status | Result |
|-------|--------|--------|
| 7. Real SuperCollider Execution | Complete | Default source-to-output graph verified against local `scsynth` with audible playback, live parameter update, stop, panic, and restart-after-panic. |
| 8. Real Visual Runtime Path | Complete for minimal sidecar | `scrysynth-visual` starts as a separate process, acknowledges handshake and scene load, applies live parameter batches, stops, panics, and restarts after panic. Runtime Health now surfaces missing sidecar, booting, ready, degraded, disconnected, stopped, panicked, and restartable visual states. |
| 9. Hardware Input Runtime Wiring | Complete for app-runtime path | CoreMIDI virtual MIDI and local OSC sender verified live learn plus post-learn routing for macro, scene recall, transport play, transport stop, and panic. Panic stopped an active SuperCollider patch and left audio `Idle/PanicRecovered`. |
| 10. Session-Aware Agent Orchestration | Complete for deterministic/mock planner path | Bounded context packets, provider-agnostic planner requests, realistic mock proposal normalization, typed-command validation, approval/rejection, freeze/reclaim, provider-unavailable diagnostics, invalid-response diagnostics, and frontend proposal/diagnostic rendering are verified. Live provider-backed planning remains future hardening. |
| 11. Release Readiness | Complete | Packaged ad-hoc-signed `scrysynth.app` + `.dmg` for `aarch64-apple-darwin`; consolidated nine-scenario packaged-app UAT passed (save/open, graph edit, audio playback, scene/variation recall, macro control, hardware learn, agent approval, visual runtime, panic recovery); README/RELEASE_NOTES/ROADMAP/STATE/REQUIREMENTS reconciled. REL-01/02/03 verified. Planner wired into production GUI (`415e8d8`). |

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
- [Phase ?]: Phase 11 Plan 01: v1 Tauri bundle config delivered (1.0.0, ad-hoc macOS signing, Apple Silicon only, targets [app,dmg], externalBin visual sidecar via tauri-plugin-shell). BevySidecarAdapter launches the bundled sidecar through app.shell().sidecar() in the packaged path and keeps the raw-Command dev/test override. scsynth stays external with a polished discovery message. REL-01 substrate delivered here; the actual release build + first-run smoke (REL-01 verification) is owned by Plan 02.
- [Phase 11 Plan 02]: v1 released. Release build produced ad-hoc-signed `scrysynth.app` + `scrysynth_1.0.0_aarch64.dmg` for `aarch64-apple-darwin`. Planner wired into the production GUI (`415e8d8`: `send_agent_message` → `handle_agent_message` → `plan_and_apply_agent_request`; approve/reject IPC param fixed; additive `remove … agent …` high-risk fallback). Consolidated nine-scenario packaged-app UAT passed. REL-01/02/03 verified. D-03 honest caveat: no Gatekeeper "Open Anyway" prompt on the build host because local builds carry no `com.apple.quarantine`; the prompt is the end-user download path, documented in README/RELEASE_NOTES. Deferred beyond v1: live provider-backed agent (AGNT-01R), richer Bevy visuals, physical-controller click-through, Windows/Linux/Intel, full notarization + auto-update, multiplayer.

## Pending Todos

All v1 work is complete. Deferred beyond v1 (future milestone):

- Connect the verified Phase 10 provider-agnostic planner boundary to a live provider-backed agent and run provider setup/fallback UAT (AGNT-01R).
- Run a physical-controller GUI click-through pass when hardware is available.
- Replace the minimal GPU-free visual sidecar with richer Bevy-rendered output and a visible render window.
- Add Windows / Linux / Intel / universal macOS build targets.
- Add full Developer ID signing + notarization + an auto-updater.

## Latest Verification

2026-06-26 Phase 11 (Plan 02) — v1 release verified:

- Consolidated packaged-app UAT recorded in `.planning/phases/11-release-readiness/11-release-readiness-03-UAT.md`: all nine REL-02 scenario areas passed against the rebuilt ad-hoc-signed `scrysynth.app` (commit `415e8d8`, rebuilt 2026-06-26) — save/open, graph edit, audio playback (real `scsynth`), scene/variation recall, macro control, hardware learn (CoreMIDI + local OSC), agent approval (scenario 7 reachable via the planner-wiring fix; trigger `remove agent layer`), visual runtime (bundled minimal sidecar), and panic recovery (audio + visual).
- Release build + ad-hoc sign + secret-leak check + first-run smoke recorded in `.planning/phases/11-release-readiness/11-release-readiness-02-BUILD-EVIDENCE.md`: `.app` + `.dmg` for `aarch64-apple-darwin`, `Signature=adhoc`, zero `APPLE_`/`TAURA` leakage, bundled sidecar signed inside the app, app launches via right-click → Open. No Gatekeeper prompt on the build host because the local build carries no `com.apple.quarantine` (the "Open Anyway" prompt is the end-user download path, documented in README/RELEASE_NOTES).
- REL-01, REL-02, REL-03 marked complete in `.planning/REQUIREMENTS.md`; runtime-ready rows AUD-01R..04R, VIS-01R..03R, DEV-01..02 reconciled; AGNT-01R (live provider) left open. README rewritten and `RELEASE_NOTES.md` added with Supported/Not-supported matrices.
- Verification passed: `npm test`, `npm run build` (no code regression from Plan 02 doc edits).

2026-06-21 task `td-ab3c29`:

- Phase 10 UAT evidence recorded in `.planning/phases/10-session-aware-agent-orchestration/10-session-aware-agent-orchestration-06-UAT.md`.
- Provider mode was deterministic/mock planner fixtures plus local parser provider; no live remote provider, credentials, or streaming provider UI were claimed.
- UAT verified bounded context packet derivation, realistic multi-step planner-shaped proposal normalization, parameter validation rejection before mutation, scene recall, high-risk pending approval and rejection, frozen-agent rejection, reclaim ownership, provider-unavailable diagnostics, session-agent-unavailable diagnostics, invalid-provider-response diagnostics, and frontend proposal/diagnostic rendering.
- Variation save/restore remains covered by performance command/store tests, but no planner-authored Phase 10 variation fixture was claimed.
- Verification passed: targeted frontend/store tests (3 files, 36 tests), targeted Rust agent command tests (36 tests), `npm test` (5 files, 47 tests), `npm run build`, and full `cargo test --manifest-path src-tauri/Cargo.toml` after rerunning outside the sandbox for OSC UDP bind permission.

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

**Stopped at:** Completed 11-release-readiness-01-PLAN.md
**Resume file:** None

Last session: 2026-06-24T03:30:08.671Z
Phase 10 reconciled into GSD: implementation commits back-referenced via `10-session-aware-agent-orchestration-01-SUMMARY.md`, Phase 10.6 UAT + doc reconciliation committed, `td-ab3c29` review closed.
Phase 10 epic: `td-cebec0` (closed).
Next: Phase 11 (Release Readiness) planning, or the Phase 10 live-provider follow-on (connect the verified provider-agnostic planner boundary to a live provider + setup/fallback UAT).

## Performance Metrics

| Phase | Plan | Duration | Notes |
|-------|------|----------|-------|
| Phase 11 P01 | 54min | 3 tasks | 11 files |

## Deferred Items

Items acknowledged and deferred at v1.0 milestone close on 2026-06-26. The pre-close artifact audit surfaced 6 historical UAT files with non-`passed` metadata statuses; all have **0 pending scenarios** and were superseded by the consolidated Phase 11 packaged-app UAT (9/9 scenario areas passed). Recorded here for traceability, not as open work:

| Category | Item | Status |
|----------|------|--------|
| uat | Phase 01 `01-UAT.md` | `[diagnosed]` metadata — 0 pending scenarios; superseded by Phase 11 release UAT |
| uat | Phase 07 `07-real-supercollider-execution-07-UAT.md` | `[unknown]` metadata — 0 pending scenarios; audio re-verified in Phase 11 UAT scenario 3 |
| uat | Phase 08 `08-real-visual-runtime-path-05-UAT.md` | `[passed]` — 0 pending scenarios |
| uat | Phase 09 `09-hardware-input-runtime-wiring-06-UAT.md` | `[passed]` — 0 pending scenarios |
| uat | Phase 10 `10-session-aware-agent-orchestration-06-UAT.md` | `[unknown]` metadata — 0 pending scenarios; agent path re-verified in Phase 11 UAT scenario 7 |
| uat | Phase 11 `11-release-readiness-03-UAT.md` | `[pass]` — 0 pending scenarios; the consolidated release UAT |

**Known requirement gap at close:** `AGNT-01R` (live provider-backed agent planner) is the only unchecked v1 requirement. The deterministic/mock planner + production-GUI wiring is verified; a live LLM provider remains future hardening. See `.planning/milestones/v1.0-REQUIREMENTS.md`.

## Operator Next Steps

- Start the next milestone with `/gsd-new-milestone` (will define fresh requirements + roadmap)
