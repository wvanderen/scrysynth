# Phase 12: Node Catalog Foundation - Pattern Map

**Mapped:** 2026-06-26
**Files analyzed:** 16 (6 new file/module groups, 14 in-place refactors)
**Analogs found:** 14 exact / role-match, 2 partial (no v1 equivalent)

> This phase is overwhelmingly an **in-place content refactor** of v1's three hardcoded allowlists + two enum-dispatch spots into one `NodeCatalogEntry` const table. Most "files to modify" ARE their own analog — the planner copies the surrounding convention (imports, error enum, test harness, ts-rs export list) and replaces the dispatch *contents*. The two genuinely new capabilities (CV control-bus allocation, app-driven step sequencer) extend existing functions rather than introducing new architecture.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src-tauri/src/catalog/mod.rs` (NEW) | config / model | lookup (read) | `src-tauri/src/audio/synthdefs.rs:10-15` const block + `:429 synthdef_resource()` | role-match (the thing being replaced) |
| `src-tauri/src/catalog/entries.rs` (NEW) | config | lookup (read) | `generate_synthdefs.py:388 definitions()` + `synthdefs.rs:10-15` | role-match |
| `src-tauri/src/audio/sequencer.rs` (NEW) | controller / driver | event-driven (periodic tick) | `supercollider.rs:286 send_live_parameter` (OSC send) + `:7 std::thread` | partial (no v1 periodic driver) |
| `src-tauri/resources/synthdefs/v2/generate_synthdefs.py` (NEW) | build tooling | file I/O (byte writer) | `src-tauri/resources/synthdefs/v1/generate_synthdefs.py` | exact (extend in place / copy to v2/) |
| `src-tauri/resources/synthdefs/v2/*.scsyndef` (NEW) | resource | file I/O | `src-tauri/resources/synthdefs/v1/*.scsyndef` | exact |
| `src-tauri/src/audio/synthdefs.rs` (MODIFY) | service / adapter | request-response (plan) | itself — refactor the 3 allowlists in place | exact |
| `src-tauri/src/audio/compiler.rs` (MODIFY) | service | transform (topology compile) | itself — `CompiledNodeKind` + `node_sort_key` | exact |
| `src-tauri/src/domain/session.rs` (MODIFY) | model | CRUD | itself — closed enums + `write_generated_typescript_contract` | exact |
| `src-tauri/src/visual/compiler.rs` (MODIFY) | service | transform | itself — `:56` NodeType→shape match | exact |
| `src-tauri/src/audio/mod.rs` (MODIFY) | config | — | itself + `domain/mod.rs` | exact |
| `src-tauri/src/application/session_store.rs` (MODIFY) | service / store | event-driven | itself — `:220 poll_hardware_events`, audio start/stop | exact |
| `src-tauri/src/persistence/session_file.rs` (MODIFY) | service | file I/O | itself — `:50 open_session_from_path` | exact |
| `src-tauri/src/audio/supercollider.rs` (MODIFY) | adapter | request-response (OSC) | itself — `:188 apply_resource_plan`, `/d_recv`, `/c_set` hook | exact |
| `src/components/audio/PrimitivePalette.tsx` (MODIFY) | component | request-response | itself — `:62 buildPrimitiveNode` | exact |
| `src/components/session/NodeInspector.tsx` (MODIFY) | component | request-response | itself — `:100` parameter slider + `:90` ports | exact |
| `src/lib/session-client.ts` (MODIFY) | service / client | request-response (validate) | itself — `:70 nodeSchema` Zod block | exact |
| `src-tauri/tests/audio_runtime.rs` (MODIFY) | test | — | itself — `:433 checked_in_v1_synthdef...` + `:201 synthdefs` mod | exact |
| `src/lib/session-client.test.ts` (MODIFY) | test | — | itself — browser-preview round-trip | exact |
| `src/generated/session-types.ts` (MODIFY, regenerated) | generated types | — | itself — ts-rs output | exact |

## Pattern Assignments

### `src-tauri/src/catalog/mod.rs` + `catalog/entries.rs` (NEW — config, lookup)

**Analog:** `src-tauri/src/audio/synthdefs.rs` const block + the three allowlist functions it replaces. This is the **single source of truth** the rest of the phase consumes.

**Const-table convention** (synthdefs.rs:10-15) — copy this `pub const` style:
```rust
pub const SOURCE_OSCILLATOR_SYNTHDEF: &str = "scrysynth_v1_source_oscillator";
pub const SOURCE_NOISE_SYNTHDEF: &str = "scrysynth_v1_source_noise";
// ... 6 total
```

**The function signature the catalog lookup replaces** (synthdefs.rs:429-457 — the panic at :455 is success criterion #3):
```rust
fn synthdef_resource(name: &str) -> SynthDefResource {
    match name {
        SOURCE_OSCILLATOR_SYNTHDEF => SynthDefResource {
            name: SOURCE_OSCILLATOR_SYNTHDEF,
            relative_path: "resources/synthdefs/v1/scrysynth_v1_source_oscillator.scsyndef",
        },
        // ... 5 more arms ...
        _ => unreachable!("unknown v1 synthdef name"),   // ← becomes Err (success criterion #3)
    }
}
```
**Catalog replacement** (per RESEARCH.md Pattern 1 + Catalog Data Model):
```rust
pub fn find_catalog_entry(id: &str) -> Result<&'static NodeCatalogEntry, ScResourcePlanError> {
    CATALOG.iter().find(|e| e.id == id).ok_or_else(|| ScResourcePlanError::UnknownCatalogEntry {
        node_type_id: id.to_string(),
    })
}
```

**Entry-list convention** (generate_synthdefs.py:388-396) — the `definitions()` list is the Python-side analog of `CATALOG`; the planner extends both in lockstep:
```python
def definitions() -> list[SynthDefSpec]:
    return [
        source_oscillator(),
        source_noise(),
        lowpass(),
        delay(),
        mixer(),
        output(),
    ]
```

**Reuse existing domain enums** (do NOT redefine): `PortDirection` (session.rs:281) and `SignalType` (session.rs:288) are imported by the catalog's `CatalogPortSpec` (RESEARCH.md:426-427). The catalog types derive `Clone, Copy, Debug` (RESEARCH.md:417-453); follow the `#[derive]` style of `SynthDefResource` (synthdefs.rs:17) and `ScResourcePlan` (synthdefs.rs:23).

**Module declaration:** add `pub mod catalog;` to `src-tauri/src/lib.rs:1-6` alongside `pub mod domain;` etc., OR nest under `domain/` — match the flat top-level pattern in `domain/mod.rs` (single `pub mod session;`).

**Divergence note:** the catalog is `&'static` (compiled-in, no allocations) — do NOT introduce `phf` or a `BTreeMap` builder; RESEARCH.md:100 confirms `phf` is absent and the project favors stable primitives. Linear scan over ~16 entries is trivial.

---

### `src-tauri/src/audio/synthdefs.rs` (MODIFY — service, request-response)

**Analog:** itself. The refactor replaces the contents of 4 functions, keeps the file's structure (imports, `ScResourcePlanError`, `ScResourcePlan` structs).

**Error-enum pattern to extend** (synthdefs.rs:61-94) — add `UnknownCatalogEntry` here; follow the exact `#[error("...")]` + named-field style:
```rust
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ScResourcePlanError {
    #[error("duplicate bus id `{bus_id}`")]
    DuplicateBus { bus_id: String },
    // ... existing variants ...
    #[error("node `{node_id}` targets unsupported runtime `{runtime_target}`")]
    UnsupportedRuntimeTarget { node_id: String, runtime_target: String },
    // ADD: (success criterion #3 — replaces unreachable!() at :455)
    #[error("unknown catalog entry `{node_type_id}`")]
    UnknownCatalogEntry { node_type_id: String },
}
```

**The big dispatch to collapse** (synthdefs.rs:116-217) — `match &node.node_kind { Source{..} => ... }` becomes `find_catalog_entry(&node.node_type_id)?.synthdef_name`. Keep the surrounding `for (index, node) in topology.node_launch_order.iter().enumerate()` loop and the `synthdefs.insert(synthdef_name)` / `synths.push(ScSynthPlan { ... })` accumulation unchanged (synthdefs.rs:105-228).

**`apply_parameters` — the catalog replaces normalize_parameter_name** (synthdefs.rs:382-405):
```rust
fn apply_parameters(...) -> Result<(), ScResourcePlanError> {
    for parameter in parameters {
        let Some(name) = normalize_parameter_name(&parameter.name) else {   // ← :390, becomes entry param lookup
            return Err(ScResourcePlanError::UnsupportedParameter { ... });
        };
        args.push(arg(name, parameter.value as f32));
        controls.push(ScControlPlan { control_key: format!("{node_id}:{parameter_id}"), ... });
    }
}
```
Catalog replacement: `entry.parameters.iter().find(|p| p.id == parameter.name).map(|p| p.sc_arg)` (RESEARCH.md:201).

**`plan_buses` — extend with control-bus allocation** (synthdefs.rs:246-267 — RESEARCH.md Pattern 2). Copy this function's structure (BTreeMap accumulation, `ScResourcePlanError::DuplicateBus`/`InvalidBusChannels` returns) for the new control-bus pass:
```rust
fn plan_buses(topology: &CompiledTopology) -> Result<BTreeMap<String, f32>, ScResourcePlanError> {
    let mut bus_map = BTreeMap::new();
    for bus in &topology.buses {
        if bus.channels == 0 { return Err(ScResourcePlanError::InvalidBusChannels { ... }); }
        if bus_map.insert(bus.bus_id.clone(), (HARDWARE_AUDIO_BUS_OFFSET + bus.index) as f32).is_some() {
            return Err(ScResourcePlanError::DuplicateBus { ... });
        }
    }
    Ok(bus_map)
}
// ADD pub const FIRST_CONTROL_BUS_OFFSET: u32 = 1024; (Pitfall #2) and a sibling control-bus pass.
```

**`synthdef_resource` caller fix (Pitfall #4)** — synthdefs.rs:232 currently is `.into_iter().map(synthdef_resource).collect()` (expects `FnMut -> SynthDefResource`). Change to `iter().map(|n| find_catalog_entry(n).map(|e| SynthDefResource { name: e.synthdef_name, relative_path: e.synthdef_resource })).collect::<Result<Vec<_>,_>>()?`.

**Imports convention** (synthdefs.rs:1-4) — currently pulls `CompiledNodeKind` + closed enums from `domain::session`; after refactor the `domain::session` import shrinks (drop `AudioSourceType`/`AudioEffectType`) and a `use crate::catalog::{find_catalog_entry, NodeCatalogEntry};` is added.

---

### `src-tauri/src/audio/compiler.rs` (MODIFY — service, transform)

**Analog:** itself. The `CompiledNodeKind` closed enum (compiler.rs:51-67) collapses; `node_sort_key` (compiler.rs:418-426) and `compile_node_launch` (compiler.rs:324-385) become catalog-driven.

**`node_sort_key` — the dispatch to rewrite** (compiler.rs:418-426):
```rust
fn node_sort_key(node: &Node) -> (u8, &str) {
    let kind_rank = match node.node_type {           // ← :419, becomes find_catalog_entry(&node.node_type_id)?.category rank
        NodeType::Source => 0,
        NodeType::Effect => 1,
        NodeType::Mixer => 2,
        NodeType::Output => 3,
    };
    (kind_rank, node.id.as_str())
}
```
Replacement: `let kind_rank = entry.category.rank();` where `NodeCategory` carries the rank (RESEARCH.md:418 — `Source, Modulator, Effect, Utility, Sequencer, Mixer, Output`).

**`compile_node_launch` — AudioPrimitive→CompiledNodeKind** (compiler.rs:335-363) — the `match primitive { AudioPrimitive::Source(...) => CompiledNodeKind::Source{...} }` becomes catalog-derived. The `CompiledParameter` mapping (compiler.rs:375-383) stays as-is.

**`validate_routes` — gains CV-port type checking** (compiler.rs:194-221). Today `validate_port_exists` (compiler.rs:245-259) only checks the port id exists; CV wiring (D-04/D-05) requires also checking the port's `signal_type` matches between source/target on a CV route. Extend `validate_port_exists` or add a sibling `validate_port_signal_type`. Keep the `TopologyCompileError::UnknownPortReference` style (compiler.rs:81-86).

**Imports** (compiler.rs:1-9) — drop `AudioPrimitive, AudioSourceNode, AudioEffectNode, AudioMixerNode, AudioOutputNode, AudioSourceType, AudioEffectType, AudioOutputType, NodeType`; add `use crate::catalog::{find_catalog_entry, NodeCategory};`.

**Topology pipeline UNCHANGED** — `compile_session_to_topology` (compiler.rs:93-121), `topo_sort_nodes` (compiler.rs:261-322), `compile_buses` (compiler.rs:123-141) keep their structure. RESEARCH.md:248 (Don't Hand-Roll) explicitly says do NOT re-architect the topo-sort.

---

### `src-tauri/src/domain/session.rs` (MODIFY — model, CRUD)

**Analog:** itself. D-09 (clean break) means the closed enums can be reshaped freely.

**Closed enums to supersede** (session.rs:193-262):
- `NodeType` (:195) — `Source/Effect/Mixer/Output` → replaced by `node_type_id: String` + `NodeCategory` tag
- `AudioPrimitive` (:204) — tagged enum → removed (per-node config moves to flat optional fields / `NodeData`)
- `AudioSourceType` (:221), `AudioEffectType` (:236), `AudioOutputType` (:258) — removed

**`Node` struct reshape** (session.rs:177-191) — keep `#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]` + `#[serde(rename_all = "camelCase")]`. Drop `node_type`/`audio_primitive`; add `node_type_id: String`. Add sequencer pattern payload (RESEARCH.md:511-517):
```rust
pub struct Node {
    pub id: String,
    pub node_type_id: String,              // ← replaces node_type + audio_primitive
    pub ports: Vec<Port>,
    pub parameters: Vec<ParameterValue>,
    pub runtime_target: Option<String>,
    pub scene_membership: Vec<String>,
    pub ownership: OwnershipAssignment,
    pub enabled: bool,
    // sequencer pattern (D-07/D-08); Option so non-sequencer nodes are unaffected
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequencer_pattern: Option<SequencerPattern>,
}
pub struct SequencerPattern {
    pub gate: [bool; 16],
    pub cv: [f64; 16],
}
```

**Reuse existing `SignalType`** (session.rs:286-291 — `Audio`/`Control`) — already present; the catalog's CV ports use it directly. No new signal-type enum needed.

**Schema bump** (session.rs:9): `pub const CURRENT_SCHEMA_VERSION: u32 = 1;` → `2`. (D-09)

**`GraphEditCommand` — add sequencer variant** (session.rs:471-502). Copy the `#[serde(tag = "type", content = "payload", rename_all = "camelCase")]` style and the named-field tuple shape:
```rust
pub enum GraphEditCommand {
    AddNode { node: Node },
    RemoveNode { node_id: String },
    // ... existing ...
    // ADD (RESEARCH.md:608 — dedicated variant preferred over SetParameterValue reuse):
    SetStepValue { node_id: String, step_index: u8, gate: Option<bool>, cv: Option<f64> },
}
```

**ts-rs export list — one line per new type** (session.rs:806-875). Copy the `Type::decl(&cfg)` pattern verbatim, add to the `declarations` array:
```rust
NodeCatalogEntry::decl(&cfg),
CatalogPortSpec::decl(&cfg),
CatalogParamSpec::decl(&cfg),
NodeCategory::decl(&cfg),
SequencerPattern::decl(&cfg),
```
The catalog types live in `catalog/mod.rs` but must be `#[derive(TS)]` + `pub` so `write_generated_typescript_contract` can name them. The `Config::default()` + `.join("\n\n")` + `.replace(...)` post-processing (session.rs:807-892) stays unchanged.

**Test style** (session.rs:903-1141) — `session_document_default_round_trip_preserves_required_collections` (:914) and `audio_graph_schema_round_trips_supported_v1_primitives` (:993) must be updated for the new `Node` shape; follow the `contract_write_lock()` OnceLock pattern (:908) for any new contract-write test.

---

### `src-tauri/src/visual/compiler.rs` (MODIFY — service, transform)

**Analog:** itself. The NodeType→shape match (:56-61) becomes `entry.visual_shape`.

**The dispatch to replace** (visual/compiler.rs:55-61):
```rust
.map(|(index, node)| {
    let element_type = match node.node_type {                    // ← :56, becomes find_catalog_entry(&node.node_type_id)?.visual_shape
        crate::domain::session::NodeType::Source => "sphere",
        crate::domain::session::NodeType::Effect => "box",
        crate::domain::session::NodeType::Mixer => "ring",
        crate::domain::session::NodeType::Output => "plane",
    };
    // ...
})
```
Replacement: `let element_type = find_catalog_entry(&node.node_type_id).map(|e| e.visual_shape).unwrap_or("box");` (graceful default keeps visual compile from hard-failing — matches the current non-Result `compile_session_to_visual_scene` signature at :28).

**Keep everything else** — `CompiledVisualElement` struct (:13), `merged_parameters` (:150), macro-override walk (:123) are catalog-agnostic.

---

### `src-tauri/src/audio/sequencer.rs` (NEW — controller, event-driven)

**Analog (OSC send):** `supercollider.rs:286-332 send_live_parameter` — the `/n_set` send pattern. The sequencer's `/c_set` is the same `osc.send_message(addr, vec![OscType::...])` shape (RESEARCH.md:226-232).
**Analog (thread/periodic):** `supercollider.rs:7 use std::thread` + `:381-403 wait_for_scsynth_boot` retry loop with `thread::sleep`. **No tokio** (RESEARCH.md:95 — verified absent).
**Analog (test):** `supercollider.rs:1031 ScriptedOscTransport` — capture sent `/c_set` packets and assert (RESEARCH.md:579).

**OSC send template** (supercollider.rs:318-328) — copy this `osc.send_message` + `OscType` arg shape for `/c_set`:
```rust
osc.send_message(
    "/n_set",                                    // sequencer uses "/c_set"
    vec![
        rosc::OscType::Int(control.synth_node_id),
        rosc::OscType::String(control.parameter_name.clone()),
        rosc::OscType::Float(value as f32),
    ],
).map_err(|err| format!("... failed to send /n_set to scsynth: {err}"))?;
```

**Periodic-loop template** (supercollider.rs:381-403) — the `Instant::now() + timeout` / `thread::sleep` pattern:
```rust
let deadline = Instant::now() + SCSYNTH_BOOT_TIMEOUT;
loop {
    match self.sync_scsynth("boot") {
        Ok(()) => return Ok(()),
        Err(error) => {
            if Instant::now() >= deadline { return Err(error); }
            thread::sleep(SCSYNTH_BOOT_RETRY_DELAY);
        }
    }
}
```
Sequencer tick loop (RESEARCH.md:524 recommendation b): `std::thread::spawn` + `thread::sleep(Duration::from_secs_f64(60.0 / bpm / 4.0))`, advancing `current_step = (current_step + 1) % 16`.

**Access to the OSC client:** the adapter owns `ScOscClient` (supercollider.rs:496). RESEARCH.md:302 recommends exposing a `tick_transport(...)` method on `AudioRuntimeAdapter` (runtime_manager.rs:11) OR having the controller hold an OSC handle. The trait-method route is cleaner — mirror the existing `set_parameter_value` signature (runtime_manager.rs:17).

**Transport math source** — `TransportState { tempo_bpm, is_playing, position_beats }` (session.rs:159-175). Step boundary every `60.0 / bpm / 4.0` s; current step `= floor(position_beats / 0.25) % 16` (RESEARCH.md:522, A2).

**Divergence note (weakest analog):** there is NO v1 periodic-driver or transport-tick loop. This is the one genuinely new runtime structure. The planner must explicitly decide spawn/kill lifecycle (spawn on `AudioRuntimeLifecycle::Ready` + `is_playing`, kill on stop/panic) — see RESEARCH.md Pitfall #6.

---

### `src-tauri/resources/synthdefs/v2/generate_synthdefs.py` (NEW — build tooling, file I/O)

**Analog:** `src-tauri/resources/synthdefs/v1/generate_synthdefs.py` (exact — extend/copy). The `SynthDefBuilder` + `SynthDefSpec` + `synthdef_bytes()` model (generate_synthdefs.py:44-115, :351-385) is the reusable asset; do NOT rewrite the byte serializer (RESEARCH.md:245).

**Def-authoring template** (generate_synthdefs.py:158-193 `lowpass()`) — this is the closest analog for a CV-input node. The v2 filter adds a `cutoff_cv_bus` control read via `In.kr`:
```python
def lowpass() -> SynthDefSpec:
    builder = SynthDefBuilder("scrysynth_v1_fx_lowpass")
    in_bus, out_bus, cutoff_hz, resonance, mix, bypassed = builder.controls([...])
    input_left, input_right = builder.ugen("In", RATE_AUDIO, [in_bus], outputs=2)   # ← audio-bus read template
    clipped_cutoff = builder.clip(cutoff_hz, 20.0, 20000.0)
    # v2 ADD: cutoff_cv_bus control + In.kr read, summed into cutoff (RESEARCH.md:344-350)
    filtered_left = builder.ugen("RLPF", RATE_AUDIO, [input_left, clipped_cutoff, ...])[0]
    # ... XFade2 wet/dry, Select bypass, builder.out(...) — unchanged pattern
```

**Rate constants** (generate_synthdefs.py:16-18): `RATE_SCALAR=0, RATE_CONTROL=1, RATE_AUDIO=2`. CV-bus reads use `RATE_CONTROL` + `In` UGen; the catalog `CatalogPortSpec.signal_type` decides the rate (Pitfall #3).

**Output convention** (generate_synthdefs.py:399-404 `main()`): writes `spec.name + ".scsyndef"` into `Path(__file__).resolve().parent` (the `v2/` dir). Copy this; the `definitions()` list (:388-396) grows to ~13 entries.

**New UGens per RESEARCH.md:470-488** — `LFTri`, `LFCub`/`LFSaw`/`LFPulse`, `EnvGen`/`Done`/`FreeSelf`, `LPF`/`HPF`/`BPF`/`BRF`, `FreeVerb`/`GVerb`, `CombC`/`AllpassN`, `Round`. The Python writer emits UGen names verbatim (`builder.ugen("SinOsc", ...)`) — no code change, just new def functions. Conformance test (below) catches any unavailable UGen at `/d_recv`.

**`.scd` mirror (optional):** `src-tauri/resources/synthdefs/v1/scrysynth_v1_synthdefs.scd` is the human-readable analog. RESEARCH.md:101 says `.scd` is optional; the Python writer is the deterministic source.

---

### `src-tauri/src/persistence/session_file.rs` (MODIFY — service, file I/O)

**Analog:** itself. Two-phase parse (Pitfall #1) + friendly message (D-10).

**The version check to extend** (session_file.rs:50-67) — current single-phase parse fails on serde before the version check:
```rust
pub fn open_session_from_path(path: &Path) -> Result<SessionDocument, SessionFileError> {
    let contents = fs::read_to_string(path).map_err(|source| SessionFileError::Read { ... })?;
    let session: SessionDocument = serde_json::from_str(&contents).map_err(SessionFileError::Deserialize)?;  // ← :57 fails first on v1
    if session.schema_version != CURRENT_SCHEMA_VERSION {                                                       // ← :59 never reached for v1
        return Err(SessionFileError::UnsupportedSchemaVersion { expected: CURRENT_SCHEMA_VERSION, found: session.schema_version });
    }
    Ok(session)
}
```
**Two-phase replacement** (RESEARCH.md:273, :549): parse `{ schemaVersion }` first via a tiny `SchemaVersionProbe` struct, check, then full-parse only if version == 2.

**Error enum to extend** (session_file.rs:8-28) — add a friendly v1 variant; follow the `#[error("...")]` + rename style:
```rust
#[derive(Debug, Error)]
pub enum SessionFileError {
    // ... existing ...
    #[error("unsupported schemaVersion {found}; expected {expected}")]
    UnsupportedSchemaVersion { expected: u32, found: u32 },
    // ADD (D-10 friendly message — emit when found == 1):
    #[error("This is a v1 session — unsupported in Scrysynth v2. Open a v2 session or start a new one.")]
    LegacyV1Session,
}
```

**Frontend surfacing (no IPC change):** `lib.rs:66-76 open_session_from_path` maps `SessionFileError → String` via `.map_err(|e| e.to_string())`; `session-client.ts:434 openSessionFromPath` rejects with that string. The `Display` string IS the friendly message — no new IPC needed (RESEARCH.md:550).

---

### `src-tauri/src/audio/supercollider.rs` (MODIFY — adapter, request-response)

**Analog:** itself. The sequencer's `/c_set` rides on the same `ScOscClient` (supercollider.rs:496) + `OscTransport` trait (:612).

**`apply_resource_plan` — CV-bus arg injection** (supercollider.rs:188-279): the `/s_new` send (:254-272) currently builds args from `synth.args`. The catalog-derived plan adds `<param>_cv_bus` args; the existing `synth_args_to_osc` (:425-434) handles them with zero changes — `ScSynthArg { name, value }` is generic.

**`/d_recv` path (conformance test reuse)** (supercollider.rs:200-215): the conformance test (below) calls this exact block per catalog entry to verify each synthdef loads on real scsynth.

**`resolve_scsynth_executable`** (supercollider.rs:459-477): the conformance test reuses this to find scsynth (env override → PATH → macOS bundle fallback). Gate the test on `Some` here (RESEARCH.md:372).

**Module-level constants** (supercollider.rs:14-23) — copy the `const` style for any sequencer timing constants; the `std::thread`/`std::time` imports (:7-8) are already present.

---

### `src-tauri/src/application/session_store.rs` (MODIFY — service/store, event-driven)

**Analog:** itself. The sequencer driver hooks into audio start/stop alongside `self.audio_runtime_manager` (referenced at :174-202 region for reconcile). 

**`poll_hardware_events`** (session_store.rs:220-229) — the only existing periodic entry point (hardware-driven). RESEARCH.md:302 recommends a dedicated `std::thread::spawn` sequencer loop over piggybacking here, but the spawn/kill calls live in the audio start/stop paths this file owns.

**Pattern:** mirror how `audio_runtime_manager` is stored as a field and invoked from `start_audio_runtime`/`stop_audio_runtime`/`panic_audio_runtime`. Add a `sequencer_controller: Option<SequencerController>` field with matching start/stop calls.

---

### `src-tauri/src/audio/mod.rs` + `src-tauri/src/lib.rs` (MODIFY — config)

**Analog:** `src-tauri/src/audio/mod.rs` (the 4-line `pub mod` list) + `src-tauri/src/lib.rs:1-6`.

**Module declaration convention** (audio/mod.rs:1-4):
```rust
pub mod compiler;
pub mod runtime_manager;
pub mod supercollider;
pub mod synthdefs;
// ADD:
pub mod sequencer;
```
Add `pub mod catalog;` to `lib.rs:1-6` (top-level, alongside `pub mod domain;`).

---

### `src/components/audio/PrimitivePalette.tsx` (MODIFY — component, request-response)

**Analog:** itself. The hardcoded `buildPrimitiveNode` factory (:62-152) becomes a catalog iterator.

**The factory to replace** (PrimitivePalette.tsx:62-92):
```tsx
function buildPrimitiveNode(kind: PrimitiveKind, session: SessionDocument | null): Node {
  const suffix = globalThis.crypto.randomUUID().slice(0, 8);
  const id = `${kind}-${suffix}`;
  // ... hardcoded ports/params per kind ...
  if (kind === "source") {
    return {
      id, nodeType: "source",
      ports: [{ id: `${id}-out`, name: "main_out", direction: "output", signalType: "audio" }],
      parameters: [{ id: `${id}-level`, name: "level", value: 0.75, ... }],
      runtimeTarget: `audio/source/${id}`,   // ← :84 the quirk the catalog fixes (RESEARCH.md:534)
      // ...
      audioPrimitive: { kind: "source", config: { sourceType: "oscillator", ... } },
    };
  }
  // ... effect, mixer ...
}
```
**Catalog-driven replacement** (RESEARCH.md:531-535): iterate `NodeCatalogEntry[]` (from regenerated `session-types.ts`); one button per entry; build `Node` from `entry.ports` + `entry.parameters` (defaults); `runtimeTarget = entry.id` (fixes the `audio/source/${id}` quirk at :84).

**Import convention** (PrimitivePalette.tsx:1): `import type { Node, SessionDocument } from "../../generated/session-types";` — add `NodeCatalogEntry` to this import after ts-rs regen.

**Button block** (PrimitivePalette.tsx:34-44): the 3 hardcoded `<button>` elements become `.map` over the catalog array; keep the `disabled={isLoading}` + `onClick={() => addPrimitive(...)}` prop pattern.

---

### `src/components/session/NodeInspector.tsx` (MODIFY — component, request-response)

**Analog:** itself. Parameter slider (:100-127) is reusable; add CV-port display + sequencer-step UI.

**Reusable parameter slider** (NodeInspector.tsx:100-127) — works as-is for catalog-derived params; each `parameter` already carries `minValue`/`maxValue`/`unit`:
```tsx
selectedNode.parameters.map((parameter) => (
  <div key={parameter.id} className="list-card">
    <div className="parameter-header">
      <p>{parameter.name}</p>
      <span>{parameter.value.toFixed(2)} {parameter.unit}</span>
    </div>
    <input type="range" min={parameter.minValue} max={parameter.maxValue}
      step={(parameter.maxValue - parameter.minValue) / 100 || 0.01}
      value={parameter.value} disabled={isLoading}
      onChange={(event) => onUpdateParameter(selectedNode.id, parameter.id, Number(event.target.value))} />
  </div>
))
```
**Sequencer-step UI (NEW):** mirror this `.map` pattern over `selectedNode.sequencerPattern.gate` (16 toggles) and `.cv` (16 sliders). The `onUpdateParameter` callback shape suggests a new `onUpdateStep(nodeId, stepIndex, gate?, cv?)` prop matching `GraphEditCommand::SetStepValue`.

**Ports display** (NodeInspector.tsx:90-98) — already iterates `selectedNode.ports` generically; CV ports (added by catalog) render for free once the Node carries them. No change needed for port listing; only the identity header (:43-49 `selectedNode.nodeType`) needs to switch to `node_type_id` / display name.

**Identity header** (NodeInspector.tsx:43, :49): `selectedNode.nodeType` → catalog display-name lookup (or just `selectedNode.nodeTypeId`).

---

### `src/lib/session-client.ts` (MODIFY — service/client, validate)

**Analog:** itself. The Zod block (:42-114) is the hand-maintained mirror that must relax (Pitfall #5).

**Schemas to relax** (session-client.ts:70-114):
```tsx
const nodeSchema = z.object({
  id: z.string(),
  nodeType: z.enum(["source", "effect", "mixer", "output"]),   // ← :72, relax to z.string() or drop
  // ...
  audioPrimitive: z.discriminatedUnion("kind", [                 // ← :79-113, remove or regenerate from catalog
    z.object({ kind: z.literal("source"), config: z.object({ sourceType: z.enum(["oscillator","noise"]), ... }) }),
    // ...
  ]).nullable(),
});
```
**Replacement (RESEARCH.md:295, :540):** `nodeTypeId: z.string()`, drop the closed-enum `nodeType` and the `audioPrimitive` discriminated union (the model flattens). Add `sequencerPattern: z.object({ gate: z.array(z.boolean()).length(16), cv: z.array(z.number()).length(16) }).nullable().optional()`. Port schema (:58-63) is already generic (`signalType: z.enum(["audio","control"])`) — CV ports validate without change.

**`invokeSession` parse gate** (session-client.ts:413-416): `sessionDocumentSchema.parse(payload)` — if schemas drift, every call throws. Must regenerate in lockstep with Rust (Pitfall #5).

**`openSessionFromPath`** (session-client.ts:434-436) — unchanged; the friendly v1 message arrives as the rejected string. Caller catches and shows a dialog.

---

### `src-tauri/tests/audio_runtime.rs` (MODIFY — test)

**Analog:** itself. The conformance test extends `checked_in_v1_synthdef_resources_are_present_and_named` (:433) + the lifecycle `FakeAdapter` (:514-585).

**Conformance template (extend)** (audio_runtime.rs:432-490) — currently checks SCgf header + embedded name WITHOUT booting scsynth. Phase 12 success criterion #4 goes further: boot real scsynth per entry:
```rust
#[test]
fn checked_in_v1_synthdef_resources_are_present_and_named() {
    let resources = [ (SOURCE_OSCILLATOR_SYNTHDEF, "resources/synthdefs/v1/..."), ... ];
    for (name, relative_path) in resources {
        let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap();
        assert!(bytes.starts_with(b"SCgf"), "{relative_path} has SCgf header");
        assert_eq!(i32::from_be_bytes(bytes[4..8].try_into().unwrap()), 2, "SynthDef v2");
        // ... name check ...
    }
}
```
**v2 conformance (NEW, `#[ignore]` + scsynth-gated)** (RESEARCH.md:364-374): resolve scsynth via `resolve_scsynth_executable` (supercollider.rs:459), boot via `SuperColliderAdapter::start`, `/d_recv` each catalog entry's bytes, assert `/done`, `/d_free`. Gate on scsynth availability (skip with clear message if absent — bundle is at `/Applications/SuperCollider.app/Contents/Resources/scsynth`, not on PATH).

**Unknown-id Err test (success criterion #3)** — extend `resource_plan_fails_loudly_for_unsupported_runtime_target` (audio_runtime.rs:339). Assert an unknown `node_type_id` returns `Err(ScResourcePlanError::UnknownCatalogEntry)`, NOT a panic.

**CV-bus allocation test (NEW)** — extend `resource_plan_maps_supported_primitives_to_known_synthdefs_and_params` (audio_runtime.rs:209) with an LFO→filter Control-type route; assert the plan carries `out_cv_bus` on the LFO and `cutoff_cv_bus` on the filter (RESEARCH.md:575).

**Sequencer `/c_set` test (NEW)** — mirror `apply_resource_plan_sends_groups_then_synths_in_plan_order` (supercollider.rs:768) using `ScriptedOscTransport` (supercollider.rs:1031) to capture `/c_set` packets and assert 16 steps advance (RESEARCH.md:579).

**`deterministic_session()` helper** (audio_runtime.rs:10-86) — must be rebuilt for the new `Node` shape (drop `node_type`/`audio_primitive`, add `node_type_id`); every test in the file depends on it.

---

### `src/lib/session-client.test.ts` (MODIFY — test)

**Analog:** itself. Browser-preview round-trip pattern (session-client.test.ts:13-52).

**`buildPreviewEffectNode` helper** (session-client.test.ts:54-82) — must be rebuilt for the new `Node` shape (drop `nodeType`/`audioPrimitive`, add `nodeTypeId`). Add a v2 catalog-derived round-trip test (RESEARCH.md:578): serialize a catalog-derived session through Zod and assert it parses.

---

## Shared Patterns

### Typed Error Enum (thiserror)
**Source:** `src-tauri/src/audio/synthdefs.rs:61-94` (`ScResourcePlanError`) + `src-tauri/src/persistence/session_file.rs:8-28` (`SessionFileError`) + `src-tauri/src/audio/compiler.rs:69-91` (`TopologyCompileError`)
**Apply to:** `catalog/mod.rs` (new `UnknownCatalogEntry` variant on `ScResourcePlanError`), `session_file.rs` (new `LegacyV1Session` variant)
```rust
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ScResourcePlanError {
    #[error("...named fields...")]
    Variant { field: String },
}
```
Convention: named-field variants (not tuple), `#[error]` with backtick-quoted identifiers, `PartialEq` derive on `ScResourcePlanError` (test assertions use `matches!`/`assert_eq!`).

### Catalog-Driven Dispatch (replaces all 5 hardcoded spots)
**Source:** RESEARCH.md Pattern 1 + the 5 spots in synthdefs.rs/compiler.rs/visual/compiler.rs
**Apply to:** every former `match node_kind/node_type { ... }`. The replacement is uniformly `find_catalog_entry(&node.node_type_id)?` then field access. **Anti-pattern (RESEARCH.md:235):** never hand-roll a second `match` for new node kinds — every new node is data in `CATALOG`.

### serde + ts-rs Dual-Derive Domain Types
**Source:** `src-tauri/src/domain/session.rs:11-6` (every domain type) + `:806 write_generated_typescript_contract`
**Apply to:** all new domain types (`SequencerPattern`, catalog types if they live in `domain/`)
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SequencerPattern { ... }
```
Then add `SequencerPattern::decl(&cfg)` to the `declarations` array (session.rs:808-875). The generated file `src/generated/session-types.ts` is checked in and consumed by both `PrimitivePalette.tsx` and `session-client.ts`.

### OSC Send via ScOscClient
**Source:** `src-tauri/src/audio/supercollider.rs:527-532 send_message` + `:318-328 /n_set` + `:208 /d_recv`
**Apply to:** sequencer `/c_set` (sequencer.rs), conformance `/d_recv`+`/d_free` (audio_runtime.rs)
```rust
osc.send_message("/c_set", vec![
    rosc::OscType::Int(gate_bus_index), rosc::OscType::Float(if step.gate { 1.0 } else { 0.0 }),
    rosc::OscType::Int(cv_bus_index),   rosc::OscType::Float(step.cv as f32),
]).map_err(|err| format!("... failed: {err}"))?;
```
`/c_set` is fire-and-forget (no `/sync` needed — RESEARCH.md:249); `/d_recv` requires the existing `sync_scsynth` pattern (supercollider.rs:368).

### Typed-Command Mutation Gate
**Source:** `src-tauri/src/domain/session.rs:471-502 GraphEditCommand` + `src-tauri/src/audio/runtime_manager.rs:174-197 reconcile_graph_edit`
**Apply to:** sequencer step edits (D-06/D-08) — add `GraphEditCommand::SetStepValue`; `reconcile_graph_edit`'s `match command { ... }` gains an arm (or routes through `reapply_live_topology` like `AddRoute`).

### std::thread Periodic Loop (no tokio)
**Source:** `src-tauri/src/audio/supercollider.rs:7` (`use std::thread`) + `:381-403 wait_for_scsynth_boot` (Instant deadline + thread::sleep)
**Apply to:** sequencer transport-tick driver (RESEARCH.md Pitfall #6, recommendation b)
```rust
std::thread::spawn(move || loop {
    let period = Duration::from_secs_f64(60.0 / bpm / 4.0);
    send_cset_for_step(current_step); 
    current_step = (current_step + 1) % 16;
    thread::sleep(period);
});
```
tokio is **not** a dependency (RESEARCH.md:95, verified); introducing it is a larger surface change than this phase warrants.

### ScriptedOscTransport Test Harness
**Source:** `src-tauri/src/audio/supercollider.rs:1031-1086` (`ScriptedOscTransport` + `ScriptedResponse`) + `:768 apply_resource_plan_sends_groups_then_synths_in_plan_order`
**Apply to:** sequencer `/c_set` advance test — capture sent packets, assert addr sequence + arg values. Reuse the `osc_message_addr`/`osc_message_int_arg`/`osc_message_float_arg` helpers (supercollider.rs:941-981).

## No Analog Found

| File | Role | Data Flow | Reason / Fallback |
|------|------|-----------|-------------------|
| `src-tauri/src/audio/sequencer.rs` (periodic tick driver) | controller | event-driven (periodic) | No v1 periodic-driver or transport-tick loop exists. **Closest partials:** `supercollider.rs:381-403` (std::thread + Instant deadline loop) for the timing primitive; `supercollider.rs:286-332 send_live_parameter` for the OSC-send shape; `runtime_manager.rs:11 AudioRuntimeAdapter` trait for the integration surface. Planner must explicitly design spawn/kill lifecycle (RESEARCH.md Pitfall #6). |
| `src-tauri/src/catalog/mod.rs` (the catalog itself) | config / model | lookup | No v1 equivalent — it IS the new single source of truth replacing the 3 allowlists. **Closest partials:** `synthdefs.rs:10-15` const block (representation), `synthdefs.rs:429-457 synthdef_resource` (the lookup+panic shape it replaces), `generate_synthdefs.py:388 definitions()` (the Python-side entry list to mirror). Use RESEARCH.md "Catalog Data Model" (:411-462) as the spec. |

## Metadata

**Analog search scope:** `src-tauri/src/{audio,domain,application,visual,persistence}/**/*.rs`, `src-tauri/tests/audio_runtime.rs`, `src-tauri/resources/synthdefs/v1/*`, `src/{components,lib}/**/*.{tsx,ts}`, `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml` (rg for tokio — absent).
**Files scanned:** 19 source files read in full (synthdefs.rs, compiler.rs, session.rs, session_file.rs, visual/compiler.rs, supercollider.rs, runtime_manager.rs, audio_runtime.rs, generate_synthdefs.py, PrimitivePalette.tsx, NodeInspector.tsx, session-client.ts + .test.ts, lib.rs, mod.rs ×2, agent_planner.rs, session_store.rs) + CONTEXT.md/RESEARCH.md.
**Pattern extraction date:** 2026-06-26
**Key insight for planner:** Phase 12 is a *content* refactor, not new infrastructure. v1 already built the hard parts (topology compiler, OSC sync/cleanup, ts-rs pipeline, deterministic SynthDef byte-writer, friendly typed-error enums). Copy the surrounding convention of each file and replace the dispatch *contents* with catalog lookups. The only genuinely new runtime structure is the sequencer's periodic tick driver (partial analog only).
