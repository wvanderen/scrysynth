---
phase: 01-session-core-recall
plan: 06
subsystem: infra
tags: [rust, build-rs, ts-rs, contract-generation]

requires:
  - phase: 01-session-core-recall
    provides: write_generated_typescript_contract function and session schema

provides:
  - Contract generation errors cause visible app failure with diagnostic
  - Build re-triggers when session.rs changes

affects: [build-pipeline, type-safety]

tech-stack:
  added: []
  patterns: [fail-loud-contracts, build-time-schema-tracking]

key-files:
  created: []
  modified:
    - src-tauri/build.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "if let Err + process::exit(1) over .expect() for clearer diagnostic at app startup"
  - "build.rs rerun-if-changed tracks session.rs to catch schema drift at compile time"

patterns-established:
  - "Contract generation failures must be visible — no silent let _ = discards"
  - "Build scripts track domain schema files for recompilation triggers"

requirements-completed: [SESS-01]

duration: 1min
completed: 2026-04-14
---

# Phase 01 Plan 06: Contract Generation Fail Loudly + Build Guard Summary

**Contract generation errors now exit with diagnostics; build.rs re-triggers on session schema changes**

## Performance

- **Duration:** ~1 min
- **Started:** 2026-04-14T01:41:02Z
- **Completed:** 2026-04-14T01:42:00Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- Replaced `let _ =` silent error discard with `if let Err` + diagnostic message + `process::exit(1)`
- Added `cargo:rerun-if-changed=src/domain/session.rs` to build.rs for schema-aware recompilation
- All 11 Rust tests pass

## Task Commits

1. **Task 1: Make contract generation fail loudly and add build guard** - `84ddaed` (fix)

## Files Created/Modified
- `src-tauri/build.rs` - Added rerun-if-changed directive for session.rs
- `src-tauri/src/lib.rs` - Error propagation replacing silent discard

## Decisions Made
- Used `if let Err` + `process::exit(1)` instead of `.expect()` for more actionable error message (expect panics with less helpful backtrace)
- Build.rs tracks session.rs (the schema file) rather than the generated output — recompilation is the trigger, not the result

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Contract generation is now fail-safe and build-aware
- Phase 01 gap closure complete

---
*Phase: 01-session-core-recall*
*Completed: 2026-04-14*
