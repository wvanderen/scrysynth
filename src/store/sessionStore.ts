import { create } from "zustand";

import type { Edge, Node as FlowNode } from "@xyflow/react";

import type { Node, SessionDocument } from "../generated/session-types";
import {
  createDefaultSession,
  getCurrentSession,
  openSessionFromPath,
  saveSessionToPath,
} from "../lib/session-client";

type GraphNodeData = {
  title: string;
  subtitle: string;
  isSelected: boolean;
};

type GraphNode = FlowNode<GraphNodeData>;
type GraphEdge = Edge;

type SessionStore = {
  session: SessionDocument | null;
  selectedNodeId: string | null;
  graphNodes: GraphNode[];
  graphEdges: GraphEdge[];
  selectedNode: Node | null;
  isLoading: boolean;
  error: string | null;
  bootstrapSession: () => Promise<void>;
  newSession: () => Promise<void>;
  saveSession: (path: string) => Promise<void>;
  openSession: (path: string) => Promise<void>;
  selectNode: (nodeId: string | null) => void;
};

function projectGraphNodes(session: SessionDocument, selectedNodeId: string | null): GraphNode[] {
  return session.nodes.map((node, index) => ({
    id: node.id,
    position: {
      x: 80 + (index % 3) * 260,
      y: 72 + Math.floor(index / 3) * 180,
    },
    draggable: false,
    selectable: true,
    data: {
      title: labelForNode(node),
      subtitle: `${node.nodeType} / ${node.ownership.controller}`,
      isSelected: selectedNodeId === node.id,
    },
    style: {
      borderRadius: 18,
      border: selectedNodeId === node.id ? "1px solid #f7c66a" : "1px solid #2d4442",
      background: selectedNodeId === node.id ? "#173734" : "#112725",
      color: "#f2eee5",
      padding: 16,
      width: 190,
      boxShadow: "0 20px 40px rgba(5, 12, 12, 0.35)",
    },
  }));
}

function projectGraphEdges(session: SessionDocument): GraphEdge[] {
  return session.routes.map((route) => ({
    id: route.id,
    source: route.sourceNodeId,
    target: route.targetNodeId,
    animated: false,
    style: {
      stroke: "#f7c66a",
      strokeWidth: 2,
    },
    label: route.busId ? "bus" : undefined,
    labelStyle: {
      fill: "#d9c8a0",
      fontSize: 12,
    },
  }));
}

function labelForNode(node: Node) {
  const primaryPort = node.ports[0]?.name ?? "portless";
  return `${node.nodeType}:${primaryPort}`;
}

function deriveSelectedNode(session: SessionDocument, selectedNodeId: string | null): Node | null {
  if (!selectedNodeId) {
    return session.nodes[0] ?? null;
  }

  return session.nodes.find((node) => node.id === selectedNodeId) ?? session.nodes[0] ?? null;
}

function applySession(session: SessionDocument, selectedNodeId: string | null) {
  const selectedNode = deriveSelectedNode(session, selectedNodeId);
  const resolvedSelectedNodeId = selectedNode?.id ?? null;

  return {
    session,
    selectedNodeId: resolvedSelectedNodeId,
    selectedNode,
    graphNodes: projectGraphNodes(session, resolvedSelectedNodeId),
    graphEdges: projectGraphEdges(session),
  };
}

export const useSessionStore = create<SessionStore>((set, get) => ({
  session: null,
  selectedNodeId: null,
  graphNodes: [],
  graphEdges: [],
  selectedNode: null,
  isLoading: false,
  error: null,
  bootstrapSession: async () => {
    set({ isLoading: true, error: null });

    try {
      const session = await getCurrentSession();
      set({ ...applySession(session, get().selectedNodeId), isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unable to load session.";
      set({ isLoading: false, error: message });
    }
  },
  newSession: async () => {
    set({ isLoading: true, error: null });

    try {
      const session = await createDefaultSession();
      set({ ...applySession(session, null), isLoading: false });
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
      set({ ...applySession(session, null), isLoading: false });
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

    set(applySession(session, nodeId));
  },
}));
