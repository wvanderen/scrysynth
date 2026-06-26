---
phase: 12-node-catalog-foundation
plan: "01"
subsystem: infra
tags: [rust, supercollider, synthdef, catalog, ts-rs, serde, cv-modulation, topo-compiler]

# Dependency graph
requires:
  - phase: 11-release-readiness (v1.0 foundation)
    provides: v1 closed-enum node model + topology compiler + OSC adapter + ts-rs pipeline + deterministic Python SynthDef byte-writer
provides:
  - "Compiled-in NodeCatalogEntry const table as single source of truth for node identity, SynthDef mapping, ports, parameters, visual shape"
  - "Catalog-driven dispatch (find_catalog_entry) replacing v1's 3 allowlists + 2 enum-dispatch spots; UnknownCatalogEntry Err replaces the unreachable!() panic"
  - "~15 catalog entries (oscillator/noise/filter/envelope/lfo/vca/quantizer/mixer/delay/reverb/distortion/chorus/flanger/step_sequencer/output)"
  - "Per-parameter CV-input ports declared in the catalog (D-04/D-05) + compiler control-bus allocation (FIRST_CONTROL_BUS_OFFSET=1024)"
  - "14 v2 SuperCollider SynthDefs (.scsyndef, SCgf v2) authored sclang-free via generate_synthdefs.py with CV-bus In.kr/In.ar reads"
  - "Reshaped Node domain (node_type_id + flat config + SequencerPattern); schemaVersion 2; two-phase v1 rejection (LegacyV1Session)"
  - "Regenerated src/generated/session-types.ts with NodeCatalogEntry/CatalogPortSpec/CatalogParamSpec/NodeCategory/SequencerPattern/OutputKind"
affects: [13-graph-ux-rebuild, 15-visuals-behind-the-grid, 16-focused-shell-live-agent, 12-02]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Catalog-as-data: one &'static const table drives compiler dispatch, route validation, visual shape, ts-rs export — adding a node is a data edit, never a new match arm"
    - "Control-bus allocation alongside audio-bus allocation (disjoint index ranges prevent audio/control clobber)"
    - "Two-phase session parse (schemaVersion probe before full deserialize) so model drift surfaces a friendly version error"

key-files:
  created:
    - src-tauri/src/catalog/mod.rs
    - src-tauri/src/catalog/entries.rs
    - src-tauri/resources/synthdefs/v2/generate_synthdefs.py
    - src-tauri/resources/synthdefs/v2/*.scsyndef (14 binaries)
  modified:
    - src-tauri/src/domain/session.rs
    - src-tauri/src/audio/synthdefs.rs
    - src-tauri/src/audio/compiler.rs
    - src-tauri/src/visual/compiler.rs
    - src-tauri/src/persistence/session_file.rs
    - src/generated/session-types.ts

key-decisions:
  - "Catalog returns ScResourcePlanError::UnknownCatalogEntry (single field node_type_id) so find_catalog_entry can construct it without node context; node_id is not needed for the success-criterion-#3 guarantee"
  - "CompiledNodeKind closed enum replaced by node_type_id: String + flat per-node config (bypassed/output_kind/channel_count) on CompiledNodeLaunch; category-level match remains for structural audio-bus wiring (Source/Effect/Mixer/Output) — adding a node of an existing category adds no match arm"
  - "Control-bus allocation: one bus per mod-source OUTPUT PORT (shared by all readers), not per route; audio-rate FM reuses the source's existing audio out bus (no new allocation)"
  - "CV-bus gating uses the silent-bus convention (unconnected CV args default to an unwritten bus index that reads 0.0) rather than a `bus >= 0` BinaryOp gate — avoids hand-rolling an error-prone BinaryOpUGen special-index in the byte writer; SC unwritten-bus-reads-zero is well established"
  - "Removed obsolete v1 runtime_target tests (legacy aliases, mismatched primitive) — D-09 clean break removed those concepts; replaced with the unknown-node_type_id → UnknownCatalogEntry test"

patterns-established:
  - "find_catalog_entry(&node.node_type_id)? is THE dispatch primitive — every former closed-enum match is now this lookup"
  - "ts-rs handles &'static str -> string and &'static [T] -> Array<T> natively; catalog types export their full shape without hand-maintained TS"
  - "Direct modulation routes (no audio bus_id) compile into CompiledTopology.cv_routes; audio signal-chain routes keep bus_id"

requirements-completed: [NODES-01, NODES-02, NODES-03, NODES-05]

# Metrics
duration: 55min
completed: 2026-06-26
status: complete
---

# Phase 12 Plan 01: Node Catalog Foundation Summary

**One compiled-in NodeCatalogEntry table replaces v1's 3 hardcoded allowlists + 2 enum-dispatch spots as the single source of truth, ships 14 sclang-free v2 SuperCollider SynthDefs with CV-bus args, and turns the v1 unreachable!() panic into a real UnknownCatalogEntry Err**

## Performance

- **Duration:** ~55 min
- **Started:** 2026-06-26T21:55:12Z
- **Completed:** 2026-06-26T22:49:51Z
- **Tasks:** 2
- **Files modified/created:** 40

## Accomplishments
- Established `crate::catalog` as the single source of truth: ~15 entries covering the full synthesis chain (oscillator/noise/filter/envelope/lfo/vca/quantizer/mixer), the effect family (delay/reverb/distortion/chorus/flanger), the app-driven step_sequencer (no SynthDef, D-06), and output — each declaring audio + per-parameter CV-input ports (D-04/D-05).
- Replaced all five v1 hardcoded dispatch spots with `find_catalog_entry`. An unknown `node_type_id` now returns `Err(ScResourcePlanError::UnknownCatalogEntry)` instead of the v1 `unreachable!()` panic — **success criterion #3 delivered** and verified by a dedicated test.
- Reshaped the domain model (D-09 clean break): `Node` carries `node_type_id: String` + flat per-node config + optional `SequencerPattern`; the closed enums (`NodeType`/`AudioPrimitive`/`AudioSource*`/`AudioEffect*`/`AudioOutput*`) and their config structs are removed; `CURRENT_SCHEMA_VERSION` bumped 1 → 2.
- Wired end-to-end CV modulation (NODES-05): the topology compiler compiles direct modulation routes into `cv_routes`, and the resource planner allocates control buses from `FIRST_CONTROL_BUS_OFFSET = 1024` (disjoint from audio buses), injecting `out_cv_bus` on mod sources and `<cv_port>_bus` on targets. Audio-rate FM reuses the source's audio out bus.
- Authored 14 v2 SuperCollider SynthDefs via the extended deterministic Python byte-writer (no sclang dependency); each `.scsyndef` has an SCgf v2 header and CV-bus `In.kr`/`In.ar` reads.
- Two-phase session parse (D-10, Pitfall #1): a v1 session file now surfaces the friendly `LegacyV1Session` message ("This is a v1 session — unsupported in Scrysynth v2…") before serde ever sees the removed `audioPrimitive` shape.
- Port signal-type validation (Pitfall #3): `validate_routes` rejects Audio↔Control CV mismatches at compile time (`TopologyCompileError::PortSignalTypeMismatch`).

## Task Commits

Each task was committed atomically (the domain reshape forced the dispatch refactor into the first commit, since removing the closed enums breaks the compiler/synthdefs at `cargo check` time):

1. **Task 1: Catalog module + domain reshape + v2 SynthDefs + catalog-driven dispatch + v1 rejection** — `a293cef` (feat)
2. **Task 2: CV control-bus allocation + port signal-type validation (NODES-05)** — `55e1097` (feat)

## Files Created/Modified
- `src-tauri/src/catalog/mod.rs` — NodeCatalogEntry/CatalogPortSpec/CatalogParamSpec/NodeCategory types + CATALOG const + find_catalog_entry
- `src-tauri/src/catalog/entries.rs` — the ~15 catalog entries (single source of truth)
- `src-tauri/src/domain/session.rs` — reshaped Node, SequencerPattern, OutputKind, SetStepValue, schema 2; removed closed enums; ts-rs decls
- `src-tauri/src/audio/synthdefs.rs` — catalog-driven plan_sc_resources, UnknownCatalogEntry, FIRST_CONTROL_BUS_OFFSET + plan_cv_buses, apply_parameters via catalog
- `src-tauri/src/audio/compiler.rs` — CompiledNodeKind → node_type_id + config; catalog-driven node_sort_key; cv_routes + CV-port validation
- `src-tauri/src/visual/compiler.rs` — catalog-driven visual shape
- `src-tauri/src/persistence/session_file.rs` — two-phase parse + LegacyV1Session
- `src-tauri/resources/synthdefs/v2/generate_synthdefs.py` + 14 `.scsyndef` — v2 SynthDef byte writer + binaries
- `src/generated/session-types.ts` — regenerated with NodeCatalogEntry/CatalogPortSpec/CatalogParamSpec/NodeCategory/SequencerPattern/OutputKind
- Application layer (graph_edit, session_store, agent_command, macro/performance/midi_learn, both runtime managers) + 8 integration test files — rewired to the new Node shape

## Decisions Made
- **find_catalog_entry error shape:** `UnknownCatalogEntry { node_type_id }` (single field) so the catalog can construct it from the id alone — keeps the catalog foundational (no node context needed).
- **CompiledNodeKind removed, not repurposed:** `CompiledNodeLaunch` carries `node_type_id` + flat config (bypassed/output_kind/channel_count); a category-level match remains for structural audio-bus wiring (sources have out_bus, effects have in+out+bypassed, etc.) — this is structural, not per-node-kind, so adding a new oscillator/filter/etc. adds no match arm.
- **Control-bus = one per mod-source output port** (shared by readers), not per route; audio-rate FM reuses the source audio out bus.
- **CV-bus silent-default convention** over a `bus >= 0` BinaryOp gate (avoids an error-prone hand-rolled BinaryOpUGen special-index; relies on SC's well-established unwritten-bus-reads-zero).
- **Removed obsolete v1 runtime_target tests** (legacy aliases / mismatched primitive) — those concepts don't exist in the catalog model (D-09); replaced by the unknown-node_type_id → UnknownCatalogEntry test.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Removed enums forced broader consumer rewire than the 5 listed spots**
- **Found during:** Task 1 (domain reshape)
- **Issue:** The plan listed 5 dispatch spots + tests, but removing the closed enums (`NodeType`/`AudioPrimitive`/…) also breaks `graph_edit.rs`, `session_store.rs` (build_default_session + generate_diff_summary), `agent_command.rs` (parse_add + validation), both runtime managers (exhaustive GraphEditCommand matches), and 8 integration test files. Leaving these un-updated fails `cargo check`.
- **Fix:** Mechanically rewired every consumer to the new `node_type_id` + flat-config Node shape; added `SetStepValue` arms to the runtime-manager + agent risk/validate matches; regenerated all node-construction in tests.
- **Files modified:** all files in the commit
- **Verification:** `cargo check --lib --tests` clean; 98 lib + 22 audio_runtime tests pass
- **Committed in:** `a293cef`

**2. [Rule 1 - Bug] Test session-construction bugs in two new audio_runtime tests**
- **Found during:** Task 2 verification
- **Issue:** The delay test replaced `nodes[0]` (the output node) instead of pushing the delay; the CV test's filter had no audio input, so `first_input_bus` failed.
- **Fix:** Pushed the delay node and kept the output; added a source→filter audio route so the effect has an input bus.
- **Files modified:** `src-tauri/tests/audio_runtime.rs`
- **Verification:** both tests pass
- **Committed in:** `55e1097`

**3. [Rule 3 - Blocking] ts-rs needed full catalog shape, not skipped fields**
- **Found during:** Task 1 (contract regen)
- **Issue:** Initial `#[ts(skip)]` on catalog fields produced a hollow TS type (no id/ports) — useless for the frontend palette/inspector.
- **Fix:** Removed the skips; verified ts-rs handles `&'static str` → `string` and `&'static [T]` → `Array<T>` natively.
- **Files modified:** `src-tauri/src/catalog/mod.rs`
- **Verification:** regenerated `session-types.ts` carries the full NodeCatalogEntry shape
- **Committed in:** `a293cef`

---

**Total deviations:** 3 auto-fixed (2 blocking, 1 bug)
**Impact on plan:** All deviations were necessary to deliver a compiling, test-green catalog foundation. No scope creep — every change is in service of the plan's must_haves and success criteria.

## Issues Encountered
- The plan's Task 1 ↔ Task 2 boundary is slightly optimistic: removing the closed enums in Task 1 breaks the compiler/synthdefs at `cargo check`, so the catalog-driven dispatch (nominally Task 2's "5 spots") had to land in Task 1. Resolved by splitting the two commits as catalog+domain+dispatch+v1-rejection (Task 1) vs control-bus-allocation+CV-validation (Task 2) — both compile and test green independently.
- `block v0.1.6` (a transitive Bevy dependency) emits a future-incompat warning under `cargo check`; this is pre-existing and unrelated to this plan.

## User Setup Required
None — no external service configuration required. (scsynth bundle fallback at `/Applications/SuperCollider.app/Contents/Resources/scsynth` is unchanged; the full "boots real scsynth per entry" conformance test is Plan 02's scope.)

## Next Phase Readiness
- The catalog foundation is complete and drives compiler/visual dispatch from one table. **Success criteria #1, #3, and #4 (foundation) are delivered**; the "boots real scsynth per entry" conformance test (#4 full) is Plan 02 Task 3.
- Plan 02 can now build on this: frontend catalog consumption (PrimitivePalette/NodeInspector), the app-driven sequencer transport-tick loop (`SequencerPattern` + `SetStepValue` are in canonical state), the `session-client.ts` Zod relaxation (Pitfall #5), and the real-scsynth conformance gate.
- No blockers for Plan 02. Frontend Zod schemas in `session-client.ts` still mirror the OLD closed-enum shape and will throw on every v2 catalog-derived `invokeSession` until Plan 02 relaxes them — flagged but out of this plan's scope per the plan boundary.

---
*Phase: 12-node-catalog-foundation*
*Completed: 2026-06-26*
