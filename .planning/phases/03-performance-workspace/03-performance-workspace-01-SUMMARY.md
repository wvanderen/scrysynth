## Plan 01 Summary: Scene and Variation Backend Commands

**Completed:** 2026-04-11

### What was done
- Added `PerformanceCommand` enum to domain with `RecallScene`, `SaveVariation`, `RestoreVariation` variants
- Implemented `apply_performance_command` in `application/performance_command.rs`:
  - `recall_scene`: enables scene nodes, disables non-scene nodes, applies macro overrides with scaled values
  - `save_variation`: snapshots all parameters for a scene's active nodes as a new `VariationDefinition`
  - `restore_variation`: applies stored parameter overrides with range validation
- Added Tauri IPC handler `apply_performance_command`
- Registered `PerformanceCommand` for TypeScript contract generation
- All commands use `store.mutate_current()` clone-and-replace for transactional safety

### Tests
- 8 unit tests in `performance_command.rs` module
- 8 integration tests in `tests/performance_commands.rs`
- Tests cover: scene recall, macro override application, variation save, variation restore, error rejection, full recall-save-restore cycle

### Files modified
- `src-tauri/src/domain/session.rs` — PerformanceCommand enum + TS export
- `src-tauri/src/application/mod.rs` — module registration
- `src-tauri/src/application/performance_command.rs` — new file, command handlers
- `src-tauri/src/lib.rs` — Tauri IPC handler + registration
- `src-tauri/tests/performance_commands.rs` — new file, integration tests
