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
  - src/App.tsx
  - src/App.css
  - src/components/session/GraphViewport.tsx
  - src/components/session/NodeInspector.tsx
  - src/components/session/SessionToolbar.tsx
  - src/lib/session-client.ts
  - src/store/sessionStore.ts
autonomous: false
requirements:
  - SESS-02
  - SESS-03
  - PERS-01
must_haves:
  truths:
    - User can inspect the current session as visible graph nodes and connections.
    - User can inspect a selected node's identity, type, ports, parameters, runtime target, scene membership, and ownership metadata.
    - User can trigger new, save, and open actions from the workspace against app-owned session commands.
  artifacts:
    - path: src/App.tsx
      provides: Phase 1 session workspace shell
      contains: "SessionToolbar"
    - path: src/components/session/GraphViewport.tsx
      provides: Visible graph rendering from canonical session data
      contains: "ReactFlow"
    - path: src/components/session/NodeInspector.tsx
      provides: Selected-node metadata inspector
      contains: "runtime target"
  key_links:
    - from: src/lib/session-client.ts
      to: src-tauri/src/lib.rs
      via: invoke calls for create/get/save/open session commands
      pattern: "invoke\(\"(create_default_session|get_current_session|save_session_to_path|open_session_from_path)\""
    - from: src/store/sessionStore.ts
      to: src/components/session/GraphViewport.tsx
      via: projected node and edge data
      pattern: "selectedNodeId|graphNodes|graphEdges"
    - from: src/components/session/NodeInspector.tsx
      to: src/store/sessionStore.ts
      via: selected canonical node lookup
      pattern: "selectedNode"
---

<objective>
Replace the starter screen with a read-first Scrysynth workspace that proves the canonical session is inspectable and recallable.

Purpose: Phase 1 is only complete when the user can see the session graph, inspect node metadata, and drive save/open through the app-owned backend.
Output: Session workspace UI, frontend session client/store, graph viewport, node inspector, and one final human verification checkpoint.
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
@package.json
@src/App.tsx
@src/App.css
@src/generated/session-types.ts

<interfaces>
The frontend will consume `SessionDocument` from `src/generated/session-types.ts` and call these Tauri commands from Plan 02:

```ts
create_default_session(): Promise<SessionDocument>
get_current_session(): Promise<SessionDocument>
save_session_to_path(path: string): Promise<void>
open_session_from_path(path: string): Promise<SessionDocument>
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add frontend session client and mirror store</name>
  <files>package.json, src/lib/session-client.ts, src/store/sessionStore.ts</files>
  <read_first>
    - package.json
    - src/generated/session-types.ts
    - src/App.tsx
    - .planning/phases/01-session-core-recall/01-RESEARCH.md
  </read_first>
  <action>Update `package.json` to add the exact Phase 1 frontend libraries `@xyflow/react`, `zustand`, `immer`, and `zod`. Create `src/lib/session-client.ts` with typed `invoke` wrappers for `create_default_session`, `get_current_session`, `save_session_to_path`, and `open_session_from_path`. Create `src/store/sessionStore.ts` as the single frontend mirror store holding `session`, `selectedNodeId`, derived `graphNodes`, derived `graphEdges`, `selectedNode`, and async actions `bootstrapSession`, `newSession`, `saveSession`, `openSession`, and `selectNode`. Parse backend payloads with `zod` before committing them to the store so invalid payloads fail fast at the UI boundary.</action>
  <acceptance_criteria>
    - package.json lists `@xyflow/react`
    - package.json lists `zustand`
    - src/lib/session-client.ts contains `createDefaultSession`
    - src/lib/session-client.ts contains `openSessionFromPath`
    - src/store/sessionStore.ts contains `graphNodes`
    - src/store/sessionStore.ts contains `selectedNode`
  </acceptance_criteria>
  <verify>
    <automated>npm run build</automated>
  </verify>
  <done>The frontend has one typed session client and one mirror store that turn canonical session data into graph and inspector projections.</done>
</task>

<task type="auto">
  <name>Task 2: Build the Phase 1 workspace with graph, inspector, and session actions</name>
  <files>src/App.tsx, src/App.css, src/components/session/GraphViewport.tsx, src/components/session/NodeInspector.tsx, src/components/session/SessionToolbar.tsx</files>
  <read_first>
    - src/App.tsx
    - src/App.css
    - src/store/sessionStore.ts
    - src/generated/session-types.ts
    - .planning/ROADMAP.md
  </read_first>
  <action>Replace the starter Tauri greeting UI with a desktop instrument workspace tailored to Scrysynth rather than generic defaults. Create `SessionToolbar` with `New Session`, `Save Session`, and `Open Session` actions plus the current session title. Create `GraphViewport` using `ReactFlow` in read-first mode to render nodes and routes from `graphNodes` and `graphEdges`, with click selection wired to `selectNode`. Create `NodeInspector` that shows the selected node's `id`, `node type`, `ports`, `parameters`, `runtime target`, `scene membership`, and `ownership` fields verbatim from canonical state. Update `src/App.css` to use a deliberate desktop layout with a toolbar row, graph canvas center, inspector side panel, and a small runtime-status strip that reads from canonical placeholder runtime states; avoid keeping any Tauri/Vite/React starter branding or purple-on-white defaults.</action>
  <acceptance_criteria>
    - src/App.tsx contains `SessionToolbar`
    - src/App.tsx contains `GraphViewport`
    - src/App.tsx contains `NodeInspector`
    - src/components/session/GraphViewport.tsx contains `ReactFlow`
    - src/components/session/NodeInspector.tsx contains `runtime target`
    - src/components/session/NodeInspector.tsx contains `scene membership`
    - src/App.tsx no longer contains `Welcome to Tauri + React`
  </acceptance_criteria>
  <verify>
    <automated>npm run build</automated>
  </verify>
  <done>The app opens into a Scrysynth session workspace where the graph is visible, node metadata is inspectable, and session lifecycle actions are wired to backend commands.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 3: Verify the workspace proves inspectable recall</name>
  <action>Launch the Tauri app after Tasks 1 and 2 complete and verify that the seeded canonical session is visible, inspectable, and recallable through the new workspace rather than the starter template.</action>
  <what-built>Phase 1 workspace with canonical graph rendering, node inspector, and new/save/open actions wired to the Rust session commands</what-built>
  <how-to-verify>
    1. Run `npm run tauri dev`.
    2. Confirm the app opens to a session workspace instead of the Tauri starter screen.
    3. Confirm the graph viewport shows the seeded sample nodes and route.
    4. Click a node and confirm the inspector shows identity, type, ports, parameters, runtime target, scene membership, and ownership metadata.
    5. Save the session, close the app, reopen it, and open the saved file; confirm the same graph and inspector data return.
  </how-to-verify>
  <verify>
    <automated>npm run tauri dev</automated>
  </verify>
  <done>The user has confirmed that the graph, inspector, and save/open recall flow behave as described or has returned a precise defect list.</done>
  <resume-signal>Type "approved" or describe the issue precisely.</resume-signal>
</task>

</tasks>

<verification>
Build the frontend, then perform one end-to-end human verification pass covering graph visibility, inspector fidelity, and save/open recall from the workspace.
</verification>

<success_criteria>
Phase 1 UI coverage is complete when the starter app is gone, the canonical session is visible as a graph, the selected node exposes all required metadata, and save/open is reachable from the workspace.
</success_criteria>

<output>
After completion, create `.planning/phases/01-session-core-recall/01-session-core-recall-03-SUMMARY.md`
</output>
