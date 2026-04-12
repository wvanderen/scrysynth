# Phase 3 Research: Performance Workspace

**Phase:** 3 (Performance Workspace)
**Researched:** 2026-04-11
**Goal:** Users can navigate the live instrument as one coherent workspace and recall structured performance states during a session.

## Requirements

| ID    | Description |
|-------|-------------|
| UI-01 | User can switch between conversation, graph, and performance views that all reflect the same live session state. |
| CTRL-02 | User can trigger scenes that recall predefined session states for live performance. |
| CTRL-03 | User can save a variation or snapshot of the current session and restore it later during the same working session. |

## What Phase 1 and Phase 2 Built

### Phase 1: Session Core & Recall
- Canonical `SessionDocument` in Rust with `Node`, `Route`, `Bus`, `MacroDefinition`, `SceneDefinition`, `VariationDefinition`, `OwnershipRule`, `RuntimeStatusRef`
- TypeScript contract generation via `ts-rs`
- Zustand mirror store projecting graph nodes, edges, selected node, and audio runtime
- JSON persistence with save/open via Tauri commands
- Graph viewport, node inspector, session toolbar

### Phase 2: Playable Audio Graph
- `GraphEditCommand` enum for add/remove/route/parameter/bus mutations
- Transactional `apply_graph_edit` with validation (cycle detection, port direction, signal type, range checks)
- Audio compiler (`compile_session_to_topology`) producing deterministic `CompiledTopology`
- `AudioRuntimeManager` with `start`, `stop`, `panic` lifecycle
- `SuperColliderAdapter` process supervision
- Frontend: `AudioTransportStrip`, `PrimitivePalette`, live edit integration
- All mutations flow through `store.mutate_current()` clone-and-replace pattern

## Existing Domain Types That Support Phase 3

The canonical session already defines:

```rust
pub struct SceneDefinition {
    pub id: String,
    pub name: String,
    pub active_node_ids: Vec<String>,
    pub macro_overrides: Vec<MacroOverride>,
}

pub struct MacroOverride {
    pub macro_id: String,
    pub value: f64,
}

pub struct VariationDefinition {
    pub id: String,
    pub name: String,
    pub scene_id: String,
    pub parameter_overrides: Vec<ParameterOverride>,
}

pub struct ParameterOverride {
    pub parameter_id: String,
    pub value: f64,
}
```

These types exist in the schema and round-trip through JSON, but Phase 2 did not add any commands or UI to *use* them. Phase 3 needs to make them operational.

## Research Findings

### 1. View Switching (UI-01)

**Current workspace layout:** Single flat layout with toolbar, transport strip, graph + inspector grid, and runtime footer. All rendered simultaneously.

**Needed:** A workspace where conversation, graph, and performance are distinct views sharing the same live session. The graph view already exists; conversation can be a placeholder for Phase 4; performance view needs scene/variation controls.

**Approach:**
- Add a `workspaceView` state to the frontend store: `'graph' | 'conversation' | 'performance'`
- Add a `WorkspaceViewSwitcher` component in the toolbar area
- Each view reads from the same `useSessionStore` — no separate state slices
- The graph view is the existing layout; performance view adds scene/variation panels; conversation view is a minimal placeholder
- The transport strip and runtime footer should remain visible across all views (they are global controls)

**Key insight:** The architecture already ensures all views share one canonical `SessionDocument` because the Zustand store is the single mirror. View switching is purely a UI concern — no backend changes needed for UI-01.

### 2. Scene Recall (CTRL-02)

**What a scene recall means in this codebase:**
A `SceneDefinition` stores:
- `active_node_ids`: which nodes should be enabled when the scene is active
- `macro_overrides`: macro values to apply when the scene is active

Recalling a scene should:
1. Enable nodes listed in `active_node_ids`
2. Disable all other nodes (nodes not in `active_node_ids` for that scene)
3. Apply macro override values to the corresponding macro definitions' target parameters

**Implementation approach (Rust backend):**
- Add a `PerformanceCommand` enum to the domain layer:
  ```rust
  pub enum PerformanceCommand {
      RecallScene { scene_id: String },
      SaveVariation { name: String, scene_id: String },
      RestoreVariation { variation_id: String },
  }
  ```
- `RecallScene` validates the scene exists, then:
  - Sets enabled=true for nodes in `active_node_ids`
  - Sets enabled=false for all other nodes
  - Applies macro overrides to their target parameters via the same parameter range validation used by `SetParameterValue`
- Add a `performance_command` Tauri IPC handler
- Return the updated `SessionDocument`

**Pitfall from PITFALLS.md (Pitfall 9):** Scene transitions need defined semantics. For v1, hard-cuts (immediate state swap) are the correct choice. Crossfading, morphing, and ramped transitions are deferred to Phase 5.

### 3. Variation Save and Restore (CTRL-03)

**What a variation means:**
A `VariationDefinition` stores parameter overrides relative to a scene. Saving a variation snapshots current parameter values. Restoring a variation applies those stored parameter values back.

**Save variation:**
1. Take the current session's node parameters
2. For each node in the current scene's `active_node_ids`, snapshot all parameter values
3. Store as a `VariationDefinition` with the scene reference

**Restore variation:**
1. Find the `VariationDefinition` by ID
2. For each `ParameterOverride`, find the corresponding parameter on the node and apply the value
3. Use the same range validation as `SetParameterValue`

**Implementation approach:**
- Add `SaveVariation` and `RestoreVariation` to `PerformanceCommand`
- `SaveVariation` creates a new `VariationDefinition` from current parameters
- `RestoreVariation` applies stored parameter overrides to the current session
- Both go through `store.mutate_current()` for transactional safety

### 4. Frontend Architecture for Performance View

**Performance view components needed:**
- `ScenePanel`: Lists scenes, shows which is active, triggers recall
- `VariationPanel`: Lists variations for the active scene, saves new variations, restores existing ones
- `PerformanceTransport`: Enhanced transport with scene recall buttons (could extend existing `AudioTransportStrip`)

**Store additions needed:**
- `recallScene(sceneId: string)`: IPC call to `recall_scene`
- `saveVariation(name: string, sceneId: string)`: IPC call to `save_variation`
- `restoreVariation(variationId: string)`: IPC call to `restore_variation`
- `activeSceneId` derived from current session state (which scene's nodes are enabled)

### 5. Technical Constraints

- **No new Rust dependencies needed** — all types exist, `serde`/`ts-rs` handle serialization
- **No new frontend dependencies needed** — React + Zustand + existing patterns handle view switching
- **Same Tauri IPC pattern** — new commands follow `#[tauri::command]` + `State<Mutex<SessionStore>>`
- **Same clone-and-replace mutation pattern** — `store.mutate_current()` ensures rejected commands never leak
- **Generated TypeScript types already include** `SceneDefinition`, `VariationDefinition`, `MacroOverride`, `ParameterOverride`

## Recommended Plan Breakdown

### Plan 1: Scene and Variation Backend Commands
- Add `PerformanceCommand` enum to domain
- Implement `recall_scene`, `save_variation`, `restore_variation` in application layer
- Add Tauri IPC handlers
- Add comprehensive Rust tests
- Update TypeScript contract generation

### Plan 2: View Switching Workspace Layout
- Add `workspaceView` state to frontend store
- Create `WorkspaceViewSwitcher` component
- Create placeholder `ConversationView` component
- Create `PerformanceView` shell component
- Restructure `App.tsx` to use view switching
- Add frontend tests

### Plan 3: Performance View Scene and Variation Controls
- Create `ScenePanel` component with scene recall buttons
- Create `VariationPanel` component with save/restore
- Wire to IPC via session client
- Add scene/variation actions to session store
- Derive active scene from session state
- Add frontend tests
