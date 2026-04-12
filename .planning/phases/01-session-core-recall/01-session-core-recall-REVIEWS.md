---
phase: 1
reviewers: [gemini, claude, codex]
reviewed_at: 2026-04-11T18:03:52-05:00
plans_reviewed:
  - 01-session-core-recall-01-PLAN.md
  - 01-session-core-recall-02-PLAN.md
  - 01-session-core-recall-03-PLAN.md
successful_reviewers: [gemini, claude]
failed_reviewers: [codex]
---

# Cross-AI Plan Review - Phase 1

## Gemini Review

# Phase 1 Plan Review: Session Core & Recall

## Summary
The implementation plans for Phase 1 are exceptionally well-structured, prioritizing architectural integrity and a "source of truth" strategy that guards against the most common pitfalls of hybrid Tauri/React applications. By establishing the canonical session model in Rust and projecting it to a read-only frontend via typed IPC and `ts-rs`, the plans ensure that the core instrument logic remains engine-independent and persistent. The focus on a "read-first" workspace is a pragmatic choice that secures the "trust chain" (semantics -> recall -> inspection) before introducing the volatility of live mutation.

## Strengths
- **Single Source of Truth**: Defining the `SessionDocument` in Rust ensures that persistence and internal logic never drift from the frontend's view.
- **Type Safety**: The use of `ts-rs` for contract generation combined with `zod` for runtime validation at the IPC boundary creates a robust defense-in-depth against data corruption.
- **Atomic Persistence**: Implementing save-via-rename (temporary files) prevents session corruption during power loss or app crashes.
- **Future-Proof Schema**: Including `ownershipRules` and `runtimeStatus` placeholders in Phase 1 prevents costly database/schema migrations in later phases.
- **Seeded Defaults**: Providing a meaningful sample graph in the default session allows for immediate validation of the graph viewport and inspector without needing a full editor.

## Concerns
- **UI Interaction Patterns (LOW)**: The use of `window.prompt` for file paths in Plan 03 is a "web-first" pattern that feels out of place in a desktop instrument. While functional for Phase 1, it lacks the native safety (e.g., file extensions, overwrite warnings) of the Tauri Dialog plugin.
- **Error Granularity (LOW)**: Commands return `Result<T, String>`. While sufficient for Phase 1, the loss of typed error variants (e.g., `IoError` vs. `ValidationError` vs. `VersionMismatch`) may make providing helpful recovery hints to the user difficult as the system grows.
- **TS Export Fragility (LOW)**: The `write_generated_typescript_contract` function uses manual string replacement (`.replace("\ntype ", "\nexport type ")`). This is a common shortcut but can be brittle if `ts-rs` output formatting changes in future versions.

## Suggestions
- **Native File Dialogs**: Replace `window.prompt` with `@tauri-apps/plugin-dialog` to provide a standard desktop experience for saving and opening sessions.
- **Explicit Schema Validation**: In `open_session_from_path`, consider implementing a "migration" or "compatibility" check more robust than a simple equality check on `schemaVersion` to handle minor version drifts without total rejection.
- **Automated TS Generation**: Ensure `write_generated_typescript_contract` is hooked into the build script (or a pre-commit hook) so that developers cannot accidentally commit Rust changes without updating the TypeScript definitions.

## Risk Assessment
**Overall Risk: LOW**

The phase is technically conservative and architecturally sound. The primary risk is **"Semantics Over-Engineering"**-creating a session model so complex that it becomes difficult for the agent (Phase 4) to manipulate. However, the plan's focus on "stable primitives" in the research findings suggests the team is already aware of this. The decision to defer deep editing to Phase 2 further mitigates implementation risk for this milestone.

---

## Claude Review

---

# Cross-AI Plan Review: Phase 1 - Session Core & Recall

## Plan 01: Canonical Session Schema & Store

### Summary

Plan 01 establishes the foundational session schema and Rust-managed store with a clear TDD approach. The plan is well-scoped, correctly identifies that Rust should own canonical truth, and produces both backend state management and generated TypeScript contracts in two focused tasks. The execution matched the plan closely and the resulting code is clean.

### Strengths

- **Correct dependency ordering**: Schema first (Task 1), then store + commands (Task 2). No forward references.
- **TDD throughout**: Both tasks specify test behavior before implementation actions. Tests cover round-trip serialization and field presence.
- **UUID-backed string IDs**: Avoids coupling to SuperCollider bus indices or any runtime-specific identifiers - directly addresses research pitfall #1.
- **Ownership/runtime fields reserved now**: Schema includes `OwnershipRule`, `RuntimeStatusRef`, `OwnershipAssignment` even in Phase 1, preventing backfill churn.
- **Generated TS contracts**: `ts-rs` export from the same Rust structs prevents schema drift.
- **Seeded default graph**: The store ships with meaningful sample data (source node, master output, route, bus, macro, scene, variation) so the UI immediately has something to render.
- **Clean module separation**: `domain/` for schema, `application/` for store, `persistence/` deferred to Plan 02.

### Concerns

- **MEDIUM - `Mutex<SessionStore>` contention**: The store uses `std::sync::Mutex` which will block the Tauri command thread during all operations. Adequate for Phase 1 but will need `tokio::sync::RwLock` or similar once async runtime supervision is added.
- **LOW - `write_generated_typescript_contract()` called in `run()`**: If the write fails, the error is silently discarded (`let _ = ...`). This is fine for Phase 1 but should be logged or handled once `tracing` is wired.
- **LOW - `created_at`/`updated_at` are static strings**: `SessionDocument::default()` uses `"2026-04-11T00:00:00Z"` as a fixed timestamp. Not a bug, but worth noting that timestamps are not yet generated from system time.
- **LOW - No `Eq`/`Hash` on enums**: `NodeType`, `PortDirection`, etc. lack `Eq` derives. Not needed now but may be useful for graph lookups in Phase 2.

### Suggestions

- Consider adding `#[serde(deny_unknown_fields)]` to `SessionDocument` to future-proof deserialization against schema evolution.
- The `new_id()` helper could return a newtype (`SessionId`) rather than bare `String` to prevent ID confusion later, though this is arguably a Phase 2+ concern.

### Risk Assessment: **LOW**

The plan is well-bounded, TDD-protected, and the implementation matches. The Mutex concern is real but not blocking for Phase 1 scope.

---

## Plan 02: Versioned JSON Persistence

### Summary

Plan 02 adds file-based save/open with version checking, atomic writes, and comprehensive round-trip testing. The persistence layer is correctly isolated from the store and commands, and the test suite covers success, corrupt input, and version mismatch. This is a solid, focused plan that achieves PERS-01 cleanly.

### Strengths

- **Atomic writes**: Uses temp file + rename pattern to prevent half-written corrupt files on crash.
- **Version gate**: Rejects unsupported `schemaVersion` before touching the store - prevents silent data corruption.
- **Typed errors**: `SessionFileError` enum with `thiserror` gives structured error handling, not string soup.
- **Thorough test suite** (5 tests): Round-trip preservation, corrupt JSON rejection, unsupported version rejection, open-failure isolation, and store replacement verification.
- **Clean separation**: `session_file.rs` is a pure function module - no global state, no Tauri dependency. Easy to test independently.
- **Store guard**: `open_session_from_path` returns a `SessionDocument` without mutating the store; the Tauri command layer handles `replace_current`, keeping the persistence module stateless.

### Concerns

- **MEDIUM - Atomic write assumes same filesystem**: `fs::rename` across filesystems fails. If a user specifies a path on a different mount from the temp file location, the rename will fail. For Phase 1 this is unlikely but worth noting.
- **LOW - No file permissions or content-size guard**: A maliciously large file would be fully read into memory before validation. Low risk for local single-user app.
- **LOW - Version check is exact-match only**: No migration path exists yet. When `schemaVersion` bumps to 2, opening v1 files will fail until a migration layer is added. This is explicitly deferred, which is correct.
- **LOW - `save_session_to_path` overwrites without confirmation**: The Tauri command delegates path choice to `window.prompt`, but the backend itself doesn't guard against overwriting existing files. Acceptable for Phase 1.

### Suggestions

- The `temporary_path` function places the temp file in the same directory as the target, which correctly handles the same-filesystem rename constraint. This is good.
- Consider adding a `#[serde(tag = "schemaVersion")]` or similar discriminated union pattern in future if multiple schema versions need to coexist.

### Risk Assessment: **LOW**

Clean, well-tested persistence layer. The atomic write concern is real but handled correctly by placing temp files alongside targets.

---

## Plan 03: Session Workspace UI

### Summary

Plan 03 replaces the starter Tauri template with a purpose-built session workspace including graph viewport, node inspector, toolbar, and runtime status strip. The plan correctly specifies a read-first approach (no drag editing) and includes a human verification checkpoint. The resulting implementation is clean and productively scoped.

### Strengths

- **Read-first constraint respected**: `nodesDraggable={false}`, `nodesConnectable={false}` in `ReactFlow`. This is exactly right for Phase 1 - prove inspection before building editing.
- **Zod validation at the boundary**: `session-client.ts` validates every backend payload with a full Zod schema matching the Rust contract before committing to the store. Catches Rust/TS drift immediately.
- **Single mirror store pattern**: One Zustand store derives `graphNodes`, `graphEdges`, and `selectedNode` from the canonical session. No duplicate state.
- **Deliberate visual identity**: CSS uses warm dark palette (amber/gold accents on deep green-black), not Tauri/Vite starter defaults. The product looks like an instrument, not a template.
- **Human checkpoint task**: Task 3 is a blocking human-verify gate with precise verification steps. This is the right call for the first UI integration.
- **Inspector covers all SESS-03 fields**: Identity (id, type, runtime target, scene membership), ownership (controller, locked), ports (name, direction, signal type), parameters (name, value, unit).
- **Runtime status strip**: Reads from canonical `runtimeStatus` and renders disconnected placeholders - proves the architecture without requiring actual runtimes.

### Concerns

- **MEDIUM - Node positioning is index-based**: `projectGraphNodes` lays out nodes in a grid by array index (`80 + (index % 3) * 260`). Once the graph grows beyond 3-6 nodes, this will produce confusing layouts. Acceptable for Phase 1's 2-node default graph, but the research doc recommended `elkjs` for auto-layout, which is not wired up yet.
- **MEDIUM - `deriveSelectedNode` falls back to first node**: When `selectedNodeId` is null or doesn't match, it defaults to `session.nodes[0]`. This means the inspector always shows *something*, but it may confuse users who click the pane to deselect - the inspector won't actually clear. The empty-state path in `NodeInspector` is effectively unreachable.
- **LOW - No frontend tests**: Plans 01 and 02 use TDD; Plan 03 only has `npm run build` as automated verification. The store projection logic (`projectGraphNodes`, `projectGraphEdges`, `deriveSelectedNode`) is testable in isolation. Not blocking, but a gap.
- **LOW - `window.prompt` for file paths**: Save/open uses browser prompts for file paths. The research doc explicitly called this out as "simple with prompt-driven file paths so Phase 1 proves backend recall without adding native dialog complexity yet." Acceptable as stated.
- **LOW - No loading/error UI for graph viewport**: If the session fails to load, only the error banner renders - the graph panel shows empty. Minor UX gap.

### Suggestions

- The `deriveSelectedNode` fallback should be removed or made explicit: clicking the pane background should truly deselect, showing the "Select a node" empty state. This makes the inspector's empty state reachable and matches user expectation.
- Consider adding `immer` middleware to the Zustand store for Phase 2 mutations, since the plan adds it as a dependency but the current store uses spread operators throughout. The `immer` import in `package.json` is unused.
- Wire `elkjs` (already in the tech stack recommendations) for layout in Phase 2 when node counts grow.

### Risk Assessment: **LOW**

The UI plan is conservatively scoped, correctly read-first, and includes a human verification gate. The selection fallback quirk is the most notable UX issue but doesn't block any success criteria.

---

## Cross-Phase Assessment

### Dependency Ordering

The wave structure (1 -> 2 -> 3) is correct:
- Plan 01 produces the schema and store that Plans 02 and 03 depend on.
- Plan 02 produces the persistence layer that Plan 03's save/open actions call.
- Plan 03 is the integration layer that proves everything works together.

### Requirement Coverage

| Requirement | Plan Coverage | Status |
|-------------|---------------|--------|
| SESS-01 (canonical session collections) | Plan 01 Task 1 + Task 2 | Fully covered |
| SESS-02 (visible graph nodes/connections) | Plan 03 Task 2 | Fully covered |
| SESS-03 (node inspection metadata) | Plan 03 Task 2 | Fully covered |
| PERS-01 (save/reload with all collections) | Plan 02 + Plan 03 Task 2 | Fully covered |

All four Phase 1 requirements are addressed with test or verification coverage.

### Architecture Concerns Across Plans

- **The `immer` dependency is declared but unused**: `package.json` lists `immer` but `sessionStore.ts` uses spread operators exclusively. This is dead weight in the bundle until Phase 2 adds mutation logic.
- **No `ts-rs` casing configuration for TS output**: The Rust types use `#[serde(rename_all = "camelCase")]` which matches the TS output, but this alignment is implicit. If someone adds a new struct without the serde annotation, the TS types will silently diverge from what the frontend expects.

### Overall Phase 1 Risk: **LOW**

These plans are well-structured, correctly sequenced, and conservatively scoped. They achieve all four Phase 1 success criteria with automated tests on the backend and a human verification gate on the UI. The main items to address going forward are: selection semantics in the inspector, `elkjs` layout for growing graphs, and frontend unit tests for the store projection layer.

---

## Codex Review

Codex CLI did not complete a review in this run.

```text
Reading additional input from stdin...
Failed to refresh token: 401 Unauthorized
Provided authentication token is expired. Please try signing in again.
```

---

## Consensus Summary

The successful reviewers agree Phase 1 is well-scoped, correctly ordered, and low risk because it establishes Rust-owned canonical session state first, proves persistence with explicit tests, and keeps the UI read-first rather than prematurely expanding into editing. Both reviews see the phase as strong on architecture and requirement coverage, with the main follow-up work centered on polish and future-proofing rather than foundational redesign.

### Agreed Strengths

- Rust is treated as the canonical source of truth, with generated TypeScript contracts and validation at the UI boundary.
- The dependency ordering across schema, persistence, and UI is sound and matches the phase trust chain.
- Persistence is handled conservatively with explicit schema/version thinking and strong automated test coverage.
- The Phase 1 workspace stays read-first, which protects scope and proves inspectability before live mutation work.

### Agreed Concerns

- Prompt-based save/open paths are acceptable for Phase 1 but feel mismatched to a desktop instrument and should move to native file dialogs soon.
- Schema/version handling is intentionally strict now, but migration and richer compatibility handling will be needed once the schema evolves.
- Generated TypeScript contract export needs a more durable build-time workflow so schema drift or silent write failures do not slip through.

### Divergent Views

- Claude focused more on implementation-shaped follow-ups such as `Mutex<SessionStore>` scalability, graph selection semantics, index-based layout, and missing frontend tests; Gemini did not flag those as notable risks.
- Gemini emphasized error-surface quality and TS export fragility at a higher level, while Claude judged those items as low-risk compared with UI behavior and future concurrency concerns.
