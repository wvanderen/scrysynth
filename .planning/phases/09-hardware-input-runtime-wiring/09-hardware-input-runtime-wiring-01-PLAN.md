---
phase: 09-hardware-input-runtime-wiring
plan: 01
type: hardening
status: planned
created: 2026-06-17
depends_on:
  - 08-real-visual-runtime-path
td_epic: td-dcaf9a
requirements:
  - CTRL-04R
  - HW-01
  - REL-02
---

# Phase 9: Hardware Input Runtime Wiring

## Goal

MIDI/OSC learn works against live devices and senders in the desktop app.

This phase should turn the existing hardware binding model, learn state machine, and UI scaffold into a real runtime path. A performer should be able to select/configure input sources, learn controls from live MIDI or OSC input, and then use those bindings during performance without the app drifting away from the audio and visual runtimes.

## Product Boundary

Canonical session state remains app-owned. Hardware listeners, MIDI connection handles, UDP sockets, OS port IDs, receiver handles, thread handles, and transport-specific runtime details must stay outside `SessionDocument`.

Bindings are canonical. Listener processes and OS handles are runtime resources.

## Current Baseline

- `HardwareBinding`, `HardwareSource`, `BindingTarget`, and `ValueTransform` are part of the Rust session contract and generated TypeScript types.
- `MidiInputManager` can list devices, connect to a MIDI input port, parse CC/note/pitch-bend messages, and emit `MidiLearnEvent` values.
- `OscInputManager` can bind a UDP port and decode OSC messages/bundles into `OscLearnEvent` values, but shutdown/restart behavior needs hardening.
- `HardwareInputRouter` can learn a source for a target and can route idle live events to macros, scene recall, transport play, transport stop, and panic in app state.
- `SessionStore` exposes learn/poll/remove helpers, but it does not own live MIDI or OSC listener startup/configuration yet.
- Tauri commands exist for `start_hardware_learn`, `stop_hardware_learn`, `poll_hardware_events`, and `remove_hardware_binding`.
- The React performance view exposes basic learn buttons for macros/scenes and a binding list.

## Release Gap

The current code proves the binding and routing model, but not desktop runtime behavior. Phase 9 closes that gap by wiring live listener lifecycle, app commands/settings, receiver attachment, runtime reconciliation, UI status, and manual UAT.

## Success Criteria

1. Users can list/select a MIDI input port and see actionable diagnostics when MIDI is unavailable.
2. Users can configure an OSC listen endpoint and see actionable diagnostics when binding fails.
3. MIDI and OSC listeners start, stop, restart, and attach receivers to `HardwareInputRouter` as app runtime resources.
4. Learn mode captures live MIDI and OSC input into durable `HardwareBinding` records.
5. Existing bindings route live input to macro, scene recall, transport play, transport stop, and panic targets.
6. Hardware-routed macro, scene, transport, and panic actions reconcile with active audio and visual runtimes instead of only mutating app state.
7. The workspace shows hardware settings, listener status, captured bindings, learn state, and setup errors clearly enough for performance use.
8. UAT verifies real or virtual MIDI and OSC input end to end and records evidence before Phase 9 is marked complete.

## Task Breakdown

### 9.1 Define hardware runtime config and command contract

Task: `td-fca6b3`

Define the app-level hardware runtime contract before wiring live listeners. Cover MIDI device listing/selection, OSC bind host and listen port, listener status, setup diagnostics, and how hardware runtime state is exposed to TypeScript/UI without making runtime handles part of `SessionDocument`.

Acceptance:

- A Phase 9 contract document records the command surface, settings shape, status projection, error states, and non-goals.
- Rust domain/application types exist for hardware runtime settings and status, with generated TypeScript contract updates if exposed to the UI.
- Commands are planned or stubbed for listing MIDI inputs, reading/updating hardware settings, starting/stopping listeners, and polling/draining hardware events.
- Diagnostics are specified for no MIDI ports, invalid MIDI port selection, OSC bind failure, port-in-use, and listener restart after configuration changes.
- Runtime-owned handles, OS port IDs, socket objects, threads, and connection handles remain outside `SessionDocument`.

### 9.2 Wire MIDI input listener lifecycle into SessionStore

Task: `td-c18ac3`

Promote `MidiInputManager` from a testable helper to a real app-owned runtime resource. `SessionStore` should own MIDI listener lifecycle, attach the manager receiver to `HardwareInputRouter`, expose MIDI port listing and selection through Tauri commands, and keep learn/live routing functional after listener restarts.

Acceptance:

- `SessionStore` owns a MIDI input manager/receiver path and can start, stop, and restart MIDI listening for the selected port.
- A Tauri command lists available MIDI input ports with stable user-facing labels and returns actionable errors when `midir` cannot enumerate devices.
- Starting hardware learn ensures MIDI listening is active or reports clear setup guidance.
- Changing the selected MIDI port tears down the prior connection and attaches the new receiver to `HardwareInputRouter` without losing existing `HardwareBinding` records.
- Tests cover successful receiver wiring, invalid port selection, no-port diagnostics where mockable, and learn capture through the app-level store path.

### 9.3 Wire OSC listener lifecycle and reliable shutdown

Task: `td-499618`

Promote `OscInputManager` from scaffold to real app-owned listener. The app should configure an OSC bind host/listen port, start and stop the UDP listener predictably, attach its receiver to `HardwareInputRouter`, and recover cleanly from bind errors or config changes.

Known implementation risk: `OscInputManager::stop_listening` currently stores a shutdown socket but does not retain the shutdown sender. Verify and fix shutdown signaling so listener threads cannot linger or make restart flaky.

Acceptance:

- `SessionStore` owns OSC listener lifecycle and attaches the returned `Receiver<OscLearnEvent>` to `HardwareInputRouter`.
- OSC listen settings include at least bind host and port, with localhost as the safe default unless the contract chooses otherwise.
- Starting, stopping, and restarting the OSC listener releases the UDP port reliably and does not leave listener threads running.
- Bind failures, invalid ports, and port-in-use states return clear diagnostics to the command/UI layer.
- Tests cover OSC event capture through the app-level store path, listener restart on the same port, and shutdown behavior.

### 9.4 Reconcile hardware-routed actions with audio and visual runtimes

Task: `td-eb6bc2`

Make live hardware input do more than mutate `SessionDocument` fields. When `HardwareInputRouter` routes a bound event, the app should reconcile affected macros, scenes, transport, panic, audio parameter updates, and visual parameter updates through the same runtime-manager paths used by direct UI commands.

Acceptance:

- Hardware macro bindings update canonical macro-targeted parameters and reconcile live audio parameter updates when the audio runtime is active.
- Hardware macro bindings targeting visual parameters send visual parameter batches when the visual runtime is active.
- Hardware scene recall follows the performance command semantics, including active visual scene reload and any required runtime state updates.
- Hardware transport play, stop, and panic targets invoke the appropriate runtime lifecycle behavior or explicitly document any v1 limitation.
- Polling/draining multiple hardware events is bounded, deterministic, and avoids unbounded UI-triggered work per tick.
- Tests cover macro, scene, transport, panic, audio reconcile, visual reconcile, and error/degraded-state behavior through `SessionStore`.

### 9.5 Add hardware settings, status, and learn controls to the workspace

Task: `td-962e1b`

Expose the hardware runtime path in the desktop workspace. Users should be able to choose a MIDI input, configure the OSC listen endpoint, see listener health, start learn for macro/scene/transport targets, cancel learn, and understand captured bindings or setup failures without reading logs.

Acceptance:

- UI exposes MIDI port selection and OSC listen host/port settings using the command contract from Phase 9.1.
- UI shows listener states such as stopped, listening, learning, captured, error, and unavailable with concise diagnostics.
- Learn controls cover macro, scene recall, transport play, transport stop, and transport panic targets.
- Binding rows identify MIDI vs OSC sources, target labels, and removal behavior clearly.
- Polling starts/stops predictably based on active listeners, learn state, or existing live bindings, without leaking intervals.
- Frontend tests cover status projection, learn controls, binding display, cancel/remove behavior, and error display.

### 9.6 Verify live MIDI/OSC hardware UAT and update docs

Task: `td-8062dd`

Run and document end-to-end Phase 9 UAT using real or virtual hardware inputs. The verification should prove that live MIDI and OSC input can be learned in the desktop app and then used during performance for macros, scene recall, transport stop/play, and panic.

Acceptance:

- Phase 9 UAT doc records local setup, commands/tools used, device or virtual source names, expected behavior, observed behavior, and pass/fail result.
- MIDI learn is verified from a live source for at least one macro target and one transport or scene target.
- OSC learn is verified from a live sender for at least one macro target and one transport or scene target.
- Existing bindings are verified after learn by moving/sending controls outside learn mode and observing session/runtime changes.
- Panic target is verified to stop active runtime behavior safely.
- README, ROADMAP, REQUIREMENTS, and STATE are updated only to the level of behavior actually verified.

## Suggested Dependency Order

1. 9.1 command/config/status contract.
2. 9.2 MIDI listener lifecycle.
3. 9.3 OSC listener lifecycle.
4. 9.4 runtime reconciliation for routed hardware actions.
5. 9.5 workspace settings/status/learn controls.
6. 9.6 live MIDI/OSC UAT and docs.

## UAT Notes

Use real hardware if available. If not, virtual sources are acceptable:

- macOS IAC Driver or another virtual MIDI source for MIDI learn/routing.
- A local OSC sender for messages such as `/scrysynth/energy 0.75`, `/scrysynth/scene 1`, or a target-specific address chosen by learned binding behavior.

Record exact device names, OSC endpoint, command/tool versions where relevant, and observed app/runtime behavior.

## Non-Goals

- No advanced MIDI controller scripting or per-device template marketplace.
- No DAW-style automation lanes or timeline recording.
- No multiplayer hardware state synchronization.
- No persistence of runtime listener handles, OS MIDI port IDs, UDP sockets, thread handles, or receiver handles in `SessionDocument`.
- No Phase 10 agent orchestration work.
- No release packaging work beyond documentation updates required by verified Phase 9 behavior.
