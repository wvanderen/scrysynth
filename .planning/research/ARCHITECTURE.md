# Architecture Patterns

**Domain:** Graph-native desktop audiovisual instrument with shared canonical session state
**Project:** Scrysynth
**Researched:** 2026-04-11
**Overall confidence:** HIGH

## Recommended Architecture

Build Scrysynth as an event-sourced application core inside the Tauri/Rust host, with the UI, agent layer, SuperCollider runtime, and visual runtime all acting as clients of the same canonical session model. Do not let either runtime become the source of truth. The app core owns semantic state; runtime adapters own execution details.

Use this shape:

```text
Frontend UI (graph / chat / performance)
        |
        | typed commands + subscriptions
        v
Tauri IPC boundary
        |
        v
Rust App Core
  - Session Store (canonical graph + metadata)
  - Command Handlers
  - Reducer / Validator
  - Event Bus
  - Scheduler / Transaction Coordinator
  - Ownership + Safety Policy
  - Persistence / Snapshotting
        |
        +--> Agent Orchestrator
        |      - plans mutations
        |      - proposes commands
        |      - consumes state/events
        |
        +--> Audio Runtime Adapter
        |      - compiles graph primitives to SC resources
        |      - sends OSC to scsynth
        |      - listens for status / node feedback
        |
        +--> Visual Runtime Adapter
               - compiles graph primitives to visual engine messages
               - sends control + scene updates
               - reports runtime health / acknowledgements
```

This gives Scrysynth a clean split between semantic authorship and runtime realization:

- `Session graph`: what the instrument means
- `Domain events`: what changed and why
- `Runtime projections`: how audio and visuals should realize the meaning
- `Runtime feedback`: what actually happened in execution

## Core Architectural Rule

All mutations flow through one path:

```text
intent -> validated command -> domain event(s) -> session state update -> adapter projections -> runtime side effects -> runtime feedback events
```

Never allow direct UI -> runtime or agent -> runtime mutation for anything that should persist in the session. If a change matters to the user, it must exist first as session state.

## Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|----------------|-------------------|
| Frontend UI | Presents graph, conversation, controls, transport, scene state; issues user intents; renders projections and health | Tauri commands/channels only |
| App Core / Session Store | Owns canonical Session, Node, Route, Bus, Macro, Binding, Scene, Variation, OwnershipRule, AgentRole state | All components |
| Command Layer | Accepts typed intents from UI/agents, enforces invariants, emits domain events | Frontend UI, Agent Orchestrator, App Core |
| Event Bus | Fan-out of domain events, projection events, runtime feedback, telemetry | App Core, UI subscriptions, adapters, agents |
| Scheduler / Transaction Coordinator | Batches graph edits, schedules scene changes, sequences multi-runtime apply steps | Command Layer, adapters |
| Ownership + Safety Policy | Human override, locks, confirmation gates, reclaim/delegation rules | Command Layer, Agent Orchestrator, UI |
| Agent Orchestrator | Turns conversation and goals into proposed graph mutations or macro actions; never writes session state directly | Command Layer, Event Bus |
| Audio Runtime Adapter | Maps canonical graph to SuperCollider groups, synths, buses, buffers, and OSC commands | Scheduler, Event Bus, scsynth |
| Visual Runtime Adapter | Maps canonical graph to visual runtime nodes/scenes/params through a separate protocol | Scheduler, Event Bus, visual engine |
| Persistence Layer | Saves snapshots plus optional event log and media/runtime metadata | App Core |
| Observability Layer | Structured logs, runtime health, diff inspection, adapter metrics | All backend components |

## Canonical Model vs Runtime Model

Preserve two explicit layers.

### Canonical Session Layer

Use stable product terms only:

- `Session`
- `Node`
- `Route`
- `Bus`
- `Macro`
- `Binding`
- `Scene`
- `Variation`
- `OwnershipRule`
- `AgentRole`

This layer is user-facing, persisted, diffable, and shared across UI, agents, audio, and visuals.

### Runtime Projection Layer

Each adapter owns a compiled projection:

- audio projection -> SC synthdefs, groups, node IDs, bus indices, buffer handles, control mappings
- visual projection -> engine-specific scene graph IDs, resources, uniforms/parameters, transport bindings

Never persist raw runtime IDs as canonical product state. Persist only enough metadata to reconcile and rebuild projections.

## Data Flow

### 1. Authoring Flow

```text
Human action or agent proposal
-> typed command
-> validation against session schema + ownership rules
-> domain event(s)
-> reducer updates canonical session
-> projection builders derive adapter diffs
-> adapters apply diffs to runtimes
-> adapters publish success/failure/health events
-> UI updates from canonical state + runtime status
```

### 2. Transport / Performance Flow

```text
play/stop/tempo/macro gesture
-> transport command
-> session transport state update
-> event bus emits transport changed
-> audio + visual adapters consume same transport event
-> each runtime applies engine-specific timing/control message
```

### 3. Scene Change Flow

```text
scene recall request
-> transaction coordinator freezes target scope
-> reducer swaps scene-linked canonical values
-> adapter diffs computed
-> audio adapter applies safe order first
-> visual adapter applies paired scene update
-> commit complete event emitted
```

Scene recall should be transactional at the session layer even if runtime application is stepwise.

### 4. Runtime Feedback Flow

```text
runtime status / node end / resource failure / ack
-> adapter translates engine-specific signal into typed runtime event
-> event bus publishes runtime event
-> app core updates ephemeral runtime state only
-> UI surfaces health / mismatch / degraded mode
```

Do not let runtime feedback mutate canonical creative intent except through explicit recovery commands.

## Control Flow

### Control Plane

The Rust app core is the control plane. It decides:

- what the current session means
- whether a mutation is allowed
- how a mutation gets ordered
- which adapters must react
- what constitutes success, retry, rollback, or degraded operation

### Execution Plane

The audio and visual runtimes are execution planes. They should be replaceable without redefining the product model.

This matters because SuperCollider is excellent at synth execution and bus-based routing, but its node tree, bus indices, and OSC commands are implementation details, not product semantics. The same should be true for the visual engine.

## Event Bus and Adapter Coordination

Use one internal typed event bus in Rust. Do not use Tauri frontend events as the primary internal bus.

Reason: Tauri documents its event system as suitable for simple multi-producer/multi-consumer communication, but not for low-latency or high-throughput streaming; channels are the recommended mechanism for ordered streaming. That makes Tauri events appropriate for UI notifications, not for core runtime coordination.

### Prescriptive Event Split

Use three lanes:

1. `Domain events`
   - durable or at least replayable within session lifetime
   - examples: `NodeAdded`, `RoutePatched`, `MacroMapped`, `SceneRecalled`, `OwnershipChanged`

2. `Projection events`
   - internal adapter work queue
   - examples: `AudioProjectionUpdated`, `VisualProjectionUpdated`, `TransportTickChanged`

3. `Runtime events`
   - ephemeral execution feedback
   - examples: `ScNodeEnded`, `ScSyncComplete`, `VisualRuntimeDisconnected`, `AdapterApplyFailed`

### Coordination Pattern

Adapters subscribe to projection events, not raw UI actions.

Recommended apply cycle:

```text
domain event batch
-> compute adapter diffs
-> emit projection work items with correlation ID
-> adapters apply in deterministic order
-> adapters emit ack/failure with same correlation ID
-> coordinator marks batch settled
```

Use correlation IDs for every multi-step mutation. Without them, debugging agent-driven changes and scene transitions will get painful fast.

### Runtime Adapter Rules

- adapters are pure translators at their input boundary: canonical diff in, runtime operations out
- adapters may keep ephemeral caches for reconciliation and ID mapping
- adapters must expose `connect`, `warmup`, `apply_diff`, `sync`, `health`, `teardown`
- adapters must never invent canonical entities on their own
- adapter failures should degrade locally before they poison the entire session

## SuperCollider-Specific Pattern

Design the audio adapter around SuperCollider's real primitives: nodes, groups, audio buses, control buses, buffers, synthdefs, and OSC commands. This is a HIGH-confidence recommendation from the official SC server architecture and command reference.

### Recommended Audio Projection Shape

```text
Session graph
-> audio projection compiler
-> SC resource plan
   - synthdefs to load
   - groups to create
   - buses to allocate/map
   - buffers to allocate/load
   - node ordering plan
-> OSC bundle emission
-> scsynth
```

### SC-Specific Best Practices

- model execution order explicitly because SC node tree order defines processing order
- reserve stable group structure early: e.g. `root -> input -> instruments -> fx -> master -> analysis`
- prefer bus-based routing and control mapping over ad hoc synth-to-synth coupling
- batch related mutations into OSC bundles and use `/sync` for asynchronous completion boundaries when needed
- treat node IDs, bus indices, and buffer IDs as adapter-managed resources, not user-facing identifiers

### Why This Pattern Fits Scrysynth

SC already assumes dynamic creation/destruction/repaching through buses and ordered node trees. That matches a graph-native instrument if Scrysynth compiles its graph into SC's execution model instead of exposing SC's raw model directly.

## Visual Runtime Pattern

Keep the visual runtime separate but architecturally symmetrical with audio.

Required adapter contract:

- accepts canonical scene/control/transport diffs
- supports deterministic parameter application
- supports health and readiness reporting
- can acknowledge when a scene or preset has been applied
- can operate even if audio is still the product priority

Do not require graph isomorphism between audio and visuals in v1. Shared semantics should come from common canonical abstractions like `Scene`, `Macro`, `Binding`, `Transport`, and ownership state, not from forcing both runtimes to share the same engine-level node schema.

## Patterns to Follow

### Pattern 1: Command -> Event -> Reduce -> Project

**What:** All user and agent intents become validated commands, then domain events, then state reduction, then runtime projection.
**When:** Always; this is the default mutation path.
**Why:** It preserves legibility, auditability, undo/redo potential, and safe agent collaboration.
**Example:**

```typescript
type Command =
  | { type: 'AddNode'; payload: AddNodeInput; actor: ActorRef }
  | { type: 'RecallScene'; payload: { sceneId: string }; actor: ActorRef }

type DomainEvent =
  | { type: 'NodeAdded'; nodeId: string; correlationId: string }
  | { type: 'SceneRecalled'; sceneId: string; correlationId: string }

function handleCommand(command: Command, state: SessionState): DomainEvent[] {
  validate(command, state)
  return decide(command, state)
}

function apply(events: DomainEvent[], state: SessionState): SessionState {
  return events.reduce(reduceEvent, state)
}
```

### Pattern 2: Transactional Scene Apply

**What:** Scene recall is computed as one canonical transaction, even if adapter application happens in multiple steps.
**When:** Scene recall, variation morph, macro snapshots.
**Why:** Prevents half-applied states across chat, graph, audio, and visuals.

### Pattern 3: Adapter Diffing Instead of Full Rebuild

**What:** Compute runtime diffs from prior projection to next projection.
**When:** Continuous editing and live performance.
**Why:** Full runtime rebuilds are too glitch-prone for a live instrument.

### Pattern 4: Ownership-Aware Commands

**What:** Every mutation includes actor identity and ownership scope.
**When:** Human edits, agent proposals, delegated macros, automated transitions.
**Why:** Human override is a core product promise, not a UI affordance.

### Pattern 5: Ephemeral Runtime State Separate from Canonical State

**What:** Keep runtime health, connection state, node IDs, CPU load, and adapter warnings outside persisted session data.
**When:** Always.
**Why:** Prevents runtime noise from contaminating the creative model.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Runtime-Owned Truth

**What:** Letting SuperCollider patch structure or the visual engine scene graph become the real session source.
**Why bad:** Persistence, explainability, and agent coordination collapse into engine-specific state.
**Instead:** Keep a canonical app-owned session graph and compile it outward.

### Anti-Pattern 2: Chat-as-Primary-Authoring-Surface

**What:** Agent or chat actions bypass graph and performance models.
**Why bad:** Product drifts into "generator with windows" and loses instrument legibility.
**Instead:** Chat produces commands/proposals over the same session graph the UI manipulates.

### Anti-Pattern 3: One Event Bus for Everything Including Hot Data

**What:** Pumping high-frequency meter/control streams through the same general event path as semantic mutations.
**Why bad:** Backpressure, hard-to-debug lag, and polluted history.
**Instead:** Separate semantic events from fast telemetry streams; use channels/streams for hot data.

### Anti-Pattern 4: Leaking SC Primitives into Product Model Too Early

**What:** Designing `Node` and `Route` directly as `Synth`, `Group`, and raw bus indices.
**Why bad:** Locks the product to one engine and distorts UX around engine constraints.
**Instead:** Map product primitives to SC in the adapter.

### Anti-Pattern 5: Rebuilding Entire Runtimes on Every Edit

**What:** Tear down and recreate the whole audio/visual world after each graph change.
**Why bad:** Audio glitches, visual discontinuities, unusable live control.
**Instead:** Incremental projection diffs plus explicit sync points.

### Anti-Pattern 6: Cross-Runtime Two-Phase Commit Illusion

**What:** Pretending audio and visuals can always switch atomically at the exact same engine frame.
**Why bad:** Creates brittle orchestration and false guarantees.
**Instead:** Make the session transaction atomic, then expose adapter apply status and bounded skew.

## Suggested Internal Module Layout

```text
src/
  domain/
    session.rs
    commands.rs
    events.rs
    reducers.rs
    ownership.rs
    invariants.rs
  application/
    coordinator.rs
    scheduler.rs
    event_bus.rs
    projections/
      audio.rs
      visual.rs
  infrastructure/
    tauri/
    persistence/
    logging/
  runtimes/
    audio/
      sc_adapter.rs
      osc_client.rs
      resource_map.rs
    visual/
      visual_adapter.rs
      protocol.rs
      resource_map.rs
  agents/
    orchestrator.rs
    tools.rs
    proposal_policy.rs
```

## Build Order and Dependency Implications

Use this order for roadmap planning.

### Phase 1: Canonical Domain and Mutation Pipeline

Build first:

- session schema
- command types
- reducer/event model
- persistence snapshot format
- ownership and safety rules

Why first: every later subsystem depends on stable session semantics. If this is wrong, UI, agents, audio, and visuals all churn.

### Phase 2: Audio Runtime Adapter Foundation

Build second:

- SC process/connection management
- OSC client
- resource allocator for groups/buses/buffers/node IDs
- minimal graph-to-audio projection compiler
- runtime health reporting

Why second: the project explicitly prioritizes meaningful audio execution over elaborate visual editing. This is the shortest path to a playable instrument loop.

### Phase 3: UI Over Canonical State

Build third:

- graph read view
- transport/performance surface
- command dispatch from UI
- runtime status surfaces

Dependency note: UI should sit on top of the already-stable command/event system, not invent its own local state model.

### Phase 4: Agent Orchestration

Build fourth:

- proposal API
- command generation from agent intents
- confirmation and ownership gates
- audit trail and replay support

Dependency note: agents depend on stable commands and readable session diffs. Building them earlier tempts unsafe direct mutation.

### Phase 5: Visual Runtime Adapter

Build fifth:

- adapter contract implementation
- shared transport/macro binding support
- scene apply acknowledgements

Dependency note: keep visuals architecturally first-class, but sequence them after the audio/control loop proves the canonical model.

### Phase 6: Advanced Coordination

Build last:

- transactional scene recall
- variation morphing
- richer macro/binding system
- failure recovery and degraded mode handling

Why last: these features require the underlying eventing, ownership, and adapter contracts to already be correct.

## Dependency Rules for Roadmap Creation

- Do not schedule agent autonomy before ownership rules, command validation, and auditability exist.
- Do not schedule visual depth before audio projection and transport synchronization exist.
- Do not schedule rich graph editing before the canonical model has strict invariants.
- Do not schedule scene/variation sophistication before adapter diffing and correlation IDs exist.
- Do not schedule performance polish before runtime health and recovery signals are visible in the UI.

## Scalability Considerations

| Concern | At 100 users | At 10K users | At 1M users |
|---------|--------------|--------------|-------------|
| Session complexity per local machine | Primary concern is graph size, not user count | Same; product is local-first | Same; distribution affects releases/support, not runtime model |
| Event volume | In-memory typed bus is enough | Add bounded queues and telemetry sampling | Same architecture; optimize streams, not semantics |
| Runtime coordination | Manual reconciliation acceptable | Need stronger correlation IDs and retry policy | Same per-client architecture |
| Persistence | Snapshot only is acceptable | Snapshot + short event log helps recovery/debugging | Same, with migration tooling |
| UI responsiveness | Straight subscriptions are enough | Need memoized selectors and stream separation | Same architectural split remains valid |

For v1, "scaling" means sustaining larger sessions on one machine without losing legibility or control, not multi-user distributed systems.

## Prescriptive Decisions for the Roadmap

1. Treat the Rust app core as the product backend, even though this is a desktop app.
2. Freeze the canonical session vocabulary before deep UI or agent work.
3. Implement a typed internal Rust event bus; use Tauri events/channels only as app-edge IPC.
4. Build SuperCollider as an adapter compiled from canonical graph state, not as the graph model.
5. Define the visual runtime strictly behind the same adapter contract, even before engine choice is finalized.
6. Make correlation IDs, ownership metadata, and runtime health part of the architecture from day one.

## Research Flags

- Visual runtime contract details need follow-up once engine choice is narrowed; confidence is MEDIUM there because engine capabilities are intentionally unresolved.
- Scene transition semantics need deeper phase-specific research before committing to morphing vs stepped recall behaviors.
- Undo/redo and event-log retention policy should be decided once the mutation grammar stabilizes.

## Sources

- Tauri v2 docs, "Calling Rust from the Frontend" (updated 2025-11-19): https://v2.tauri.app/develop/calling-rust/ — HIGH confidence
- Tauri v2 docs, "Calling the Frontend from Rust" (updated 2025-05-12): https://v2.tauri.app/develop/calling-frontend/ — HIGH confidence
- Tauri v2 docs, "State Management" (updated 2025-05-07): https://v2.tauri.app/develop/state-management/ — HIGH confidence
- Tauri v2 docs, "Embedding External Binaries" / sidecars (updated 2026-01-07): https://v2.tauri.app/develop/sidecar/ — HIGH confidence
- SuperCollider 3.14 help, "Server Architecture": https://doc.sccode.org/Reference/Server-Architecture.html — HIGH confidence
- SuperCollider 3.14 help, "Server Command Reference": https://doc.sccode.org/Reference/Server-Command-Reference.html — HIGH confidence
- SuperCollider 3.14 help, "Node Messaging": https://doc.sccode.org/Guides/NodeMessaging.html — HIGH confidence
