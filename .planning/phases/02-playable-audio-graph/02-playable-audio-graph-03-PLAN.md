---
phase: 02-playable-audio-graph
plan: 03
type: execute
wave: 3
depends_on:
  - 02-playable-audio-graph-01
  - 02-playable-audio-graph-02
files_modified:
  - package.json
  - src/App.tsx
  - src/App.css
  - src/components/session/GraphViewport.tsx
  - src/components/session/NodeInspector.tsx
  - src/components/session/SessionToolbar.tsx
  - src/components/audio/AudioTransportStrip.tsx
  - src/components/audio/PrimitivePalette.tsx
  - src/lib/session-client.ts
  - src/store/session-projections.ts
  - src/store/session-projections.test.ts
  - src/store/sessionStore.ts
autonomous: false
requirements:
  - SESS-04
  - AUD-02
  - AUD-03
  - AUD-04
must_haves:
  truths:
    - The workspace lets the user add, remove, enable, and reroute supported primitives without touching raw runtime internals.
    - Supported parameter changes apply during playback through incremental backend updates where safe, with errors surfaced in the workspace instead of silent failure.
    - Play, stop, panic, and runtime health are visible from the same instrument workspace used to inspect the canonical session graph.
  artifacts:
    - path: src/components/audio/PrimitivePalette.tsx
      provides: Minimal performer-facing controls to add supported audio primitives
      contains: "Add Source"
    - path: src/components/audio/AudioTransportStrip.tsx
      provides: Play, stop, panic, and runtime health controls
      contains: "Panic"
    - path: src/store/sessionStore.ts
      provides: Frontend actions for graph edits, live parameter updates, and runtime transport
      contains: "updateNodeParameter"
  key_links:
    - from: src/lib/session-client.ts
      to: src-tauri/src/lib.rs
      via: invoke calls bridge frontend graph edits and runtime transport to backend commands
      pattern: "apply_graph_edit|start_audio_runtime|stop_audio_runtime|panic_audio_runtime"
    - from: src/store/sessionStore.ts
      to: src/components/session/GraphViewport.tsx
      via: projected nodes and edges re-render after live edits or runtime state changes
      pattern: "graphNodes|graphEdges"
    - from: src/components/audio/PrimitivePalette.tsx
      to: src/store/sessionStore.ts
      via: UI palette issues bounded graph-edit actions rather than mutating local-only state
      pattern: "applyGraphEdit"
---

<objective>
Finish Phase 2 by making the playable audio graph audible and controllable from the Scrysynth workspace.

Purpose: The phase is only complete when the user can hear the canonical graph, mutate supported parameters and routes during playback, and recover with an obvious panic control from the same desktop instrument surface.
Output: Frontend client and store support for audio graph edits and runtime transport, minimal playback UI, live parameter controls, projection tests, and one blocking human verification pass.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/REQUIREMENTS.md
@.planning/phases/02-playable-audio-graph/02-RESEARCH.md
@.planning/phases/02-playable-audio-graph/02-playable-audio-graph-01-PLAN.md
@.planning/phases/02-playable-audio-graph/02-playable-audio-graph-02-PLAN.md
@package.json
@src/App.tsx
@src/App.css
@src/components/session/GraphViewport.tsx
@src/components/session/NodeInspector.tsx
@src/components/session/SessionToolbar.tsx
@src/lib/session-client.ts
@src/store/sessionStore.ts

<interfaces>
The frontend should consume the Phase 2 backend through explicit client helpers such as:

```ts
applyGraphEdit(command: GraphEditCommand): Promise<SessionDocument>
startAudioRuntime(): Promise<SessionDocument>
stopAudioRuntime(): Promise<SessionDocument>
panicAudioRuntime(): Promise<SessionDocument>
```

Keep the edit surface minimal and structured. The workspace may expose add source, add effect, add mixer, remove selected node, toggle enabled, change supported parameter values, and create one valid reroute path at a time.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add frontend audio graph actions and projection tests for live edits</name>
  <files>package.json, src/lib/session-client.ts, src/store/session-projections.ts, src/store/session-projections.test.ts, src/store/sessionStore.ts</files>
  <read_first>
    - package.json
    - src/lib/session-client.ts
    - src/store/session-projections.ts
    - src/store/sessionStore.ts
    - src/generated/session-types.ts
  </read_first>
  <behavior>
    - Test 1: applying a successful graph edit updates projected nodes, edges, and selected inspector data from the returned canonical session.
    - Test 2: runtime health projection preserves the latest audio status while parameter-only edits avoid unnecessary local graph rebuild assumptions.
    - Test 3: rejected edits keep the previous store state and surface an error banner message.
  </behavior>
  <action>Extend `src/lib/session-client.ts` with helpers for graph-edit and runtime transport commands, then update `src/store/sessionStore.ts` to expose actions such as `applyGraphEdit`, `updateNodeParameter`, `toggleNodeEnabled`, `startAudio`, `stopAudio`, and `panicAudio`. Keep the store driven by backend-returned `SessionDocument` snapshots rather than speculative frontend mutation. Move or extend projection helpers into `src/store/session-projections.ts` so `src/store/session-projections.test.ts` can cover live edit and runtime health behavior directly. Add or retain `vitest` support in `package.json` if needed.</action>
  <acceptance_criteria>
    - src/lib/session-client.ts contains `applyGraphEdit`
    - src/lib/session-client.ts contains `panicAudioRuntime`
    - src/store/sessionStore.ts contains `updateNodeParameter`
    - src/store/sessionStore.ts contains `startAudio`
    - src/store/session-projections.test.ts contains `runtime health`
  </acceptance_criteria>
  <verify>
    <automated>npx vitest run src/store/session-projections.test.ts</automated>
  </verify>
  <done>The frontend store can consume live backend graph and runtime updates safely and under test.</done>
</task>

<task type="auto">
  <name>Task 2: Build the minimal playable workspace controls for bounded live audio editing</name>
  <files>src/App.tsx, src/App.css, src/components/session/GraphViewport.tsx, src/components/session/NodeInspector.tsx, src/components/session/SessionToolbar.tsx, src/components/audio/AudioTransportStrip.tsx, src/components/audio/PrimitivePalette.tsx, src/store/sessionStore.ts</files>
  <read_first>
    - src/App.tsx
    - src/App.css
    - src/components/session/GraphViewport.tsx
    - src/components/session/NodeInspector.tsx
    - src/components/session/SessionToolbar.tsx
    - src/store/sessionStore.ts
    - src/generated/session-types.ts
  </read_first>
  <action>Create `src/components/audio/PrimitivePalette.tsx` and `src/components/audio/AudioTransportStrip.tsx` and integrate them into `src/App.tsx` so the workspace can start or stop audio, trigger panic, view audio runtime health, add supported primitives, remove the selected node, toggle enabled state, and edit bounded parameters on the selected node. Keep the graph as the main visual anchor and preserve the existing deliberate desktop instrument tone instead of falling back to a generic form-heavy admin layout. Update `src/components/session/GraphViewport.tsx` and `src/components/session/NodeInspector.tsx` only as needed to support selection-aware edits, route affordances, and parameter controls, and make routing changes go through the bounded backend command surface rather than direct local mutation.</action>
  <acceptance_criteria>
    - src/components/audio/PrimitivePalette.tsx contains `Add Source`
    - src/components/audio/AudioTransportStrip.tsx contains `Panic`
    - src/App.tsx contains `AudioTransportStrip`
    - src/App.tsx contains `PrimitivePalette`
    - src/components/session/NodeInspector.tsx contains `Enabled`
    - src/components/session/GraphViewport.tsx contains `onConnect`
  </acceptance_criteria>
  <verify>
    <automated>npm run build</automated>
  </verify>
  <done>The workspace exposes the minimum controls needed to hear and safely edit the supported audio graph live.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 3: Verify audible playback, live mutation, rerouting, and panic recovery</name>
  <files>src/App.tsx, src/components/audio/AudioTransportStrip.tsx, src/components/audio/PrimitivePalette.tsx, src/components/session/GraphViewport.tsx, src/components/session/NodeInspector.tsx, src/store/sessionStore.ts</files>
  <read_first>
    - src/App.tsx
    - src/components/audio/AudioTransportStrip.tsx
    - src/components/audio/PrimitivePalette.tsx
    - src/components/session/GraphViewport.tsx
    - src/components/session/NodeInspector.tsx
    - src/store/sessionStore.ts
  </read_first>
  <action>Launch the Tauri app after Tasks 1 and 2 complete and verify that the user can hear the canonical graph, adjust supported parameters while audio is running, reroute within the supported graph shape, and recover immediately with panic.</action>
  <acceptance_criteria>
    - Starting audio produces an audible supported source-to-output path through the SuperCollider adapter.
    - Adjusting at least one supported parameter during playback changes the heard result without rebuilding the whole session.
    - Creating or changing one supported route or bus path updates playback and graph state safely.
    - Triggering `Panic` stops all sound immediately and the runtime can start again cleanly afterward.
    - The workspace surfaces audio runtime health changes instead of hiding them in logs only.
  </acceptance_criteria>
  <what-built>Phase 2 playable workspace with bounded graph editing, live parameter updates, runtime transport, and panic-safe recovery</what-built>
  <how-to-verify>
    1. Run `npm run tauri dev` with a working local `scsynth` installation available on `PATH`.
    2. Confirm the workspace shows the existing graph plus the new primitive palette and audio transport strip.
    3. Start audio and confirm one supported source-to-output path is audible.
    4. Select a node and adjust a supported parameter; confirm the sound changes while playback continues.
    5. Add or reroute one supported connection and confirm both the graph and audible result update safely.
    6. Trigger `Panic` and confirm sound stops immediately, then restart audio and confirm playback can recover cleanly.
    7. Confirm runtime health text or indicators reflect ready, error, or panic-recovered states as actions occur.
  </how-to-verify>
  <verify>
    <automated>npm run tauri dev</automated>
  </verify>
  <done>The user has confirmed the graph is playable, live edits are audible, and panic recovery behaves safely, or has returned a precise defect list.</done>
  <resume-signal>Type "approved" or describe the issue precisely.</resume-signal>
</task>

</tasks>

<verification>
Run the projection test file and production build, then perform one human verification pass that covers audible playback, live parameter change, supported rerouting, runtime health visibility, and panic recovery.
</verification>

<success_criteria>
Phase 2 is complete when the user can hear the canonical graph through the SuperCollider adapter, make supported live changes safely from the workspace, and stop all sound instantly with a recoverable panic control.
</success_criteria>

<output>
After completion, create `.planning/phases/02-playable-audio-graph/02-playable-audio-graph-03-SUMMARY.md`
</output>
