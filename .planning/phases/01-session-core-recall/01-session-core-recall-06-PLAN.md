---
phase: 01-session-core-recall
plan: 06
type: execute
wave: 2
depends_on: ["01-session-core-recall-05"]
files_modified:
  - src-tauri/build.rs
  - src-tauri/src/lib.rs
autonomous: true
gap_closure: true
requirements: [SESS-01]

must_haves:
  truths:
    - "Contract generation failure causes the app to fail visibly, not silently continue with stale types"
    - "Build re-runs contract generation when the session schema changes"
  artifacts:
    - path: "src-tauri/build.rs"
      provides: "Build-time rerun directive for session schema changes"
      contains: "cargo:rerun-if-changed"
    - path: "src-tauri/src/lib.rs"
      provides: "Contract generation with error propagation instead of silent discard"
      contains: "write_generated_typescript_contract"
  key_links:
    - from: "src-tauri/build.rs"
      to: "src-tauri/src/domain/session.rs"
      via: "cargo:rerun-if-changed directive"
      pattern: "rerun-if-changed.*session.rs"
    - from: "src-tauri/src/lib.rs"
      to: "write_generated_typescript_contract()"
      via: "expect() instead of let _ ="
      pattern: "write_generated_typescript_contract\\(\\)\\.expect"
---

<objective>
Make TypeScript contract generation fail loudly instead of silently discarding errors, and add a build guard that re-triggers generation when the session schema changes.

Purpose: The verification report found that `lib.rs` line 250 uses `let _ = write_generated_typescript_contract()` which silently discards write errors. Additionally, `build.rs` is bare — it has no awareness of the session schema, so contract drift won't be caught at build time. Both prevent the Phase 1 "contract generation fails loudly if it drifts" requirement from being met.

Output: Contract generation errors cause a visible failure, and the build re-triggers when session.rs changes.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/01-session-core-recall/01-VERIFICATION.md
@.planning/phases/01-session-core-recall/01-session-core-recall-01-SUMMARY.md

<interfaces>
<!-- Exact code that needs changing -->

From src-tauri/build.rs (entire file):
```rust
fn main() {
    tauri_build::build()
}
```

From src-tauri/src/lib.rs — run() function (lines 248-281):
```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = write_generated_typescript_contract();

    tauri::Builder::default()
        .manage(Mutex::new(SessionStore::new_default()))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        // ... rest of builder
```

From src-tauri/src/domain/session.rs — write_generated_typescript_contract (lines 641-724):
```rust
pub fn write_generated_typescript_contract() -> std::io::Result<()> {
    let cfg = Config::default();
    let declarations = [
        SessionDocument::decl(&cfg),
        // ... 50+ type declarations
    ]
    .join("\n\n");

    let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GENERATED_TYPES_PATH);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    // ... formatting and write
    fs::write(file_path, format!("..."))
}
```

Note: Tests in session.rs (lines 805 and 955) already call `write_generated_typescript_contract().expect("typescript contract is written")`.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Make contract generation fail loudly and add build-time schema awareness</name>
  <files>src-tauri/src/lib.rs, src-tauri/build.rs</files>
  <read_first>
    - src-tauri/src/lib.rs (contains the `let _ =` that must change)
    - src-tauri/build.rs (needs rerun directive)
    - src-tauri/src/domain/session.rs (the schema file to watch — lines 641-724 for context)
  </read_first>
  <action>
    **Change 1: Make lib.rs propagate contract generation errors**

    In `src-tauri/src/lib.rs`, find line 250:
    ```rust
    let _ = write_generated_typescript_contract();
    ```

    Replace with:
    ```rust
    if let Err(err) = write_generated_typescript_contract() {
        eprintln!("ERROR: Failed to write TypeScript type contract: {err}");
        eprintln!("Run `cargo test write_generated_typescript_contract --manifest-path src-tauri/Cargo.toml` to diagnose.");
        std::process::exit(1);
    }
    ```

    This prints a clear diagnostic and exits with failure instead of silently continuing. Using `if let Err` + `process::exit` instead of `.expect()` to provide a more actionable error message (`.expect()` would panic with a less helpful backtrace at app startup).

    **Change 2: Add build rerun guard in build.rs**

    In `src-tauri/build.rs`, replace the entire file with:
    ```rust
    fn main() {
        println!("cargo:rerun-if-changed=src/domain/session.rs");
        tauri_build::build()
    }
    ```

    This tells Cargo to re-run the build script (and thus recompile the app) whenever `session.rs` changes, ensuring the TS contract stays in sync with the Rust schema.

    Do NOT move the contract generation function into build.rs — it depends on the compiled Rust types from `session.rs` and cannot run before compilation. The build.rs directive ensures recompilation triggers on schema changes; the lib.rs change ensures generation failures are visible.
  </action>
  <verify>
    <automated>grep -c "if let Err" src-tauri/src/lib.rs && grep -c "rerun-if-changed" src-tauri/build.rs && cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5</automated>
  </verify>
  <done>
    - lib.rs no longer contains `let _ = write_generated_typescript_contract()`
    - lib.rs contains error propagation with diagnostic message and process::exit(1)
    - build.rs contains cargo:rerun-if-changed=src/domain/session.rs directive
    - All Rust tests pass
  </done>
  <acceptance_criteria>
    - `grep "let _ = write_generated_typescript_contract" src-tauri/src/lib.rs` returns nothing (exit code 1)
    - `grep "if let Err(err) = write_generated_typescript_contract" src-tauri/src/lib.rs` returns a match
    - `grep "process::exit(1)" src-tauri/src/lib.rs` returns a match
    - `grep "rerun-if-changed.*session.rs" src-tauri/build.rs` returns a match
    - `cargo test --manifest-path src-tauri/Cargo.toml` exits 0
  </acceptance_criteria>
</task>

</tasks>

<verification>
```bash
# No silent error discard
grep "let _ = write_generated_typescript_contract" src-tauri/src/lib.rs
# Expected: exit code 1 (no matches)

# Error propagation present
grep "if let Err" src-tauri/src/lib.rs
# Expected: 1 match

# Build guard present
grep "rerun-if-changed" src-tauri/build.rs
# Expected: 1 match

# All Rust tests pass
cargo test --manifest-path src-tauri/Cargo.toml
# Expected: all pass
```
</verification>

<success_criteria>
- Contract generation errors cause the app to exit with a diagnostic message instead of silently continuing
- build.rs tracks session.rs for changes, triggering recompilation on schema edits
- All existing Rust tests pass
- The app still compiles and runs
</success_criteria>

<output>
After completion, create `.planning/phases/01-session-core-recall/01-session-core-recall-06-SUMMARY.md`
</output>
