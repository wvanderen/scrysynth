# Phase 7.1: SuperCollider Runtime Resource Plan

Task: `td-64eb52`

## Current Runtime Shape

The app already compiles enabled canonical audio graph state into a deterministic `CompiledTopology` in `src-tauri/src/audio/compiler.rs`. `AudioRuntimeManager::start` compiles the topology, marks the session booting, asks `SuperColliderAdapter` to boot `scsynth`, then calls `load_topology`. The adapter currently launches `scsynth` and returns `Ready` without allocating real SC resources.

`SessionDocument` must remain the canonical product model. Raw SuperCollider node IDs, group IDs, and bus indexes are adapter-owned ephemeral runtime state and must not be added to or persisted in `SessionDocument`.

## Compiler Prerequisite

`CompiledNodeKind` currently distinguishes only `Source`, `Effect`, `Mixer`, and `Output`. That is not enough to choose oscillator vs noise or low-pass filter vs delay. Before real SynthDef application, extend the compiled topology so `CompiledNodeLaunch` carries the concrete primitive variant needed by the adapter.

Recommended minimal shape:

```rust
pub enum CompiledNodeKind {
    Source { source_type: AudioSourceType, channel_mode: ChannelMode },
    Effect { effect_type: AudioEffectType, bypassed: bool },
    Mixer { channel_mode: ChannelMode },
    Output { output_type: AudioOutputType, channels: u32 },
}
```

This keeps the runtime adapter independent of the full `SessionDocument` while preserving every v1 primitive decision needed to apply SC resources.

## Stable Adapter Identifiers

The adapter should maintain an in-memory `AppliedPatch` keyed by canonical IDs:

- `patch_id`: stable string returned in `RuntimeAdapterStatus::Ready`, derived from a deterministic topology fingerprint such as `patch-{sha256(topology)}` or, for the first pass, `patch-{node_count}-{bus_count}` if tests only require stability.
- `bus_key`: canonical `CompiledBus.bus_id`.
- `group_key`: canonical `CompiledGroup.group_id`.
- `node_key`: canonical `CompiledNodeLaunch.node_id`.
- `control_key`: canonical `{node_id}:{parameter_id}`.

The adapter maps those keys to SC runtime resources:

- `bus_key -> ScBus { index, channels, bus_type }`
- `group_key -> ScGroup { node_id }`
- `node_key -> ScSynth { node_id, synthdef_name, group_key }`
- `control_key -> ScControl { synth_node_id, parameter_name }`

Use deterministic adapter-owned numeric ID allocation per topology load:

- Reserve audio bus indexes from compiled topology order. The current compiler assigns contiguous indexes starting at 0. For v1, keep that as the adapter-visible logical bus index, but SC hardware output uses bus 0 separately in the output synth.
- Allocate SC group IDs from a high adapter range, starting at `1000`.
- Allocate SC synth node IDs from a high adapter range, starting at `2000`.
- Allocate `/sync` IDs from a monotonic counter, starting at `1`.
- Map `CompiledBus.index` to a private SC audio bus with `sc_bus_index = hardware_audio_bus_count + CompiledBus.index`, where `hardware_audio_bus_count` is at least the configured output channel count plus input channel count. If runtime bus counts are not queried in the first implementation, use `2` as the v1 stereo output offset and document that input buses are unsupported.

These numeric IDs may be reused after `stop`, `panic`, or a full topology reload. They must never be serialized into sessions, action history, routes, macros, or generated TypeScript contracts.

## V1 SynthDefs

Load these exact SynthDef names before creating nodes:

| SynthDef | Purpose |
|---|---|
| `scrysynth_v1_source_oscillator` | Oscillator source into an internal audio bus. |
| `scrysynth_v1_source_noise` | Noise source into an internal audio bus. |
| `scrysynth_v1_fx_lowpass` | Low-pass filter from one input bus to an output bus. |
| `scrysynth_v1_fx_delay` | Delay effect from one input bus to an output bus. |
| `scrysynth_v1_mixer` | Sum one or more input buses into one output bus. |
| `scrysynth_v1_output` | Route an internal bus to SC hardware output channels. |

Parameter names are stable adapter API and should be used in OSC `/s_new` and `/n_set` calls:

| Parameter | Meaning | Default |
|---|---|---:|
| `out_bus` | Output audio bus index for sources/effects/mixers. | required |
| `in_bus` | Primary input audio bus index for effects/output. | required |
| `in_bus_1` through `in_bus_8` | Mixer input bus indexes. Missing inputs use `-1`. | `-1` |
| `input_count` | Number of active mixer inputs. | `0` |
| `hardware_out` | SC hardware output bus index. | `0` |
| `channels` | Output channel count. | `2` |
| `level` | Linear amplitude. | `1.0` |
| `frequency` | Oscillator frequency in Hz. | `220.0` |
| `wave_shape` | Oscillator shape selector: `0` sine, `1` saw, `2` square. | `0` |
| `noise_color` | Noise selector: `0` white, `1` pink. | `0` |
| `cutoff_hz` | Low-pass cutoff in Hz. | `1200.0` |
| `resonance` | Low-pass resonance/rq control. | `0.5` |
| `delay_time_s` | Delay time in seconds. | `0.25` |
| `feedback` | Delay feedback ratio. | `0.25` |
| `mix` | Wet/dry or mixer level ratio depending on SynthDef. | `1.0` |
| `bypassed` | Effect bypass switch, `0.0` false and `1.0` true. | `0.0` |

Canonical `ParameterValue.name` should map by normalized name first, then by known legacy names:

- `level`, `gain`, `amplitude` -> `level`
- `frequency`, `freq` -> `frequency`
- `wave_shape`, `waveShape` -> `wave_shape`
- `noise_color`, `noiseColor` -> `noise_color`
- `cutoff`, `cutoff_hz`, `cutoffHz` -> `cutoff_hz`
- `resonance`, `rq` -> `resonance`
- `delay_time`, `delay_time_s`, `delayTime` -> `delay_time_s`
- `feedback` -> `feedback`
- `mix` -> `mix`

Unknown parameter names should not fail topology load in v1. Log them with `tracing::warn!` and leave them out of `/s_new` and `/n_set`.

## Primitive Mapping

| Compiled node | SC operation |
|---|---|
| `Source { Oscillator, .. }` | Create `scrysynth_v1_source_oscillator` in the node's group with `out_bus`, `level`, `frequency`, and optional `wave_shape`. |
| `Source { Noise, .. }` | Create `scrysynth_v1_source_noise` in the node's group with `out_bus`, `level`, and optional `noise_color`. |
| `Effect { LowPassFilter, bypassed }` | Create `scrysynth_v1_fx_lowpass` with first `input_bus_ids[0]` as `in_bus`, `output_bus_id` as `out_bus`, mapped cutoff/resonance/mix params, and `bypassed`. |
| `Effect { Delay, bypassed }` | Create `scrysynth_v1_fx_delay` with first `input_bus_ids[0]` as `in_bus`, `output_bus_id` as `out_bus`, mapped delay/feedback/mix params, and `bypassed`. |
| `Mixer { .. }` | Create `scrysynth_v1_mixer` with up to eight sorted `input_bus_ids`, `input_count`, `out_bus`, and optional `level`/`mix`. |
| `Output { Master, channels }` | Create `scrysynth_v1_output` with first `input_bus_ids[0]` or `output_bus_id` as `in_bus`, `hardware_out = 0`, and `channels`. |
| `Output { Cue, channels }` | Create `scrysynth_v1_output` with first `input_bus_ids[0]` or `output_bus_id` as `in_bus`, `hardware_out = channels` for v1 cue separation, and `channels`. |

For effects and output nodes with multiple input buses, v1 uses the first sorted bus and emits a warning. Mixing multiple inputs belongs in an explicit mixer node. A missing required bus is a topology application failure, not a silent ready state.

## Apply Order

`SuperColliderAdapter::load_topology` should be all-or-failed from the session point of view:

1. Confirm `scsynth` process is running. If not, return `RuntimeAdapterStatus::Failed`.
2. Send or load all v1 SynthDefs.
3. `/sync` and wait for acknowledgement. Failure returns `Failed`.
4. Build in-memory bus map from `CompiledTopology.buses`; validate unique bus IDs, positive channel counts, and bus lookup for every node.
5. Create SC groups in `CompiledTopology.group_order` order using `/g_new`.
6. `/sync` and wait.
7. Create synth nodes in `CompiledTopology.node_launch_order` order using `/s_new`, target group IDs, and add action `1` (`addToTail`) so graph order remains deterministic inside each group.
8. Record `control_key -> ScControl` for every recognized parameter applied to each synth.
9. `/sync` and wait.
10. Atomically replace the adapter's active `AppliedPatch`.
11. Return `RuntimeAdapterStatus::Ready { active_patch_id }`.

Route/control wiring is therefore expressed by bus arguments during `/s_new` for the initial v1 path. Live parameter updates should use `/n_set` against the adapter's `control_key` map. Live reroutes can initially trigger a full `load_topology` reapply; incremental bus rewiring is a later optimization.

## Sync And Failure Behavior

Each `/sync` must use a unique sync ID and a bounded timeout. Recommended initial timeout: 2 seconds per sync stage. On timeout, unexpected OSC response, send failure, missing required bus, unknown primitive, or SynthDef load failure:

- Return `RuntimeAdapterStatus::Failed { message }` from the adapter.
- Preserve enough detail in `message` to identify the stage and canonical ID, for example `failed to create synth for node node-fx: missing input bus`.
- Do not mark the session ready.
- Do not partially replace the active `AppliedPatch`.
- If a previous patch was active and the new load fails, leave the previous `AppliedPatch` in memory but report failure so `AudioRuntimeManager` marks the session degraded. A later task can decide whether failed reloads should actively silence old sound.

`panic` remains the hard safety path: terminate `scsynth`, clear the active patch map, clear pending sync waiters, and let `AudioRuntimeManager` record `PanicRecovered`.

## Tests To Add Or Update

- `src-tauri/tests/audio_runtime.rs`: assert the compiler preserves concrete source/effect/output primitive types in `CompiledNodeKind`.
- `src-tauri/tests/audio_runtime.rs`: add an adapter-planning unit test that converts a topology into expected SC operations without launching `scsynth`.
- `src-tauri/tests/audio_runtime.rs`: assert SynthDefs are loaded before groups and synth nodes, and `/sync` gates occur after synthdefs, groups, and node creation.
- `src-tauri/tests/audio_runtime.rs`: assert stable adapter IDs are deterministic for a fixed topology and are not written into `SessionDocument`.
- `src-tauri/tests/audio_runtime.rs`: assert adapter failure during topology load leaves `AudioRuntimeLifecycle::Failed`, `AudioRuntimeHealth::Degraded`, and `RuntimeConnectionState::Error`.
- `src-tauri/tests/audio_graph_commands.rs`: keep cycle and route pruning tests unchanged; add coverage only if compiler primitive enrichment changes route validation behavior.

Prefer testing the resource planner as a pure function before testing real OSC IO. The real `scsynth` integration should stay opt-in or environment-gated so CI does not require an installed SuperCollider runtime.

## Explicit Non-Goals

- No visual runtime changes.
- No agent planning, prompt, or ownership behavior changes.
- No new canonical graph primitives beyond oscillator source, noise source, low-pass filter, delay, mixer, output, buses, groups, and parameter controls.
- No persistence of raw SC node IDs, group IDs, sync IDs, or process-specific bus allocations.
- No arbitrary user-authored SynthDef loading.
- No feedback cycles, audio-rate modulation graph, sidechains, sends, or nested subgraphs.
- No incremental reroute optimizer in the first implementation; full topology reapply is acceptable.
- No DAW transport, recording, timeline, clip launcher, or plugin-host scope.
