# Roadmap: Scrysynth

## Overview

Scrysynth reaches v1 by first making the session graph canonical and reloadable, then turning that graph into a safe playable audio instrument, then adding the linked performance workspace, agent collaboration, and finally cross-modal visual/control integration. The phase order follows the product's trust chain: semantics first, sound second, shared surfaces third, collaborative mutation fourth, and audiovisual coordination last.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Session Core & Recall** - Sessions are canonical, inspectable, and reloadable from app-owned state.
- [ ] **Phase 2: Playable Audio Graph** - Users turn the canonical graph into reliable sound with safe live mutation.
- [ ] **Phase 3: Performance Workspace** - Users move through linked views and recall scenes or variations during a live session.
- [ ] **Phase 4: Agent Collaboration** - Users co-create through natural language with visible ownership and gated mutations.
- [ ] **Phase 5: Visual Sync & Cross-Modal Control** - Audio and visuals respond together to shared macros, hardware input, and runtime feedback.

## Phase Details

### Phase 1: Session Core & Recall
**Goal**: Users can create, inspect, save, and reopen a canonical Scrysynth session without runtime engines becoming the source of truth.
**Depends on**: Nothing (first phase)
**Requirements**: SESS-01, SESS-02, SESS-03, PERS-01
**Success Criteria** (what must be TRUE):
  1. User can create or open a session whose app-owned state includes nodes, routes, buses, macros, scenes, variations, ownership rules, and runtime status references.
  2. User can inspect the current session as visible graph nodes and connections instead of hidden runtime-only state.
  3. User can inspect a node's identity, type, ports, parameters, runtime target, scene membership, and ownership metadata.
  4. User can save, close, and reload a session with graph structure, macro definitions, scene data, ownership rules, and runtime mapping state restored.
**Plans**: 3 plans
Plans:
- [x] `01-session-core-recall-01-PLAN.md` — Define the canonical Rust session schema and Tauri session-store commands.
- [x] `01-session-core-recall-02-PLAN.md` — Add versioned JSON save/open persistence and round-trip recall tests.
- [ ] `01-session-core-recall-03-PLAN.md` — Build the Phase 1 session workspace with graph inspection and save/open controls.
**UI hint**: yes

### Phase 2: Playable Audio Graph
**Goal**: Users can build, hear, and safely recover a live audio patch driven by the canonical session graph.
**Depends on**: Phase 1
**Requirements**: SESS-04, AUD-01, AUD-02, AUD-03, AUD-04
**Success Criteria** (what must be TRUE):
  1. User can create, remove, enable, and re-route supported v1 graph primitives without editing raw runtime internals.
  2. User can hear playable audio from supported source, effect, and routing primitives executed through the SuperCollider adapter.
  3. User can update supported audio parameters during playback and hear the result without rebuilding the whole session.
  4. User can route supported audio nodes through buses and grouped processing defined by the canonical session graph.
  5. User can stop all sound immediately with a panic-safe control that returns the app to a known safe state.
**Plans**: TBD

### Phase 3: Performance Workspace
**Goal**: Users can navigate the live instrument as one coherent workspace and recall structured performance states during a session.
**Depends on**: Phase 2
**Requirements**: UI-01, CTRL-02, CTRL-03
**Success Criteria** (what must be TRUE):
  1. User can switch between conversation, graph, and performance views that all reflect the same live session state.
  2. User can trigger scenes that recall predefined session states for live performance.
  3. User can save a variation or snapshot of the current session and restore it later during the same working session.
**Plans**: TBD
**UI hint**: yes

### Phase 4: Agent Collaboration
**Goal**: Users can direct Scrysynth in natural language while keeping changes, ownership, and override behavior legible and safe.
**Depends on**: Phase 3
**Requirements**: UI-03, AGNT-01, AGNT-02, AGNT-03, AGNT-04
**Success Criteria** (what must be TRUE):
  1. User can direct the system in natural language and receive proposed or applied changes against the current canonical session.
  2. User can inspect what changed after a user or agent action through visible diffs, activity history, or equivalent structured feedback.
  3. User can see which nodes, macros, scenes, or controls are agent-controlled, shared, or user-controlled.
  4. User can approve, reject, or cancel higher-risk agent actions before they mutate the live session.
  5. User can reclaim control from the agent, freeze agent changes, or disable the conductor role without restarting the session.
**Plans**: TBD
**UI hint**: yes

### Phase 5: Visual Sync & Cross-Modal Control
**Goal**: Users can extend the live session into visuals and cross-modal performance control through the shared workspace.
**Depends on**: Phase 4
**Requirements**: UI-02, CTRL-01, CTRL-04, PERS-02
**Success Criteria** (what must be TRUE):
  1. User can run a basic visual runtime that responds to shared session events, scenes, or macros without making the visual engine the source of truth.
  2. User can create and adjust macros that map one control to multiple audio and visual parameters.
  3. User can bind supported MIDI or OSC input through learn to v1 macros or performance actions and use that input during performance.
  4. User can see runtime health, activity, and error status for the audio runtime, visual runtime, and agent system from the shared workspace.
**Plans**: TBD
**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Session Core & Recall | 2/3 | In Progress | 2026-04-11 |
| 2. Playable Audio Graph | 0/TBD | Not started | - |
| 3. Performance Workspace | 0/TBD | Not started | - |
| 4. Agent Collaboration | 0/TBD | Not started | - |
| 5. Visual Sync & Cross-Modal Control | 0/TBD | Not started | - |
