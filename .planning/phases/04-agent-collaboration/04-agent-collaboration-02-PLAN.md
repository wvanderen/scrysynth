---
phase: 04-agent-collaboration
plan: 02
type: execute
wave: 1
depends_on:
  - 04-agent-collaboration-01
files_modified:
  - src-tauri/src/domain/session.rs
  - src-tauri/src/application/agent_command.rs
  - src-tauri/src/application/session_store.rs
  - src-tauri/src/application/graph_edit.rs
  - src-tauri/src/application/performance_command.rs
  - src-tauri/src/lib.rs
  - src-tauri/tests/approval_flow.rs
autonomous: true
requirements:
  - AGNT-03
  - UI-03
must_haves:
  truths:
    - High-risk agent commands (RemoveNode, RemoveRoute, ownership changes) create PendingActions that require explicit user approval before mutating the session.
    - Low and medium-risk agent commands auto-apply if ownership permits.
    - Every applied mutation logs an ActionHistoryEntry with actor, timestamp, command, and diff summary.
    - User can approve or reject pending actions; rejected actions are discarded and logged.
    - Action history is capped at 200 entries to prevent unbounded growth.
    - TypeScript contracts include PendingAction, RiskTier, ActionHistoryEntry, DiffSummary types.
  artifacts:
    - path: src-tauri/src/domain/session.rs
      provides: PendingAction, RiskTier, ActionHistoryEntry, DiffSummary domain types
      contains: "RiskTier"
    - path: src-tauri/tests/approval_flow.rs
      provides: integration tests for risk classification, approval, rejection, history logging
      contains: "RiskTier"
  tasks:
    - id: 1
      title: Add PendingAction, RiskTier, ActionHistoryEntry, DiffSummary to domain
      action: implement
      file: src-tauri/src/domain/session.rs
      description: Add RiskTier enum (Low, Medium, High). Add PendingAction with id, correlation_id, TypedCommand, RiskTier, created_at, status. Add DiffSummary with description, affected_node_ids, before_snippet, after_snippet. Add ActionHistoryEntry with id, timestamp, ActorRef, TypedCommand, DiffSummary. Add pending_actions and action_history fields to SessionDocument. Register all for ts-rs.
    - id: 2
      title: Implement risk classification for command types
      action: implement
      file: src-tauri/src/application/agent_command.rs
      description: Create classify_risk function mapping command variants to RiskTier. Low: SetParameterValue (small delta), RestoreVariation. Medium: AddNode, SetNodeEnabled, RecallScene, AddRoute. High: RemoveNode, RemoveRoute, ownership changes, ClearNodeBusAssignment. This determines whether approval is needed.
    - id: 3
      title: Implement approval flow
      action: implement
      file: src-tauri/src/application/agent_command.rs
      description: Modify agent command handler: if risk tier is High AND target node is user-owned, create PendingAction instead of applying. Add approve_pending_action handler that applies the stored command. Add reject_pending_action handler that marks it rejected. Both update the PendingActionStatus.
    - id: 4
      title: Add action history logging to mutation pipeline
      action: implement
      file: src-tauri/src/application/session_store.rs
      description: After every successful mutation (user or agent), create an ActionHistoryEntry with actor, timestamp, command, and a generated DiffSummary. Append to session.action_history. Cap at 200 entries by dropping oldest. Generate diff description by comparing the command type to produce human-readable strings.
    - id: 5
      title: Register approval Tauri IPC handlers
      action: implement
      file: src-tauri/src/lib.rs
      description: Add approve_pending_action and reject_pending_action Tauri commands. Both return updated SessionDocument. Register in invoke_handler.
    - id: 6
      title: Write integration tests
      action: implement
      file: src-tauri/tests/approval_flow.rs
      description: Test risk classification for all command types. Test high-risk agent commands create pending actions. Test low/medium-risk commands auto-apply. Test approve applies the command. Test reject discards it. Test action history logs entries with correct actor and diff. Test history cap at 200 entries. Test error cases (approve non-existent, already-resolved action).
