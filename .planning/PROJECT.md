# Scrysynth

## What This Is

Scrysynth is a graph-native desktop audiovisual instrument for live co-creation between a human performer and one or more AI agents. It is built as a Tauri application centered on visible signal flow, live control surfaces, and shared performance control rather than a DAW-style production workflow, terminal wrapper, or one-shot generator.

## Core Value

The instrument must let a human and agent shape a live audiovisual session together through conversation, graph structure, and direct performance control without losing legibility or human override.

## Current Stage

Scrysynth is in **v1 runtime hardening**.

The foundation phase is complete: the codebase now has a Rust-owned canonical session graph, generated TypeScript contracts, JSON persistence, graph/conversation/performance UI surfaces, command handlers, ownership and approval scaffolding, runtime health state, macro controls, visual adapter scaffolding, and MIDI/OSC binding models.

It is not yet release-ready. The remaining v1 work is to make runtime paths real and locally verifiable:

- SuperCollider must receive real synth/resource/bus/node/parameter OSC operations rather than only lifecycle state.
- The separate visual runtime needs an actual sidecar binary and typed protocol.
- Hardware listeners must be started/configured and wired into the desktop app runtime.
- Agent behavior must move beyond deterministic keyword parsing while staying constrained by typed commands and approval gates.
- Packaging, UAT, and setup diagnostics need to be completed.

## Requirements

### Validated Foundation

- [x] A canonical session graph models audio, visual, control, scene, variation, ownership, runtime, pending-action, history, and hardware-binding state inside the app.
- [x] The desktop UI supports linked conversation, graph, and performance views over the same live session.
- [x] The command layer supports graph edits, scene/variation recall, macro CRUD, agent approvals, ownership controls, runtime health, and hardware binding models.

### Active Runtime Hardening

- [ ] A new developer can install and run local checks from documented setup instructions.
- [ ] A v1 runtime foundation executes actual audio through SuperCollider.
- [ ] A separate visual sidecar receives and applies scene/control updates.
- [ ] MIDI/OSC learn works from live desktop runtime listeners.
- [ ] Agent collaboration is session-aware, explainable, and constrained by human override.

### Out of Scope

- Multiplayer collaboration — v1 is explicitly local and single-user.
- Full DAW-style production workflow — the product is performance-native, not a studio replacement.
- Browser-first deployment — desktop Tauri is the intended v1 shell.
- Unrestricted plugin ecosystem — would expand surface area before the core instrument model is proven.
- Highly autonomous low-level DSP self-rewriting — the system should favor explainable primitive-based mutation.

## Context

The project started from a greenfield foundation document describing an audiovisual co-creation instrument distinct from Mindrave. Mindrave remains terminal-native, text-first, and algorave/live-code oriented; Scrysynth is GUI-first, graph-native, and oriented around inhabitable patch space, live control, and readable agent behavior.

The product is organized around three equally important interaction modes: conversational direction, graph inspection/manipulation, and direct performance control.

The app owns canonical session truth. Audio and visual engines are runtime targets rather than the source of semantic state. SuperCollider remains the recommended v1 audio execution engine, while visuals stay behind a separate adapter. Audiovisual coherence should come from shared session abstractions, macros, scenes, transport, and runtime feedback rather than coupling all behavior to one engine.

## Constraints

- **Platform**: Desktop Tauri application — v1 is local and single-user.
- **Product Identity**: Must remain distinct from Mindrave — avoid terminal-first interaction patterns and "Mindrave with windows" framing.
- **Architecture**: Canonical session state lives in the app — runtime adapters consume and report state rather than own authorship.
- **Audio Runtime**: SuperCollider is the recommended v1 audio engine — use it for execution, not as the total product model.
- **Visual Runtime**: Visuals must run through a separate adapter — audiovisual coherence comes from shared session abstractions.
- **Interaction Model**: Conversation, graph, and performance control are co-equal — chat cannot become the only authoring surface.
- **Control Safety**: Human override must stay easy and reliable — shared control is a core differentiator, not an optional detail.
- **Scope**: Audio foundation takes precedence over elaborate visual graph editing — avoid delaying the core instrument loop.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Build Scrysynth as a graph-native Tauri desktop app | The product is intended as an inhabitable GUI instrument for live performance | Accepted |
| Keep canonical session truth inside the app | Shared semantics, explainability, and persistence should not depend on a runtime engine | Accepted |
| Use SuperCollider as the v1 audio runtime adapter | It is strong for synths, effects, routing, modulation, and real-time control | Accepted, runtime hardening pending |
| Keep the visual runtime separate from the audio runtime | Audio and visuals should coordinate through shared abstractions without forcing one engine to own both | Accepted, sidecar pending |
| Make shared human/agent control first-class | Delegation, reclaim, override, and negotiation are product differentiators | Accepted |
| Favor stable primitives over arbitrary internal mutation | This keeps the system composable, explainable, debuggable, performant, and agent-friendly | Accepted |
| Treat Phases 1-5 as foundation-complete, not release-complete | The code has real scaffolding, but runtime execution is not yet fully implemented | Accepted 2026-06-12 |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**Next milestone review:** after v1 runtime hardening proves real audio execution, visual sidecar behavior, hardware input, and local developer verification.

---
*Last updated: 2026-06-12 after foundation audit and runtime-hardening consolidation*
