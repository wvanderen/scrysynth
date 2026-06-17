---
phase: 09-hardware-input-runtime-wiring
type: contract
status: draft
td_task: td-fca6b3
created: 2026-06-17
---

# Phase 9 Hardware Runtime Contract

## Boundary

`SessionDocument` owns durable musical intent: `HardwareBinding`, `HardwareSource`,
`BindingTarget`, and `ValueTransform`.

The app runtime owns hardware configuration, listener lifecycle, receiver attachment,
diagnostics, and all operating-system resources. MIDI connection handles, UDP sockets,
thread handles, receiver handles, OSC socket objects, and OS-provided MIDI port IDs do
not enter `SessionDocument`.

## Settings Shape

`HardwareRuntimeSettings` is app-level runtime configuration, not session graph data.
It may later be persisted in app preferences, but it is not embedded into exported
session files.

- `midi.selectedInputId`: optional app-scoped input id from `list_midi_input_ports`,
  currently shaped as `midi-input-N`. This id is for command routing only and must not
  be treated as a durable OS port identifier.
- `midi.autoStart`: whether the app should try to start the selected MIDI listener
  when the hardware runtime is initialized.
- `osc.bindHost`: host/interface for the OSC UDP listener. The safe default is
  `127.0.0.1`.
- `osc.listenPort`: UDP listen port. Valid range is `1..=65535`; default is `9000`.
- `osc.autoStart`: whether the app should try to start the OSC listener when the
  hardware runtime is initialized.

## Status Projection

`HardwareRuntimeStatus` is the UI-facing projection of runtime state.

- `midi.lifecycle`: `unavailable`, `stopped`, `starting`, `listening`,
  `restarting`, or `error`.
- `midi.selectedInputId`: selected app-scoped MIDI input id, if any.
- `midi.selectedDisplayName`: latest display name for the selected input, if it is
  still present.
- `midi.availableInputCount`: latest known input count, or `null` before enumeration.
- `midi.lastError`: latest MIDI setup/listener error.
- `osc.lifecycle`: `unavailable`, `stopped`, `starting`, `listening`,
  `restarting`, or `error`.
- `osc.bindHost` and `osc.listenPort`: effective OSC endpoint settings.
- `osc.lastError`: latest OSC setup/listener error.
- `learn.lifecycle`: `idle`, `learning`, or `captured`.
- `learn.target`: target being learned or captured.
- `learn.source`: captured MIDI/OSC source when learn has captured an input.
- `diagnostics`: recoverable setup/runtime messages for the UI.

Status may refer to configured ids and display labels, but it must never expose live
connection handles, sockets, threads, or OS-native port objects.

## Command Surface

- `list_midi_input_ports() -> MidiInputPort[]`
  - Enumerates current MIDI inputs.
  - Returns an empty list and records `no_midi_ports` when no ports are available.
  - Returns app-scoped ids and user-facing display names.

- `get_hardware_runtime_settings() -> HardwareRuntimeSettings`
  - Reads app-level hardware configuration.

- `update_hardware_runtime_settings(settings) -> HardwareRuntimeStatus`
  - Validates selected MIDI input id and OSC endpoint shape.
  - Updates app-level runtime configuration.
  - If active listeners are affected, status records `listener_restart_required`
    and moves affected listener projections toward `restarting` until later wiring
    performs the actual stop/start.

- `get_hardware_runtime_status() -> HardwareRuntimeStatus`
  - Reads the current UI-facing runtime projection.

- `start_hardware_listeners() -> HardwareRuntimeStatus`
  - Phase 9.1 stub: records `listener_start_pending`.
  - Phase 9.2 starts/restarts MIDI and attaches the MIDI receiver to
    `HardwareInputRouter`.
  - Phase 9.3 starts/restarts OSC and attaches the OSC receiver to
    `HardwareInputRouter`.

- `stop_hardware_listeners() -> HardwareRuntimeStatus`
  - Stops runtime-owned listeners and releases handles. In Phase 9.1 this is a
    status stub; Phase 9.2/9.3 perform the actual teardown.

- `start_hardware_learn(target) -> void`
  - Existing command. Starts learn mode against the app-owned router.
  - Later lifecycle wiring should ensure listeners are active or return setup
    diagnostics before leaving the UI in a misleading learning state.

- `stop_hardware_learn() -> void`
  - Existing command. Returns learn state to idle.

- `poll_hardware_events() -> SessionDocument`
  - Existing single-tick poll path.

- `drain_hardware_events(maxEvents?) -> SessionDocument`
  - Bounded drain path for UI polling and later runtime reconciliation. Default
    bound is 16 events, clamped to `1..=128`.

- `remove_hardware_binding(bindingId) -> SessionDocument`
  - Existing command. Removes durable binding records from `SessionDocument`.

## Diagnostics

Diagnostics use `HardwareRuntimeDiagnosticCode` and should include concise UI copy,
recoverability, and optional technical detail.

- `no_midi_ports`: MIDI enumeration succeeded but no input ports are available.
- `invalid_midi_port_selection`: selected app-scoped MIDI input id is malformed or no
  longer appears in the current enumeration.
- `midi_enumeration_failed`: the platform MIDI API could not enumerate inputs.
- `osc_bind_failed`: OSC UDP bind failed for a configured host/port.
- `osc_port_in_use`: OSC bind failed because the configured port is already in use.
- `listener_restart_required`: settings changed while a listener was active and the
  listener must be restarted before the new config is effective.
- `listener_restarted`: listener restart completed after a configuration change.
- `listener_stopped`: listeners are stopped and no live input is being consumed.
- `listener_start_pending`: command surface exists, but Phase 9.2/9.3 still need to
  wire concrete listener handles.

## Error States

- No MIDI ports: command returns an empty port list, MIDI lifecycle becomes
  `unavailable`, and status records `no_midi_ports`.
- Invalid MIDI selection: settings update fails with a recoverable error and status
  records `invalid_midi_port_selection`.
- OSC bind failure: Phase 9.3 must return `osc_bind_failed`, set OSC lifecycle to
  `error`, and preserve the failed endpoint in status.
- OSC port in use: Phase 9.3 must prefer `osc_port_in_use` when the platform error can
  be identified as address-in-use.
- Listener restart after config changes: update settings records
  `listener_restart_required`; Phase 9.2/9.3 must stop old resources, start new ones,
  attach new receivers to `HardwareInputRouter`, then record `listener_restarted`.

## Non-Goals

- Do not persist runtime handles or OS port identifiers in `SessionDocument`.
- Do not make chat the only hardware authoring surface.
- Do not add advanced MIDI controller scripts, templates, or controller marketplace
  behavior in Phase 9.
- Do not make OSC the canonical graph or binding model.
- Do not add DAW-style automation recording.
- Do not couple hardware input directly to SuperCollider or the visual sidecar; route
  through app-owned session/runtime abstractions.
