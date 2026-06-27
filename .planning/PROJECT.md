# Scrysynth

## What This Is

Scrysynth is a graph-native desktop audiovisual instrument for live co-creation between a human performer and one or more AI agents. It is built as a Tauri application centered on visible signal flow, live control surfaces, and shared performance control rather than a DAW-style production workflow, terminal wrapper, or one-shot generator.

## Core Value

The instrument must let a human and agent shape a live audiovisual session together through conversation, graph structure, and direct performance control without losing legibility or human override.

## Current Milestone: v2.0 Studio-Grade Instrument

**Goal:** Transform Scrysynth from a verified v1 foundation into a studio-grade, fluently-playable audiovisual instrument — deep modular nodes, a rebuilt graph surface, visuals behind the grid, and a focused pro shell — while landing the v1 carry-overs.

**Target features:**
- Pro-grade focused shell (graph as hero, chat sidebar, progressive-disclosure menus/panels, iconography)
- Graph UX rebuild (draggable nodes; intuitive & flexible edge connect/reconnect)
- Curated modular node library (~12–16 node types: oscillators, filters, envelopes, LFOs, FX, sequencing) with rich per-node parameters
- Visuals behind the grid (richer Bevy runtime as ambient layer behind the graph surface)
- Live provider-backed agent orchestration (AGNT-01R)
- Cross-platform builds (Windows / Linux / Intel / universal macOS)
- Full Developer ID notarization + auto-updater

## Current Stage

Scrysynth **v1.0 (1.0.0) shipped** on macOS Apple Silicon — an ad-hoc-signed `scrysynth.app` + `.dmg`, verified end-to-end against the packaged app.

The foundation (Phases 1–5) and the v1 runtime-hardening milestone (Phases 7–11, with Phase 6 dev-readiness folded into Phase 11) are both complete. The canonical Rust session graph, generated TypeScript contracts, JSON persistence, graph/conversation/performance UI surfaces, command handlers, ownership and approval scaffolding, runtime health state, macro controls, visual adapter, and MIDI/OSC binding models are implemented. Real SuperCollider audio execution, a minimal bundled visual sidecar, the app-owned MIDI/OSC hardware runtime path, session-aware agent orchestration, and release packaging have each been verified against the packaged app in the consolidated Phase 11 UAT (9/9 scenario areas passed).

The next milestone has not been started. Candidate work (tracked, not committed): richer Bevy-rendered visuals + visible render window, live provider-backed agent orchestration (AGNT-01R), physical-controller GUI click-through UAT, Windows/Linux/Intel/universal builds, and full Developer ID notarization + auto-update.

**v2.0 (Studio-Grade Instrument) milestone is now active** — see "Current Milestone" above. The v1 carry-overs (live provider agent, richer Bevy visuals, cross-platform builds, full notarization + auto-update) are folded into v2.0 alongside the new instrument-depth and shell-overhaul work.

**Phase 12 (Node Catalog Foundation) complete** — the compiled-in `NodeCatalogEntry` table is now the single source of truth replacing v1's three hardcoded compiler allowlists + two enum-dispatch spots; ~15 catalog entries drive compiler/visual dispatch, palette, inspector, and ts-rs export; 14 v2 SuperCollider SynthDefs with CV-bus args; per-parameter CV ports with control-bus allocation; an app-driven 16-step sequencer (OSC `/c_set`); two-phase v1 session rejection; and a real-`scsynth` conformance test that `/d_recv`s every catalog entry. Verified 4/4 success criteria (real scsynth conformance passed locally). Next: Phase 13 (Graph UX Rebuild).

## Requirements

### Validated

- ✓ A canonical session graph models audio, visual, control, scene, variation, ownership, runtime, pending-action, history, and hardware-binding state inside the app — v1.0
- ✓ The desktop UI supports linked conversation, graph, and performance views over the same live session — v1.0
- ✓ The command layer supports graph edits, scene/variation recall, macro CRUD, agent approvals, ownership controls, runtime health, and hardware binding models — v1.0
- ✓ A new developer can install and run local checks from documented setup instructions (DEV-01) — v1.0
- ✓ A v1 runtime foundation executes actual audio through SuperCollider (AUD-01R..04R) — v1.0
- ✓ A separate visual sidecar receives and applies scene/control updates (VIS-01R..03R) — v1.0
- ✓ MIDI/OSC learn works from live desktop runtime listeners (CTRL-04R, HW-01) — v1.0
- ✓ Agent collaboration is session-aware, explainable, and constrained by human override — deterministic/mock planner path + production-GUI wiring verified (AGNT-02R..03R) — v1.0
- ✓ Tauri app packages for the target platform (REL-01), manual UAT covers the v1 scenario matrix (REL-02), and docs describe supported behavior without overstating stubs (REL-03) — v1.0
- ✓ Curated modular node library as a data-driven catalog (NODES-01..05): ~15 node types (oscillator/noise/filter/envelope/LFO/VCA/quantizer/mixer/delay/reverb/distortion/chorus/flanger/output/step-sequencer) mapped to v2 SuperCollider SynthDefs, with per-parameter CV ports, control-bus allocation, app-driven 16-step sequencer, frontend palette/inspector consumption, and a real-scsynth conformance gate — Phase 12 (v2.0)

### Active — v2.0 Studio-Grade Instrument

- [ ] Pro-grade focused shell: graph as hero, chat sidebar, progressive-disclosure menus/panels, iconography, intelligent screenspace — retire the "card webui" feel (design philosophy = focused instrument, not dense/maximalist).
- [ ] Graph UX rebuild: draggable nodes; intuitive & flexible edge connect/reconnect.
- [ ] Visuals behind the grid: richer Bevy visual runtime rendered as an ambient layer behind the graph surface (Bespoke-Synth-style); unifies the v1 "richer Bevy" carry-over.
- [ ] Live provider-backed agent orchestration (AGNT-01R carry-over): connect the verified planner boundary to a live LLM provider.
- [ ] Cross-platform builds: Windows / Linux / Intel / universal macOS targets.
- [ ] Full Developer ID signing + notarization + an auto-updater.

### Out of Scope

- Multiplayer collaboration — v1 is explicitly local and single-user.
- Full DAW-style production workflow — the product is performance-native, not a studio replacement.
- Browser-first deployment — desktop Tauri is the intended v1 shell.
- Unrestricted plugin ecosystem — would expand surface area before the core instrument model is proven.
- Highly autonomous low-level DSP self-rewriting — the system should favor explainable primitive-based mutation.

## Context

Scrysynth started from a greenfield foundation document describing an audiovisual co-creation instrument distinct from Mindrave. Mindrave remains terminal-native, text-first, and algorave/live-code oriented; Scrysynth is GUI-first, graph-native, and oriented around inhabitable patch space, live control, and readable agent behavior.

The product is organized around three equally important interaction modes: conversational direction, graph inspection/manipulation, and direct performance control.

The app owns canonical session truth. Audio and visual engines are runtime targets rather than the source of semantic state. SuperCollider is the v1 audio execution engine, while visuals run behind a separate adapter. Audiovisual coherence comes from shared session abstractions, macros, scenes, transport, and runtime feedback rather than coupling all behavior to one engine.

**Shipped state (v1.0):** ~53.7k LOC across Rust core, TypeScript frontend, and a minimal visual sidecar. Tech stack: Tauri 2, Rust (serde/tokio/rusqlite/rosc), React 19 + Vite + TypeScript + Zustand/Immer + @xyflow/react + Radix + vanilla-extract, SuperCollider 3.14 via OSC, bundled `scrysynth-visual` sidecar via `tauri-plugin-shell` externalBin, SQLite persistence. Verified by `npm test`, `npm run build`, `cargo test`, and a 9-scenario packaged-app UAT. Ad-hoc-signed for `aarch64-apple-darwin` only.

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
| Build Scrysynth as a graph-native Tauri desktop app | The product is intended as an inhabitable GUI instrument for live performance | ✓ Good — v1 shipped as Tauri 2 |
| Keep canonical session truth inside the app | Shared semantics, explainability, and persistence should not depend on a runtime engine | ✓ Good — Rust `SessionDocument` is the source of truth; adapters consume it |
| Use SuperCollider as the v1 audio runtime adapter | It is strong for synths, effects, routing, modulation, and real-time control | ✓ Good — audible playback + panic/restart verified against real `scsynth` |
| Keep the visual runtime separate from the audio runtime | Audio and visuals should coordinate through shared abstractions without forcing one engine to own both | ✓ Good for boundary; minimal sidecar shipped — richer Bevy rendering deferred |
| Make shared human/agent control first-class | Delegation, reclaim, override, and negotiation are product differentiators | ✓ Good — ownership/freeze/reclaim/approval all verified |
| Favor stable primitives over arbitrary internal mutation | Composable, explainable, debuggable, performant, agent-friendly | ✓ Good — all agent mutations flow through typed commands |
| Treat Phases 1-5 as foundation-complete, not release-complete | Real runtime execution was not yet implemented at that point | ✓ Good — drove the Phases 7–11 hardening milestone |
| Provider-agnostic planner boundary before live LLM provider (Phase 10) | Keep safety/typed-command surface stable; defer provider setup/fallback UAT | ✓ Good — boundary verified; AGNT-01R live provider deferred as future hardening |
| Ad-hoc signing + Apple-Silicon-only for v1 release (Phase 11) | Avoid blocking v1 on Developer ID/notarization; ship a verifiable build | — Pending — unblocks UAT; full notarization + auto-update deferred |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-06-27 after Phase 12 (Node Catalog Foundation) completion*
