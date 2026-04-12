# Phase 4 Research: Agent Collaboration

**Phase:** 4 (Agent Collaboration)
**Researched:** 2026-04-12
**Goal:** Users can direct Scrysynth in natural language while keeping changes, ownership, and override behavior legible and safe.

## Requirements

| ID    | Description |
|-------|-------------|
| AGNT-01 | User can direct the system in natural language and have the agent propose or apply changes against the current canonical session. |
| AGNT-02 | User can see which nodes, macros, scenes, or controls are currently agent-controlled, shared, or user-controlled. |
| AGNT-03 | User can approve, reject, or cancel higher-risk agent actions before they mutate the live session. |
| AGNT-04 | User can reclaim control from the agent, freeze agent changes, or disable the conductor role without restarting the session. |
| UI-03  | User can inspect what changed in the session after a user or agent action through visible diffs, activity history, or equivalent structured feedback. |

## What Phases 1-3 Built

### Phase 1: Session Core & Recall
- Canonical `SessionDocument` with all domain types including `OwnershipRule`, `ControllerKind` (User/Agent/Shared), `OwnershipAssignment`
- TypeScript contracts via `ts-rs`
- Zustand mirror store with projections
- JSON persistence with save/open

### Phase 2: Playable Audio Graph
- `GraphEditCommand` enum with 8 variants (add/remove/route/param/bus mutations)
- Transactional `apply_graph_edit` with full validation
- SuperCollider adapter with lifecycle management
- Audio transport controls

### Phase 3: Performance Workspace
- `PerformanceCommand` enum (RecallScene, SaveVariation, RestoreVariation)
- View switching (graph/conversation/performance)
- Scene recall with hard-cut semantics
- `ConversationView` placeholder stub
- `OwnershipAssignment` displayed in NodeInspector but NOT enforced

## Existing Domain Types That Support Phase 4

The canonical session already defines ownership infrastructure:

```rust
pub struct OwnershipRule {
    pub id: String,
    pub scope: String,
    pub controller: ControllerKind,
    pub can_override: bool,
}

pub enum ControllerKind {
    User,
    Agent,
    Shared,
}

pub struct OwnershipAssignment {
    pub controller: ControllerKind,
    pub is_locked: bool,
}
```

Every `Node` carries `ownership: OwnershipAssignment`. Session-level `ownership_rules: Vec<OwnershipRule>` exist. `RuntimeKind::Agent` is defined but no agent runtime status is seeded.

**Critical gap:** No code currently checks ownership before applying mutations. `apply_graph_edit()` and `apply_performance_command()` ignore `OwnershipAssignment` entirely.

## Research Findings

### 1. Agent Adapter Architecture (AGNT-01)

**What the agent must do:**
1. Accept natural language input from the user
2. Interpret intent against the current session state
3. Produce typed commands (GraphEditCommand, PerformanceCommand, or new AgentCommand variants)
4. Route through the same validation/mutation pipeline as user actions

**Approach: No LLM in v1. Use a deterministic intent parser.**

Rationale: Shipping a real LLM integration in Phase 4 would introduce:
- External API dependency and latency
- Non-deterministic command generation
- Complex error handling for malformed AI output
- Security surface for prompt injection

Instead, Phase 4 should build the **agent infrastructure** with a deterministic intent parser that maps keyword patterns to typed commands. This proves the ownership, approval, and history pipeline works correctly with predictable inputs. LLM integration becomes a runtime swap of the parser for an API-backed service later.

**Intent parser design:**
- Recognizes a bounded grammar of session operations: add/remove nodes, set parameters, recall scenes, save variations
- Produces the same `GraphEditCommand` and `PerformanceCommand` types the UI already uses
- Each produced command carries an `ActorRef` identifying it as agent-originated
- All agent commands flow through the same validation pipeline as user commands

**New domain types needed:**

```rust
pub struct ActorRef {
    pub actor_id: String,       // "user" or "agent"
    pub correlation_id: String, // links proposal to approval/diff
}

pub struct AgentIntent {
    pub raw_input: String,
    pub parsed_commands: Vec<TypedCommand>,
    pub confidence: f64,
}

pub enum TypedCommand {
    GraphEdit(GraphEditCommand),
    Performance(PerformanceCommand),
}
```

### 2. Ownership Enforcement (AGNT-02, AGNT-04)

**Current state:** Ownership fields exist but are inert. No mutation handler checks them.

**What enforcement means:**
- Before applying any mutation, check the target node's `OwnershipAssignment`
- If `controller == Agent` and the actor is the user: allow (user always overrides)
- If `controller == User` and the actor is the agent: reject unless `can_override == true`
- If `is_locked == true`: reject all mutations regardless of actor
- Shared nodes allow both actors

**Enforcement layer placement:**
Insert an ownership check in `SessionStore::mutate_current()` as a pre-mutation gate. This keeps enforcement centralized — every mutation path (user, agent, performance) passes through the same gate.

**Freeze agent (AGNT-04):**
Add a session-level flag `agent_frozen: bool` (default false). When true, all agent-originated commands are rejected at the ownership gate. User commands are unaffected. This is a single boolean toggle with no runtime restart required.

**Reclaim control (AGNT-04):**
Add a `reclaim_ownership` command that batch-sets all agent-controlled nodes to user-controlled. This is a new command type, not a graph edit.

### 3. Approval Gates (AGNT-03)

**Risk classification:**
Not all agent actions carry equal risk. Define risk tiers:

| Risk Tier | Examples | Behavior |
|-----------|----------|----------|
| Low | SetParameterValue (small delta), RestoreVariation | Auto-apply |
| Medium | AddNode, SetNodeEnabled, RecallScene | Auto-apply if node is shared or agent-owned |
| High | RemoveNode, RemoveRoute, ownership changes | Require explicit approval |

**Approval flow:**
1. Agent produces a command with `ActorRef { actor_id: "agent", correlation_id }`
2. Command hits the ownership gate
3. If risk tier requires approval AND target is user-owned: create a `PendingAction`
4. UI shows the pending action with a diff preview
5. User approves or rejects
6. On approval: apply the command through normal mutation path
7. On rejection: discard the command, log the rejection

**New domain types:**

```rust
pub struct PendingAction {
    pub id: String,
    pub correlation_id: String,
    pub command: TypedCommand,
    pub risk_tier: RiskTier,
    pub created_at: String,
    pub status: PendingActionStatus,
}

pub enum RiskTier {
    Low,
    Medium,
    High,
}

pub enum PendingActionStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}
```

`PendingAction` lives in the `SessionDocument` as `pending_actions: Vec<PendingAction>` so it persists across saves and survives UI churn.

### 4. Activity History and Diffs (UI-03)

**What a history entry needs:**
- Who made the change (user vs agent)
- What changed (typed command + before/after snapshot)
- When (timestamp)
- Correlation ID linking to conversation if agent-initiated

**Approach: Append-only action log.**

Add `ActionHistoryEntry` to the session:

```rust
pub struct ActionHistoryEntry {
    pub id: String,
    pub timestamp: String,
    pub actor: ActorRef,
    pub command: TypedCommand,
    pub diff_summary: DiffSummary,
}

pub struct DiffSummary {
    pub description: String,            // human-readable: "Set oscillator frequency to 440Hz"
    pub affected_node_ids: Vec<String>,
    pub before_snippet: Option<String>,  // JSON snapshot of changed fields
    pub after_snippet: Option<String>,
}
```

Store as `action_history: Vec<ActionHistoryEntry>` in `SessionDocument`. Cap at a reasonable size (e.g., last 200 entries) to prevent unbounded growth.

**Frontend rendering:**
- Add an `ActivityPanel` component in the conversation/performance views
- Each entry shows actor badge, timestamp, description, and expandable diff
- Filterable by actor (all / user / agent)

### 5. Conversation View Upgrade (AGNT-01)

**Current:** Placeholder with static text.

**Needed:**
- Message list showing user inputs and agent responses
- Input field for natural language commands
- Agent response rendering: parsed intent + command preview
- Approval controls for pending actions
- Activity history feed

**Frontend store additions:**
- `conversationMessages: ConversationMessage[]`
- `pendingActions: PendingAction[]` (mirrored from session)
- `agentFrozen: boolean`

**No new frontend dependencies needed.** React + Zustand + existing Radix primitives handle everything.

### 6. Technical Constraints

- **No external API dependency in v1** — deterministic parser only
- **Same Tauri IPC pattern** — new commands follow `#[tauri::command]` + `State<Mutex<SessionStore>>`
- **Same clone-and-replace mutation pattern** — ownership gate wraps existing validation
- **Ownership enforcement is backend-only** — frontend displays status but never decides access
- **Generated TypeScript types updated** — `PendingAction`, `ActionHistoryEntry`, `ActorRef`, `RiskTier`
- **No new Rust crate dependencies needed** for the parser — simple string matching with `str::contains` and bounded tokenization suffices for v1

## Recommended Plan Breakdown

### Plan 1: Agent Command Layer and Ownership Enforcement
- Add `ActorRef`, `TypedCommand`, `AgentIntent` to domain
- Add ownership gate to `SessionStore::mutate_current()`
- Add `agent_frozen` flag and `reclaim_ownership` command
- Add deterministic intent parser
- Add `AgentCommand` Tauri IPC handler
- Wire agent commands through ownership enforcement
- Add Rust tests for ownership gate, freeze, reclaim
- Update TypeScript contracts

### Plan 2: Activity History and Approval Gates
- Add `ActionHistoryEntry`, `DiffSummary`, `PendingAction`, `RiskTier` to domain
- Add risk classification logic for command types
- Implement approval flow (create pending action, approve, reject)
- Add action history logging to mutation pipeline
- Add Tauri IPC handlers for approve/reject
- Add Rust tests for approval flow and history
- Update TypeScript contracts

### Plan 3: Conversation View and Agent Integration
- Replace ConversationView placeholder with message list + input
- Add `ActivityPanel` component for action history
- Wire conversation input to agent intent parser via IPC
- Display agent responses with parsed commands
- Add pending action approval/rejection UI
- Add agent freeze toggle
- Wire ownership badges throughout existing UI
- Add frontend tests
