import { create } from "zustand";

import type {
  ActionHistoryEntry,
  AgentIntent,
  AgentRuntimeState,
  BindingTarget,
  ControllerKind,
  GraphEditCommand,
  HardwareBinding,
  HardwareRuntimeSettings,
  HardwareRuntimeStatus,
  MacroCommand,
  MacroDefinition,
  MacroTarget,
  MidiInputPort,
  Node,
  PendingAction,
  PerformanceCommand,
  Route,
  SessionDocument,
} from "../generated/session-types";
import {
  applyGraphEdit as applyGraphEditCommand,
  applyMacroCommand as applyMacroCommandIPC,
  applyPerformanceCommand as applyPerformanceCommandIPC,
  approvePendingAction as approvePendingActionIPC,
  createDefaultSession,
  getCurrentSession,
  getAgentRuntimeState,
  getHardwareRuntimeSettings,
  getHardwareRuntimeStatus,
  listMidiInputPorts,
  openSessionFromPath,
  panicAudioRuntime,
  panicVisualRuntime as panicVisualRuntimeIPC,
  pollHardwareEvents as pollHardwareEventsIPC,
  reclaimOwnership as reclaimOwnershipIPC,
  rejectPendingAction as rejectPendingActionIPC,
  removeHardwareBinding as removeHardwareBindingIPC,
  saveSessionToPath,
  sendAgentMessage as sendAgentMessageIPC,
  startAudioRuntime,
  startHardwareLearn as startHardwareLearnIPC,
  startHardwareListeners,
  startVisualRuntime as startVisualRuntimeIPC,
  stopAudioRuntime,
  stopHardwareLearn as stopHardwareLearnIPC,
  stopHardwareListeners,
  stopVisualRuntime as stopVisualRuntimeIPC,
  toggleAgentFreeze as toggleAgentFreezeIPC,
  updateHardwareRuntimeSettings,
} from "../lib/session-client";
import {
  type AgentRuntimeProjection,
  type AudioRuntimeProjection,
  type GraphEdge,
  type GraphNode,
  type MacroProjection,
  type VisualRuntimeProjection,
  projectSessionState,
} from "./session-projections";

export type WorkspaceView = "graph" | "performance" | "conversation" | "runtime" | "hardware";

export type ConversationMessage = {
  id: string;
  role: "user" | "agent";
  content: string;
  timestamp: string;
  intent?: AgentIntent;
};

type SessionStore = {
  session: SessionDocument | null;
  selectedNodeId: string | null;
  graphNodes: GraphNode[];
  graphEdges: GraphEdge[];
  selectedNode: Node | null;
  audioRuntime: AudioRuntimeProjection | null;
  visualRuntime: VisualRuntimeProjection | null;
  agentRuntime: AgentRuntimeProjection | null;
  macros: MacroProjection[];
  hardwareBindings: HardwareBinding[];
  hardwareSettings: HardwareRuntimeSettings | null;
  hardwareStatus: HardwareRuntimeStatus | null;
  midiInputPorts: MidiInputPort[];
  midiLearnActive: boolean;
  midiLearnTarget: BindingTarget | null;
  isLoading: boolean;
  error: string | null;
  workspaceView: WorkspaceView;
  conversationMessages: ConversationMessage[];
  agentFrozen: boolean;
  pendingActions: PendingAction[];
  actionHistory: ActionHistoryEntry[];
  bootstrapSession: () => Promise<void>;
  newSession: () => Promise<void>;
  saveSession: (path: string) => Promise<void>;
  openSession: (path: string) => Promise<void>;
  selectNode: (nodeId: string | null) => void;
  applyGraphEdit: (command: GraphEditCommand) => Promise<void>;
  addNode: (node: Node) => Promise<void>;
  removeNode: (nodeId: string) => Promise<void>;
  connectNodes: (sourceNodeId: string, targetNodeId: string) => Promise<void>;
  assignNodeToBus: (nodeId: string, busId: string) => Promise<void>;
  clearNodeBusAssignment: (nodeId: string) => Promise<void>;
  updateNodeParameter: (nodeId: string, parameterId: string, value: number) => Promise<void>;
  toggleNodeEnabled: (nodeId: string, enabled: boolean) => Promise<void>;
  startAudio: () => Promise<void>;
  stopAudio: () => Promise<void>;
  panicAudio: () => Promise<void>;
  startVisual: () => Promise<void>;
  stopVisual: () => Promise<void>;
  panicVisual: () => Promise<void>;
  refreshAgentRuntime: () => Promise<void>;
  setWorkspaceView: (view: WorkspaceView) => void;
  recallScene: (sceneId: string) => Promise<void>;
  saveVariation: (name: string, sceneId: string) => Promise<void>;
  restoreVariation: (variationId: string) => Promise<void>;
  sendAgentMessage: (message: string) => Promise<void>;
  toggleFreezeAgent: () => Promise<void>;
  reclaimOwnership: (nodeIds?: string[], targetController?: ControllerKind) => Promise<void>;
  setNodeOwnership: (nodeIds: string[], targetController: ControllerKind) => Promise<void>;
  approvePendingAction: (actionId: string) => Promise<void>;
  rejectPendingAction: (actionId: string) => Promise<void>;
  createMacro: (definition: MacroDefinition) => Promise<void>;
  updateMacro: (macroId: string, updates: { name?: string; targets?: MacroTarget[]; rangeStart?: number; rangeEnd?: number }) => Promise<void>;
  removeMacro: (macroId: string) => Promise<void>;
  setMacroValue: (macroId: string, value: number) => Promise<void>;
  refreshHardwareRuntime: () => Promise<void>;
  updateHardwareSettings: (settings: HardwareRuntimeSettings) => Promise<void>;
  startHardwareRuntime: (settings?: HardwareRuntimeSettings) => Promise<void>;
  stopHardwareRuntime: () => Promise<void>;
  startMidiLearn: (target: BindingTarget) => Promise<void>;
  stopMidiLearn: () => Promise<void>;
  removeHardwareBinding: (bindingId: string) => Promise<void>;
};

function applySession(
  session: SessionDocument,
  selectedNodeId: string | null,
  previous: {
    selectedNodeId: string | null;
    graphNodes: GraphNode[];
    graphEdges: GraphEdge[];
    topologySignature?: string;
  },
) {
  const projection = projectSessionState(session, selectedNodeId, previous.topologySignature
    ? {
        selectedNodeId: previous.selectedNodeId,
        graphNodes: previous.graphNodes,
        graphEdges: previous.graphEdges,
        topologySignature: previous.topologySignature,
      }
    : undefined);

  return {
    session: projection.session,
    selectedNodeId: projection.selectedNodeId,
    selectedNode: projection.selectedNode,
    graphNodes: projection.graphNodes,
    graphEdges: projection.graphEdges,
    audioRuntime: projection.audioRuntime,
    visualRuntime: projection.visualRuntime,
    agentRuntime: projection.agentRuntime,
    macros: projection.macros,
    topologySignature: projection.topologySignature,
    agentFrozen: session.agentFrozen,
    pendingActions: session.pendingActions,
    actionHistory: session.actionHistory,
    hardwareBindings: session.hardwareBindings ?? [],
  };
}

function errorMessage(error: unknown, fallback: string): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string" && error.trim().length > 0) return error;
  return fallback;
}

function nextSelectedNodeIdForCommand(
  command: GraphEditCommand,
  currentSelectedNodeId: string | null,
): string | null {
  switch (command.type) {
    case "addNode":
      return command.payload.node.id;
    case "removeNode":
      return currentSelectedNodeId === command.payload.node_id ? null : currentSelectedNodeId;
    default:
      return currentSelectedNodeId;
  }
}

function createRouteFromNodes(session: SessionDocument, sourceNodeId: string, targetNodeId: string): Route | null {
  const sourceNode = session.nodes.find((node) => node.id === sourceNodeId);
  const targetNode = session.nodes.find((node) => node.id === targetNodeId);

  if (!sourceNode || !targetNode) {
    return null;
  }

  const sourcePort = sourceNode.ports.find((port) => port.direction === "output" && port.signalType === "audio");
  const targetPort = targetNode.ports.find((port) => port.direction === "input" && port.signalType === "audio");

  if (!sourcePort || !targetPort) {
    return null;
  }

  return {
    id: `route-${globalThis.crypto.randomUUID()}`,
    sourceNodeId,
    sourcePortId: sourcePort.id,
    targetNodeId,
    targetPortId: targetPort.id,
    busId: null,
  };
}

function projectAgentRuntimeFromState(state: AgentRuntimeState): import("./session-projections").AgentRuntimeProjection {
  let status = "Available";
  if (state.isFrozen) {
    status = "Frozen";
  } else if (state.pendingActionCount > 0) {
    status = `${state.pendingActionCount} pending action(s)`;
  }

  return {
    isAvailable: state.isAvailable,
    pendingActionCount: state.pendingActionCount,
    isFrozen: state.isFrozen,
    status,
  };
}

export const useSessionStore = create<SessionStore>((set, get) => ({
  session: null,
  selectedNodeId: null,
  graphNodes: [],
  graphEdges: [],
  selectedNode: null,
  audioRuntime: null,
  visualRuntime: null,
  agentRuntime: null,
  macros: [],
  isLoading: false,
  error: null,
  workspaceView: "graph" as WorkspaceView,
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
  bootstrapSession: async () => {
    set({ isLoading: true, error: null });

    try {
      const session = await getCurrentSession();
      const [hardwareSettings, hardwareStatus, midiInputPorts] = await Promise.all([
        getHardwareRuntimeSettings().catch(() => null),
        getHardwareRuntimeStatus().catch(() => null),
        listMidiInputPorts().catch(() => []),
      ]);
      const current = get();
      set({
        ...applySession(session, current.selectedNodeId, current),
        hardwareSettings,
        hardwareStatus,
        midiInputPorts,
        midiLearnActive: hardwareStatus?.learn.lifecycle === "learning",
        midiLearnTarget: hardwareStatus?.learn.target ?? null,
        isLoading: false,
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to load session.";
      set({ isLoading: false, error: message });
    }
  },
  newSession: async () => {
    set({ isLoading: true, error: null });

    try {
      const session = await createDefaultSession();
      const current = get();
      set({ ...applySession(session, null, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to create a session.";
      set({ isLoading: false, error: message });
    }
  },
  saveSession: async (path: string) => {
    set({ isLoading: true, error: null });

    try {
      await saveSessionToPath(path);
      set({ isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to save the session.";
      set({ isLoading: false, error: message });
    }
  },
  openSession: async (path: string) => {
    set({ isLoading: true, error: null });

    try {
      const session = await openSessionFromPath(path);
      const current = get();
      set({ ...applySession(session, null, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to open the session.";
      set({ isLoading: false, error: message });
    }
  },
  selectNode: (nodeId) => {
    const session = get().session;
    if (!session) {
      return;
    }

    set(applySession(session, nodeId, get()));
  },
  applyGraphEdit: async (command) => {
    set({ isLoading: true, error: null });

    try {
      const session = await applyGraphEditCommand(command);
      const current = get();
      const selectedNodeId = nextSelectedNodeIdForCommand(command, current.selectedNodeId);
      set({
        ...applySession(session, selectedNodeId, current),
        isLoading: false,
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to apply graph edit.";
      set({ isLoading: false, error: message });
    }
  },
  addNode: async (node) => {
    await get().applyGraphEdit({
      type: "addNode",
      payload: { node },
    });
  },
  removeNode: async (nodeId) => {
    await get().applyGraphEdit({
      type: "removeNode",
      payload: { node_id: nodeId },
    });
  },
  connectNodes: async (sourceNodeId, targetNodeId) => {
    const session = get().session;
    if (!session || sourceNodeId === targetNodeId) {
      return;
    }

    const route = createRouteFromNodes(session, sourceNodeId, targetNodeId);
    if (!route) {
      set({ error: "Unable to connect those nodes with the supported audio ports." });
      return;
    }

    const existingIncomingRoute = session.routes.find(
      (candidate) =>
        candidate.targetNodeId === route.targetNodeId &&
        candidate.targetPortId === route.targetPortId &&
        candidate.sourceNodeId !== route.sourceNodeId,
    );

    await get().applyGraphEdit({
      type: "addRoute",
      payload: { route },
    });

    if (get().error || !existingIncomingRoute) {
      return;
    }

    await get().applyGraphEdit({
      type: "removeRoute",
      payload: { route_id: existingIncomingRoute.id },
    });
  },
  assignNodeToBus: async (nodeId, busId) => {
    await get().applyGraphEdit({
      type: "assignNodeToBus",
      payload: {
        node_id: nodeId,
        bus_id: busId,
      },
    });
  },
  clearNodeBusAssignment: async (nodeId) => {
    await get().applyGraphEdit({
      type: "clearNodeBusAssignment",
      payload: { node_id: nodeId },
    });
  },
  updateNodeParameter: async (nodeId, parameterId, value) => {
    await get().applyGraphEdit({
      type: "setParameterValue",
      payload: {
        node_id: nodeId,
        parameter_id: parameterId,
        value,
      },
    });
  },
  toggleNodeEnabled: async (nodeId, enabled) => {
    await get().applyGraphEdit({
      type: "setNodeEnabled",
      payload: {
        node_id: nodeId,
        enabled,
      },
    });
  },
  startAudio: async () => {
    set({ isLoading: true, error: null });

    try {
      const session = await startAudioRuntime();
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to start audio.";
      set({ isLoading: false, error: message });
    }
  },
  stopAudio: async () => {
    set({ isLoading: true, error: null });

    try {
      const session = await stopAudioRuntime();
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to stop audio.";
      set({ isLoading: false, error: message });
    }
  },
  panicAudio: async () => {
    set({ isLoading: true, error: null });

    try {
      const session = await panicAudioRuntime();
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to panic audio runtime.";
      set({ isLoading: false, error: message });
    }
  },
  startVisual: async () => {
    set({ isLoading: true, error: null });

    try {
      const session = await startVisualRuntimeIPC();
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to start visual runtime.";
      set({ isLoading: false, error: message });
    }
  },
  stopVisual: async () => {
    set({ isLoading: true, error: null });

    try {
      const session = await stopVisualRuntimeIPC();
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to stop visual runtime.";
      set({ isLoading: false, error: message });
    }
  },
  panicVisual: async () => {
    set({ isLoading: true, error: null });

    try {
      const session = await panicVisualRuntimeIPC();
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to panic visual runtime.";
      set({ isLoading: false, error: message });
    }
  },
  refreshAgentRuntime: async () => {
    try {
      const agentState = await getAgentRuntimeState();
      set({ agentRuntime: projectAgentRuntimeFromState(agentState) });
    } catch {
      // agent state refresh is non-critical
    }
  },
  setWorkspaceView: (view) => {
    set({ workspaceView: view });
  },
  recallScene: async (sceneId) => {
    set({ isLoading: true, error: null });

    try {
      const command: PerformanceCommand = { type: "recallScene", payload: { scene_id: sceneId } };
      const session = await applyPerformanceCommandIPC(command);
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to recall scene.";
      set({ isLoading: false, error: message });
    }
  },
  saveVariation: async (name, sceneId) => {
    set({ isLoading: true, error: null });

    try {
      const command: PerformanceCommand = { type: "saveVariation", payload: { name, scene_id: sceneId } };
      const session = await applyPerformanceCommandIPC(command);
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to save variation.";
      set({ isLoading: false, error: message });
    }
  },
  restoreVariation: async (variationId) => {
    set({ isLoading: true, error: null });

    try {
      const command: PerformanceCommand = { type: "restoreVariation", payload: { variation_id: variationId } };
      const session = await applyPerformanceCommandIPC(command);
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to restore variation.";
      set({ isLoading: false, error: message });
    }
  },
  sendAgentMessage: async (message) => {
    const userMessage: ConversationMessage = {
      id: globalThis.crypto.randomUUID(),
      role: "user",
      content: message,
      timestamp: new Date().toISOString(),
    };

    set((state) => ({
      conversationMessages: [...state.conversationMessages, userMessage],
      isLoading: true,
      error: null,
    }));

    try {
      const { session, intent } = await sendAgentMessageIPC(message);
      const agentMessage: ConversationMessage = {
        id: globalThis.crypto.randomUUID(),
        role: "agent",
        content: intent.parsedCommands.length > 0
          ? `Understood ${intent.parsedCommands.length} command(s).`
          : "No commands parsed from that input.",
        timestamp: new Date().toISOString(),
        intent,
      };
      const current = get();
      set({
        ...applySession(session, current.selectedNodeId, current),
        conversationMessages: [...current.conversationMessages, agentMessage],
        isLoading: false,
      });
    } catch (error) {
      const errMessage = error instanceof Error ? error.message : "Unable to send message.";
      const agentMessage: ConversationMessage = {
        id: globalThis.crypto.randomUUID(),
        role: "agent",
        content: `Error: ${errMessage}`,
        timestamp: new Date().toISOString(),
      };
      set((state) => ({
        conversationMessages: [...state.conversationMessages, agentMessage],
        isLoading: false,
        error: errMessage,
      }));
    }
  },
  toggleFreezeAgent: async () => {
    set({ isLoading: true, error: null });

    try {
      const session = await toggleAgentFreezeIPC();
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to toggle agent freeze.";
      set({ isLoading: false, error: message });
    }
  },
  reclaimOwnership: async (nodeIds) => {
    set({ isLoading: true, error: null });

    try {
      const session = await reclaimOwnershipIPC(nodeIds);
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to reclaim ownership.";
      set({ isLoading: false, error: message });
    }
  },
  setNodeOwnership: async (nodeIds, targetController) => {
    set({ isLoading: true, error: null });

    try {
      const session = await reclaimOwnershipIPC(nodeIds, targetController);
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to set ownership.";
      set({ isLoading: false, error: message });
    }
  },
  approvePendingAction: async (actionId) => {
    set({ isLoading: true, error: null });

    try {
      const session = await approvePendingActionIPC(actionId);
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to approve action.";
      set({ isLoading: false, error: message });
    }
  },
  rejectPendingAction: async (actionId) => {
    set({ isLoading: true, error: null });

    try {
      const session = await rejectPendingActionIPC(actionId);
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to reject action.";
      set({ isLoading: false, error: message });
    }
  },
  createMacro: async (definition) => {
    set({ isLoading: true, error: null });

    try {
      const command: MacroCommand = { type: "createMacro", payload: { definition } };
      const session = await applyMacroCommandIPC(command);
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to create macro.";
      set({ isLoading: false, error: message });
    }
  },
  updateMacro: async (macroId, updates) => {
    set({ isLoading: true, error: null });

    try {
      const command: MacroCommand = {
        type: "updateMacro",
        payload: {
          macro_id: macroId,
          name: updates.name ?? null,
          targets: updates.targets ?? null,
          range_start: updates.rangeStart ?? null,
          range_end: updates.rangeEnd ?? null,
        },
      };
      const session = await applyMacroCommandIPC(command);
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to update macro.";
      set({ isLoading: false, error: message });
    }
  },
  removeMacro: async (macroId) => {
    set({ isLoading: true, error: null });

    try {
      const command: MacroCommand = { type: "removeMacro", payload: { macro_id: macroId } };
      const session = await applyMacroCommandIPC(command);
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to remove macro.";
      set({ isLoading: false, error: message });
    }
  },
  setMacroValue: async (macroId, value) => {
    try {
      const command: MacroCommand = { type: "setMacroValue", payload: { macro_id: macroId, value } };
      const session = await applyMacroCommandIPC(command);
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current) });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to set macro value.";
      set({ error: message });
    }
  },
  refreshHardwareRuntime: async () => {
    try {
      const hardwareSettings = await getHardwareRuntimeSettings();
      let midiInputPorts = get().midiInputPorts;
      let midiPortError: unknown = null;

      try {
        midiInputPorts = await listMidiInputPorts();
      } catch (error) {
        midiPortError = error;
      }

      const hardwareStatus = await getHardwareRuntimeStatus();
      set({
        hardwareSettings,
        hardwareStatus,
        midiInputPorts,
        midiLearnActive: hardwareStatus.learn.lifecycle === "learning",
        midiLearnTarget: hardwareStatus.learn.lifecycle === "learning" ? hardwareStatus.learn.target : null,
        error: midiPortError ? errorMessage(midiPortError, "Unable to refresh MIDI inputs.") : null,
      });
      ensureHardwarePolling();
    } catch (error) {
      const message = errorMessage(error, "Unable to refresh hardware status.");
      set({ error: message });
    }
  },
  updateHardwareSettings: async (settings) => {
    try {
      const hardwareStatus = await updateHardwareRuntimeSettings(settings);
      const midiInputPorts = await listMidiInputPorts().catch(() => get().midiInputPorts);
      set({ hardwareSettings: settings, hardwareStatus, midiInputPorts });
      ensureHardwarePolling();
    } catch (error) {
      const message = errorMessage(error, "Unable to update hardware settings.");
      set({ error: message });
    }
  },
  startHardwareRuntime: async (settings) => {
    try {
      if (settings) {
        await updateHardwareRuntimeSettings(settings);
        set({ hardwareSettings: settings });
      }
      const hardwareStatus = await startHardwareListeners();
      const midiInputPorts = await listMidiInputPorts().catch(() => get().midiInputPorts);
      set({ hardwareStatus, midiInputPorts });
      ensureHardwarePolling();
    } catch (error) {
      const message = errorMessage(error, "Unable to start hardware listeners.");
      set({ error: message });
    }
  },
  stopHardwareRuntime: async () => {
    try {
      const hardwareStatus = await stopHardwareListeners();
      set({
        hardwareStatus,
        midiLearnActive: false,
        midiLearnTarget: null,
      });
      ensureHardwarePolling();
    } catch (error) {
      const message = errorMessage(error, "Unable to stop hardware listeners.");
      set({ error: message });
    }
  },
  startMidiLearn: async (target) => {
    try {
      const hardwareStatus = await startHardwareLearnIPC(target);
      set({ hardwareStatus, midiLearnActive: true, midiLearnTarget: target });
      ensureHardwarePolling();
    } catch (error) {
      const message = errorMessage(error, "Unable to start hardware learn.");
      set({ error: message });
    }
  },
  stopMidiLearn: async () => {
    try {
      await stopHardwareLearnIPC();
      set({ midiLearnActive: false, midiLearnTarget: null });
      void get().refreshHardwareRuntime();
      ensureHardwarePolling();
    } catch (error) {
      const message = errorMessage(error, "Unable to stop hardware learn.");
      set({ error: message });
    }
  },
  removeHardwareBinding: async (bindingId) => {
    try {
      const session = await removeHardwareBindingIPC(bindingId);
      const current = get();
      set({ ...applySession(session, current.selectedNodeId, current) });
      ensureHardwarePolling();
    } catch (error) {
      const message = errorMessage(error, "Unable to remove binding.");
      set({ error: message });
    }
  },
}));

let hardwarePollInterval: ReturnType<typeof setInterval> | null = null;

export function shouldPollHardware(state: Pick<SessionStore, "midiLearnActive" | "hardwareBindings" | "hardwareStatus">): boolean {
  const status = state.hardwareStatus;
  return Boolean(
    state.midiLearnActive ||
      (state.hardwareBindings ?? []).length > 0 ||
      status?.learn.lifecycle === "learning" ||
      status?.midi.lifecycle === "listening" ||
      status?.midi.lifecycle === "restarting" ||
      status?.osc.lifecycle === "listening" ||
      status?.osc.lifecycle === "restarting",
  );
}

export function ensureHardwarePolling() {
  const state = useSessionStore.getState();
  if (shouldPollHardware(state)) {
    startHardwarePolling();
  } else {
    stopHardwarePolling();
  }
}

export function startHardwarePolling() {
  if (hardwarePollInterval !== null) {
    return;
  }
  hardwarePollInterval = setInterval(async () => {
    const state = useSessionStore.getState();
    if (!shouldPollHardware(state)) {
      stopHardwarePolling();
      return;
    }
    try {
      const [session, hardwareStatus] = await Promise.all([
        pollHardwareEventsIPC(),
        getHardwareRuntimeStatus().catch(() => state.hardwareStatus),
      ]);
      const current = useSessionStore.getState();
      const hadLearnActive = current.midiLearnActive;
      const newBindings = session.hardwareBindings ?? [];
      const prevBindings = current.hardwareBindings ?? [];

      if (hadLearnActive && newBindings.length > prevBindings.length) {
        await stopHardwareLearnIPC().catch(() => undefined);
        const clearedHardwareStatus = await getHardwareRuntimeStatus().catch(() => hardwareStatus);
        useSessionStore.setState({
          ...applySession(session, current.selectedNodeId, current),
          hardwareStatus: clearedHardwareStatus,
          midiLearnActive: false,
          midiLearnTarget: null,
        });
      } else {
        useSessionStore.setState({
          ...applySession(session, current.selectedNodeId, current),
          hardwareStatus,
          midiLearnActive: hardwareStatus?.learn.lifecycle === "learning",
          midiLearnTarget: hardwareStatus?.learn.lifecycle === "learning" ? hardwareStatus.learn.target : current.midiLearnTarget,
        });
      }
    } catch {
      // polling errors are non-critical
    }
  }, 100);
}

export function stopHardwarePolling() {
  if (hardwarePollInterval !== null) {
    clearInterval(hardwarePollInterval);
    hardwarePollInterval = null;
  }
}
