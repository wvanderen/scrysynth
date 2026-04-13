---
phase: 05-visual-sync-cross-modal
plan: 03
subsystem: hardware-binding
tags: [midi, osc, hardware-binding, learn-state-machine, cross-modal-control]
dependency_graph:
  requires: [05-visual-sync-cross-modal-02]
  provides: [hardware-binding-system, midi-learn, live-routing]
  affects: [session-document, performance-view, session-store]
tech_stack:
  added: [midir@0.10, rosc@0.11]
  patterns: [std::sync::mpsc channel bridge, frontend polling, learn state machine]
key_files:
  created:
    - src-tauri/src/hardware/mod.rs
    - src-tauri/src/hardware/midi_input.rs
    - src-tauri/src/hardware/osc_input.rs
    - src-tauri/src/application/midi_learn.rs
    - src/components/workspace/MidiLearnOverlay.tsx
    - src-tauri/tests/midi_learn.rs
  modified:
    - src-tauri/src/domain/session.rs
    - src-tauri/src/application/session_store.rs
    - src-tauri/src/application/mod.rs
    - src-tauri/src/lib.rs
    - src-tauri/Cargo.toml
    - src/lib/session-client.ts
    - src/store/sessionStore.ts
    - src/components/workspace/PerformanceView.tsx
    - src/App.tsx
decisions:
  - midir 0.10 uses MidiInputPort objects not usize indices for port selection
  - std::sync::mpsc used for MIDI callback channels (midir callback runs on non-async thread)
  - rosc 0.11 uses decode_udp instead of decode for UDP OSC packets
  - Frontend polling at 100ms interval for hardware events (not background thread)
  - HardwareInputRouter has manual Debug impl to handle non-Debug Receiver fields
metrics:
  duration: 1103s
  completed: "2026-04-12"
  tasks: 3
  files: 15
  tests_added: 23
---

# Phase 5 Plan 3: Hardware Binding with MIDI/OSC Learn Summary

Hardware input binding with learn state machine, std::sync::mpsc channel bridge, live routing, and MidiLearnOverlay UI enabling tactile hardware control of macros and performance actions.

## Tasks Completed

### Task 1: Domain types, hardware module, learn state machine
- Added `HardwareSource`, `BindingTarget`, `ValueTransform`, `HardwareBinding` types to domain
- Created `hardware::midi_input` with `MidiInputManager` using midir + std::sync::mpsc
- Created `hardware::osc_input` with `OscInputManager` using rosc + UDP listener
- Created `application::midi_learn` with `HardwareInputRouter` learn state machine
- All types registered for ts-rs code generation

### Task 2: IPC wiring, Zustand actions, MidiLearnOverlay, live routing
- Added Tauri commands: `start_hardware_learn`, `stop_hardware_learn`, `poll_hardware_events`, `remove_hardware_binding`
- SessionStore methods for hardware learn state and binding management
- Zod schemas for all hardware binding types in session-client
- Zustand store: `hardwareBindings`, `midiLearnActive`, `midiLearnTarget` state + actions
- Hardware polling mechanism (100ms setInterval) for live routing during performance
- MidiLearnOverlay component with pulse animation and cancel button
- PerformanceView Hardware Bindings section with learn/remove controls

### Task 3: Integration tests
- 17 tests: serialization round-trips, learn state transitions, MIDI parsing, value transform scaling, live routing, backward compat, TypeScript contracts, binding CRUD, transport control

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] midir 0.10 API uses MidiInputPort not usize**
- **Found during:** Task 1
- **Issue:** midir 0.10 changed port API from usize indices to MidiInputPort objects
- **Fix:** Rewrote list_devices and start_listening to use ports() Vec and MidiInputPort references
- **Files modified:** src-tauri/src/hardware/midi_input.rs

**2. [Rule 1 - Bug] rosc 0.11 uses decode_udp not decode**
- **Found during:** Task 1
- **Issue:** rosc 0.11 removed decode() in favor of decode_udp() for UDP packets
- **Fix:** Changed to rosc::decoder::decode_udp
- **Files modified:** src-tauri/src/hardware/osc_input.rs

**3. [Rule 3 - Blocking] HardwareInputRouter needs Debug for SessionStore derive**
- **Found during:** Task 2
- **Issue:** HardwareInputRouter contains std::sync::mpsc::Receiver which doesn't implement Debug
- **Fix:** Added manual Debug impl with type-name placeholders for Receiver fields
- **Files modified:** src-tauri/src/application/midi_learn.rs

**4. [Rule 1 - Bug] Borrow conflict in apply_binding_target**
- **Found during:** Task 1
- **Issue:** Immutable borrow of session.macros conflicted with mutable borrow for apply_audio_parameter
- **Fix:** Clone macro data (range_start, range_end, targets) before mutable operations
- **Files modified:** src-tauri/src/application/midi_learn.rs

## Test Results

- **Rust tests:** 66 passed (49 lib + 17 midi_learn integration), 0 failed
- **TypeScript:** Compiles clean with no errors
- **All existing tests continue to pass**

## Known Stubs

None - all data paths are fully wired.

## Self-Check: PASSED

- All 6 created files verified present
- All 3 commit hashes verified in git history
- All Rust tests pass (66 total)
- TypeScript compiles clean
