import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { GraphEditCommand, SessionDocument } from "../generated/session-types";

const clientMocks = vi.hoisted(() => ({
  approvePendingAction: vi.fn(),
  applyGraphEdit: vi.fn<(command: GraphEditCommand) => Promise<SessionDocument>>(),
  applyMacroCommand: vi.fn(),
  applyPerformanceCommand: vi.fn<(command: import("../generated/session-types").PerformanceCommand) => Promise<SessionDocument>>(),
  createDefaultSession: vi.fn<() => Promise<SessionDocument>>(),
  getAgentRuntimeState: vi.fn(),
  getCurrentSession: vi.fn<() => Promise<SessionDocument>>(),
  getHardwareRuntimeSettings: vi.fn(),
  getHardwareRuntimeStatus: vi.fn(),
  listMidiInputPorts: vi.fn(),
  openSessionFromPath: vi.fn<(path: string) => Promise<SessionDocument>>(),
  panicAudioRuntime: vi.fn<() => Promise<SessionDocument>>(),
  panicVisualRuntime: vi.fn<() => Promise<SessionDocument>>(),
  pollHardwareEvents: vi.fn(),
  reclaimOwnership: vi.fn(),
  rejectPendingAction: vi.fn(),
  removeHardwareBinding: vi.fn(),
  saveSessionToPath: vi.fn<(path: string) => Promise<void>>(),
  sendAgentMessage: vi.fn(),
  startAudioRuntime: vi.fn<() => Promise<SessionDocument>>(),
  startHardwareLearn: vi.fn(),
  startHardwareListeners: vi.fn(),
  startVisualRuntime: vi.fn<() => Promise<SessionDocument>>(),
  stopAudioRuntime: vi.fn<() => Promise<SessionDocument>>(),
  stopHardwareListeners: vi.fn(),
  stopHardwareLearn: vi.fn(),
  stopVisualRuntime: vi.fn<() => Promise<SessionDocument>>(),
  toggleAgentFreeze: vi.fn(),
  updateHardwareRuntimeSettings: vi.fn(),
}));

vi.mock("../lib/session-client", () => clientMocks);

import { projectSessionState, deriveActiveSceneId } from "./session-projections";
import { shouldPollHardware, startHardwarePolling, stopHardwarePolling, useSessionStore } from "./sessionStore";

function createSession(overrides: Partial<SessionDocument> = {}): SessionDocument {
  return {
    schemaVersion: 1,
    sessionId: "session-1",
    title: "Playable Graph",
    createdAt: "2026-04-11T00:00:00Z",
    updatedAt: "2026-04-11T00:00:00Z",
    transport: {
      tempoBpm: 120,
      isPlaying: false,
      positionBeats: 0,
    },
    audioRuntime: {
      lifecycle: "idle",
      health: "unknown",
      sampleRateHz: null,
      blockSize: null,
      activePatchId: null,
      lastError: null,
      panicRecoveryCount: 0,
    },
    visualRuntime: {
      lifecycle: "idle",
      health: "unknown",
      activeSceneId: null,
      fps: null,
      lastError: null,
      renderer: null,
    },
    agentRuntime: {
      isAvailable: true,
      pendingActionCount: 0,
      isFrozen: false,
    },
    nodes: [
      {
        id: "source-1",
        nodeType: "source",
        ports: [
          { id: "source-1-out", name: "main_out", direction: "output", signalType: "audio" },
        ],
        parameters: [
          {
            id: "source-1-level",
            name: "level",
            value: 0.8,
            defaultValue: 0.8,
            minValue: 0,
            maxValue: 1,
            unit: "linear",
          },
        ],
        runtimeTarget: "audio/source/default",
        sceneMembership: ["scene-1"],
        ownership: { controller: "shared", isLocked: false },
        enabled: true,
        audioPrimitive: { kind: "source", config: { sourceType: "oscillator", channelMode: "mono", busTargetId: "bus-main" } },
      },
      {
        id: "output-1",
        nodeType: "output",
        ports: [
          { id: "output-1-in", name: "master_in", direction: "input", signalType: "audio" },
        ],
        parameters: [],
        runtimeTarget: "audio/output/master",
        sceneMembership: ["scene-1"],
        ownership: { controller: "user", isLocked: false },
        enabled: true,
        audioPrimitive: { kind: "output", config: { outputType: "master", channels: 2, busTargetId: "bus-main" } },
      },
    ],
    routes: [
      {
        id: "route-1",
        sourceNodeId: "source-1",
        sourcePortId: "source-1-out",
        targetNodeId: "output-1",
        targetPortId: "output-1-in",
        busId: null,
      },
    ],
    buses: [
      {
        id: "bus-main",
        name: "Main",
        channels: 2,
        busType: "main",
        isEnabled: true,
      },
    ],
    macros: [],
    scenes: [
      { id: "scene-1", name: "Main", activeNodeIds: ["source-1", "output-1"], macroOverrides: [] },
    ],
    variations: [],
    ownershipRules: [],
    runtimeStatus: [
      { id: "runtime-audio", runtime: "audio", status: "ready", targetId: "audio-runtime", lastError: null },
      { id: "runtime-visual", runtime: "visual", status: "disconnected", targetId: "visual-runtime", lastError: null },
    ],
    agentFrozen: false,
    pendingActions: [],
    actionHistory: [],
    hardwareBindings: [],
    ...overrides,
  };
}

function createHardwareStatus(overrides: Partial<import("../generated/session-types").HardwareRuntimeStatus> = {}): import("../generated/session-types").HardwareRuntimeStatus {
  return {
    midi: {
      lifecycle: "stopped",
      selectedInputId: null,
      selectedDisplayName: null,
      availableInputCount: 0,
      lastError: null,
    },
    osc: {
      lifecycle: "stopped",
      bindHost: "127.0.0.1",
      listenPort: 9001,
      lastError: null,
    },
    learn: {
      lifecycle: "idle",
      target: null,
      source: null,
    },
    diagnostics: [],
    ...overrides,
  };
}

function createHardwareSettings(overrides: Partial<import("../generated/session-types").HardwareRuntimeSettings> = {}): import("../generated/session-types").HardwareRuntimeSettings {
  return {
    midi: { selectedInputId: null, autoStart: false },
    osc: { bindHost: "127.0.0.1", listenPort: 9001, autoStart: false },
    ...overrides,
  };
}

describe("session projections", () => {
  beforeEach(() => {
    stopHardwarePolling();
    vi.useRealTimers();
    vi.clearAllMocks();
    clientMocks.getHardwareRuntimeSettings.mockResolvedValue(createHardwareSettings());
    clientMocks.getHardwareRuntimeStatus.mockResolvedValue(createHardwareStatus());
    clientMocks.listMidiInputPorts.mockResolvedValue([]);
    useSessionStore.setState({
      session: null,
      selectedNodeId: null,
      graphNodes: [],
      graphEdges: [],
      selectedNode: null,
      audioRuntime: null,
      visualRuntime: null,
      agentRuntime: null,
      isLoading: false,
      error: null,
      workspaceView: "graph",
      conversationMessages: [],
      agentFrozen: false,
      pendingActions: [],
      actionHistory: [],
      hardwareBindings: [],
      hardwareSettings: null,
      hardwareStatus: null,
      midiInputPorts: [],
      midiLearnActive: false,
      midiLearnTarget: null,
    });
  });

  afterEach(() => {
    stopHardwarePolling();
    vi.useRealTimers();
  });

  it("applying a successful graph edit updates projected nodes, edges, and selected inspector data", async () => {
    const initial = createSession();
    const updated = createSession({
      nodes: [
        ...initial.nodes,
        {
          id: "fx-1",
          nodeType: "effect",
          ports: [
            { id: "fx-1-in", name: "signal_in", direction: "input", signalType: "audio" },
            { id: "fx-1-out", name: "signal_out", direction: "output", signalType: "audio" },
          ],
          parameters: [
            {
              id: "fx-1-mix",
              name: "mix",
              value: 0.5,
              defaultValue: 0.5,
              minValue: 0,
              maxValue: 1,
              unit: "ratio",
            },
          ],
          runtimeTarget: "audio/effect/delay",
          sceneMembership: ["scene-1"],
          ownership: { controller: "shared", isLocked: false },
          enabled: true,
          audioPrimitive: { kind: "effect", config: { effectType: "delay", bypassed: false, busTargetId: null } },
        },
      ],
      routes: [
        ...initial.routes,
        {
          id: "route-2",
          sourceNodeId: "source-1",
          sourcePortId: "source-1-out",
          targetNodeId: "fx-1",
          targetPortId: "fx-1-in",
          busId: null,
        },
      ],
    });

    clientMocks.getCurrentSession.mockResolvedValue(initial);
    clientMocks.applyGraphEdit.mockResolvedValue(updated);

    await useSessionStore.getState().bootstrapSession();
    useSessionStore.getState().selectNode("fx-1");
    await useSessionStore.getState().applyGraphEdit({
      type: "addNode",
      payload: { node: updated.nodes[2] },
    });

    const state = useSessionStore.getState();

    expect(state.graphNodes).toHaveLength(3);
    expect(state.graphNodes.every((node) => node.width === 190 && node.height === 74)).toBe(true);
    expect(state.graphEdges).toHaveLength(2);
    expect(state.selectedNode?.id).toBe("fx-1");
    expect(state.selectedNode?.parameters[0]?.name).toBe("mix");
  });

  it("runtime health projection preserves the latest audio status while parameter-only edits avoid unnecessary local graph rebuild assumptions", () => {
    const initial = createSession();
    const firstProjection = projectSessionState(initial, "source-1");
    const updated = createSession({
      audioRuntime: {
        ...initial.audioRuntime,
        lifecycle: "running",
        health: "healthy",
        activePatchId: "patch-1",
      },
      nodes: initial.nodes.map((node) =>
        node.id === "source-1"
          ? {
              ...node,
              parameters: node.parameters.map((parameter) =>
                parameter.id === "source-1-level" ? { ...parameter, value: 0.55 } : parameter,
              ),
            }
          : node,
      ),
    });

    const nextProjection = projectSessionState(updated, "source-1", firstProjection);

    expect(nextProjection.audioRuntime?.health).toBe("healthy");
    expect(nextProjection.audioRuntime?.lifecycle).toBe("running");
    expect(nextProjection.graphNodes).toBe(firstProjection.graphNodes);
    expect(nextProjection.graphEdges).toBe(firstProjection.graphEdges);
    expect(nextProjection.selectedNode?.parameters[0]?.value).toBe(0.55);
  });

  it("projects actionable audio runtime failure details from lastError", () => {
    const session = createSession({
      audioRuntime: {
        ...createSession().audioRuntime,
        lifecycle: "failed",
        health: "degraded",
        lastError:
          "scsynth not found. Install SuperCollider, put `scsynth` on PATH, or set SCRYSYNTH_SCSYNTH_PATH to the full executable path. On macOS Scrysynth also checks the bundle fallback `/Applications/SuperCollider.app/Contents/Resources/scsynth`.",
      },
    });

    const projection = projectSessionState(session, null);

    expect(projection.audioRuntime.status).toBe("failed / degraded");
    expect(projection.audioRuntime.detail).toContain("SCRYSYNTH_SCSYNTH_PATH");
    expect(projection.audioRuntime.detail).toContain("bundle fallback");
  });

  it("keeps stop available when an active audio patch is ready", () => {
    const session = createSession({
      audioRuntime: {
        ...createSession().audioRuntime,
        lifecycle: "ready",
        health: "healthy",
        activePatchId: "patch-v1-test",
      },
    });

    const projection = projectSessionState(session, null);

    expect(projection.audioRuntime.status).toBe("ready / healthy");
    expect(projection.audioRuntime.detail).toBe("Patch patch-v1-test active.");
    expect(projection.audioRuntime.canStart).toBe(false);
    expect(projection.audioRuntime.canStop).toBe(true);
  });

  it("falls back to runtime status audio errors when audio runtime detail is stale", () => {
    const session = createSession({
      audioRuntime: {
        ...createSession().audioRuntime,
        lifecycle: "failed",
        health: "degraded",
        lastError: null,
      },
      runtimeStatus: [
        {
          id: "runtime-audio",
          runtime: "audio",
          status: "error",
          targetId: "audio-runtime",
          lastError:
            "Runtime server error during topology load synthdefs: scsynth did not confirm OSC /sync: /sync 1 timed out after 2s",
        },
      ],
    });

    const projection = projectSessionState(session, null);

    expect(projection.audioRuntime.detail).toContain("Runtime server error");
    expect(projection.audioRuntime.detail).toContain("OSC /sync");
  });

  it("projects missing visual sidecar diagnostics as restartable setup guidance", () => {
    const session = createSession({
      visualRuntime: {
        ...createSession().visualRuntime,
        lifecycle: "failed",
        health: "degraded",
        lastError:
          "visual runtime binary not found; install scrysynth-visual or set SCRYSYNTH_BEVY_PATH",
      },
      runtimeStatus: [
        { id: "runtime-visual", runtime: "visual", status: "error", targetId: "visual-runtime", lastError: null },
      ],
    });

    const projection = projectSessionState(session, null);

    expect(projection.visualRuntime.status).toBe("missing sidecar / restartable");
    expect(projection.visualRuntime.detail).toContain("SCRYSYNTH_BEVY_PATH");
    expect(projection.visualRuntime.detail).toContain("Start again");
    expect(projection.visualRuntime.canStart).toBe(true);
    expect(projection.visualRuntime.isRestartable).toBe(true);
  });

  it("projects ready visual scene, renderer, connection, and telemetry state", () => {
    const session = createSession({
      visualRuntime: {
        lifecycle: "ready",
        health: "healthy",
        activeSceneId: "scene-1",
        fps: 59.7,
        lastError: null,
        renderer: "scrysynth-minimal-visual",
      },
      runtimeStatus: [
        { id: "runtime-visual", runtime: "visual", status: "ready", targetId: "visual-runtime", lastError: null },
      ],
    });

    const projection = projectSessionState(session, null);

    expect(projection.visualRuntime.status).toBe("ready / healthy");
    expect(projection.visualRuntime.detail).toContain("Scene Main (scene-1)");
    expect(projection.visualRuntime.connectionStatus).toBe("ready");
    expect(projection.visualRuntime.activeSceneLabel).toBe("Main (scene-1)");
    expect(projection.visualRuntime.rendererLabel).toBe("scrysynth-minimal-visual");
    expect(projection.visualRuntime.fpsLabel).toBe("60 FPS");
    expect(projection.visualRuntime.canStop).toBe(true);
    expect(projection.visualRuntime.canPanic).toBe(true);
  });

  it("projects stopped and panicked visual runtime states as legible restart paths", () => {
    const stopped = projectSessionState(createSession(), null);

    expect(stopped.visualRuntime.status).toBe("stopped / disconnected");
    expect(stopped.visualRuntime.detail).toContain("Start launches");
    expect(stopped.visualRuntime.canStart).toBe(true);

    const panicked = projectSessionState(createSession({
      visualRuntime: {
        ...createSession().visualRuntime,
        lifecycle: "panicked",
        health: "degraded",
        activeSceneId: "scene-1",
        lastError:
          "visual runtime panic requested; sidecar stopped and can be restarted",
      },
    }), null);

    expect(panicked.visualRuntime.status).toBe("panicked / restartable");
    expect(panicked.visualRuntime.detail).toContain("panic requested");
    expect(panicked.visualRuntime.canStart).toBe(true);
    expect(panicked.visualRuntime.canStop).toBe(false);
  });

  it("deriveSelectedNode returns null when selectedNodeId is null or not found", () => {
    const session = createSession();
    const withNullId = projectSessionState(session, null);
    expect(withNullId.selectedNode).toBeNull();

    const withBadId = projectSessionState(session, "nonexistent-node-id");
    expect(withBadId.selectedNode).toBeNull();
  });

  it("rejected edits keep the previous store state and surface an error banner message", async () => {
    const initial = createSession();

    clientMocks.getCurrentSession.mockResolvedValue(initial);
    clientMocks.applyGraphEdit.mockRejectedValue(new Error("route cycle rejected"));

    await useSessionStore.getState().bootstrapSession();
    const previousNodes = useSessionStore.getState().graphNodes;
    const previousEdges = useSessionStore.getState().graphEdges;

    await useSessionStore.getState().applyGraphEdit({
      type: "removeRoute",
      payload: { route_id: "route-1" },
    });

    const state = useSessionStore.getState();
    expect(state.session?.routes).toHaveLength(1);
    expect(state.graphNodes).toBe(previousNodes);
    expect(state.graphEdges).toBe(previousEdges);
    expect(state.error).toContain("route cycle rejected");
  });
});

describe("performance workspace", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useSessionStore.setState({
      session: null,
      selectedNodeId: null,
      graphNodes: [],
      graphEdges: [],
      selectedNode: null,
      audioRuntime: null,
      visualRuntime: null,
      agentRuntime: null,
      isLoading: false,
      error: null,
      workspaceView: "graph",
      conversationMessages: [],
      agentFrozen: false,
      pendingActions: [],
      actionHistory: [],
    });
  });

  it("scene recall updates the session state through the store", async () => {
    const initial = createSession({
      scenes: [
        { id: "scene-a", name: "Scene A", activeNodeIds: ["source-1", "output-1"], macroOverrides: [] },
        { id: "scene-b", name: "Scene B", activeNodeIds: ["output-1"], macroOverrides: [] },
      ],
    });

    const afterRecall = createSession({
      scenes: [
        { id: "scene-a", name: "Scene A", activeNodeIds: ["source-1", "output-1"], macroOverrides: [] },
        { id: "scene-b", name: "Scene B", activeNodeIds: ["output-1"], macroOverrides: [] },
      ],
      nodes: initial.nodes.map((node) => ({
        ...node,
        enabled: node.id === "output-1",
      })),
    });

    clientMocks.getCurrentSession.mockResolvedValue(initial);
    clientMocks.applyPerformanceCommand.mockResolvedValue(afterRecall);

    await useSessionStore.getState().bootstrapSession();
    await useSessionStore.getState().recallScene("scene-b");

    const state = useSessionStore.getState();
    expect(state.session?.nodes.find((n) => n.id === "source-1")?.enabled).toBe(false);
    expect(state.session?.nodes.find((n) => n.id === "output-1")?.enabled).toBe(true);
    expect(state.error).toBeNull();
  });

  it("variation save adds a new variation to the session", async () => {
    const initial = createSession();
    const withVariation = createSession({
      variations: [
        { id: "var-1", name: "soft", sceneId: "scene-1", parameterOverrides: [{ parameterId: "source-1-level", value: 0.3 }] },
      ],
    });

    clientMocks.getCurrentSession.mockResolvedValue(initial);
    clientMocks.applyPerformanceCommand.mockResolvedValue(withVariation);

    await useSessionStore.getState().bootstrapSession();
    await useSessionStore.getState().saveVariation("soft", "scene-1");

    const state = useSessionStore.getState();
    expect(state.session?.variations).toHaveLength(1);
    expect(state.session?.variations[0]?.name).toBe("soft");
  });

  it("variation restore updates parameters in the session", async () => {
    const initial = createSession({
      variations: [
        { id: "var-1", name: "soft", sceneId: "scene-1", parameterOverrides: [{ parameterId: "source-1-level", value: 0.3 }] },
      ],
    });

    const afterRestore = createSession({
      variations: initial.variations,
      nodes: initial.nodes.map((node) =>
        node.id === "source-1"
          ? {
              ...node,
              parameters: node.parameters.map((p) =>
                p.id === "source-1-level" ? { ...p, value: 0.3 } : p,
              ),
            }
          : node,
      ),
    });

    clientMocks.getCurrentSession.mockResolvedValue(initial);
    clientMocks.applyPerformanceCommand.mockResolvedValue(afterRestore);

    await useSessionStore.getState().bootstrapSession();
    await useSessionStore.getState().restoreVariation("var-1");

    const state = useSessionStore.getState();
    const param = state.session?.nodes
      .find((n) => n.id === "source-1")
      ?.parameters.find((p) => p.id === "source-1-level");
    expect(param?.value).toBe(0.3);
  });

  it("performance command error surfaces error message without changing state", async () => {
    const initial = createSession();

    clientMocks.getCurrentSession.mockResolvedValue(initial);
    clientMocks.applyPerformanceCommand.mockRejectedValue(new Error("scene not found"));

    await useSessionStore.getState().bootstrapSession();
    const previousVariations = useSessionStore.getState().session?.variations ?? [];

    await useSessionStore.getState().recallScene("nonexistent");

    const state = useSessionStore.getState();
    expect(state.error).toContain("scene not found");
    expect(state.session?.variations).toEqual(previousVariations);
  });

  it("deriveActiveSceneId returns the best matching scene", () => {
    const session = createSession({
      scenes: [
        { id: "scene-a", name: "Scene A", activeNodeIds: ["source-1", "output-1"], macroOverrides: [] },
        { id: "scene-b", name: "Scene B", activeNodeIds: ["output-1"], macroOverrides: [] },
      ],
    });

    expect(deriveActiveSceneId(session)).toBe("scene-a");

    const onlyOutput = createSession({
      ...session,
      nodes: session.nodes.map((n) => ({
        ...n,
        enabled: n.id === "output-1",
      })),
    });

    expect(deriveActiveSceneId(onlyOutput)).toBe("scene-b");
  });

  it("view switching changes workspaceView state", () => {
    useSessionStore.setState({ workspaceView: "graph" });
    expect(useSessionStore.getState().workspaceView).toBe("graph");

    useSessionStore.getState().setWorkspaceView("performance");
    expect(useSessionStore.getState().workspaceView).toBe("performance");

    useSessionStore.getState().setWorkspaceView("conversation");
    expect(useSessionStore.getState().workspaceView).toBe("conversation");
  });

  it("bootstrapSession projects hardware settings status diagnostics and MIDI ports", async () => {
    const session = createSession();
    const status = createHardwareStatus({
      midi: {
        lifecycle: "unavailable",
        selectedInputId: null,
        selectedDisplayName: null,
        availableInputCount: 0,
        lastError: "No MIDI devices",
      },
      diagnostics: [
        {
          code: "no_midi_ports",
          message: "No MIDI input ports are available.",
          recoverable: true,
          detail: "Connect a controller.",
        },
      ],
    });
    const settings = createHardwareSettings({
      osc: { bindHost: "0.0.0.0", listenPort: 7777, autoStart: true },
    });
    clientMocks.getCurrentSession.mockResolvedValue(session);
    clientMocks.getHardwareRuntimeSettings.mockResolvedValue(settings);
    clientMocks.getHardwareRuntimeStatus.mockResolvedValue(status);
    clientMocks.listMidiInputPorts.mockResolvedValue([
      { id: "midi-0", displayName: "Launch Control", isSelected: false },
    ]);

    await useSessionStore.getState().bootstrapSession();

    const state = useSessionStore.getState();
    expect(state.hardwareSettings?.osc.listenPort).toBe(7777);
    expect(state.hardwareStatus?.midi.lifecycle).toBe("unavailable");
    expect(state.hardwareStatus?.diagnostics[0]?.message).toContain("No MIDI");
    expect(state.midiInputPorts[0]?.displayName).toBe("Launch Control");
  });

  it("hardware settings and listener actions use the command contract", async () => {
    const settings = createHardwareSettings({
      midi: { selectedInputId: "midi-0", autoStart: true },
      osc: { bindHost: "127.0.0.1", listenPort: 9123, autoStart: true },
    });
    const listening = createHardwareStatus({
      midi: { lifecycle: "listening", selectedInputId: "midi-0", selectedDisplayName: "Keys", availableInputCount: 1, lastError: null },
      osc: { lifecycle: "listening", bindHost: "127.0.0.1", listenPort: 9123, lastError: null },
    });
    const stopped = createHardwareStatus();
    clientMocks.updateHardwareRuntimeSettings.mockResolvedValue(listening);
    clientMocks.listMidiInputPorts.mockResolvedValue([
      { id: "midi-0", displayName: "Keys", isSelected: true },
    ]);
    clientMocks.startHardwareListeners.mockResolvedValue(listening);
    clientMocks.stopHardwareListeners.mockResolvedValue(stopped);

    await useSessionStore.getState().updateHardwareSettings(settings);
    expect(clientMocks.updateHardwareRuntimeSettings).toHaveBeenCalledWith(settings);
    expect(useSessionStore.getState().hardwareStatus?.osc.listenPort).toBe(9123);

    await useSessionStore.getState().startHardwareRuntime();
    expect(clientMocks.startHardwareListeners).toHaveBeenCalled();
    expect(useSessionStore.getState().hardwareStatus?.midi.lifecycle).toBe("listening");

    await useSessionStore.getState().stopHardwareRuntime();
    expect(clientMocks.stopHardwareListeners).toHaveBeenCalled();
    expect(useSessionStore.getState().hardwareStatus?.midi.lifecycle).toBe("stopped");
  });

  it("hardware refresh preserves status and surfaces MIDI enumeration errors", async () => {
    const status = createHardwareStatus({
      midi: {
        lifecycle: "error",
        selectedInputId: "midi-0",
        selectedDisplayName: null,
        availableInputCount: null,
        lastError: "CoreMIDI init failed",
      },
    });
    clientMocks.getHardwareRuntimeStatus.mockResolvedValue(status);
    clientMocks.listMidiInputPorts.mockRejectedValue("CoreMIDI init failed");
    useSessionStore.setState({
      midiInputPorts: [{ id: "midi-0", displayName: "Impact GX61 MIDI1", isSelected: true }],
    });

    await useSessionStore.getState().refreshHardwareRuntime();

    expect(useSessionStore.getState().hardwareStatus?.midi.lifecycle).toBe("error");
    expect(useSessionStore.getState().midiInputPorts).toHaveLength(1);
    expect(useSessionStore.getState().error).toBe("CoreMIDI init failed");
  });

  it("hardware start applies the current draft settings before starting listeners", async () => {
    const settings = createHardwareSettings({
      midi: { selectedInputId: "midi-0", autoStart: false },
    });
    const listening = createHardwareStatus({
      midi: {
        lifecycle: "listening",
        selectedInputId: "midi-0",
        selectedDisplayName: "Impact GX61 MIDI1",
        availableInputCount: 1,
        lastError: null,
      },
    });
    clientMocks.updateHardwareRuntimeSettings.mockResolvedValue(createHardwareStatus());
    clientMocks.startHardwareListeners.mockResolvedValue(listening);
    clientMocks.listMidiInputPorts.mockResolvedValue([
      { id: "midi-0", displayName: "Impact GX61 MIDI1", isSelected: true },
    ]);

    await useSessionStore.getState().startHardwareRuntime(settings);

    expect(clientMocks.updateHardwareRuntimeSettings).toHaveBeenCalledWith(settings);
    expect(clientMocks.startHardwareListeners).toHaveBeenCalled();
    expect(useSessionStore.getState().hardwareSettings?.midi.selectedInputId).toBe("midi-0");
    expect(useSessionStore.getState().hardwareStatus?.midi.selectedDisplayName).toBe("Impact GX61 MIDI1");
  });

  it("hardware actions surface string errors from Tauri commands", async () => {
    clientMocks.startHardwareListeners.mockRejectedValue("Selected MIDI input is no longer available.");

    await useSessionStore.getState().startHardwareRuntime();

    expect(useSessionStore.getState().error).toBe("Selected MIDI input is no longer available.");
  });

  it("learn cancel and binding removal update hardware-facing state", async () => {
    const withBinding = createSession({
      hardwareBindings: [
        {
          id: "binding-1",
          source: { kind: "oscAddress", config: { address: "/scrysynth/energy" } },
          target: { kind: "transportPanic" },
          transform: { inputMin: 0, inputMax: 1, outputMin: 0, outputMax: 1 },
        },
      ],
    });
    const learning = createHardwareStatus({
      learn: { lifecycle: "learning", target: { kind: "transportPlay" }, source: null },
    });
    clientMocks.getCurrentSession.mockResolvedValue(withBinding);
    clientMocks.startHardwareLearn.mockResolvedValue(learning);
    clientMocks.stopHardwareLearn.mockResolvedValue(undefined);
    clientMocks.removeHardwareBinding.mockResolvedValue(createSession({ hardwareBindings: [] }));

    await useSessionStore.getState().bootstrapSession();
    await useSessionStore.getState().startMidiLearn({ kind: "transportPlay" });
    expect(clientMocks.startHardwareLearn).toHaveBeenCalledWith({ kind: "transportPlay" });
    expect(useSessionStore.getState().midiLearnActive).toBe(true);

    await useSessionStore.getState().stopMidiLearn();
    expect(clientMocks.stopHardwareLearn).toHaveBeenCalled();
    expect(useSessionStore.getState().midiLearnActive).toBe(false);

    await useSessionStore.getState().removeHardwareBinding("binding-1");
    expect(clientMocks.removeHardwareBinding).toHaveBeenCalledWith("binding-1");
    expect(useSessionStore.getState().hardwareBindings).toHaveLength(0);
  });

  it("hardware polling predicate follows listeners learn state and bindings", () => {
    expect(shouldPollHardware({
      midiLearnActive: false,
      hardwareBindings: [],
      hardwareStatus: createHardwareStatus(),
    })).toBe(false);

    expect(shouldPollHardware({
      midiLearnActive: false,
      hardwareBindings: [],
      hardwareStatus: createHardwareStatus({
        osc: { lifecycle: "listening", bindHost: "127.0.0.1", listenPort: 9001, lastError: null },
      }),
    })).toBe(true);

    expect(shouldPollHardware({
      midiLearnActive: false,
      hardwareBindings: [
        {
          id: "binding-1",
          source: { kind: "midiCc", config: { channel: 0, controller: 1 } },
          target: { kind: "transportStop" },
          transform: { inputMin: 0, inputMax: 127, outputMin: 0, outputMax: 1 },
        },
      ],
      hardwareStatus: createHardwareStatus(),
    })).toBe(true);
  });

  it("hardware polling clears backend learn state after a binding is captured", async () => {
    vi.useFakeTimers();
    const captured = createSession({
      hardwareBindings: [
        {
          id: "binding-1",
          source: { kind: "midiCc", config: { channel: 0, controller: 7 } },
          target: { kind: "macro", config: { macro_id: "macro-1" } },
          transform: { inputMin: 0, inputMax: 127, outputMin: 0, outputMax: 1 },
        },
      ],
    });
    const learning = createHardwareStatus({
      learn: { lifecycle: "learning", target: { kind: "macro", config: { macro_id: "macro-1" } }, source: null },
    });
    const idle = createHardwareStatus({
      learn: { lifecycle: "idle", target: null, source: null },
    });
    clientMocks.pollHardwareEvents.mockResolvedValue(captured);
    clientMocks.getHardwareRuntimeStatus
      .mockResolvedValueOnce(learning)
      .mockResolvedValueOnce(idle);
    clientMocks.stopHardwareLearn.mockResolvedValue(undefined);
    useSessionStore.setState({
      session: createSession(),
      selectedNodeId: null,
      graphNodes: [],
      graphEdges: [],
      midiLearnActive: true,
      midiLearnTarget: { kind: "macro", config: { macro_id: "macro-1" } },
      hardwareBindings: [],
      hardwareStatus: learning,
    });

    startHardwarePolling();
    await vi.advanceTimersByTimeAsync(120);

    expect(clientMocks.stopHardwareLearn).toHaveBeenCalled();
    expect(useSessionStore.getState().midiLearnActive).toBe(false);
    expect(useSessionStore.getState().hardwareStatus?.learn.lifecycle).toBe("idle");
    expect(useSessionStore.getState().hardwareBindings).toHaveLength(1);
  });
});
