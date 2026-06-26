# Roadmap: Scrysynth

## Milestones

- ✅ **v1.0 Runtime Hardening** — Phases 1–11 (shipped 2026-06-26) — `[archive](milestones/v1.0-ROADMAP.md)`
- 📋 **Next milestone** — not yet defined (run `/gsd-new-milestone`)

## v1.0 Phases

<details>
<summary>✅ v1.0 Runtime Hardening (Phases 1–11) — SHIPPED 2026-06-26</summary>

**Foundation (Phases 1–5):**

- [x] Phase 1: Session Core & Recall (6 plans) — completed 2026-04-11
- [x] Phase 2: Playable Audio Graph Foundation (3 plans) — completed 2026-04-11
- [x] Phase 3: Performance Workspace (3 plans) — completed 2026-04-11
- [x] Phase 4: Agent Collaboration Scaffold (3 plans) — completed 2026-04-11
- [x] Phase 5: Visual Sync & Cross-Modal Control Scaffold (3 plans) — completed 2026-04-11

**Runtime Hardening (Phases 6–11):**

- [x] Phase 6: Local Developer Readiness — folded into Phase 11 (DEV-01/02 verified there)
- [x] Phase 7: Real SuperCollider Execution — audible playback + panic + restart verified against local `scsynth`
- [x] Phase 8: Real Visual Runtime Path — minimal `scrysynth-visual` sidecar, JSON-lines protocol, full lifecycle verified
- [x] Phase 9: Hardware Input Runtime Wiring — CoreMIDI + local OSC learn and post-learn routing verified
- [x] Phase 10: Session-Aware Agent Orchestration (1 plan) — bounded context, provider-agnostic planner, typed-command gates, freeze/reclaim verified
- [x] Phase 11: Release Readiness (2 plans) — ad-hoc-signed Apple-Silicon `.app` + `.dmg`, 9/9 packaged-app UAT scenarios passed

**Requirement outcome:** 36/37 v1 requirements verified. `AGNT-01R` (live provider-backed agent) deferred beyond v1.

</details>

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Session Core & Recall | v1.0 | 6/6 | Complete | 2026-04-11 |
| 2. Playable Audio Graph Foundation | v1.0 | 3/3 | Complete | 2026-04-11 |
| 3. Performance Workspace | v1.0 | 3/3 | Complete | 2026-04-11 |
| 4. Agent Collaboration Scaffold | v1.0 | 3/3 | Complete | 2026-04-11 |
| 5. Visual Sync & Cross-Modal Scaffold | v1.0 | 3/3 | Complete | 2026-04-11 |
| 6. Local Developer Readiness | v1.0 | — (folded into 11) | Complete | 2026-06-26 |
| 7. Real SuperCollider Execution | v1.0 | 1/1 | Complete | 2026-06-13 |
| 8. Real Visual Runtime Path | v1.0 | 1/1 | Complete | 2026-06-16 |
| 9. Hardware Input Runtime Wiring | v1.0 | 1/1 | Complete | 2026-06-19 |
| 10. Session-Aware Agent Orchestration | v1.0 | 1/1 | Complete | 2026-06-21 |
| 11. Release Readiness | v1.0 | 2/2 | Complete | 2026-06-26 |

## Backlog

Carry-over candidates from v1 deferrals (to be promoted into the next milestone via `/gsd-new-milestone`):

- Live provider-backed agent orchestration (AGNT-01R) — connect the verified planner boundary to a live LLM provider.
- Richer Bevy-rendered visuals + a visible render window (replace the minimal GPU-free sidecar).
- Physical-controller GUI click-through UAT when hardware is available.
- Windows / Linux / Intel / universal macOS build targets.
- Full Developer ID signing + notarization + an auto-updater.
- Multiplayer collaboration (v2 scope).

## Deferred Beyond v1

- Multiplayer collaboration.
- Full DAW-style timeline/arrangement.
- Deep projection mapping or media-server workflows.
- General plugin marketplace.
- Fine-grained ownership policies beyond v1 freeze/reclaim/approval.
- Advanced visual graph editing and sophisticated transition/morph authoring.

---

_For the full v1.0 milestone record, see `.planning/milestones/v1.0-ROADMAP.md` and `.planning/milestones/v1.0-REQUIREMENTS.md`. For shipped-state summary, see `.planning/MILESTONES.md`._
