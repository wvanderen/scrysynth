---
phase: 04-agent-collaboration
plan: 03
type: execute
wave: 1
depends_on:
  - 04-agent-collaboration-01
  - 04-agent-collaboration-02
files_modified:
  - src/components/workspace/ConversationView.tsx
  - src/components/workspace/ActivityPanel.tsx
  - src/components/workspace/PendingActionCard.tsx
  - src/components/session/NodeInspector.tsx
  - src/components/audio/PrimitivePalette.tsx
  - src/store/sessionStore.ts
  - src/lib/session-client.ts
  - src/App.tsx
  - src/__tests__/ConversationView.test.tsx
  - src/__tests__/ActivityPanel.test.tsx
autonomous: true
requirements:
  - AGNT-01
  - AGNT-02
  - AGNT-03
  - AGNT-04
  - UI-03
must_haves:
  truths:
    - User can type natural language commands in the conversation view and see parsed intents with command previews before execution.
    - User can see a scrollable activity history with actor badges, timestamps, and diff descriptions for all mutations.
    - User can approve or reject pending agent actions from the conversation view.
    - User can freeze/unfreeze the agent and reclaim ownership from the conversation view.
    - Node inspector shows ownership controller badge (user/agent/shared) and lock status.
    - All new frontend components use the same Zustand store and IPC patterns as existing components.
  artifacts:
    - path: src/components/workspace/ConversationView.tsx
      provides: full conversation UI with message list, input, approval controls, freeze toggle
      contains: "ConversationView"
    - path: src/components/workspace/ActivityPanel.tsx
      provides: scrollable action history with actor badges and diff summaries
      contains: "ActivityPanel"
  tasks:
    - id: 1
      title: Update session-client.ts with new Zod schemas and IPC wrappers
      action: implement
      file: src/lib/session-client.ts
      description: Add Zod schemas for PendingAction, RiskTier, ActionHistoryEntry, DiffSummary, ActorRef, TypedCommand, AgentIntent. Add invoke wrappers for send_agent_message, toggle_agent_freeze, reclaim_ownership, approve_pending_action, reject_pending_action.
    - id: 2
      title: Update Zustand store with agent state and actions
      action: implement
      file: src/store/sessionStore.ts
      description: Add conversationMessages, pendingActions, actionHistory, agentFrozen to state. Add actions: sendAgentMessage (calls IPC, parses response, updates messages), toggleFreezeAgent, reclaimOwnership, approvePendingAction, rejectPendingAction. Update projections to derive pending actions and history from session.
    - id: 3
      title: Replace ConversationView placeholder with full implementation
      action: implement
      file: src/components/workspace/ConversationView.tsx
      description: Build message list rendering user inputs and agent responses. Add input field that calls sendAgentMessage. Display parsed intent and command preview in agent response bubbles. Add freeze/unfreeze toggle button. Add reclaim ownership button. Show agent frozen status.
    - id: 4
      title: Create PendingActionCard component
      action: implement
      file: src/components/workspace/PendingActionCard.tsx
      description: Card showing pending action details: risk tier badge, command description, affected nodes, created timestamp. Approve and Reject buttons calling store actions. Display within ConversationView when pending actions exist.
    - id: 5
      title: Create ActivityPanel component
      action: implement
      file: src/components/workspace/ActivityPanel.tsx
      description: Scrollable list of ActionHistoryEntry items. Each item shows actor badge (user/agent), timestamp, diff description, affected nodes. Filter buttons for all/user/agent. Add to PerformanceView as a tab or collapsible section.
    - id: 6
      title: Update NodeInspector with ownership badges
      action: implement
      file: src/components/session/NodeInspector.tsx
      description: Enhance the existing Ownership display section with a colored badge for controller type (user=blue, agent=amber, shared=green). Show lock icon when isLocked is true. Add quick-set buttons to change ownership controller.
    - id: 7
      title: Write frontend tests
      action: implement
      file: src/__tests__/ConversationView.test.tsx, src/__tests__/ActivityPanel.test.tsx
      description: Test ConversationView renders messages and input. Test sending a message calls the store action. Test ActivityPanel renders history entries with correct actor badges. Test filter toggles work. Test PendingActionCard approve/reject calls correct store actions.
