---
phase: 05-visual-sync-cross-modal
plan: 03
type: execute
wave: 3
depends_on:
  - 05-visual-sync-cross-modal-02
files_modified:
  - src-tauri/src/domain/session.rs
  - src-tauri/src/hardware/mod.rs
  - src-tauri/src/hardware/midi_input.rs
  - src-tauri/src/hardware/osc_input.rs
  - src-tauri/src/application/midi_learn.rs
  - src-tauri/src/application/mod.rs
  - src-tauri/src/application/session_store.rs
  - src-tauri/src/lib.rs
  - src-tauri/Cargo.toml
  - src/lib/session-client.ts
  - src/store/sessionStore.ts
  - src/components/workspace/MidiLearnOverlay.tsx
  - src/components/workspace/PerformanceView.tsx
  - src-tauri/tests/midi_learn.rs
autonomous: true
requirements:
  - CTRL-04
must_haves:
  truths:
    - User can activate MIDI learn mode from the UI, move a hardware control, and have it bound to a macro or performance action.
    - User can activate OSC learn mode, send an OSC message, and have it bound to a macro or performance action.
    - Bound hardware input triggers the target action in real-time during performance (MIDI CC → macro value, MIDI note → scene recall, etc.).
    - Hardware bindings persist in the session document and survive save/load.
    - Bound hardware input routes to its target in real-time during performance (continuous polling when bindings exist).
    - MIDI input runs on a background thread with std::sync::mpsc channel bridging to avoid blocking the UI or session store.
    - Removing a hardware binding stops it from triggering.
  artifacts:
    - path: src-tauri/src/domain/session.rs
      provides: HardwareBinding, HardwareSource, BindingTarget, ValueTransform types
      contains: "HardwareBinding"
    - path: src-tauri/src/hardware/midi_input.rs
      provides: MIDI device enumeration, callback-based input, tokio channel bridge
      contains: "MidiInputManager"
    - path: src-tauri/src/hardware/osc_input.rs
      provides: OSC input listener for learn and live input
      contains: "OscInputManager"
    - path: src-tauri/src/application/midi_learn.rs
      provides: MIDI/OSC learn state machine, hardware event → macro/action routing
      contains: "HardwareLearnState"
    - path: src/components/workspace/MidiLearnOverlay.tsx
      provides: UI overlay for learn mode activation and binding display
      contains: "MidiLearnOverlay"
    - path: src-tauri/tests/midi_learn.rs
      provides: Integration tests for learn state machine, binding CRUD, event routing
      contains: "start_midi_learn"
  key_links:
    - from: src-tauri/src/hardware/midi_input.rs
      to: src-tauri/src/application/midi_learn.rs
      via: std::sync::mpsc channel carrying MidiLearnEvent (bridged from midir callback thread)
      pattern: "MidiLearnEvent.*mpsc"
    - from: src-tauri/src/application/midi_learn.rs
      to: src-tauri/src/application/macro_command.rs
      via: Hardware binding routes MIDI/OSC values to SetMacroValue commands
      pattern: "BindingTarget::Macro"
    - from: src/components/workspace/MidiLearnOverlay.tsx
      to: src/store/sessionStore.ts
      via: User activates learn mode, UI shows awaiting state, hardware event captured, binding created
      pattern: "startMidiLearn.*macro_id"
---

<objective>
Add MIDI and OSC hardware input binding with a learn state machine, std::sync::mpsc channel bridge for thread-safe input, continuous live routing during performance, and a UI overlay for learn mode activation.

Purpose: Enables tactile hardware control of macros and performance actions — the final piece of cross-modal performance control.

Output: HardwareBinding domain types, MIDI input manager with std::sync::mpsc bridge, learn state machine, continuous live routing, MidiLearnOverlay UI, integration tests.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/05-visual-sync-cross-modal/05-RESEARCH.md

<interfaces>
<!-- Hardware binding domain types from research -->

```rust
// HardwareSource — what hardware generates the event
#[serde(tag = "kind", content = "config", rename_all = "camelCase")]
pub enum HardwareSource {
    MidiCc { channel: u8, controller: u8 },
    MidiNote { channel: u8, note: u8 },
    MidiPitchBend { channel: u8 },
    OscAddress { address: String },
}

// BindingTarget — what the event triggers
#[serde(tag = "kind", content = "config", rename_all = "camelCase")]
pub enum BindingTarget {
    Macro { macro_id: String },
    SceneRecall { scene_id: String },
    TransportPlay,
    TransportStop,
    TransportPanic,
}

// ValueTransform — maps hardware value range to target range
pub struct ValueTransform {
    pub input_min: f64,
    pub input_max: f64,
    pub output_min: f64,
    pub output_max: f64,
}
```

<!-- MIDI learn event types -->
```rust
pub enum MidiLearnEvent {
    MidiCc { channel: u8, controller: u8, value: u8 },
    MidiNote { channel: u8, note: u8, velocity: u8 },
    MidiPitchBend { channel: u8, value: u16 },
}
```

<!-- Existing session/macros from Plan 02 -->
```rust
pub struct MacroDefinition { pub id, pub name, pub targets: Vec<MacroTarget>, pub range_start, pub range_end }
pub struct SessionDocument { ... pub macros: Vec<MacroDefinition>, pub hardware_bindings: Vec<HardwareBinding>, ... }
```

<!-- Macro command from Plan 02 -->
```rust
pub enum MacroCommand {
    SetMacroValue { macro_id: String, value: f64 },
    // ... other variants
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add HardwareBinding types to domain, create hardware module with MIDI/OSC input managers, implement learn state machine</name>
  <files>src-tauri/Cargo.toml, src-tauri/src/domain/session.rs, src-tauri/src/hardware/mod.rs, src-tauri/src/hardware/midi_input.rs, src-tauri/src/hardware/osc_input.rs, src-tauri/src/application/midi_learn.rs, src-tauri/src/application/mod.rs</files>
  <action>
1. In `src-tauri/Cargo.toml`:
   - Add `midir = "0.10"` to `[dependencies]`.
   - `rosc` should already be present (used for SC adapter). If not, add `rosc = "0.11"`.

2. In `src-tauri/src/domain/session.rs`:
   - Add `HardwareSource` tagged enum: `MidiCc { channel: u8, controller: u8 }`, `MidiNote { channel: u8, note: u8 }`, `MidiPitchBend { channel: u8 }`, `OscAddress { address: String }`. Derive Clone, Debug, PartialEq, Serialize, Deserialize, TS.
   - Add `BindingTarget` tagged enum: `Macro { macro_id: String }`, `SceneRecall { scene_id: String }`, `TransportPlay`, `TransportStop`, `TransportPanic`. Same derives.
   - Add `ValueTransform` struct: `input_min: f64, input_max: f64, output_min: f64, output_max: f64`. Same derives.
   - Add `HardwareBinding` struct: `id: String, source: HardwareSource, target: BindingTarget, transform: ValueTransform`. Same derives.
   - Add `hardware_bindings: Vec<HardwareBinding>` field to SessionDocument with `#[serde(default)]`.
   - Register all new types in `write_generated_typescript_contract()`.

3. Create `src-tauri/src/hardware/mod.rs` with `pub mod midi_input; pub mod osc_input;`.

4. Create `src-tauri/src/hardware/midi_input.rs`:
   - Define `MidiLearnEvent` enum: `MidiCc { channel, controller, value }`, `MidiNote { channel, note, velocity }`, `MidiPitchBend { channel, value }`.
   - Define `MidiInputManager` struct:
     ```rust
     pub struct MidiInputManager {
         connection: Option<midir::MidiInputConnection<()>>,
         event_sender: std::sync::mpsc::Sender<MidiLearnEvent>,
     }
     ```
   - Implement `new(sender: std::sync::mpsc::Sender<MidiLearnEvent>)` — stores sender, connection is None.
   - Implement `list_devices() -> Result<Vec<String>, String>`: creates MidiInput, iterates ports, returns port names.
   - Implement `start_listening(&mut self, port_index: Option<usize>) -> Result<(), String>`:
     - Creates `midir::MidiInput::new("scrysynth")`.
     - Selects port by index or falls back to first available.
     - Connects with callback that parses MIDI bytes via `parse_midi_message` and sends through `event_sender`.
     - Stores connection.
   - Implement `stop_listening(&mut self)`: drops the connection (set to None).
   - Implement `parse_midi_message(message: &[u8]) -> Option<MidiLearnEvent>`:
     - Parse status byte: 0xB0 = CC, 0x90 = Note On, 0xE0 = Pitch Bend.
     - Extract channel from lower nibble.
     - Map to MidiLearnEvent variants.
   - IMPORTANT: Use `std::sync::mpsc::Sender` (not tokio) in the MIDI callback since midir's callback runs on a non-async thread. A separate tokio task bridges std::sync::mpsc → session mutations.

5. Create `src-tauri/src/hardware/osc_input.rs`:
   - Define `OscLearnEvent` struct: `address: String, args: Vec<rosc::OscType>`.
   - Define `OscInputManager` struct with a UDP socket handle.
   - Implement `start_listening(port: u16) -> Result<std::sync::mpsc::Receiver<OscLearnEvent>, String>`:
     - Binds UDP socket to 127.0.0.1:port.
     - Spawns std::thread that reads UDP packets, parses OSC via rosc, sends events through channel.
   - Implement `stop_listening()`: drops the socket (join the thread).
   - For v1, OSC input is listen-only and receives on a configurable port (default 9100).

6. Create `src-tauri/src/application/midi_learn.rs`:
   - Define `HardwareLearnState` enum: `Idle, Learning { target: BindingTarget }, Captured { source: HardwareSource, target: BindingTarget }`.
   - Define `HardwareInputRouter` struct:
     - `learn_state: HardwareLearnState`
     - `midi_rx: Option<std::sync::mpsc::Receiver<MidiLearnEvent>>`
     - `osc_rx: Option<std::sync::mpsc::Receiver<OscLearnEvent>>`
   - Implement `start_learn(&mut self, target: BindingTarget)`: sets state to Learning.
   - Implement `stop_learn(&mut self)`: resets state to Idle.
   - Implement `poll_and_route(&mut self, session: &mut SessionDocument) -> Option<HardwareBinding>`:
     - Non-blocking try_recv on midi_rx and osc_rx.
     - If learn_state is Learning and an event arrives:
       - Convert event to HardwareSource.
       - Create HardwareBinding with default ValueTransform (input 0-127 for MIDI, output 0-1 for macro).
       - Add to session.hardware_bindings.
       - Set learn_state to Captured.
       - Return the binding.
     - If learn_state is Idle and an event arrives:
       - Find matching HardwareBinding in session.hardware_bindings.
       - Apply the binding: compute scaled value via ValueTransform, dispatch to target (SetMacroValue, RecallScene, etc.).
       - Return None (binding already applied).
   - For v1, the `poll_and_route` method is called periodically from a Tauri command (not a background thread) to keep things simple. The MIDI/OSC callbacks just buffer events into channels.

7. In `src-tauri/src/application/mod.rs`: Add `pub mod midi_learn;`.
  </action>
  <verify>
    <automated>cd src-tauri && cargo test --lib -- hardware --nocapture 2>&1 | tail -20</automated>
  </verify>
  <done>midir dependency added. HardwareBinding types added to domain. hardware/ module created with MIDI input manager (midir + std::sync::mpsc) and OSC input manager (rosc + UDP). Learn state machine implemented. All types registered for ts-rs.</done>
</task>

<task type="auto">
  <name>Task 2: Wire learn commands through Tauri IPC, add Zustand actions, build MidiLearnOverlay UI, add live input routing</name>
  <files>src-tauri/src/lib.rs, src-tauri/src/application/session_store.rs, src/lib/session-client.ts, src/store/sessionStore.ts, src/components/workspace/MidiLearnOverlay.tsx, src/components/workspace/PerformanceView.tsx</files>
  <action>
1. In `src-tauri/src/application/session_store.rs`:
   - Add `hardware_router: HardwareInputRouter` field to SessionStore (wrapped in RefCell or similar for interior mutability since poll_and_route takes &mut self).
   - Actually, simpler approach: store the MIDI/OSC channel receivers and learn state directly in SessionStore. The router can be a separate concern.
   - Add `start_hardware_learn(target: BindingTarget)`: sets learn state to Learning with target.
   - Add `stop_hardware_learn()`: resets learn state to Idle.
   - Add `poll_hardware_events()`: calls router.poll_and_route, returns updated SessionDocument if bindings changed.
   - Add `remove_hardware_binding(binding_id: String)`: removes from session.hardware_bindings.

2. In `src-tauri/src/lib.rs`:
   - Add `pub mod hardware;` at top.
   - Add Tauri commands:
     - `start_midi_learn(target: BindingTarget, state: tauri::State<Mutex<SessionStore>>)`: calls store.start_hardware_learn(target).
     - `stop_midi_learn(state)`: calls store.stop_hardware_learn().
     - `poll_hardware_events(state)`: calls store.poll_hardware_events(), returns updated session.
     - `remove_hardware_binding(binding_id: String, state)`: calls store.remove_hardware_binding(id).
   - Register all in invoke_handler.

3. In `src/lib/session-client.ts`:
   - Add zod schemas for `hardwareSourceSchema`, `bindingTargetSchema`, `valueTransformSchema`, `hardwareBindingSchema`.
   - Add `hardwareBindings` field to `sessionDocumentSchema`.
   - Add IPC functions: `startMidiLearn(target)`, `stopMidiLearn()`, `pollHardwareEvents()`, `removeHardwareBinding(bindingId)`.

4. In `src/store/sessionStore.ts`:
   - Add `hardwareBindings` to store state (from session).
   - Add `midiLearnActive: boolean` and `midiLearnTarget: BindingTarget | null` to store state.
   - Add store actions:
     - `startMidiLearn(target)`: calls `startMidiLearn(target)` IPC, sets midiLearnActive=true, midiLearnTarget=target.
     - `stopMidiLearn()`: calls `stopMidiLearn()` IPC, sets midiLearnActive=false, midiLearnTarget=null.
     - `removeHardwareBinding(bindingId)`: calls IPC, updates session.
    - Add polling mechanism: set up a setInterval (100ms) that calls `pollHardwareEvents()` IPC whenever `hardwareBindings.length > 0` OR `midiLearnActive === true`. This ensures:
      - During learn mode: events are captured for binding creation.
      - During performance: bound hardware events are routed to their targets in real-time.
      - When no bindings exist and learn is inactive: no polling overhead.
      When a binding is captured during learn, update midiLearnActive. Clear interval when store is unmounted or no longer needed.
    - Derive `hardwareBindings` from session in `applySession`.

5. Create `src/components/workspace/MidiLearnOverlay.tsx`:
   - Shows when `midiLearnActive` is true.
   - Displays "Learning..." message with animated indicator.
   - Shows the target (e.g., "Waiting for input to bind to macro: energy" or "Waiting for input to bind to scene: intro").
   - "Cancel" button calls stopMidiLearn.
   - When binding is captured (new binding appears), shows success message with source details (e.g., "MIDI CC Ch1 #7 bound to macro: energy").
   - Uses useSessionStore for state.
   - Renders as a fixed overlay at the bottom of the workspace (consistent with workspace styling).

6. In `src/components/workspace/PerformanceView.tsx`:
   - Add a "Hardware Bindings" section:
     - List all existing hardware bindings: source → target (e.g., "MIDI CC Ch1 #7 → macro: energy").
     - "Remove" button on each binding.
     - "Learn" button next to each macro slider that activates MIDI learn for that macro.
     - "Learn" button for scene recall (binds a MIDI note to scene recall).
   - Import and render `MidiLearnOverlay` when learn mode is active.
  </action>
  <verify>
    <automated>cd /home/lem/dev/scrysynth && npx tsc --noEmit 2>&1 | tail -20</automated>
  </verify>
  <done>Hardware learn commands wired through IPC. Zustand manages learn state with polling for captured events. MidiLearnOverlay shows learn status. Performance view displays bindings with learn/remove controls. TypeScript compiles.</done>
</task>

<task type="auto">
  <name>Task 3: Write integration tests for hardware binding lifecycle, learn state machine, and event routing</name>
  <files>src-tauri/tests/midi_learn.rs</files>
  <action>
Create integration tests in `src-tauri/tests/midi_learn.rs`:

1. Test HardwareBinding serialization round-trip: create binding with MidiCc source and Macro target, serialize/deserialize, verify equality.
2. Test OscAddress binding round-trip similarly.
3. Test all BindingTarget variants serialize correctly.
4. Test learn state machine transitions: Idle → Learning { target } → Captured { source, target } → Idle.
5. Test learn state machine: start_learn → stop_learn returns to Idle without capturing.
6. Test adding a HardwareBinding to session and verifying it persists in session.hardware_bindings.
7. Test removing a HardwareBinding: verify it's gone from session.hardware_bindings.
8. Test MIDI message parsing: verify parse_midi_message correctly handles CC (0xB0+channel, controller, value), Note On (0x90+channel, note, velocity), Pitch Bend (0xE0+channel, LSB, MSB).
9. Test MIDI message parsing edge cases: empty message → None, unknown status byte → None, too-short messages → None.
10. Test ValueTransform scaling: input_min=0, input_max=127, output_min=0, output_max=1, input=63 → ~0.496, input=0 → 0, input=127 → 1.
11. Test SessionDocument with hardware_bindings field loads correctly from JSON (backward compat: no hardware_bindings → empty vec via serde default).
12. Test TypeScript contract generation includes HardwareBinding, HardwareSource, BindingTarget, ValueTransform types.
13. Test live routing: create a session with existing HardwareBinding (MidiCc → Macro), construct a matching MidiLearnEvent, call poll_and_route in Idle state, verify SetMacroValue is dispatched with correctly scaled value.

Note: These tests do NOT require actual MIDI hardware. They test the parsing, state machine, and persistence logic using constructed events.
  </action>
  <verify>
    <automated>cd src-tauri && cargo test --test midi_learn --nocapture 2>&1 | tail -30</automated>
  </verify>
  <done>All hardware binding integration tests pass. Learn state machine transitions verified. MIDI parsing verified. Serialization round-trips verified. Backward compatibility verified. TypeScript contracts include all new types.</done>
</task>

</tasks>

<verification>
1. `cd src-tauri && cargo test` — all tests pass including hardware binding tests.
2. `cd src-tauri && cargo build` — compiles with midir dependency.
3. `npx tsc --noEmit` — TypeScript compiles.
4. MIDI learn flow: start learn → simulated event → binding created → binding visible in UI.
5. Live routing flow: existing binding → simulated event → macro value updated → visual/audio responds.
6. Hardware bindings persist in session save/load.
6. Existing functionality unaffected.
</verification>

<success_criteria>
- User can activate MIDI learn mode, which captures the next hardware event and binds it to a macro or performance action.
- Bound hardware input routes to the correct target (macro value, scene recall, transport).
- Hardware bindings persist in the session document.
- MidiLearnOverlay shows learn state and captured bindings.
- Performance view displays existing bindings with remove capability.
- MIDI input runs safely on a background thread via std::sync::mpsc channel.
</success_criteria>

<output>
After completion, create `.planning/phases/05-visual-sync-cross-modal/05-visual-sync-cross-modal-03-SUMMARY.md`
</output>
