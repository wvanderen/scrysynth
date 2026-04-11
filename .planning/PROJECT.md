# Scrysynth

## What This Is

Scrysynth is a graph-native desktop audiovisual instrument for live co-creation between a human performer and one or more AI agents. It is built as a Tauri application centered on visible signal flow, live control surfaces, and shared performance control rather than a DAW-style production workflow, terminal wrapper, or one-shot generator.

## Core Value

The instrument must let a human and agent shape a live audiovisual session together through conversation, graph structure, and direct performance control without losing legibility or human override.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] A canonical session graph models audio, visual, control, scene, variation, and ownership state inside the app.
- [ ] The desktop UI supports linked conversation, graph, and performance views over the same live session.
- [ ] A v1 runtime foundation executes audio through SuperCollider, coordinates a separate visual runtime, and supports meaningful agent-driven mutation through stable primitives.

### Out of Scope

- Multiplayer collaboration — v1 is explicitly local and single-user.
- Full DAW-style production workflow — the product is performance-native, not a studio replacement.
- Browser-first deployment — desktop Tauri is the intended v1 shell.
- Unrestricted plugin ecosystem — would expand surface area before the core instrument model is proven.
- Highly autonomous low-level DSP self-rewriting — the system should favor explainable primitive-based mutation.

## Context

The project starts from a greenfield foundation document describing an audiovisual co-creation instrument distinct from Mindrave. Mindrave remains terminal-native, text-first, and algorave/live-code oriented; Scrysynth must instead be GUI-first, graph-native, and oriented around inhabitable patch space, live control, and readable agent behavior. The product is organized around three equally important interaction modes: conversational direction, graph inspection/manipulation, and direct performance control.

The app owns canonical session truth. Audio and visual engines are runtime targets rather than the source of semantic state, with a shared event bus coordinating synchronization, control changes, and runtime feedback. SuperCollider is the recommended v1 audio execution engine, but the visual runtime should remain separate and integrated through shared abstractions rather than engine-level coupling.

The core product model already has a strong conceptual vocabulary: Session, Node, Route, Bus, Macro, Binding, Scene, Variation, OwnershipRule, and AgentRole. Important unresolved design questions remain around visual runtime choice, depth of direct graph editing in v1, scene transition behavior, primitive set size, ownership recovery behavior, and when agent actions need confirmation.

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
| Build Scrysynth as a graph-native Tauri desktop app | The product is intended as an inhabitable GUI instrument for live performance | — Pending |
| Keep canonical session truth inside the app | Shared semantics, explainability, and persistence should not depend on a runtime engine | — Pending |
| Use SuperCollider as the v1 audio runtime adapter | It is strong for synths, effects, routing, modulation, and real-time control | — Pending |
| Keep the visual runtime separate from the audio runtime | Audio and visuals should coordinate through shared abstractions without forcing one engine to own both | — Pending |
| Make shared human/agent control first-class | Delegation, reclaim, override, and negotiation are product differentiators | — Pending |
| Favor stable primitives over arbitrary internal mutation | This keeps the system composable, explainable, debuggable, performant, and agent-friendly | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? -> Move to Out of Scope with reason
2. Requirements validated? -> Move to Validated with phase reference
3. New requirements emerged? -> Add to Active
4. Decisions to log? -> Add to Key Decisions
5. "What This Is" still accurate? -> Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check - still the right priority?
3. Audit Out of Scope - reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-11 after initialization*
