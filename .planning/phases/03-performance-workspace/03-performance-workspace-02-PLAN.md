---
phase: 03-performance-workspace
plan: 02
type: execute
wave: 2
depends_on:
  - 03-performance-workspace-01
files_modified:
  - src/lib/session-client.ts
  - src/store/sessionStore.ts
  - src/components/workspace/WorkspaceViewSwitcher.tsx
  - src/components/workspace/ConversationView.tsx
  - src/components/workspace/PerformanceView.tsx
  - src/App.tsx
  - src/App.css
autonomous: true
requirements:
  - UI-01
must_haves:
  truths:
    - User can switch between conversation, graph, and performance views using a visible view switcher.
    - All three views read from the same live session state via useSessionStore.
    - The graph view preserves its existing functionality (node selection, graph editing, inspector).
    - The conversation view renders as a placeholder that shows it shares the same session.
    - The performance view renders with a scene list and transport controls.
    - Transport strip and runtime footer remain visible across all views.
  artifacts:
    - path: src/components/workspace/WorkspaceViewSwitcher.tsx
      provides: View switching tabs
      contains: "WorkspaceViewSwitcher"
    - path: src/components/workspace/ConversationView.tsx
      provides: Placeholder conversation view
      contains: "ConversationView"
    - path: src/components/workspace/PerformanceView.tsx
      provides: Performance view shell with scene list
      contains: "PerformanceView"
  tasks:
    - id: 1
      title: Add workspaceView state and view switching actions to session store
      action: implement
      file: src/store/sessionStore.ts
      description: Add workspaceView type ('graph' | 'conversation' | 'performance') to store state. Add setWorkspaceView action. Default to 'graph'.
    - id: 2
      title: Add performance IPC client functions
      action: implement
      file: src/lib/session-client.ts
      description: Add recallScene, saveVariation, restoreVariation functions that invoke the new Tauri commands and validate the returned SessionDocument.
    - id: 3
      title: Add scene and variation actions to session store
      action: implement
      file: src/store/sessionStore.ts
      description: Add recallScene, saveVariation, restoreVariation actions that call IPC and apply the updated session through applySession.
    - id: 4
      title: Create WorkspaceViewSwitcher component
      action: implement
      file: src/components/workspace/WorkspaceViewSwitcher.tsx
      description: Tab-style component with Graph, Conversation, and Performance tabs. Highlights active view. Calls setWorkspaceView on click.
    - id: 5
      title: Create ConversationView placeholder
      action: implement
      file: src/components/workspace/ConversationView.tsx
      description: Simple placeholder showing session title and a message that conversation with agents arrives in a future phase. Reads from useSessionStore to prove shared state.
    - id: 6
      title: Create PerformanceView shell
      action: implement
      file: src/components/workspace/PerformanceView.tsx
      description: Performance view layout with scene list, variation controls placeholder, and current session readout. Shows available scenes from session.scenes with recall buttons. Shows variations from session.variations. Reads all data from useSessionStore.
    - id: 7
      title: Restructure App.tsx for view switching
      action: implement
      file: src/App.tsx
      description: Add WorkspaceViewSwitcher to the toolbar area. Conditionally render ConversationView, existing graph layout, or PerformanceView based on workspaceView state. Keep transport strip and runtime footer outside the view switching area. Import new components.
    - id: 8
      title: Add CSS for view switcher and new views
      action: implement
      file: src/App.css
      description: Add styles for view-switcher tabs, conversation view placeholder, and performance view layout. Follow existing dark theme with gold accents.
---
