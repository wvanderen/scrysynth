---
phase: 01-session-core-recall
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/Cargo.toml
  - src-tauri/src/lib.rs
  - src-tauri/src/domain/mod.rs
  - src-tauri/src/domain/session.rs
  - src-tauri/src/application/mod.rs
  - src-tauri/src/application/session_store.rs
  - src/generated/session-types.ts
autonomous: true
requirements:
  - SESS-01
must_haves:
  truths:
    - User can start from an app-owned canonical session before any runtime exists.
    - The current session includes nodes, routes, buses, macros, scenes, variations, ownership rules, and runtime status references.
    - Frontend code can consume the same schema Rust persists and serves.
  artifacts:
    - path: src-tauri/src/domain/session.rs
      provides: Canonical SessionDocument schema and nested entities
      contains: "pub struct SessionDocument"
    - path: src-tauri/src/application/session_store.rs
      provides: In-memory canonical session store
      contains: "pub struct SessionStore"
    - path: src/generated/session-types.ts
      provides: Rust-exported frontend session contracts
      contains: "export type SessionDocument"
  key_links:
    - from: src-tauri/src/lib.rs
      to: src-tauri/src/application/session_store.rs
      via: Tauri managed state and commands
      pattern: "State<.*SessionStore"
    - from: src-tauri/src/domain/session.rs
      to: src/generated/session-types.ts
      via: ts-rs export
      pattern: "TS"
---

<objective>
Define the canonical session contracts and Rust-owned session service that every later phase builds on.

Purpose: Freeze app-native session semantics early so persistence, UI, runtimes, and agents all depend on one source of truth.
Output: Rust session schema, in-memory session store, Tauri session commands, and generated frontend types.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/01-session-core-recall/01-RESEARCH.md
@src-tauri/Cargo.toml
@src-tauri/src/lib.rs

<interfaces>
From `src-tauri/src/lib.rs`:
```rust
#[tauri::command]
fn greet(name: &str) -> String
```

The starter app exposes a single command and no managed session state. Replace this with typed session commands and shared state.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Define canonical session schema and export frontend contracts</name>
  <files>src-tauri/Cargo.toml, src-tauri/src/domain/mod.rs, src-tauri/src/domain/session.rs, src/generated/session-types.ts</files>
  <read_first>
    - src-tauri/Cargo.toml
    - src-tauri/src/lib.rs
    - .planning/ROADMAP.md
    - .planning/phases/01-session-core-recall/01-RESEARCH.md
  </read_first>
  <behavior>
    - Test 1: serializing and deserializing `SessionDocument::default()` preserves `nodes`, `routes`, `buses`, `macros`, `scenes`, `variations`, `ownership_rules`, and `runtime_status` collections.
    - Test 2: each `Node` exposes `id`, `node_type`, `ports`, `parameters`, `runtime_target`, `scene_membership`, and `ownership` fields required by `SESS-03` even before the inspector UI exists.
  </behavior>
  <action>Create `src-tauri/src/domain/session.rs` with app-native structs and enums for `SessionDocument`, `TransportState`, `Node`, `Port`, `Route`, `Bus`, `MacroDefinition`, `SceneDefinition`, `VariationDefinition`, `OwnershipRule`, and `RuntimeStatusRef`. Add `serde::{Serialize, Deserialize}` and `ts_rs::TS` derives, use `schema_version: u32` fixed to `1`, and use `uuid::Uuid`-backed string IDs for canonical identifiers rather than runtime-specific IDs. Export the module from `src-tauri/src/domain/mod.rs`. Update `src-tauri/Cargo.toml` to add the exact crates needed for this contract layer: `ts-rs`, `uuid` with serde support, and any small error/test helpers used by the tests. Generate or check in `src/generated/session-types.ts` from the Rust types so the frontend has an explicit `SessionDocument` type instead of handwritten drift-prone interfaces. Implement the tests before finalizing the structs, per `SESS-01`.</action>
  <acceptance_criteria>
    - src-tauri/src/domain/session.rs contains `pub struct SessionDocument`
    - src-tauri/src/domain/session.rs contains `pub struct Node`
    - src-tauri/src/domain/session.rs contains `pub struct OwnershipRule`
    - src/generated/session-types.ts contains `export type SessionDocument`
    - cargo test exits 0 for the session schema tests
  </acceptance_criteria>
  <verify>
    <automated>cargo test session_document --manifest-path src-tauri/Cargo.toml</automated>
  </verify>
  <done>The canonical schema exists in Rust, round-trips under test, and the frontend has generated types for the same contract.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Replace greet with a Rust session store and typed session commands</name>
  <files>src-tauri/src/lib.rs, src-tauri/src/application/mod.rs, src-tauri/src/application/session_store.rs</files>
  <read_first>
    - src-tauri/src/lib.rs
    - src-tauri/src/domain/session.rs
    - .planning/phases/01-session-core-recall/01-RESEARCH.md
  </read_first>
  <behavior>
    - Test 1: `create_default_session` returns a canonical `SessionDocument` with at least one sample node, route, bus, macro, scene, variation, ownership rule, and runtime status reference.
    - Test 2: `get_current_session` returns the same in-memory session after initialization without querying any runtime engine.
  </behavior>
  <action>Create `src-tauri/src/application/session_store.rs` with `SessionStore { current: SessionDocument }` and methods `new_default()`, `current()`, and `replace_current(session: SessionDocument)`. Seed the default session with a small but meaningful sample graph so the UI can render real nodes and connections immediately: one source node, one bus, one route into a master node, one macro, one scene, one variation, one ownership rule, and disconnected runtime status placeholders. Update `src-tauri/src/lib.rs` to remove the `greet` command, manage `SessionStore` through `tauri::State<std::sync::Mutex<SessionStore>>`, and expose `#[tauri::command] fn create_default_session(...) -> Result<SessionDocument, String>` plus `#[tauri::command] fn get_current_session(...) -> Result<SessionDocument, String>`. Register only these session commands in `generate_handler!` for this phase.</action>
  <acceptance_criteria>
    - src-tauri/src/application/session_store.rs contains `pub struct SessionStore`
    - src-tauri/src/lib.rs contains `create_default_session`
    - src-tauri/src/lib.rs contains `get_current_session`
    - src-tauri/src/lib.rs contains `State<Mutex<SessionStore>>`
    - src-tauri/src/lib.rs no longer contains `fn greet`
  </acceptance_criteria>
  <verify>
    <automated>cargo test session_store --manifest-path src-tauri/Cargo.toml</automated>
  </verify>
  <done>The Tauri backend boots with a canonical in-memory session store and typed commands that serve app-owned session state with no runtime dependency.</done>
</task>

</tasks>

<verification>
Run the Rust test subset for schema and session-store coverage, then inspect the generated TS contract to confirm the UI will consume the same `SessionDocument` shape the backend owns.
</verification>

<success_criteria>
`SESS-01` groundwork is complete when the app has one canonical session schema, one Rust-managed in-memory session, and one typed frontend contract generated from the same source.
</success_criteria>

<output>
After completion, create `.planning/phases/01-session-core-recall/01-session-core-recall-01-SUMMARY.md`
</output>
