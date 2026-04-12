---
phase: 01-session-core-recall
plan: 02
type: execute
wave: 2
depends_on:
  - 01-session-core-recall-01
files_modified:
  - src-tauri/Cargo.toml
  - src-tauri/src/lib.rs
  - src-tauri/src/persistence/mod.rs
  - src-tauri/src/persistence/session_file.rs
  - src-tauri/tests/session_persistence.rs
autonomous: true
requirements:
  - PERS-01
  - SESS-01
must_haves:
  truths:
    - User can save the current canonical session to app-owned data on disk.
    - User can reopen a saved session and recover graph structure, macros, scenes, variations, ownership rules, and runtime mapping state.
    - Save and open work without any runtime engine acting as the source of truth.
  artifacts:
    - path: src-tauri/src/persistence/session_file.rs
      provides: Versioned JSON save/open implementation
      contains: "save_session_to_path"
    - path: src-tauri/tests/session_persistence.rs
      provides: Round-trip persistence coverage
      contains: "round_trip"
    - path: src-tauri/src/lib.rs
      provides: Tauri save/open commands
      contains: "open_session_from_path"
  key_links:
    - from: src-tauri/src/lib.rs
      to: src-tauri/src/persistence/session_file.rs
      via: command dispatch to file persistence helpers
      pattern: "save_session_to_path|open_session_from_path"
    - from: src-tauri/src/lib.rs
      to: src-tauri/src/application/session_store.rs
      via: successful open replaces current session only after validation passes
      pattern: "replace_current"
---

<objective>
Add real save/open persistence for the canonical session so Phase 1 proves cold recall from app-owned data.

Purpose: Persistence is the trust anchor for later runtime compilation, scene recall, and agent collaboration, so the app must save and reopen one exact canonical document instead of runtime-owned state.
Output: Versioned JSON session file service, explicit Tauri save/open commands, and persistence tests for round-trip plus failure paths.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/01-session-core-recall/01-RESEARCH.md
@.planning/phases/01-session-core-recall/01-session-core-recall-01-PLAN.md
@.planning/phases/01-session-core-recall/01-session-core-recall-REVIEWS.md
@src-tauri/src/lib.rs
@src-tauri/src/application/session_store.rs
@src-tauri/src/domain/session.rs
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Implement versioned JSON session save/open with typed file errors</name>
  <files>src-tauri/Cargo.toml, src-tauri/src/persistence/mod.rs, src-tauri/src/persistence/session_file.rs, src-tauri/src/lib.rs</files>
  <read_first>
    - src-tauri/Cargo.toml
    - src-tauri/src/lib.rs
    - src-tauri/src/application/session_store.rs
    - src-tauri/src/domain/session.rs
    - .planning/phases/01-session-core-recall/01-RESEARCH.md
    - .planning/phases/01-session-core-recall/01-session-core-recall-REVIEWS.md
  </read_first>
  <behavior>
    - Test 1: saving the current session writes a JSON file containing `schemaVersion`, `nodes`, `routes`, `buses`, `macros`, `scenes`, `variations`, `ownershipRules`, and `runtimeStatus`.
    - Test 2: opening that file returns a `SessionDocument` that can replace the in-memory session exactly.
    - Test 3: opening corrupt JSON or a mismatched `schemaVersion` returns a typed error and does not replace the in-memory session.
  </behavior>
  <action>Create `src-tauri/src/persistence/session_file.rs` with `save_session_to_path(session: &SessionDocument, path: &Path)` and `open_session_from_path(path: &Path) -> Result<SessionDocument, SessionFileError>`. Define `SessionFileError` variants for read failure, write failure, deserialize failure, and schema version mismatch so the backend can distinguish error classes later instead of collapsing them into one string. Serialize with pretty JSON, create the temporary file in the same directory as the target path, flush it, and atomically rename it into place so the write stays on the same filesystem. Reject every `schemaVersion` except `1`; do not add migration logic in this phase. Export the module from `src-tauri/src/persistence/mod.rs`. Update `src-tauri/src/lib.rs` to expose Tauri commands `save_session_to_path(path: String, state: State<Mutex<SessionStore>>) -> Result<(), String>` and `open_session_from_path(path: String, state: State<Mutex<SessionStore>>) -> Result<SessionDocument, String>`, and only call `replace_current` after `open_session_from_path` successfully validates and decodes the file.</action>
  <acceptance_criteria>
    - src-tauri/src/persistence/session_file.rs contains `enum SessionFileError`
    - src-tauri/src/persistence/session_file.rs contains `save_session_to_path`
    - src-tauri/src/persistence/session_file.rs contains `open_session_from_path`
    - src-tauri/src/persistence/session_file.rs contains `schemaVersion`
    - src-tauri/src/lib.rs contains `save_session_to_path`
    - src-tauri/src/lib.rs contains `open_session_from_path`
    - src-tauri/src/lib.rs contains `replace_current`
  </acceptance_criteria>
  <verify>
    <automated>cargo test session_persistence --manifest-path src-tauri/Cargo.toml</automated>
  </verify>
  <done>The backend can save the current canonical session to disk and reopen it into the shared store with explicit typed errors and schema-version validation.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Add round-trip and failure-path persistence tests</name>
  <files>src-tauri/tests/session_persistence.rs</files>
  <read_first>
    - src-tauri/tests/session_persistence.rs
    - src-tauri/src/persistence/session_file.rs
    - src-tauri/src/application/session_store.rs
    - src-tauri/src/domain/session.rs
  </read_first>
  <behavior>
    - Test 1: round-trip save/open keeps the same node IDs, route IDs, macros, scenes, variations, ownership rules, and runtime status payloads.
    - Test 2: corrupt JSON fails cleanly with the deserialize error variant.
    - Test 3: unsupported schema version fails cleanly with the schema mismatch variant.
    - Test 4: a failed open leaves the managed session store unchanged.
  </behavior>
  <action>Create `src-tauri/tests/session_persistence.rs` using `tempfile` to write throwaway session files next to their final targets. Assert that a saved default session can be reopened with the same semantic content even if the pretty JSON formatting changes, and add negative tests for corrupt JSON, schema-version mismatch, and store replacement isolation. Keep the fixture focused on the exact Phase 1 canonical collections so `PERS-01` is objectively covered and future migration work has a stable regression suite.</action>
  <acceptance_criteria>
    - src-tauri/tests/session_persistence.rs contains `round_trip`
    - src-tauri/tests/session_persistence.rs contains `schema version`
    - src-tauri/tests/session_persistence.rs contains `replace_current`
    - cargo test exits 0 for `session_persistence`
  </acceptance_criteria>
  <verify>
    <automated>cargo test session_persistence --manifest-path src-tauri/Cargo.toml</automated>
  </verify>
  <done>Persistence behavior is covered by automated tests for successful round-trip and the main failure paths, including store safety after a failed open.</done>
</task>

</tasks>

<verification>
Use the persistence test suite to prove save/open round-trip behavior, then inspect one emitted JSON file during execution to verify the canonical collections are present and runtime-independent.
</verification>

<success_criteria>
`PERS-01` is satisfied when a saved session can be reopened into the Rust store with the same canonical data, typed file-error coverage, and no partial state replacement on failure.
</success_criteria>

<output>
After completion, create `.planning/phases/01-session-core-recall/01-session-core-recall-02-SUMMARY.md`
</output>
