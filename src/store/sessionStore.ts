import { create } from "zustand";

import type { GraphEditCommand, Node, SessionDocument } from "../generated/session-types";
import {
  applyGraphEdit as applyGraphEditCommand,
  createDefaultSession,
  getCurrentSession,
  openSessionFromPath,
  panicAudioRuntime,
  saveSessionToPath,
  startAudioRuntime,
  stopAudioRuntime,
} from "../lib/session-client";
import {
  type AudioRuntimeProjection,
  type GraphEdge,
  type GraphNode,
  projectSessionState,
} from "./session-projections";

type SessionStore = {
  session: SessionDocument | null;
  selectedNodeId: string | null;
  graphNodes: GraphNode[];
  graphEdges: GraphEdge[];
  selectedNode: Node | null;
  audioRuntime: AudioRuntimeProjection | null;
  isLoading: boolean;
  error: string | null;
  bootstrapSession: () => Promise<void>;
  newSession: () => Promise<void>;
  saveSession: (path: string) => Promise<void>;
  openSession: (path: string) => Promise<void>;
  selectNode: (nodeId: string | null) => void;
  applyGraphEdit: (command: GraphEditCommand) => Promise<void>;
  updateNodeParameter: (nodeId: string, parameterId: string, value: number) => Promise<void>;
  toggleNodeEnabled: (nodeId: string, enabled: boolean) => Promise<void>;
  startAudio: () => Promise<void>;
  stopAudio: () => Promise<void>;
  panicAudio: () => Promise<void>;
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
    topologySignature: projection.topologySignature,
  };
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

export const useSessionStore = create<SessionStore>((set, get) => ({
  session: null,
  selectedNodeId: null,
  graphNodes: [],
  graphEdges: [],
  selectedNode: null,
  audioRuntime: null,
  isLoading: false,
  error: null,
  bootstrapSession: async () => {
    set({ isLoading: true, error: null });

    try {
      const session = await getCurrentSession();
      const current = get();
      set({
        ...applySession(session, current.selectedNodeId, current),
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
}));
