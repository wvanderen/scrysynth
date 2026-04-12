---
phase: 02-playable-audio-graph
plan: 02
type: execute
wave: 2
depends_on:
  - 02-playable-audio-graph-01
files_modified:
  - src-tauri/Cargo.toml
  - src-tauri/src/audio/mod.rs
  - src-tauri/src/audio/compiler.rs
  - src-tauri/src/audio/runtime_manager.rs
  - src-tauri/src/audio/supercollider.rs
  - src-tauri/src/application/session_store.rs
  - src-tauri/src/lib.rs
  - src-tauri/tests/audio_runtime.rs
autonomous: true
requirements:
  - AUD-01
  - AUD-04
must_haves:
  truths:
    - The backend can compile the canonical audio graph into deterministic runtime instructions and bus or group ordering without making SuperCollider the source of truth.
    - The app can boot, supervise, stop, and panic the audio runtime through a Rust-owned lifecycle manager that reports health back into canonical session state.
    - Panic-safe stop-all returns the app to a known safe audio state even after runtime failures or disconnects.
  artifacts:
    - path: src-tauri/src/audio/compiler.rs
      provides: Deterministic canonical-graph to runtime topology compilation
      contains: "compile_session_to_topology"
    - path: src-tauri/src/audio/runtime_manager.rs
      provides: Supervised runtime lifecycle and panic-safe controls
      contains: "AudioRuntimeManager"
    - path: src-tauri/src/lib.rs
      provides: Audio transport and panic commands exposed to the frontend
      contains: "panic_audio_runtime"
  key_links:
    - from: src-tauri/src/lib.rs
      to: src-tauri/src/audio/runtime_manager.rs
      via: Tauri commands delegate runtime lifecycle changes to the manager
      pattern: "start_audio_runtime|stop_audio_runtime|panic_audio_runtime"
    - from: src-tauri/src/audio/runtime_manager.rs
      to: src-tauri/src/audio/compiler.rs
      via: Runtime manager compiles the current canonical session before boot or safe rebuild
      pattern: "compile_session_to_topology"
    - from: src-tauri/src/audio/compiler.rs
      to: src-tauri/src/domain/session.rs
      via: Compiler consumes canonical primitives and routes rather than SC-owned state
      pattern: "SessionDocument"
---

<objective>
Turn the bounded canonical audio graph into a supervised SuperCollider runtime with deterministic playback and panic-safe recovery.

Purpose: Phase 2 only becomes playable when the backend can translate app-owned session state into reliable sound, monitor runtime health, and stop all sound immediately without trusting the runtime as the system of record.
Output: Audio compiler, runtime lifecycle manager, SuperCollider adapter, runtime-facing Tauri commands, and tests for deterministic compilation and lifecycle transitions.
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
@.planning/phases/02-playable-audio-graph/02-playable-audio-graph-01-PLAN.md
@src-tauri/Cargo.toml
@src-tauri/src/domain/session.rs
@src-tauri/src/application/session_store.rs
@src-tauri/src/lib.rs

<interfaces>
For the first working path, target a local `scsynth` executable discoverable on `PATH` or via one explicit app-configured override, and keep bundling or sidecar packaging out of this plan. Expose transport commands shaped like:

```rust
#[tauri::command]
fn start_audio_runtime(state: tauri::State<'_, Mutex<SessionStore>>) -> Result<SessionDocument, String>

#[tauri::command]
fn stop_audio_runtime(state: tauri::State<'_, Mutex<SessionStore>>) -> Result<SessionDocument, String>

#[tauri::command]
fn panic_audio_runtime(state: tauri::State<'_, Mutex<SessionStore>>) -> Result<SessionDocument, String>
```

Keep realtime audio-rate messaging out of Tauri IPC; use it only for lifecycle and command-and-control.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Compile the canonical audio graph into deterministic runtime topology</name>
  <files>src-tauri/Cargo.toml, src-tauri/src/audio/mod.rs, src-tauri/src/audio/compiler.rs, src-tauri/tests/audio_runtime.rs</files>
  <read_first>
    - src-tauri/Cargo.toml
    - src-tauri/src/domain/session.rs
    - .planning/phases/02-playable-audio-graph/02-RESEARCH.md
  </read_first>
  <behavior>
    - Test 1: compiling the same canonical session twice yields the same ordered buses, groups, and node launch instructions.
    - Test 2: invalid or incomplete graph shapes fail compilation with explicit errors instead of partial runtime instructions.
  </behavior>
  <action>Create `src-tauri/src/audio/compiler.rs` with a deterministic compiler from `SessionDocument` into a runtime topology structure that names launch order, bus allocation, and grouped processing boundaries. Sort or normalize canonical entities where needed so reloads and live rebuilds preserve the same runtime ordering. Keep the output adapter-facing and ephemeral: it should contain enough information to launch SuperCollider safely, but it must not become the persisted session schema. Add compiler tests to `src-tauri/tests/audio_runtime.rs` that lock ordering and error behavior.</action>
  <acceptance_criteria>
    - src-tauri/src/audio/compiler.rs contains `compile_session_to_topology`
    - src-tauri/src/audio/compiler.rs contains `CompiledTopology`
    - src-tauri/tests/audio_runtime.rs contains `deterministic`
    - src-tauri/tests/audio_runtime.rs contains `compile_error`
  </acceptance_criteria>
  <verify>
    <automated>cargo test audio_runtime::compiler --manifest-path src-tauri/Cargo.toml</automated>
  </verify>
  <done>The backend has a stable compiler from canonical graph state to runtime launch instructions.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Add supervised SuperCollider lifecycle management and panic-safe commands</name>
  <files>src-tauri/Cargo.toml, src-tauri/src/audio/runtime_manager.rs, src-tauri/src/audio/supercollider.rs, src-tauri/src/application/session_store.rs, src-tauri/src/lib.rs, src-tauri/tests/audio_runtime.rs</files>
  <read_first>
    - src-tauri/src/lib.rs
    - src-tauri/src/application/session_store.rs
    - src-tauri/src/audio/compiler.rs
    - .planning/phases/02-playable-audio-graph/02-RESEARCH.md
  </read_first>
  <behavior>
    - Test 1: starting the runtime transitions canonical runtime health from disconnected to booting to ready when the adapter reports success.
    - Test 2: panic tears down or mutes every active node, updates runtime health to a known safe state, and allows a clean restart.
    - Test 3: process launch or adapter failure marks degraded or error state without corrupting the canonical graph.
  </behavior>
  <action>Create `src-tauri/src/audio/runtime_manager.rs` and `src-tauri/src/audio/supercollider.rs` to supervise the `scsynth` process, send the compiled topology into the adapter, and surface runtime lifecycle updates back into `SessionStore`. Use traits or another seam so tests can exercise lifecycle transitions without requiring a real SuperCollider install. Add Tauri commands in `src-tauri/src/lib.rs` for start, stop, and panic, and make panic the fastest safe path back to silence rather than a UI-only flag. Keep executable discovery local-install-first for now, with one clear error when `scsynth` cannot be found.</action>
  <acceptance_criteria>
    - src-tauri/src/audio/runtime_manager.rs contains `pub struct AudioRuntimeManager`
    - src-tauri/src/audio/supercollider.rs contains `scsynth`
    - src-tauri/src/lib.rs contains `start_audio_runtime`
    - src-tauri/src/lib.rs contains `stop_audio_runtime`
    - src-tauri/src/lib.rs contains `panic_audio_runtime`
    - src-tauri/tests/audio_runtime.rs contains `panic`
  </acceptance_criteria>
  <verify>
    <automated>cargo test audio_runtime --manifest-path src-tauri/Cargo.toml && cargo build --manifest-path src-tauri/Cargo.toml</automated>
  </verify>
  <done>The backend can boot and supervise a playable SuperCollider runtime, recover from panic, and keep the canonical session as the source of truth.</done>
</task>

</tasks>

<verification>
Run the compiler and runtime test suite, then manually verify one local `scsynth` start, stop, and panic flow during development if the executable is available on the workstation.
</verification>

<success_criteria>
`AUD-01` and `AUD-04` backend groundwork are complete when the canonical graph compiles deterministically, the app supervises the runtime through Rust-owned commands, and panic returns the runtime to a known safe state without persisting SC-specific IDs.
</success_criteria>

<output>
After completion, create `.planning/phases/02-playable-audio-graph/02-playable-audio-graph-02-SUMMARY.md`
</output>
