import type { Edge, Node as FlowNode } from "@xyflow/react";

import type { Node, SessionDocument } from "../generated/session-types";

export type GraphNodeData = {
  title: string;
  subtitle: string;
  isSelected: boolean;
  isEnabled: boolean;
};

export type GraphNode = FlowNode<GraphNodeData>;
export type GraphEdge = Edge;

export type AudioRuntimeProjection = {
  lifecycle: SessionDocument["audioRuntime"]["lifecycle"];
  health: SessionDocument["audioRuntime"]["health"];
  status: string;
  detail: string;
  canStart: boolean;
  canStop: boolean;
};

export type VisualRuntimeProjection = {
  lifecycle: SessionDocument["visualRuntime"]["lifecycle"];
  health: SessionDocument["visualRuntime"]["health"];
  status: string;
  detail: string;
  canStart: boolean;
  canStop: boolean;
};

export type AgentRuntimeProjection = {
  isAvailable: boolean;
  pendingActionCount: number;
  isFrozen: boolean;
  status: string;
};

export type SessionProjection = {
  session: SessionDocument;
  selectedNodeId: string | null;
  selectedNode: Node | null;
  graphNodes: GraphNode[];
  graphEdges: GraphEdge[];
  audioRuntime: AudioRuntimeProjection;
  visualRuntime: VisualRuntimeProjection;
  agentRuntime: AgentRuntimeProjection;
  topologySignature: string;
};

export function projectSessionState(
  session: SessionDocument,
  selectedNodeId: string | null,
  previous?: Pick<SessionProjection, "graphNodes" | "graphEdges" | "topologySignature" | "selectedNodeId">,
): SessionProjection {
  const selectedNode = deriveSelectedNode(session, selectedNodeId);
  const resolvedSelectedNodeId = selectedNode?.id ?? null;
  const topologySignature = buildTopologySignature(session);
  const canReuseGraph =
    previous !== undefined &&
    previous.topologySignature === topologySignature &&
    previous.selectedNodeId === resolvedSelectedNodeId;

  return {
    session,
    selectedNodeId: resolvedSelectedNodeId,
    selectedNode,
    graphNodes: canReuseGraph ? previous.graphNodes : projectGraphNodes(session, resolvedSelectedNodeId),
    graphEdges: canReuseGraph ? previous.graphEdges : projectGraphEdges(session),
    audioRuntime: projectAudioRuntime(session),
    visualRuntime: projectVisualRuntime(session),
    agentRuntime: projectAgentRuntime(session),
    topologySignature,
  };
}

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
      isEnabled: node.enabled,
    },
    style: {
      borderRadius: 18,
      border: selectedNodeId === node.id ? "1px solid #f7c66a" : "1px solid #2d4442",
      background: node.enabled
        ? selectedNodeId === node.id
          ? "#173734"
          : "#112725"
        : "#1d2323",
      color: "#f2eee5",
      opacity: node.enabled ? 1 : 0.7,
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

function buildTopologySignature(session: SessionDocument): string {
  const nodeSignature = session.nodes
    .map((node) => ({
      id: node.id,
      nodeType: node.nodeType,
      enabled: node.enabled,
      ownership: node.ownership.controller,
      ports: node.ports.map((port) => `${port.id}:${port.direction}:${port.signalType}`).join(","),
    }))
    .map((node) => `${node.id}:${node.nodeType}:${node.enabled}:${node.ownership}:${node.ports}`)
    .join("|");

  const routeSignature = session.routes
    .map(
      (route) =>
        `${route.id}:${route.sourceNodeId}:${route.sourcePortId}:${route.targetNodeId}:${route.targetPortId}:${route.busId ?? "direct"}`,
    )
    .join("|");

  return `${nodeSignature}::${routeSignature}`;
}

function projectAudioRuntime(session: SessionDocument): AudioRuntimeProjection {
  const { health, lastError, lifecycle } = session.audioRuntime;

  let detail = "Ready for the next transport command.";
  if (lastError) {
    detail = lastError;
  } else if (session.audioRuntime.activePatchId) {
    detail = `Patch ${session.audioRuntime.activePatchId} active.`;
  } else if (session.audioRuntime.panicRecoveryCount > 0) {
    detail = `Recovered ${session.audioRuntime.panicRecoveryCount} time(s) after panic.`;
  }

  return {
    lifecycle,
    health,
    status: `${lifecycle.replace(/_/g, " ")} / ${health.replace(/_/g, " ")}`,
    detail,
    canStart: lifecycle === "idle" || lifecycle === "ready",
    canStop: lifecycle === "booting" || lifecycle === "running" || lifecycle === "recovering",
  };
}

export function deriveActiveSceneId(session: SessionDocument): string | null {
  const enabledIds = new Set(
    session.nodes.filter((node) => node.enabled).map((node) => node.id),
  );

  if (enabledIds.size === 0) {
    return null;
  }

  let bestMatch: { sceneId: string; score: number } | null = null;

  for (const scene of session.scenes) {
    const sceneIds = new Set(scene.activeNodeIds);
    let matchCount = 0;
    let mismatchCount = 0;

    for (const id of enabledIds) {
      if (sceneIds.has(id)) {
        matchCount++;
      } else {
        mismatchCount++;
      }
    }

    for (const id of sceneIds) {
      if (!enabledIds.has(id)) {
        mismatchCount++;
      }
    }

    const score = matchCount - mismatchCount;

    if (!bestMatch || score > bestMatch.score) {
      bestMatch = { sceneId: scene.id, score };
    }
  }

  return bestMatch?.sceneId ?? null;
}

function projectVisualRuntime(session: SessionDocument): VisualRuntimeProjection {
  const { health, lastError, lifecycle } = session.visualRuntime;

  let detail = "Visual runtime idle.";
  if (lastError) {
    detail = lastError;
  } else if (session.visualRuntime.activeSceneId) {
    detail = `Scene ${session.visualRuntime.activeSceneId} loaded.`;
    if (session.visualRuntime.fps != null) {
      detail += ` ${session.visualRuntime.fps} FPS.`;
    }
  }

  return {
    lifecycle,
    health,
    status: `${lifecycle.replace(/_/g, " ")} / ${health.replace(/_/g, " ")}`,
    detail,
    canStart: lifecycle === "idle" || lifecycle === "failed",
    canStop: lifecycle === "starting" || lifecycle === "ready" || lifecycle === "rendering",
  };
}

function projectAgentRuntime(session: SessionDocument): AgentRuntimeProjection {
  const isAvailable = session.agentRuntime?.isAvailable ?? true;
  const pendingActionCount = session.agentRuntime?.pendingActionCount ?? session.pendingActions.length;
  const isFrozen = session.agentRuntime?.isFrozen ?? session.agentFrozen;

  let status = "Available";
  if (isFrozen) {
    status = "Frozen";
  } else if (pendingActionCount > 0) {
    status = `${pendingActionCount} pending action(s)`;
  }

  return {
    isAvailable,
    pendingActionCount,
    isFrozen,
    status,
  };
}
