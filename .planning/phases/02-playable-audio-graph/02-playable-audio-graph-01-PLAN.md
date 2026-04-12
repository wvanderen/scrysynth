---
phase: 02-playable-audio-graph
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/Cargo.toml
  - src-tauri/src/domain/mod.rs
  - src-tauri/src/domain/session.rs
  - src-tauri/src/application/mod.rs
  - src-tauri/src/application/session_store.rs
  - src-tauri/src/application/graph_edit.rs
  - src-tauri/src/lib.rs
  - src/generated/session-types.ts
  - src-tauri/tests/audio_graph_commands.rs
autonomous: true
requirements:
  - SESS-04
  - AUD-03
must_haves:
  truths:
    - The canonical session exposes a small explicit v1 audio primitive set with enable state, bounded parameters, buses, and runtime intent without storing raw SuperCollider internals.
    - The backend accepts structured graph-edit commands for supported add, remove, enable, disable, and reroute actions and rejects illegal routes or unsupported cycles before runtime work begins.
    - The generated frontend contract stays aligned with the Rust audio graph model and edit command payloads.
  artifacts:
    - path: src-tauri/src/domain/session.rs
      provides: Canonical v1 audio primitive schema and runtime-facing audio state
      contains: "GraphEditCommand"
    - path: src-tauri/src/application/graph_edit.rs
      provides: Graph mutation and validation service
      contains: "apply_graph_edit"
    - path: src-tauri/tests/audio_graph_commands.rs
      provides: Validation coverage for allowed and rejected graph edits
      contains: "rejects"
  key_links:
    - from: src-tauri/src/lib.rs
      to: src-tauri/src/application/graph_edit.rs
      via: Tauri command delegates bounded graph edits to application validation
      pattern: "apply_graph_edit"
    - from: src-tauri/src/application/graph_edit.rs
      to: src-tauri/src/domain/session.rs
      via: Command application mutates canonical session entities only after validation passes
      pattern: "GraphEditCommand|SessionDocument"
    - from: src-tauri/src/domain/session.rs
      to: src/generated/session-types.ts
      via: ts-rs export keeps frontend contracts synchronized with canonical audio graph changes
      pattern: "export type GraphEditCommand"
---

<objective>
Expand the canonical session from a read-only seed into a bounded editable audio graph the runtime can trust.

Purpose: Phase 2 starts by making performer-facing audio semantics explicit in Rust so later runtime compilation, live mutation, and panic recovery operate on one safe edit surface instead of ad hoc UI state or raw SuperCollider details.
Output: Extended session schema, bounded graph-edit commands, validation rules, Tauri graph-edit command wiring, regenerated TS contracts, and tests for legal and illegal graph mutations.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/REQUIREMENTS.md
@.planning/phases/02-playable-audio-graph/02-RESEARCH.md
@src-tauri/Cargo.toml
@src-tauri/src/domain/session.rs
@src-tauri/src/application/session_store.rs
@src-tauri/src/lib.rs

<interfaces>
Introduce one structured mutation entrypoint rather than a loose set of runtime-shaped commands:

```rust
#[tauri::command]
fn apply_graph_edit(
    command: GraphEditCommand,
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String>
```

The command grammar should cover the supported v1 edit surface only: create or remove primitives, enable or disable nodes, change bounded parameters, and create or remove routes or bus assignments.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Extend the canonical session with explicit v1 audio primitives and runtime-facing audio state</name>
  <files>src-tauri/Cargo.toml, src-tauri/src/domain/mod.rs, src-tauri/src/domain/session.rs, src/generated/session-types.ts</files>
  <read_first>
    - src-tauri/Cargo.toml
    - src-tauri/src/domain/session.rs
    - src/generated/session-types.ts
    - .planning/ROADMAP.md
    - .planning/phases/02-playable-audio-graph/02-RESEARCH.md
  </read_first>
  <behavior>
    - Test 1: the session schema round-trips supported v1 source, effect, mixer, bus, and output primitives with enable state and bounded parameters intact.
    - Test 2: generated TypeScript contracts expose the new audio graph types and graph-edit payloads from the same Rust source.
  </behavior>
  <action>Extend `src-tauri/src/domain/session.rs` so the canonical graph can represent a deliberately small v1 audio set such as oscillator or noise sources, gain or mixer stages, one or two bounded effect types, buses, and master output intent. Keep the schema performer-facing: persist typed primitive definitions, enable or bypass state, channel or bus intent, and runtime health references, but do not persist raw SuperCollider node IDs or SynthDef internals. Export the new audio types and `GraphEditCommand` payloads through `ts-rs`, regenerate `src/generated/session-types.ts`, and keep serde naming aligned with the existing camelCase public contract.</action>
  <acceptance_criteria>
    - src-tauri/src/domain/session.rs contains `GraphEditCommand`
    - src-tauri/src/domain/session.rs contains `AudioRuntimeState`
    - src/generated/session-types.ts contains `export type GraphEditCommand`
    - src/generated/session-types.ts contains `export type AudioRuntimeState`
    - cargo test exits 0 for the schema-focused audio graph tests
  </acceptance_criteria>
  <verify>
    <automated>cargo test audio_graph_schema --manifest-path src-tauri/Cargo.toml</automated>
  </verify>
  <done>The canonical session can describe the supported v1 audio graph in a frontend-consumable contract without leaking runtime internals.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Add bounded graph-edit application and validation in the backend</name>
  <files>src-tauri/src/application/mod.rs, src-tauri/src/application/session_store.rs, src-tauri/src/application/graph_edit.rs, src-tauri/src/lib.rs, src-tauri/tests/audio_graph_commands.rs</files>
  <read_first>
    - src-tauri/src/application/session_store.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/domain/session.rs
    - .planning/phases/02-playable-audio-graph/02-RESEARCH.md
  </read_first>
  <behavior>
    - Test 1: adding a supported source node and routing it to a valid downstream target returns an updated `SessionDocument` with deterministic IDs and routes.
    - Test 2: attempting an illegal route shape such as output-to-output, duplicate bus target conflict, or unsupported cycle returns a typed validation error and leaves the store unchanged.
    - Test 3: removing a node prunes or rejects dependent routes predictably instead of leaving dangling references.
  </behavior>
  <action>Create `src-tauri/src/application/graph_edit.rs` with the graph mutation service that validates and applies `GraphEditCommand` values against the canonical session. Keep the rules intentionally narrow for Phase 2: supported node categories only, explicit route direction checks, no unsupported feedback cycles, and deterministic ordering for newly inserted nodes or routes so later compilation stays stable. Update `src-tauri/src/application/session_store.rs` with a safe mutation helper, expose `apply_graph_edit` from `src-tauri/src/lib.rs`, and add `src-tauri/tests/audio_graph_commands.rs` to cover accepted edits, rejected edits, and store isolation after failure.</action>
  <acceptance_criteria>
    - src-tauri/src/application/graph_edit.rs contains `apply_graph_edit`
    - src-tauri/src/application/graph_edit.rs contains `validate_route`
    - src-tauri/src/lib.rs contains `fn apply_graph_edit`
    - src-tauri/tests/audio_graph_commands.rs contains `rejects_cycle`
    - src-tauri/tests/audio_graph_commands.rs contains `store_unchanged`
  </acceptance_criteria>
  <verify>
    <automated>cargo test audio_graph_commands --manifest-path src-tauri/Cargo.toml</automated>
  </verify>
  <done>The backend exposes one safe graph-edit entrypoint that can grow with the runtime while already enforcing the bounded v1 edit surface.</done>
</task>

</tasks>

<verification>
Use the new Rust test suite to prove the extended schema round-trips, the generated TypeScript contract updates, and illegal graph mutations are rejected before any runtime adapter exists.
</verification>

<success_criteria>
`SESS-04` groundwork is complete when the canonical session supports a small editable audio primitive set, the app exposes one bounded graph-edit command surface, and invalid routes or unsupported cycles fail before runtime compilation.
</success_criteria>

<output>
After completion, create `.planning/phases/02-playable-audio-graph/02-playable-audio-graph-01-SUMMARY.md`
</output>
