---
phase: 01-session-core-recall
plan: 03
type: execute
wave: 3
depends_on:
  - 01-session-core-recall-01
  - 01-session-core-recall-02
files_modified:
  - package.json
  - src-tauri/Cargo.toml
  - src-tauri/src/lib.rs
  - src/App.tsx
  - src/App.css
  - src/components/session/GraphViewport.tsx
  - src/components/session/NodeInspector.tsx
  - src/components/session/SessionToolbar.tsx
  - src/lib/session-client.ts
  - src/store/session-projections.ts
  - src/store/session-projections.test.ts
  - src/store/sessionStore.ts
autonomous: false
requirements:
  - SESS-02
  - SESS-03
  - PERS-01
must_haves:
  truths:
    - User can inspect the current session as visible graph nodes and connections.
    - User can inspect a selected node's identity, type, ports, parameters, runtime target, scene membership, and ownership metadata, and can also clear the selection.
    - User can trigger new, save, and open actions from the workspace through native desktop dialogs rather than browser prompts.
  artifacts:
    - path: src/App.tsx
      provides: Phase 1 session workspace shell
      contains: "SessionToolbar"
    - path: src/components/session/GraphViewport.tsx
      provides: Visible graph rendering from canonical session data
      contains: "ReactFlow"
    - path: src/components/session/NodeInspector.tsx
      provides: Selected-node metadata inspector and empty state
      contains: "Select a node"
    - path: src/store/session-projections.test.ts
      provides: Frontend projection and selection tests
      contains: "deriveSelectedNode"
  key_links:
    - from: src/lib/session-client.ts
      to: src-tauri/src/lib.rs
      via: invoke calls for create/get/save/open session commands
      pattern: "invoke\(\"(create_default_session|get_current_session|save_session_to_path|open_session_from_path)\""
    - from: src/store/sessionStore.ts
      to: src/components/session/GraphViewport.tsx
      via: projected node and edge data
      pattern: "graphNodes|graphEdges|selectedNodeId"
    - from: src/components/session/GraphViewport.tsx
      to: src/components/session/NodeInspector.tsx
      via: pane click clears selection and inspector shows empty state
      pattern: "onPaneClick"
---

<objective>
Replace the starter screen with a read-first Scrysynth workspace that proves the canonical session is inspectable and recallable.

Purpose: Phase 1 is only complete when the user can see the session graph, inspect node metadata, drive save/open through the app-owned backend, and do so with desktop-native file dialogs and test-covered selector behavior.
Output: Session workspace UI, frontend session client/store, tested graph projection helpers, native file dialogs, graph viewport, node inspector, and one final human verification checkpoint.
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
@.planning/phases/01-session-core-recall/01-session-core-recall-02-PLAN.md
@.planning/phases/01-session-core-recall/01-session-core-recall-REVIEWS.md
@package.json
@src-tauri/Cargo.toml
@src-tauri/src/lib.rs
@src/App.tsx
@src/generated/session-types.ts

<interfaces>
The frontend will consume `SessionDocument` from `src/generated/session-types.ts` and call these Tauri commands from Plan 02:

```ts
create_default_session(): Promise<SessionDocument>
get_current_session(): Promise<SessionDocument>
save_session_to_path(path: string): Promise<void>
open_session_from_path(path: string): Promise<SessionDocument>
```

The current UI uses prompt-driven paths in `src/App.tsx` and selection fallback logic in `src/store/sessionStore.ts`; this plan removes both behaviors per cross-review feedback.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Extract projection helpers and add frontend tests for graph selection behavior</name>
  <files>package.json, src/store/session-projections.ts, src/store/session-projections.test.ts, src/store/sessionStore.ts</files>
  <read_first>
    - package.json
    - src/store/sessionStore.ts
    - src/generated/session-types.ts
    - .planning/phases/01-session-core-recall/01-session-core-recall-REVIEWS.md
  </read_first>
  <behavior>
    - Test 1: `deriveSelectedNode(session, null)` returns `null`, making the inspector empty state reachable instead of falling back to the first node.
    - Test 2: `deriveSelectedNode(session, "missing-id")` returns `null` rather than silently substituting another node.
    - Test 3: projected graph edges mirror canonical routes and projected nodes preserve deterministic positions for the seeded graph.
  </behavior>
  <action>Add `vitest` to `package.json` and create `src/store/session-projections.ts` that exports pure `projectGraphNodes`, `projectGraphEdges`, `deriveSelectedNode`, and `applySession` helpers. Move the current projection logic out of `src/store/sessionStore.ts` into that file so it can be tested directly. Create `src/store/session-projections.test.ts` with fixtures built from the canonical `SessionDocument` shape and assert the exact selection semantics above, plus one route-to-edge projection assertion. Update `src/store/sessionStore.ts` to import those helpers and keep the store focused on async session actions. This task addresses the review concerns about missing frontend tests and the unreachable inspector empty state.</action>
  <acceptance_criteria>
    - package.json lists `vitest`
    - src/store/session-projections.ts contains `export function deriveSelectedNode`
    - src/store/session-projections.ts contains `export function projectGraphNodes`
    - src/store/session-projections.test.ts contains `deriveSelectedNode(session, null)`
    - src/store/sessionStore.ts imports from `./session-projections`
  </acceptance_criteria>
  <verify>
    <automated>npx vitest run src/store/session-projections.test.ts</automated>
  </verify>
  <done>The frontend graph/selection projection logic is isolated, test-covered, and no longer forces the inspector to show the first node when nothing is selected.</done>
</task>

<task type="auto">
  <name>Task 2: Build the workspace with native dialogs, clear deselection, and canonical inspection</name>
  <files>package.json, src-tauri/Cargo.toml, src-tauri/src/lib.rs, src/App.tsx, src/App.css, src/components/session/GraphViewport.tsx, src/components/session/NodeInspector.tsx, src/components/session/SessionToolbar.tsx, src/lib/session-client.ts, src/store/sessionStore.ts</files>
  <read_first>
    - package.json
    - src-tauri/Cargo.toml
    - src-tauri/src/lib.rs
    - src/App.tsx
    - src/App.css
    - src/components/session/GraphViewport.tsx
    - src/components/session/NodeInspector.tsx
    - src/components/session/SessionToolbar.tsx
    - src/lib/session-client.ts
    - src/store/sessionStore.ts
    - src/generated/session-types.ts
    - .planning/phases/01-session-core-recall/01-session-core-recall-REVIEWS.md
  </read_first>
  <action>Update `package.json` to add `@tauri-apps/plugin-dialog` and keep `@xyflow/react`, `zustand`, `immer`, and `zod` present. Update `src-tauri/Cargo.toml` and `src-tauri/src/lib.rs` to register the Tauri dialog plugin so the frontend can open native save/open pickers. Replace the `window.prompt` path flow in `src/App.tsx` with toolbar handlers that call store actions `saveSession()` and `openSession()` with no path argument. In `src/store/sessionStore.ts`, use `save` and `open` from `@tauri-apps/plugin-dialog` with a single-file JSON filter, abort cleanly on `null`, and pass the selected string path into the existing `save_session_to_path` and `open_session_from_path` invoke wrappers in `src/lib/session-client.ts`. Keep the workspace read-first: `GraphViewport` must render `graphNodes` and `graphEdges` through `ReactFlow`, select a node on click, and clear selection on pane click with `onPaneClick={() => onSelectNode(null)}`. Keep `NodeInspector` showing the selected node's `id`, `node type`, `ports`, `parameters`, `runtime target`, `scene membership`, and ownership fields verbatim from canonical state, and keep its existing empty-state copy visible when nothing is selected. Retain the deliberate desktop instrument layout in `src/App.css`; do not reintroduce any Tauri starter branding or browser-first prompt UX. This task addresses the consensus review concern about native file dialogs and the Claude review concern about selection semantics.</action>
  <acceptance_criteria>
    - package.json lists `@tauri-apps/plugin-dialog`
    - src-tauri/src/lib.rs contains `tauri_plugin_dialog`
    - src/App.tsx no longer contains `window.prompt`
    - src/store/sessionStore.ts contains `saveSession: () => Promise<void>`
    - src/store/sessionStore.ts contains `openSession: () => Promise<void>`
    - src/components/session/GraphViewport.tsx contains `onPaneClick`
    - src/components/session/NodeInspector.tsx contains `Select a node to inspect its canonical metadata.`
    - src/App.tsx no longer contains `Welcome to Tauri + React`
  </acceptance_criteria>
  <verify>
    <automated>npm run build</automated>
  </verify>
  <done>The app opens into a Scrysynth workspace where the graph is visible, node metadata is inspectable, save/open use native desktop dialogs, and clearing selection shows the inspector empty state.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 3: Verify inspectable recall with native dialogs and clear deselection</name>
  <files>src/App.tsx, src/components/session/GraphViewport.tsx, src/components/session/NodeInspector.tsx, src/components/session/SessionToolbar.tsx, src/store/sessionStore.ts</files>
  <read_first>
    - src/App.tsx
    - src/components/session/GraphViewport.tsx
    - src/components/session/NodeInspector.tsx
    - src/components/session/SessionToolbar.tsx
    - src/store/sessionStore.ts
  </read_first>
  <action>Launch the Tauri app after Tasks 1 and 2 complete and verify that the seeded canonical session is visible, inspectable, and recallable through the new workspace with native file dialogs instead of browser prompts.</action>
  <acceptance_criteria>
    - The running app shows the Scrysynth workspace instead of the starter screen.
    - Clicking a rendered node reveals identity, type, ports, parameters, runtime target, scene membership, and ownership metadata.
    - Clicking empty graph space returns the inspector to `Select a node to inspect its canonical metadata.`
    - `Save Session` and `Open Session` open native file dialogs instead of prompt boxes.
    - Reopening a saved file restores the same graph and inspector data.
  </acceptance_criteria>
  <what-built>Phase 1 workspace with canonical graph rendering, node inspector, native save/open dialogs, tested selection helpers, and toolbar actions wired to the Rust session commands</what-built>
  <how-to-verify>
    1. Run `npm run tauri dev`.
    2. Confirm the app opens to the Scrysynth workspace instead of the Tauri starter screen.
    3. Confirm the graph viewport shows the seeded sample nodes and route.
    4. Click a node and confirm the inspector shows identity, type, ports, parameters, runtime target, scene membership, and ownership metadata.
    5. Click empty graph space and confirm the inspector returns to the `Select a node` empty state.
    6. Trigger `Save Session` and `Open Session` and confirm native file dialogs appear instead of prompt boxes.
    7. Save the session, close the app, reopen it, and open the saved file; confirm the same graph and inspector data return.
  </how-to-verify>
  <verify>
    <automated>npm run tauri dev</automated>
  </verify>
  <done>The user has confirmed that graph visibility, node inspection, deselection, and save/open recall through native dialogs behave as described or has returned a precise defect list.</done>
  <resume-signal>Type "approved" or describe the issue precisely.</resume-signal>
</task>

</tasks>

<verification>
Run the frontend projection test file and production build, then perform one end-to-end human verification pass covering graph visibility, inspector fidelity, clear deselection, and native save/open recall from the workspace.
</verification>

<success_criteria>
Phase 1 UI coverage is complete when the starter app is gone, the canonical session is visible as a graph, the selected node exposes all required metadata, the inspector can clear to an empty state, and save/open use native desktop dialogs from the workspace.
</success_criteria>

<output>
After completion, create `.planning/phases/01-session-core-recall/01-session-core-recall-03-SUMMARY.md`
</output>
