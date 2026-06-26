# Phase 12: Node Catalog Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-26
**Phase:** 12-Node Catalog Foundation
**Areas discussed:** Node granularity, CV/modulation depth, Step sequencer model, v1 session compatibility

---

## Node granularity

### Q1 — Oscillator granularity (test case for the principle)

| Option | Description | Selected |
|--------|-------------|----------|
| Param-driven (v1 style) | One 'Oscillator' node; a waveform param selects sine/saw/square/triangle inside one SynthDef. Fewer catalog entries, richer params. Matches how v1 already does osc + noise. | ✓ |
| Node-per-variant | Separate Sine, Saw, Square, Triangle nodes — each its own catalog entry + SynthDef. Bespoke/VCV Rack feel. | |
| Hybrid | One basic Oscillator (waveform param) PLUS separate specialty oscs (FM, supersaw). | |
| You decide | Let the agent/researcher pick per-family based on DSP fit. | |

**User's choice:** Param-driven (v1 style)
**Notes:** Generalizes the existing v1 osc/noise pattern (wave_shape/noise_color selects internally). Establishes the default principle: one node per family, params carry the variety.

### Q2 — FX family (stress test for the principle)

| Option | Description | Selected |
|--------|-------------|----------|
| Separate FX nodes | One Delay, Reverb, Distortion, Chorus/Flanger node — each its own entry + SynthDef (DSP graphs fundamentally differ). | |
| One FX node, type param | A single 'FX' node with an effect-type param selecting delay/reverb/etc internally. Maximally consistent with osc decision. | |
| You decide | Agent picks: param-driven where one SynthDef holds variants (filter LPF/HPF/BPF), separate where DSP differs (delay vs reverb). | ✓ |

**User's choice:** You decide (delegated to agent)
**Notes:** DSP-fit rule: param-driven where one SynthDef can sensibly hold the variants (filter LPF/HPF/BPF share the *LPF UGen family), separate nodes where DSP fundamentally differs (delay vs reverb).

---

## CV/modulation depth

### Q1 — How deep does CV/modulation go in Phase 12?

| Option | Description | Selected |
|--------|-------------|----------|
| Ports declared, wiring deferred | Phase 12 declares CV/mod ports + route validation; actual audio-rate modulation wiring deferred to Phase 13. | |
| Modulation works end-to-end | A real patch works end-to-end this phase (LFO sweeps filter cutoff through SC). CV inputs map to SC control/audio args via buses NOW. | ✓ |
| Control-rate only this phase | Control-rate modulation works; audio-rate deferred. Pragmatic middle ground. | |

**User's choice:** Modulation works end-to-end
**Notes:** Ambitious — includes audio-rate + control-rate modulation this phase.

### Q2 — Modulation connection model

| Option | Description | Selected |
|--------|-------------|----------|
| Per-param CV ports | Each modulatable parameter gets its own CV-input port (Filter: audio_out + cutoff_cv + resonance_cv). Draw LFO→cutoff_cv. Most modular/explicit. | ✓ |
| One mod-in + target | One 'mod in' port; a setting picks which param it drives. Fewer ports, less clutter. | |
| Param-targeted routes | No extra CV ports — route connects source directly to (target_node, target_parameter_id). Cleanest node bodies. | |

**User's choice:** Per-param CV ports
**Notes:** Fits NODES-05's "ports" language and the graph-native identity. Catalog declares a CV-in port per modulatable param.

### Q3 — Which parameters get a CV-input port?

| Option | Description | Selected |
|--------|-------------|----------|
| Continuous params only | Continuous params (frequency, cutoff, level, feedback…) get CV ports; discrete selectors (wave_shape, noise_color) and toggles (bypass) do not. | ✓ |
| Every parameter | Every parameter without exception gets a CV port. Maximally modular. | |
| Curated subset per node | Catalog author picks the most useful mod targets per node. Fewer ports, opinionated. | |

**User's choice:** Continuous params only
**Notes:** Modulating a selector index or an on/off toggle is unusual and noisy.

---

## Step sequencer model

### Q1 — Where does sequencer logic live + transport binding?

| Option | Description | Selected |
|--------|-------------|----------|
| App-driven (Rust) | Logic lives in Rust app; tracks transport, advances steps, sends gate/CV to SC per step. SC stays 'dumb'. | ✓ |
| SC-internal (demand-rate) | Logic in a SC SynthDef using Duty/Demand/TDuty clocked from SC tempo. App only seeds pattern. | |
| Hybrid (app data, SC clock) | App owns pattern data; lightweight SC trigger SynthDef drives sample-accurate timing. | |

**User's choice:** App-driven (Rust)
**Notes:** On-brand — canonical state + transport live in the app, not the engine (TransportState is already app-owned).

### Q2 — Output shape (mono vs poly)?

| Option | Description | Selected |
|--------|-------------|----------|
| Mono (gate + CV) | One gate + one CV out per step, one voice per step. Classic mono CV/gate. | ✓ |
| Multi-lane CV | One gate + multiple CV outs (pitch + velocity + mod). Richer, more ports. | |
| Poly (multi-voice) | Multiple independent gate+CV pairs. Polyphonic. Most complex. | |

**User's choice:** Mono (gate + CV)
**Notes:** Matches NODES-04's singular "per-step gate/CV"; fewest ports; most legible.

### Q3 — Step count?

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed 16 steps | 16 steps, fixed. Predictable, simplest, standard. Fixed-size array in canonical state. | ✓ |
| Configurable (8/16/32) | Small fixed set of lengths selectable via a param. | |
| Flexible/arbitrary | Arbitrary length per instance. Most powerful, most complex. | |

**User's choice:** Fixed 16 steps
**Notes:** Predictable; simplest control-message shape. Clock division (16th notes over a bar) left to planner.

---

## v1 session compatibility

### Q1 — Must Phase 12 keep loading v1 session files?

| Option | Description | Selected |
|--------|-------------|----------|
| Keep loading v1 (migrate) | Ship a real migration; v1 files auto-upgrade to new catalog identity. | |
| Clean break allowed | Schema bump; old files unsupported. Catalog defines identity from scratch. | ✓ |
| Migration, but cheap | Migration path exists because v1 nodes are a subset of v2. | |

**User's choice:** Clean break allowed
**Notes:** v1.0 shipped days ago; single-user local-first app; blast radius tiny. Catalog need not accommodate old closed-enum audioPrimitive shape.

### Q2 — What happens when a v1 file is opened in the v2 app?

| Option | Description | Selected |
|--------|-------------|----------|
| Friendly error, no load | Clear specific message ("v1 session unsupported in v2"); file not loaded. | ✓ |
| Generic validation error | Old files fail serde/zod with a generic error. | |
| Best-effort import prompt | Offer a one-time best-effort import attempt. | |

**User's choice:** Friendly error, no load
**Notes:** Fits the legibility principle. No silent serde failure, no best-effort import.

---

## the agent's Discretion

- FX/filter/utility per-family split (apply the DSP-fit rule from Node granularity Q2).
- SynthDef authoring toolchain for the ~12-16 new nodes (extend Python byte-writer vs hand-write `.scd` vs hybrid).
- Catalog storage representation (locked as "one Rust-owned table"; exact form is the planner's call).
- Control-bus allocation strategy for modulation wiring.
- Sequencer clock division / retrigger / legato behavior.

## Deferred Ideas

None — discussion stayed within phase scope.
