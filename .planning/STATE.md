---
gsd_state_version: 1.0
milestone: v2.0
milestone_name: Studio-Grade Instrument
current_phase: 12
current_phase_name: first v2.0 phase
status: planning
stopped_at: Phase 12 planned (2 plans, checker PASS) — ready for execute
last_updated: "2026-06-26T21:45:49.119Z"
last_activity: 2026-06-26
last_activity_desc: "v2.0 roadmap created (7 phases: 12–18), 29/29 requirements mapped"
progress:
  total_phases: 7
  completed_phases: 0
  total_plans: 2
  completed_plans: 0
  percent: 0
---

# Project State

## Project Reference

See `.planning/PROJECT.md` and `.planning/ROADMAP.md`.

**Core value:** The instrument must let a human and agent shape a live audiovisual session together through conversation, graph structure, and direct performance control without losing legibility or human override.

**Current focus:** v2.0 Studio-Grade Instrument — Phase 12 (Node Catalog Foundation) ready to plan.

## Current Position

Phase: 12 of 18 — Node Catalog Foundation (first v2.0 phase)
Plan: — (not yet planned)
Status: Ready to plan
Last activity: 2026-06-26 — v2.0 roadmap created (7 phases: 12–18), 29/29 requirements mapped

Progress: [░░░░░░░░░░] 0% (0/7 v2.0 phases complete)

## v2.0 Milestone

**Goal:** Transform Scrysynth from a verified v1 foundation into a studio-grade, fluently-playable audiovisual instrument.

| Phase | Goal (one-liner) | Requirements | Status |
|-------|------------------|--------------|--------|
| 12. Node Catalog Foundation | Data-driven node catalog as single source of truth; full synthesis chain | NODES-01..05 | Not started |
| 13. Graph UX Rebuild | Fluent patching through typed commands; canonical positions | GRAPH-01..05 | Not started |
| 14. Visuals Compositing Spike | One-day macOS PoC; decision gate for Phase 15 | VISUAL-01 | Not started |
| 15. Visuals Behind the Grid | Richer Bevy ambient layer behind graph, separate adapter | VISUAL-02..04 | Not started |
| 16. Focused Shell & Live Agent | Graph-hero shell + live LLM agent with safety inheritance (parallel tracks) | SHELL-01..04, AGENT-01..04 | Not started |
| 17. Cross-Platform Builds | Windows / Linux / Intel / universal macOS | PLATFORM-01..04 | Not started |
| 18. Notarization & Auto-Updater | Signed + notarized + Ed25519 auto-update via CI | RELEASE-01..03 | Not started |

**Coverage:** 29/29 v2.0 requirements mapped. No orphans, no duplicates.

## Completed Foundation (v1.0 — shipped 2026-06-26)

v1.0 (1.0.0) shipped as ad-hoc-signed macOS Apple Silicon `scrysynth.app` + `.dmg`. 36/37 v1 requirements verified. 11 phases complete (foundation 1–5 + hardening 7–11; Phase 6 folded into 11). Consolidated Phase 11 packaged-app UAT passed 9/9 scenario areas. See `.planning/milestones/v1.0-ROADMAP.md` for the full v1 record.

Implemented & verified in v1: canonical Rust `SessionDocument` + generated TS contracts, versioned JSON persistence, React Flow workspace, conversation/graph/performance views, command layer with ownership/approval/freeze/reclaim gates, real SuperCollider execution (audible playback + panic + restart), minimal `scrysynth-visual` sidecar (bundled via `tauri-plugin-shell`), CoreMIDI + local OSC hardware learn, provider-agnostic session-aware agent planner (deterministic/mock path, wired into production GUI), ad-hoc release packaging.

## Decisions

Recent decisions affecting v2.0 work (full log in PROJECT.md Key Decisions):

- **[v2.0 Roadmap]** Honor the research's converged 8-phase dependency order with coarse-granularity compression: combine parallelizable Pro Shell + Live Agent into Phase 16; keep Visuals Compositing Spike (Phase 14) as a standalone decision gate per HIGH-RISK flag; keep Cross-Platform (Phase 17) and Notarization (Phase 18) separate because RELEASE needs final stable per-platform binaries.
- **[v2.0 Catalog-as-data]** Refactor v1's three hardcoded allowlists (`synthdef_resource`, `normalize_parameter_name`, `validate_runtime_target`) into one Rust-owned `NodeCatalogEntry` table — single highest-leverage v2 decision, unblocks Phases 13, 15, and 16 simultaneously.
- **[v2.0 Layout command]** Add a dedicated `SetNodeLayout`/`MoveNode` command variant that bypasses `AudioRuntimeManager.reconcile_graph_edit`, so drag does not recompile topology or re-trigger envelopes every frame (Pitfalls V2-2, V2-14).
- **[v2.0 Agent safety]** Insert `validate_planner_proposal(&proposal, &session)` before typed-command gates; re-derive confidence → risk-tier → approval-threshold mapping for LLM continuous values; fuzz the validator (Pitfall V2-4).
- **[v1 carry-overs]** Live provider-backed agent (AGNT-01R), richer Bevy visuals, cross-platform builds, and full notarization + auto-update fold into v2.0 as AGENT-01..04, VISUAL-02..04, PLATFORM-01..04, RELEASE-01..03.

## Pending Todos

None yet for v2.0. (v1 close items are recorded under Deferred Items below.)

## Blockers/Concerns

- **Visuals compositing (Phase 14):** HIGHEST-RISK v2 item. Must be settled by the Phase 14 spike before Phase 15 is planned in detail. If Strategy A (borderless Bevy behind transparent webview) fails on macOS, default to Strategy B (headless render-to-texture frame stream) everywhere.
- **Linux SuperCollider bundling (Phase 17):** No official prebuilt binary. Product decision (require system install + document vs. build-from-source in CI vs. distro package) to make at Phase 17 start, not discovered mid-build.
- **macOS minimum version (Phase 15/17):** `tauri.conf.json` currently says `11.0`; Bevy 0.19 / wgpu 29 may require macOS 12+. Validate during Phase 15, lock during Phase 17.
- **Windows Authenticode cert lead time (Phase 18):** EV/OV cert acquisition is its own lead-time item — start early in Phase 18 (or Phase 17) to avoid blocking release.
- **Signing graph (Phase 18):** App + sidecar + bundled SC must all be signed; ad-hoc signing cannot ship an updater; notarization rejects any unsigned bundled binary.

## Deferred Items

Items acknowledged and carried forward from v1.0 close. The pre-close artifact audit surfaced 6 historical UAT files with non-`passed` metadata statuses; all have 0 pending scenarios and were superseded by the consolidated Phase 11 packaged-app UAT (9/9 scenario areas passed). Recorded for traceability, not as open work:

| Category | Item | Status |
|----------|------|--------|
| uat | Phase 01/07/08/09/10/11 historical UAT metadata | All superseded by Phase 11 consolidated UAT (9/9 passed); 0 pending scenarios |
| carry-over | Physical-controller GUI click-through UAT | Deferred — perform when hardware is available (beyond v2.0) |
| carry-over | VST/AU/LV2 plugin hosting | Out of scope — preserve adapter seam only |
| carry-over | Node library expansion beyond ~16 curated nodes | Deferred beyond v2.0 — quality over quantity in v2 |

## Session Continuity

**Last session:** 2026-06-26T21:45:49.111Z

**Stopped at:** Phase 12 planned (2 plans, checker PASS) — ready for execute
**Resume file:** .planning/phases/12-node-catalog-foundation/12-01-PLAN.md
**Next action:** `/gsd-plan-phase 12` (Node Catalog Foundation) — or run the Visuals Compositing Spike (Phase 14) in parallel since it has no v2 dependencies.

## Performance Metrics

| Phase | Plan | Duration | Notes |
|-------|------|----------|-------|
| _(v2.0 execution not yet started)_ | | | |

_Velocity baseline from v1.0: Phase 11 Plan 01 = 54min, 3 tasks, 11 files. Per-phase v1 plan counts: P1=6, P2=3, P3=3, P4=3, P5=3, P7=1, P8=1, P9=1, P10=1, P11=2._

## Operator Next Steps

- `/gsd-plan-phase 12` to plan Node Catalog Foundation, OR
- `/gsd-plan-phase 14` to run the Visuals Compositing Spike in parallel (no v2 dependencies), OR
- `/gsd-progress` to see the unified situational view.
