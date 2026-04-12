---
phase: 03-performance-workspace
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/domain/session.rs
  - src-tauri/src/application/mod.rs
  - src-tauri/src/application/performance_command.rs
  - src-tauri/src/lib.rs
  - src-tauri/tests/performance_commands.rs
autonomous: true
requirements:
  - CTRL-02
  - CTRL-03
must_haves:
  truths:
    - User can trigger a scene recall that enables scene nodes, disables non-scene nodes, and applies macro overrides.
    - User can save a variation that snapshots all current parameter values for nodes in a given scene.
    - User can restore a variation that applies stored parameter overrides to the current session.
    - All performance commands validate against the canonical session and use the same clone-and-replace mutation pattern as graph edits.
    - TypeScript contracts include the PerformanceCommand type.
  artifacts:
    - path: src-tauri/src/application/performance_command.rs
      provides: recall_scene, save_variation, restore_variation command handlers
      contains: "PerformanceCommand"
    - path: src-tauri/tests/performance_commands.rs
      provides: integration tests for scene recall, variation save, variation restore
      contains: "recall_scene"
  tasks:
    - id: 1
      title: Add PerformanceCommand enum to domain
      action: implement
      file: src-tauri/src/domain/session.rs
      description: Add a PerformanceCommand tagged enum with RecallScene, SaveVariation, and RestoreVariation variants. Register it for TypeScript contract generation.
    - id: 2
      title: Implement performance command handlers
      action: implement
      file: src-tauri/src/application/performance_command.rs
      description: Create apply_performance_command with recall_scene (toggle node enabled state, apply macro overrides), save_variation (snapshot parameters as VariationDefinition), and restore_variation (apply ParameterOverride values). Use store.mutate_current for transactional safety. Add a PerformanceCommandError enum with validation errors.
    - id: 3
      title: Register application module and Tauri IPC handler
      action: implement
      file: src-tauri/src/application/mod.rs, src-tauri/src/lib.rs
      description: Add pub mod performance_command. Add apply_performance_command Tauri command that delegates to the handler. Register in invoke_handler.
    - id: 4
      title: Write integration tests
      action: implement
      file: src-tauri/tests/performance_commands.rs
      description: Test scene recall enables correct nodes and disables others. Test scene recall applies macro overrides. Test save variation snapshots parameters. Test restore variation applies overrides. Test error cases (missing scene, missing variation, out-of-range override values).
---
