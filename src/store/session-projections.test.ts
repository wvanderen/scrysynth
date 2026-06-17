import { beforeEach, describe, expect, it, vi } from "vitest";

import type { GraphEditCommand, SessionDocument } from "../generated/session-types";

const clientMocks = vi.hoisted(() => ({
  approvePendingAction: vi.fn(),
  applyGraphEdit: vi.fn<(command: GraphEditCommand) => Promise<SessionDocument>>(),
  applyMacroCommand: vi.fn(),
  applyPerformanceCommand: vi.fn<(command: import("../generated/session-types").PerformanceCommand) => Promise<SessionDocument>>(),
  createDefaultSession: vi.fn<() => Promise<SessionDocument>>(),
  getAgentRuntimeState: vi.fn(),
  getCurrentSession: vi.fn<() => Promise<SessionDocument>>(),
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
  startVisualRuntime: vi.fn<() => Promise<SessionDocument>>(),
  stopAudioRuntime: vi.fn<() => Promise<SessionDocument>>(),
  stopHardwareLearn: vi.fn(),
  stopVisualRuntime: vi.fn<() => Promise<SessionDocument>>(),
  toggleAgentFreeze: vi.fn(),
}));

vi.mock("../lib/session-client", () => clientMocks);

import { projectSessionState, deriveActiveSceneId } from "./session-projections";
import { useSessionStore } from "./sessionStore";

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

describe("session projections", () => {
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
});
