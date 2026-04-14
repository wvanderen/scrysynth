---
phase: 01-session-core-recall
plan: 05
subsystem: ui
tags: [react, tauri, dialog, projections, zustand]

requires:
  - phase: 01-session-core-recall
    provides: session workspace with graph viewport and node inspector

provides:
  - deriveSelectedNode returns null for no selection (inspector shows empty state)
  - Native OS file dialogs for save/open via tauri-plugin-dialog

affects: [session-projections, App.tsx]

tech-stack:
  added: ["@tauri-apps/plugin-dialog", "tauri-plugin-dialog"]
  patterns: [native-dialog-for-file-ops, null-selection-semantics]

key-files:
  created: []
  modified:
    - src/store/session-projections.ts
    - src/store/session-projections.test.ts
    - src/App.tsx
    - package.json
    - src-tauri/Cargo.toml
    - src-tauri/src/lib.rs
    - src-tauri/capabilities/default.json

key-decisions:
  - "deriveSelectedNode returns null (not first node) for null/unfound selection — inspector empty state is the correct UX"
  - "openDialog returns string (not object) when multiple: false — simplified path handling"

patterns-established:
  - "Native Tauri dialogs for all file operations — no browser prompts in desktop app"
  - "Selection null semantics: no selection means null, not implicit first-node"

requirements-completed: [SESS-02, SESS-03, PERS-01]

duration: 3min
completed: 2026-04-14
---

# Phase 01 Plan 05: Fix Selection Clearing + Native File Dialogs Summary

**Selection returns null for empty state; save/open use OS-native file dialogs via tauri-plugin-dialog**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-04-14T01:36:39Z
- **Completed:** 2026-04-14T01:41:02Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Fixed deriveSelectedNode to return null when nothing is selected — inspector now shows empty state
- Added test for null/not-found selection behavior (10 tests all pass)
- Replaced browser window.prompt with native Tauri file dialogs for save and open
- Added tauri-plugin-dialog to both frontend and Rust backend

## Task Commits

1. **Task 1: Fix deriveSelectedNode to return null** - `9629168` (fix)
2. **Task 2: Replace window.prompt with native Tauri file dialogs** - `2bbb695` (feat)

## Files Created/Modified
- `src/store/session-projections.ts` - deriveSelectedNode returns null for no selection
- `src/store/session-projections.test.ts` - Added null selection test
- `src/App.tsx` - Native dialog handlers replacing window.prompt
- `package.json` - Added @tauri-apps/plugin-dialog
- `src-tauri/Cargo.toml` - Added tauri-plugin-dialog dependency
- `src-tauri/src/lib.rs` - Registered dialog plugin
- `src-tauri/capabilities/default.json` - Added dialog:default permission

## Decisions Made
- deriveSelectedNode returns null (not first node) for null/unfound selection — inspector empty state is the correct UX for "nothing selected"
- openDialog returns `string | null` when `multiple: false` in Tauri 2 — simplified from plan's `typeof result === "string"` check

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Simplified openDialog result handling**
- **Found during:** Task 2 (native file dialogs)
- **Issue:** Plan assumed openDialog returns an object with `.path` property, but Tauri 2 plugin-dialog returns `string | null` directly when `multiple: false`
- **Fix:** Removed unnecessary `typeof result === "string" ? result : result.path` — just use the string directly
- **Files modified:** src/App.tsx
- **Verification:** TypeScript compiles cleanly, no LSP errors

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minimal — simplified code is more correct for actual Tauri 2 API.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Session workspace now has proper selection semantics and native file dialogs
- Ready for contract generation hardening (Plan 06)

---
*Phase: 01-session-core-recall*
*Completed: 2026-04-14*
