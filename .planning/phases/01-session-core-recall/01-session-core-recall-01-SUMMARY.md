---
phase: 01-session-core-recall
plan: 01
subsystem: api
tags: [tauri, rust, ts-rs, serde, uuid, session]
requires: []
provides:
  - Canonical Rust SessionDocument schema with ownership and runtime status metadata
  - In-memory SessionStore managed through typed Tauri commands
  - Generated frontend TypeScript session contracts from Rust definitions
affects: [phase-2-audio-graph, phase-3-workspace, frontend-session-ui]
tech-stack:
  added: [ts-rs, uuid]
  patterns: [rust-owned canonical session schema, tauri managed session store, generated frontend contracts]
key-files:
  created:
    - src-tauri/src/domain/mod.rs
    - src-tauri/src/domain/session.rs
    - src-tauri/src/application/mod.rs
    - src-tauri/src/application/session_store.rs
    - src/generated/session-types.ts
    - src-tauri/Cargo.lock
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/lib.rs
key-decisions:
  - "Kept the canonical session schema in Rust and exported a single self-contained TypeScript contract file from the same definitions."
  - "Seeded the managed session store with a meaningful default graph so later UI work can render canonical data immediately."
patterns-established:
  - "Rust domain types derive serde and TS together so persisted and frontend contracts stay aligned."
  - "Tauri commands read from SessionStore state instead of querying runtimes directly."
requirements-completed: [SESS-01]
duration: 6min
completed: 2026-04-11
---

# Phase 1 Plan 1: Session Core Contracts Summary

**Canonical Rust session contracts, managed in-memory session state, and shared frontend bindings for Scrysynth's app-owned source of truth.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-04-11T22:35:02Z
- **Completed:** 2026-04-11T22:40:56Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Added the canonical `SessionDocument` schema with nodes, routes, buses, macros, scenes, variations, ownership rules, and runtime status references.
- Added schema and session-store Rust tests covering round-trip serialization, required node fields, seeded default session content, and in-memory replacement semantics.
- Replaced the starter `greet` command with managed `create_default_session` and `get_current_session` Tauri commands backed by `SessionStore`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define canonical session schema and export frontend contracts** - `3f4ee27` (feat)
2. **Task 2: Replace greet with a Rust session store and typed session commands** - `5240648` (feat)

**Plan metadata:** pending

## Files Created/Modified
- `src-tauri/src/domain/session.rs` - Defines the canonical session graph types, TS export helper, and schema tests.
- `src/generated/session-types.ts` - Provides the frontend-facing session contract generated from Rust type declarations.
- `src-tauri/src/application/session_store.rs` - Seeds and serves the in-memory canonical session.
- `src-tauri/src/lib.rs` - Registers managed session state and typed Tauri commands.
- `src-tauri/Cargo.toml` - Adds `ts-rs` and `uuid` for contract generation and canonical IDs.
- `src-tauri/Cargo.lock` - Records the resolved Rust dependency graph for the new contract-layer crates.

## Decisions Made
- Used Rust domain types as the single schema source and exported a single checked-in TypeScript file so the UI can consume the same contract without handwritten drift.
- Seeded the default session with a small real graph instead of an empty shell so the Phase 1 workspace can render and inspect canonical data immediately.
- Managed the session through `tauri::State<Mutex<SessionStore>>` to keep runtime adapters out of the source-of-truth path.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Reworked TS export generation to avoid broken cross-file imports**
- **Found during:** Task 1 (Define canonical session schema and export frontend contracts)
- **Issue:** `ts-rs` manual export output initially produced imports pointing at per-type files that were not being generated, which would leave the checked-in `session-types.ts` contract unusable in the frontend.
- **Fix:** Switched the export helper to aggregate `TS::decl(&Config::default())` output into one self-contained `src/generated/session-types.ts` file.
- **Files modified:** `src-tauri/src/domain/session.rs`, `src/generated/session-types.ts`
- **Verification:** `cargo test session_document --manifest-path src-tauri/Cargo.toml`
- **Committed in:** `3f4ee27` (part of Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The auto-fix kept the generated frontend contract usable without changing scope.

## Issues Encountered
- `ts-rs` export helpers needed a self-contained generation path for this repo layout; resolved by generating one aggregated contract file from the Rust declarations.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 1 persistence work can now serialize and replace one canonical `SessionDocument` instead of inventing a separate save format.
- The frontend workspace plan can call typed session commands and consume `src/generated/session-types.ts` directly.

## Self-Check
PASSED
- Found `.planning/phases/01-session-core-recall/01-session-core-recall-01-SUMMARY.md`
- Found commit `3f4ee27`
- Found commit `5240648`

---
*Phase: 01-session-core-recall*
*Completed: 2026-04-11*
