# Phase 1 Research: Session Core & Recall

**Phase:** 1 - Session Core & Recall
**Researched:** 2026-04-11
**Status:** Ready for planning

## Research Answer

Phase 1 should establish the app-owned session model and a minimal read-first workspace, not a deep editor. The safest implementation path is a Rust-owned canonical `SessionDocument` with typed Tauri commands for session lifecycle, versioned JSON snapshot persistence for save/open, and a React workspace that mirrors the session via typed IPC while treating runtimes as absent or disconnected placeholders.

## Prescriptive Decisions

1. Use Rust as the source of truth for the canonical session schema. Export frontend-facing types from Rust with `ts-rs`; validate file imports and IPC payloads in the UI with `zod`.
2. Keep Phase 1 persistence file-based and explicit: versioned JSON session documents with atomic write + read/replace. Defer SQLite catalogs/history until later phases.
3. Build a read-first UI shell for this phase: graph viewport, inspector, and session actions. Do not spend the phase on unconstrained drag-editing.
4. Include ownership and runtime status references in the schema now, even if runtime adapters are still placeholder-only. This prevents later backfill churn.
5. Route every mutation through Tauri commands into a Rust session store. Do not let the frontend invent canonical entities locally and sync them later.

## Recommended Phase Shape

### Canonical Session Contract

Define a persisted `SessionDocument` with:

- `schemaVersion`
- `sessionId`, `title`, `createdAt`, `updatedAt`
- `transport`
- `nodes`, `routes`, `buses`, `macros`, `scenes`, `variations`
- `ownershipRules`
- `runtimeStatus`

Each node should already carry the inspection fields required by `SESS-03`: identity, type, ports, parameters, runtime target, scene membership, and ownership metadata.

### Session Service Boundary

Expose typed Tauri commands for:

- `create_default_session`
- `get_current_session`
- `save_session_to_path`
- `open_session_from_path`

Back them with one Rust session store guarded by Tauri managed state. The UI should only read or replace the current session through these commands.

### Save / Open Strategy

For this phase, save/open should use versioned JSON session documents on disk:

- save: serialize canonical session, pretty-print JSON, write atomically
- open: parse JSON, validate version, replace current in-memory session
- tests: round-trip equality, fixture load, corrupt-file rejection

This satisfies `PERS-01` without prematurely expanding into database indexing or history subsystems.

### UI Strategy

Replace the starter Tauri screen with a workspace that proves the canonical session is inspectable:

- top bar: new session, save, open, current session title
- center: graph viewport from `@xyflow/react` in read-first mode
- right panel: selected node inspector
- status strip: runtime placeholders showing disconnected/not-started state

The graph should render from canonical state, not hardcoded demo nodes.

## Libraries To Add

### Rust

- `ts-rs` for frontend type export
- `uuid` for stable app-native IDs
- `thiserror` or equivalent typed error crate for command/file failures
- `tempfile` in tests for atomic-write and round-trip coverage

### Frontend

- `@xyflow/react` for graph rendering
- `zustand` + `immer` for UI projection state
- `zod` for runtime validation of session payloads/files

## Pitfalls To Avoid In This Phase

1. Do not store SuperCollider IDs, bus indices, or visual-engine handles as canonical node identifiers.
2. Do not let the starter React app create the session shape locally and treat Rust as persistence-only.
3. Do not hide ownership metadata or runtime status until later; the schema should reserve those fields now.
4. Do not build freeform patch editing before the app can cold-load the same session reliably.
5. Do not make save/open depend on a running runtime engine.

## Verification Focus

- `cargo test` proves schema round-trip and persistence replacement
- saved JSON contains all canonical collections required by `SESS-01`
- `npm run build` proves the UI consumes typed session data
- the app can start, show a graph from canonical state, inspect a node, save to disk, and reopen the same session

## Planning Implications

- Plan 1 should define Rust session contracts and session-store commands.
- Plan 2 should implement file persistence and restore behavior.
- Plan 3 should replace the starter UI with the read-first session workspace.
