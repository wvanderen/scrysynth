# Architecture Research — v2.0 "Studio-Grade Instrument"

**Domain:** Graph-native desktop audiovisual instrument — NEW feature integration onto a shipping v1 app
**Project:** Scrysynth
**Researched:** 2026-06-26
**Confidence:** HIGH (integration points grounded in the actual shipped v1 codebase; compositing recommendation MEDIUM-HIGH pending a vertical spike)

> **Scope:** This document covers ONLY how the seven v2.0 features integrate with the *existing* v1 architecture. It does not re-document the shipped v1 design — see the prior `ARCHITECTURE.md` baseline and the integration map below for that.

## Existing v1 Baseline (Integration Reference)

The v1 architecture is an event-sourced Rust core that owns a canonical `SessionDocument` and compiles it outward to two execution planes. All seven v2 features plug into **existing seams**; none requires rewriting the core mutation model.

| v1 Seam (file) | Role v2 builds on |
|---|---|
| `domain/session.rs` — `SessionDocument`, `Node`, `Route`, `GraphEditCommand`, `AudioPrimitive` | The canonical model. **Modified** (not replaced) for the node catalog + graph UX. |
| `application/graph_edit.rs` — `apply_graph_edit` + `validate_route` | Command validation. **Extended** with new commands (move, reroute) and catalog-driven validation. |
| `application/session_store.rs` — `reconcile_audio_graph_edit` / `reconcile_visual_graph_edit` | Adapter reconciliation. **Extended**, not replaced. |
| `audio/compiler.rs` — `compile_session_to_topology` → `CompiledTopology` | Audio projection. **Modified** to consume the catalog instead of hardcoded enum arms. |
| `audio/synthdefs.rs` — `plan_sc_resources` + `normalize_parameter_name` | Synthdef selection. **Modified** to be catalog-driven (replaces the giant `match` + alias table). |
| `audio/runtime_manager.rs` — `reconcile_graph_edit` (live param vs full topology reapply) | Reconciliation strategy. **Unchanged contract**; benefits from catalog metadata (knows which params are live-routable). |
| `visual/compiler.rs` + `visual/bevy_sidecar.rs` + `bin/scrysynth-visual.rs` + `visual/bevy_runtime.rs` | Visual adapter + sidecar. **Modified** for behind-grid compositing + richer Bevy render. |
| `application/agent_planner.rs` — `PlannerProvider` trait + `ParserPlannerProvider` | Provider-agnostic boundary. **Extended** with one new `PlannerProvider` impl for the live LLM. |
| `src/components/session/GraphViewport.tsx` — `@xyflow/react` with `nodesDraggable={false}` | Graph surface. **Rebuilt** (the whole point of feature 1). |
| `src/App.tsx` — view-switcher shell | Shell layout. **Rebuilt** for focused-shell layout. |
| `tauri.conf.json` — `externalBin`, `resources/synthdefs`, ad-hoc signing | Bundling. **Extended** per-OS targets, updater, notarization. |
| `lib.rs` — `invoke_handler!` command surface + `.setup` app-handle wiring | IPC + plugin registration. **Extended** (updater, window mgmt, streaming channel). |

**The single architectural rule preserved unchanged:** `intent → validated command → domain event → session state → adapter projection → runtime side effects → runtime feedback`. Every v2 feature either (a) produces typed commands at the front of this pipeline, or (b) is a projection/execution-plane change at the back. Nothing is allowed to shortcut the pipeline.

## Recommended v2 Architecture

```text
                        ┌──────────────────────────────────────────────────────────┐
                        │  Focused Pro Shell (React 19) — App.tsx rebuilt          │
                        │   graph hero  │  chat sidebar  │  progressive panels      │
                        │   ┌───────────────────────────────────────────────────┐  │
                        │   │  Graph Surface (xyflow v12) — DRAGGABLE + reconnect│  │
                        │   │   custom catalog nodes · typed handles · validate  │  │
                        │   │   transparent background ⇒ visuals show THROUGH    │  │
                        │   └───────────────────────────────────────────────────┘  │
                        └───────────────┬──────────────────────────┬───────────────┘
                  typed commands        │ Tauri IPC + channels     │ frame stream / window-sync
                                        ▼                          ▼
        ┌───────────────────────────────────────────┐   ┌───────────────────────────────┐
        │  Rust App Core (SessionStore + mutex)     │   │  scrysynth-visual sidecar      │
        │   ┌─────────────────────────────────────┐ │   │   (separate PROCESS — Bevy)    │
        │   │ Node Catalog (NEW, Rust-owned)      │ │   │   headless/behind-window render │
        │   │  ↳ drives: validation, compiler,    │ │   │   frame → webview OR own window │
        │   │    synthdef plan, palette, context  │ │   └───────────────┬───────────────┘
        │   └─────────────────────────────────────┘ │                   │
        │   GraphEdit command layer (EXTENDED)      │                   │
        │   audio compiler ← catalog (MODIFIED)     │                   │
        │   visual compiler (MODIFIED)              │◀──────────────────┘
        │   AgentPlanner + LiveProvider (NEW impl)  │
        └───────────┬───────────────────┬───────────┘
                    ▼                   ▼
        ┌────────────────────┐  ┌────────────────────┐
        │ SuperCollider via  │  │ SQLite persistence │
        │ OSC (rosc)         │  │ + JSON export      │
        │ per-OS scsynth     │  │                    │
        └────────────────────┘  └────────────────────┘
```

### Component Responsibilities (v2 deltas)

| Component | v2 Responsibility Delta | Status |
|---|---|---|
| **Node Catalog** (`domain/node_catalog.rs`, NEW) | Single source of truth for node-type definitions: ports, parameters, synthdef mapping, validation rules, UI metadata. Feeds compiler, palette, inspector, agent context. | NEW |
| Graph command layer | Add `MoveNode` + `RerouteEdge`; route validation reads port types from the catalog instead of the loose `Port` list. | MODIFIED |
| Audio topology compiler | Replace hardcoded `CompiledNodeKind` enum arms with catalog-driven dispatch. Catalog entry declares its `synthdef` + arg mapping. | MODIFIED |
| Synthdef planner | Replace `normalize_parameter_name` alias table + `validate_runtime_target` match with catalog metadata. | MODIFIED |
| Visual compiler + sidecar | Richer scene compilation; behind-grid compositing (window-sync OR frame-stream mode). | MODIFIED |
| Visual Bevy runtime | Headless/render-to-texture option + borderless-behind-window option; richer GPU rendering. | MODIFIED |
| Live LLM provider (`application/providers/live_llm.rs`, NEW) | One new `PlannerProvider` impl: streaming chat + tool-calling that emits typed commands. | NEW |
| Graph surface (React) | Draggable nodes, typed multi-handle nodes, edge reconnect, `isValidConnection`. | REBUILT |
| Focused shell (React) | Graph-hero layout, chat sidebar, progressive-disclosure panels. | REBUILT |
| Bundling (`tauri.conf.json`) | Per-OS targets, updater config, signing/notarization, per-triple sidecars + scsynth. | MODIFIED |

## Feature 2 — Curated Modular Node Library (catalog ↔ compiler mapping)

> Addressed first because **it is the foundation features 1, 3, and 5 build on.** The catalog must exist before the graph rebuild, the visual scene, and agent context can reference real node types.

### Where the catalog lives: **Rust-owned, single source of truth**

The catalog is a Rust module (`domain/node_catalog.rs`) serialized through `ts-rs` so the frontend consumes *generated* catalog types — the same contract-export pattern already used for `SessionDocument`. This is mandatory, not optional, for three reasons:

1. **The audio compiler must read it.** `audio/compiler.rs` and `audio/synthdefs.rs` are Rust. If the catalog lived in the frontend, the compiler would need a round-trip or a duplicated source — both already rejected by the v1 "canonical Rust core" decision.
2. **Port-type constraints must be enforced at command validation time**, in Rust (`graph_edit.rs`), before any adapter sees the edit.
3. **The agent context packet** (`agent_planner.rs::SessionContextPacket`) is built in Rust and must carry accurate node-type/parameter schemas so the live LLM can emit valid typed commands.

A frontend-only catalog would force three copies (TS palette, TS validation, Rust compiler) that drift — exactly the anti-pattern v1 avoided with `ts-rs`.

### Catalog entry shape (integration with existing `Node`)

The existing `Node` carries `audio_primitive: Option<AudioPrimitive>` — a closed enum of 4 primitives (`Source/Effect/Mixer/Output`) with sub-enums for `source_type`/`effect_type`. This is the v1 bottleneck: adding a new oscillator variant today means editing the enum, the compiler `match`, `normalize_parameter_name`, `validate_runtime_target`, the palette builder, and the zod schema.

**Recommended evolution (non-breaking, schema-versioned):** add a catalog key to `Node` and make the existing `audio_primitive` *derivable* from it. Keep `audio_primitive` for backwards compatibility with v1 session files.

```rust
// NEW: domain/node_catalog.rs (sketch)
#[derive(Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct NodeCatalogEntry {
    pub catalog_id: String,           // e.g. "osc.saw stereo", stable, not localized
    pub family: NodeFamily,           // Source | Effect | Modulator | Sequencer | Output | Utility
    pub display_name_key: String,     // i18n key, not raw text
    pub ports: PortTemplate,          // declarative input/output port specs with signal_type
    pub parameters: Vec<ParameterSpec>, // id, name_key, default, min, max, unit, warp, live_routable
    pub audio: Option<AudioMapping>,  // synthdef name + arg mapping + runtime_target pattern
    pub visual: Option<VisualMapping>,// element_type + param→visual param binding
}

#[derive(Clone, Serialize, Deserialize, TS)]
pub struct AudioMapping {
    pub synthdef_name: &'static str,   // replaces synthdefs.rs constants
    pub runtime_target: String,         // replaces validate_runtime_target()
    pub arg_map: Vec<(String, String)>, // (parameter_id, synthdef arg name) — replaces normalize_parameter_name
    pub bus_kind: BusRequirement,       // None | NeedsOutputBus | NeedsInputBus | Mixer{n}
}
```

`Node` gains `pub catalog_id: Option<String>` (and the existing `audio_primitive` stays, populated for legacy sessions). The catalog lookup (`catalog.get(&node.catalog_id)`) becomes the single dispatch point.

### How port-type constraints flow (the key question)

Today `validate_route` (graph_edit.rs) checks `source_port.direction == Output`, `target_port.direction == Input`, and `signal_type` equality — but ports are an ad-hoc `Vec<Port>` per node, populated by whatever the palette builder happened to set. There is no guarantee a node's declared ports match its primitive.

With the catalog:

- A node's **canonical ports come from its catalog entry**, materialized at node-creation time. The `AddNode` command (or a new catalog instantiation helper) stamps `node.ports` from `catalog_entry.ports`.
- `validate_route` is **unchanged in mechanics** but now operates over ports that are guaranteed-consistent with the node's audio mapping, because both derive from the same catalog entry.
- A new `GraphEditCommand::InstantiateCatalogNode { catalog_id, position }` (or extending `AddNode` to accept a `catalog_id`) becomes the preferred creation path, replacing `PrimitivePalette`'s hand-built node literals.

### How parameters map to synthdef controls

This replaces `synthdefs.rs::normalize_parameter_name` (the 10-entry alias match) and `validate_runtime_target` (the 14-arm match). Each catalog entry's `AudioMapping.arg_map` is the explicit `(parameter_id → synthdef_control_name)` table. The compiler/synthdef planner reads it directly:

```text
Node.parameters[i]  →  catalog.audio.arg_map[i]  →  ScSynthArg{name: arg_map.1, value: param.value}
```

`live_routable: bool` per parameter tells `runtime_manager::reconcile_graph_edit` whether a `SetParameterValue` can use the cheap live-OSC path (`/n_set`) or whether it must reapply the full topology (because the parameter changes the synthdef graph, e.g. wave-shape). v1 currently re-derives this implicitly; the catalog makes it explicit metadata.

### Data flow change

```text
v1:  palette(TS) builds Node{audio_primitive} → compiler match(audio_primitive) → synthdefs match(...)
v2:  catalog(Rust) → InstantiateCatalogNode stamps Node{catalog_id, ports, params}
        → compiler resolves catalog.audio → synthdef plan (no enum match)
        → validate_route reads catalog ports (consistent by construction)
        → agent context packet includes catalog schemas
```

### New vs Modified

| NEW | MODIFIED |
|---|---|
| `domain/node_catalog.rs` + ~12-16 entries | `domain/session.rs` — add `catalog_id` to `Node`, bump `CURRENT_SCHEMA_VERSION` |
| Catalog → TS export in `write_generated_typescript_contract` | `audio/compiler.rs` — catalog dispatch |
| `InstantiateCatalogNode` command (or extend `AddNode`) | `audio/synthdefs.rs` — read `arg_map`, drop alias table |
| New `.scsyndef` files per catalog entry under `resources/synthdefs/v2/` | `graph_edit.rs` — port validation via catalog |
| Catalog fixtures + property tests (proptest) | `agent_planner.rs` — include catalog in context packet |
| | `PrimitivePalette` → reads generated catalog (no hand-built nodes) |

**Confidence: HIGH** — the catalog is a refactor of existing hardcoded knowledge into data, all within the existing command/event/reconcile pipeline.

## Feature 1 — Graph UX Rebuild (draggable nodes + flexible edge connect/reconnect)

### Integration with existing `@xyflow/react` workspace

The shipped `GraphViewport.tsx` sets `nodesDraggable={false}` and only supports `onConnect` (create). The rebuild flips this: draggable nodes, multi-handle typed nodes, edge reconnect, and connection validation. This is a **surface rebuild over an unchanged command model** — the win is that the new UX maps onto new/extended commands, so the Rust core stays in control.

### Command model changes

The v1 `GraphEditCommand` enum has `AddNode`, `RemoveNode`, `SetNodeEnabled`, `SetParameterValue`, `AddRoute`, `RemoveRoute`, `AssignNodeToBus`, `ClearNodeBusAssignment`. The rebuild needs two additions:

| New command | What it does | Why it can't reuse an existing one |
|---|---|---|
| `MoveNode { node_id, position }` | Persist node viewport position in the session | Position is currently ephemeral/derived. To survive reload + be agent-visible, it must be canonical state. |
| `RerouteEdge { route_id, new_source_port_id, new_target_port_id }` | Atomically change a route's endpoints | `RemoveRoute`+`AddRoute` is two commands = two reconciliations = an audible gap mid-patch. One atomic command keeps the audio adapter's diff single-step. |

`MoveNode` requires adding a `position: Option<(f64,f64)>` to `Node` (canonical, so layout survives reload and the agent can reason about spatial structure). This is a minor, additive schema change.

### How port-type constraints flow from the node-type model (xyflow side)

`@xyflow/react` v12 connection validation is driven by an `isValidConnection(connection, edges)` prop returning boolean. The catalog already declares each node type's ports (`direction`, `signal_type`). The mapping:

```text
catalog.ports (Rust)  →  ts-rs generated types  →  frontend builds xyflow Handle components per port
                                              →  isValidConnection(connection) mirrors Rust validate_route:
                                                    source handle is Output, target is Input,
                                                    signalType matches, no cycle (best-effort client-side)
```

Validation is **duplicated by design but not in conflict**: the client `isValidConnection` is a UX hint (grays invalid handles, blocks the drag); the Rust `validate_route` is the authority (rejects the command if it arrives invalid). This mirrors how zod (frontend) and serde (Rust) both validate today. Custom nodes register via `nodeTypes` and render one `<Handle>` per catalog port, each typed by `signalType` for color/style.

### Edge reconnect mechanics

xyflow v12 exposes `onReconnect(connection, newConnection)` + `reconnectEdge` + per-edge `reconnectEdgeId`. The frontend translates `onReconnect` into the new `RerouteEdge` command. On success, the returned `SessionDocument` re-derives the xyflow edges (the existing `session-projections.ts` selector already maps `Route` → `Edge`). If Rust rejects (cycle/port mismatch), the edge springs back — xyflow's optimistic update + revert pattern.

### Data flow change

```text
drag node handle → xyflow onConnect/onReconnect → build GraphEditCommand{AddRoute|RerouteEdge}
   → invoke("apply_graph_edit") → validate_route(catalog) → reducer → reconcile_audio/visual
   → SessionDocument returned → Zustand mirror updates → xyflow edges re-derived
```

### New vs Modified

| NEW | MODIFIED |
|---|---|
| Custom node/edge React components (`CatalogNode`, `TypedEdge`) | `GraphViewport.tsx` — draggable, typed handles, `isValidConnection`, `onReconnect` |
| `MoveNode` + `RerouteEdge` commands | `domain/session.rs` — add `position` to `Node` |
| `session-projections.ts` — port→Handle, signalType→style | `graph_edit.rs` — handle `MoveNode`/`RerouteEdge`, cycle check on reroute |
| Reconnect-revert test harness | `runtime_manager.rs` — `RerouteEdge` reconciles as one topology diff |

**Confidence: HIGH** — xyflow v12 natively supports all of this; the work is mapping it to the typed command layer, which is already there.

## Feature 3 — Visuals Behind the Grid (ARCHITECTURE-CRITICAL)

This is the hardest v2 decision. The constraint is explicit: **the visual runtime must stay a separate adapter process** (per PROJECT.md), and the result must *appear* behind the graph surface like Bespoke Synth. The shipped v1 sidecar already opens its own 960×540 Bevy window (`bevy_runtime.rs::run_visible_runtime`); v2 must make that render appear *behind/within* the Tauri webview graph area.

### Option comparison

| Option | Mechanism | "Separate process?" | GPU-composited? | Cross-platform robustness | Per-frame cost |
|---|---|---|---|---|---|
| **A. Borderless Bevy window behind a transparent webview** | Bevy owns a borderless window; Tauri main window is `transparent:true`+`macOSPrivateApi:true`; webview CSS background is transparent; OS window manager composites Bevy *behind* the (transparent) webview. App keeps the two windows geometry-synced. | ✅ yes | ✅ yes (OS WM does it) | MEDIUM — macOS/Windows solid; Linux (Wayland) weakest | **Zero** — no copy |
| **B. Headless Bevy → render-to-texture → frame stream → WebGL canvas behind graph** | Bevy runs with no window, renders to a GPU `Image`, extracts frames, ships RGBA bytes over a Tauri `Channel`, a `<canvas>` layered behind the graph paints them. | ✅ yes | ❌ round-trip GPU→CPU→IPC→GPU | HIGH — one window, pure DOM layering | **High** — full-res RGBA × fps over IPC |
| **C. Shared raw-window-handle (wgpu surface from Tauri's window)** | Bevy renders into the *same* Tauri window via its raw handle. | ❌ **NO — violates constraint** | ✅ | HIGH | Zero |
| **D. Separate always-on-top transparent Bevy window *above* graph** | Floats over the graph. | ✅ yes | ✅ | MEDIUM | Zero |

**Option C is rejected outright** — it merges the visual runtime into the app process and breaks the "separate adapter" constraint that is a documented product decision.

**Option D is rejected** — the requirement is *behind* the grid, not a floating overlay.

### Recommendation: **Option A as primary, Option B as documented fallback**

For a **live audiovisual instrument**, per-frame visual fidelity and latency matter more than cross-platform uniformity of the *compositing trick*. Option A gives true OS-level GPU compositing with **zero per-frame copy** — the Bevy process renders to its own surface and the OS stacks it behind the transparent webview. This is the closest to "Bespoke Synth's visuals behind the patch" and keeps the Bevy process fully independent (it already owns a window in v1; we just borderless-ify it, hide it from the taskbar/dock, and geometry-sync it).

Option B is kept as the **per-platform fallback** for configurations where behind-window compositing is unreliable (Linux Wayland is the known weak spot; some Windows GPU drivers mishandle transparent webviews). The fallback trades performance for robustness: render at the graph-surface resolution (one panel, not fullscreen), throttle to ~30fps, ship bytes over `tauri::ipc::Channel`, paint on a canvas. The frontend decides which mode the sidecar runs via the existing `--minimal`/mode flag already in `scrysynth-visual.rs` (add `--behind` / `--headless-stream`).

> **Required vertical spike before committing:** a one-day proof of concept of Option A on macOS (the v1 target) confirming (1) `macOSPrivateApi:true` + transparent webview CSS lets the Bevy window show through, (2) the two windows stay geometry-synced on move/resize/focus, (3) input passes through to the graph. If the spike fails on macOS, default to Option B everywhere. **Confidence in the recommendation: MEDIUM-HIGH, gated on this spike.**

### How Option A integrates with the existing sidecar

The v1 sidecar is launched via `app.shell().sidecar("scrysynth-visual")` (`bevy_sidecar.rs`) with a JSON-lines handshake. The changes are additive:

- **Sidecar side (`bevy_runtime.rs`):** add a `--behind` mode that creates a borderless, `decorations:false`, `skipTaskbar:true`, `resizable:false` window, sets it to follow the parent window's frame (frame messages received over the existing protocol), and disables its own activation so it never steals focus. The richer Bevy render (shaders, GPU sprites, post-FX) is built on top of the current `Camera2d` + `Sprite` skeleton.
- **Protocol side (`visual/protocol.rs`):** add a `SetWindowFrame { x, y, w, h, visible }` message so the app can drive the sidecar window geometry. The app already owns the `AppHandle` (wired in `.setup`); it uses `get_webview_window("main")` to read its frame and push it to the sidecar on move/resize via the existing channel.
- **App window side (`tauri.conf.json` + `App.tsx`):** `macOSPrivateApi:true`, main window `transparent:true`; the graph panel's CSS background becomes `transparent` so the Bevy window behind shows through the webview. The graph nodes/edges render opaquely on top.

### How Option B integrates (fallback)

- Sidecar runs headless: `WindowPlugin` with `primary_window: None`, a `Camera` targeting a `Handle<Image>` render target, an `ImageCopyPlugin`-style system extracting frames to a `Vec<u8>`.
- Frames stream over a **new** Tauri `ipc::Channel<FrameChunk>` (not the JSON-lines control channel — that stays for control; frames are hot data and must use a separate channel per the v1 "don't pump hot data through the semantic bus" anti-pattern).
- Frontend `<canvas>` (WebGL or 2D) sits as the first child of the graph panel, behind the xyflow surface.

### Visual compiler change (both modes)

`visual/compiler.rs::compile_session_to_visual_scene` currently derives element type from `NodeType` (`Source→sphere`, etc.). With the catalog (Feature 2), element type + visual param bindings come from `catalog_entry.visual`. This couples Feature 3 to Feature 2 — the catalog must land first.

### New vs Modified

| NEW | MODIFIED |
|---|---|
| `SetWindowFrame` protocol message | `visual/bevy_runtime.rs` — borderless/behind mode + richer render |
| `--behind` / `--headless-stream` sidecar modes | `visual/bevy_sidecar.rs` — push window frame; dual spawn mode |
| Window-frame sync service in `.setup` (Option A) | `tauri.conf.json` — `macOSPrivateApi:true`, transparent main window |
| Frame `ipc::Channel` + canvas painter (Option B fallback) | `visual/protocol.rs` — frame message |
| Option A vertical spike (decision gate) | `visual/compiler.rs` — catalog-driven element/param mapping |
| | `App.tsx` / graph panel CSS — transparent background |

**Confidence: MEDIUM-HIGH** (gated on the Option A spike). The integration is additive to a sidecar that already owns a window and already has a typed protocol — no core-model change.

## Feature 4 — Pro-Grade Focused Shell

### Integration with existing view-switching + Zustand stores

The v1 `App.tsx` is a top-chrome + view-switcher (`graph|conversation|performance|runtime|hardware`) + main column + context inspector + runtime strip. The v2 shell collapses this into a **graph-hero with a persistent chat sidebar and progressive-disclosure panels** — but this is a *layout* change, not a state-model change. The Zustand `sessionStore` (the canonical-state mirror) and the command dispatch surface (`session-client.ts`) are reused as-is.

### Component architecture for progressive disclosure

Three layers of disclosure, each a presentational component reading from the same store:

1. **Always-visible:** graph hero (transparent, visuals behind it), slim transport strip, agent-safety chrome (freeze/reclaim), chat sidebar collapsed/expanded toggle.
2. **On-demand panels:** node inspector (slides over the sidebar when a node is selected — reuses `NodeInspector.tsx`), runtime health, hardware bindings, macro editor. These are the existing workspace panels, re-housed as overlays/drawers rather than full view-swaps.
3. **Deep panels:** scene/variation manager, full agent history, session I/O. Reached via menu, not cluttering the default screen.

The view-switcher (`WorkspaceViewSwitcher`) is retired in favor of a **single primary surface** (graph) with overlays. This is the "retire the card-webui feel" directive from PROJECT.md.

### Data flow change

Minimal. The store already exposes everything (`session`, `graphNodes`, `graphEdges`, `pendingActions`, `hardwareBindings`, etc.). The shell reorganizes *which* components mount and *where*, not *what data they read*. The one real change: the chat sidebar becomes a first-class always-present surface (today `ConversationView` is a mutually-exclusive view), so the agent conversation is co-visible with the graph — a core product-identity requirement.

### New vs Modified

| NEW | MODIFIED |
|---|---|
| `FocusedShell` layout component (replaces `App.tsx` cockpit) | `App.tsx` — new layout |
| `ChatSidebar` (always-present `ConversationView`) | `WorkspaceViewSwitcher` — retired or reduced to panel toggles |
| Iconography set (lucide-react or similar; React 19-compatible) | `App.css` (40k chars today) — token-driven refactor toward the design system |
| Drawer/overlay primitives | Existing workspace panels — re-housed as overlays |

**Confidence: HIGH** — layout-only; no core/data change. Should be sequenced *after* the graph rebuild (Feature 1) so the new shell wraps the new graph surface, not the old.

## Feature 5 — Live LLM Provider Agent

### Integration with the existing provider-agnostic boundary

The v1 `PlannerProvider` trait (`agent_planner.rs`) is exactly the seam designed for this:

```rust
pub trait PlannerProvider {
    fn provider_id(&self) -> &str;
    fn is_available(&self) -> bool;
    fn plan(&self, request: &PlannerRequest) -> Result<PlannerProviderOutput, PlannerProviderError>;
}
```

`ParserPlannerProvider` is the deterministic/mock impl (verified in v1). The live LLM is **one new struct implementing this trait** — nothing in the orchestration, ownership, approval, or risk-tiering path changes. This is the lowest-risk way to land AGNT-01R.

### Recommended crate: `async-openai`

`async-openai` is the most mature Rust crate for OpenAI-compatible chat APIs with both **streaming** (`Chat::stream` → `ChatCompletionResponseStream`) and **tool/function calling** (`FunctionDefinition` / `tool_calls` in responses). It works against any OpenAI-compatible endpoint (OpenAI, local Ollama, LM Studio, Groq, OpenRouter) by overriding the base URL — so "provider-agnostic" at the *product* level is preserved even with a single concrete crate, because the trait is the product boundary and the crate is one impl.

Alternatives considered: `rig` (higher-level, multi-provider, good RAG/tool abstractions, newer) and `genai` (multi-provider, simpler). Use `rig`/`genai` only if you need *multiple non-OpenAI-compatible* providers behind one trait inside Rust — but the `PlannerProvider` trait already provides that abstraction, so one concrete OpenAI-compatible impl is sufficient for v2.

### Tool-calling for typed commands

The LLM is given **tools** whose schema is the `GraphEditCommand` / `PerformanceCommand` / `MacroCommand` unions (already exported to TS via `ts-rs`; the JSON schema for tools is derivable from the same serde types). The provider's `plan()`:

1. Builds a system prompt + the existing `SessionContextPacket` (bounded context, already implemented) as the user/context message.
2. Declares tools = the typed command constructors (`add_node`, `set_parameter_value`, `recall_scene`, etc.), each with a JSON schema matching the Rust command payload.
3. Streams the model's response; tool-call deltas accumulate into `Vec<TypedCommand>`.
4. Returns `PlannerProviderOutput::Typed(PlannerProposal { commands, confidence, .. })`.

The commands then flow through the **exact same** `agent_command::handle_agent_message` → ownership gate → risk tiering → pending-action approval path that the mock provider uses. The live LLM cannot bypass safety; it only *proposes* faster/richer.

### Streaming, fallback, telemetry

- **Streaming:** the current `send_agent_message` Tauri command is request/response. To show live token/tool-call progress, add a streaming variant that takes a `tauri::ipc::Channel<AgentStreamEvent>` and forwards `async-openai` stream chunks (reusing the pattern the updater plugin docs show for download progress). The final `AgentIntent` is sent when the stream completes.
- **Fallback:** wrap the live provider in a small chain — `LiveLlmProvider` → on `Unavailable`/`ProviderFailed`, fall back to `ParserPlannerProvider` (the deterministic local parser). This keeps the agent usable offline / when the API key is missing, exactly matching the v1 "deterministic/mock backend verified" carry-over.
- **Telemetry:** emit `tracing` spans for provider latency, tool-call counts, token usage, and fallback events. The v1 `tracing` layer already exists.
- **Secrets:** API key stored in OS keychain (or a settings file the Rust core reads), never in the frontend or `tauri.conf.json`. The provider reads it at construction.

### Data flow change

```text
v1:  user msg → send_agent_message → ParserPlannerProvider.plan → AgentIntent → ownership/approval → commands
v2:  user msg → send_agent_message_stream(channel)
         → LiveLlmProvider.plan (async-openai stream + tool-calling)
              ↳ on failure → ParserPlannerProvider.plan (fallback)
         → AgentStreamEvent chunks over channel → final AgentIntent
         → ownership/approval (UNCHANGED) → commands
```

### New vs Modified

| NEW | MODIFIED |
|---|---|
| `application/providers/live_llm.rs` — `LiveLlmProvider: PlannerProvider` | `application/agent_command.rs` — streaming variant w/ channel |
| `application/providers/mod.rs` + fallback chain | `lib.rs` — register `send_agent_message_stream` command |
| Tool-schema generation from serde command types | `application/agent_planner.rs` — provider chain wiring |
| API-key settings (keychain/file) + provider config in session/settings | `ConversationView.tsx` — render streaming deltas |

**Confidence: HIGH** — the boundary was explicitly designed for this in v1 (Phase 10). The work is one trait impl + streaming plumbing, with safety fully inherited.

## Feature 6 — Cross-Platform Builds (Windows / Linux / Intel / universal macOS)

### Integration with existing bundling

v1 ships `aarch64-apple-darwin` only, ad-hoc-signed. The bundling seam is `tauri.conf.json::bundle.externalBin` + `resources`. Confirmed from official docs: Tauri looks for sidecar binaries as `binary-name-{target-triple}{ext}`:

| Target | Sidecar filename required |
|---|---|
| Windows x86_64 | `scrysynth-visual-x86_64-pc-windows-msvc.exe` |
| macOS Apple Silicon | `scrysynth-visual-aarch64-apple-darwin` |
| macOS Intel | `scrysynth-visual-x86_64-apple-darwin` |
| Linux x86_64 | `scrysynth-visual-x86_64-unknown-linux-gnu` |

For a **universal macOS** build, either provide both arch suffixed binaries (Tauri's `tauri build --target universal-apple-darwin` resolves at runtime) or build a single `lipo`'d universal binary. The existing `scripts/prepare-sidecar.sh` (referenced in `beforeBuildCommand`) is extended to produce all needed triples.

### Per-OS SuperCollider bundling

`scsynth` must be bundled per OS. Two viable patterns:

- **`externalBin`** (recommended for `scsynth`): treat `scsynth` as a sidecar-like binary with target-triple suffixes. The audio adapter already supervises the SC process; launching it via `app.shell().sidecar("scsynth")` (or a raw `Command` resolved from the bundle resource dir) keeps the existing OSC-based supervision unchanged.
- **`resources`**: ship `scsynth` (+ SC plugins/help files) as bundled resources and resolve the path via `app.path().resource_dir()`. Simpler if SC needs many support files (synthdefs, plugins).

The current `resources/synthdefs` entry stays; add the per-OS `scsynth` binary. On Linux, SC has shared-library dependencies — either bundle them or document a `supercollider` package dependency in the `.deb`/`.rpm` `depends` list (Tauri's `DebConfig.depends`).

### Platform-specific configs

Use Tauri's platform-merge feature: `tauri.windows.conf.json` / `tauri.linux.conf.json` / `tauri.macos.conf.json` merge into the base. Put OS-specific bundle keys (Windows `webviewInstallMode`, Linux `appimage.bundleMediaFramework`, macOS `minimumSystemVersion`) here rather than `#[cfg]`-ladder Rust.

### CI matrix

A GitHub Actions matrix (`macos-latest` for both arches via `--target`, `windows-latest`, `ubuntu-latest`) with per-job sidecar/scsynth preparation steps. `tauri-action` can produce the updater `latest.json` across the matrix in one release.

### Data flow change

None at the runtime level — the audio/visual adapters are already target-agnostic (they talk OSC / JSON-lines). The change is purely build/packaging.

### New vs Modified

| NEW | MODIFIED |
|---|---|
| Per-triple sidecar + scsynth binaries in `binaries/` | `scripts/prepare-sidecar.sh` — multi-triple build |
| Platform-specific `tauri.{os}.conf.json` | `tauri.conf.json::bundle` — `externalBin` scsynth, per-OS resources |
| CI matrix workflow | Audio adapter SC-path resolution — prefer `sidecar()` or `resource_dir()` |

**Confidence: HIGH** — well-trodden Tauri path; the runtime is already platform-neutral.

## Feature 7 — Notarization + Auto-Updater

### Two distinct concerns (do not conflate)

1. **Code signing + notarization** = OS-trust (will macOS Gatekeeper / Windows SmartScreen let it run). Independent of the updater.
2. **Updater signing** = Tauri's own signature scheme verifying the *update artifact* came from you. Independent of OS notarization.

Both must ship for v2.

### Updater (Tauri plugin, confirmed from official docs updated 2025-11)

- Add `tauri-plugin-updater` (Rust) + `@tauri-apps/plugin-updater` (JS) + `tauri-plugin-process` (for relaunch).
- `tauri.conf.json`: `bundle.createUpdaterArtifacts: true` + `plugins.updater.{ pubkey, endpoints }`. The `pubkey` is the **content** of the public key (not a path). Endpoints use `{{target}}`/`{{arch}}`/`{{current_version}}` placeholders.
- Generate keys: `tauri signer generate -w ~/.tauri/scrysynth.key`. Set `TAURI_SIGNING_PRIVATE_KEY` + `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` env at build (env vars, **not** `.env` files).
- Build produces per-platform artifacts + `.sig`: macOS `.app.tar.gz`+`.sig`, Windows `.exe`/`.msi`+`.sig`, Linux `.AppImage`+`.sig`.
- Update flow: `check()` → `downloadAndInstall(onEvent)` → `relaunch()`. Use a `tauri::ipc::Channel<DownloadEvent>` to stream progress to the frontend (official pattern).
- For a **universal macOS** updater target, set a custom `target: "macos-universal"` so the static JSON `platforms` key matches.

### Notarization (macOS)

v1 used `signingIdentity: "-"` (ad-hoc). v2 needs a Developer ID. Two toolchains:

- **`xcrun notarytool`** (Apple, needs macOS + App Store Connect API key or Apple ID): `notarytool submit <zip> --wait` then `stapler staple <app>`. Requires hardened runtime (Tauri defaults `hardenedRuntime: true`).
- **`rcodesign` / `apple-codesign`** (Rust crate `rcodesign`, installable on Linux/CI): signs + notarizes + staples in one flow **without Xcode** — ideal for Linux CI runners or fully automated pipelines. Recommended for the CI matrix since cross-platform builds may not all run on macOS.

Set `bundle.macOS.signingIdentity` to the Developer ID Application cert name. Notarization is a build/release-pipeline step, not a runtime architecture change.

### Windows signing

EV or standard code-signing cert via `bundle.windows.certificateThumbprint` + `timestampUrl`, or the `signCommand`/custom-sign-command hook for remote/HSM signing. Needed for SmartScreen reputation.

### Integration with existing release pipeline

The v1 release was manual ad-hoc. v2 automates: CI matrix builds → signs → notarizes (macOS) → produces updater artifacts + sigs → publishes a static `latest.json` to GitHub Releases (or a CDN) → the `pubkey` embedded in the app verifies future downloads. No change to the app's *runtime* architecture; this is release engineering.

### New vs Modified

| NEW | MODIFIED |
|---|---|
| `tauri-plugin-updater` + `tauri-plugin-process` registration | `tauri.conf.json` — `createUpdaterArtifacts`, `plugins.updater`, macOS signing identity |
| `checkForUpdates` Tauri command + channel | `lib.rs` — plugin registration in `.setup` |
| Update UI (modal/drawer with progress) | `bundle.macOS.signingIdentity` — Developer ID |
| Signing/notarization CI steps (`rcodesign` or `notarytool`) | `scripts/prepare-sidecar.sh` / release script |
| Embedded `pubkey` + secret management for private key | `RELEASE_NOTES.md` / release docs |

**Confidence: HIGH** — fully documented official Tauri plugin; no novel architecture.

## New vs Modified Component Matrix (all features)

| Component | F1 Graph | F2 Catalog | F3 Visuals | F4 Shell | F5 LLM | F6 X-plat | F7 Update |
|---|:--:|:--:|:--:|:--:|:--:|:--:|:--:|
| `domain/session.rs` | M | M | — | — | — | — | — |
| `domain/node_catalog.rs` | — | **N** | — | — | — | — | — |
| `application/graph_edit.rs` | M | M | — | — | — | — | — |
| `audio/compiler.rs` + `synthdefs.rs` | — | M | — | — | — | — | — |
| `audio/runtime_manager.rs` | M | M | — | — | — | — | — |
| `visual/compiler.rs` | — | M | M | — | — | — | — |
| `visual/bevy_runtime.rs` + `bevy_sidecar.rs` | — | — | M | — | — | — | — |
| `visual/protocol.rs` | — | — | M | — | — | — | — |
| `application/agent_planner.rs` + `agent_command.rs` | — | M | — | — | M | — | — |
| `application/providers/live_llm.rs` | — | — | — | — | **N** | — | — |
| `lib.rs` (commands + plugins) | — | — | — | — | M | — | M |
| `GraphViewport.tsx` | **R** | M | M | M | — | — | — |
| `App.tsx` shell | — | — | M | **R** | — | — | — |
| `session-client.ts` + zod | M | M | — | — | M | — | — |
| `tauri.conf.json` | — | — | M | — | — | M | M |
| `scripts/prepare-sidecar.sh` | — | — | — | — | — | M | M |
| CI / release pipeline | — | — | — | — | — | **N** | **N** |

(N = new, M = modified, R = rebuilt, — = untouched)

## Architectural Patterns to Follow (v2)

### Pattern: Catalog-as-Data, not Catalog-as-Code
Every node type's ports, params, synthdef mapping, and UI metadata live in one Rust data structure that flows to compiler, validation, palette, inspector, and agent context via `ts-rs`. No node-type knowledge is duplicated across language boundaries. **When:** adding any new node type. **Trade-off:** upfront refactor cost, but kills the multi-place enum-update tax permanently.

### Pattern: Two-Tier Connection Validation
Client-side `isValidConnection` is a UX hint; Rust `validate_route` is authority. **When:** any connect/reconnect. **Trade-off:** duplicated logic, but the duplication is mechanical (mirror of the same rule) and prevents blocked drags from looking allowed.

### Pattern: Mode-Flagged Sidecar, Not Two Binaries
The visual sidecar gains `--minimal` / `--behind` / `--headless-stream` modes (it already has `--minimal`) rather than shipping separate binaries. **When:** visuals-behind-grid. **Trade-off:** one fatter binary, but simpler bundling and runtime mode-switching.

### Pattern: Separate Channel for Hot Data
Frame streams (Option B) and download progress and LLM token streams each get their own `ipc::Channel`, never the JSON-lines control bus. **When:** any high-frequency or streaming data. **Why:** the v1 anti-pattern "one bus for everything including hot data" is already documented; v2 must not regress it.

### Pattern: Provider Chain, Not Provider Switch
Live LLM falls back to the local parser inside a chain of `PlannerProvider` impls, so the agent never hard-fails when offline. **When:** live agent. **Trade-off:** tiny indirection, huge reliability win.

## Anti-Patterns to Avoid (v2-specific)

### Anti-Pattern: Frontend-owned node catalog
**What:** defining node types in TS and syncing to Rust. **Why bad:** three copies drift; Rust validation and the audio compiler can't read it without a round-trip; agent context becomes inconsistent. **Instead:** Rust-owned catalog, `ts-rs`-generated frontend types.

### Anti-Pattern: Shared window handle for visuals
**What:** letting Bevy render into the Tauri window's raw handle to get "free" compositing. **Why bad:** violates the documented "visual runtime must be a separate adapter" product constraint; couples lifecycles. **Instead:** Option A (behind-window) or Option B (frame stream) — both keep the sidecar a separate process.

### Anti-Pattern: Agent writes session state directly
**What:** live LLM tool-calls that mutate the graph without the command/approval pipeline. **Why bad:** erases the v1 safety differentialiator (human override, risk tiering). **Instead:** tools only *propose* `TypedCommand`s that flow through the unchanged ownership/approval gates.

### Anti-Pattern: Full-topology reapply for every parameter tweak
**What:** treating all params as topology-changing. **Why bad:** audible glitches on live performance. **Instead:** catalog `live_routable` flag drives `runtime_manager` to use `/n_set` for routable params and full reapply only for structural ones.

### Anti-Pattern: Conflating notarization with updater signing
**What:** assuming a notarized app auto-updates. **Why bad:** they're independent systems; missing either breaks a different trust path. **Instead:** ship both — OS notarization for Gatekeeper, Tauri updater `pubkey`/`.sig` for artifact authenticity.

## Suggested Build Order (dependency-respecting)

The roadmapper should sequence phases to respect these dependencies:

1. **Node Catalog (F2)** — *first, unblocks F1, F3, F5.* Everything else references real node types. Low risk, high leverage.
2. **Graph UX Rebuild (F1)** — *depends on F2.* Custom typed-handle nodes + reconnect need the catalog. Flips `nodesDraggable` on.
3. **Visuals-behind-grid Option A spike (F3, decision gate)** — *depends on F2 (catalog-driven visual compiler).* One-day macOS spike to confirm/kill Option A before committing.
4. **Visuals-behind-grid full (F3)** — *depends on the spike.* Implement whichever mode the spike validates, with the other as fallback.
5. **Focused Shell (F4)** — *depends on F1 + F3.* The new shell wraps the new (transparent, visuals-behind) graph surface. Doing it earlier wastes effort on a surface about to change.
6. **Live LLM Agent (F5)** — *depends on F2 (catalog in context packet).* Independent of shell; can parallelize with F4. Inherits all safety.
7. **Cross-platform builds (F6)** — *depends on nothing architectural, but practically after F3 (sidecar modes) settles.* Bundle per-triple sidecars + scsynth.
8. **Notarization + Auto-updater (F7)** — *last.* Needs the cross-platform artifacts (F6) to sign/notarize/update. Release engineering on top of a frozen feature set.

**Parallelization:** F4 and F5 can run concurrently after F1+F2 land. F6 and F7 are sequential at the end. The F3 spike is a fast gate that should happen early to de-risk the whole visual story.

**Dependency rationale (why this order):**
- F2 before F1: graph nodes need real types before you rebuild the surface that renders them.
- F2 before F3: the visual compiler must read the catalog to map nodes → visual elements/params.
- F2 before F5: the agent context packet must carry catalog schemas for valid tool-calling.
- F3 spike before F3 full: de-risk the single most uncertain v2 decision with a day of work, not a phase.
- F1+F3 before F4: the shell composes the graph + transparent-visuals surface; rebuilding the shell first means rebuilding it twice.
- F6 before F7: you sign/notarize the artifacts you can build; updater needs the cross-platform matrix.

## Integration Points Summary

| Boundary | Communication | v2 change |
|---|---|---|
| UI ↔ Rust core | Tauri `invoke` commands (existing) | Add `MoveNode`, `RerouteEdge`, `InstantiateCatalogNode`, `send_agent_message_stream`, `check_for_updates` |
| UI ↔ Rust core (streaming) | Tauri `ipc::Channel` (new for v2) | LLM token stream, updater progress, visual frame stream (Option B) |
| Rust core ↔ SC | OSC over UDP (`rosc`, existing) | Unchanged; catalog drives synthdef selection |
| Rust core ↔ visual sidecar | JSON-lines stdio (existing) + window-frame msgs + frame channel (new) | `SetWindowFrame` (Option A); frame channel (Option B) |
| Rust core ↔ LLM provider | HTTPS streaming (`async-openai`) | New; wrapped behind `PlannerProvider` trait |
| Rust core ↔ persistence | SQLite + JSON (existing) | Schema bump for `catalog_id`, `Node.position` |

## Scaling Considerations

| Concern | At a small patch | At a dense 40-node session | At extreme complexity |
|---|---|---|---|
| Graph render perf | Trivial | xyflow memoization + virtualization needed | Group nodes / subgraphs (future) |
| Topology reapply latency | Inaudible | Batch edits into one reapply; prefer `/n_set` | Incremental adapter diffing (v1 already designed for this) |
| Visual frame bandwidth (Option B) | N/A | 30fps @ panel-res RGBA is fine | Drop to Option A or lower res |
| Agent context size | Small | `SessionContextBounds` already truncates | Tighter bounds + catalog-aware summarization |

v2 "scaling" is still single-machine session complexity, not multi-user. The architecture stays local-first.

## Research Flags for Phases

- **F3 (visuals):** The Option A vs Option B decision MUST be settled by a vertical spike before phase planning commits to one. Flag the phase for spike-first.
- **F2 (catalog):** Schema migration of v1 sessions (adding `catalog_id` to existing `Node`s, mapping old `AudioPrimitive` → catalog entry) needs a migration test phase. `CURRENT_SCHEMA_VERSION` bump.
- **F5 (LLM):** Tool-schema generation from serde types needs validation — confirm the generated JSON schema matches what OpenAI-compatible tool-calling expects. Possible follow-up research.
- **F6 (cross-platform):** Linux SC shared-library bundling is the known friction point; may need a dedicated research spike for `.deb`/`appimage` dependency strategy.

## Sources

- **Tauri 2 configuration reference** (`macOSPrivateApi`, `transparent`, `externalBin` target-triple pattern, `createUpdaterArtifacts`, `hardenedRuntime`, platform-specific conf merge) — https://v2.tauri.app/reference/config/ — **HIGH** (official, current)
- **Tauri 2 Window Customization** (transparent titlebar, NSWindow background via `objc2-app-kit`, decorations) — https://v2.tauri.app/learn/window-customization/ — **HIGH** (official, updated 2026-06)
- **Tauri 2 Updater plugin** (`tauri-plugin-updater`, `pubkey`/`endpoints`, `TAURI_SIGNING_PRIVATE_KEY`, per-platform `.sig` artifacts, `ipc::Channel` progress, custom `macos-universal` target, Rust ≥1.77.2) — https://v2.tauri.app/plugin/updater/ — **HIGH** (official, updated 2025-11)
- **Existing v1 codebase** — `domain/session.rs`, `application/{graph_edit,session_store,agent_planner}.rs`, `audio/{compiler,synthdefs,runtime_manager}.rs`, `visual/{compiler,bevy_sidecar,bevy_runtime,sidecar}.rs`, `bin/scrysynth-visual.rs`, `src/{App.tsx, components/session/GraphViewport.tsx, components/audio/PrimitivePalette.tsx, lib/session-client.ts}`, `tauri.conf.json`, `lib.rs` — **HIGH** (read directly; integration points grounded here)
- **Prior v1 architecture research** — `.planning/research/ARCHITECTURE.md` (2026-04-11) — **HIGH** (the baseline this extends)
- Bevy 0.18 render-to-texture / headless camera / `RenderTarget::Image` — **MEDIUM** (from Bevy engine knowledge; confirm against 0.18 docs during the F3 spike)
- `@xyflow/react` v12 `onReconnect`/`isValidConnection`/custom nodes/multi-handle — **MEDIUM-HIGH** (React Flow v12 feature surface; confirm against reactflow.dev during F1 planning)
- `async-openai` streaming + tool-calling — **MEDIUM-HIGH** (well-established crate; confirm exact API surface during F5 planning)
- `rcodesign`/`apple-codesign` for notarization without Xcode — **MEDIUM** (recommended CI tooling; confirm version/flow during F7 planning)

---
*Architecture research for: Scrysynth v2.0 "Studio-Grade Instrument" feature integration*
*Researched: 2026-06-26*
