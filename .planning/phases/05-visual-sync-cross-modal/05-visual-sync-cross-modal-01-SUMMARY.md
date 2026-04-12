---
phase: 05-visual-sync-cross-modal
plan: 01
subsystem: visual-runtime
tags: [visual, runtime, adapter, domain-types, ipc, zustand, ui]
dependency_graph:
  requires: [domain-session, audio-adapter-pattern]
  provides: [visual-adapter, visual-runtime-manager, bevy-sidecar, visual-compiler, runtime-health-panel, agent-runtime-state]
  affects: [session-document, session-store, lib-commands, session-client, sessionStore, session-projections]
tech_stack:
  added: [visual/adapter.rs, visual/runtime_manager.rs, visual/bevy_sidecar.rs, visual/compiler.rs, RuntimeHealthPanel.tsx]
  patterns: [adapter-trait-mirroring-audio, std-mem-take-lifecycle, zod-validation, zustand-projection]
key_files:
  created:
    - src-tauri/src/visual/mod.rs
    - src-tauri/src/visual/adapter.rs
    - src-tauri/src/visual/runtime_manager.rs
    - src-tauri/src/visual/bevy_sidecar.rs
    - src-tauri/src/visual/compiler.rs
    - src/components/workspace/RuntimeHealthPanel.tsx
    - src-tauri/tests/visual_runtime.rs
  modified:
    - src-tauri/src/domain/session.rs
    - src-tauri/src/application/session_store.rs
    - src-tauri/src/lib.rs
    - src/lib/session-client.ts
    - src/store/sessionStore.ts
    - src/store/session-projections.ts
    - src/components/workspace/WorkspaceViewSwitcher.tsx
decisions:
  - Visual adapter mirrors AudioRuntimeAdapter pattern exactly (trait + manager + sidecar)
  - Bevy sidecar gracefully degrades when binary not found (returns Failed status, no crash)
  - Visual compiler maps node types to shapes (source=sphere, effect=box, mixer=ring, output=plane) for v1
  - AgentRuntimeState derived from session rather than stored (computed on demand)
  - Agent RuntimeStatusRef added to default session for unified dashboard
metrics:
  duration: 1093s
  completed: "2026-04-12"
  tasks: 3
  files: 14
---

# Phase 5 Plan 1: Visual Runtime Adapter & Runtime Health Dashboard Summary

Visual runtime adapter infrastructure mirroring the proven SC audio adapter pattern, with domain types, lifecycle management, IPC wiring, and a unified runtime health dashboard for audio/visual/agent systems.

## What Was Implemented

### Task 1: Domain Types & Visual Module Skeleton
- Added `VisualRuntimeLifecycle` (Idle, Starting, Ready, Rendering, Failed), `VisualRuntimeHealth` (Unknown, Healthy, Degraded, Error), and `VisualRuntimeState` to domain
- Added `AgentRuntimeState` (is_available, pending_action_count, is_frozen) to domain
- Created `visual/` module with adapter trait, runtime manager, Bevy sidecar adapter, and session-to-scene compiler
- All new types registered for ts-rs TypeScript contract generation with `serde(default)` for backward compatibility
- Added Agent `RuntimeStatusRef` to default session

### Task 2: IPC Wiring & Runtime Health Panel
- Wired visual runtime lifecycle (start/stop/panic) through SessionStore → VisualRuntimeManager → Tauri commands
- Added `get_agent_runtime_state` command deriving `AgentRuntimeState` from session
- Added Zod schemas for `visualRuntimeStateSchema` and `agentRuntimeStateSchema` with IPC functions
- Added Zustand actions: `startVisual`, `stopVisual`, `panicVisual`, `refreshAgentRuntime`
- Created `RuntimeHealthPanel` with colored status dots (green/yellow/red/gray), lifecycle labels, and start/stop/panic buttons for each runtime
- Added `VisualRuntimeProjection` and `AgentRuntimeProjection` to session projections
- Integrated RuntimeHealthPanel into WorkspaceViewSwitcher (visible across all views)

### Task 3: Integration Tests (11 tests)
- Visual runtime state default verification
- Session serialization round-trip with new fields
- Compiler output for enabled-node sessions and empty sessions
- Full lifecycle transitions with `TestVisualAdapter`: start→Ready, start failure→Failed, stop→Idle, panic→Idle
- Agent runtime state derivation matching session fields
- TypeScript contract generation includes all new types

## Test Results

- **Rust tests:** 47 passed (31 lib + 6 audio runtime + 5 persistence + 11 visual runtime)
- **Frontend tests:** 19 passed (3 test files)
- **Build:** Clean compilation with no errors

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

- `BevySidecarAdapter::load_scene` acknowledges scene load without actual IPC (v1 stub — no Bevy sidecar binary exists yet)
- `BevySidecarAdapter::update_parameters` returns Ok(()) unconditionally (v1 stub)
- Visual compiler maps all node types to simple shapes with no complex visual graph editing
