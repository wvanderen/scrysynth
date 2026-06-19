# Requirements: Scrysynth

**Defined:** 2026-04-11
**Consolidated:** 2026-06-12
**Core Value:** The instrument must let a human and agent shape a live audiovisual session together through conversation, graph structure, and direct performance control without losing legibility or human override.

## Status Model

This file separates foundation completion from release completion:

- **Foundation complete** means the app state, command path, UI projection, and tests/scaffolding exist.
- **Runtime hardening needed** means the feature is represented but not yet proven as production local behavior.
- **Release complete** means the feature works end to end in the desktop app with real runtime dependencies and documented setup.

## v1 Foundation Requirements

### Session Graph

- [x] **SESS-01**: User can create and open a session whose canonical state includes nodes, routes, buses, macros, scenes, variations, ownership rules, and runtime status references.
- [x] **SESS-02**: User can inspect the current session graph as visible nodes and connections instead of hidden runtime-only state.
- [x] **SESS-03**: User can inspect a node's identity, type, ports, parameters, runtime target, scene membership, and ownership metadata.
- [x] **SESS-04**: User can create, remove, enable, and re-route supported v1 graph primitives without editing raw runtime internals.

### Audio Runtime Foundation

- [x] **AUD-01F**: Audio runtime lifecycle exists with compile/start/stop/panic state management.
- [x] **AUD-02F**: Supported audio parameters can be changed in canonical session state.
- [x] **AUD-03F**: Audio graph buses and route compilation are represented in the canonical model.
- [x] **AUD-04F**: Panic-safe control path resets runtime state to a known app state.

### Interface

- [x] **UI-01**: User can switch between conversation, graph, and performance views that all reflect the same live session state.
- [x] **UI-02**: User can see current runtime health, activity, and error status for the audio runtime, visual runtime, and agent system.
- [x] **UI-03F**: User can inspect structured action history/diff summaries for command-layer mutations.

### Performance Control

- [x] **CTRL-01F**: User can create and adjust macros that map one control to multiple audio and visual targets in canonical state.
- [x] **CTRL-02**: User can trigger scenes that recall predefined session states for live performance.
- [x] **CTRL-03**: User can save a variation or snapshot of the current session and restore it later during the same working session.
- [x] **CTRL-04F**: Hardware binding model and learn/routing state machine exist for MIDI/OSC sources.

### Agent Collaboration Foundation

- [x] **AGNT-01F**: User can direct deterministic natural-language commands against the current canonical session.
- [x] **AGNT-02F**: User can see node ownership metadata and ownership badges.
- [x] **AGNT-03F**: High-risk agent actions can be held as pending actions and approved/rejected.
- [x] **AGNT-04F**: User can reclaim control and freeze agent changes without restarting the session.

### Persistence and Sync Foundation

- [x] **PERS-01**: User can save and reload a session with graph structure, macro definitions, scene data, ownership rules, and runtime mapping state restored from app-owned data.
- [x] **PERS-02F**: Basic visual runtime state and scene projection exist without making the visual engine the source of truth.

## v1 Release Requirements Still Needed

### Local Setup

- [ ] **DEV-01**: A new developer can install dependencies and run documented verification commands successfully.
- [ ] **DEV-02**: Missing Node/npm dependencies, missing Rust/cargo, missing `scsynth`, and missing visual sidecar produce actionable setup messages.

### Real Audio Execution

- [ ] **AUD-01R**: User can hear playable audio from supported source, effect, and routing primitives executed through SuperCollider.
- [ ] **AUD-02R**: User can update supported audio parameters during playback and hear the change without rebuilding the whole session.
- [ ] **AUD-03R**: User can route supported audio nodes through buses and grouped processing in `scsynth`.
- [ ] **AUD-04R**: Panic control stops actual sound immediately and returns both app state and runtime process state to safety.

### Real Visual Execution

- [ ] **VIS-01R**: User can run a basic visual sidecar that receives compiled scene state from the app.
- [ ] **VIS-02R**: Visual parameters targeted by macros are delivered to the visual runtime.
- [ ] **VIS-03R**: Visual runtime failures and restarts are visible and recoverable from the workspace.

### Hardware Runtime

- [x] **CTRL-04R**: User can bind live MIDI or OSC input through learn in the app-owned hardware runtime path and use it during performance for macros, scene recall, transport play/stop, and panic. Verified with a CoreMIDI virtual source and local OSC sender; GUI click-through hardware UAT remains release polish.
- [x] **HW-01**: User can choose/configure MIDI input and OSC listen settings through the app runtime settings path. Verified with virtual MIDI input selection and a local OSC listen endpoint.

### Agent Runtime

- [ ] **AGNT-01R**: User can collaborate with a session-aware agent beyond deterministic keyword parsing.
- [ ] **AGNT-02R**: Agent proposals remain explainable, reviewable, and constrained to typed commands.
- [ ] **AGNT-03R**: Approval/rejection flows are verified against realistic agent proposals.

### Release Quality

- [ ] **REL-01**: Tauri app packages successfully for the target desktop platform.
- [ ] **REL-02**: Manual UAT covers save/open, graph edit, audio playback, scene/variation recall, macro control, hardware learn, agent approvals, visual runtime, and panic recovery.
- [ ] **REL-03**: README, planning docs, and release notes describe supported behavior without overstating scaffolded paths.

## v2 Requirements

Deferred to future release. Tracked but not in current v1 hardening milestone.

### Ownership and Collaboration

- **OWNR-01**: User can assign fine-grained ownership policies such as nudge-only, temporary delegation, and priority-shared control per object.
- **OWNR-02**: User can tune approval thresholds by action risk, target object type, or performance mode.
- **COLL-01**: User can branch, compare, and merge rehearsal variations across longer creative sessions.

### Visual Authoring

- **VIS-04**: User can perform deeper direct graph editing of visual pipelines beyond bounded v1 controls.
- **VIS-05**: User can configure advanced audiovisual transition behaviors such as morphing, preload policies, and hybrid scene switching.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Multiplayer collaboration | v1 is intentionally local and single-user |
| Full DAW-style arrangement workflow | product is performance-native, not a studio replacement |
| Unrestricted plugin ecosystem | would expand reliability, security, and support surface before the core model is proven |
| Deep projection mapping and media-server scope | large separate product area that would delay the playable instrument loop |
| Browser-first deployment | desktop Tauri is the intended v1 platform |
| Autonomous low-level DSP self-rewriting | conflicts with explainability, safety, and stable primitive-based mutation |

## Traceability

| Requirement | Phase/Milestone | Status |
|-------------|-----------------|--------|
| SESS-01..04 | Phases 1-2 | Foundation complete |
| UI-01 | Phase 3 | Foundation complete |
| UI-02 | Phase 5 | Foundation complete |
| UI-03F | Phase 4 | Foundation complete |
| CTRL-01F | Phase 5 | Foundation complete |
| CTRL-02..03 | Phase 3 | Foundation complete |
| CTRL-04F | Phase 5 | Foundation complete |
| AGNT-01F..04F | Phase 4 | Foundation complete |
| PERS-01 | Phase 1 | Foundation complete |
| PERS-02F | Phase 5 | Foundation complete |
| DEV-01..02 | Phase 6 | Pending |
| AUD-01R..04R | Phase 7 | Pending |
| VIS-01R..03R | Phase 8 | Pending |
| CTRL-04R, HW-01 | Phase 9 | App-runtime path verified |
| AGNT-01R..03R | Phase 10 | Pending |
| REL-01..03 | Phase 11 | Pending |
