---
status: diagnosed
phase: 01-session-core-recall
source: 01-session-core-recall-01-SUMMARY.md, 01-session-core-recall-02-SUMMARY.md, 01-session-core-recall-03-SUMMARY.md
started: 2026-04-12T00:00:00Z
updated: 2026-04-12T00:15:00Z
---

## Current Test

[testing complete — diagnosed]

## Tests

### 1. Cold Start Smoke Test
expected: Kill any running server/service. Clear ephemeral state. Start the app from scratch via `npm run tauri dev`. Server boots without errors, the default session is created, and the workspace window appears showing the graph viewport.
result: pass

### 2. Default Session Loads on Start
expected: After launching, `get_current_session` returns a seeded default session with nodes, routes, buses, macros, scenes, and ownership metadata — not an empty document.
result: issue
reported: "Graph viewport appears and lists correctly number of notes but actual viewport is empty and shows no nodes"
severity: major

### 3. Graph Viewport Renders Canonical Nodes
expected: The workspace shows a React Flow graph viewport rendering the nodes from the default session. Nodes are visible and selectable by clicking.
result: blocked
blocked_by: prior-phase
reason: "Graph viewport is empty — nodes not rendered visually (same root issue as test 2)"

### 4. Node Inspector Shows Selected Node Details
expected: Click a node in the graph. The inspector panel updates to show that node's identity, type, ports, parameters, runtime target, scene membership, and ownership metadata.
result: blocked
blocked_by: prior-phase
reason: "Can't select individual nodes since graph doesn't render them. Inspector does show identity, ownership, ports, parameters, bus path for the default selected node."

### 5. Save Session to JSON File
expected: Click Save in the toolbar. Provide a file path. The session is saved as pretty JSON to that path with a valid schemaVersion field. No errors.
result: pass

### 6. Open Saved Session
expected: Click Open in the toolbar. Provide the path of a previously saved session file. The workspace graph and inspector update to reflect the loaded session data.
result: pass

### 7. New Session Replaces Current
expected: Click New in the toolbar. The current session is replaced with a fresh default session and the graph viewport updates accordingly.
result: pass

### 8. Save/Load Round-Trip Fidelity
expected: Save a session, open it again. All nodes, routes, buses, parameters, and ownership metadata are identical to what was saved — no data loss or drift.
result: blocked
blocked_by: prior-phase
reason: "this is blocked by the graph viewport bug"

### 9. Corrupt JSON Rejected Gracefully
expected: Manually edit a saved session file to contain invalid JSON. Attempt to open it. The app rejects the file with a clear error and does not replace the current session.
result: pass

### 10. Schema Version Mismatch Rejected
expected: Manually edit a saved session file to set an unsupported schemaVersion (e.g. "99.0.0"). Attempt to open it. The app rejects the file with a version mismatch error and does not replace the current session.
result: pass

## Summary

total: 10
passed: 6
issues: 1
pending: 0
skipped: 0
blocked: 3
skipped: 0
blocked: 0

## Gaps

- truth: "Default session renders nodes visually in the graph viewport"
  status: failed
  reason: "User reported: Graph viewport appears and lists correctly number of notes but actual viewport is empty and shows no nodes"
  severity: major
  test: 2
  root_cause: "projectGraphNodes creates nodes with custom data fields (title, subtitle, isSelected, isEnabled) but no React Flow custom node component is registered and data.label is not set. Default node renderer needs data.label to display anything."
  artifacts:
    - path: "src/store/session-projections.ts"
      issue: "projectGraphNodes sets data.title and data.subtitle but not data.label; no custom node type registered"
    - path: "src/components/session/GraphViewport.tsx"
      issue: "ReactFlow component uses default node types which require data.label"
  missing:
    - "Either add data.label to projectGraphNodes output, or register a custom node component that renders data.title/subtitle"
