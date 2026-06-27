# Phase 13: Graph UX Rebuild - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-27
**Phase:** 13-Graph UX Rebuild
**Areas discussed:** Position storage shape, Port surface & edge model, Ownership canvas visibility, Drag & reroute feel

---

## Position storage shape

### Q1 — Where should node positions live in canonical state?

| Option | Description | Selected |
|--------|-------------|----------|
| Separate layout map | `node_layout: HashMap<node_id, {x,y}>` on SessionDocument, outside the audio Node struct. Pure layout; doesn't pollute compiler/synthdef dispatch; agent mutations stay layout-agnostic; aligns with MoveNode-bypasses-reconcile. Two places to sync on node add/remove. | ✓ |
| Position on Node struct | `position: {x,y}` directly on the `Node` struct. Single source, simpler serialization. But positions pollute audio-graph identity the compiler consumes; every agent mutation must decide a position. | |
| You decide | Agent discretion. | |

**User's choice:** Separate layout map
**Notes:** Clean separation between audio identity (compiler-consumed) and spatial arrangement (performer-owned).

### Q2 — Should recalling a scene/variation restore node positions?

| Option | Description | Selected |
|--------|-------------|----------|
| Layout stays put | Scenes recall audio state only; positions are performer-owned and stay where you left them across scene switches. Keeps SceneDefinition lean. | ✓ |
| Scenes save positions too | Each scene snapshots node_layout and recall restores it. Couples spatial arrangement to musical recall; doubles scene state. | |
| You decide | Agent discretion. | |

**User's choice:** Layout stays put
**Notes:** Positions are performer property, not musical state.

### Q3 — Where does a new node initially appear?

| Option | Description | Selected |
|--------|-------------|----------|
| Viewport center | Drops at current viewport center with cascading offset for rapid adds. Predictable, always visible, matches Bespoke/VCV/Blender. | ✓ |
| Keep auto-grid | v1 index-based grid (x: 80 + (index%3)*260). Deterministic but ignores where the performer is looking — new nodes may appear off-screen. | |
| You decide | Agent discretion. | |

**User's choice:** Viewport center

### Q4 — How does a multi-selection drag commit?

| Option | Description | Selected |
|--------|-------------|----------|
| One batch command | MoveNode accepts a list of {node_id, position} pairs — atomic undo, clean history, fewer IPC calls. Matches ActionHistoryEntry. | ✓ |
| One command per node | Each node fires its own MoveNode. Simpler type, but N history entries + N IPC round-trips. | |
| You decide | Agent discretion. | |

**User's choice:** One batch command

---

## Port surface & edge model

### Q1 — How should typed ports surface on the canvas?

| Option | Description | Selected |
|--------|-------------|----------|
| Visible port handles | Each catalog port renders as a xyflow Handle on a custom node body; edges connect port-to-port (sourcePortId/targetPortId become real). Modular-synth feel; catalog per-param CV ports become drawable targets. | ✓ |
| Node-level edges | Keep node-to-node edges with isValidConnection validation. Less clutter but performer can't target specific CV params. | |
| You decide | Agent discretion. | |

**User's choice:** Visible port handles

### Q2 — Which port-type connections are valid?

| Option | Description | Selected |
|--------|-------------|----------|
| Same-type only | Audio→Audio and Control→Control only. Simplest, but breaks audio-rate FM (oscillator frequency_cv Audio-in unreachable). | |
| Same-type + audio→ctrl | Adds Audio-out → Control-in for audio-rate modulation; only Control→Audio rejected. Matches Phase 12 D-03. | |
| You decide | Agent discretion. | ✓ |

**User's choice:** You decide
**Notes:** Hard constraint captured: the matrix MUST preserve audio-rate FM (Phase 12 D-03 — oscillator frequency_cv is Audio-typed). Researcher derives from catalog signal_types + validate_route.

### Q3 — How are audio vs control ports visually distinguished?

| Option | Description | Selected |
|--------|-------------|----------|
| Color + shape | Audio = filled circles (cyan); Control/CV = smaller diamonds or hollow (amber/violet). Shape differs so colorblind-considerate. Bespoke/VCV convention. | ✓ |
| Position + labels | Outputs right, inputs left, grouped with text labels. No color coding — slower visual scan during live patching. | |
| You decide | Agent discretion. | |

**User's choice:** Color + shape

### Q4 — What connection cardinality do ports allow?

| Option | Description | Selected |
|--------|-------------|----------|
| Out many, in one | Outputs fan out; each input accepts ONE edge (new replaces old). Classic modular patching; matches AddRoute/RemoveRoute. | ✓ |
| Out many, in many | Inputs sum multiple incoming edges. More flexible but complicates validation and edge-delete semantics. Unusual for visual patchers. | |
| You decide | Agent discretion. | |

**User's choice:** Out many, in one

---

## Ownership canvas visibility (GRAPH-05)

### Q1 — How prominent should ownership/agent chrome be on canvas nodes?

| Option | Description | Selected |
|--------|-------------|----------|
| Inline chrome | Controller badge + lock/snowflake icon + approve/reject buttons INSIDE each node body. Most actionable (GRAPH-05 "visible AND actionable"); nodes get taller. | ✓ |
| Subtle tinting | Border color per controller + small badge; pending actions still need inspector. Cleaner, less actionable. | |
| Hybrid | Subtle always-on tinting + inline approve/reject ONLY on pending nodes. | |

**User's choice:** Inline chrome

### Q2 — Should edges also display ownership/pending state?

| Option | Description | Selected |
|--------|-------------|----------|
| Edges show state too | Agent-proposed route = dashed edge + midpoint approve/reject. Matches GRAPH-05 "nodes/edges" literally. | |
| Nodes only | Minimal edge chrome; ownership/pending on endpoint nodes + pending-actions panel. | |
| You decide | Agent discretion. | ✓ |

**User's choice:** You decide
**Notes:** Planner decides based on how PendingAction correlates to specific AddRoute/RemoveRoute vs whole-node edits.

### Q3 — Default interaction with agent-owned/frozen nodes?

| Option | Description | Selected |
|--------|-------------|----------|
| Drag = override | Direct manipulation implicitly reclaims; single gesture unfreezes. No modal lockout. Matches Control Safety constraint. | ✓ |
| Reclaim first | Must explicitly reclaim before editing agent-owned nodes. Adds a step to every override. | |
| You decide | Agent discretion. | |

**User's choice:** Drag = override
**Notes:** User was emphatic — override must be frictionless. Undo (action history) is the safety net for accidental edits.

### Q4 — How is a globally-frozen session signaled at canvas level?

| Option | Description | Selected |
|--------|-------------|----------|
| Canvas banner + badges | Frozen indicator bar across top of graph surface (or ice-tint canvas border) PLUS per-node snowflake badges. Hard to miss. | ✓ |
| Badges only | Per-node icons only; frozen session not obvious from canvas alone. Less noise. | |
| You decide | Agent discretion. | |

**User's choice:** Canvas banner + badges

---

## Drag & reroute feel

### Q1 — How should node dragging commit to canonical state?

| Option | Description | Selected |
|--------|-------------|----------|
| Optimistic + commit on release | xyflow moves node visually during drag via local state; MoveNode fires once on mouseup. 60fps, zero IPC during drag, one history entry. How React Flow is designed to be used. | ✓ |
| Command-per-move | Every mousemove fires MoveNode. Live canonical sync but N IPC + N history entries; fence churn risk even though MoveNode bypasses reconcile. | |
| You decide | Agent discretion. | |

**User's choice:** Optimistic + commit on release

### Q2 — How should a performer reroute an existing edge?

| Option | Description | Selected |
|--------|-------------|----------|
| Drag edge endpoint | xyflow onReconnect — grab either end, drop on new port. One fluent gesture; RemoveRoute+AddRoute as one typed sequence (atomic undo). | ✓ |
| Delete + redraw | Select edge, Delete, then drag new cable. 2-gesture reroute; less fluent. | |
| Both | Support drag-endpoint AND delete-then-redraw. Most flexible, more surface. | |
| You decide | Agent discretion. | |

**User's choice:** Drag edge endpoint

### Q3 — Selection model and keyboard shortcuts?

| Option | Description | Selected |
|--------|-------------|----------|
| Full patching keyset | Rubber-band box-select + Shift-click; Delete, Cmd+Z/Cmd+Shift+Z (via ActionHistoryEntry), Cmd+C/V, Cmd+A, Cmd+D. Covers GRAPH-04 + extras. | ✓ |
| GRAPH-04 minimum | Multi-select + Delete, Cmd+Z, Cmd+C/V only. Skip select-all/duplicate/redo. | |
| You decide | Agent discretion. | |

**User's choice:** Full patching keyset

### Q4 — What does copy/paste capture, and what ownership does a pasted node get?

| Option | Description | Selected |
|--------|-------------|----------|
| Subgraph + reset ownership | Nodes + internal routes + current param values; new ids; routes re-attached; ownership resets to 'user'. "Copy a voice/channel" workflow. | ✓ |
| Nodes only, keep ownership | No routes/params copied; ownership preserved. Loses modulation — less useful. | |
| You decide | Agent discretion. | |

**User's choice:** Subgraph + reset ownership

---

## the agent's Discretion

- Port-type validation matrix (D-06) — derive from catalog signal_types + validate_route; preserve audio-rate FM.
- Edge ownership/pending chrome (D-10) — decide based on PendingAction correlation to routes vs nodes.
- `AddRoute`-to-occupied-input behavior (from D-08) — error vs auto-replace; keep canvas + Rust consistent.
- Exact `MoveNode` command naming/shape (D-04).
- Custom node component structure (D-05/D-07/D-09).
- Property-test framework/scope (GRAPH-05 #5).
- Undo wiring (D-15) — reuse ActionHistoryEntry if possible.

## Deferred Ideas

None raised — discussion stayed within phase scope. Related-but-deferred items noted for other phases:

- Live streaming agent proposal previews on canvas → Phase 16 (AGENT-04).
- Auto-layout / auto-arrange button (elkjs) → not required by GRAPH-01..05; available later.
- Edge animation showing active signal flow → visual polish, not Phase 13.
