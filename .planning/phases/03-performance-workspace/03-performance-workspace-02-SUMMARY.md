## Plan 02 Summary: View Switching Workspace Layout

**Completed:** 2026-04-11

### What was done
- Added `WorkspaceView` type (`'graph' | 'conversation' | 'performance'`) to session store
- Added `setWorkspaceView`, `recallScene`, `saveVariation`, `restoreVariation` actions to store
- Added `applyPerformanceCommand` IPC client function
- Created `WorkspaceViewSwitcher` component with tab-style navigation
- Created `ConversationView` placeholder showing shared session state
- Created `PerformanceView` shell with scene list, variation list, and save variation UI
- Restructured `App.tsx` to conditionally render views based on `workspaceView`
- Transport strip and runtime footer remain visible across all views
- Added CSS for view switcher, conversation view, performance view
- Updated frontend tests to include `workspaceView` in store reset

### Files modified
- `src/store/sessionStore.ts` — workspaceView state + performance actions
- `src/lib/session-client.ts` — applyPerformanceCommand IPC
- `src/components/workspace/WorkspaceViewSwitcher.tsx` — new
- `src/components/workspace/ConversationView.tsx` — new
- `src/components/workspace/PerformanceView.tsx` — new
- `src/App.tsx` — view switching layout
- `src/App.css` — view switcher + new view styles
- `src/store/session-projections.test.ts` — store reset update
