---
phase: 05-visual-sync-cross-modal
plan: 02
subsystem: macros
tags: [rust, typescript, serde, zustand, tauri, zod, cross-domain, macros]

requires:
  - phase: 05-visual-sync-cross-modal-01
    provides: Visual runtime types, RuntimeHealthPanel
provides:
  - MacroTarget enum (AudioParameter, VisualParameter) for cross-domain targeting
  - Expanded MacroDefinition with targets field and backward compat
  - MacroCommand CRUD + live value with Tauri IPC
  - MacroEditor component for creating/editing/deleting macros
  - MacroSlider component for live performance control
  - Integration tests for macro lifecycle
affects: [05-visual-sync-cross-modal-03]

tech-stack:
  added: []
  patterns:
    - "MacroTarget tagged enum for cross-domain parameter addressing"
    - "serde(default) on targets field for backward compatibility with old target_parameter_ids"
    - "Debounced slider (300ms) for performance macro value changes"

key-files:
  created:
    - src-tauri/src/application/macro_command.rs
    - src/components/workspace/MacroEditor.tsx
    - src/components/workspace/MacroSlider.tsx
    - src-tauri/tests/macro_commands.rs
  modified:
    - src-tauri/src/domain/session.rs
    - src-tauri/src/application/performance_command.rs
    - src-tauri/src/lib.rs
    - src/lib/session-client.ts
    - src/store/sessionStore.ts
    - src/store/session-projections.ts
    - src/components/workspace/PerformanceView.tsx

key-decisions:
  - "MacroTarget uses tagged serde enum (kind+config) matching AudioPrimitive pattern"
  - "Backward compat: targets defaults to empty vec; if empty, falls back to target_parameter_ids"
  - "Visual parameter targeting records value but actual visual update deferred to scene compile in v1"

patterns-established:
  - "MacroTarget discriminated union for cross-domain parameter addressing"
  - "MacroCommand CRUD pattern mirroring PerformanceCommand structure"

requirements-completed: [CTRL-01]

duration: 14min
completed: 2026-04-12
---

# Phase 5 Plan 02: Cross-Domain Macro System Summary

**Cross-domain macro system with MacroTarget enum targeting audio and visual parameters, CRUD commands, live value sliders, and backward-compatible migration from flat target_parameter_ids**

## Performance

- **Duration:** 14 min
- **Started:** 2026-04-12T22:41:49Z
- **Completed:** 2026-04-12T22:55:45Z
- **Tasks:** 3
- **Files modified:** 14

## Accomplishments
- MacroTarget enum supports AudioParameter (node_id + parameter_id) and VisualParameter (element_id + parameter_id) addressing
- MacroCommand provides Create/Update/Remove/SetMacroValue with full CRUD lifecycle
- Scene recall uses MacroTarget-aware resolution with backward compat for old target_parameter_ids
- MacroEditor UI allows creating, editing, and deleting macros with audio/visual target selectors
- MacroSlider provides debounced live performance control (0.0-1.0 range)
- 11 integration tests cover CRUD, cross-domain targeting, backward compatibility, scaling, and scene recall

## Task Commits

Each task was committed atomically:

1. **Task 1: Add MacroTarget enum, expand MacroDefinition, create macro_command.rs** - `ca93e85` (feat)
2. **Task 2: Wire macro IPC to frontend, add Zustand actions, create MacroEditor and MacroSlider** - `2d57dc1` (feat)
3. **Task 3: Write integration tests** - `034a5ea` (test)

## Files Created/Modified
- `src-tauri/src/domain/session.rs` - Added MacroTarget enum, expanded MacroDefinition with targets field, added MacroCommand enum, registered in TS contract generator
- `src-tauri/src/application/macro_command.rs` - CRUD + SetMacroValue with backward compat, MacroCommandError
- `src-tauri/src/application/performance_command.rs` - Updated apply_macro_override to use MacroTarget-aware resolution
- `src-tauri/src/application/mod.rs` - Registered macro_command module
- `src-tauri/src/application/session_store.rs` - Added targets field to default MacroDefinition
- `src-tauri/src/lib.rs` - Added apply_macro_command Tauri command
- `src/lib/session-client.ts` - Added MacroTarget/MacroCommand zod schemas and applyMacroCommand IPC
- `src/store/sessionStore.ts` - Added createMacro/updateMacro/removeMacro/setMacroValue actions
- `src/store/session-projections.ts` - Added MacroProjection type and projectMacros function
- `src/components/workspace/MacroEditor.tsx` - Macro CRUD editor with audio/visual target selectors
- `src/components/workspace/MacroSlider.tsx` - Live performance slider with 300ms debounce
- `src/components/workspace/PerformanceView.tsx` - Added macro sliders and editor sections
- `src/App.tsx` - Passes macro-related props to PerformanceView
- `src-tauri/tests/macro_commands.rs` - 11 integration tests

## Decisions Made
- MacroTarget uses serde(tag="kind", content="config") matching the established AudioPrimitive pattern
- Backward compat uses serde(default) on targets; old sessions with target_parameter_ids but empty targets still work via fallback path
- Visual parameter targeting records value in session but actual visual runtime update deferred to scene compile (v1 limitation)
- setMacroValue does NOT set isLoading (non-blocking for performance responsiveness)
- MacroSlider uses 300ms debounce to avoid excessive IPC during rapid slider movement

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Initial multi-target test failed because range_start=0/range_end=1 with parameter min=20/max=20000 caused clamp. Fixed by using range_start=20/range_end=20000 matching the parameter bounds.

## Next Phase Readiness
- Macro system fully operational for cross-domain parameter control
- Ready for Plan 03 which should add visual scene compilation integration
- The visual parameter targeting currently records but doesn't dispatch — next plan should wire the visual adapter

---
*Phase: 05-visual-sync-cross-modal*
*Completed: 2026-04-12*
