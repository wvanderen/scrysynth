# Phase 5: Visual Sync & Cross-Modal Control - Research

**Researched:** 2026-04-12
**Domain:** Visual runtime adapter, macro system, MIDI/OSC input binding, runtime health monitoring
**Confidence:** MEDIUM-HIGH

## Summary

Phase 5 extends Scrysynth from an audio-only instrument into a cross-modal audiovisual instrument. The codebase already has four complete phases of infrastructure: a canonical session graph in Rust, a SuperCollider audio adapter with process management, a performance scene/variation system, agent collaboration with ownership, and a React workspace with three views. The existing architecture provides clear adapter patterns, command mutation pipelines, and runtime status tracking that Phase 5 can extend rather than reinvent.

The four requirements (UI-02, CTRL-01, CTRL-04, PERS-02) decompose into three technical pillars: (1) a visual runtime adapter mirroring the SC adapter pattern, (2) an expanded macro system with cross-domain parameter targeting and hardware binding, and (3) a runtime health dashboard aggregating audio, visual, and agent status. The existing `RuntimeStatusRef` type already has `RuntimeKind::Visual` and `RuntimeKind::Agent` entries in the default session, indicating the schema was designed to anticipate this phase.

**Primary recommendation:** Extend the existing adapter/command/store patterns rather than introducing new architectural layers. The SC adapter trait (`AudioRuntimeAdapter`) provides the exact template for a `VisualRuntimeAdapter`. The `MacroDefinition` type needs expansion for cross-domain targeting and MIDI/OSC binding, but its current shape (id, name, target_parameter_ids, range) is a solid foundation. Runtime health is already partially surfaced in the footer pills — it needs a proper panel, not a new data model.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| UI-02 | User can see current runtime health, activity, and error status for the audio runtime, visual runtime, and agent system. | Existing `RuntimeStatusRef` + `AudioRuntimeState` types already model this; need visual/agent runtime state structs, a new `RuntimeHealthPanel` UI component, and Tauri commands for starting/stopping the visual runtime. |
| CTRL-01 | User can create and adjust macros that map one control to multiple audio and visual parameters. | Existing `MacroDefinition` has `target_parameter_ids: Vec<String>` — needs expansion to target visual parameters too (or a unified parameter addressing scheme). Macro CRUD commands and UI editor needed. |
| CTRL-04 | User can bind supported hardware control input through MIDI or OSC learn to v1 macros or performance actions. | `midir` crate (0.10.3) for cross-platform MIDI. New `HardwareBinding` type in domain. MIDI learn state machine in Rust. OSC input via existing `rosc` crate pattern. |
| PERS-02 | User can run a basic visual runtime that responds to shared session events, scenes, or macros without making the visual engine the source of truth. | New `VisualRuntimeAdapter` trait mirroring `AudioRuntimeAdapter`. Bevy sidecar process management. Typed IPC protocol over tokio channels/WebSocket. Visual runtime receives compiled scene descriptions, never owns session state. |
</phase_requirements>

## Codebase Architecture Findings

### Existing Adapter Pattern (High Fidelity Template)

The audio runtime adapter follows a clean trait-based pattern:

```
AudioRuntimeAdapter trait
├── start() → Result<RuntimeAdapterStatus, String>
├── load_topology(&mut self, topology: &CompiledTopology) → Result<...>
├── stop() → Result<...>
└── panic() → Result<...>
```

`AudioRuntimeManager<A>` wraps any adapter implementing this trait, manages lifecycle state transitions on `SessionDocument`, and updates `RuntimeStatusRef` entries. The `SuperColliderAdapter` implementation manages a child process (`std::process::Child`).

**Key pattern:** Adapter owns the process; Manager owns the session state transitions. The `SessionStore` holds the manager and delegates lifecycle calls through `start_audio_runtime()`, `stop_audio_runtime()`, `panic_audio_runtime()`.

**Implication for visual adapter:** A parallel `VisualRuntimeAdapter` trait + `VisualRuntimeManager` + `BevySidecarAdapter` should follow this exact pattern. The `SessionStore` gains `start_visual_runtime()`, `stop_visual_runtime()`, `panic_visual_runtime()`.

### Existing Macro System (Needs Expansion)

Current `MacroDefinition`:
```rust
pub struct MacroDefinition {
    pub id: String,
    pub name: String,
    pub target_parameter_ids: Vec<String>,  // Only audio parameter IDs
    pub range_start: f64,
    pub range_end: f64,
}
```

Current macro resolution (in `performance_command.rs` `apply_macro_override`):
- Takes a `MacroOverride` (macro_id + value 0..1)
- Scales value by macro's range_start/range_end
- Iterates all nodes, finds matching parameter IDs, clamps to min/max
- Direct mutation — no adapter notification

**Gaps for Phase 5:**
1. `target_parameter_ids` is flat strings — no domain prefix (audio vs visual vs transport)
2. No way to target visual parameters
3. No MIDI/OSC binding storage
4. No macro CRUD commands (macros are only mutated through scene overrides)
5. No macro slider/knob UI for live adjustment

### Existing Runtime Status Infrastructure

`SessionDocument` already has:
- `audio_runtime: AudioRuntimeState` — lifecycle, health, sample_rate, block_size, active_patch_id, last_error, panic_recovery_count
- `runtime_status: Vec<RuntimeStatusRef>` — generic per-runtime status with `RuntimeKind::Audio | Visual | Agent`
- Default session seeds two `RuntimeStatusRef` entries: one for Audio, one for Visual

**What's missing:**
- `visual_runtime: VisualRuntimeState` — parallel to `AudioRuntimeState`
- `agent_runtime: AgentRuntimeState` — health/status for the agent system
- A proper UI component for the runtime health dashboard (currently just footer pills in `App.tsx`)

### Existing Command Pipeline

All mutations flow through:
1. Frontend calls Zustand action
2. Zustand calls `session-client.ts` IPC function (with zod validation)
3. IPC invokes Tauri command in `lib.rs`
4. Tauri command acquires `Mutex<SessionStore>`, calls application layer
5. Application layer mutates session via `store.mutate_current()`
6. Updated `SessionDocument` returned to frontend
7. Zustand runs `applySession()` which calls `projectSessionState()`

**Implication:** New macro commands, visual runtime commands, and MIDI binding commands all follow this pipeline. No architectural change needed.

### Existing Tauri Command Surface

Current commands in `lib.rs`: create_default_session, get_current_session, apply_graph_edit, apply_performance_command, save_session_to_path, open_session_from_path, start_audio_runtime, stop_audio_runtime, panic_audio_runtime, send_agent_message, toggle_agent_freeze, reclaim_ownership, approve_pending_action, reject_pending_action.

**New commands needed:**
- `start_visual_runtime`, `stop_visual_runtime`, `panic_visual_runtime`
- `create_macro`, `update_macro`, `remove_macro` (CRUD)
- `set_macro_value` (live performance control)
- `start_midi_learn`, `stop_midi_learn`, `bind_hardware_input`, `remove_hardware_binding`
- `get_runtime_health` (or reuse get_current_session which already returns everything)

## Architecture Patterns

### Recommended Project Structure (Phase 5 Additions)

```
src-tauri/src/
├── domain/session.rs          # Expand: VisualRuntimeState, HardwareBinding, expanded MacroDefinition
├── application/
│   ├── macro_command.rs       # NEW: Macro CRUD + live value commands
│   ├── midi_learn.rs          # NEW: MIDI/OSC learn state machine
│   └── session_store.rs       # Expand: visual runtime lifecycle methods
├── audio/                     # Existing — no changes needed
├── visual/                    # NEW module
│   ├── mod.rs
│   ├── runtime_manager.rs     # VisualRuntimeManager (mirrors audio pattern)
│   ├── adapter.rs             # VisualRuntimeAdapter trait
│   ├── bevy_sidecar.rs        # Bevy process management
│   └── compiler.rs            # Session → visual scene description compiler
├── hardware/                  # NEW module
│   ├── mod.rs
│   ├── midi_input.rs          # midir-based MIDI input polling
│   └── osc_input.rs           # OSC input listener (rosc)
└── lib.rs                     # Register new Tauri commands

src/
├── store/sessionStore.ts      # Expand: macro actions, visual runtime, MIDI learn
├── components/
│   ├── workspace/
│   │   ├── RuntimeHealthPanel.tsx  # NEW: Health dashboard
│   │   ├── MacroEditor.tsx         # NEW: Macro creation/editing
│   │   ├── MacroSlider.tsx         # NEW: Live macro control
│   │   └── MidiLearnOverlay.tsx    # NEW: MIDI learn mode
│   └── performance/               # Expand with macro controls
└── lib/session-client.ts      # Expand: new IPC functions + zod schemas
```

### Pattern 1: Visual Runtime Adapter (Mirror Audio Pattern)

**What:** A `VisualRuntimeAdapter` trait + `VisualRuntimeManager` + `BevySidecarAdapter` that mirrors the existing audio adapter architecture exactly.

**When to use:** All visual runtime lifecycle management.

**Example:**
```rust
// src-tauri/src/visual/adapter.rs
pub trait VisualRuntimeAdapter {
    fn start(&mut self) -> Result<VisualAdapterStatus, String>;
    fn load_scene(&mut self, scene: &CompiledVisualScene) -> Result<VisualAdapterStatus, String>;
    fn update_parameters(&mut self, params: &[VisualParameterUpdate]) -> Result<(), String>;
    fn stop(&mut self) -> Result<VisualAdapterStatus, String>;
    fn panic(&mut self) -> Result<VisualAdapterStatus, String>;
}

// src-tauri/src/visual/bevy_sidecar.rs
pub struct BevySidecarAdapter {
    process: Option<Child>,
    ipc_sender: Option<UnixSocket>, // or WebSocket client
}
```

### Pattern 2: Unified Macro Target Addressing

**What:** A `MacroTarget` enum that distinguishes audio, visual, and transport parameter targets, replacing the flat `target_parameter_ids: Vec<String>`.

**When to use:** Whenever a macro needs to address parameters across domains.

**Example:**
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum MacroTarget {
    AudioParameter { node_id: String, parameter_id: String },
    VisualParameter { element_id: String, parameter_id: String },
    TransportTempo,
}

pub struct MacroDefinition {
    pub id: String,
    pub name: String,
    pub targets: Vec<MacroTarget>,           // replaces target_parameter_ids
    pub range_start: f64,
    pub range_end: f64,
    pub hardware_binding: Option<HardwareBinding>,  // NEW
}
```

### Pattern 3: MIDI Learn State Machine

**What:** A learn-mode state machine in Rust that listens for the next MIDI/OSC message and binds it to a macro or performance action.

**When to use:** When the user activates "learn" mode in the UI.

**Example:**
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct HardwareBinding {
    pub id: String,
    pub source: HardwareSource,
    pub target: BindingTarget,
    pub transform: ValueTransform,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "config", rename_all = "camelCase")]
pub enum HardwareSource {
    MidiCc { channel: u8, controller: u8 },
    MidiNote { channel: u8, note: u8 },
    MidiPitchBend { channel: u8 },
    OscAddress { address: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "config", rename_all = "camelCase")]
pub enum BindingTarget {
    Macro { macro_id: String },
    SceneRecall { scene_id: String },
    TransportPlay,
    TransportStop,
    TransportPanic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ValueTransform {
    pub input_min: f64,
    pub input_max: f64,
    pub output_min: f64,
    pub output_max: f64,
}
```

### Anti-Patterns to Avoid

- **Don't make the Bevy sidecar a library crate inside the Tauri process:** Bevy and Tauri both want to own the event loop and GPU context. They must be separate processes.
- **Don't put visual scene state in the Bevy sidecar's ownership:** The canonical session document owns all scene definitions. The Bevy process receives compiled scene descriptions and renders them — it never writes back scene state.
- **Don't use raw OSC as the visual adapter protocol:** OSC is great for audio but visual scenes need richer typed messages with acknowledgements. Use a typed serde protocol over local IPC (Unix domain socket on Linux/macOS, named pipe on Windows) or localhost WebSocket.
- **Don't expand MacroDefinition with breaking changes without migration:** The session file schema is versioned (CURRENT_SCHEMA_VERSION = 1). Changing MacroDefinition means bumping to version 2 and providing a migration path.

## Standard Stack

### Core (Phase 5 New Dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `midir` | 0.10.3 | Cross-platform MIDI input | Standard Rust MIDI library, inspired by RtMidi, cross-platform (ALSA on Linux, CoreMidi on macOS, WinMM on Windows). Active maintenance. |
| `rosc` | 0.11.4 (already in stack) | OSC input listener for hardware binding | Already recommended in STACK.md for SC communication; reuse for inbound OSC learn. |
| `tokio` | 1.51.1 (already in stack) | Async MIDI/OSC input, IPC with Bevy sidecar | Already recommended in STACK.md; needed for non-blocking hardware input polling and sidecar IPC. |
| `serde_json` | Already in deps | Typed IPC messages to Bevy sidecar | Already in the project; use JSON over local IPC for simplicity in v1. |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `uuid` | Already in deps | Generate IDs for new types (HardwareBinding, etc.) | Already in the project. |
| `ts-rs` | 12.0.1 (already in deps) | TypeScript type generation for new domain types | Already in the project. |
| `thiserror` | 2 (already in deps) | Error types for new command modules | Already in the project. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `midir` | `wmidi` (Rust) | `wmidi` is lower-level; `midir` has better cross-platform device enumeration and callback-based input |
| Unix domain socket IPC | gRPC / Cap'n Proto | Overkill for v1 localhost IPC; JSON over Unix socket is simpler and debuggable |
| JSON over Unix socket | MessagePack over Unix socket | Slightly more efficient but JSON is inspectable and debuggable — more valuable for v1 |
| Bevy sidecar as Tauri sidecar | Standalone binary discovery | Tauri sidecar bundling is the correct approach per STACK.md recommendation |

**New Rust dependency to add:**
```toml
midir = "0.10"
# tokio, rosc, serde, serde_json, ts-rs, thiserror, uuid — already present
```

**No new frontend dependencies needed.** All UI uses existing React, Zustand, Zod, Radix primitives.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| MIDI device enumeration and input | Custom ALSA/CoreMidi/WinMM bindings | `midir` | Cross-platform, handles device hotplug, callback-based input |
| OSC message parsing | Custom UDP packet parser | `rosc` | Already in stack, handles OSC bundles and type tags |
| Process supervision for Bevy | Manual std::process::Child management | Follow existing `SuperColliderAdapter` pattern but with health checks | The SC adapter pattern works but add periodic health check (sidecar should heartbeat) |
| Macro value scaling | Custom math | Existing `apply_macro_override` pattern in `performance_command.rs` | Already handles range scaling and clamping — extend, don't replace |
| Frontend form validation | Custom validators | Zod schemas in `session-client.ts` | Already established pattern |

**Key insight:** The biggest risk is the Bevy sidecar IPC protocol, not any individual library. Start with the simplest protocol (JSON over localhost TCP/WebSocket) and evolve it.

## Common Pitfalls

### Pitfall 1: Bevy and Tauri Event Loop Conflict
**What goes wrong:** Attempting to embed Bevy in the same process as Tauri causes both frameworks to fight over the main thread event loop and GPU context.
**Why it happens:** Both Bevy (wgpu-based) and Tauri (WebView-based) want to own the rendering pipeline.
**How to avoid:** Run Bevy as a completely separate process (sidecar). Communication only through IPC.
**Warning signs:** GPU initialization failures, deadlocks on startup, window creation errors.

### Pitfall 2: MIDI Input Thread Starvation
**What goes wrong:** MIDI callbacks arrive on a background thread but session mutations happen under a `Mutex<SessionStore>`, causing contention or deadlocks.
**Why it happens:** MIDI input is real-time; session mutations are synchronous under lock.
**How to avoid:** MIDI input should post events to a `tokio::mpsc` channel. A tokio task consumes events and applies mutations asynchronously. Never hold the store lock in a MIDI callback.
**Warning signs:** Dropped MIDI events, UI freezes when moving MIDI controllers.

### Pitfall 3: MacroDefinition Schema Migration Breaking Existing Sessions
**What goes wrong:** Changing `target_parameter_ids: Vec<String>` to `targets: Vec<MacroTarget>` breaks deserialization of existing `.scrysynth-session.json` files.
**Why it happens:** No schema migration path; version check exists but no transformation logic.
**How to avoid:** Either (a) add the new `targets` field alongside `target_parameter_ids` with `#[serde(default)]` and deprecate the old field, or (b) implement a migration from schema version 1 → 2 that transforms old macro definitions.
**Warning signs:** Existing sessions fail to load after the schema change.

### Pitfall 4: Visual Sidecar Becomes Source of Truth
**What goes wrong:** The Bevy sidecar starts tracking its own state (which scene is active, which parameters are set) and the canonical session drifts from what's rendered.
**Why it happens:** Natural tendency to let the rendering engine own its state.
**How to avoid:** The visual sidecar is strictly a consumer. It receives scene descriptions, renders them, and reports health/telemetry. It never reports "current visual state" back as authoritative. All state changes go through the canonical session document.
**Warning signs:** Sidecar sends state update messages; visual state diverges after disconnect/reconnect.

### Pitfall 5: Non-Backward-Compatible Bevy Sidecar Protocol
**What goes wrong:** Updating the IPC protocol version breaks communication with a running sidecar.
**Why it happens:** No protocol versioning or handshake.
**How to avoid:** Include a protocol version in the initial handshake between Tauri app and Bevy sidecar. Reject mismatched versions gracefully.
**Warning signs:** Sidecar starts but visuals don't update; silent deserialization failures.

## Code Examples

### Visual Runtime Adapter Trait (Mirroring Audio Pattern)
```rust
// Source: Derived from existing AudioRuntimeAdapter pattern in runtime_manager.rs
pub trait VisualRuntimeAdapter {
    fn start(&mut self) -> Result<VisualAdapterStatus, String>;
    fn load_visual_scene(
        &mut self,
        scene: &CompiledVisualScene,
    ) -> Result<VisualAdapterStatus, String>;
    fn update_visual_parameters(
        &mut self,
        updates: &[VisualParameterUpdate],
    ) -> Result<(), String>;
    fn stop(&mut self) -> Result<VisualAdapterStatus, String>;
    fn panic(&mut self) -> Result<VisualAdapterStatus, String>;
    fn health_check(&mut self) -> Result<VisualAdapterHealth, String>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum VisualAdapterStatus {
    Booted { renderer: String },
    SceneLoaded { scene_id: String },
    Stopped,
    Panicked,
    Failed { message: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct VisualAdapterHealth {
    pub fps: f32,
    pub gpu_usage_percent: Option<f32>,
    pub is_responsive: bool,
}
```

### Visual Runtime State (Mirroring AudioRuntimeState)
```rust
// Source: Derived from existing AudioRuntimeState pattern in session.rs
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct VisualRuntimeState {
    pub lifecycle: VisualRuntimeLifecycle,
    pub health: VisualRuntimeHealth,
    pub active_scene_id: Option<String>,
    pub fps: Option<f32>,
    pub last_error: Option<String>,
    pub renderer: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum VisualRuntimeLifecycle {
    #[default]
    Idle,
    Starting,
    Ready,
    Rendering,
    Failed,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum VisualRuntimeHealth {
    #[default]
    Unknown,
    Healthy,
    Degraded,
    Error,
}
```

### Compiled Visual Scene (What Gets Sent to Bevy)
```rust
// Source: Derived from existing CompiledTopology pattern in compiler.rs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledVisualScene {
    pub scene_id: String,
    pub background: VisualBackground,
    pub elements: Vec<CompiledVisualElement>,
    pub parameter_defaults: Vec<(String, f64)>,  // (param_id, default_value)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledVisualBackground {
    pub color: [f32; 4],  // RGBA
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledVisualElement {
    pub element_id: String,
    pub element_type: VisualElementType,
    pub position: [f32; 2],
    pub scale: f32,
    pub parameters: Vec<CompiledVisualParameter>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VisualElementType {
    Shape { shape_kind: String },
    ParticleSystem { max_particles: u32 },
    Waveform { source_node_id: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledVisualParameter {
    pub parameter_id: String,
    pub value: f64,
    pub min_value: f64,
    pub max_value: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualParameterUpdate {
    pub parameter_id: String,
    pub value: f64,
}
```

### MIDI Learn Integration (tokio Channel Pattern)
```rust
// Source: midir documentation pattern + tokio channel
use midir::{MidiInput, MidiInputConnection};
use tokio::sync::mpsc;

pub enum MidiLearnEvent {
    MidiCc { channel: u8, controller: u8, value: u8 },
    MidiNote { channel: u8, note: u8, velocity: u8 },
    MidiPitchBend { channel: u8, value: u16 },
}

pub struct MidiInputManager {
    connection: Option<MidiInputConnection<()>>,
    event_sender: mpsc::UnboundedSender<MidiLearnEvent>,
}

impl MidiInputManager {
    pub fn new(sender: mpsc::UnboundedSender<MidiLearnEvent>) -> Self {
        Self {
            connection: None,
            event_sender: sender,
        }
    }

    pub fn start_listening(&mut self) -> Result<(), String> {
        let midi_in = MidiInput::new("scrysynth").map_err(|e| e.to_string())?;
        let port = midi_in.ports().first()
            .ok_or("No MIDI input ports found".to_string())?
            .clone();

        let sender = self.event_sender.clone();
        self.connection = Some(midi_in.connect(
            &port,
            "scrysynth-input",
            move |_timestamp, message, _| {
                if let Some(event) = parse_midi_message(message) {
                    let _ = sender.send(event);
                }
            },
            (),
        ).map_err(|e| e.to_string())?);

        Ok(())
    }
}

fn parse_midi_message(message: &[u8]) -> Option<MidiLearnEvent> {
    if message.is_empty() { return None; }
    let status = message[0];
    let channel = status & 0x0F;
    match status & 0xF0 {
        0xB0 if message.len() >= 3 => Some(MidiLearnEvent::MidiCc {
            channel, controller: message[1], value: message[2],
        }),
        0x90 if message.len() >= 3 => Some(MidiLearnEvent::MidiNote {
            channel, note: message[1], velocity: message[2],
        }),
        0xE0 if message.len() >= 3 => Some(MidiLearnEvent::MidiPitchBend {
            channel, value: ((message[2] as u16) << 7) | message[1] as u16,
        }),
        _ => None,
    }
}
```

### Macro Command Pattern (Following Existing graph_edit.rs Pattern)
```rust
// Source: Derived from apply_graph_edit pattern in graph_edit.rs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum MacroCommand {
    CreateMacro { definition: MacroDefinition },
    UpdateMacro { macro_id: String, name: Option<String>, targets: Option<Vec<MacroTarget>> },
    RemoveMacro { macro_id: String },
    SetMacroValue { macro_id: String, value: f64 },
}

pub fn apply_macro_command(
    store: &mut SessionStore,
    command: MacroCommand,
) -> Result<SessionDocument, MacroCommandError> {
    store.mutate_current(|session| match command {
        MacroCommand::CreateMacro { definition } => {
            if session.macros.iter().any(|m| m.id == definition.id) {
                return Err(MacroCommandError::DuplicateMacro { macro_id: definition.id });
            }
            session.macros.push(definition);
            session.macros.sort_by(|a, b| a.id.cmp(&b.id));
            Ok(())
        }
        MacroCommand::SetMacroValue { macro_id, value } => {
            apply_macro_value(session, &macro_id, value)
        }
        // ... other variants
    })
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Custom Bevy IPC via raw TCP | JSON-RPC over localhost WebSocket | Standard practice 2024+ | Simpler debugging, broader language support |
| MIDI via `portmidi` bindings | `midir` crate | midir has been standard since ~2020 | Better cross-platform, active maintenance |
| Embedding renderer in WebView | Separate native process | Industry consensus for GPU-heavy rendering | Avoids GPU context conflicts, better performance isolation |
| Flat parameter ID lists | Tagged enum parameter addressing | Pattern from DAW macro systems | Enables cross-domain targeting without string conventions |

**Deprecated/outdated:**
- `portmidi-rs`: Superseded by `midir` for new projects
- Rendering in WebView via WebGL: Fine for simple visuals but not for a dedicated visual runtime

## Open Questions

1. **Bevy sidecar binary distribution**
   - What we know: Tauri supports sidecar bundling. Bevy compiles to a native binary.
   - What's unclear: How large the Bevy binary will be; whether it should be bundled or user-installed for v1.
   - Recommendation: For v1, require the Bevy sidecar binary to be pre-built and placed next to the app binary (or in a configurable path). Bundling can be added later. The visual runtime is optional — the app should work without it.

2. **Visual scene compilation scope for v1**
   - What we know: The existing audio compiler (`compiler.rs`) is sophisticated — it does topological sort, bus allocation, and validation. The visual compiler needs similar rigor.
   - What's unclear: How complex visual scenes need to be in v1. Minimum viable: background color + a few shape elements + parameter mapping. No visual graph editing in v1 (deferred to VIS-01, VIS-02).
   - Recommendation: Keep the v1 visual scene model very simple — background, N visual elements (shapes, particle system), each with a few parameters (position, scale, color, intensity). The visual compiler maps session macros and transport state to these parameters.

3. **MIDI input thread safety with Mutex<SessionStore>**
   - What we know: SessionStore is behind a `Mutex` in Tauri state. MIDI callbacks come from a foreign thread (midir's callback).
   - What's unclear: Whether `tokio::sync::mpsc` is the right bridge or whether `std::sync::mpsc` is simpler since the MIDI callback is not async.
   - Recommendation: Use `std::sync::mpsc` from the MIDI callback to a tokio task that bridges to the store. Simpler than trying to make midir's callback async-safe.

4. **Agent runtime health status**
   - What we know: UI-02 requires showing agent system health. The current agent system is a simple text parser, not a running service.
   - What's unclear: What "agent health" means for a stateless parser. It's always "available" unless the intent parser crashes.
   - Recommendation: Agent health is derived from the session state (is agent frozen? are there pending actions?) rather than a separate runtime check. Add `AgentRuntimeState` with simple fields: `is_available: bool`, `pending_action_count: u32`, `is_frozen: bool`.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Core build | ✓ | 1.94.1 | — |
| Node.js | Frontend build | ✓ | 24.13.0 | — |
| Bun | Package manager | ✓ | 1.3.11 | npm |
| scsynth | Audio runtime adapter | ✓ | On PATH | Graceful failure message |
| midir (crate) | MIDI input | ✗ (needs install) | 0.10.3 | N/A — add to Cargo.toml |
| Bevy binary | Visual sidecar | ✗ | — | Visual features gracefully disabled |
| MIDI hardware | CTRL-04 testing | Unknown | — | Test with virtual MIDI device |

**Missing dependencies with no fallback:**
- `midir` crate: Must be added to Cargo.toml (build-time dependency, not a runtime installation)
- Bevy sidecar binary: Must be built separately for visual features. App should work without it (visual runtime shows "disconnected" status).

**Missing dependencies with fallback:**
- MIDI hardware: Use `aconnect` (ALSA) virtual MIDI ports for testing on Linux, or test without MIDI
- Bevy sidecar: Visual runtime status shows "disconnected"; all other features work

## Validation Architecture

> Skipping per config.json: `"nyquist_validation": false`

## Sources

### Primary (HIGH confidence)
- Codebase analysis: All Rust source files in `src-tauri/src/`, all TypeScript source in `src/`
- Existing adapter pattern: `audio/runtime_manager.rs`, `audio/supercollider.rs`, `audio/compiler.rs`
- Domain model: `domain/session.rs` — complete type definitions
- Tauri command surface: `lib.rs` — all registered commands
- Frontend store: `store/sessionStore.ts`, `store/session-projections.ts`

### Secondary (MEDIUM confidence)
- `midir` crate (0.10.3): Cross-platform MIDI standard for Rust, verified on crates.io
- Bevy sidecar architecture: Based on STACK.md recommendations and Tauri sidecar documentation
- MIDI message parsing: Standard MIDI spec (status byte channel encoding)

### Tertiary (LOW confidence)
- Bevy IPC protocol specifics: Recommended JSON over local socket/WebSocket but actual Bevy integration untested in this project — needs validation during implementation
- Visual scene model complexity: Minimal v1 scope assumed based on deferred VIS-01/VIS-02 requirements

## Metadata

**Confidence breakdown:**
- Adapter pattern for visual runtime: HIGH — directly mirrors existing SC adapter with zero guesswork
- Macro system expansion: HIGH — current MacroDefinition is well-structured, needs additive changes
- MIDI input binding: MEDIUM-HIGH — midir is proven but thread-safety bridge needs implementation validation
- Bevy sidecar IPC: MEDIUM — pattern is clear but actual Bevy integration has no prior art in this codebase
- Runtime health UI: HIGH — existing RuntimeStatusRef already models this, just needs a UI component

**Research date:** 2026-04-12
**Valid until:** 2026-05-12 (stable domain, no fast-moving dependencies)

## Recommended Plan Breakdown

### Option A: 3 Plans (Recommended)

**Plan 1: Visual Runtime Adapter + Runtime Health Dashboard**
- Add `VisualRuntimeState` to domain types
- Create `visual/` module with adapter trait, manager, bevy_sidecar, compiler
- Add visual runtime lifecycle commands to Tauri and SessionStore
- Build `RuntimeHealthPanel` UI component (audio + visual + agent status)
- Expand `AgentRuntimeState` for agent health
- Schema migration v1 → v2 if needed
- Requirements: PERS-02, UI-02 (partial)

**Plan 2: Cross-Modal Macro System**
- Expand `MacroDefinition` with `MacroTarget` enum for audio + visual + transport targets
- Create `macro_command.rs` with CRUD + live value commands
- Build `MacroEditor` and `MacroSlider` UI components
- Integrate macro value changes with visual adapter (parameter updates)
- Macro-aware scene recall (macros trigger visual scene updates too)
- Requirements: CTRL-01

**Plan 3: Hardware Input Binding (MIDI/OSC Learn)**
- Add `HardwareBinding` type to domain
- Create `hardware/` module with midir MIDI input + OSC input listener
- Implement MIDI learn state machine (start/stop learn, capture next event, bind)
- Build `MidiLearnOverlay` UI component
- Wire hardware input events through tokio channel → macro value commands
- Add hardware bindings to session persistence
- Requirements: CTRL-04

### Option B: 2 Plans (Compressed)

**Plan 1: Visual Runtime + Macros + Runtime Health**
- Merge Option A's Plans 1 and 2 into one larger plan
- Risk: Larger surface area per plan, but macros and visuals are deeply intertwined

**Plan 2: Hardware Input Binding**
- Same as Option A's Plan 3

### Recommendation
**Option A (3 plans)** is preferred because:
1. Visual adapter infrastructure is independent of macros — can be built and tested standalone
2. Macro expansion touches domain types, persistence, and UI — deserves its own focused plan
3. MIDI/OSC input is the most technically risky part (thread safety, cross-platform) — deserves isolation
4. Each plan has a clear completion gate matching one or two specific success criteria
