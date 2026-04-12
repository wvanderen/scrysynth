---
phase: 03-performance-workspace
plan: 03
type: execute
wave: 2
depends_on:
  - 03-performance-workspace-02
files_modified:
  - src/components/workspace/PerformanceView.tsx
  - src/components/workspace/ScenePanel.tsx
  - src/components/workspace/VariationPanel.tsx
  - src/store/sessionStore.ts
  - src/App.css
  - src/store/session-projections.test.ts
autonomous: true
requirements:
  - CTRL-02
  - CTRL-03
must_haves:
  truths:
    - User can trigger scene recall from the performance view and see the session state update immediately.
    - User can save a variation of the current session state and see it appear in the variation list.
    - User can restore a saved variation and see parameters update across all views.
    - Scene recall disables non-scene nodes and enables scene nodes visibly in the graph.
    - All performance controls show loading and error states consistently with the rest of the workspace.
  artifacts:
    - path: src/components/workspace/ScenePanel.tsx
      provides: Scene recall UI with trigger buttons
      contains: "ScenePanel"
    - path: src/components/workspace/VariationPanel.tsx
      provides: Variation save/restore UI
      contains: "VariationPanel"
  tasks:
    - id: 1
      title: Create ScenePanel component
      action: implement
      file: src/components/workspace/ScenePanel.tsx
      description: Component that lists all scenes from the session, shows which nodes are active per scene, has a Recall button for each scene, and highlights the currently active scene. Derives active scene by checking which scene's activeNodeIds best matches the current enabled nodes.
    - id: 2
      title: Create VariationPanel component
      action: implement
      file: src/components/workspace/VariationPanel.tsx
      description: Component that lists variations for the active scene, has a Restore button per variation, and a Save Variation button that prompts for a name. Shows parameter override counts per variation.
    - id: 3
      title: Wire PerformanceView with ScenePanel and VariationPanel
      action: implement
      file: src/components/workspace/PerformanceView.tsx
      description: Import and render ScenePanel and VariationPanel in the performance view layout. Pass session data and callbacks from useSessionStore.
    - id: 4
      title: Add activeSceneId derivation to store or projections
      action: implement
      file: src/store/sessionStore.ts
      description: Derive activeSceneId from the current session by finding the scene whose activeNodeIds best matches the set of currently enabled nodes. Expose as a computed value.
    - id: 5
      title: Add frontend tests for performance workspace flows
      action: implement
      file: src/store/session-projections.test.ts
      description: Add tests for scene recall updating session state, variation save creating a new variation, and variation restore updating parameters. Test error handling for missing scene/variation IDs.
    - id: 6
      title: Add CSS for scene and variation panels
      action: implement
      file: src/App.css
      description: Add styles for scene panel cards, variation panel cards, active scene highlighting, and performance control buttons. Follow existing dark/gold theme.
---
