---
phase: 05-visual-sync-cross-modal
plan: 02
type: execute
wave: 2
depends_on:
  - 05-visual-sync-cross-modal-01
files_modified:
  - src-tauri/src/domain/session.rs
  - src-tauri/src/application/macro_command.rs
  - src-tauri/src/application/performance_command.rs
  - src-tauri/src/application/mod.rs
  - src-tauri/src/application/session_store.rs
  - src-tauri/src/lib.rs
  - src/lib/session-client.ts
  - src/store/sessionStore.ts
  - src/store/session-projections.ts
  - src/components/workspace/MacroEditor.tsx
  - src/components/workspace/MacroSlider.tsx
  - src/components/workspace/PerformanceView.tsx
  - src-tauri/tests/macro_commands.rs
autonomous: true
requirements:
  - CTRL-01
must_haves:
  truths:
    - User can create a macro that targets both audio parameters (node_id + parameter_id) and visual parameters (element_id + parameter_id) through a unified MacroTarget addressing scheme.
    - User can adjust a macro value (0.0-1.0) and see both audio and visual parameters update simultaneously.
    - Macro definitions are persisted in the session document and survive save/load.
    - Existing macros with flat target_parameter_ids load correctly via backward-compatible migration (serde default).
    - Scene recall applies macro overrides to both audio and visual parameters.
  artifacts:
    - path: src-tauri/src/domain/session.rs
      provides: MacroTarget enum, expanded MacroDefinition with targets field
      contains: "MacroTarget"
    - path: src-tauri/src/application/macro_command.rs
      provides: MacroCommand enum with CreateMacro, UpdateMacro, RemoveMacro, SetMacroValue
      contains: "apply_macro_command"
    - path: src/components/workspace/MacroEditor.tsx
      provides: UI for creating and editing macro definitions
      contains: "MacroEditor"
    - path: src/components/workspace/MacroSlider.tsx
      provides: Live performance macro control slider
      contains: "MacroSlider"
    - path: src-tauri/tests/macro_commands.rs
      provides: Integration tests for macro CRUD, live values, cross-domain targeting
      contains: "apply_macro_command"
  key_links:
    - from: src-tauri/src/application/macro_command.rs
      to: src-tauri/src/domain/session.rs
      via: Mutates session.macros and applies scaled values to node parameters and visual parameters
      pattern: "MacroTarget::AudioParameter"
    - from: src/components/workspace/MacroSlider.tsx
      to: src/store/sessionStore.ts
      via: Calls setMacroValue action which invokes IPC
      pattern: "setMacroValue.*macro_id"
    - from: src-tauri/src/application/performance_command.rs
      to: src-tauri/src/application/macro_command.rs
      via: Scene recall applies macro overrides using the expanded macro resolution
      pattern: "apply_macro_override.*MacroTarget"
---

<objective>
Expand the macro system to support cross-domain parameter targeting (audio + visual) with CRUD commands, live value control, and a UI for macro editing and performance sliders.

Purpose: Enables the core cross-modal control feature — one macro knob that moves both audio and visual parameters simultaneously.

Output: MacroTarget enum, expanded MacroDefinition, macro CRUD commands, MacroEditor/MacroSlider UI components, integration tests.
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
<!-- Current MacroDefinition — needs expansion -->

From src-tauri/src/domain/session.rs:
```rust
pub struct MacroDefinition {
    pub id: String,
    pub name: String,
    pub target_parameter_ids: Vec<String>,  // flat strings — needs expansion
    pub range_start: f64,
    pub range_end: f64,
}
```

<!-- Current macro resolution in performance_command.rs -->
```rust
fn apply_macro_override(session: &mut SessionDocument, macro_override: &MacroOverride) -> Result<(), PerformanceCommandError> {
    let macro_def = session.macros.iter().find(|m| m.id == macro_override.macro_id).cloned();
    let scaled_value = macro_def.range_start + (macro_override.value * (macro_def.range_end - macro_def.range_start));
    for target_param_id in &macro_def.target_parameter_ids {
        for node in &mut session.nodes {
            for parameter in &mut node.parameters {
                if parameter.id == *target_param_id {
                    parameter.value = scaled_value.clamp(parameter.min_value, parameter.max_value);
                }
            }
        }
    }
}
```

<!-- Visual runtime types from Plan 01 -->
```rust
pub struct VisualRuntimeState { lifecycle: VisualRuntimeLifecycle, health: VisualRuntimeHealth, active_scene_id: Option<String>, fps: Option<f32>, last_error: Option<String>, renderer: Option<String> }
```

<!-- SessionDocument macro field and SceneDefinition -->
```rust
pub struct SessionDocument { ... pub macros: Vec<MacroDefinition>, pub scenes: Vec<SceneDefinition>, ... }
pub struct SceneDefinition { pub id: String, pub name: String, pub active_node_ids: Vec<String>, pub macro_overrides: Vec<MacroOverride> }
pub struct MacroOverride { pub macro_id: String, pub value: f64 }
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add MacroTarget enum, expand MacroDefinition, create macro_command.rs with CRUD + live value</name>
  <files>src-tauri/src/domain/session.rs, src-tauri/src/application/macro_command.rs, src-tauri/src/application/performance_command.rs, src-tauri/src/application/mod.rs, src-tauri/src/application/session_store.rs, src-tauri/src/lib.rs</files>
  <action>
1. In `src-tauri/src/domain/session.rs`:
   - Add `MacroTarget` tagged enum:
     ```rust
     #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
     #[serde(tag = "kind", content = "config", rename_all = "camelCase")]
     pub enum MacroTarget {
         AudioParameter { node_id: String, parameter_id: String },
         VisualParameter { element_id: String, parameter_id: String },
     }
     ```
   - Expand `MacroDefinition`: Add `targets: Vec<MacroTarget>` field with `#[serde(default)]` for backward compatibility. Keep `target_parameter_ids` with `#[serde(default)]` and `#[serde(skip_serializing_if = "Vec::is_empty")]` — it becomes a deprecated field. New macros use `targets`; old macros loaded from files use `target_parameter_ids` (migrated lazily on next save).
   - Add `MacroCommand` enum:
     ```rust
     #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
     #[serde(tag = "type", content = "payload", rename_all = "camelCase")]
     pub enum MacroCommand {
         CreateMacro { definition: MacroDefinition },
         UpdateMacro { macro_id: String, name: Option<String>, targets: Option<Vec<MacroTarget>>, range_start: Option<f64>, range_end: Option<f64> },
         RemoveMacro { macro_id: String },
         SetMacroValue { macro_id: String, value: f64 },
     }
     ```
   - Register `MacroTarget` and `MacroCommand` in `write_generated_typescript_contract()`.

2. Create `src-tauri/src/application/macro_command.rs`:
   - Define `MacroCommandError` enum (DuplicateMacro, MissingMacro, ParameterOutOfRange).
   - Implement `apply_macro_command(store: &mut SessionStore, command: MacroCommand) -> Result<SessionDocument, MacroCommandError>`:
     - `CreateMacro`: Validate no duplicate ID, push to session.macros, sort by ID.
     - `UpdateMacro`: Find by ID, update name/targets/range if provided.
     - `RemoveMacro`: Find by ID, remove from session.macros. Also remove any MacroOverrides in scenes that reference this macro_id.
     - `SetMacroValue`: Resolve macro, compute scaled value, apply to ALL targets:
       - For `MacroTarget::AudioParameter { node_id, parameter_id }`: find node, find parameter, clamp and set value.
       - For `MacroTarget::VisualParameter { element_id, parameter_id }`: update in session.visual_runtime (store visual parameter overrides in a new field — or simply log for now since visual adapter receives compiled scenes). For v1, visual parameter targeting records the value but actual visual update happens on next scene compile.
     - For backward compatibility: if macro has `target_parameter_ids` but empty `targets`, auto-migrate by creating `MacroTarget::AudioParameter` entries from the flat IDs (best-effort — use parameter IDs to look up which node they belong to).
   - Extract `resolve_macro_and_apply_value` helper function from the existing `apply_macro_override` in performance_command.rs.

3. In `src-tauri/src/application/performance_command.rs`:
   - Update `apply_macro_override` to use the new `MacroTarget`-aware resolution when `targets` is non-empty. Fall back to the old flat `target_parameter_ids` path when `targets` is empty (backward compat).
   - The new path iterates `macro_def.targets` and dispatches based on MacroTarget kind.

4. In `src-tauri/src/application/mod.rs`: Add `pub mod macro_command;`.

5. In `src-tauri/src/lib.rs`:
   - Add `apply_macro_command` Tauri command: takes MacroCommand, acquires store lock, calls `macro_command::apply_macro_command`, returns updated SessionDocument.
   - Register in invoke_handler.
  </action>
  <verify>
    <automated>cd src-tauri && cargo test --lib -- application::macro_command --nocapture 2>&1 | tail -20</automated>
  </verify>
  <done>MacroTarget enum supports AudioParameter and VisualParameter addressing. MacroDefinition expanded with targets field. MacroCommand provides Create/Update/Remove/SetMacroValue. Performance scene recall uses MacroTarget-aware resolution. Backward compatibility maintained via serde default on targets.</done>
</task>

<task type="auto">
  <name>Task 2: Wire macro IPC to frontend, add Zustand actions, create MacroEditor and MacroSlider components</name>
  <files>src/lib/session-client.ts, src/store/sessionStore.ts, src/store/session-projections.ts, src/components/workspace/MacroEditor.tsx, src/components/workspace/MacroSlider.tsx, src/components/workspace/PerformanceView.tsx</files>
  <action>
1. In `src/lib/session-client.ts`:
   - Add zod schemas for `macroTargetSchema` (discriminated union on "kind": "audioParameter" with {nodeId, parameterId}, "visualParameter" with {elementId, parameterId}).
   - Update `macroDefinitionSchema` to include `targets: z.array(macroTargetSchema).optional().default([])`.
   - Add `macroCommandSchema` as discriminated union: createMacro, updateMacro, removeMacro, setMacroValue.
   - Add IPC function: `applyMacroCommand(command: MacroCommand): Promise<SessionDocument>`.

2. In `src/store/sessionStore.ts`:
   - Import new types (MacroCommand, MacroTarget, MacroDefinition).
   - Add store actions:
     - `createMacro(definition: MacroDefinition)`: calls `applyMacroCommand({ type: "createMacro", payload: { definition } })`.
     - `updateMacro(macroId: string, updates: {...})`: calls `applyMacroCommand({ type: "updateMacro", payload: { macro_id: macroId, ...updates } })`.
     - `removeMacro(macroId: string)`: calls `applyMacroCommand({ type: "removeMacro", payload: { macro_id: macroId } })`.
     - `setMacroValue(macroId: string, value: number)`: calls `applyMacroCommand({ type: "setMacroValue", payload: { macro_id: macroId, value } })`.
   - All follow the standard pattern: set loading, call IPC, applySession.

3. In `src/store/session-projections.ts`:
   - Add `MacroProjection` type: `id, name, targets, rangeStart, rangeEnd, currentValues (derived from session state)`.
   - Add `projectMacros(session)` function that maps macro definitions to projections.
   - Add to SessionProjection.

4. Create `src/components/workspace/MacroEditor.tsx`:
   - Renders a list of all macros in the session.
   - Each macro shows: name, target list (audio params labeled with node:type, visual params labeled with element), range start/end.
   - "Add Macro" button: prompts for name, creates a macro with empty targets and range 0-1.
   - Each macro has "Edit" and "Delete" buttons.
   - Edit mode: text input for name, number inputs for range_start/range_end.
   - "Add Audio Target" dropdown: list all node parameters from session.nodes.
   - "Add Visual Target" dropdown: list elements from visual scene (if compiled).
   - Use inline styles consistent with workspace components.
   - Uses `useSessionStore` for state and actions.

5. Create `src/components/workspace/MacroSlider.tsx`:
   - Props: `macroId: string`.
   - Renders a horizontal slider (range input 0-1) for live macro value.
   - Displays macro name and current value.
   - `onChange` calls `useSessionStore().setMacroValue(macroId, value)`.
   - Debounce rapid changes (300ms) to avoid excessive IPC.
   - Use inline styles: dark background, accent-colored track, large enough for performance use.

6. In `src/components/workspace/PerformanceView.tsx`:
   - Import and render `MacroSlider` for each macro in the session.
   - Add a "Macros" section below the existing scene/variation controls.
   - Import and render `MacroEditor` in a collapsible panel (or as a tab within the performance view).
  </action>
  <verify>
    <automated>cd /home/lem/dev/scrysynth && npx tsc --noEmit 2>&1 | tail -20</automated>
  </verify>
  <done>Macro CRUD and live value control available through Zustand store. MacroEditor component allows creating/editing/deleting macros with cross-domain targets. MacroSlider component provides live performance control. Performance view shows macro sliders. TypeScript compiles without errors.</done>
</task>

<task type="auto">
  <name>Task 3: Write integration tests for macro CRUD, cross-domain targeting, and backward compatibility</name>
  <files>src-tauri/tests/macro_commands.rs</files>
  <action>
Create integration tests in `src-tauri/tests/macro_commands.rs`:

1. Test CreateMacro: creates a macro with MacroTarget::AudioParameter targets, verify it appears in session.macros.
2. Test CreateMacro with duplicate ID returns error.
3. Test UpdateMacro: change name and range, verify updates applied.
4. Test UpdateMacro with missing ID returns error.
5. Test RemoveMacro: verify removed from session.macros and scene macro_overrides cleaned up.
6. Test SetMacroValue with AudioParameter target: verify the target node parameter is updated with scaled+clamped value.
7. Test SetMacroValue with VisualParameter target: verify value is recorded (visual parameter update path).
8. Test SetMacroValue with multiple targets (audio + visual): verify all targets updated.
9. Test backward compatibility: create a session with old-style `target_parameter_ids` (empty targets), load it, apply SetMacroValue — verify the old path still works.
10. Test scene recall with macros: recall a scene with macro_overrides, verify both audio and visual parameters are updated according to the macro targets.
11. Test macro value scaling: range_start=0.2, range_end=0.8, value=0.5 → scaled to 0.5 (midpoint), value=0.0 → 0.2, value=1.0 → 0.8.
  </action>
  <verify>
    <automated>cd src-tauri && cargo test --test macro_commands --nocapture 2>&1 | tail -30</automated>
  </verify>
  <done>All macro integration tests pass. CRUD operations verified. Cross-domain targeting verified. Backward compatibility with old target_parameter_ids verified. Scene recall with macros verified.</done>
</task>

</tasks>

<verification>
1. `cd src-tauri && cargo test` — all tests pass including new macro tests.
2. `cd src-tauri && cargo build` — compiles without errors.
3. `npx tsc --noEmit` — TypeScript compiles.
4. Macro create/update/remove lifecycle works end-to-end.
5. MacroSlider adjusts live values affecting audio parameters.
6. Existing sessions with old-style macros load correctly.
</verification>

<success_criteria>
- User can create a macro with both AudioParameter and VisualParameter targets.
- Adjusting a macro value (via SetMacroValue or MacroSlider) updates all targeted parameters.
- Macros survive save/load with backward compatibility.
- Scene recall applies macro overrides to cross-domain targets.
- MacroEditor and MacroSlider render correctly in the performance view.
</success_criteria>

<output>
After completion, create `.planning/phases/05-visual-sync-cross-modal/05-visual-sync-cross-modal-02-SUMMARY.md`
</output>
