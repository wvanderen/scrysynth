---
phase: 04-agent-collaboration
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/domain/session.rs
  - src-tauri/src/application/mod.rs
  - src-tauri/src/application/agent_command.rs
  - src-tauri/src/application/session_store.rs
  - src-tauri/src/application/graph_edit.rs
  - src-tauri/src/application/performance_command.rs
  - src-tauri/src/lib.rs
  - src-tauri/tests/agent_commands.rs
autonomous: true
requirements:
  - AGNT-01
  - AGNT-02
  - AGNT-04
must_haves:
  truths:
    - User can direct the system through a deterministic intent parser that produces typed GraphEditCommand or PerformanceCommand variants.
    - Every mutation checks ownership before applying; agent-originated commands are rejected if the target node is user-owned and locked.
    - User can freeze the agent with a single toggle that rejects all agent commands without restarting the session.
    - User can reclaim control by batch-setting all agent-owned nodes to user-owned.
    - All agent commands carry an ActorRef identifying the origin for audit and display.
    - TypeScript contracts include ActorRef, TypedCommand, and AgentIntent types.
  artifacts:
    - path: src-tauri/src/application/agent_command.rs
      provides: parse_agent_intent, apply_agent_command, ownership gate, freeze toggle, reclaim
      contains: "ActorRef"
    - path: src-tauri/tests/agent_commands.rs
      provides: integration tests for intent parsing, ownership enforcement, freeze, reclaim
      contains: "parse_agent_intent"
  tasks:
    - id: 1
      title: Add ActorRef, TypedCommand, and AgentIntent to domain
      action: implement
      file: src-tauri/src/domain/session.rs
      description: Add ActorRef (actor_id, correlation_id), TypedCommand wrapping GraphEditCommand and PerformanceCommand, and AgentIntent (raw_input, parsed_commands, confidence). Add agent_frozen bool to SessionDocument. Register all for ts-rs generation.
    - id: 2
      title: Add ownership gate to SessionStore mutation pipeline
      action: implement
      file: src-tauri/src/application/session_store.rs
      description: Add check_ownership method that takes ActorRef + target node IDs. Reject agent commands on user-owned locked nodes. Reject all agent commands when agent_frozen is true. User always passes. Integrate gate into mutate_current so all command paths are covered.
    - id: 3
      title: Implement deterministic intent parser
      action: implement
      file: src-tauri/src/application/agent_command.rs
      description: Create parse_agent_intent that maps keyword patterns to TypedCommand variants. Support: "add oscillator/noise/filter/delay/mixer", "remove [node]", "set [parameter] to [value] on [node]", "recall scene [name]", "save variation [name] for scene [id]", "restore variation [id]". Return AgentIntent with parsed commands and confidence score.
    - id: 4
      title: Implement agent command handler with ownership enforcement
      action: implement
      file: src-tauri/src/application/agent_command.rs
      description: Create apply_agent_command that takes AgentIntent, runs ownership gate, then applies each TypedCommand through existing apply_graph_edit or apply_performance_command handlers. Return results including any rejected commands with reasons. Add AgentCommandError enum.
    - id: 5
      title: Add freeze toggle and reclaim ownership commands
      action: implement
      file: src-tauri/src/application/agent_command.rs
      description: Add toggle_agent_freeze command that flips session.agent_frozen. Add reclaim_ownership command that sets all agent-owned nodes to user-owned. Both go through mutate_current. Both are user-only actions.
    - id: 6
      title: Register application module and Tauri IPC handlers
      action: implement
      file: src-tauri/src/application/mod.rs, src-tauri/src/lib.rs
      description: Add pub mod agent_command. Add send_agent_message, toggle_agent_freeze, reclaim_ownership Tauri commands. Register in invoke_handler.
    - id: 7
      title: Write integration tests
      action: implement
      file: src-tauri/tests/agent_commands.rs
      description: Test intent parsing for all supported patterns. Test ownership gate rejects agent on locked user-owned nodes. Test ownership gate allows agent on shared/agent-owned nodes. Test freeze toggle blocks all agent commands. Test reclaim transfers all agent nodes to user. Test user always passes ownership gate. Test error cases (unparseable input, missing targets).
