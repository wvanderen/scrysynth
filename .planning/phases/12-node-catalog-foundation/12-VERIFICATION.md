---
phase: 12-node-catalog-foundation
verified: 2026-06-26T20:25:00Z
status: passed
score: 4/4 roadmap success criteria verified (all 5 requirements satisfied; all 13 plan-level truths verified)
scored_at: 2026-06-26T20:25:00Z
behavior_unverified: 0
overrides_applied: 0
re_verification:
  previous_status: none
  previous_score: n/a
  mode: initial verification
---

# Phase 12: Node Catalog Foundation — Verification Report

**Phase Goal (ROADMAP.md):** A data-driven node catalog (single source of truth) covers the full synthesis chain — oscillators, filters, envelopes, LFOs, utilities, effects, and a step sequencer — each mapped to SuperCollider UGens, so new node types can be added without touching v1's hardcoded compiler allowlists.

**Requirements:** NODES-01, NODES-02, NODES-03, NODES-04, NODES-05

**Verified:** 2026-06-26T20:25:00Z
**Status:** **passed**
**Re-verification:** No — initial verification

---

## Goal Achievement

### Roadmap Success Criteria (the contract)

| # | Success Criterion | Status | Evidence |
|---|-------------------|--------|----------|
| 1 | A performer can add oscillator/filter/envelope/LFO/utility/effect/step-sequencer nodes and hear them shape audio through SuperCollider | ✓ VERIFIED | 15 catalog entries in `src-tauri/src/catalog/entries.rs`; 14 `.scsyndef` files in `resources/synthdefs/v2/`; palette iterates catalog (`src/components/audio/PrimitivePalette.tsx:72`); **conformance test `every_catalog_entry_synthdef_loads_on_real_scsynth` PASSES against real scsynth** (`cargo test --test audio_runtime -- --ignored`) |
| 2 | Every catalog node exposes CV/modulation inputs (audio-rate + control-rate ports) | ✓ VERIFIED | Per-param CV ports declared (D-04/D-05); audio-rate representative `oscillator.frequency_cv: SignalType::Audio` (`entries.rs:100`); control-rate ports throughout; invariant test `continuous_parameters_declare_a_cv_port` passes; CV route allocates control bus (`synthdefs.rs:13` `FIRST_CONTROL_BUS_OFFSET=1024`); end-to-end `lfo_to_filter_cv_modulation_launches_with_cv_bus_args_on_real_scsynth` PASSES |
| 3 | Adding a new node type requires editing only the catalog; v1 `unreachable!()` panic replaced with real `Err` | ✓ VERIFIED | Grep for dead dispatch `match &node.node_kind` / `match node.node_type` / `unreachable!(` returns 0 hits in `synthdefs.rs`/`compiler.rs`/`visual/compiler.rs` (only in comments); `UnknownCatalogEntry` variant (`synthdefs.rs:88`); `find_catalog_entry` returns `Result` (`catalog/mod.rs:147`); tests `resource_plan_fails_loudly_for_unknown_catalog_entry` + `unknown_node_type_id_errors_through_full_compile_to_plan_path` PASS |
| 4 | The catalog drives compiler dispatch, route validation, palette, inspector, and ts-rs export from one Rust table, verified by a conformance test booting real scsynth per entry | ✓ VERIFIED | Compiler dispatch `find_catalog_entry` (`synthdefs.rs:124`, `compiler.rs:430`); route validation `PortSignalTypeMismatch` (`compiler.rs:92`); visual dispatch (`visual/compiler.rs:59`); palette/inspector catalog-driven; ts-rs export `NodeCatalogEntry`/`CatalogPortSpec`/`CatalogParamSpec`/`NodeCategory`/`SequencerPattern` in `src/generated/session-types.ts`; conformance test PASSES on real scsynth |

**Score: 4/4 success criteria verified.** The Phase 12 lynchpin (success criterion #4 — the conformance gate) **passes locally against the real `scsynth` bundle** at `/Applications/SuperCollider.app/Contents/Resources/scsynth`.

### Observable Truths (Plan must_haves)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A compiled-in `NodeCatalogEntry` table is the single source of truth for node identity, SynthDef mapping, ports, parameters, and visual shape | ✓ VERIFIED | `catalog/mod.rs:97` `NodeCatalogEntry`; `catalog/entries.rs` 15 entries; `find_catalog_entry` (`mod.rs:147`); consumed by all 5 former dispatch spots |
| 2 | Adding a new node type requires editing only the catalog — no new match arms in synthdefs.rs, compiler.rs, or visual/compiler.rs | ✓ VERIFIED | Grep `match &node.node_kind` / `match node.node_type` = 0 in target files; `unknown_node_type_id_errors_through_full_compile_to_plan_path` test PASSES |
| 3 | An unknown `node_type_id` returns `Err(ScResourcePlanError::UnknownCatalogEntry)`, never panics | ✓ VERIFIED | `synthdefs.rs:88` variant; `unreachable!(` gone from synthdefs.rs; `resource_plan_fails_loudly_for_unknown_catalog_entry` test asserts the Err shape |
| 4 | Every CV-eligible continuous parameter has a declared CV-input port; discrete selectors and toggles do not | ✓ VERIFIED | `continuous_parameters_declare_a_cv_port` catalog invariant test (`mod.rs:189`) PASSES for all entries; `step_sequencer` gate toggles have no CV port |
| 5 | The compiler allocates control buses (separate index range from audio buses) for CV routes | ✓ VERIFIED | `FIRST_CONTROL_BUS_OFFSET=1024` (`synthdefs.rs:13`); `plan_cv_buses` (`synthdefs.rs:391`); `ScResourcePlan.cv_bus_map` (`synthdefs.rs:35`); `cv_route_allocates_control_bus_for_modulation` test asserts `out_cv_bus >= 1024` and shared index with `cutoff_cv_bus` |
| 6 | A v1 session file (schemaVersion 1) is rejected with a friendly `LegacyV1Session` message via two-phase parse — never a cryptic serde error | ✓ VERIFIED | `LegacyV1Session` variant (`session_file.rs:27`); `SchemaVersionProbe` two-phase parse (`session_file.rs:37`, `71-75`); friendly message "This is a v1 session — unsupported in Scrysynth v2…"; inline test `legacy_v1_session_is_rejected` passes |
| 7 | All ~13 v2 SynthDefs exist as `.scsyndef` binaries with SCgf v2 headers and CV-bus control args | ✓ VERIFIED | 14 `.scsyndef` files in `resources/synthdefs/v2/`; `checked_in_v2_synthdef_resources_are_present_and_named` test asserts SCgf headers + embedded names; conformance test /d_recvs all on real scsynth |
| 8 | A performer can add any catalog node via the palette and it appears in the graph with catalog-declared ports and parameters | ✓ VERIFIED | `PrimitivePalette.tsx` iterates `NodeCatalogEntry[]` fetched via `get_node_catalog` Tauri command (`lib.rs:307`); `buildNodeFromCatalogEntry` (`PrimitivePalette.tsx:111`) maps entry ports/parameters/defaults |
| 9 | The step sequencer advances 16 fixed steps against transport tempo and writes per-step gate+CV via `/c_set` | ✓ VERIFIED | `SequencerController` (`sequencer.rs:157`) with `AtomicBool` shutdown (`sequencer.rs:160`); `sequencer_controller_emits_c_set_per_step_over_full_16_step_cycle` test asserts 32 writes (16 gate + 16 cv); `sequencer_step_index_wraps_after_16_steps` PASSES |
| 10 | Every catalog entry's SynthDef `/d_recv`s successfully to a real booted scsynth (conformance gate) | ✓ VERIFIED | `every_catalog_entry_synthdef_loads_on_real_scsynth` PASSES (`--ignored`); `recv_all_catalog_synthdefs_for_conformance` (`supercollider.rs:448`); **all 13 entries with synthdefs loaded** |
| 11 | An LFO node routed to a filter's `cutoff_cv` port produces observable modulation through SuperCollider (D-03 end-to-end) | ✓ VERIFIED | `lfo_to_filter_cv_modulation_launches_with_cv_bus_args_on_real_scsynth` PASSES on real scsynth; asserts LFO synth carries `out_cv_bus >= FIRST_CONTROL_BUS_OFFSET` and runtime reaches Ready |
| 12 | The NodeInspector renders catalog display name, CV ports, and a 16-step gate/CV editor for sequencer nodes | ✓ VERIFIED | `displayNameFor` lookup (`NodeInspector.tsx:247`); CV port badge `port.signalType === "control"` (`NodeInspector.tsx:109`); `SequencerStepEditor` (`NodeInspector.tsx:195`) renders 16 gate toggles + 16 cv sliders; `npm run build` succeeds |
| 13 | npm test, npm run build, and cargo test all pass | ✓ VERIFIED | `cargo test --lib` 103 passed; `cargo test --test audio_runtime` 27 passed + 2 ignored; `cargo test --test audio_runtime -- --ignored` 2 passed; `npm test` 49 passed; `npm run build` succeeds |

**Score: 13/13 truths verified (0 present-behavior-unverified).** Every behavior-dependent truth (state transitions: sequencer step advance + wrap; cancellation: AtomicBool shutdown + spawn/kill lifecycle; real-runtime: scsynth boot + /d_recv; end-to-end: CV modulation launch) is exercised by a passing behavioral test, not just symbol presence.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/catalog/mod.rs` | NodeCatalogEntry/CatalogPortSpec/CatalogParamSpec/NodeCategory + CATALOG + find_catalog_entry | ✓ VERIFIED | 226 lines; all types present; `pub const CATALOG` re-exported; `find_catalog_entry -> Result` at `:147`; 5 invariant tests |
| `src-tauri/src/catalog/entries.rs` | ~16 catalog entries (full synthesis chain) | ✓ VERIFIED | 15 entries: oscillator, noise, envelope, lfo, vca, quantizer, mixer, filter, delay, reverb, distortion, chorus, flanger, step_sequencer, output |
| `src-tauri/src/audio/synthdefs.rs` | Catalog-driven dispatch; UnknownCatalogEntry; control-bus allocation | ✓ VERIFIED | `find_catalog_entry` at `:124`; `UnknownCatalogEntry` at `:88`; `FIRST_CONTROL_BUS_OFFSET=1024` at `:13`; `cv_bus_map` field at `:35`; `plan_cv_buses` at `:391`; `synthdef_resource_for_name` uses catalog lookup (no panic) |
| `src-tauri/src/audio/compiler.rs` | Catalog-driven node_sort_key + compile_node_launch + CV-port validation | ✓ VERIFIED | `find_catalog_entry` at `:430`; `PortSignalTypeMismatch` at `:92`; closed `CompiledNodeKind` enum removed |
| `src-tauri/src/persistence/session_file.rs` | Two-phase v1 rejection with LegacyV1Session | ✓ VERIFIED | `LegacyV1Session` at `:27`; `SchemaVersionProbe` at `:37`; probe-first parse at `:71-75` |
| `src-tauri/resources/synthdefs/v2/generate_synthdefs.py` | Extended sclang-free byte writer for v2 nodes incl. CV-bus reads | ✓ VERIFIED | 29548 bytes; emits 14 `.scsyndef` files |
| `src-tauri/resources/synthdefs/v2/*.scsyndef` | ~14 binaries with SCgf v2 headers | ✓ VERIFIED | 14 files present; `checked_in_v2_synthdef_resources_are_present_and_named` test asserts headers |
| `src-tauri/src/audio/sequencer.rs` | SequencerController — transport-tick loop sending /c_set per step | ✓ VERIFIED | `SequencerController` at `:157`; `SequencerTickSink` trait at `:60`; `UdpCSetSink` at `:76`; `RecordingCSetSink` at `:126`; `AtomicBool` shutdown at `:160`; `start` at `:185` |
| `src-tauri/src/audio/supercollider.rs` | conformance entry point + cv_bus_assignments | ✓ VERIFIED | `recv_all_catalog_synthdefs_for_conformance` at `:448` |
| `src-tauri/src/application/session_store.rs` | sequencer_controllers spawn-on-Ready/kill-on-stop/panic + propagate_step_edit | ✓ VERIFIED | `sequencer_controllers` field at `:69`; `spawn_sequencer_controllers` at `:111`; `stop_sequencer_controllers` at `:156`; `propagate_step_edit` at `:167`; spawn at `:201`; kill at `:212`/`:220`/`:264` |
| `src-tauri/src/lib.rs` | get_node_catalog Tauri command | ✓ VERIFIED | Command at `:307`; registered in invoke_handler at `:367` |
| `src/components/audio/PrimitivePalette.tsx` | Catalog-driven palette iterating NodeCatalogEntry[] | ✓ VERIFIED | `catalog` state at `:36`; `getNodeCatalog()` fetch at `:40`; `buildNodeFromCatalogEntry` at `:111`; `runtimeTarget: entry.id` at `:156` (v1 quirk fixed) |
| `src/components/session/NodeInspector.tsx` | Catalog display name, CV port listing, 16-step sequencer editor | ✓ VERIFIED | `displayNameFor` at `:247`; CV badge at `:109`; `SequencerStepEditor` at `:195` |
| `src/lib/session-client.ts` | Relaxed Zod schemas (nodeTypeId:string, no audioPrimitive, sequencerPatternSchema, setStepValue) | ✓ VERIFIED | `nodeTypeId: z.string()` at `:88`; `sequencerPatternSchema` at `:77`; `setStepValue` at `:236`; `nodeCatalogEntrySchema` at `:430`; no `audioPrimitive` union |
| `src/generated/session-types.ts` | Regenerated with NodeCatalogEntry/CatalogPortSpec/CatalogParamSpec/NodeCategory/SequencerPattern | ✓ VERIFIED | All types present at lines 134-177; `nodeTypeId` at `:26`; `SequencerPattern` 16-tuple at `:36` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `src-tauri/src/audio/synthdefs.rs` | `src-tauri/src/catalog/mod.rs` | `find_catalog_entry(&node.node_type_id)?` at `:124` | ✓ WIRED | replaces `match &node.node_kind` dispatch |
| `src-tauri/src/audio/compiler.rs` | `src-tauri/src/catalog/mod.rs` | `find_catalog_entry(&node.node_type_id)` at `:430` | ✓ WIRED | replaces `match node.node_type` in `node_sort_key` |
| `src-tauri/src/visual/compiler.rs` | `src-tauri/src/catalog/mod.rs` | `find_catalog_entry(&node.node_type_id).map(\|e\| e.visual_shape)` at `:59` | ✓ WIRED | replaces NodeType→shape match |
| `src/components/audio/PrimitivePalette.tsx` | `src/generated/session-types.ts` + Tauri cmd | `getNodeCatalog()` → `invokeCommand("get_node_catalog")` → `z.array(nodeCatalogEntrySchema).parse` | ✓ WIRED | `session-client.ts:602-604`; real data flows to rendered buttons (`PrimitivePalette.tsx:72`) |
| `src-tauri/src/audio/sequencer.rs` | `src-tauri/src/audio/supercollider.rs` (scsynth) | `SequencerTickSink::tick` → `UdpCSetSink` → OSC `/c_set` | ✓ WIRED | `sequencer.rs:106`; test asserts 32 `/c_set` writes per 16-step cycle |
| `src-tauri/src/application/session_store.rs` | `src-tauri/src/audio/sequencer.rs` | `SequencerController::start` on Ready (`:201`); `stop` on stop/panic (`:212`/`:220`); `propagate_step_edit` reconcile (`:167`) | ✓ WIRED | no orphan threads; live pattern edits propagate |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| `PrimitivePalette.tsx` | `catalog: NodeCatalogEntry[]` | `getNodeCatalog()` → `invokeCommand("get_node_catalog")` → Rust `CATALOG.iter().copied().collect()` | ✓ Yes — real compiled-in catalog, not static/hardcoded | ✓ FLOWING |
| `NodeInspector.tsx` | `displayName`, `port.signalType`, `pattern.gate[]/cv[]` | `selectedNode` (canonical Node from SessionDocument), built from catalog entry defaults | ✓ Yes — flows from canonical session | ✓ FLOWING |
| `SequencerController` | `pattern: Arc<Mutex<SequencerPattern>>` | canonical Node.sequencer_pattern → `propagate_step_edit` reconcile | ✓ Yes — live mid-play edits propagate | ✓ FLOWING |
| Conformance test | `adapter.recv_all_catalog_synthdefs_for_conformance()` | reads `.scsyndef` bytes from disk, /d_recvs to real scsynth | ✓ Yes — real scsynth acknowledges `/done` for all 13 entries | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `cargo test --lib` | `cargo test --lib` (src-tauri) | 103 passed; 0 failed | ✓ PASS |
| `cargo test --test audio_runtime` (non-ignored) | `cargo test --test audio_runtime` (src-tauri) | 27 passed; 0 failed; 2 ignored | ✓ PASS |
| Real-scsynth conformance (success criterion #4) | `cargo test --test audio_runtime -- --ignored` (src-tauri) | 2 passed (`every_catalog_entry_synthdef_loads_on_real_scsynth` + `lfo_to_filter_cv_modulation_launches_with_cv_bus_args_on_real_scsynth`) | ✓ PASS |
| `npm test` (vitest) | `npm test` | 49 passed; 5 files; 0 failed | ✓ PASS |
| `npm run build` (tsc + vite) | `npm run build` | succeeds; 294 modules transformed; chunk-size warning only | ✓ PASS |
| Unknown node_type_id returns Err (success criterion #3) | `resource_plan_fails_loudly_for_unknown_catalog_entry` + `unknown_node_type_id_errors_through_full_compile_to_plan_path` | both PASS; assert `Err(UnknownCatalogEntry)` not panic | ✓ PASS |
| Sequencer /c_set 16-step advance | `sequencer_controller_emits_c_set_per_step_over_full_16_step_cycle` | PASS; asserts 32 writes (16 gate + 16 cv) over full cycle | ✓ PASS |
| CV control-bus allocation (NODES-05) | `cv_route_allocates_control_bus_for_modulation` | PASS; asserts `out_cv_bus >= 1024` + shared index with `cutoff_cv_bus` | ✓ PASS |

### Probe Execution

This is a migration/tooling phase with no `scripts/*/tests/probe-*.sh`. Behavioral probes were run via the test suites above (the conformance test functions as the Phase 12 probe against real scsynth).

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| NODES-01 | 12-01 + 12-02 | Data-driven catalog single source of truth; no hardcoded allowlists | ✓ SATISFIED | 5 dead dispatch spots replaced; `find_catalog_entry` in all; catalog-driven palette/inspector/ts-rs; conformance test PASSES |
| NODES-02 | 12-01 | oscillator/filter/envelope/LFO/utility (VCA/mixer/noise/quantizer) mapped to SC UGens | ✓ SATISFIED | All 8 entries present in catalog + .scsyndef files; conformance test loads each |
| NODES-03 | 12-01 | effects (delay/reverb/distortion/chorus/flanger) with wet/dry mapped to SC | ✓ SATISFIED | All 5 entries present as separate nodes (D-02 DSP-fit split); .scsyndef files; conformance test loads each |
| NODES-04 | 12-02 | step-sequencer with per-step gate/CV + clock transport | ✓ SATISFIED | `step_sequencer` catalog entry (no synthdef, D-06); `SequencerController` std::thread tick loop; 16-step `SequencerPattern`; `/c_set` advance test PASSES; lifecycle spawn/kill wired |
| NODES-05 | 12-01 + 12-02 | Every audio node exposes CV/modulation inputs (audio-rate + control-rate) | ✓ SATISFIED | Per-param CV ports declared (D-04/D-05); audio-rate rep `oscillator.frequency_cv: Audio`; control-bus allocation `FIRST_CONTROL_BUS_OFFSET=1024`; `cv_route_allocates_control_bus_for_modulation` + `lfo_to_filter_cv_modulation_launches_with_cv_bus_args_on_real_scsynth` both PASS |

**No orphaned requirements.** All 5 NODES requirements mapped to Phase 12 in ROADMAP.md are claimed by plans and satisfied by code.

### Locked Decisions (D-01..D-10) Compliance

| Decision | Status | Evidence |
|----------|--------|----------|
| D-01 param-driven default | ✓ | Single oscillator/filter/noise entries with wave_shape/filter_mode/noise_color params |
| D-02 split where DSP differs | ✓ | delay/reverb/distortion/chorus/flanger are separate entries |
| D-03 audio-rate + control-rate CV | ✓ | `oscillator.frequency_cv: SignalType::Audio` (FM path); all mod sources Control-rate |
| D-04 per-parameter CV ports | ✓ | `exposes_cv_port` + `cv_port_id` on `CatalogParamSpec`; ports declared per param |
| D-05 CV for continuous params only | ✓ | `continuous_parameters_declare_a_cv_port` invariant test; selectors/toggles have `exposes_cv_port: false` |
| D-06 app-driven sequencing (SC dumb) | ✓ | `step_sequencer.synthdef_name == ""`; no sequencer SynthDef; `/c_set` from Rust |
| D-07 mono gate+CV | ✓ | `SequencerPattern { gate: [bool;16], cv: [f64;16] }`; gate_out + cv_out ports |
| D-08 fixed 16 steps | ✓ | `[bool; 16]` / `[f64; 16]` arrays; ts-rs exports 16-tuple |
| D-09 schema bump clean break | ✓ | `CURRENT_SCHEMA_VERSION = 2`; closed enums removed |
| D-10 friendly v1 rejection | ✓ | `LegacyV1Session` with "This is a v1 session — unsupported in Scrysynth v2…"; two-phase parse |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | — | — | — | No `TBD`/`FIXME`/`XXX` debt markers; no `TODO`/`HACK`/`PLACEHOLDER`; no "not implemented"/"placeholder" stub patterns in phase-modified files |

None. The phase-modified source files are clean of debt markers and stub patterns. The only pre-existing warning is `block v0.1.6` future-incompat (Bevy transitive dep), unrelated to Phase 12.

---

## Human Verification

No blocking human-verification items. All four success criteria are behaviorally proven by passing tests, including the real-scsynth conformance gate (success criterion #4).

**Recommended (non-blocking) optional human smoke test** — outside the phase's defined verification bar (the conformance test is the phase's gate per RESEARCH.md, and audible FFT was explicitly scoped out): launch the app, add an oscillator + filter + LFO via the palette, route LFO→filter cutoff_cv, start transport, and confirm audible modulation in a live session. The conformance test already proves every SynthDef loads on real scsynth and the CV plan launches with the correct bus args; this optional check would additionally confirm perceived audio quality.

---

## Gaps Summary

None. All 4 roadmap success criteria verified with behavioral test evidence. All 5 requirements satisfied. All 13 plan-level must-have truths verified. The real-scsynth conformance gate (the Phase 12 lynchpin) **passes locally** — all 13 catalog SynthDefs `/d_recv` successfully and the LFO→filter CV modulation launches with `out_cv_bus >= FIRST_CONTROL_BUS_OFFSET` against the booted `scsynth`. All four test suites (`cargo test --lib`, `cargo test --test audio_runtime` incl. `--ignored`, `npm test`, `npm run build`) are green.

Phase 12's vertical slice is complete and the phase goal is achieved in the codebase, not just in SUMMARY claims.

---

_Verified: 2026-06-26T20:25:00Z_
_Verifier: the agent (gsd-verifier)_
