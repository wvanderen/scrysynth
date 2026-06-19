import { beforeEach, describe, expect, it, vi } from "vitest";

import type { AgentIntent, SessionDocument } from "../generated/session-types";

const clientMocks = vi.hoisted(() => ({
  approvePendingAction: vi.fn<(actionId: string) => Promise<SessionDocument>>(),
  applyGraphEdit: vi.fn(),
  applyMacroCommand: vi.fn(),
  applyPerformanceCommand: vi.fn(),
  createDefaultSession: vi.fn(),
  getAgentRuntimeState: vi.fn(),
  getCurrentSession: vi.fn(),
  getHardwareRuntimeSettings: vi.fn(),
  getHardwareRuntimeStatus: vi.fn(),
  listMidiInputPorts: vi.fn(),
  openSessionFromPath: vi.fn(),
  panicAudioRuntime: vi.fn(),
  panicVisualRuntime: vi.fn(),
  pollHardwareEvents: vi.fn(),
  reclaimOwnership: vi.fn<(nodeIds?: string[]) => Promise<SessionDocument>>(),
  rejectPendingAction: vi.fn<(actionId: string) => Promise<SessionDocument>>(),
  removeHardwareBinding: vi.fn(),
  saveSessionToPath: vi.fn(),
  sendAgentMessage: vi.fn<(message: string) => Promise<{ session: SessionDocument; intent: AgentIntent }>>(),
  startAudioRuntime: vi.fn(),
  startHardwareLearn: vi.fn(),
  startHardwareListeners: vi.fn(),
  startVisualRuntime: vi.fn(),
  stopAudioRuntime: vi.fn(),
  stopHardwareListeners: vi.fn(),
  stopHardwareLearn: vi.fn(),
  stopVisualRuntime: vi.fn(),
  toggleAgentFreeze: vi.fn<() => Promise<SessionDocument>>(),
  updateHardwareRuntimeSettings: vi.fn(),
}));

vi.mock("../lib/session-client", () => clientMocks);

import { useSessionStore } from "../store/sessionStore";

function createSession(overrides: Partial<SessionDocument> = {}): SessionDocument {
  return {
    schemaVersion: 1,
    sessionId: "session-1",
    title: "Agent Test Session",
    createdAt: "2026-04-12T00:00:00Z",
    updatedAt: "2026-04-12T00:00:00Z",
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
        ports: [{ id: "source-1-out", name: "main_out", direction: "output", signalType: "audio" }],
        parameters: [
          { id: "source-1-level", name: "level", value: 0.8, defaultValue: 0.8, minValue: 0, maxValue: 1, unit: "linear" },
        ],
        runtimeTarget: "audio/source/default",
        sceneMembership: ["scene-1"],
        ownership: { controller: "shared", isLocked: false },
        enabled: true,
        audioPrimitive: { kind: "source", config: { sourceType: "oscillator", channelMode: "mono", busTargetId: null } },
      },
    ],
    routes: [],
    buses: [],
    macros: [],
    scenes: [{ id: "scene-1", name: "Main", activeNodeIds: ["source-1"], macroOverrides: [] }],
    variations: [],
    ownershipRules: [],
    runtimeStatus: [],
    agentFrozen: false,
    pendingActions: [],
    actionHistory: [],
    hardwareBindings: [],
    ...overrides,
  };
}

describe("agent collaboration store actions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    clientMocks.getHardwareRuntimeSettings.mockResolvedValue({
      midi: { selectedInputId: null, autoStart: false },
      osc: { bindHost: "127.0.0.1", listenPort: 9001, autoStart: false },
    });
    clientMocks.getHardwareRuntimeStatus.mockResolvedValue({
      midi: { lifecycle: "stopped", selectedInputId: null, selectedDisplayName: null, availableInputCount: 0, lastError: null },
      osc: { lifecycle: "stopped", bindHost: "127.0.0.1", listenPort: 9001, lastError: null },
      learn: { lifecycle: "idle", target: null, source: null },
      diagnostics: [],
    });
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

  it("sendAgentMessage adds user and agent messages to conversation", async () => {
    const initial = createSession();
    const intent: AgentIntent = {
      rawInput: "add an oscillator",
      parsedCommands: [
        {
          type: "graphEdit",
          payload: {
            type: "addNode",
            payload: { node: initial.nodes[0] },
          },
        },
      ],
      confidence: 0.92,
    };

    clientMocks.getCurrentSession.mockResolvedValue(initial);
    clientMocks.sendAgentMessage.mockResolvedValue({ session: initial, intent });

    await useSessionStore.getState().bootstrapSession();
    await useSessionStore.getState().sendAgentMessage("add an oscillator");

    const state = useSessionStore.getState();
    expect(state.conversationMessages).toHaveLength(2);
    expect(state.conversationMessages[0]?.role).toBe("user");
    expect(state.conversationMessages[0]?.content).toBe("add an oscillator");
    expect(state.conversationMessages[1]?.role).toBe("agent");
    expect(state.conversationMessages[1]?.intent?.confidence).toBe(0.92);
    expect(state.error).toBeNull();
  });

  it("sendAgentMessage handles errors gracefully", async () => {
    const initial = createSession();
    clientMocks.getCurrentSession.mockResolvedValue(initial);
    clientMocks.sendAgentMessage.mockRejectedValue(new Error("intent parse failed"));

    await useSessionStore.getState().bootstrapSession();
    await useSessionStore.getState().sendAgentMessage("bad input");

    const state = useSessionStore.getState();
    expect(state.conversationMessages).toHaveLength(2);
    expect(state.conversationMessages[1]?.role).toBe("agent");
    expect(state.conversationMessages[1]?.content).toContain("Error");
    expect(state.error).toContain("intent parse failed");
  });

  it("toggleFreezeAgent updates agentFrozen state", async () => {
    const initial = createSession();
    const frozen = createSession({ agentFrozen: true });

    clientMocks.getCurrentSession.mockResolvedValue(initial);
    clientMocks.toggleAgentFreeze.mockResolvedValue(frozen);

    await useSessionStore.getState().bootstrapSession();
    expect(useSessionStore.getState().agentFrozen).toBe(false);

    await useSessionStore.getState().toggleFreezeAgent();
    expect(useSessionStore.getState().agentFrozen).toBe(true);
  });

  it("approvePendingAction removes the action from pending list", async () => {
    const initial = createSession({
      pendingActions: [
        {
          id: "action-1",
          correlationId: "corr-1",
          command: { type: "graphEdit", payload: { type: "addNode", payload: { node: createSession().nodes[0] } } },
          riskTier: "medium",
          createdAt: "2026-04-12T01:00:00Z",
          status: "pending",
        },
      ],
    });
    const approved = createSession({ pendingActions: [] });

    clientMocks.getCurrentSession.mockResolvedValue(initial);
    clientMocks.approvePendingAction.mockResolvedValue(approved);

    await useSessionStore.getState().bootstrapSession();
    expect(useSessionStore.getState().pendingActions).toHaveLength(1);

    await useSessionStore.getState().approvePendingAction("action-1");
    expect(useSessionStore.getState().pendingActions).toHaveLength(0);
  });

  it("rejectPendingAction removes the action from pending list", async () => {
    const initial = createSession({
      pendingActions: [
        {
          id: "action-1",
          correlationId: "corr-1",
          command: { type: "graphEdit", payload: { type: "addNode", payload: { node: createSession().nodes[0] } } },
          riskTier: "high",
          createdAt: "2026-04-12T01:00:00Z",
          status: "pending",
        },
      ],
    });
    const rejected = createSession({ pendingActions: [] });

    clientMocks.getCurrentSession.mockResolvedValue(initial);
    clientMocks.rejectPendingAction.mockResolvedValue(rejected);

    await useSessionStore.getState().bootstrapSession();
    await useSessionStore.getState().rejectPendingAction("action-1");
    expect(useSessionStore.getState().pendingActions).toHaveLength(0);
  });

  it("reclaimOwnership updates session", async () => {
    const initial = createSession();
    const reclaimed = createSession({
      nodes: initial.nodes.map((n) => ({
        ...n,
        ownership: { controller: "user", isLocked: false },
      })),
    });

    clientMocks.getCurrentSession.mockResolvedValue(initial);
    clientMocks.reclaimOwnership.mockResolvedValue(reclaimed);

    await useSessionStore.getState().bootstrapSession();
    await useSessionStore.getState().reclaimOwnership(["source-1"]);

    const node = useSessionStore.getState().session?.nodes[0];
    expect(node?.ownership.controller).toBe("user");
  });

  it("actionHistory is projected from session", async () => {
    const initial = createSession({
      actionHistory: [
        {
          id: "history-1",
          timestamp: "2026-04-12T01:00:00Z",
          actor: { actorId: "agent", correlationId: "corr-1" },
          command: { type: "graphEdit", payload: { type: "addNode", payload: { node: createSession().nodes[0] } } },
          diff: {
            description: "Added oscillator node",
            affectedNodeIds: ["source-1"],
            beforeSnippet: "",
            afterSnippet: "source-1",
          },
        },
      ],
    });

    clientMocks.getCurrentSession.mockResolvedValue(initial);

    await useSessionStore.getState().bootstrapSession();
    expect(useSessionStore.getState().actionHistory).toHaveLength(1);
    expect(useSessionStore.getState().actionHistory[0]?.actor.actorId).toBe("agent");
  });
});
