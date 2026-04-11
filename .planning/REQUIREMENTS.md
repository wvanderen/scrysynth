# Requirements: Scrysynth

**Defined:** 2026-04-11
**Core Value:** The instrument must let a human and agent shape a live audiovisual session together through conversation, graph structure, and direct performance control without losing legibility or human override.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Session Graph

- [x] **SESS-01**: User can create and open a session whose canonical state includes nodes, routes, buses, macros, scenes, variations, ownership rules, and runtime status references.
- [x] **SESS-02**: User can inspect the current session graph as visible nodes and connections instead of hidden runtime-only state.
- [x] **SESS-03**: User can inspect a node's identity, type, ports, parameters, runtime target, scene membership, and ownership metadata.
- [ ] **SESS-04**: User can create, remove, enable, and re-route supported v1 graph primitives without editing raw runtime internals.

### Audio Runtime

- [ ] **AUD-01**: User can hear playable audio from supported source, effect, and routing primitives executed through the SuperCollider adapter.
- [ ] **AUD-02**: User can update supported audio parameters during playback and hear the change without rebuilding the whole session.
- [ ] **AUD-03**: User can route supported audio nodes through buses and grouped processing defined by the canonical session graph.
- [ ] **AUD-04**: User can stop all sound immediately with a panic-safe control that recovers the app to a known safe state.

### Interface

- [ ] **UI-01**: User can switch between conversation, graph, and performance views that all reflect the same live session state.
- [ ] **UI-02**: User can see current runtime health, activity, and error status for the audio runtime, visual runtime, and agent system.
- [ ] **UI-03**: User can inspect what changed in the session after a user or agent action through visible diffs, activity history, or equivalent structured feedback.

### Performance Control

- [ ] **CTRL-01**: User can create and adjust macros that map one control to multiple audio and visual parameters.
- [ ] **CTRL-02**: User can trigger scenes that recall predefined session states for live performance.
- [ ] **CTRL-03**: User can save a variation or snapshot of the current session and restore it later during the same working session.
- [ ] **CTRL-04**: User can bind supported hardware control input through MIDI or OSC learn to v1 macros or performance actions.

### Agent Collaboration

- [ ] **AGNT-01**: User can direct the system in natural language and have the agent propose or apply changes against the current canonical session instead of a blank context.
- [ ] **AGNT-02**: User can see which nodes, macros, scenes, or controls are currently agent-controlled, shared, or user-controlled.
- [ ] **AGNT-03**: User can approve, reject, or cancel higher-risk agent actions before they mutate the live session.
- [ ] **AGNT-04**: User can reclaim control from the agent, freeze agent changes, or disable the conductor role without restarting the session.

### Persistence and Sync

- [x] **PERS-01**: User can save and reload a session with graph structure, macro definitions, scene data, ownership rules, and runtime mapping state restored from app-owned data.
- [ ] **PERS-02**: User can run a basic visual runtime that responds to shared session events, scenes, or macros without making the visual engine the source of truth.

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Ownership and Collaboration

- **OWNR-01**: User can assign fine-grained ownership policies such as nudge-only, temporary delegation, and priority-shared control per object.
- **OWNR-02**: User can tune approval thresholds by action risk, target object type, or performance mode.
- **COLL-01**: User can branch, compare, and merge rehearsal variations across longer creative sessions.

### Visual Authoring

- **VIS-01**: User can perform deeper direct graph editing of visual pipelines beyond bounded v1 controls.
- **VIS-02**: User can configure advanced audiovisual transition behaviors such as morphing, preload policies, and hybrid scene switching.

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Multiplayer collaboration | v1 is intentionally local and single-user |
| Full DAW-style arrangement workflow | product is performance-native, not a studio replacement |
| Unrestricted plugin ecosystem | would expand reliability, security, and support surface before the core model is proven |
| Deep projection mapping and media-server scope | large separate product area that would delay the playable instrument loop |
| Browser-first deployment | desktop Tauri is the intended v1 platform |
| Autonomous low-level DSP self-rewriting | conflicts with explainability, safety, and stable primitive-based mutation |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| SESS-01 | Phase 1 | Complete |
| SESS-02 | Phase 1 | Complete |
| SESS-03 | Phase 1 | Complete |
| SESS-04 | Phase 2 | Pending |
| AUD-01 | Phase 2 | Pending |
| AUD-02 | Phase 2 | Pending |
| AUD-03 | Phase 2 | Pending |
| AUD-04 | Phase 2 | Pending |
| UI-01 | Phase 3 | Pending |
| UI-02 | Phase 5 | Pending |
| UI-03 | Phase 4 | Pending |
| CTRL-01 | Phase 5 | Pending |
| CTRL-02 | Phase 3 | Pending |
| CTRL-03 | Phase 3 | Pending |
| CTRL-04 | Phase 5 | Pending |
| AGNT-01 | Phase 4 | Pending |
| AGNT-02 | Phase 4 | Pending |
| AGNT-03 | Phase 4 | Pending |
| AGNT-04 | Phase 4 | Pending |
| PERS-01 | Phase 1 | Complete |
| PERS-02 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 21 total
- Mapped to phases: 21
- Unmapped: 0

---
*Requirements defined: 2026-04-11*
*Last updated: 2026-04-11 after roadmap creation*
