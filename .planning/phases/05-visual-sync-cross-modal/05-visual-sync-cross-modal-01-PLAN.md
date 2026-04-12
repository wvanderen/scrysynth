---
phase: 05-visual-sync-cross-modal
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/domain/session.rs
  - src-tauri/src/visual/mod.rs
  - src-tauri/src/visual/adapter.rs
  - src-tauri/src/visual/runtime_manager.rs
  - src-tauri/src/visual/bevy_sidecar.rs
  - src-tauri/src/visual/compiler.rs
  - src-tauri/src/application/session_store.rs
  - src-tauri/src/application/mod.rs
  - src-tauri/src/lib.rs
  - src/lib/session-client.ts
  - src/store/sessionStore.ts
  - src/store/session-projections.ts
  - src/components/workspace/RuntimeHealthPanel.tsx
  - src/components/workspace/WorkspaceViewSwitcher.tsx
  - src-tauri/tests/visual_runtime.rs
autonomous: true
requirements:
  - PERS-02
  - UI-02
must_haves:
  truths:
    - User can start and stop a visual runtime that receives compiled scene descriptions from the canonical session without owning session state.
    - Visual runtime lifecycle (Idle, Starting, Ready, Rendering, Failed) and health (Unknown, Healthy, Degraded, Error) are tracked in the session document.
    - User can see runtime health, activity, and error status for the audio runtime, visual runtime, and agent system in a single dashboard panel.
    - RuntimeStatusRef entries for Audio, Visual, and Agent update in real-time with lifecycle commands.
    - Visual runtime gracefully degrades when the Bevy sidecar binary is unavailable (shows "disconnected" status, does not crash).
  artifacts:
    - path: src-tauri/src/visual/adapter.rs
      provides: VisualRuntimeAdapter trait mirroring AudioRuntimeAdapter
      contains: "VisualRuntimeAdapter"
    - path: src-tauri/src/visual/runtime_manager.rs
      provides: VisualRuntimeManager with start/stop/panic lifecycle
      contains: "VisualRuntimeManager"
    - path: src-tauri/src/visual/bevy_sidecar.rs
      provides: BevySidecarAdapter managing child process
      contains: "BevySidecarAdapter"
    - path: src-tauri/src/visual/compiler.rs
      provides: Session-to-visual-scene compiler
      contains: "compile_session_to_visual_scene"
    - path: src-tauri/src/domain/session.rs
      provides: VisualRuntimeState, VisualRuntimeLifecycle, VisualRuntimeHealth, AgentRuntimeState
      contains: "VisualRuntimeState"
    - path: src/components/workspace/RuntimeHealthPanel.tsx
      provides: Dashboard showing audio, visual, agent runtime status
      contains: "RuntimeHealthPanel"
    - path: src-tauri/tests/visual_runtime.rs
      provides: Integration tests for visual lifecycle, compiler, health dashboard data
      contains: "start_visual_runtime"
  key_links:
    - from: src-tauri/src/lib.rs
      to: src-tauri/src/application/session_store.rs
      via: Tauri commands start/stop/panic_visual_runtime
      pattern: "start_visual_runtime.*SessionStore"
    - from: src-tauri/src/application/session_store.rs
      to: src-tauri/src/visual/runtime_manager.rs
      via: lifecycle method delegation (mirrors audio pattern)
      pattern: "visual_runtime_manager\\.start"
    - from: src/components/workspace/RuntimeHealthPanel.tsx
      to: src/store/sessionStore.ts
      via: reads runtimeStatus, audioRuntime, visualRuntime, agentRuntime from session
      pattern: "runtimeStatus.*map"
---

<objective>
Add the visual runtime adapter infrastructure (mirroring the existing SC audio adapter) and a runtime health dashboard that shows status for all three runtime systems (audio, visual, agent).

Purpose: Extends Scrysynth from audio-only to audiovisual by adding a visual runtime that follows the proven adapter pattern, and gives the user a unified view of all runtime health.

Output: Visual runtime module, expanded domain types (VisualRuntimeState, AgentRuntimeState), runtime health dashboard UI component, integration tests.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/05-visual-sync-cross-modal/05-RESEARCH.md

<interfaces>
<!-- Existing adapter pattern — the exact template for the visual adapter -->

From src-tauri/src/audio/runtime_manager.rs:
```rust
pub trait AudioRuntimeAdapter {
    fn start(&mut self) -> Result<RuntimeAdapterStatus, String>;
    fn load_topology(&mut self, topology: &CompiledTopology) -> Result<RuntimeAdapterStatus, String>;
    fn stop(&mut self) -> Result<RuntimeAdapterStatus, String>;
    fn panic(&mut self) -> Result<RuntimeAdapterStatus, String>;
}

pub struct AudioRuntimeManager<A = SuperColliderAdapter> {
    adapter: A,
}
```

From src-tauri/src/audio/supercollider.rs:
```rust
pub struct SuperColliderAdapter {
    process: Option<Child>,
}
// Implements AudioRuntimeAdapter: start spawns process, load_topology sends patch, stop/panic kill process
```

From src-tauri/src/domain/session.rs — types to parallel:
```rust
pub struct AudioRuntimeState { lifecycle, health, sample_rate_hz, block_size, active_patch_id, last_error, panic_recovery_count }
pub struct RuntimeStatusRef { id, runtime: RuntimeKind, status: RuntimeConnectionState, target_id, last_error }
pub enum RuntimeKind { Audio, Visual, Agent }  // Visual and Agent already exist!
pub enum RuntimeConnectionState { Disconnected, Connecting, Ready, Error }
```

From src-tauri/src/application/session_store.rs:
```rust
pub struct SessionStore {
    current: SessionDocument,
    audio_runtime_manager: AudioRuntimeManager,
}
// Methods: start_audio_runtime, stop_audio_runtime, panic_audio_runtime
// Pattern: std::mem::take manager, call, put back
```

From src-tauri/src/lib.rs — Tauri command pattern:
```rust
#[tauri::command]
fn start_audio_runtime(state: tauri::State<'_, Mutex<SessionStore>>) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.start_audio_runtime().map_err(|err| err.to_string())
}
```

From src/store/sessionStore.ts — Zustand action pattern:
```typescript
startAudio: async () => {
    set({ isLoading: true, error: null });
    try {
        const session = await startAudioRuntime();
        const current = get();
        set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) { ... }
},
```

From src/lib/session-client.ts — IPC function pattern:
```typescript
export async function startAudioRuntime(): Promise<SessionDocument> {
    return invokeSession("start_audio_runtime");
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add VisualRuntimeState, VisualRuntimeLifecycle, VisualRuntimeHealth, AgentRuntimeState to domain + create visual module skeleton</name>
  <files>src-tauri/src/domain/session.rs, src-tauri/src/visual/mod.rs, src-tauri/src/visual/adapter.rs, src-tauri/src/visual/runtime_manager.rs, src-tauri/src/visual/bevy_sidecar.rs, src-tauri/src/visual/compiler.rs</files>
  <action>
1. In `src-tauri/src/domain/session.rs`:
   - Add `VisualRuntimeLifecycle` enum: `Idle, Starting, Ready, Rendering, Failed` (derive Clone, Debug, Default=Idle, PartialEq, Serialize, Deserialize, TS, serde rename_all="snake_case").
   - Add `VisualRuntimeHealth` enum: `Unknown, Healthy, Degraded, Error` (derive same, Default=Unknown).
   - Add `VisualRuntimeState` struct: `lifecycle: VisualRuntimeLifecycle, health: VisualRuntimeHealth, active_scene_id: Option<String>, fps: Option<f32>, last_error: Option<String>, renderer: Option<String>` (derive same as AudioRuntimeState, serde rename_all="camelCase", Default).
   - Add `visual_runtime: VisualRuntimeState` field to SessionDocument with `#[serde(default)]`.
   - Add `AgentRuntimeState` struct: `is_available: bool, pending_action_count: u32, is_frozen: bool` (derive same, Default gives is_available=true, others 0/false).
   - Add `agent_runtime: AgentRuntimeState` field to SessionDocument with `#[serde(default)]`.
   - Add all new types to `write_generated_typescript_contract()` declarations array.
   - Add `RuntimeKind::Agent` entry to default session's `runtime_status` vec (currently only has Audio and Visual).

2. Create `src-tauri/src/visual/mod.rs` with `pub mod adapter; pub mod runtime_manager; pub mod bevy_sidecar; pub mod compiler;`.

3. Create `src-tauri/src/visual/adapter.rs`:
   - Define `VisualAdapterStatus` enum: `Booted { renderer: String }, SceneLoaded { scene_id: String }, Stopped, Panicked, Failed { message: String }`.
   - Define `VisualRuntimeAdapter` trait with methods: `start(&mut self) -> Result<VisualAdapterStatus, String>`, `load_scene(&mut self, scene: &CompiledVisualScene) -> Result<VisualAdapterStatus, String>`, `update_parameters(&mut self, params: &[VisualParameterUpdate]) -> Result<(), String>`, `stop(&mut self) -> Result<VisualAdapterStatus, String>`, `panic(&mut self) -> Result<VisualAdapterStatus, String>`.
   - Define `CompiledVisualScene` struct: `scene_id: String, background_color: [f32; 4], elements: Vec<CompiledVisualElement>`.
   - Define `CompiledVisualElement` struct: `element_id: String, element_type: String, position: [f32; 2], scale: f32, parameters: Vec<(String, f64)>`.
   - Define `VisualParameterUpdate` struct: `element_id: String, parameter_id: String, value: f64`.

4. Create `src-tauri/src/visual/runtime_manager.rs`:
   - Define `VisualRuntimeManager<A>` struct with `adapter: A`.
   - Implement `Default for VisualRuntimeManager<BevySidecarAdapter>` using `BevySidecarAdapter::default()`.
   - Implement `start`, `stop`, `panic` methods following the EXACT pattern from `AudioRuntimeManager`: take adapter via std::mem::take (NO — use &mut self pattern matching AudioRuntimeManager), call adapter, update session via store.mutate_current, update visual_runtime fields and RuntimeStatusRef for RuntimeKind::Visual.
   - `start` compiles visual scene via `compile_session_to_visual_scene`, sets lifecycle to Starting, calls adapter.start(), then adapter.load_scene(), sets lifecycle to Ready/Failed.
   - `stop` calls adapter.stop(), resets all VisualRuntimeState fields.
   - `panic` calls adapter.panic(), resets fields, increments panic count.
   - Helper functions: `set_visual_runtime_status` mirrors `set_runtime_status` but targets RuntimeKind::Visual.

5. Create `src-tauri/src/visual/bevy_sidecar.rs`:
   - Define `BevySidecarAdapter` struct with `process: Option<Child>` (same as SuperColliderAdapter).
   - Implement `VisualRuntimeAdapter` for `BevySidecarAdapter`:
     - `start`: Look for `SCRYSYNTH_BEVY_PATH` env var or `scrysynth-visual` on PATH. If not found, return `Failed` with message "visual runtime binary not found". If found, spawn as child process. Return `Booted { renderer: "bevy" }`.
     - `load_scene`: Return `SceneLoaded { scene_id }` (no actual IPC in v1 — stub that acknowledges).
     - `update_parameters`: Return Ok(()) (stub for v1).
     - `stop`/`panic`: Kill child process, set process to None. Follow terminate_process pattern from supercollider.rs.
   - Implement `Drop` to clean up process on drop.
   - Use same `resolve_executable` and `terminate_process` patterns from supercollider.rs.

6. Create `src-tauri/src/visual/compiler.rs`:
   - Define `compile_session_to_visual_scene(session: &SessionDocument) -> CompiledVisualScene`.
   - Build a minimal visual scene from session state: background_color [0.0, 0.0, 0.0, 1.0] (black), one element per enabled node (shape type based on node_type), parameters derived from node parameters.
   - This is intentionally simple for v1 — no visual graph editing, no complex element types.
  </action>
  <verify>
    <automated>cd src-tauri && cargo test --lib -- domain::session::tests --nocapture 2>&1 | tail -20</automated>
  </verify>
  <done>VisualRuntimeState, VisualRuntimeLifecycle, VisualRuntimeHealth, AgentRuntimeState added to domain. visual/ module created with adapter trait, runtime manager, bevy sidecar adapter, and compiler. All types registered for ts-rs generation. Default session includes Agent runtime status.</done>
</task>

<task type="auto">
  <name>Task 2: Wire visual runtime lifecycle into SessionStore and Tauri commands, add IPC + Zustand actions, build RuntimeHealthPanel UI</name>
  <files>src-tauri/src/application/session_store.rs, src-tauri/src/application/mod.rs, src-tauri/src/lib.rs, src/lib/session-client.ts, src/store/sessionStore.ts, src/store/session-projections.ts, src/components/workspace/RuntimeHealthPanel.tsx, src/components/workspace/WorkspaceViewSwitcher.tsx</files>
  <action>
1. In `src-tauri/src/application/session_store.rs`:
   - Add `visual_runtime_manager: VisualRuntimeManager` field to SessionStore.
   - Initialize in `new_default()` with `VisualRuntimeManager::default()`.
   - Add `start_visual_runtime`, `stop_visual_runtime`, `panic_visual_runtime` methods following the EXACT pattern of the audio equivalents: `std::mem::take` manager, call method, put back, return result.
   - Add `derive_agent_runtime_state(&self)` method that computes `AgentRuntimeState` from session: `is_available = true`, `pending_action_count = session.pending_actions.len() as u32`, `is_frozen = session.agent_frozen`.

2. In `src-tauri/src/application/mod.rs`: no changes needed (visual is a separate top-level module).

3. In `src-tauri/src/lib.rs`:
   - Add `pub mod visual;` at top.
   - Add three Tauri commands: `start_visual_runtime`, `stop_visual_runtime`, `panic_visual_runtime` — exact same pattern as audio commands, delegating to store methods.
   - Add `get_agent_runtime_state` command that returns the derived AgentRuntimeState.
   - Register all four in `invoke_handler`.

4. In `src/lib/session-client.ts`:
   - Add zod schemas for `visualRuntimeStateSchema` (lifecycle enum, health enum, activeSceneId nullable string, fps nullable number, lastError nullable string, renderer nullable string) and `agentRuntimeStateSchema` (isAvailable bool, pendingActionCount number, isFrozen bool).
   - Add `visualRuntime` field to `sessionDocumentSchema` using the new schema.
   - Add `agentRuntime` field to `sessionDocumentSchema` using the new schema.
   - Add IPC functions: `startVisualRuntime`, `stopVisualRuntime`, `panicVisualRuntime`, `getAgentRuntimeState`.

5. In `src/store/sessionStore.ts`:
   - Add imported types for the new runtime state.
   - Add `visualRuntime` and `agentRuntime` to the store type (derived from session).
   - Add `applySession` derivation for `visualRuntime` and `agentRuntime` from the session document.
   - Add store actions: `startVisual`, `stopVisual`, `panicVisual` — following exact pattern of `startAudio`/`stopAudio`/`panicAudio`.

6. In `src/store/session-projections.ts`:
   - Add `VisualRuntimeProjection` type: `lifecycle, health, status: string, detail: string, canStart: boolean, canStop: boolean`.
   - Add `AgentRuntimeProjection` type: `isAvailable: boolean, pendingActionCount: number, isFrozen: boolean, status: string`.
   - Add `projectVisualRuntime(session)` function mirroring `projectAudioRuntime`.
   - Add `projectAgentRuntime(session)` function deriving from session fields.
   - Include both projections in `SessionProjection` type and `projectSessionState`.

7. Create `src/components/workspace/RuntimeHealthPanel.tsx`:
   - Accept session as prop (or use useSessionStore).
   - Render a panel with three sections: Audio Runtime, Visual Runtime, Agent System.
   - Each section shows: status indicator (colored dot: green=healthy/ready, yellow=degraded/starting, red=error/failed, gray=disconnected/idle), lifecycle label, health label, detail text, last error if any.
   - Audio section: shows lifecycle, health, sample rate, patch ID, start/stop/panic buttons (reuse existing logic).
   - Visual section: shows lifecycle, health, FPS (when rendering), renderer, start/stop buttons.
   - Agent section: shows available status, pending action count, frozen state.
   - Use inline styles consistent with existing workspace components (dark theme, same colors from session-projections.ts).
   - Export as named export.

8. In `src/components/workspace/WorkspaceViewSwitcher.tsx`:
   - Import and render `RuntimeHealthPanel` in the footer area or as a collapsible panel accessible from any view.
   - Place it below the main content area, visible across all workspace views.
  </action>
  <verify>
    <automated>cd src-tauri && cargo test --lib -- application::session_store::tests --nocapture 2>&1 | tail -20</automated>
  </verify>
  <done>Visual runtime lifecycle commands (start/stop/panic) wired through Tauri IPC → SessionStore → VisualRuntimeManager. RuntimeHealthPanel shows audio, visual, and agent status. Zustand store exposes visual runtime actions. All session projections include visual and agent runtime state.</done>
</task>

<task type="auto">
  <name>Task 3: Write integration tests for visual runtime lifecycle, compiler, and runtime health data</name>
  <files>src-tauri/tests/visual_runtime.rs</files>
  <action>
Create integration tests in `src-tauri/tests/visual_runtime.rs`:

1. Test visual runtime state defaults correctly (Idle lifecycle, Unknown health, no active scene).
2. Test SessionDocument serialization round-trip with visual_runtime and agent_runtime fields.
3. Test compile_session_to_visual_scene produces a scene from a session with enabled nodes.
4. Test compile_session_to_visual_scene handles empty sessions (no enabled nodes → empty elements).
5. Test VisualRuntimeManager with a test adapter (not BevySidecarAdapter — create a simple struct that implements VisualRuntimeAdapter with controlled responses):
   - Test start succeeds: lifecycle goes Idle → Starting → Ready, RuntimeStatusRef for Visual updates to Ready.
   - Test start with adapter failure: lifecycle goes to Failed, error message captured.
   - Test stop: lifecycle returns to Idle, RuntimeStatusRef back to Disconnected.
   - Test panic: lifecycle returns to Idle, health shows recovery.
6. Test AgentRuntimeState derivation: is_frozen matches session.agent_frozen, pending_action_count matches session.pending_actions.len().
7. Test that TypeScript contract generation includes VisualRuntimeState, VisualRuntimeLifecycle, VisualRuntimeHealth, AgentRuntimeState types.

Create a simple `TestVisualAdapter` in the test file that implements `VisualRuntimeAdapter` with configurable behavior (success/failure).
  </action>
  <verify>
    <automated>cd src-tauri && cargo test --test visual_runtime --nocapture 2>&1 | tail -30</automated>
  </verify>
  <done>All integration tests pass. Visual runtime lifecycle transitions verified. Compiler output verified. Agent runtime state derivation verified. TypeScript contracts include new types.</done>
</task>

</tasks>

<verification>
1. `cd src-tauri && cargo test` — all domain, application, and integration tests pass.
2. `cd src-tauri && cargo build` — compiles without errors.
3. Visual runtime start/stop lifecycle works through Tauri commands.
4. RuntimeHealthPanel renders status for all three runtime systems.
5. Existing session files load correctly with new fields (backward compatible via serde defaults).
</verification>

<success_criteria>
- Visual runtime can be started and stopped through Tauri commands, with lifecycle tracked in SessionDocument.
- VisualRuntimeState, AgentRuntimeState, and RuntimeStatusRef update correctly during lifecycle transitions.
- RuntimeHealthPanel displays audio, visual, and agent status with colored indicators.
- Existing sessions load without errors (new fields have serde defaults).
- All integration tests pass.
</success_criteria>

<output>
After completion, create `.planning/phases/05-visual-sync-cross-modal/05-visual-sync-cross-modal-01-SUMMARY.md`
</output>
