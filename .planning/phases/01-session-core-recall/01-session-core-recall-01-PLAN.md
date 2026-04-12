---
phase: 01-session-core-recall
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/Cargo.toml
  - src-tauri/build.rs
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
    - Frontend code consumes the same session contract Rust persists and serves, and contract generation fails loudly if it drifts.
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
    - path: src-tauri/build.rs
      provides: Build-time contract generation guard
      contains: "session-types.ts"
  key_links:
    - from: src-tauri/src/lib.rs
      to: src-tauri/src/application/session_store.rs
      via: Tauri managed state and commands
      pattern: "State<.*SessionStore"
    - from: src-tauri/src/domain/session.rs
      to: src/generated/session-types.ts
      via: ts-rs export aggregated into one checked-in file
      pattern: "TS"
    - from: src-tauri/build.rs
      to: src/generated/session-types.ts
      via: build step writes contract file and fails on write errors
      pattern: "session-types\\.ts"
---

<objective>
Define the canonical session contracts and Rust-owned session service that every later phase builds on.

Purpose: Freeze app-native session semantics early so persistence, UI, runtimes, and agents all depend on one source of truth, and make generated frontend contracts durable instead of best-effort.
Output: Rust session schema, in-memory session store, Tauri session commands, build-time TS contract generation, and checked-in frontend types.
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
@.planning/phases/01-session-core-recall/01-session-core-recall-REVIEWS.md
@src-tauri/Cargo.toml
@src-tauri/build.rs
@src-tauri/src/lib.rs

<interfaces>
From `src-tauri/src/lib.rs` before implementation:
```rust
#[tauri::command]
fn greet(name: &str) -> String
```

The starter app exposes a single command and no managed session state. Replace this with typed session commands and shared state.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Define the canonical session schema and checked-in frontend contract</name>
  <files>src-tauri/Cargo.toml, src-tauri/src/domain/mod.rs, src-tauri/src/domain/session.rs, src/generated/session-types.ts</files>
  <read_first>
    - src-tauri/Cargo.toml
    - src-tauri/src/lib.rs
    - .planning/ROADMAP.md
    - .planning/phases/01-session-core-recall/01-RESEARCH.md
    - .planning/phases/01-session-core-recall/01-session-core-recall-REVIEWS.md
  </read_first>
  <behavior>
    - Test 1: serializing and deserializing `SessionDocument::default()` preserves `nodes`, `routes`, `buses`, `macros`, `scenes`, `variations`, `ownershipRules`, and `runtimeStatus`.
    - Test 2: each `Node` exposes `id`, `nodeType`, `ports`, `parameters`, `runtimeTarget`, `sceneMembership`, and `ownership` fields required by `SESS-03` even before the inspector UI exists.
  </behavior>
  <action>Create `src-tauri/src/domain/session.rs` with app-native structs and enums for `SessionDocument`, `TransportState`, `Node`, `Port`, `Route`, `Bus`, `MacroDefinition`, `SceneDefinition`, `VariationDefinition`, `OwnershipRule`, `OwnershipAssignment`, and `RuntimeStatusRef`. Derive `serde::{Serialize, Deserialize}` and `ts_rs::TS`, add `#[serde(rename_all = "camelCase")]` to every public persistence boundary type so disk files, IPC payloads, and generated TS use the same field names, and keep `schema_version` fixed to `1` internally while serializing as `schemaVersion`. Use `uuid::Uuid`-backed string IDs for canonical identifiers rather than runtime-specific IDs. Generate one self-contained `src/generated/session-types.ts` file from the Rust `TS` declarations so the frontend imports `SessionDocument` and nested types from a single file instead of handwritten interfaces. Implement the schema tests first per `SESS-01` and keep the default session collections non-empty enough for later UI rendering.</action>
  <acceptance_criteria>
    - src-tauri/src/domain/session.rs contains `pub struct SessionDocument`
    - src-tauri/src/domain/session.rs contains `pub struct Node`
    - src-tauri/src/domain/session.rs contains `pub struct OwnershipRule`
    - src-tauri/src/domain/session.rs contains `rename_all = "camelCase"`
    - src/generated/session-types.ts contains `export type SessionDocument`
    - src/generated/session-types.ts contains `export type Node`
    - cargo test exits 0 for the session schema tests
  </acceptance_criteria>
  <verify>
    <automated>cargo test session_document --manifest-path src-tauri/Cargo.toml</automated>
  </verify>
  <done>The canonical schema exists in Rust, round-trips under test, and the frontend has one checked-in TS contract generated from the same source.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Replace the starter command with a managed session store and fail-loud contract generation</name>
  <files>src-tauri/Cargo.toml, src-tauri/build.rs, src-tauri/src/lib.rs, src-tauri/src/application/mod.rs, src-tauri/src/application/session_store.rs</files>
  <read_first>
    - src-tauri/build.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/domain/session.rs
    - src/generated/session-types.ts
    - .planning/phases/01-session-core-recall/01-session-core-recall-REVIEWS.md
  </read_first>
  <behavior>
    - Test 1: `create_default_session` returns a canonical `SessionDocument` with at least one sample node, route, bus, macro, scene, variation, ownership rule, and runtime status reference.
    - Test 2: `get_current_session` returns the same in-memory session after initialization without querying any runtime engine.
    - Test 3: contract generation errors stop the Rust build instead of being silently ignored, addressing the review concern about TS export fragility.
  </behavior>
  <action>Create `src-tauri/src/application/session_store.rs` with `SessionStore { current: SessionDocument }` and methods `new_default()`, `current()`, and `replace_current(session: SessionDocument)`. Seed the default session with one source node, one bus, one route into a master node, one macro, one scene, one variation, one ownership rule, and disconnected runtime status placeholders. Update `src-tauri/src/lib.rs` to remove `greet`, manage `SessionStore` through `tauri::State<std::sync::Mutex<SessionStore>>`, and expose `#[tauri::command] fn create_default_session(...) -> Result<SessionDocument, String>` plus `#[tauri::command] fn get_current_session(...) -> Result<SessionDocument, String>`. Update `src-tauri/build.rs` so `cargo build`, `cargo test`, and Tauri startup regenerate `src/generated/session-types.ts` from the Rust `TS` declarations and return a non-zero build error if the file cannot be written. Do not keep a `let _ = write_generated_typescript_contract(...)` path that can fail silently; the build must stop on contract-write failure. This task addresses the cross-review concern about durable TS generation.</action>
  <acceptance_criteria>
    - src-tauri/src/application/session_store.rs contains `pub struct SessionStore`
    - src-tauri/src/lib.rs contains `create_default_session`
    - src-tauri/src/lib.rs contains `get_current_session`
    - src-tauri/src/lib.rs contains `State<Mutex<SessionStore>>`
    - src-tauri/src/lib.rs no longer contains `fn greet`
    - src-tauri/build.rs contains `session-types.ts`
    - src-tauri/build.rs no longer contains only `tauri_build::build()`
  </acceptance_criteria>
  <verify>
    <automated>cargo test session_store --manifest-path src-tauri/Cargo.toml && cargo build --manifest-path src-tauri/Cargo.toml</automated>
  </verify>
  <done>The Tauri backend boots with a canonical in-memory session store, typed session commands, and a build step that refuses to hide TS contract generation failures.</done>
</task>

</tasks>

<verification>
Run the Rust schema and session-store test subsets, then run a full Rust build to prove the checked-in TypeScript contract is regenerated by build-time automation rather than a best-effort runtime side effect.
</verification>

<success_criteria>
`SESS-01` groundwork is complete when the app has one canonical session schema, one Rust-managed in-memory session, one build-guarded TypeScript contract file, and no starter command left in the backend.
</success_criteria>

<output>
After completion, create `.planning/phases/01-session-core-recall/01-session-core-recall-01-SUMMARY.md`
</output>
