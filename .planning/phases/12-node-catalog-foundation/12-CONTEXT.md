# Phase 12: Node Catalog Foundation - Context

**Gathered:** 2026-06-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace v1's three hardcoded compiler allowlists with **one data-driven `NodeCatalogEntry` table** (single source of truth) that drives compiler dispatch, route validation, palette, inspector, and `ts-rs` schema export — and populate it with the full synthesis chain (oscillator, filter, envelope, LFO, utility, effect, step-sequencer), each mapped to SuperCollider UGens, with CV/modulation ports on every audio node. Verified by a conformance test that boots real `scsynth` for every entry.

**In scope (NODES-01..05):**
- Data-driven catalog as single source of truth (no hardcoded compiler allowlists).
- Oscillator/filter/envelope/LFO/utility(VCA/mixer/noise/quantizer) nodes mapped to SC UGens.
- Effect nodes (delay/reverb/distortion/chorus/flanger) mapped to SC.
- Step-sequencer node with per-step gate/CV + clock transport.
- Every audio node exposes CV/modulation inputs (audio-rate + control-rate ports); end-to-end modulation audibly works.
- Catalog drives compiler dispatch, route validation, palette, inspector, ts-rs export from one Rust table.
- Conformance test boots real `scsynth` for every catalog entry.
- The v1 `unreachable!()` panic path in `synthdef_resource()` becomes a real `Err`.

**Out of scope (other phases):**
- Graph UX rebuild (drag, edge connect/reconnect, multi-select, ownership badges on canvas) → Phase 13.
- Visual runtime / visuals behind the grid → Phase 14/15.
- Pro shell + live agent → Phase 16.

</domain>

<decisions>
## Implementation Decisions

### Node Granularity
- **D-01: Param-driven is the default granularity.** Each node family is ONE node whose parameters carry the variety (e.g. one Oscillator node; a `waveform`/`wave_shape` param selects sine/saw/square/triangle inside one SynthDef). This generalizes the existing v1 pattern (`source_oscillator` already uses `wave_shape` 0/1/2; `source_noise` uses `noise_color`). Fewer, richer catalog entries.
- **D-02: Split into separate nodes only where DSP fundamentally differs.** When one SynthDef cannot sensibly hold the variants (different UGen graphs), the family becomes separate catalog entries. The agent applies this per-family: e.g. a Filter node can stay param-driven (LPF/HPF/BPF/BRF share the `*LPF` UGen family), but delay vs reverb vs distortion are separate nodes (their DSP graphs are fundamentally different). The oscillator's param-driven choice is locked; the per-family split for FX/filter/utility is delegated to the agent with this DSP-fit rule.

### CV / Modulation Depth (NODES-05)
- **D-03: Modulation works end-to-end in Phase 12.** A real patch audibly modulates through SuperCollider (e.g. an LFO node's output sweeps a filter's cutoff). CV inputs map to SC control/audio args via buses THIS phase — not deferred to Phase 13. Both control-rate (LFO/envelope → param) and audio-rate modulation are in scope.
- **D-04: Per-parameter CV ports, declared in the catalog.** Each modulatable parameter gets its own CV-input port declared in the `NodeCatalogEntry` (e.g. a Filter node has `audio_out`, plus `cutoff_cv` and `resonance_cv` control-input ports). A performer draws an edge from a mod source's output to the target param's CV-in port. Most modular/explicit; matches NODES-05's "ports" language and the graph-native identity. (This implies the catalog schema must declare a CV-in port per modulatable param, and route validation must accept these.)
- **D-05: CV ports for continuous parameters only.** Continuous parameters (frequency, cutoff, resonance, level, feedback, delay_time, etc.) get a CV port. Discrete selectors (`wave_shape`, `noise_color`) and toggles (`bypass`) do NOT — modulating a selector index or an on/off is unusual and noisy.

### Step Sequencer Model (NODES-04)
- **D-06: App-driven sequencing (Rust owns the logic).** Sequencer logic lives in the Rust app: it tracks transport position, advances steps, and sends gate/CV values to SC as control messages per step. SuperCollider stays "dumb" — it only receives values. This is on-brand: canonical state + transport live in the app, not the audio engine (consistent with `TransportState` already being app-owned in `domain/session.rs`).
- **D-07: Mono output — one gate + one CV out.** The sequencer outputs one gate and one CV value per step (one voice triggered per step). Classic mono CV/gate sequencer; fewest ports; most legible. Matches NODES-04's singular "per-step gate/CV" phrasing.
- **D-08: Fixed 16 steps.** The pattern grid is a fixed 16-step array in canonical state. Predictable; simplest control-message shape; standard step-sequencer feel. (Clock division — e.g. 16 steps = 16th notes over one bar — is an implementation detail for the planner.)

### v1 Session Compatibility
- **D-09: Clean break allowed — schema bump, old files unsupported.** Phase 12 may bump the session schema version and break v1 session files. The catalog defines node identity from scratch; it does NOT need to accommodate the old closed-enum `audioPrimitive` shape. v1.0 shipped days ago; single-user local-first app; blast radius is tiny.
- **D-10: Friendly error on v1 file open — no load.** When a v1 session file is opened in the v2 app, show a clear, specific message ("This is a v1 session — unsupported in Scrysynth v2") and do NOT load it. No silent serde failure, no best-effort import. Fits the legibility principle.

### the agent's Discretion
- **FX/filter/utility per-family split:** Where exactly to draw param-driven vs separate-node lines across the FX and utility families (apply the DSP-fit rule from D-02).
- **SynthDef authoring toolchain for the ~12-16 new nodes:** The project has both `.scd` (sclang) source AND a deterministic Python SynthDef byte-writer (`generate_synthdefs.py`, a sclang-free path) for the existing 6 defs. The researcher/planner chooses how to author the new SynthDefs (extend the Python writer vs hand-write `.scd` vs hybrid) — not a user-facing preference.
- **Catalog storage representation:** STATE.md locks "one Rust-owned `NodeCatalogEntry` table" (compiled-in, type-safe) — the exact representation (const table / `phf` map / builder module) is the planner's call.
- **Control-bus allocation strategy** for modulation wiring (D-03/D-04): how SC control/audio buses are allocated and mapped to CV ports — implementation detail for the researcher/planner.
- **Sequencer clock division / retrigger behavior** (D-06/D-08): sensible defaults for the planner.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project / milestone decisions (locked architecture)
- `.planning/STATE.md` §Decisions — locks **[v2.0 Catalog-as-data]**: refactor v1's three hardcoded allowlists (`synthdef_resource`, `normalize_parameter_name`, `validate_runtime_target`) into one Rust-owned `NodeCatalogEntry` table. Also locks **[v2.0 Layout command]** (Phase 13, not here) and **[v2.0 Agent safety]** (Phase 16, not here).
- `.planning/PROJECT.md` §Constraints + §Key Decisions — canonical-session-truth-in-the-app, SuperCollider-as-execution-not-model, favor stable explainable primitives.
- `.planning/REQUIREMENTS.md` §Curated Modular Node Library — NODES-01..05 verbatim (the requirements this phase satisfies).
- `.planning/ROADMAP.md` §Phase 12 — goal, dependencies ("v1.0 shipped foundation; no v2 phase dependencies"), and the 4 success criteria.

### Code to refactor (the v1 hardcoded allowlists — the heart of this phase)
- `src-tauri/src/audio/synthdefs.rs` — all THREE allowlists live here:
  - `synthdef_resource()` (:429) — maps synthdef name → `.scsyndef` path; contains the **`unreachable!("unknown v1 synthdef name")` panic at :455** that success criteria #3 requires become a real `Err`.
  - `normalize_parameter_name()` (:407) — maps parameter aliases to canonical SC synth args.
  - `validate_runtime_target()` (:284) — validates runtime-target strings per node kind.
  - `plan_sc_resources()` (:96) — the big `match &node.node_kind` dispatch that picks the synthdef name per `CompiledNodeKind` (this becomes catalog-driven).
- `src-tauri/src/audio/compiler.rs` — `CompiledNodeKind` enum (:51) and `compile_session_to_topology()` (:93); the topology compiler that feeds `plan_sc_resources`. `node_sort_key()` (:418) also dispatches on `NodeType`.
- `src-tauri/src/domain/session.rs` — the closed enums to supersede: `NodeType` (:195), `AudioSourceType` (:221), `AudioEffectType` (:236), `AudioPrimitive` (:204); the `Node` struct (:179), `Port` (:272), `SignalType` (:288, currently only `Audio`/`Control`), `ParameterValue` (:295); and `write_generated_typescript_contract()` (:806, the ts-rs export list the catalog must join).
- `src-tauri/src/visual/compiler.rs` (:57) — also dispatches on `NodeType` (Source→sphere, Effect→box, etc.). A 5th hardcoded spot; the catalog should drive this too (or at minimum not break it).

### SynthDef generation (reusable assets for new nodes)
- `src-tauri/resources/synthdefs/v1/scrysynth_v1_synthdefs.scd` — canonical sclang source for the 6 v1 SynthDefs (oscillator/noise/lowpass/delay/mixer/output). Template for new `.scd` defs.
- `src-tauri/resources/synthdefs/v1/generate_synthdefs.py` — deterministic Python SynthDef-v2 byte-writer (sclang-free path) for the same 6 defs. Key reusable asset: new SynthDefs can be authored here without a sclang dependency. The `SynthDefBuilder`/`SynthDefSpec` model + `synthdef_bytes()` serializer are the building blocks.
- `src-tauri/resources/synthdefs/v1/*.scsyndef` — the 6 checked-in compiled SynthDef binaries (loaded by `supercollider.rs:200` via `/d_recv`).

### Frontend (consume the catalog — palette/inspector)
- `src/components/audio/PrimitivePalette.tsx` — currently 3 hardcoded buttons (Add Source/Effect/Mixer) + a hardcoded `buildPrimitiveNode()` factory. Must become catalog-driven. (Note: v1 sets `runtimeTarget: audio/source/${id}` using the node id — a quirk; catalog-driven targets replace this.)
- `src/components/session/NodeInspector.tsx` — parameter editing; must consume catalog parameter definitions (incl. CV ports).
- `src/generated/session-types.ts` — the ts-rs output the catalog export joins.

### Tests (the conformance gate + existing synthdef tests)
- `src-tauri/tests/audio_runtime.rs` §synthdefs (:201) — existing tests including `checked_in_v1_synthdef_resources_are_present_and_named()` (:433) and the plan-mapping tests. The new conformance test ("boots real `scsynth` for every entry") extends this pattern.

No external specs/ADRs were referenced by the user during discussion.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Python SynthDef byte-writer** (`generate_synthdefs.py`): a sclang-free SynthDef-v2 generator with a `SynthDefBuilder`/`UGenSpec`/`SynthDefSpec` model and a byte-accurate serializer. Adding new node SynthDefs can extend this without making `sclang` a hard dev dependency. The `.scd` source is the human-readable mirror.
- **Topology compiler + SC resource planner**: `compile_session_to_topology()` (compiler.rs:93) already does bus allocation, topo-sort, and launch ordering; `plan_sc_resources()` (synthdefs.rs:96) already maps nodes→synthdefs/args/controls. The refactor replaces the hardcoded dispatch inside these, not the pipeline itself.
- **ts-rs export pipeline**: `write_generated_typescript_contract()` (session.rs:806) already exports every domain type to `src/generated/session-types.ts`; adding `NodeCatalogEntry` is a one-line addition to the declarations list.
- **`ScResourcePlanError`** (synthdefs.rs:62): typed error enum already used for validation failures — the new `Err` replacing the `unreachable!()` panic fits this pattern (e.g. a `UnknownCatalogEntry` variant).

### Established Patterns
- **Canonical truth in Rust, engines are consumers**: the app owns the graph; SC/visual adapters consume compiled projections. The catalog continues this — it is Rust-owned, and the SC adapter consumes catalog-derived plans.
- **Typed-command gate**: all mutations flow through `GraphEditCommand`/`TypedCommand` (session.rs:473). New catalog-derived node creation/pattern edits must go through typed commands (relevant to the sequencer pattern data, D-06/D-08).
- **Closed-enum → catalog-driven is the core refactor**: `NodeType`/`AudioSourceType`/`AudioEffectType`/`CompiledNodeKind` are closed enums threaded through domain/compiler/synthdefs/visual. The catalog must supersede them as the single source; D-09 (clean break) means the enums can be reshaped freely without v1 migration.

### Integration Points
- **Compiler dispatch**: `plan_sc_resources()`'s `match &node.node_kind` (synthdefs.rs:116) becomes a catalog lookup by node-type id.
- **Route validation**: `validate_routes()` (compiler.rs:194) + port matching gains CV-port type checking (D-04/D-05).
- **Bus allocation**: `plan_buses()` (synthdefs.rs:246) currently allocates only audio buses; D-03 (end-to-end modulation) requires control-bus allocation for CV wiring.
- **Palette/inspector**: `PrimitivePalette.buildPrimitiveNode()` and `NodeInspector` read from the catalog (via generated TS types) instead of hardcoded factories.
- **Visual compiler**: `visual/compiler.rs:57` NodeType→shape dispatch should become catalog-driven (or at least not break).

</code_context>

<specifics>
## Specific Ideas

- The user's mental model is consistently **modular and legible**: param-driven families (D-01), explicit per-param CV ports (D-04), mono gate+CV sequencer (D-07), friendly errors (D-10). Favor fewer-richer-explicit over many-cluttered whenever there's a choice.
- The **oscillator stays one node with a waveform param** (locked, D-01) — do not split it into sine/saw/square/triangle nodes even though that's the Bespoke/VCV convention.
- v1 SynthDefs (osc/noise/lowpass/delay/mixer/output) fold into the new catalog as entries largely unchanged (they're a subset of the v2 catalog); the refactor is about the *dispatch mechanism*, not redesigning those 6 defs.

</specifics>

<deferred>
## Deferred Ideas

None raised — discussion stayed within phase scope. (Clock-division detail, retrigger/legato behavior, and SynthDef-authoring toolchain were noted as planner/researcher discretion in the agent's Discretion section, not deferred scope-creep.)

### Reviewed Todos (not folded)
None — `todo.match-phase` returned 0 matches for Phase 12.

</deferred>

---

*Phase: 12-Node Catalog Foundation*
*Context gathered: 2026-06-26*
