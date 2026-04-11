# Feature Landscape

**Domain:** Graph-native desktop co-creative human/AI audiovisual instrument for live performance
**Project:** Scrysynth
**Researched:** 2026-04-11
**Overall confidence:** MEDIUM

## Recommendation Stance

This category sits between modular patching tools, live performance environments, and newer AI co-creation interfaces. Users will expect the reliability and directness of a live instrument first, visible signal flow second, and AI assistance only if it stays legible, reversible, and subordinate to performance control. That means v1 should ship a small number of deeply dependable features rather than broad studio or media-server scope.

## Table Stakes

Features users will reasonably expect in v1. Missing any of the first six makes the product feel underpowered or unsafe on stage.

| Feature | What user expects | Why expected in this category | Complexity | Dependencies | Confidence |
|---------|-------------------|-------------------------------|------------|--------------|------------|
| Canonical live session graph | A visible patch showing nodes, routes, buses, macros, scenes, and current state | Graph-native tools like Max, TouchDesigner, and VCV Rack make signal flow inspectable and editable in real time | High | Session model; graph renderer; runtime adapters | HIGH |
| Reliable low-latency audio execution | Immediate sonic response, stable timing, no audio dropouts during edits and performance | Live instruments are judged first on playability and timing, not feature count | High | SuperCollider adapter; transport; scheduler; device I/O | HIGH |
| Direct performance control surface | Macros, knobs, scene triggers, mutes, fades, and quick-access controls separate from graph editing | Ableton Live and Resolume both center hands-on performance control and fast manipulation | Medium | Canonical graph; binding system; controller mapping | HIGH |
| Save/load session state with fast recall | Open a patch, resume exact state, and recover quickly after interruption | Patch-based instruments are expected to persist routings, parameter state, and reusable setups | Medium | Session serialization; asset references; runtime rehydration | HIGH |
| Scene and variation recall | Trigger prepared states and switch musical/visual variations without rebuilding patches live | Performers expect recallable states, especially for improvisation and set structure | Medium | Session graph; macros; transition rules | MEDIUM |
| MIDI/OSC and hardware mapping | Bind external controllers to key parameters with low friction | Live AV tools routinely support controller mapping for tactile performance | Medium | Binding model; input abstraction; learn mode | HIGH |
| Basic audiovisual synchronization | Audio and visual engines stay coherently timed via shared transport/events | AV performers expect sound and image to move together even if engines are separate | High | Shared event bus; clock; scene model; visual adapter | HIGH |
| Conversational agent anchored to session state | Ask for changes in natural language and have the system act on the current graph, not in a vacuum | The "co-creative AI instrument" framing makes this part of the product promise, not a bonus | High | Canonical graph; agent action schema; conversation history | MEDIUM |
| Reversible edits and safety recovery | Undo/redo, cancel, panic/stop-all, and easy reclaim of control from the agent | Live tools need safety valves; AI adds extra need for reversibility and override | Medium | Event log; ownership model; transport safety; runtime recovery | MEDIUM |
| Inspectable runtime feedback | Metering, activity indicators, error surfacing, and current ownership/action status | Patching systems are expected to show what is happening right now, not hide it | Medium | Runtime telemetry; graph annotations; diagnostics UI | HIGH |

## Differentiators

Features that can create durable advantage if executed well. These should not all land in initial MVP, but the product architecture should preserve room for them.

| Feature | Value proposition | Why it matters here | Complexity | Dependencies | Confidence |
|---------|-------------------|---------------------|------------|--------------|------------|
| Shared ownership model for human and agent | Every node, macro, scene, or route has explicit ownership, delegation, and reclaim behavior | This turns AI from "assistant panel" into a real co-performer without sacrificing trust | High | Canonical graph; policy model; UI status language; undo/event log | MEDIUM |
| Explainable agent mutations | Each agent action names what changed, why, and where in the graph it landed | Makes AI behavior legible in a live setting and differentiates from black-box generators | Medium | Structured action schema; graph diffs; conversation linkage | MEDIUM |
| Mixed-initiative scene and variation generation | Human can request proposals, accept parts, reject parts, or ask for alternates at graph/scene level | Fits iterative collaboration better than one-shot content generation | High | Scene model; primitive library; agent planner; reversible actions | MEDIUM |
| Conversation-graph-performance triad | Chat, graph, and control surface stay linked to the same live object model | Many tools do one or two of these well; integrating all three cleanly is distinctive | High | Canonical session model across all views | HIGH |
| Stable primitive-based mutation library | Agent works through safe musical/visual primitives instead of arbitrary low-level rewrites | Preserves explainability, reliability, and stylistic consistency under performance pressure | High | Primitive taxonomy; validation rules; runtime capability map | HIGH |
| Cross-modal macros and scenes | One gesture or scene change can coordinate audio, visuals, and control-state together | Creates a true audiovisual instrument rather than separate audio and video tools taped together | High | Shared abstraction layer; scene graph; synchronized runtime adapters | HIGH |
| Intent-aware approvals | High-impact actions ask for confirmation; low-risk actions can execute immediately | Makes co-creation feel fluid without making the performer nervous | Medium | Risk classification; ownership rules; action queue | MEDIUM |
| Rehearsal memory and iterative branch workflow | Save branches, compare variations, and promote winning versions into the live set | Supports repeated human/AI iteration without turning the product into a DAW | Medium | Session history; variation graph; diff/merge UX | LOW |

## Anti-Features

Tempting additions that should be deliberately excluded from early phases.

| Anti-Feature | Why avoid early | What to do instead |
|--------------|-----------------|--------------------|
| Full DAW arrangement and editing workflow | Pulls the product toward timeline production, asset management, and offline editing instead of live playability | Keep v1 performance-native with scenes, variations, macros, and recallable session state |
| Unrestricted plugin marketplace or arbitrary user code everywhere | Blows up reliability, support burden, security surface, and agent action complexity before the core model is proven | Support a small built-in primitive set plus carefully bounded adapters |
| Multiplayer/network collaboration | Adds sync, conflict resolution, permissions, and latency problems that obscure the single-performer instrument thesis | Stay local and single-user in v1; design state/event model so collaboration can be explored later |
| Deep projection mapping / media-server scope | Resolume-class output routing, mapping, and venue infrastructure is a large separate product area | Provide basic visual runtime sync and simple output control only |
| Autonomous low-level DSP self-rewriting | Hard to trust, debug, explain, or recover from in a live set | Keep mutations at validated primitive and graph-operation level |
| Built-in model training or fine-tuning workflows | Expensive, slow, and orthogonal to proving the instrument UX | Treat AI as orchestration/planning/mutation logic over session primitives |
| Cloud-first sessions or online dependency for core play | Undermines stage reliability and conflicts with desktop-local product framing | Make the core loop fully local; optional network features can come later |
| Huge starter content marketplace | Easy to build noise instead of a coherent instrument identity | Ship a focused starter library of exemplary patches, scenes, and mappings |

## Complexity Notes

| Area | Notes for requirements definition |
|------|----------------------------------|
| Graph editing depth | Full freeform graph authoring is expensive; v1 should prioritize graph inspection plus bounded editing over unlimited patch construction |
| Agent collaboration | The hard part is not prompting; it is action scoping, explainability, reversibility, and ownership recovery under live conditions |
| Visual runtime | Basic sync is table stakes, but deep visual authoring can consume the roadmap if not bounded behind a thin adapter and a small primitive set |
| Safety UX | Undo alone is not enough; requirements should include panic, stop-all, mute-safe defaults, and visible ownership state |
| Scene transitions | Simple hard cuts and timed fades are cheap; musically coherent multi-engine transitions are materially harder and should be staged |
| Preset/primitives design | A small, composable primitive vocabulary is more important than a large catalog; too many primitives weaken both UX and agent reliability |
| Hardware integration | Controller learn and stable mappings are relatively standard; feedback, motorized state sync, and advanced device profiles can wait |

## Feature Dependencies

```text
Canonical session graph -> graph view, session persistence, agent action schema, scene/variation model
SuperCollider runtime adapter -> reliable audio execution -> performance controls -> stage-safe playback
Canonical session graph + binding model -> MIDI/OSC learn + macros
Canonical session graph + event bus -> audiovisual synchronization -> cross-modal scenes/macros
Canonical session graph + conversation history -> session-anchored agent collaboration
Structured agent actions -> explainable mutations -> intent-aware approvals -> shared ownership model
Event log + ownership model -> undo/redo -> safety recovery -> trustworthy live co-creation
Scene model -> variation generation -> rehearsal memory/branch workflow
```

## MVP Recommendation

Prioritize in this order:

1. Canonical live session graph with inspectable state
2. Reliable audio runtime, transport, and panic-safe performance controls
3. Scene/variation recall with save/load sessions
4. MIDI/OSC controller mapping and macros
5. Session-anchored conversational agent using a small safe primitive set
6. Basic visual runtime synchronization through shared events, not deep visual patching

Defer until after core validation:

- Shared ownership at fine granularity: valuable, but only after the basic agent action model is proven
- Rehearsal branching and compare/merge workflows: useful for iteration, not required to validate live co-creation
- Projection mapping, advanced output routing, plugin ecosystem, multiplayer, and DAW features: each is a roadmap fork, not a v1 requirement

## Sources

- Project context: `/home/lem/dev/scrysynth/.planning/PROJECT.md`
- Ableton Live overview (Session View, Macros, controller mapping, performance workflow): https://www.ableton.com/en/live/what-is-live/ (HIGH)
- Cycling '74 Max overview (visual patching, UI control, realtime audio/visual interaction): https://cycling74.com/products/max (HIGH)
- TouchDesigner overview (node-based, always-live procedural authoring, inspectable realtime operators): https://docs.derivative.ca/TouchDesigner (MEDIUM; page content is older but still useful for core interaction model)
- Resolume Avenue & Arena overview (live video mixing, AV playback, FFT audio analysis, MIDI/OSC/DMX control): https://resolume.com/software/avenue-arena (HIGH)
- Resolume Wire overview (node-based visual patching, BPM sync, MIDI/OSC, plugin authoring): https://resolume.com/software/wire (HIGH)
- VCV Rack manual and overview (modular patching basics, signal categories, controller/audio I/O expectations): https://vcvrack.com/manual/GettingStarted and https://vcvrack.com/ (HIGH)
- SuperCollider overview (realtime audio engine, client/server structure, suitability as execution runtime): https://supercollider.github.io/ (HIGH)

## Confidence Notes

- HIGH confidence on live-instrument, graph, controller, sync, and persistence expectations because they are well-established across adjacent official tools.
- MEDIUM confidence on AI-specific expectations because there is not yet a mature, stable product category for "co-creative human/AI live AV instrument"; recommendations are extrapolated from adjacent live tools plus the project thesis.
- LOW confidence only on rehearsal-memory branch workflow as a differentiator; it is promising, but not strongly validated by the surveyed ecosystem.
