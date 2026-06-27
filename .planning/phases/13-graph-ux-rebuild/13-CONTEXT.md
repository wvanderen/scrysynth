# Phase 13: Graph UX Rebuild - Context

**Gathered:** 2026-06-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Rebuild the patch surface into a fluent, draggable, edge-reconnectable graph where every interaction flows through typed commands and persists in canonical `SessionDocument` state — load-bearing for the v2 "feel" and the surface that agent proposals, the node palette, and ownership badges render onto.

**In scope (GRAPH-01..05):**
- Free node dragging with positions persisted via a dedicated `SetNodeLayout`/`MoveNode` command that **bypasses audio reconciliation** (locked in STATE.md).
- Edge create / disconnect / reconnect / reroute by dragging between typed ports, with port-type validation at both canvas (`isValidConnection`) and Rust (`validate_route`).
- Multi-select (rubber-band + Shift-click) and keyboard shortcuts for common patching actions (add, delete, undo/redo, copy/paste, duplicate, select-all).
- Ownership, freeze, reclaim, and pending-approval states visible AND actionable directly on nodes/edges in the patch surface — not only in chat/inspector.
- xyflow canvas state always equals canonical `SessionDocument` state after every interaction (no orphan edges, no untracked moves) — verified by a property test.

**Out of scope (other phases):**
- Live streaming LLM agent proposal previews on the canvas → Phase 16 (AGENT-04). (Phase 13 only renders already-created `PendingAction`s on the canvas; the streaming preview is later.)
- Richer Bevy visuals behind the grid → Phase 14/15.
- Pro shell restructure (graph as hero, chat sidebar, progressive disclosure) → Phase 16.

**Depends on:** Phase 12 (custom typed-handle nodes need the `NodeCatalogEntry` catalog; per-param CV ports already declared).

</domain>

<decisions>
## Implementation Decisions

### Position Storage Shape
- **D-01: Separate layout map (NOT on the Node struct).** Node x/y positions live in a new `node_layout: HashMap<node_id, {x,y}>` field on `SessionDocument`, kept **outside** the audio `Node` struct. Positions are pure layout, not audio identity — they must not pollute the topology compiler, synthdef dispatch, or the catalog. This aligns with the locked STATE.md decision that `MoveNode` bypasses `AudioRuntimeManager.reconcile_graph_edit` (Pitfalls V2-2, V2-14). Tradeoff accepted: two places to keep in sync on node add/remove (the planner handles cleanup on `RemoveNode`).
- **D-02: Scenes/variations do NOT recall positions.** Scene recall restores audio state only (enabled nodes, params, routes). Node positions are the performer's spatial arrangement and stay put across scene switches. `SceneDefinition`/`VariationDefinition` do NOT snapshot `node_layout`. Matches "layout is performer-owned, not musical state."
- **D-03: New nodes spawn at viewport center.** When a performer adds a node from the palette, it drops at the center of the current xyflow viewport, with a small cascading offset if multiple are added rapidly (so they don't stack identically). Replaces v1's index-based auto-grid (`projectGraphNodes`: `x: 80 + (index%3)*260`).
- **D-04: Batch `MoveNode` command.** `MoveNode`/`SetNodeLayout` accepts a **list** of `{node_id, position}` pairs, not a single pair. A multi-selection drag commits ONE atomic command with all moved positions. Gives clean single-entry undo (matches the existing `ActionHistoryEntry` model) and minimizes IPC. The command shape: `GraphEditCommand::MoveNode { moves: Vec<NodeLayoutDelta> }` (exact name = planner's call).

### Port Surface & Edge Model
- **D-05: Visible typed port handles (modular-synth style).** Each catalog port renders as a visible xyflow `Handle` on a custom node body component. Edges connect **port-to-port** — `sourcePortId`/`targetPortId` (already on `Route` but ignored by v1 xyflow) become the real edge endpoints. This surfaces the catalog's per-parameter CV ports (Phase 12 D-04: e.g. a Filter's `cutoff_cv`, `resonance_cv`) as drawable modulation targets. Requires custom node components (v1 uses default rectangles) + Handle positioning logic.
- **D-06: Port-type validation matrix → agent discretion.** The researcher/planner derives the exact valid-connection matrix from the catalog's `SignalType` (Audio | Control) declarations and the existing `validate_route` (graph_edit.rs:273) + `validate_routes` (compiler.rs:230, incl. `PortSignalTypeMismatch`). **Hard constraint:** Phase 12 D-03 confirmed audio-rate modulation works end-to-end (the oscillator's `frequency_cv` is an `Audio`-typed input port reachable from audio-rate sources); the matrix MUST NOT regress audio-rate FM. Both canvas `isValidConnection` and Rust `validate_route` must agree.
- **D-07: Color + shape distinguishes port types.** Audio ports render as filled circles in one color (e.g. cyan/blue); Control/CV ports render as smaller diamonds or hollow circles in another (e.g. amber/violet). Shape differs in addition to color so the canvas is colorblind-considerate. Convention matches Bespoke/VCV/Reaktor.
- **D-08: Out-many, in-one cardinality.** Outputs fan out freely (one LFO out → many filter `cutoff_cv` inputs). Each input accepts exactly ONE incoming edge — dragging a new cable to an occupied input **replaces** the old edge. Classic modular-patching semantics; clear ownership per input; matches the typed-command `AddRoute`/`RemoveRoute` model. (Planner: this implies `AddRoute` to an already-occupied input either errors or auto-removes the prior route — pick one consistently between canvas and Rust.)

### Ownership Canvas Visibility (GRAPH-05)
- **D-09: Inline chrome on node bodies.** Ownership/agent state renders INSIDE each node body: a colored controller badge (`user`/`agent`/`shared`), a lock/snowflake icon when the node is locked or frozen, and approve/reject buttons rendered directly on nodes that have a pending agent action. This is the most actionable treatment (matches GRAPH-05's "visible AND actionable directly on nodes") and accepts that nodes get taller. Replaces v1, where ownership lived only in `NodeInspector.tsx:78-93`.
- **D-10: Edge ownership/pending chrome → agent discretion.** Whether routes (edges) also display pending-approval state (dashed edge + midpoint approve/reject) is the planner's call, decided by how `PendingAction` correlates to specific `AddRoute`/`RemoveRoute` commands vs whole-node edits. GRAPH-05 says "nodes/edges" — at minimum, node-level chrome is locked (D-09); edge-level is flexible.
- **D-11: Drag = override (no modal lockout).** Direct manipulation always works as override. Dragging, deleting, or reconnecting an agent-owned node **implicitly** flips controller back to `user`/`shared` and performs the action (no explicit "reclaim first" gate). A globally-frozen session is unfrozen by a single gesture (e.g. clicking a frozen node, or the existing `reclaimOwnership` button). Matches PROJECT.md's Control Safety constraint ("human override must stay easy and reliable") and v1's reclaim semantics. Tradeoff accepted: accidental edits to agent patches are possible — the action history (undo) is the safety net.
- **D-12: Canvas banner + per-node badges for global freeze.** When `agentFrozen` is true, a thin frozen-indicator bar renders across the top of the graph surface (or a subtle ice-tint border around the whole canvas) IN ADDITION TO per-node snowflake badges. A performer must instantly read "the whole session is frozen" from the canvas alone, not just the sidebar. Pairs with D-11's single-gesture unfreeze.

### Drag & Reroute Feel
- **D-13: Optimistic local drag, commit on release.** During a drag, xyflow moves the node visually using local/ephemeral state; the `MoveNode` command (D-04) fires **once** on `mouseup` with the final position. Zero IPC during the drag, 60fps feel, one history entry per drag. This is how React Flow is designed to be used and is the smoothest option. Canonical `node_layout` is updated once per drag (not per mousemove).
- **D-14: Drag-edge-endpoint reroute.** Reconnecting an edge uses xyflow's `onReconnect` — the performer grabs either endpoint of an existing edge and drops it on a new port. One fluent gesture. Internally fires `RemoveRoute(old)` + `AddRoute(new)` as one typed sequence (so undo restores the original route atomically). Most "fluent reroute"; matches Bespoke/VCV. (Delete-then-redraw also works as a fallback path via multi-select + Delete.)
- **D-15: Full patching keyset.** Multi-select via rubber-band box-select (drag on empty canvas) + Shift-click to add/toggle. Keyboard shortcuts: `Delete`/`Backspace` (remove selection), `Cmd+Z` / `Cmd+Shift+Z` (undo/redo via the existing `ActionHistoryEntry` history — do NOT build a separate canvas undo stack), `Cmd+C` / `Cmd+V` (copy/paste), `Cmd+A` (select all), `Cmd+D` (duplicate). Covers GRAPH-04's "add, delete, undo, copy" explicitly plus useful extras.
- **D-16: Subgraph copy/paste with ownership reset.** Copy captures the selected nodes + their **internal routes** (edges between selected nodes) + current parameter values. Paste creates new node ids at a viewport offset, re-attaches the internal routes to the new ids, and **resets ownership to `user`** (the performer just authored the copy). Enables a "copy a voice/channel" workflow. Pasting into empty space spawns at viewport center with cascade (consistent with D-03).

### the agent's Discretion
- **Port-type validation matrix (D-06):** which exact Audio/Control cross-type pairs are valid. Derive from catalog `signal_type`s + existing `validate_route`; must preserve audio-rate FM.
- **Edge ownership/pending chrome (D-10):** whether/where pending-approval renders on edges vs only on nodes.
- **`AddRoute`-to-occupied-input behavior (from D-08):** error vs auto-replace — pick one and keep canvas + Rust consistent.
- **Exact `MoveNode` command variant naming/shape** (D-04): `MoveNode { moves: Vec<NodeLayoutDelta> }` vs `SetNodeLayout { … }` — planner's call, but it MUST be a new `GraphEditCommand` variant that the audio runtime manager treats as a no-op for reconcile (per STATE.md).
- **Custom node component structure** (D-05/D-07/D-09): how to compose the port-handles + inline ownership chrome into one custom xyflow node type. Vanilla-extract + Radix are in the stack.
- **Property-test framework/scope** (GRAPH-05 success criterion #5): `proptest` is already in the stack — planner defines the invariant (xyflow projection ⇄ canonical `SessionDocument`).
- **Undo wiring (D-15):** whether `Cmd+Z` calls an existing `undo` command or synthesizes inverse commands from `ActionHistoryEntry`. Reuse existing infrastructure if present.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project / milestone decisions (locked architecture)
- `.planning/STATE.md` §Decisions — locks **[v2.0 Layout command]**: a dedicated `SetNodeLayout`/`MoveNode` command variant that bypasses `AudioRuntimeManager.reconcile_graph_edit` (Pitfalls V2-2, V2-14). This is THE foundational constraint for D-01/D-04/D-13.
- `.planning/PROJECT.md` §Constraints (esp. Control Safety, Interaction Model) + §Key Decisions — canonical-session-truth-in-the-app, human-override-must-stay-easy. Drives D-11.
- `.planning/REQUIREMENTS.md` §Graph UX Rebuild — GRAPH-01..05 verbatim (the requirements this phase satisfies; GRAPH-04 explicitly lists "add, delete, undo, copy").
- `.planning/ROADMAP.md` §Phase 13 — goal, dependency on Phase 12, and the 5 success criteria (incl. the property-test invariant in #5).
- `.planning/phases/12-node-catalog-foundation/12-CONTEXT.md` — locks the catalog-driven port model Phase 13 builds on. esp. **D-03** (modulation works end-to-end incl. audio-rate), **D-04** (per-parameter CV ports declared in `NodeCatalogEntry`), **D-05** (CV ports for continuous params only).

### Domain & command layer (Rust — the heart of this phase)
- `src-tauri/src/domain/session.rs` — the types to extend: `Node` (:179), `Port` (:248, has `signal_type: SignalType`), `SignalType` (:260, `Audio` | `Control`), `Route` (has `source_port_id`/`target_port_id`/`signal_type`), `GraphEditCommand` enum (the new `MoveNode` variant joins this), `OwnershipAssignment` (`controller`/`is_locked`), and `write_generated_typescript_contract()` (:806, ts-rs export list the new types must join).
- `src-tauri/src/application/graph_edit.rs` — `apply_graph_edit()` (:50) dispatches commands; `validate_route()` (:273) is the Rust authority for port/signal-type matching (the canvas `isValidConnection` must agree with this); `GraphEditError` (:23) is the typed error enum.
- `src-tauri/src/audio/runtime_manager.rs` — `reconcile_graph_edit()` (:193). The new `MoveNode` variant MUST be a no-op here (or bypass entirely) so dragging never recompiles topology. The existing `SetParameterValue` arm (:200) shows the pattern for a non-topology-changing command.
- `src-tauri/src/audio/compiler.rs` — `validate_routes()` (:230) and `TopologyCompileError::PortSignalTypeMismatch` (:92). The catalog-driven port matching Phase 13 surfaces at the canvas lives here at compile time.
- `src-tauri/src/application/session_store.rs` — `ActionHistoryEntry` + undo infrastructure (D-15 reuse target for `Cmd+Z`). `TypedCommand` history is recorded here.

### Catalog (the typed-port source of truth — from Phase 12)
- `src-tauri/src/catalog/mod.rs` — `NodeCatalogEntry` + `CatalogPortSpec` (:69, `signal_type: SignalType`). Each entry declares its ports; the canvas Handle layout (D-05) and validation matrix (D-06) derive from these.
- `src-tauri/src/catalog/entries.rs` — the populated catalog. Note the oscillator's `frequency_cv` Audio-typed input (:100) — this is the audio-rate FM port the validation matrix MUST keep reachable (Phase 12 D-03).

### Current frontend graph surface (what gets rebuilt)
- `src/components/session/GraphViewport.tsx` — the current xyflow canvas. v1 has `nodesDraggable={false}` (line 59) and node-to-node edges; this is the component to rebuild with custom node types, port Handles, drag, reroute, multi-select, and inline ownership chrome.
- `src/store/session-projections.ts` — `projectGraphNodes()` (:98, auto-grids positions — replaced by canonical `node_layout`), `projectGraphEdges()` (:133, ignores port ids — must emit port-to-port edges), `buildTopologySignature()` (:171). The projection layer between canonical state and xyflow; the property test (GRAPH-05 #5) exercises the round-trip through here.
- `src/components/session/NodeInspector.tsx:78-93` — current ownership badge UI. Inline canvas chrome (D-09) draws from this styling; the inspector continues to exist for deep editing.
- `src/components/workspace/PendingActionCard.tsx` + `ConversationView.tsx` — current pending-action approve/reject UI. The inline approve/reject buttons (D-09) call the same IPC (`approvePendingAction`/`rejectPendingAction`).
- `src/store/sessionStore.ts` — Zustand store. `applyGraphEdit` (:184), `reclaimOwnership` (:604), `approvePendingAction` (:628), `rejectPendingAction` (:640). The canvas wires drag/connect/reclaim/approve through these.
- `src/lib/session-client.ts` — IPC layer. `graphEditCommandSchema` (:226) and `typedCommandSchema` (:252) Zod validators must be extended for the new `MoveNode` variant; `applyGraphEdit` (:466).
- `src/generated/session-types.ts` — ts-rs output. `GraphEditCommand` (:82), `Route`, `Port` (:42), `OwnershipAssignment`. New types appear here after Rust changes.
- `src/lib/browser-preview-session.ts` — the browser-preview (non-Tauri) session shim that mirrors command application (:467 `applyGraphEdit`, :733 `applyTypedCommand`). MUST be updated in parallel with Rust so vitest tests keep passing.

### Tests (patterns to extend)
- `src/__tests__/ConversationView.test.ts`, `src/store/session-projections.test.ts` — existing projection/store tests. The new property test (GRAPH-05 #5) joins this surface.
- `src-tauri/tests/audio_graph_commands.rs`, `src-tauri/tests/agent_commands.rs` — Rust command tests; the new `MoveNode` variant and updated `validate_route` get parallel coverage here.

No external specs/ADRs were referenced by the user during discussion.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **xyflow (`@xyflow/react` 12.10.2)** already in stack — supports custom node types with multiple `Handle`s, `isValidConnection`, `onConnect`/`onConnectStart`/`onConnectEnd`, `onReconnect`/`onReconnectStart` (edge-endpoint reroute, D-14), `onSelectionChange` (multi-select, D-15), `useOnSelectionChange`, and keyboard handling. The rebuild uses first-class library features, not custom mouse event plumbing.
- **`elkjs` 0.11.1** is in the stack (STACK.md) — available if an auto-layout fallback is ever wanted (not required by any decision here, but the dependency is present).
- **Typed-command gate** (`GraphEditCommand`/`TypedCommand`): every canvas mutation already has a canonical path. Drag → `MoveNode`, connect → `AddRoute`, disconnect/reroute → `RemoveRoute` (+`AddRoute`), delete → `RemoveNode`/`RemoveRoute`. The rebuild adds `MoveNode`; the rest reuse existing commands.
- **`validate_route` / `validate_routes`**: the Rust authority for port matching already exists and already checks signal types. D-06 extends/clarifies the matrix; it does not start from scratch.
- **Ownership/approval IPC**: `reclaimOwnership`, `approvePendingAction`, `rejectPendingAction`, `toggleAgentFreeze` are all already wired through `sessionStore.ts` → `session-client.ts` → Tauri commands. Inline canvas chrome (D-09) calls these directly.
- **Vanilla-extract + Radix** in the stack for the custom node component styling and any tooltips/popovers on port handles.

### Established Patterns
- **Canonical truth in Rust, engines are consumers**: the app owns the graph; xyflow is a projection. `projectSessionState` (session-projections.ts:71) already projects `SessionDocument` → xyflow nodes/edges. The rebuild keeps this direction; positions become canonical (D-01) and the projection reads from `node_layout`.
- **Typed-command gate for ALL mutations**: nothing mutates the session except through `applyGraphEdit`/typed commands. Drag, connect, reconnect, paste — all flow through commands. This is why `MoveNode` must exist (you can't side-step the gate for drag).
- **Optimistic UI + commit-on-release** is already how parameter edits work (slider → `SetParameterValue` on release/commit). D-13 extends the same idea to positions.
- **`buildTopologySignature`** (session-projections.ts:171) is the existing memoization key for when graph nodes/edges can be reused vs re-projected. The rebuild extends this to include `node_layout` in the signature so position changes don't force full re-projection.

### Integration Points
- **New `MoveNode` command variant** joins `GraphEditCommand` (session.rs) → ts-rs export (session-types.ts) → Zod schema (session-client.ts) → browser-preview shim (browser-preview-session.ts). Five files, one plumbing change, repeated from Phase 12's `SetStepValue` addition.
- **Custom xyflow node type**: `GraphViewport` switches from default nodes to `nodeTypes={{ catalogNode: CatalogNodeComponent }}`. The component renders port Handles from the catalog, inline ownership chrome, and pending-action buttons.
- **Projection rewrite**: `projectGraphNodes` reads positions from `session.node_layout` (falling back to viewport-center for orphans); `projectGraphEdges` emits port-to-port edges (`sourceHandle`/`targetHandle` = port ids).
- **Audio runtime no-op**: `reconcile_graph_edit` (runtime_manager.rs:193) gains a `MoveNode { .. } => Ok(store.current().clone())` arm (mirroring `SetParameterValue`'s non-topology treatment). Same for `visual/runtime_manager.rs:251`.
- **Property test (GRAPH-05 #5)**: a `proptest` (Rust) or vitest (TS) invariant asserting `projectSessionState(session).graphNodes/graphEdges` round-trips with `session.nodes/routes/node_layout` after every command application.

</code_context>

<specifics>
## Specific Ideas

- The performer's mental model is consistently **modular and legible** — visible typed ports (D-05), explicit per-param CV handles (inherited from Phase 12 D-04), classic out-many/in-one cardinality (D-08), fluent drag-endpoint reroute (D-14). Favor Bespoke/VCV/Reaktor conventions whenever there's a choice.
- **Override must be frictionless (D-11)** — the user was emphatic that "drag = override" with no modal/mode lockout is the only acceptable default. This is a core differentiator vs DAWs and agent-only tools. Do not add confirm dialogs or reclaim-first gating on the canvas.
- **Positions are performer property, not musical state (D-01/D-02)** — the user drew a clean line between "audio identity" (Node struct, compiler-consumed) and "spatial arrangement" (node_layout map, performer-owned). Don't blur it.
- Node height grows to fit inline ownership chrome (D-09) — acceptable trade; the user preferred actionability over minimal pixel density.
- Copy/paste = "copy a voice/channel" (D-16) — the user thinks in subgraphs (nodes + internal routes + params), not single nodes.

</specifics>

<deferred>
## Deferred Ideas

None raised — discussion stayed within phase scope. The following were noted as related but correctly belong to other phases:

- **Live streaming agent proposal previews on canvas** (dashed/ghost nodes/edges for in-flight LLM proposals) — Phase 16 (AGENT-04). Phase 13 only renders already-created `PendingAction`s.
- **Auto-layout / auto-arrange button** (elkjs-driven tidy of messy graphs) — not required by any GRAPH-01..05 success criterion; `elkjs` is in the stack if a later phase wants it.
- **Edge animation showing active signal flow** (moving dashes when audio is running) — visual polish, not a Phase 13 requirement.

### Reviewed Todos (not folded)
None — `todo.match-phase` returned 0 matches for Phase 13.

</deferred>

---

*Phase: 13-Graph UX Rebuild*
*Context gathered: 2026-06-27*
