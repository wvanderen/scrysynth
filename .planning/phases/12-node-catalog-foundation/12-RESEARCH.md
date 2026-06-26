# Phase 12: Node Catalog Foundation - Research

**Researched:** 2026-06-26
**Domain:** Rust domain-model refactor + SuperCollider SynthDef authoring + CV/modulation bus wiring + app-driven step sequencer + ts-rs/Zod contract regeneration
**Confidence:** HIGH (findings are grounded in direct reads of the v1 source at exact file:line; SuperCollider mechanics are well-established)

## Summary

Phase 12 is the single highest-leverage v2 refactor: it replaces v1's **three hardcoded compiler allowlists** (`synthdef_resource`, `normalize_parameter_name`, `validate_runtime_target` — all in `src-tauri/src/audio/synthdefs.rs`) plus two closed-enum dispatch spots (`audio/compiler.rs:418` `node_sort_key`, `visual/compiler.rs:57` shape dispatch) with **one compiled-in `NodeCatalogEntry` const table** that becomes the single source of truth for compiler dispatch, route validation, palette, inspector, `ts-rs` export, and visual shape. The closed identity enums (`NodeType`, `AudioSourceType`, `AudioEffectType`, `AudioPrimitive`) collapse into a flat `node_type_id: String` on `Node` plus a `NodeCategory` tag for sort/shape semantics. D-09 (clean break) means this reshape needs no v1 migration.

The two genuinely new capabilities are: (1) **end-to-end CV/modulation** (D-03/D-04/D-05) — the compiler must allocate **control buses** (it currently only allocates audio buses at `synthdefs.rs:246`) and each modulatable parameter's SynthDef must gain a `<param>_cv_bus` arg that reads `In.kr(bus)` and adds to the base value; mod sources (LFO/Env/Sequencer) write via `Out.kr`. (2) **app-driven step sequencing** (D-06/D-07/D-08) — Rust owns a transport-tick loop that advances 16 fixed steps and writes per-step gate/CV to allocated control buses via SC's `/c_set` OSC message; SC stays "dumb." The transport-tick loop needs a periodic driver, and **`tokio` is NOT yet a dependency** (`Cargo.toml` confirmed via `rg`) — the audio layer is fully synchronous (std::thread/UdpSocket), so the planner must either add tokio or use a std::thread sleep loop / extend the existing poll path.

The SynthDef authoring toolchain is already in place: `generate_synthdefs.py` is a deterministic, sclang-free SynthDef-v2 byte-writer that the planner should **extend** for the ~12-16 new node defs (the `.scd` files are optional human-readable mirrors). The conformance gate ("boots real `scsynth` for every entry") extends the existing `checked_in_v1_synthdef_resources_are_present_and_named` test pattern (`audio_runtime.rs:433`) but actually `/d_recv`s each catalog synthdef to a real scsynth — gated on scsynth availability (bundle fallback is present locally at `/Applications/SuperCollider.app/Contents/Resources/scsynth`; not on PATH). The v1 rejection (D-10) is largely pre-wired: `SessionFileError::UnsupportedSchemaVersion` already exists and is checked (`session_file.rs:26,59`); bumping `CURRENT_SCHEMA_VERSION` to 2 auto-rejects v1 files — the only real work is a two-phase parse (so a model-drift serde failure doesn't preempt the friendly version message) and a friendlier error string.

**Primary recommendation:** Model the catalog as a `pub const CATALOG: &[NodeCatalogEntry]` slice (no new dependency — `phf` is not present and the project favors stable primitives), keyed by string `node_type_id`. Each entry carries `synthdef_name`, `synthdef_resource` path, `&[CatalogPortSpec]` (audio + per-param CV ports), `&[CatalogParamSpec]` (each with `sc_arg` + aliases + `exposes_cv_port` flag), a `NodeCategory`, and a `visual_shape`. Add control-bus allocation to the compiler alongside the existing audio-bus allocation; wire CV via `In.kr`/`Out.kr` and the app-driven sequencer via `/c_set`. Keep control buses as a **compiler artifact** (not canonical session state) — routes + ports are canonical, bus indices are derived.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01: Param-driven is the default granularity.** Each node family is ONE node whose parameters carry the variety (e.g. one Oscillator node; a `waveform`/`wave_shape` param selects sine/saw/square/triangle inside one SynthDef). Fewer, richer catalog entries.
- **D-02: Split into separate nodes only where DSP fundamentally differs.** When one SynthDef cannot sensibly hold the variants (different UGen graphs), the family becomes separate catalog entries. The agent applies this per-family: a Filter node can stay param-driven (LPF/HPF/BPF/BRF share the `*LPF` UGen family), but delay vs reverb vs distortion are separate nodes.
- **D-03: Modulation works end-to-end in Phase 12.** A real patch audibly modulates through SuperCollider (e.g. an LFO node's output sweeps a filter's cutoff). CV inputs map to SC control/audio args via buses THIS phase — not deferred to Phase 13. Both control-rate (LFO/envelope → param) and audio-rate modulation are in scope.
- **D-04: Per-parameter CV ports, declared in the catalog.** Each modulatable parameter gets its own CV-input port declared in the `NodeCatalogEntry` (e.g. a Filter node has `audio_out`, plus `cutoff_cv` and `resonance_cv` control-input ports). A performer draws an edge from a mod source's output to the target param's CV-in port.
- **D-05: CV ports for continuous parameters only.** Continuous parameters (frequency, cutoff, resonance, level, feedback, delay_time, etc.) get a CV port. Discrete selectors (`wave_shape`, `noise_color`) and toggles (`bypass`) do NOT.
- **D-06: App-driven sequencing (Rust owns the logic).** Sequencer logic lives in the Rust app: it tracks transport position, advances steps, and sends gate/CV values to SC as control messages per step. SuperCollider stays "dumb."
- **D-07: Mono output — one gate + one CV out.** The sequencer outputs one gate and one CV value per step.
- **D-08: Fixed 16 steps.** The pattern grid is a fixed 16-step array in canonical state.
- **D-09: Clean break allowed — schema bump, old files unsupported.** Phase 12 may bump the session schema version and break v1 session files. The catalog defines node identity from scratch.
- **D-10: Friendly error on v1 file open — no load.** When a v1 session file is opened in the v2 app, show a clear, specific message and do NOT load it. No silent serde failure, no best-effort import.

### the agent's Discretion
- **FX/filter/utility per-family split:** Where exactly to draw param-driven vs separate-node lines across the FX and utility families (apply the DSP-fit rule from D-02).
- **SynthDef authoring toolchain for the ~12-16 new nodes:** extend the Python writer vs hand-write `.scd` vs hybrid — not a user-facing preference.
- **Catalog storage representation:** const table / `phf` map / builder module — planner's call.
- **Control-bus allocation strategy** for modulation wiring (D-03/D-04): how SC control/audio buses are allocated and mapped to CV ports — implementation detail.
- **Sequencer clock division / retrigger behavior** (D-06/D-08): sensible defaults for the planner.

### Deferred Ideas (OUT OF SCOPE)
None raised — discussion stayed within phase scope. (Clock-division detail, retrigger/legato behavior, and SynthDef-authoring toolchain were noted as planner/researcher discretion, not deferred scope-creep.)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| NODES-01 | A data-driven node catalog (single source of truth) lets new node types be added without touching hardcoded compiler allowlists | `NodeCatalogEntry` const table replaces the 3 allowlists in `synthdefs.rs` + 2 enum-dispatch spots; see **Catalog Data Model** + **The Five Hardcoded Spots**. |
| NODES-02 | User can add oscillator, filter, envelope, LFO, utility (VCA/mixer/noise/quantizer) nodes covering the full synthesis chain, mapped to SC UGens | Catalog entries per family + SynthDef authoring via extended `generate_synthdefs.py`; see **SynthDef Authoring Toolchain**. |
| NODES-03 | User can add effect nodes (delay, reverb, distortion, chorus/flanger) with characteristic + wet/dry parameters mapped to SC | Catalog entries for FX (separate nodes per DSP-fit rule D-02); new UGens (`FreeVerb`/`GVerb`, `CombC`/`AllpassN`, distortion via `tanh`/`Clip`); see **New SynthDefs Needed**. |
| NODES-04 | User can add a step-sequencer node with per-step gate/CV and clock transport | App-driven sequencer: Rust transport-tick loop + `/c_set` OSC on allocated control buses; 16-step pattern in canonical Node state; see **Step Sequencer (App-Driven)**. |
| NODES-05 | Every audio node exposes CV/modulation inputs (audio-rate + control-rate ports), making patches modular | Compiler gains control-bus allocation; SynthDefs gain `<param>_cv_bus` args read via `In.kr`; catalog declares per-param CV ports (D-04/D-05); see **CV/Modulation Wiring**. |
</phase_requirements>

## Project Constraints (from AGENTS.md)

- **Use `td` for task management** (`td usage --new-session` at session start; `td usage -q` for reads). Planner/executor must honor this.
- **Canonical session truth lives in the Rust app** — SC/visual adapters consume compiled projections. The catalog continues this: Rust-owned, SC consumes catalog-derived plans. `[VERIFIED: AGENTS.md + PROJECT.md §Constraints]`
- **SuperCollider is execution, not the model** — the catalog is the model; SC SynthDefs are downstream artifacts. `[VERIFIED: AGENTS.md]`
- **Favor stable, explainable primitives** — prefer the existing deterministic Python SynthDef writer over sclang; prefer a plain const table over new dependencies; app-driven sequencing over SC-side sequencing. `[VERIFIED: AGENTS.md + PROJECT.md §Key Decisions]`
- **GSD workflow enforcement** — no direct repo edits outside a GSD workflow. This research is part of `/gsd-plan-phase 12`. `[VERIFIED: AGENTS.md]`

## Architectural Responsibility Map

Mapped before framework research to prevent tier misassignment in the plan.

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Node catalog (single source of truth) | Rust core (`domain` + new `catalog` module) | — | Canonical truth is Rust-owned; ts-rs exports a projection. Matches the locked architecture. |
| SynthDef authoring (new node DSP graphs) | Build-time tooling (`generate_synthdefs.py`) | checked-in `.scsyndef` resources | Byte-accurate artifacts compiled ahead-of-time; runtime loads bytes via `/d_recv`. |
| Compiler dispatch (node→synthdef+args) | Rust audio compiler (`audio/compiler.rs` + `synthdefs.rs`) | — | Topology compiler already owns this; the refactor replaces the dispatch *contents*, not the pipeline. |
| Control-bus allocation (CV wiring) | Rust audio compiler | — | Audio-bus allocation already lives here (`plan_buses` :246); control buses are the same tier. |
| CV modulation read/write (DSP) | SuperCollider SynthDefs (`In.kr`/`Out.kr`) | — | SC executes the DSP; Rust only allocates bus indices and passes them as args. |
| Step sequencer logic (transport tick, step advance) | Rust app (audio layer / new controller) | — | D-06 locks this: app owns sequencing. SC is "dumb" (receives `/c_set`). |
| Palette / inspector UI | Frontend (React) | generated TS types | Consumes catalog via ts-rs-exported `NodeCatalogEntry`; replaces hardcoded `buildPrimitiveNode`. |
| Schema version + v1 rejection | Rust persistence (`session_file.rs`) | Frontend (error dialog surfacing) | Version check already lives in Rust; frontend shows the message. |
| Visual shape dispatch (Phase 15 consumer) | Rust visual compiler (`visual/compiler.rs`) | catalog `visual_shape` field | Today dispatches on `NodeType`; becomes catalog-driven so Phase 15 reads the field. |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde` / `serde_json` | 1 (Cargo.toml:26) | Serialize catalog-augmented `SessionDocument`; two-phase v1-version parse | Already the canonical Rust serialization layer. `[VERIFIED: Cargo.toml]` |
| `ts-rs` | 12 (Cargo.toml:29) | Export `NodeCatalogEntry` + new domain types to `src/generated/session-types.ts` | Already drives `write_generated_typescript_contract()` (session.rs:806); catalog join is a one-line addition. `[VERIFIED: Cargo.toml + session.rs:806]` |
| `rosc` | 0.11 (Cargo.toml:32) | OSC `/c_set` (sequencer gate/CV), `/d_recv` (conformance), `/n_set` (live params) | Already the SC transport; `/c_set` is a new OSC address, same library. `[VERIFIED: Cargo.toml + supercollider.rs]` |
| `thiserror` | 2 (Cargo.toml:28) | Typed errors (`UnknownCatalogEntry`, friendly v1-reject variant) | Already backs `ScResourcePlanError` (synthdefs.rs:61) and `SessionFileError` (session_file.rs:8). `[VERIFIED: Cargo.toml]` |
| `zod` | 4 (package.json) | Validate v2 session payloads at the TS boundary; relax closed-enum schemas to catalog-driven strings | Already mirrors Rust types in `session-client.ts`; schemas must be regenerated in lockstep. `[VERIFIED: src/lib/session-client.ts:70-114]` |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Python 3 (stdlib only) | 3.14.6 locally (`/opt/homebrew/bin/python3`) | Extend `generate_synthdefs.py` with new SynthDef-v2 byte writers | Authoring the ~12-16 new `.scsyndef` files sclang-free. `[VERIFIED: bash probe]` |
| std::thread / std::time (Rust std) | stable | Sequencer transport-tick loop (if tokio not adopted) | Periodic `/c_set` dispatch. Matches the current sync audio layer. `[VERIFIED: audio layer uses std::thread — supercollider.rs:7]` |
| `tokio` | **NOT YET A DEPENDENCY** | Optional: async sequencer tick + future async adapter | Only if the planner chooses async; current audio layer is fully sync. `[VERIFIED: Cargo.toml via rg — tokio absent]` |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `&'static [NodeCatalogEntry]` const slice + linear/BTree lookup | `phf` perfect-hash map | `phf` is not a current dependency; the catalog is ~16 entries (linear scan is trivial). Favor stable primitives → const slice. `[VERIFIED: phf absent from Cargo.toml]` |
| Extending `generate_synthdefs.py` (sclang-free byte writer) | Hand-writing `.scd` + requiring `sclang` | Python path is deterministic, already checked-in, no sclang dev dependency; `.scd` files are optional human-readable mirrors. `[VERIFIED: generate_synthdefs.py:1-7 docstring states the sclang-free intent]` |
| `/c_set` OSC for sequencer gate/CV | A tiny SC "writer" synth per sequencer | `/c_set` is a direct app→scsynth control-bus write — zero SC-side logic, matches D-06 "SC stays dumb." `[CITED: SuperCollider OSC `/c_set` semantics]` |
| std::thread tick loop (sync) | tokio interval | tokio not yet adopted in the audio layer; introducing it is a larger surface change. std::thread matches existing patterns. Planner's call. |

**Installation:** No new crates or npm packages are required for this phase. All work is internal Rust + extending the existing Python script + new `.scsyndef` resource files. The planner adds catalog code to existing crates and regenerates `src/generated/session-types.ts` via the existing `write_generated_typescript_contract()` test/build step.

**Version verification:** All listed crates already pinned in `src-tauri/Cargo.toml` (`rg` confirmed versions above). No `npm view`/`cargo search` needed — no new packages.

## Package Legitimacy Audit

> This phase installs **no external packages**. All implementation reuses existing Rust crates (`serde`, `ts-rs`, `rosc`, `thiserror`), the existing `zod` npm dep, and the existing stdlib-only Python tooling. The Package Legitimacy Gate is therefore trivially satisfied — no new registry lookups, no `[SLOP]`/`[SUS]` risk surface.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| (none new this phase) | — | — | — | — | N/A | N/A |

**Packages removed due to [SLOP] verdict:** none (none proposed).
**Packages flagged as suspicious [SUS]:** none.

*All crates referenced are already locked in `Cargo.toml` and verified present via `rg`. No `[ASSUMED]` package claims are introduced.*

## Architecture Patterns

### System Architecture Diagram

Data flow for the catalog-driven audio pipeline (the primary use case: a performer adds an LFO and routes it to a filter's cutoff CV port, then plays a step sequencer):

```
 [Performer / Agent]
        │  (typed commands via GraphEditCommand — session.rs:473)
        ▼
 [SessionStore] ── owns ──▶ [SessionDocument]  (canonical: nodes w/ node_type_id, ports, params, routes, 16-step patterns)
        │                          │
        │ reconcile_graph_edit     │ compile_session_to_topology (compiler.rs:93)
        ▼                          ▼
 [AudioRuntimeManager]      ┌─ CATALOG (const table) ◀── single source of truth ──┐
   runtime_manager.rs:174   │  NodeCatalogEntry { id, synthdef, ports, params,     │
        │                   │    sc_arg, exposes_cv_port, visual_shape, category } │
        │                   └─ drives ────────────────────────────────────────────┘
        ▼                            │           │            │           │
 [Topology Compiler] ◀──────────── lookup   port-type     param→sc_arg  visual_shape
   compiler.rs:93                   │         validate      normalize     (Phase 15)
        │                           │
        │  allocates AUDIO buses (synthdefs.rs:246) + NEW CONTROL buses (CV)
        ▼
 [ScResourcePlan] ── synthdefs + groups + synths + args + controls + cv-bus-map
        │
        ├──▶ [SuperColliderAdapter] (/d_recv synthdefs, /s_new synths w/ bus args)
        │      supercollider.rs:188                                  │
        │                                                            ▼
        │                                          [scsynth process] (dumb executor)
        │                                          In.kr(cv_bus) → param + cv
        │                                          Out.kr(out_cv_bus) ← LFO/Env
        │                                          In.kr(gate_bus) ← sequencer target
        │                                          ▲
        │                                          │ /c_set gate_bus, cv_bus  (per step)
        └──▶ [SequencerController] ── transport tick (Rust-owned, D-06) ─┘
               advances 16 steps against TransportState (session.rs:161)
```

A reader can trace: command → canonical mutation → topology compile (catalog lookup) → bus allocation → SC synth launch with CV args → live `/c_set` from the sequencer → audible modulation.

### Recommended Project Structure
```
src-tauri/src/
├── domain/
│   └── session.rs         # Node loses closed enums; gains node_type_id + NodeCategory + sequencer pattern data
├── catalog/               # NEW module — the single source of truth
│   ├── mod.rs             # NodeCatalogEntry, CatalogPortSpec, CatalogParamSpec, NodeCategory, CATALOG const, lookup fn
│   └── entries.rs         # the ~16 catalog entries (oscillator, filter, lfo, env, vca, noise, quantizer,
│                          #   mixer, delay, reverb, distortion, chorus, step_sequencer, output, ...)
├── audio/
│   ├── compiler.rs        # compile_session_to_topology — catalog-driven node_kind, control-bus allocation
│   └── synthdefs.rs       # plan_sc_resources — catalog-driven dispatch; unreachable!() → Err; synthdef_resource → catalog
└── persistence/
    └── session_file.rs    # two-phase parse; friendly v1-reject message
src-tauri/resources/synthdefs/
├── v1/                    # existing 6 .scsyndef (kept for traceability; not loaded by v2 catalog)
└── v2/                    # NEW — generated .scsyndef for the v2 catalog entries
    └── generate_synthdefs.py  # extended (move/copy from v1/) — emits v2 defs incl. CV-bus args
src/components/audio/
└── PrimitivePalette.tsx   # catalog-driven (iterate NodeCatalogEntry[] instead of hardcoded buttons)
src/lib/
└── session-client.ts      # Zod schemas relaxed: node_type_id: z.string(), catalog-driven ports/params
```

### Pattern 1: Catalog-Driven Dispatch (replaces the 3 allowlists)
**What:** One `&'static [NodeCatalogEntry]` table is the authority for synthdef name, SC arg names, ports, visual shape, and node existence. Every former `match node_kind { ... }` becomes a `find_catalog_entry(&node.node_type_id)?` lookup.
**When to use:** Everywhere the v1 code dispatched on a closed enum (`plan_sc_resources` :116, `node_sort_key` :418, `visual/compiler.rs:57`, `synthdef_resource` :429, `validate_runtime_target` :284, `normalize_parameter_name` :407).
**Example:**
```rust
// Source: recommended replacement for synthdefs.rs:116 + :429 + :407 + :284
pub fn find_catalog_entry(node_type_id: &str) -> Result<&'static NodeCatalogEntry, ScResourcePlanError> {
    CATALOG.iter().find(|e| e.id == node_type_id).ok_or_else(|| ScResourcePlanError::UnknownCatalogEntry {
        node_type_id: node_type_id.to_string(),
    })
}
// plan_sc_resources body becomes:
let entry = find_catalog_entry(&node.node_type_id)?;       // replaces match &node.node_kind
let synthdef_name = entry.synthdef_name;                    // replaces SOURCE_OSCILLATOR_SYNTHDEF etc.
// args: each parameter → entry.parameters[idx].sc_arg      // replaces normalize_parameter_name
```

### Pattern 2: Control-Bus Allocation alongside Audio-Bus Allocation
**What:** The compiler allocates audio buses (as today, `plan_buses` :246) AND control buses for CV routes. Control buses live in a separate index range to avoid collision with audio buses (scsynth bus indices are shared; rate is determined by `In.ar` vs `In.kr`). `[CITED: SuperCollider bus model — audio/control share index space, rate chosen by UGen]`
**When to use:** Any route whose target port `signal_type == Control` (a CV-input port declared in the catalog).
**Example:**
```rust
// Source: recommended extension to plan_buses (synthdefs.rs:246)
pub const FIRST_CONTROL_BUS_OFFSET: u32 = 1024; // high range, clear of audio buses
// After allocating audio buses, walk CV routes:
//   for each Route targeting a Control-type port: allocate 1 control bus index,
//   record (route_id → control_bus_index) in a new CvBusMap.
// The mod-source node's launch gets arg out_cv_bus = index;
// the target node's launch gets arg <param>_cv_bus = index.
```

### Pattern 3: App-Driven Sequencer via `/c_set` (D-06)
**What:** Rust owns the transport tick and step advance. Per step, it sends SC `/c_set [bus, value]` for the gate bus and CV bus allocated to the sequencer's output ports. SC target nodes read those buses with `In.kr`. No SC-side sequencer synth.
**When to use:** The step-sequencer node (D-07 mono, D-08 16 steps).
**Example:**
```rust
// Source: recommended SequencerController sketch (new — no v1 equivalent)
// Spawned when audio runtime is Ready and transport is_playing.
// On each 16th-note tick (60.0 / bpm / 4.0 seconds):
let step = pattern.steps[current_step]; // { gate: bool, cv: f64 }
osc.send_message("/c_set", vec![
    OscType::Int(gate_bus_index), OscType::Float(if step.gate { 1.0 } else { 0.0 }),
    OscType::Int(cv_bus_index),   OscType::Float(step.cv as f32),
])?;
current_step = (current_step + 1) % 16;
```

### Anti-Patterns to Avoid
- **Hand-rolling a second `match` for new node kinds** — that recreates the v1 problem the catalog exists to solve. Every new node must be *data* in `CATALOG`, never a new `match` arm. `[VERIFIED: success criterion #3]`
- **Surfacing control buses as canonical `Bus` records in `SessionDocument`** — CV wiring is a compiler/adapter concern; routes + ports are the canonical truth. Adding control buses to `session.buses` duplicates derived state and complicates persistence. Keep them a compiler artifact. (Planner may revisit if Phase 13 needs named CV buses, but default: keep canonical clean.)
- **Letting serde fail before the v1-version check** — if the domain model changes, `serde_json::from_str::<SessionDocument>` on a v1 file fails cryptically *before* `session_file.rs:59` runs. Must two-phase parse (read `schemaVersion` first). See **Common Pitfalls #1**.
- **Audio-rate CV via control buses (or vice versa)** — `Out.kr` into a bus read by `In.ar` (or the reverse) is a classic SC silent-failure. The catalog port `signal_type` must drive both the bus allocation rate AND the matching `Out.<rate>`/`In.<rate>` UGen choice. `[CITED: SuperCollider rate-mismatch behavior]`
- **Modulating discrete selectors / toggles** — D-05 explicitly excludes these from CV ports. The catalog's `exposes_cv_port` flag must be false for `wave_shape`, `noise_color`, `bypass`, etc.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SynthDef byte serialization | A new SC byte writer | `generate_synthdefs.py` `SynthDefBuilder` + `synthdef_bytes()` | Already byte-accurate (`audio_runtime.rs:433` verifies SCgf v2 headers); sclang-free; deterministic. Extending it is ~50 lines/def. `[VERIFIED: generate_synthdefs.py:351]` |
| Type export Rust→TS | Manual TS types for the catalog | `ts-rs` `#[derive(TS)]` + the `write_generated_typescript_contract()` list | Already the pipeline (session.rs:806); catalog join is one line per type. `[VERIFIED: session.rs:808-875]` |
| Runtime TS validation | Re-implementing the catalog in Zod by hand | Regenerate from ts-rs output; relax closed enums to catalog-driven strings | The hand-mirror in `session-client.ts:70-114` already drifts; the catalog makes it data-driven. `[VERIFIED: session-client.ts:80-113]` |
| Topo-sort / launch ordering | A new graph algorithm | The existing `topo_sort_nodes` (compiler.rs:261) | Already deterministic and tested; the refactor changes *what* each node maps to, not the ordering. `[VERIFIED: compiler.rs:261]` |
| SC OSC sync/cleanup | New sync logic | The existing `sync_scsynth` + `free_created_resources` (supercollider.rs:368, :334) | Already handles `/sync` timeouts + rollback; `/c_set` is fire-and-forget (no sync needed). `[VERIFIED: supercollounder.rs:368]` |
| Sequencer transport math | A new timing core | `TransportState` (session.rs:161) + step = `position_beats` mapping | Transport is already app-owned; 16 steps = 16th notes over 1 bar (4/4). `[VERIFIED: session.rs:159-175]` |

**Key insight:** v1 already built the hard infrastructure (topology compiler, OSC sync/cleanup, ts-rs pipeline, deterministic SynthDef byte-writer, friendly typed-error enums). Phase 12 is overwhelmingly a *content* refactor (replace dispatch contents, add catalog entries, extend the Python writer, add control-bus allocation) — not new infrastructure. The planner should resist re-architecting the pipeline.

## Runtime State Inventory

> Phase 12 involves a schema bump + domain-model reshape (rename/migration-class change). D-09 permits a clean break — but the inventory still must be answered so the plan addresses every runtime system that holds v1 identity.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | v1 session `.json` files on disk (`schemaVersion: 1`, closed-enum `audioPrimitive`). User's local `~/...scrysynth*.json` saves. `[VERIFIED: session_file.rs:59 checks schema_version]` | **No migration** (D-09 clean break). Friendly reject on open (D-10). Bump `CURRENT_SCHEMA_VERSION` 1→2. |
| Live service config | `scsynth` process holds an ephemeral `ScResourcePlan` (active patch) in memory only — no persistent config, no node-type registry. `[VERIFIED: supercollounter.rs:29 active_patch: Option<ScResourcePlan>, dropped on stop]` | None — patch is rebuilt from canonical state on every audio start. |
| OS-registered state | None. No pm2/launchd/Task-Scheduler entries reference node types. No global installs. `[VERIFIED: project is a Tauri desktop app; STATE.md lists no OS registrations]` | None. |
| Secrets/env vars | `SCRYSYNTH_SCSYNTH_PATH` (supercollider.rs:14) — points to the scsynth binary; unaffected by catalog rename. No node-type-keyed env vars. `[VERIFIED: supercollounter.rs:14]` | None — env var unchanged. |
| Build artifacts | `src-tauri/resources/synthdefs/v1/*.scsyndef` (6 binaries) + `generate_synthdefs.py` outputs to `v1/` dir. The v2 catalog points to `v2/` paths. `src/generated/session-types.ts` (checked-in ts-rs output) will be regenerated. `[VERIFIED: glob — only v1/ dir exists; session.rs:8 GENERATED_TYPES_PATH]` | Generate new `v2/*.scsyndef` via extended Python script; regenerate `session-types.ts`. Old `v1/` files can remain (harmless; not loaded by v2 catalog) or be archived. |

**The canonical question — "After every repo file is updated, what runtime systems still hold the old string?"**: Only on-disk v1 session JSON files, and D-10 explicitly requires rejecting those with a friendly message rather than loading them. scsynth is stateless across restarts. No other runtime system caches node identity.

## Common Pitfalls

### Pitfall 1: Serde failure preempts the friendly v1 message
**What goes wrong:** After the domain model changes (e.g. `AudioPrimitive` removed/reshaped), opening a v1 file calls `serde_json::from_str::<SessionDocument>` (session_file.rs:57) which fails with a cryptic "missing field" error *before* the schema-version check at :59 runs. The user sees a serde parse error, not the friendly "v1 unsupported" message D-10 requires.
**Why it happens:** The current code deserializes the full document, then checks version. Any model drift makes serde fail first.
**How to avoid:** Two-phase parse — first `serde_json::from_str::<SchemaVersionProbe>(&contents)` on a tiny struct `{ #[serde(rename="schemaVersion")] schema_version: u32 }`, check the version, THEN (only if version matches) deserialize the full `SessionDocument`. Emit the friendly `UnsupportedSchemaVersion` (or a new `LegacyV1Session`) error from the first phase.
**Warning signs:** A v1 file open shows "failed to deserialize session JSON: missing field `audioPrimitive`" instead of the v1 message.

### Pitfall 2: Audio-bus / control-bus index collision
**What goes wrong:** scsynth bus indices are a single shared space; audio buses and control buses are distinguished only by whether you read them with `In.ar`/`Out.ar` or `In.kr`/`Out.kr`. If control buses are allocated from the same low offset as audio buses (`HARDWARE_AUDIO_BUS_OFFSET = 2`), a control write can clobber an audio signal (or vice versa). `[CITED: SuperCollider bus model]`
**Why it happens:** v1 only ever allocated audio buses (`plan_buses` :246), so there was no collision surface. Adding CV wiring introduces a second bus population.
**How to avoid:** Allocate control buses from a clearly separated high range (e.g. `FIRST_CONTROL_BUS_OFFSET = 1024`, well above any plausible audio-bus count). Document the split in a comment.
**Warning signs:** Modulating a filter cutoff audibly clicks/distorts the audio path; or a CV write is silent because the target reads the wrong rate.

### Pitfall 3: Rate mismatch on CV (Out.kr into In.ar or reverse)
**What goes wrong:** A mod source writes `Out.kr(bus, sig)` but the target reads `In.ar(bus)` (or the reverse). SC does not error — it silently produces wrong-length buffers. Audio-rate modulation into a `.kr` read downsamples; control-rate into `.ar` reads garbage. `[CITED: SuperCollider rate semantics]`
**Why it happens:** The catalog port `signal_type` (Audio vs Control) must drive BOTH the bus-rate allocation AND the SynthDef's `In.<rate>` UGen. If they disagree, silent failure.
**How to avoid:** Make the catalog `CatalogPortSpec.signal_type` the single authority; the compiler allocates the bus at the matching rate and the SynthDef uses the matching `In.ar`/`In.kr`. Add a conformance check (unit test) that asserts every CV-input port's SynthDef reads the bus at the declared rate.
**Warning signs:** Modulation is silent or produces artifacts only at certain rates.

### Pitfall 4: `unreachable!()` lives on a non-Result return path
**What goes wrong:** Success criterion #3 requires the `synthdef_resource()` panic at `synthdefs.rs:455` to become a real `Err`. But `synthdef_resource()` currently returns `SynthDefResource` (not `Result`), and its caller at `:232` is `.map(synthdef_resource).collect()` (expects an `FnMut -> SynthDefResource`, not `-> Result<...>`).
**Why it happens:** The signature itself must change to propagate the error.
**How to avoid:** Change `synthdef_resource(name) -> Result<SynthDefResource, ScResourcePlanError>` and switch the caller to `iter().map(|n| find_catalog_entry(n)).collect::<Result<Vec<_>,_>>()?`. Add an `UnknownCatalogEntry` variant to `ScResourcePlanError` (synthdefs.rs:62). In the catalog-driven world this whole function likely collapses into `entry.synthdef_resource` field access — but the *panic removal* must be explicit and verified by a test that an unknown id returns `Err`, not panics.

### Pitfall 5: Frontend Zod schemas are a hand-maintained mirror that drifts
**What goes wrong:** `session-client.ts:70-114` hand-codes Zod schemas with closed enums (`sourceType: z.enum(["oscillator","noise"])`, `effectType: z.enum(["low_pass_filter","delay"])`). After the catalog refactor these are wrong and every `invokeSession` call's `.parse(payload)` throws, breaking the whole UI.
**Why it happens:** The Zod schemas are not generated from ts-rs; they're a parallel hand-maintained contract.
**How to avoid:** Treat the `session-client.ts` Zod block as a first-class deliverable of this phase: relax closed enums to `z.string()` (or to `z.enum(...)` generated from the catalog), regenerate to match the new `Node`/`NodeCatalogEntry` shape. Add a test (`session-client.test.ts` exists — extend it) that round-trips a v2 catalog-derived session.
**Warning signs:** UI loads but every node/parameter operation throws a Zod parse error.

### Pitfall 6: Sequencer tick drift / no periodic driver
**What goes wrong:** The app-driven sequencer (D-06) needs a periodic tick to advance steps, but the audio layer is fully synchronous (`AudioRuntimeManager` has no event loop; `SessionStore.poll_hardware_events` is the only periodic entry point and it's hardware-driven). If the planner assumes a tokio interval exists, it won't compile — `tokio` is not a dependency. `[VERIFIED: Cargo.toml — tokio absent]`
**Why it happens:** STACK.md *recommends* tokio 1.51.1, but v1 never adopted it; the audio layer uses std::thread + UdpSocket synchronously.
**How to avoid:** Explicitly decide the tick mechanism in the plan: either (a) add `tokio` and spawn an interval task, (b) spawn a `std::thread::spawn` loop with `thread::sleep` keyed to tempo, or (c) piggyback on the existing poll path. Recommend (b) for minimal surface change — matches the existing sync architecture. The OSC `/c_set` send needs access to the adapter's `ScOscClient`; expose a `tick_transport(...)` method on `AudioRuntimeAdapter` or have the controller hold an OSC handle.
**Warning signs:** Sequencer steps don't advance, or advance only when hardware events arrive.

## Code Examples

Verified patterns from the v1 source (the templates the planner extends):

### Existing audio-bus allocation (the template for control-bus allocation)
```rust
// Source: src-tauri/src/audio/synthdefs.rs:246 (VERIFIED)
fn plan_buses(topology: &CompiledTopology) -> Result<BTreeMap<String, f32>, ScResourcePlanError> {
    let mut bus_map = BTreeMap::new();
    for bus in &topology.buses {
        if bus.channels == 0 { return Err(ScResourcePlanError::InvalidBusChannels { bus_id: bus.bus_id.clone() }); }
        if bus_map.insert(bus.bus_id.clone(), (HARDWARE_AUDIO_BUS_OFFSET + bus.index) as f32).is_some() {
            return Err(ScResourcePlanError::DuplicateBus { bus_id: bus.bus_id.clone() });
        }
    }
    Ok(bus_map)
}
// CV wiring extends this with a second pass: walk CV routes, allocate control buses
// from FIRST_CONTROL_BUS_OFFSET, populate a CvBusMap { route_id -> (bus_index, rate) }.
```

### Existing SC arg binding + live `/n_set` (works unchanged for CV-modulated base params)
```rust
// Source: src-tauri/src/audio/synthdefs.rs:382 (VERIFIED — apply_parameters)
// Each parameter pushes an ScSynthArg named by normalize_parameter_name() and records
// an ScControlPlan { control_key: "{node_id}:{param_id}", synth_node_id, parameter_name }.
// The live-update path (supercollider.rs:286 send_live_parameter) sends /n_set on the
// synth_node_id with parameter_name. For a CV-modulated param, the BASE value is still
// set via /n_set; the CV is read inside the synth graph via In.kr(<param>_cv_bus).
// => The existing live-parameter path needs NO change for CV. Only the SynthDef graph changes.
```

### Existing SC SynthDef with an input bus read (template for CV-input args)
```rust
// Source: src-tauri/resources/synthdefs/v1/generate_synthdefs.py:158 (lowpass, VERIFIED)
// v1 lowpass reads audio in:  In.ar(in_bus, 2) -> RLPF -> Out.ar(out_bus, ...)
// v2 filter with cutoff CV adds a control-bus read:
//     In.kr(cutoff_cv_bus) summed into cutoff_hz before RLPF.
//   args gain: cutoff_cv_bus = -1  (default -1 = no modulation)
//   graph:     effective_cutoff = cutoff_hz + (In.kr(cutoff_cv_bus) * (cutoff_cv_bus >= 0))
// Python writer extension (sketch):
//   cutoff_cv_bus, = builder.controls([("cutoff_cv_bus", -1.0)])
//   cv_in = builder.ugen("In", RATE_CONTROL, [cutoff_cv_bus])[0]
//   active = builder.ugen(">= ", ...) // or clip gate; planner chooses exact math
//   eff_cutoff = builder.mul_add(RATE_CONTROL, cv_in, clipped_cutoff) // base + cv
```

### Existing v1-version check (the hook D-10 extends)
```rust
// Source: src-tauri/src/persistence/session_file.rs:59 (VERIFIED)
if session.schema_version != CURRENT_SCHEMA_VERSION {
    return Err(SessionFileError::UnsupportedSchemaVersion { expected: CURRENT_SCHEMA_VERSION, found: session.schema_version });
}
// D-10 work: (1) bump CURRENT_SCHEMA_VERSION (session.rs:9) 1 -> 2,
//            (2) two-phase parse so serde doesn't fail first (Pitfall #1),
//            (3) friendlier message variant for the v1->v2 case.
```

### Existing conformance test (the template for "boots real scsynth per entry")
```rust
// Source: src-tauri/tests/audio_runtime.rs:433 (checked_in_v1_synthdef_resources_are_present_and_named, VERIFIED)
// v1 checks the SCgf header + embedded name of each checked-in .scsyndef WITHOUT booting scsynth.
// The Phase 12 success criterion #4 ("boots real scsynth for every entry") goes further:
//   - resolve scsynth (resolve_scsynth_executable pattern, supercollounter.rs:459)
//   - boot it (SuperColliderAdapter::start)
//   - for each CATALOG entry: /d_recv its synthdef bytes, assert /done, /d_free
//   - must be GATED on scsynth availability (skip with clear message if absent) —
//     scsynth is NOT on PATH locally; bundle fallback is present.
//   - recommend #[ignore] by default + run via cargo test -- --ignored or a CI job with scsynth.
```

## State of the Art

| Old Approach (v1) | Current Approach (v2 Phase 12) | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Closed-enum node identity (`NodeType`/`AudioSourceType`/`AudioEffectType`/`AudioPrimitive`) | String `node_type_id` + catalog lookup + `NodeCategory` tag | Phase 12 | Adding a node = adding a catalog entry, not editing 5 enums + 3 allowlists |
| Three hardcoded allowlists (`synthdef_resource`, `normalize_parameter_name`, `validate_runtime_target`) | One `NodeCatalogEntry` table drives all three concerns | Phase 12 | Single source of truth (success criterion #1, #3) |
| `unreachable!()` panic on unknown synthdef (`synthdefs.rs:455`) | `Err(ScResourcePlanError::UnknownCatalogEntry)` | Phase 12 | Success criterion #3; testable |
| Audio-bus-only allocation (`plan_buses` :246) | Audio + control bus allocation | Phase 12 | Enables end-to-end CV modulation (NODES-05) |
| Hardcoded `PrimitivePalette` 3-button factory | Catalog-driven palette iterating `NodeCatalogEntry[]` | Phase 12 | Success criterion #4; fixes the `runtimeTarget: audio/source/${id}` quirk |
| `schemaVersion: 1` + generic unsupported-version error | `schemaVersion: 2` + friendly v1-reject (D-10) | Phase 12 | Clean break (D-09) |
| `NodeType`→shape dispatch (`visual/compiler.rs:57`) | `NodeCatalogEntry.visual_shape` field | Phase 12 (unblocks Phase 15) | Catalog drives visual compiler too |

**Deprecated/outdated:**
- The closed `NodeType`/`AudioSourceType`/`AudioEffectType` enums and the `AudioPrimitive` tag-enum (session.rs:195-239): superseded. D-09 means they can be removed/replaced freely without migration. `[VERIFIED: session.rs:195-262]`
- `CompiledNodeKind` (compiler.rs:51): the closed enum threaded through the compiler — superseded by a catalog-derived compiled node representation.
- The v1 `runtime_target` string-per-node quirk (`audio/source/${id}` using node id in `PrimitivePalette.tsx:84`): replaced by `node_type_id` as the runtime target identity.

## The Five Hardcoded Spots (the heart of the refactor)

All five must become catalog-driven. Verified locations:

| # | Location | What it does today | Catalog replacement |
|---|----------|-------------------|---------------------|
| 1 | `synthdefs.rs:429` `synthdef_resource()` + `:455` `unreachable!()` | synthdef name → `.scsyndef` path; panics on unknown | `entry.synthdef_resource` field; `find_catalog_entry()` returns `Err` |
| 2 | `synthdefs.rs:407` `normalize_parameter_name()` | parameter alias → canonical SC arg | `entry.parameters[i].sc_arg` + `.aliases` |
| 3 | `synthdefs.rs:284` `validate_runtime_target()` | validates runtime-target strings per node kind | collapse: `node_type_id` IS the runtime target; validation = "is it in the catalog?" |
| 4 | `synthdefs.rs:116` `match &node.node_kind` (in `plan_sc_resources`) | picks synthdef per `CompiledNodeKind` | `find_catalog_entry(&node.node_type_id)?.synthdef_name` |
| 5 | `visual/compiler.rs:57` `match node.node_type { Source=>sphere... }` | node category → visual shape | `entry.visual_shape` field |

Plus two enum-dispatch helpers that also need catalog-driven category logic:
- `compiler.rs:418` `node_sort_key()` — dispatches on `NodeType` for topo-sort ranking; use `entry.category` rank.
- `compiler.rs:335` `compile_node_launch()` — matches `AudioPrimitive` to build `CompiledNodeKind`; becomes catalog-driven.

`[VERIFIED: all five line numbers read directly]`

## Catalog Data Model (recommended `NodeCatalogEntry` shape)

```rust
// Source: recommended — no v1 equivalent (this is the new single source of truth)
use crate::domain::session::{PortDirection, SignalType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeCategory { Source, Modulator, Effect, Utility, Sequencer, Mixer, Output }
// Replaces NodeType (session.rs:195) as a SEMANTIC tag (sort order, visual grouping),
// NOT as the closed identity enum. Node identity is the string node_type_id.

#[derive(Clone, Copy, Debug)]
pub struct CatalogPortSpec {
    pub id: &'static str,          // "audio_out", "cutoff_cv", "gate_out", "cv_out"
    pub name: &'static str,        // display name
    pub direction: PortDirection,  // Input / Output (session.rs:281 — reuse existing enum)
    pub signal_type: SignalType,   // Audio / Control (session.rs:288 — reuse existing enum)
}

#[derive(Clone, Copy, Debug)]
pub struct CatalogParamSpec {
    pub id: &'static str,          // "frequency", "cutoff", "wave_shape"
    pub sc_arg: &'static str,      // "frequency", "cutoff_hz" — replaces normalize_parameter_name
    pub aliases: &'static [&'static str], // ["freq"] — backward-compat; optional
    pub default_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub unit: &'static str,
    pub exposes_cv_port: bool,     // D-05: true for continuous, false for selectors/toggles
    pub cv_port_id: Option<&'static str>, // "cutoff_cv" when exposes_cv_port (D-04)
}

#[derive(Clone, Copy, Debug)]
pub struct NodeCatalogEntry {
    pub id: &'static str,                  // "oscillator", "filter", "lfo", "step_sequencer" — the node_type_id
    pub display_name: &'static str,        // "Oscillator"
    pub category: NodeCategory,            // sort/shape semantics
    pub synthdef_name: &'static str,       // "scrysynth_v2_oscillator"
    pub synthdef_resource: &'static str,   // "resources/synthdefs/v2/scrysynth_v2_oscillator.scsyndef"
    pub ports: &'static [CatalogPortSpec], // audio_in/out + per-param CV ports (D-04/D-05)
    pub parameters: &'static [CatalogParamSpec],
    pub visual_shape: &'static str,        // "sphere"/"box"/"ring"/"plane" — drives visual/compiler.rs (Phase 15)
}

pub const CATALOG: &[NodeCatalogEntry] = &[ /* oscillator, noise, filter, lfo, envelope, vca, quantizer,
                                              mixer, delay, reverb, distortion, chorus, flanger,
                                              step_sequencer, output, ... */ ];

pub fn find_catalog_entry(id: &str) -> Option<&'static NodeCatalogEntry> {
    CATALOG.iter().find(|e| e.id == id)
}
```

**Domain model change (D-09 clean break):** `Node` (session.rs:179) loses `node_type: NodeType` and `audio_primitive: Option<AudioPrimitive>`; gains `node_type_id: String`. `AudioPrimitive`/`AudioSourceNode`/`AudioEffectNode`/`AudioMixerNode`/`AudioOutputNode`/`AudioSourceType`/`AudioEffectType`/`AudioOutputType` are removed or collapsed. Per-node config that isn't catalog-derived (channel_mode, bus_target_id, sequencer pattern) moves to a small tagged `NodeData` payload or flat optional fields. `[VERIFIED: session.rs:179-262]`

## New SynthDefs Needed (D-02 DSP-fit rule applied)

Per D-01/D-02: param-driven where one SynthDef holds the variants; separate entries only where DSP graphs fundamentally differ. All authored by extending `generate_synthdefs.py`.

| Family | Catalog node(s) | SynthDef(s) | New UGens vs v1 | CV ports (D-04/D-05) |
|--------|-----------------|-------------|-----------------|----------------------|
| Oscillator | 1: `oscillator` | 1 (extend v1: add triangle via `LFTri`, param `wave_shape` 0-3) | `LFTri` (triangle) | `frequency_cv`, `level_cv` |
| Noise | 1: `noise` | 1 (v1 has it: white/pink via `noise_color`) | none | `level_cv` |
| Filter | 1: `filter` (param `filter_mode` LPF/HPF/BPF/BRF) | 1 (D-02: `*LPF` family — `LPF`/`HPF`/`BPF`/`BRF` UGens selected by param) | `LPF`,`HPF`,`BPF`,`BRF` (v1 used `RLPF` only) | `cutoff_cv`, `resonance_cv` |
| Envelope | 1: `envelope` (control-rate source) | 1 (`EnvGen` + `Done`/`FreeSelf`) | `EnvGen`,`Done`,`FreeSelfWhenDone` | env_out (control), `gate` (trigger input) |
| LFO | 1: `lfo` (control-rate source) | 1 (`LFCub`/`LFSaw`/`LFPulse`/`LFTri` by `wave_shape`) | `LFCub`,`LFSaw`,`LFPulse`,`LFTri` | lfo_out (control), `frequency_cv` |
| VCA | 1: `vca` | 1 (`* audio_in, level`) | none (MulAdd) | `level_cv`, audio_in/out |
| Quantizer | 1: `quantizer` | 1 (`Round`/`Snap` to scale) | `Round` or scalar quantize | cv_in/out (control) |
| Mixer | 1: `mixer` | 1 (v1 has it: 8-input `Sum4`) | none | per-channel `level_cv` (optional) |
| Delay | 1: `delay` | 1 (v1 has it) | none | `delay_time_cv`, `feedback_cv` |
| Reverb | 1: `reverb` (NEW) | 1 (`FreeVerb` or `GVerb`) | `FreeVerb`/`GVerb` | `mix_cv`, `room_cv` |
| Distortion | 1: `distortion` (NEW) | 1 (`tanh`/`Clip` waveshaper) | `tanh` via `MulAdd`/`Distortion` | `drive_cv`, `mix_cv` |
| Chorus | 1: `chorus` (NEW) | 1 (`CombC` modulated) | `CombC` | `depth_cv`, `rate_cv` |
| Flanger | 1: `flanger` (NEW) | 1 (`CombC`+feedback) | `AllpassN` | `depth_cv`, `rate_cv`, `feedback_cv` |
| Step Sequencer | 1: `step_sequencer` (D-06/D-07/D-08) | **0** (app-driven; SC has no sequencer synth) | none | `gate_out`, `cv_out` (both control) |
| Output | 1: `output` | 1 (v1 has it) | none | `level_cv` |

Total: ~14 catalog entries, ~13 new/extended SynthDefs. v1's 6 defs fold in largely unchanged (oscillator/noise/filter-from-lowpass/delay/mixer/output) — the refactor is about the *dispatch mechanism*, not redesigning those. `[VERIFIED: v1 defs in scrysynth_v1_synthdefs.scd + generate_synthdefs.py:118-325]`

**Triangle waveform note:** v1 oscillator supports sine/saw/square only (`Select.ar(wave_shape.clip(0,2), ...)` — `.scd:8`, `generate_synthdefs.py:129`). D-01 mentions sine/saw/square/triangle → extend `wave_shape` to 0-3 and add `LFTri`. `[VERIFIED: v1 oscillator supports 3 waveforms]`

## CV/Modulation Wiring (D-03/D-04/D-05) — the biggest new capability

**End-to-end path (LFO → filter cutoff, control-rate):**
1. Performer adds an LFO node and a Filter node; draws edge LFO `lfo_out` (Control) → Filter `cutoff_cv` (Control). `[canonical: Route in SessionDocument]`
2. Compiler sees a Route whose target port `signal_type == Control`; allocates a control bus from `FIRST_CONTROL_BUS_OFFSET`; records `route_id → control_bus_index`. `[compiler artifact, NOT canonical]`
3. LFO launch gets arg `out_cv_bus = index`; its SynthDef does `Out.kr(out_cv_bus, lfo_signal)`.
4. Filter launch gets arg `cutoff_cv_bus = index`; its SynthDef does `effective_cutoff = cutoff_hz + In.kr(cutoff_cv_bus)` (gated when `cutoff_cv_bus >= 0`).
5. SC executes; cutoff sweeps audibly. ✓ (D-03)

**Control-rate vs audio-rate (D-03 requires both):**
- **Control-rate** (LFO/Env/Sequencer → param): `Out.kr` → `In.kr`. Bus is mono (1 channel). This is the common case.
- **Audio-rate** (e.g. oscillator FM another oscillator): `Out.ar` → `In.ar` on a 1-channel audio bus, summed into the target arg. Audio-rate CV ports are `signal_type: Audio`. The wiring is identical except for the bus rate and the `In.<rate>` UGen. **Recommendation:** support control-rate fully this phase (it covers LFO/Env/Sequencer — the named mod sources); treat audio-rate as supported-but-minimal (one validated path, e.g. oscillator→oscillator frequency) since DSP-fit nodes for audio-rate modulation are fewer. Flag for planner: the catalog port `signal_type` is the single authority for which rate.

**Sequencer → target (ties to NODES-04):** The sequencer's `gate_out`/`cv_out` ports are Control-type. The compiler allocates a gate bus and a CV bus for them (same mechanism). The Rust transport-tick loop writes per-step values via `/c_set` (Pattern 3). Target nodes (e.g. an envelope's `gate`, or a VCA's `level_cv`) read via `In.kr`. `[CITED: SuperCollider /c_set + In.kr]`

**Live parameter updates (unchanged):** `send_live_parameter` (supercollider.rs:286) sends `/n_set` on the synth's base param arg. CV modulation is additive inside the synth graph, so the existing live-update path works unchanged — the base `cutoff_hz` is still `/n_set`-able while the CV bus sweeps on top. `[VERIFIED: supercollounter.rs:286-332]`

## Step Sequencer (App-Driven) — D-06/D-07/D-08

**Canonical state (16-step pattern):** Add to the `Node` model (D-09 clean break allows it). Recommended:
```rust
pub struct SequencerPattern {
    pub gate: [bool; 16],   // D-07: one gate per step
    pub cv: [f64; 16],      // D-07: one CV per step
}
```
Stored on the sequencer `Node` (via a `NodeData::Sequencer(SequencerPattern)` payload or flat optional field). Mutations flow through `GraphEditCommand` (session.rs:473) — add a new command variant `SetStepValue { node_id, step_index, gate: Option<bool>, cv: Option<f64> }` (or reuse `SetParameterValue` with step-indexed param ids). `[VERIFIED: GraphEditCommand at session.rs:473]`

**Transport tick (D-06):**
- Transport is app-owned: `TransportState { tempo_bpm, is_playing, position_beats }` (session.rs:161). `[VERIFIED]`
- Step advance: 16 steps = one bar of 16th notes (4/4). Step boundary = every `60.0 / bpm / 4.0` seconds. Current step = `floor(position_beats / 0.25) % 16` (16th note = 0.25 beats). `[ASSUMED — clock division is planner discretion per CONTEXT.md]`
- Per step: send `/c_set [gate_bus, gate?]` + `/c_set [cv_bus, cv]` for each sequencer's allocated gate/CV bus.
- **Periodic driver (Pitfall #6):** `tokio` is NOT a dependency. Options: (a) add tokio + interval task, (b) `std::thread::spawn` + `thread::sleep` loop (recommended — matches sync audio layer), (c) piggyback on `poll_hardware_events` (session_store.rs:220). Recommend (b): a dedicated `SequencerController` thread spawned when audio reaches Ready + transport plays, killed on stop/panic. It holds an OSC handle (or calls a new `AudioRuntimeAdapter::tick_transport()` method) to send `/c_set`. `[VERIFIED: tokio absent; audio layer sync]`

**SC side:** No sequencer SynthDef (D-06 "SC stays dumb"). Target nodes simply `In.kr(gate_bus)` / `In.kr(cv_bus)`. The gate can drive an envelope's `gate` arg; the CV can drive a VCA level or oscillator frequency.

## Frontend Catalog Consumption

**`PrimitivePalette.tsx` (src/components/audio/PrimitivePalette.tsx:62):** The hardcoded `buildPrimitiveNode()` factory (3 buttons, hardcoded ports/params) becomes a catalog iterator:
```ts
// iterate NodeCatalogEntry[] (from generated session-types.ts)
// one button per entry; build Node { node_type_id: entry.id, ports: entry.ports, parameters: entry.parameters (defaults), ... }
// runtimeTarget = entry.id  (FIXES the v1 `audio/source/${id}` quirk at PrimitivePalette.tsx:84)
```
`[VERIFIED: PrimitivePalette.tsx:62-152]`

**`NodeInspector.tsx` (src/components/session/NodeInspector.tsx:100):** Already iterates `node.parameters` generically (the slider UI works as-is). Needs: (a) display CV ports (from catalog, not currently shown), (b) handle new node categories in the identity header, (c) sequencer-step editing UI (16 gate toggles + 16 CV sliders) for the sequencer node. The parameter editing core is reusable. `[VERIFIED: NodeInspector.tsx:100-127]`

**`session-client.ts` (src/lib/session-client.ts):** The Zod schemas at `:70-114` hardcode closed enums that the catalog refactor obsoletes. **Must update in lockstep** (Pitfall #5): relax `nodeType`/`sourceType`/`effectType` to catalog-driven strings, add `node_type_id`, regenerate the `audioPrimitive` discriminated union (or remove it if the model flattens). Existing test file `session-client.test.ts` should be extended to round-trip a v2 catalog-derived session. `[VERIFIED: session-client.ts:70-114]`

**ts-rs export (session.rs:806):** Add `NodeCatalogEntry::decl`, `CatalogPortSpec::decl`, `CatalogParamSpec::decl`, `NodeCategory::decl`, `SequencerPattern::decl` (and any new command variant) to the declarations array at `:808-875`. One line each. `[VERIFIED: session.rs:808-875]`

## Schema/Version Bump + v1 Rejection (D-09/D-10)

1. **Bump:** `CURRENT_SCHEMA_VERSION: u32 = 1` (session.rs:9) → `2`. `[VERIFIED: session.rs:9]`
2. **Rejection mechanism ALREADY EXISTS:** `SessionFileError::UnsupportedSchemaVersion { expected, found }` (session_file.rs:26) is already checked at `:59`. Bumping the const auto-rejects v1 files. `[VERIFIED: session_file.rs:26,59]`
3. **Friendly message (D-10):** The current message ("unsupported schemaVersion 1; expected 2") is generic. D-10 wants a specific "This is a v1 session — unsupported in Scrysynth v2" message. Add a dedicated `SessionFileError::LegacyV1Session` variant (or enrich `UnsupportedSchemaVersion` with a friendlier `Display`), and emit it when `found == 1`.
4. **Two-phase parse (Pitfall #1 — CRITICAL):** Because the domain model changes (AudioPrimitive removed), `serde_json::from_str::<SessionDocument>` on a v1 file will fail with a cryptic serde error *before* the version check. Must parse `{ schemaVersion }` first, check, then full-parse only if version == 2.
5. **Frontend surfacing:** `open_session_from_path` (lib.rs:66) maps `SessionFileError → String` via `.map_err(|e| e.to_string())`; `openSessionFromPath` (session-client.ts:434) rejects with that string. The frontend caller must catch the rejection and show a dialog (the error string IS the friendly message if step 3 makes it so). No new IPC needed. `[VERIFIED: lib.rs:66-76, session-client.ts:434]`

## Successor-Phase Compatibility

The catalog design must not block Phases 13/15/16 (all depend on it per ROADMAP.md):

| Phase | Dependency on catalog | Design requirement satisfied |
|-------|----------------------|------------------------------|
| **13 (Graph UX)** | Custom typed-handle nodes per catalog entry; port-type validation at canvas + Rust authority | `CatalogPortSpec.signal_type` + `direction` drive React Flow `isValidConnection` and Rust `validate_route` (compiler.rs:194). Catalog port list is the typed-handle source. |
| **15 (Visuals)** | Catalog-driven visual compiler | `NodeCatalogEntry.visual_shape` field replaces `visual/compiler.rs:57` enum dispatch. Phase 15 reads the field. |
| **16 (Agent)** | Catalog schemas in agent context packet so the LLM knows available node types | `NodeCatalogEntry` is `#[derive(TS)]` + serde-serializable; `agent_planner.rs` `GraphContext` (agent_planner.rs:66) can include a catalog snapshot. `ParserPlannerProvider` (agent_planner.rs:383) parses natural language → TypedCommand using catalog node_type_ids. |

`[VERIFIED: ROADMAP.md:68,92,104 phase dependencies; visual/compiler.rs:57; agent_planner.rs:66,383]`

**Non-blocking confirmation:** A flat const `NodeCatalogEntry` table with a `visual_shape` field, ts-rs-exported, satisfies all three successor phases without any forward-looking complexity. The catalog is intentionally additive data, not a framework.

## Testable Invariants (nyquist_validation is FALSE — no formal VALIDATION.md, but surface these)

> `workflow.nyquist_validation` is explicitly `false` in `.planning/config.json`, so no formal VALIDATION.md is required. However, the phase has strong testable invariants the planner should encode as tests:

| Invariant | Test type | Existing test to extend |
|-----------|-----------|-------------------------|
| Unknown node_type_id returns `Err`, not panic (success criterion #3) | unit | `audio_runtime.rs:339` `resource_plan_fails_loudly_for_unsupported_runtime_target` |
| Every catalog entry maps to a present `.scsyndef` with SCgf v2 header | unit | `audio_runtime.rs:433` `checked_in_v1_synthdef_resources_are_present_and_named` (rename/extend for v2 catalog) |
| Every catalog entry's synthdef `/d_recv`s successfully to real scsynth | integration (`#[ignore]`, scsynth-gated) | NEW — extends the above; boot scsynth via `SuperColliderAdapter`, `/d_recv` each, assert `/done` |
| CV route allocates a control bus; mod-source gets `out_cv_bus`, target gets `<param>_cv_bus` | unit | NEW — extend `resource_plan_maps_supported_primitives_to_known_synthdefs_and_params` (audio_runtime.rs:209) with an LFO→filter CV route |
| Port-type mismatch on a CV route is rejected by `validate_routes` | unit | NEW — extend compiler `validate_routes` (compiler.rs:194) test |
| v1 session file (schemaVersion 1) is rejected with the friendly message, not a serde error | unit | NEW — write a v1-shaped JSON to a temp file, assert `SessionFileError::LegacyV1Session` (two-phase parse) |
| Catalog is ts-rs-exported; round-trips through Zod | unit (TS) | `session-client.test.ts` — extend with a v2 catalog-derived session |
| Sequencer advances 16 steps and sends `/c_set` per step | unit (scripted OSC transport) | NEW — mirror `supercollider.rs:768` `apply_resource_plan_sends_groups_then_synths_in_plan_order` with a `ScriptedOscTransport` capturing `/c_set` messages |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | scsynth `/c_set [bus, value]` directly sets a control bus readable by `In.kr` (no `/b_alloc` needed for control buses) | CV/Modulation + Pattern 3 | Sequencer/CV silent; would need a writer synth. Low risk — `/c_set` is standard SC. `[CITED: SuperCollider OSC reference]` |
| A2 | 16 steps = 16th notes over one bar of 4/4 (step boundary every `60/bpm/4` seconds; current step = `floor(beats/0.25) % 16`) | Step Sequencer | Wrong clock division; planner may choose different mapping (CONTEXT.md defers this to planner). |
| A3 | `FreeVerb`/`GVerb`/`CombC`/`AllpassN`/`LFTri`/`LFCub`/`EnvGen`/`Round` are standard scsynth UGens available without plugins | New SynthDefs Needed | A named UGen unavailable → SynthDef `/d_recv` fails; the conformance test catches it. The Python byte-writer emits UGen names verbatim. `[CITED: SuperCollider UGen reference — standard built-ins]` |
| A4 | `std::thread::spawn` + `thread::sleep` is an adequate periodic driver for the sequencer (no need for tokio) | Step Sequencer + Pitfall #6 | Timing jitter at high CPU; acceptable for v1 single-user local app. Planner may choose tokio instead. |
| A5 | Control buses need NOT be canonical `Bus` records (compiler artifact only) | CV/Modulation + Runtime State Inventory | If Phase 13 needs named/persistent CV buses, this is revisited; not a blocker for Phase 12. |

**If this table is empty:** It is not — A1-A5 are the planner's confirmation points. None are blocking; all have clear fallbacks documented above.

## Open Questions

1. **Audio-rate CV scope for Phase 12** — D-03 requires "audio-rate + control-rate ports." Control-rate is the clear majority (LFO/Env/Sequencer). Audio-rate modulation (e.g. FM) needs `Out.ar`→`In.ar`. Recommendation: support control-rate fully + one validated audio-rate path. Planner confirms whether both need full catalog coverage or one representative audio-rate node suffices for the success criterion.
   - What we know: catalog ports carry `signal_type` (Audio/Control); the wiring is symmetric.
   - What's unclear: whether "audio-rate" requires a dedicated audio-rate modulator node family or just the capability on existing nodes.
   - Recommendation: implement control-rate for all mod sources + audio-rate for oscillator→oscillator frequency (FM) as the representative path; revisit if UAT demands more.

2. **Sequencer periodic-driver mechanism** — std::thread (recommended) vs tokio vs poll-piggyback.
   - What we know: tokio is not a dependency; the audio layer is sync.
   - What's unclear: whether introducing tokio now pays off for later phases (visuals streaming in Phase 15).
   - Recommendation: std::thread this phase (minimal surface); defer tokio adoption to when an async consumer (Phase 15 streaming) forces it.

3. **GraphEditCommand for step edits** — new `SetStepValue` variant vs reuse `SetParameterValue` with step-indexed param ids.
   - What we know: mutations must flow through typed commands (session.rs:473).
   - What's unclear: which is more legible for the agent (Phase 16) and the inspector.
   - Recommendation: dedicated `SetStepValue { node_id, step_index, gate: Option<bool>, cv: Option<f64> }` — most explicit, matches the "legible" identity.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `scsynth` (SuperCollider) | Audible playback (NODES-01..05), conformance test (success criterion #4) | ✓ (bundle) / ✗ (PATH) | 3.x at `/Applications/SuperCollider.app/Contents/Resources/scsynth` | `SCRYSYNTH_SCSYNTH_PATH` env var; resolve_scsynth_executable (supercollider.rs:459) handles all 3 sources |
| Python 3 | Extending `generate_synthdefs.py` (dev-time only) | ✓ | 3.14.6 (`/opt/homebrew/bin/python3`) | sclang hand-compile (not recommended) |
| Rust toolchain | All Rust changes | ✓ (v1 shipped) | stable (honor Tauri floor ≥1.77.2) | — |
| Node/npm | Frontend changes + ts-rs regen | ✓ (v1 shipped) | per STACK.md | — |
| `tokio` | Optional sequencer driver | ✗ NOT INSTALLED | — | std::thread (recommended) |

`[VERIFIED: bash probes — scsynth bundle present, scsynth not on PATH, python3 3.14.6; Cargo.toml rg — tokio absent]`

**Missing dependencies with no fallback:** None — scsynth bundle fallback covers the conformance test locally; CI must install SuperCollider (out of scope for this phase's plan, but flag for Phase 17 cross-platform).

**Missing dependencies with fallback:** `tokio` → std::thread (recommended for this phase).

## Security Domain

> `security_enforcement` is not explicitly set in `.planning/config.json` → treat as enabled. However, Phase 12 introduces **no new external input surfaces, auth, crypto, or untrusted-data handling** beyond v1's already-hardened boundaries. The catalog is compiled-in Rust (`&'static`), not user-supplied. Assessment:

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | N/A — local single-user desktop app |
| V3 Session Management | no | N/A — no network sessions |
| V4 Access Control | no | N/A — existing ownership gates (session_store.rs:696) unchanged |
| V5 Input Validation | yes (low) | Existing: serde for session files, typed `GraphEditCommand` gate (session.rs:473), `ScResourcePlanError` validation. NEW: catalog lookup rejects unknown node_type_id with `Err` (no panic). Two-phase parse rejects malformed/v1 JSON cleanly. |
| V6 Cryptography | no | N/A |
| V12 Files & Resources | yes (low) | SynthDef `.scsyndef` bytes are checked-in (not user-supplied); `/d_recv` of attacker-controlled synthdef bytes is not a surface (catalog is compiled-in). v1 file rejection (D-10) prevents loading legacy/untrusted shapes. |

**Known threat patterns for this stack:**
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed session JSON crashes load | Tampering/Denial | Two-phase parse + typed `SessionFileError`; no panic paths (the `unreachable!()` removal is itself a hardening) |
| Unknown node_type_id panics compiler | Denial | `find_catalog_entry() → Result`; `UnknownCatalogEntry` variant (replaces `unreachable!()` at synthdefs.rs:455) |
| CV bus index collision corrupts audio | Tampering | `FIRST_CONTROL_BUS_OFFSET` separation (Pitfall #2) |

**Net security impact:** Positive — the refactor *removes* a panic path (success criterion #3) and tightens input validation (catalog lookup > closed-enum assumption).

## Sources

### Primary (HIGH confidence)
- `src-tauri/src/audio/synthdefs.rs` — the 3 allowlists (:96 plan_sc_resources, :284 validate_runtime_target, :407 normalize_parameter_name, :429 synthdef_resource, :455 unreachable!(), :246 plan_buses, :382 apply_parameters, :6 HARDWARE_AUDIO_BUS_OFFSET) — read in full
- `src-tauri/src/audio/compiler.rs` — CompiledNodeKind (:51), compile_session_to_topology (:93), validate_routes (:194), topo_sort_nodes (:261), compile_node_launch (:324), node_sort_key (:418) — read in full
- `src-tauri/src/domain/session.rs` — closed enums (:195 NodeType, :204 AudioPrimitive, :221 AudioSourceType, :236 AudioEffectType), Node (:179), Port (:272), SignalType (:288), GraphEditCommand (:473), TransportState (:161), CURRENT_SCHEMA_VERSION (:9), write_generated_typescript_contract (:806) — read in full
- `src-tauri/src/audio/supercollider.rs` — SuperColliderAdapter (:26), apply_resource_plan (:188), /d_recv (:208), /s_new (:262), send_live_parameter /n_set (:286), resolve_scsynth_executable (:459), ScOscClient (:496) — read in full
- `src-tauri/src/audio/runtime_manager.rs` — reconcile_graph_edit (:174), AudioRuntimeAdapter trait (:11) — read in full
- `src-tauri/src/persistence/session_file.rs` — SessionFileError::UnsupportedSchemaVersion (:26), version check (:59), two-phase parse need — read in full
- `src-tauri/src/visual/compiler.rs` — NodeType→shape dispatch (:57) — read in full
- `src-tauri/src/application/agent_planner.rs` — GraphContext (:66), ParserPlannerProvider (:383) — read in full
- `src-tauri/src/application/session_store.rs` — SessionStore, poll_hardware_events (:220), build_default_session (:911) — read in full
- `src-tauri/src/lib.rs` — open_session_from_path command (:66) — read
- `src-tauri/resources/synthdefs/v1/scrysynth_v1_synthdefs.scd` — all 6 v1 SynthDefs — read in full
- `src-tauri/resources/synthdefs/v1/generate_synthdefs.py` — SynthDefBuilder, all 6 defs, synthdef_bytes (:351) — read in full
- `src-tauri/tests/audio_runtime.rs` — existing synthdef + lifecycle tests (:201 synthdefs, :433 conformance template, :88 lifecycle with FakeAdapter) — read in full
- `src/components/audio/PrimitivePalette.tsx` — hardcoded factory (:62), runtimeTarget quirk (:84) — read in full
- `src/components/session/NodeInspector.tsx` — parameter UI (:100) — read in full
- `src/lib/session-client.ts` — Zod schemas (:70-114), openSessionFromPath (:434) — read in full
- `src-tauri/Cargo.toml` — confirmed serde/ts-rs/rosc/thiserror versions, **tokio absent** (via `rg`)
- `.planning/config.json` — `workflow.nyquist_validation: false`

### Secondary (MEDIUM confidence)
- `.planning/phases/12-node-catalog-foundation/12-CONTEXT.md` — locked decisions D-01..D-10, canonical refs, agent's discretion
- `.planning/REQUIREMENTS.md` — NODES-01..05 verbatim
- `.planning/STATE.md` — [v2.0 Catalog-as-data] decision
- `.planning/ROADMAP.md` — Phase 12 details + 13/15/16 dependencies
- `AGENTS.md` — project constraints (Rust owns truth, SC=execution, stable primitives, td, GSD workflow)

### Tertiary (LOW confidence — SuperCollider mechanics from training, marked [CITED]/[ASSUMED] inline)
- SuperCollider bus model (audio/control shared index space, rate by UGen) — `[CITED: docs.supercollider.online]`
- `/c_set` OSC semantics — `[CITED: SuperCollider OSC reference]`
- `In.kr`/`Out.kr`/`In.ar`/`Out.ar` rate-mismatch silent failure — `[CITED: SuperCollider docs]`
- Standard UGen availability (`FreeVerb`, `LFTri`, `EnvGen`, etc.) — `[ASSUMED/CITED: SC built-ins]`

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all crates already locked in Cargo.toml; no new packages; versions verified via `rg`.
- Architecture (catalog model, 5-spot refactor, control-bus allocation): HIGH — grounded in direct file:line reads of all 5 hardcoded spots + the bus-allocation code.
- CV/modulation wiring: MEDIUM-HIGH — the compiler/adapter structure is verified; the SC `/c_set` + `In.kr`/`Out.kr` mechanics are well-established but tagged `[CITED]` (not re-verified against live scsynth this session; the conformance test will verify at execution time).
- Sequencer (app-driven): MEDIUM — the `/c_set` mechanism is sound; the periodic-driver choice (std::thread vs tokio) is an open planner decision (tokio absence is verified).
- SynthDef authoring: HIGH — `generate_synthdefs.py` is a verified working asset; extension is mechanical.
- v1 rejection (D-10): HIGH — the mechanism already exists; only message + parse-ordering work remains.

**Research date:** 2026-06-26
**Valid until:** 2026-07-26 (30 days — stable; no fast-moving external dependencies introduced)
