# Roadmap: Scrysynth

## Current Picture

Scrysynth has completed its foundation pass: the canonical session graph, desktop workspace, command handlers, persistence, performance surfaces, agent safety scaffolding, runtime health projections, cross-domain macros, and hardware-binding data paths are implemented.

The project is now in **v1 runtime hardening**, not feature-complete release. Real SuperCollider execution, the minimal visual sidecar path, the app-owned MIDI/OSC hardware runtime path, and deterministic/mock session-aware agent orchestration have been verified locally. The main remaining runtime gaps are richer Bevy visual output/packaging, live provider-backed agent orchestration, release packaging, and broader manual UAT.

Core distinction:

- **Foundation complete:** canonical state, UI projections, command APIs, tests, and adapter boundaries exist.
- **Runtime incomplete:** richer Bevy visual execution, packaged sidecar wiring, live provider-backed agent orchestration, and release packaging still need production implementation or verification. The default SuperCollider execution path, minimal visual sidecar process path, virtual MIDI/local OSC hardware runtime path, and deterministic/mock planner orchestration path are verified.

## Completed Foundation Phases

- [x] **Phase 1: Session Core & Recall** - Sessions are canonical, inspectable, and reloadable from app-owned state.
- [x] **Phase 2: Playable Audio Graph Foundation** - Users can edit supported graph primitives and drive audio-runtime lifecycle state.
- [x] **Phase 3: Performance Workspace** - Users can move through graph, conversation, and performance views and recall scenes/variations.
- [x] **Phase 4: Agent Collaboration Scaffold** - Users can send natural-language instructions through deterministic parsing, inspect diffs/history, approve high-risk pending actions, freeze/reclaim ownership, and see ownership badges.
- [x] **Phase 5: Visual Sync & Cross-Modal Control Scaffold** - Visual runtime health, cross-domain macro structures, MIDI/OSC binding models, and performance UI are in place.

## Foundation Deliverables

| Area | Current status | Notes |
|------|----------------|-------|
| Canonical session graph | Complete foundation | Rust owns `SessionDocument`; TypeScript contract is generated from Rust. |
| Session persistence | Complete foundation | JSON save/open with schema version validation. |
| Graph workspace | Complete foundation | React Flow projection, inspector, primitive palette, route gestures. |
| Audio runtime | Phase 7 complete | Can launch/stop/panic `scsynth` when installed; v1 SynthDef/topology commands are applied over OSC; default graph audible playback, live parameter control, stop, panic, and restart-after-panic are verified. |
| Performance workspace | Complete foundation | Scenes, variations, active scene derivation, macro controls. |
| Agent collaboration | Phase 10 mock planner path verified | Bounded context packets, provider-agnostic planner interface, deterministic/mock proposal fixtures, typed-command normalization, ownership gates, pending approvals, action history, freeze/reclaim, and diagnostics are verified. A live LLM/provider-backed planner remains a release hardening gap. |
| Visual runtime | Phase 8 minimal path verified | `scrysynth-visual` exists as an in-repo minimal sidecar. The app adapter launches it, handshakes, loads compiled scenes, sends parameter batches, stops, panics, and restarts after panic. Rich Bevy rendering and packaged sidecar bundling remain future hardening. |
| Hardware bindings | Phase 9 app-runtime path verified | MIDI/OSC listener settings, live learn, post-learn routing, scene recall, transport play/stop, and panic were verified with a CoreMIDI virtual source, local OSC sender, and local SuperCollider. GUI click-through hardware UAT remains a future polish pass. |
| Testing | Broad but environment-dependent | Rust and frontend tests exist, but require local `cargo` and installed npm dependencies. |
| Documentation | In progress | README and planning docs now reflect foundation-vs-runtime status. |

## Next Milestone: v1 Runtime Hardening

### Phase 6: Local Developer Readiness

**Goal:** A developer can clone, install, verify, and launch the app with clear diagnostics.

Success criteria:

1. README accurately lists required local toolchains and optional runtime binaries.
2. `npm install`, `npm test`, `npm run build`, and `cargo test --manifest-path src-tauri/Cargo.toml` are the documented verification path.
3. Missing `cargo`, missing `node_modules`, missing `scsynth`, or missing visual sidecar produce clear setup guidance.
4. Planning docs agree on current phase, completed work, and remaining runtime gaps.

### Phase 7: Real SuperCollider Execution

**Goal:** The canonical audio graph produces actual sound through SuperCollider.

Status: Complete for the default source-to-output graph UAT.

Design note: `.planning/phases/07-real-supercollider-execution/07-real-supercollider-execution-01-SC-RESOURCE-PLAN.md` defines the v1 `CompiledTopology` to SuperCollider resource mapping for task `td-64eb52`.

Success criteria:

1. Compile graph topology into SC synthdefs/resources, groups, buses, and node launch order.
2. Send OSC bundles to `scsynth` for boot, resource creation, routing, parameter updates, and teardown.
3. Apply live parameter edits and reroutes without pretending topology load succeeded.
4. Add runtime feedback, `/sync` handling, degraded states, and failure diagnostics.
5. Verify audible playback and panic recovery on a real local SuperCollider install.

Current Phase 7.7 evidence: `.planning/phases/07-real-supercollider-execution/07-real-supercollider-execution-07-UAT.md`. Resumed real-`scsynth` UAT verified audible output, live parameter control, stop, panic, and restart-after-panic. A Stop-button projection defect found during UAT was fixed and retested successfully.

### Phase 8: Real Visual Runtime Path

**Goal:** Visuals run through a separate process that consumes canonical scene/control projections.

Status: Complete for the minimal `scrysynth-visual` sidecar path. Evidence is recorded in `.planning/phases/08-real-visual-runtime-path/08-real-visual-runtime-path-05-UAT.md`.

Success criteria:

1. Provide or document the `scrysynth-visual` sidecar executable. **Done for local development.**
2. Replace visual scene-load and parameter-update stubs with a typed local protocol. **Done for JSON-lines stdio.**
3. Drive visual scene state from session nodes, scenes, macros, and runtime events. **Done for compiled scene snapshots and live parameter batches.**
4. Surface visual runtime errors and reconnect/restart behavior in the health panel. **Done for missing sidecar, booting, ready, degraded, disconnected, stopped, panicked, and restartable states.**

Remaining Phase 8 follow-on work belongs in release hardening, not in the now-verified minimal path:

- Bundle the sidecar through Tauri packaging instead of relying on `SCRYSYNTH_BEVY_PATH` or `PATH`.
- Replace the minimal GPU-free renderer with richer Bevy-rendered scene output.
- Emit and consume live visual telemetry such as FPS from the sidecar.

### Phase 9: Hardware Input Runtime Wiring

**Goal:** MIDI/OSC learn works against live devices and senders in the desktop app.

Status: Complete for the app-owned runtime path. Evidence is recorded in `.planning/phases/09-hardware-input-runtime-wiring/09-hardware-input-runtime-wiring-06-UAT.md`.

Success criteria:

1. Add app commands/settings for MIDI port selection and OSC listen port configuration. **Done.**
2. Start/stop MIDI and OSC listeners as part of runtime state, not only in tests. **Done.**
3. Wire listener receivers into `SessionStore`/router lifecycle. **Done.**
4. Verify macro, scene, transport, and panic targets from real or virtual hardware. **Done with CoreMIDI virtual MIDI and local OSC sender.**

Remaining Phase 9 follow-on work belongs in release polish:

- Run a physical-controller click-through pass when hardware is available.

### Phase 10: Session-Aware Agent Orchestration

**Goal:** Agent collaboration becomes meaningfully intelligent while preserving human override.

Status: Complete for deterministic/mock planner UAT. Evidence is recorded in `.planning/phases/10-session-aware-agent-orchestration/10-session-aware-agent-orchestration-06-UAT.md`.

Design note: Phase 10 keeps the existing typed-command, ownership, risk, approval, freeze, reclaim, and action-history pipeline as the safety boundary. The new work belongs above that boundary: a bounded session context packet, provider-agnostic planner, structured proposals, validation diagnostics, and review UI.

Success criteria:

1. Replace or augment the deterministic parser with a session-aware agent planning layer. **Done for deterministic/mock and local parser providers.**
2. Keep all agent changes flowing through typed commands, ownership gates, risk tiers, and approvals. **Done and verified.**
3. Show structured proposed changes before high-risk mutation. **Done for pending action/proposal review UI coverage.**
4. Preserve action history and readable diffs for both user and agent actions. **Done through the existing command path.**
5. Surface provider unavailable, invalid response, unsafe target, validation failure, freeze, and rejection states in agent runtime status and conversation UI. **Verified for deterministic/mock diagnostics; stale-context handling remains contract-level only.**
6. Verify realistic graph, parameter/macro, scene/variation, approval/rejection, freeze, and reclaim flows through deterministic/mock planner UAT before claiming Phase 10 complete. **Verified for graph mutation, parameter change, scene recall, approval/rejection, freeze, and reclaim. Variation command behavior is covered by performance tests but not by a planner-authored Phase 10 fixture.**

Task breakdown:

- `td-cebec0`: Phase 10 epic.
- `td-665cc6`: Define agent orchestration contract.
- `td-0e888f`: Build session context packet and planner interface.
- `td-84b783`: Validate proposals through command and ownership gates.
- `td-a842b4`: Add proposal review UI and agent runtime status.
- `td-4a9cfe`: Add deterministic/mock planner tests and realistic proposal fixtures.
- `td-ab3c29`: Verify agent orchestration UAT and update docs.

Remaining Phase 10 follow-on work belongs in release hardening:

- Connect the provider-agnostic planner interface to a live configured provider.
- Add UAT for live provider setup, unavailable/degraded provider modes, and fallback behavior.
- Add a planner-authored variation proposal fixture before claiming variation planning coverage.

### Phase 11: Release Readiness

**Goal:** The project can be packaged and evaluated as a v1 desktop instrument.

Success criteria:

1. Tauri packaging works for the target OS.
2. Runtime dependency discovery and error messages are polished.
3. Manual UAT covers save/open, graph edit, audio playback, scene/variation recall, macro control, hardware learn, agent approval, and panic recovery.
4. README, planning docs, and release notes describe actual supported behavior without overstating stubs.

## Deferred Beyond v1

- Multiplayer collaboration.
- Full DAW-style timeline/arrangement.
- Deep projection mapping or media-server workflows.
- General plugin marketplace.
- Fine-grained ownership policies beyond v1 freeze/reclaim/approval.
- Advanced visual graph editing and sophisticated transition/morph authoring.

## Recent Audit Notes

Audit date: 2026-06-12.

- `td` is available for task tracking, but the GSD slash-command workflow is not callable from the shell environment.
- The local checkout initially lacked `node_modules`, so `npm test` and `npm run build` could not run until dependencies are installed.
- `cargo` was not on `PATH` in the audit shell, so Rust tests could not be executed there.
- Worktree status during audit: branch `main`, tracking `origin/main`; `AGENTS.md` was untracked.
