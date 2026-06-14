---
phase: 08-real-visual-runtime-path
type: protocol-contract
status: accepted
created: 2026-06-14
task: td-2ca78f
---

# Visual Sidecar Protocol Contract

## Scope

The visual runtime is a separate process named `scrysynth-visual`. The app owns canonical session state and sends compiled visual projections to the sidecar. The sidecar owns renderer state, GPU resources, renderer entity IDs, process IDs, and transport sequence handling. Those runtime-owned fields must not be stored in `SessionDocument`.

The v1 transport is JSON lines over the sidecar process stdio:

- App writes newline-delimited JSON messages to sidecar stdin.
- Sidecar writes newline-delimited JSON messages to stdout.
- Sidecar stderr is reserved for human-readable diagnostics and crash logs.
- Each JSON message is one UTF-8 line ending in `\n`.
- Messages must not contain embedded newlines.

Loopback IPC is intentionally deferred. Stdio JSON lines keep launch, supervision, test fixtures, and future sidecar packaging simple while still giving typed request/response framing.

## Launch Contract

The app resolves the sidecar executable in this order:

1. `SCRYSYNTH_BEVY_PATH`, if set and pointing at a file.
2. `scrysynth-visual` on `PATH`.
3. A future Tauri bundled sidecar path once packaging is wired.

The app launches `scrysynth-visual` with piped stdin, stdout, and stderr. No canonical session data is passed through environment variables. The first app message must be `handshake`.

Readiness timeout:

- The app waits up to `DEFAULT_READY_TIMEOUT_MS` (`3000`) for a `ready` response to handshake.
- Timeout means the visual runtime failed to become ready; the app should mark visual health degraded or error and terminate the process.
- A `protocolMismatch`, invalid JSON line, process exit, or stderr-only failure before `ready` is a startup failure.

Shutdown behavior:

- Normal stop sends `shutdown` with `mode: "graceful"` and waits for `shutdownComplete`.
- Panic recovery sends `shutdown` with `mode: "panic"` when stdin is usable, then terminates the process if it does not exit promptly.
- If the process is already gone, stop is treated as disconnected cleanup.
- The sidecar should release GPU resources before emitting `shutdownComplete`.

## Envelope

All app-to-sidecar messages use this envelope:

```json
{
  "protocolVersion": 1,
  "sequenceId": 1,
  "payload": {
    "type": "handshake",
    "payload": {}
  }
}
```

All sidecar-to-app messages use this envelope:

```json
{
  "protocolVersion": 1,
  "sequenceId": 1,
  "payload": {
    "type": "ready",
    "payload": {}
  }
}
```

`sequenceId` is app-owned transport state for requests. Responses that acknowledge a request echo the same `sequenceId`. Unsolicited runtime events and telemetry omit `sequenceId`.

## App To Sidecar Messages

### `handshake`

First message sent after launch.

Fields:

- `appName`: `"scrysynth"`
- `appVersion`: app version string
- `sessionId`: canonical session ID for diagnostics only
- `capabilities`: supported protocol capabilities

Expected response: `ready` or `error`.

### `loadScene`

Loads a full compiled visual scene snapshot.

Fields:

- `scene.sceneId`: canonical scene ID or empty string for an empty session.
- `scene.backgroundColor`: `[r, g, b, a]` floats.
- `scene.elements`: compiled visual elements.
- `scene.elements[].elementId`: canonical visual element key from the app projection.
- `scene.elements[].elementType`: renderer shape/type hint.
- `scene.elements[].position`: `[x, y]` floats.
- `scene.elements[].scale`: scale float.
- `scene.elements[].parameters[]`: `{ "parameterId": "...", "value": 0.0 }`.

Expected response: `sceneLoaded` or `error`.

### `updateParameters`

Applies a batch of live parameter patches without a full scene reload.

Fields:

- `updates[].elementId`
- `updates[].parameterId`
- `updates[].value`

Expected response: `parameterBatchApplied` or `error`.

### `ping`

Health check from app to sidecar.

Fields:

- `sentAtUnixMs`: app timestamp echoed by the sidecar.

Expected response: `pong`.

### `shutdown`

Requests process shutdown.

Fields:

- `mode`: `"graceful"` or `"panic"`.
- `reason`: optional human-readable reason.

Expected response: `shutdownComplete`, followed by process exit.

## Sidecar To App Messages

### `ready`

Acknowledges handshake and declares the renderer usable.

Fields:

- `renderer`: stable renderer name, for example `"bevy"`.
- `sidecarVersion`: sidecar version string.
- `capabilities`: enabled sidecar capabilities.

### `sceneLoaded`

Acknowledges a scene snapshot.

Fields:

- `sceneId`: loaded scene ID.

### `parameterBatchApplied`

Acknowledges live parameter updates.

Fields:

- `appliedCount`: number of accepted parameter updates.

### `runtimeEvent`

Unsolicited runtime state event.

Fields:

- `level`: `"info"`, `"warning"`, or `"error"`.
- `message`: human-readable event text.
- `sceneId`: optional scene ID associated with the event.

### `telemetry`

Unsolicited renderer metrics.

Fields:

- `sceneId`: optional active scene ID.
- `fps`: current frames per second.
- `frameTimeMs`: optional frame time.

### `error`

Reports a request or runtime failure.

Fields:

- `code`: `protocol_mismatch`, `invalid_message`, `scene_rejected`, `parameter_rejected`, `renderer_unavailable`, or `internal_error`.
- `message`: actionable error detail.
- `recoverable`: optional recovery hint.

### `pong`

Responds to `ping`.

Fields:

- `sentAtUnixMs`: copied from `ping`.

### `shutdownComplete`

Acknowledges shutdown and confirms renderer resources have been released.

Fields:

- `mode`: `"graceful"` or `"panic"`.

## Rust Types

The canonical Rust protocol types live in `src-tauri/src/visual/protocol.rs`. They are serializable and deserializable with `serde` and intentionally sit outside `domain::session`. Use conversions from compiled visual projections rather than adding transport IDs or renderer-owned fields to `SessionDocument`.
