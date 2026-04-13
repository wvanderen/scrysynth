---
phase: 01-session-core-recall
plan: 04
type: execute
wave: 1
depends_on: []
files_modified:
  - src/store/session-projections.ts
autonomous: true
gap_closure: true
requirements: [SESS-02]

must_haves:
  truths:
    - "Default session renders nodes visually in the graph viewport"
    - "Each node displays its label text in the graph"
  artifacts:
    - path: "src/store/session-projections.ts"
      provides: "Graph node projections with data.label set"
      contains: "data.label"
  key_links:
    - from: "src/store/session-projections.ts"
      to: "@xyflow/react default node renderer"
      via: "data.label field on projected nodes"
      pattern: "data:.*label"
---

<objective>
Fix graph viewport rendering — nodes are invisible because React Flow's default node renderer requires `data.label` which is not set.

Purpose: The default session loads with nodes that are counted correctly but render as empty/invisible boxes because `projectGraphNodes` sets `data.title` and `data.subtitle` but omits `data.label`. React Flow's built-in node type renders `data.label` — without it, nothing appears inside the node.
Output: Patched `session-projections.ts` where `data.label` is populated on every projected node.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/01-session-core-recall/01-UAT.md
@.planning/phases/01-session-core-recall/01-session-core-recall-03-SUMMARY.md

<interfaces>
<!-- The exact code that needs changing -->

From src/store/session-projections.ts — the projectGraphNodes function (lines 88-117):
```typescript
function projectGraphNodes(session: SessionDocument, selectedNodeId: string | null): GraphNode[] {
  return session.nodes.map((node, index) => ({
    id: node.id,
    position: { ... },
    draggable: false,
    selectable: true,
    data: {
      title: labelForNode(node),
      subtitle: `${node.nodeType} / ${node.ownership.controller}`,
      isSelected: selectedNodeId === node.id,
      isEnabled: node.enabled,
      // MISSING: label — React Flow default node renderer needs this
    },
    style: { ... },
  }));
}
```

GraphNodeData type (lines 13-18):
```typescript
export type GraphNodeData = {
  title: string;
  subtitle: string;
  isSelected: boolean;
  isEnabled: boolean;
};
```

labelForNode helper (lines 138-141):
```typescript
function labelForNode(node: Node) {
  const primaryPort = node.ports[0]?.name ?? "portless";
  return `${node.nodeType}:${primaryPort}`;
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add data.label to projected graph nodes</name>
  <files>src/store/session-projections.ts</files>
  <action>
Two changes in `src/store/session-projections.ts`:

1. **Add `label` to the `GraphNodeData` type** (line 13-18). Add `label: string;` alongside the existing fields.

2. **Add `label` to the `data` object in `projectGraphNodes`** (line 97-102). Set `label` to the same value as `title` (which comes from `labelForNode(node)`). This gives React Flow's default node renderer the text it needs. The line should be:
   ```
   label: labelForNode(node),
   ```
   Place it right after the `title` field in the data object for readability.

Do NOT change GraphViewport.tsx — the default node type is fine once `data.label` is present. Do NOT create a custom node component — that would be over-engineering for this gap.
  </action>
  <verify>
    <automated>npx tsc --noEmit 2>&1 | head -20 && grep -c "label:" src/store/session-projections.ts</automated>
  </verify>
  <done>
    - `GraphNodeData` type includes `label: string`
    - `projectGraphNodes` sets `data.label` to `labelForNode(node)` on every node
    - TypeScript compiles without errors
    - Graph viewport renders node text when the app runs
  </done>
</task>

</tasks>

<verification>
```bash
# Type-check passes
npx tsc --noEmit

# Grep confirms label is set in data
grep "label:" src/store/session-projections.ts
```
</verification>

<success_criteria>
- `GraphNodeData` type has a `label: string` field
- `projectGraphNodes` populates `data.label` with the node's display text
- TypeScript compiles cleanly
- Running `npm run tauri dev` shows visible node labels in the graph viewport
</success_criteria>

<output>
After completion, create `.planning/phases/01-session-core-recall/01-session-core-recall-04-SUMMARY.md`
</output>
