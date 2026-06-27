---
gsd_state_version: 1.0
milestone: v2.0
milestone_name: Studio-Grade Instrument
current_phase: 13
current_phase_name: Graph UX Rebuild
status: verifying
stopped_at: Phase 13 context gathered
last_updated: "2026-06-27T15:05:45.226Z"
last_activity: 2026-06-27
last_activity_desc: Phase 12 complete, transitioned to Phase 13
progress:
  total_phases: 7
  completed_phases: 1
  total_plans: 2
  completed_plans: 2
  percent: 14
---

# Project State

## Project Reference

See `.planning/PROJECT.md` and `.planning/ROADMAP.md`.

**Core value:** The instrument must let a human and agent shape a live audiovisual session together through conversation, graph structure, and direct performance control without losing legibility or human override.

**Current focus:** Phase 12 — node-catalog-foundation

## Current Position

Phase: 13 — Graph UX Rebuild
Plan: Not started
Status: Phase 12 vertical slice complete — catalog + sequencer + frontend + real-scsynth conformance all verified
Last activity: 2026-06-27 — Phase 12 complete, transitioned to Phase 13

Progress: [██░░░░░░░░] 14% (2/2 Phase 12 plans; 1/7 v2.0 phases complete)

## v2.0 Milestone

**Goal:** Transform Scrysynth from a verified v1 foundation into a studio-grade, fluently-playable audiovisual instrument.

| Phase | Goal (one-liner) | Requirements | Status |
|-------|------------------|--------------|--------|
| 12. Node Catalog Foundation | Data-driven node catalog as single source of truth; full synthesis chain | NODES-01..05 | Complete |
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

- **[v2.0 Catalog-as-data — Phase 12 COMPLETE]** Both plans shipped. Plan 01 delivered the compiled-in `NodeCatalogEntry` table + catalog-driven dispatch + 14 sclang-free v2 SynthDefs + per-parameter CV ports + control-bus allocation + schema 2 + two-phase v1 rejection. Plan 02 delivered the app-driven `SequencerController` (std::thread tick loop, OSC `/c_set` per step, AtomicBool shutdown), the catalog-driven frontend (PrimitivePalette + NodeInspector + relaxed Zod schemas + `get_node_catalog` Tauri command), and the real-scsynth conformance gate that boots `scsynth` and `/d_recv`s every catalog entry's `.scsyndef` — **passes locally** (13/13 entries loaded). End-to-end CV modulation (LFO→filter cutoff_cv) verified against real scsynth. All four Phase 12 success criteria met.
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

**Last session:** 2026-06-27T15:05:45.218Z

**Stopped at:** Phase 13 context gathered
**Resume file:** .planning/phases/13-graph-ux-rebuild/13-CONTEXT.md
**Next action:** Phase 13 (Graph UX Rebuild) planning, OR run `/gsd-verify-work 12` for a UAT pass on the catalog foundation, OR `/gsd-plan-phase 14` to run the Visuals Compositing Spike in parallel (no v2 dependencies).

## Performance Metrics

| Phase | Plan | Duration | Notes |
|-------|------|----------|-------|
| 12 | 01 | 55min | Catalog foundation: NodeCatalogEntry table + catalog-driven dispatch + 14 v2 SynthDefs + CV control-bus allocation + schema 2 + two-phase v1 rejection; 2 atomic commits (a293cef, 55e1097); 98 lib + 22 audio_runtime tests pass |
| 12 | 02 | 23min | App-driven sequencer (SequencerController std::thread + /c_set) + frontend catalog consumption (palette/inspector/Zod relaxation + get_node_catalog command) + real-scsynth conformance gate (#[ignore], passes locally — 13/13 catalog synthdefs load) + end-to-end CV modulation verified; 3 atomic commits (80349db, 2c673f8, 80b3be0); 103 lib + 27 audio_runtime + 49 vitest tests pass |
| _(v2.0 execution continues)_ | | | |

_Velocity baseline from v1.0: Phase 11 Plan 01 = 54min, 3 tasks, 11 files. Per-phase v1 plan counts: P1=6, P2=3, P3=3, P4=3, P5=3, P7=1, P8=1, P9=1, P10=1, P11=2._

## Operator Next Steps

- `/gsd-verify-work 12` to run a UAT pass on the completed catalog foundation, OR
- `/gsd-plan-phase 13` to plan Graph UX Rebuild (the natural next phase — depends on Phase 12), OR
- `/gsd-plan-phase 14` to run the Visuals Compositing Spike in parallel (no v2 dependencies), OR
- `/gsd-progress` to see the unified situational view.
