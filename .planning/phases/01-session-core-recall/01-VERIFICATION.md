---
phase: 01-session-core-recall
verified: 2026-04-14T01:58:00Z
status: passed
score: 11/11 must-haves verified
gaps:
  - truth: "User can inspect a selected node's metadata and can also clear the selection"
    status: resolved
    reason: "Fixed by Plan 05: deriveSelectedNode now returns null for null/unfound selection"
    resolved_by: "01-session-core-recall-05"
  - truth: "User can trigger new, save, and open actions from the workspace through native desktop dialogs rather than browser prompts"
    status: resolved
    reason: "Fixed by Plan 05: tauri-plugin-dialog integrated, window.prompt replaced"
    resolved_by: "01-session-core-recall-05"
  - truth: "Contract generation fails loudly if it drifts (build-time guard)"
    status: resolved
    reason: "Fixed by Plan 06: if let Err + process::exit(1) replaces let _ =; build.rs has rerun-if-changed"
    resolved_by: "01-session-core-recall-06"
---

# Phase 1: Session Core & Recall Verification Report

**Phase Goal:** Users can create, inspect, save, and reopen a canonical Scrysynth session without runtime engines becoming the source of truth.
**Verified:** 2026-04-13T20:12:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can start from an app-owned canonical session before any runtime exists | ✓ VERIFIED | `SessionStore::new_default()` creates in-memory session with seeded nodes/routes/buses/macros/scenes/variations/ownership/runtime status. No runtime queried. Tests pass. |
| 2 | The current session includes nodes, routes, buses, macros, scenes, variations, ownership rules, and runtime status references | ✓ VERIFIED | `SessionDocument` struct (session.rs:13-42) has all required fields. `session-types.ts` exports all types. Schema round-trip test passes. |
| 3 | Frontend code consumes the same session contract Rust persists and serves | ✓ VERIFIED | `session-client.ts` imports from `generated/session-types.ts` and validates with zod. TS types generated from Rust `TS` derives. 9/9 frontend tests pass. |
| 4 | User can save the current canonical session to app-owned data on disk | ✓ VERIFIED | `save_session_to_path` writes pretty JSON via atomic rename. 5/5 persistence tests pass including round-trip. |
| 5 | User can reopen a saved session and recover full graph structure | ✓ VERIFIED | `open_session_from_path` reads JSON, validates schemaVersion, replaces store only after validation. Round-trip test confirms data fidelity. |
| 6 | Save and open work without any runtime engine acting as the source of truth | ✓ VERIFIED | Persistence layer uses `serde_json` directly on `SessionDocument`. No runtime query anywhere in save/open path. |
| 7 | User can inspect the current session as visible graph nodes and connections | ✓ VERIFIED | `GraphViewport` renders via `ReactFlow` with `graphNodes` and `graphEdges` from projections. `data.label` is set (Plan 04 gap fix). |
| 8 | User can inspect a selected node's metadata and can also clear the selection | ✗ FAILED | `deriveSelectedNode` (projections.ts:145-151) returns `session.nodes[0]` when `selectedNodeId` is null. Pane click → `selectNode(null)` → first node still shown. Inspector empty state unreachable. |
| 9 | User can trigger save/open through native desktop dialogs | ✗ FAILED | `App.tsx` lines 64/73 use `window.prompt()` for path input. No `@tauri-apps/plugin-dialog` in `package.json` or `tauri_plugin_dialog` in `lib.rs`. |
| 10 | Default session renders nodes visually in the graph viewport | ✓ VERIFIED | `projectGraphNodes` sets `data.label` on every node. `ReactFlow` default node renderer uses `data.label`. |
| 11 | Each node displays its label text in the graph | ✓ VERIFIED | `labelForNode` produces `${node.nodeType}:${primaryPort}`. Set as both `data.label` and `data.title`. |

**Score:** 8/11 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/domain/session.rs` | Canonical SessionDocument schema | ✓ VERIFIED | 965 lines, contains `pub struct SessionDocument`, all nested types, `#[serde(rename_all = "camelCase")]`, TS export |
| `src-tauri/src/application/session_store.rs` | In-memory session store | ✓ VERIFIED | 479 lines, `pub struct SessionStore`, managed via Tauri State |
| `src/generated/session-types.ts` | Rust-exported frontend contracts | ✓ VERIFIED | 109 lines, `export type SessionDocument` present, all types exported |
| `src-tauri/build.rs` | Build-time contract generation guard | ✗ STUB | 3 lines, bare `tauri_build::build()`, no contract generation. Required pattern `session-types.ts` missing. |
| `src-tauri/src/persistence/session_file.rs` | Versioned JSON save/open | ✓ VERIFIED | 77 lines, `save_session_to_path`, `open_session_from_path`, `SessionFileError`, atomic writes, schema version check |
| `src-tauri/tests/session_persistence.rs` | Round-trip persistence tests | ✓ VERIFIED | 113 lines, 5 tests all passing |
| `src/App.tsx` | Phase 1 workspace shell | ✓ VERIFIED | 183 lines, composes toolbar/viewport/inspector/views. Still uses `window.prompt` (gap) |
| `src/components/session/GraphViewport.tsx` | Visible graph rendering | ✓ VERIFIED | 60 lines, `ReactFlow`, `onPaneClick`, `onNodeClick` wired |
| `src/components/session/NodeInspector.tsx` | Selected-node metadata inspector | ✓ VERIFIED | 169 lines, shows id/type/ports/parameters/runtime target/scene membership/ownership. Empty state text present but unreachable due to deriveSelectedNode bug |
| `src/store/session-projections.ts` | Graph node projections with data.label | ✓ VERIFIED | 288 lines, `data.label` set, `deriveSelectedNode` exported (with fallback bug) |
| `src/store/session-projections.test.ts` | Frontend projection tests | ✓ VERIFIED | 420 lines, 9/9 tests passing |
| `src/components/session/SessionToolbar.tsx` | Toolbar with new/save/open actions | ✓ VERIFIED | 35 lines, buttons wired to handlers |
| `src/lib/session-client.ts` | Tauri invoke wrappers with zod validation | ✓ VERIFIED | 443 lines, all commands wrapped, zod validation at boundary |
| `src/store/sessionStore.ts` | Zustand mirror store | ✓ VERIFIED | 745 lines, full store with projections, actions wired |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src-tauri/src/lib.rs` | `session_store.rs` | `State<Mutex<SessionStore>>` | ✓ WIRED | 23 command functions use managed state |
| `src-tauri/src/domain/session.rs` | `session-types.ts` | `ts-rs` TS export | ✓ WIRED | 55 TS derives, `write_generated_typescript_contract()` generates one file |
| `src-tauri/build.rs` | `session-types.ts` | Build step writes contract | ✗ NOT_WIRED | build.rs is bare `tauri_build::build()`. Contract generation runs at runtime in `lib.rs::run()` line 250 with `let _ =` (error discarded) |
| `src-tauri/src/lib.rs` | `persistence/session_file.rs` | Command dispatch | ✓ WIRED | `save_session_to_path` and `open_session_from_path` commands call session_file functions |
| `src-tauri/src/lib.rs` | `session_store.rs` | `replace_current` | ✓ WIRED | Used in `open_session_from_path` and `create_default_session` |
| `src/lib/session-client.ts` | `src-tauri/src/lib.rs` | `invoke()` calls | ✓ WIRED | All 4 session commands invoked via typed wrappers with zod validation |
| `src/store/sessionStore.ts` | `GraphViewport.tsx` | Projected node/edge data | ✓ WIRED | `graphNodes`/`graphEdges` passed as props from App.tsx |
| `GraphViewport.tsx` | `NodeInspector.tsx` | Pane click clears selection | ⚠️ PARTIAL | `onPaneClick` fires `onSelectNode(null)` but `deriveSelectedNode` returns first node, so selection never visually clears |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|--------------------|--------|
| `GraphViewport.tsx` | `graphNodes` | `projectGraphNodes(session)` | Yes — maps session.nodes with label/position/style | ✓ FLOWING |
| `GraphViewport.tsx` | `graphEdges` | `projectGraphEdges(session)` | Yes — maps session.routes with source/target | ✓ FLOWING |
| `NodeInspector.tsx` | `selectedNode` | `deriveSelectedNode(session, selectedNodeId)` | Partial — always returns first node when null | ⚠️ STATIC — fallback prevents empty state |
| `session-client.ts` | `sessionDocumentSchema` | zod validation of invoke payloads | Yes — full schema with all fields | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Rust schema tests pass | `cargo test session_document --manifest-path src-tauri/Cargo.toml` | 1 passed | ✓ PASS |
| Persistence tests pass | `cargo test session_persistence --manifest-path src-tauri/Cargo.toml` | 5 passed | ✓ PASS |
| Frontend projection tests pass | `npx vitest run src/store/session-projections.test.ts` | 9 passed | ✓ PASS |
| Frontend production build succeeds | `npm run build` | Built successfully | ✓ PASS |
| TS types exported from Rust | `grep "export type SessionDocument" src/generated/session-types.ts` | Found | ✓ PASS |
| greet command removed | `grep "fn greet" src-tauri/src/lib.rs` | Not found | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| SESS-01 | Plan 01, Plan 02 | User can create/open session with canonical state | ✓ SATISFIED | SessionDocument, SessionStore, create/open commands all wired and tested |
| SESS-02 | Plan 03, Plan 04 | User can inspect current session as visible graph nodes | ✓ SATISFIED | GraphViewport renders from canonical data via projections with data.label |
| SESS-03 | Plan 03 | User can inspect node identity, type, ports, parameters, runtime target, scene membership, ownership | ✓ SATISFIED | NodeInspector shows all required fields when a node is selected |
| PERS-01 | Plan 02, Plan 03 | User can save/reload session with all canonical data | ✓ SATISFIED | save/open with round-trip tests, schema validation, atomic writes |

No orphaned requirements found — all 4 phase requirements are claimed by plans and verified.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src-tauri/src/lib.rs` | 250 | `let _ = write_generated_typescript_contract()` | 🛑 Blocker | Contract write errors silently ignored. Plan required build-time failure. |
| `src/App.tsx` | 64, 73 | `window.prompt()` for file paths | 🛑 Blocker | Plan 03 explicitly required native desktop dialogs, not browser prompts |
| `src/store/session-projections.ts` | 146-150 | `session.nodes[0] ?? null` fallback in deriveSelectedNode | 🛑 Blocker | Inspector empty state unreachable. Plan 03 required clear deselection. |
| `src-tauri/build.rs` | 1-3 | Bare `tauri_build::build()` — no contract guard | ⚠️ Warning | Contract generation not build-guarded as Plan 01 required |

### Human Verification Required

### 1. Graph Node Visibility
**Test:** Run `npm run tauri dev`, observe the graph viewport
**Expected:** Seeded nodes are visible with labels like "source:out", "effect:in", "output:in"
**Why human:** Visual rendering confirmation requires running app

### 2. Node Selection and Inspection
**Test:** Click a node, then click empty graph space
**Expected:** Clicking node shows metadata in inspector; clicking empty space shows "Select a node to inspect its canonical metadata."
**Why human:** deriveSelectedNode bug means this may not work as expected — needs human confirmation of actual behavior

### 3. Save and Open with Native Dialogs
**Test:** Click Save Session and Open Session buttons
**Expected:** Native desktop file dialogs appear
**Why human:** window.prompt is a browser dialog — behavior differs from native file picker. Need human confirmation of what actually appears.

### Gaps Summary

Three gaps block full goal achievement, all traceable to Plan 03 execution:

1. **Selection never clears** — `deriveSelectedNode` in `session-projections.ts` falls back to `session.nodes[0]` for both null input and missing IDs. This means the inspector always shows a node and the empty state copy is unreachable. The fix is to return `null` when `selectedNodeId` is `null`.

2. **Save/Open use browser prompts, not native dialogs** — Plan 03 explicitly required `@tauri-apps/plugin-dialog` and native desktop file pickers. The implementation uses `window.prompt()` instead. The Summary acknowledges this deviation but the requirement was not met.

3. **Contract generation not build-guarded** — Plan 01 required `build.rs` to fail the build if TS contract generation fails. Instead, `build.rs` is bare and the contract is generated at runtime with errors silently discarded (`let _ =`). This means TS contract drift won't be caught at build time.

All Rust-side work (schema, store, persistence) is solid and well-tested. The frontend workspace is mostly wired correctly but has these three behavioral gaps that prevent the phase goal from being fully achieved.

---

_Verified: 2026-04-13T20:12:00Z_
_Verifier: the agent (gsd-verifier)_
