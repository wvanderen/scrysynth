import type { Edge, Node as FlowNode } from "@xyflow/react";

import type { Node, SessionDocument } from "../generated/session-types";

export type MacroProjection = {
  id: string;
  name: string;
  targets: SessionDocument["macros"][number]["targets"];
  rangeStart: number;
  rangeEnd: number;
};

export type GraphNodeData = {
  label: string;
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
  connectionStatus: SessionDocument["runtimeStatus"][number]["status"] | "unknown";
  status: string;
  detail: string;
  activeSceneLabel: string;
  rendererLabel: string;
  fpsLabel: string;
  canStart: boolean;
  canStop: boolean;
  canPanic: boolean;
  isRestartable: boolean;
};

export type AgentRuntimeProjection = {
  isAvailable: boolean;
  pendingActionCount: number;
  isFrozen: boolean;
  status: string;
};

const GRAPH_NODE_WIDTH = 190;
const GRAPH_NODE_HEIGHT = 74;

export type SessionProjection = {
  session: SessionDocument;
  selectedNodeId: string | null;
  selectedNode: Node | null;
  graphNodes: GraphNode[];
  graphEdges: GraphEdge[];
  audioRuntime: AudioRuntimeProjection;
  visualRuntime: VisualRuntimeProjection;
  agentRuntime: AgentRuntimeProjection;
  macros: MacroProjection[];
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
    macros: projectMacros(session),
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
    width: GRAPH_NODE_WIDTH,
    height: GRAPH_NODE_HEIGHT,
    data: {
      label: labelForNode(node),
      title: labelForNode(node),
      subtitle: `${node.nodeTypeId} / ${node.ownership.controller}`,
      isSelected: selectedNodeId === node.id,
      isEnabled: node.enabled,
    },
    style: {
      borderRadius: 8,
      border: selectedNodeId === node.id ? "1px solid #38bdf8" : "1px solid rgba(133, 146, 166, 0.28)",
      background: node.enabled
        ? selectedNodeId === node.id
          ? "#122334"
          : "#11141c"
        : "#1b1f29",
      color: "#eef2f7",
      opacity: node.enabled ? 1 : 0.7,
      padding: 16,
      width: GRAPH_NODE_WIDTH,
      boxShadow: "0 18px 38px rgba(0, 0, 0, 0.32)",
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
      stroke: "#38bdf8",
      strokeWidth: 2,
    },
    label: route.busId ? "bus" : undefined,
    labelStyle: {
      fill: "#a8b2c1",
      fontSize: 12,
    },
    labelBgStyle: {
      fill: "#11141c",
      stroke: "rgba(133, 146, 166, 0.28)",
      strokeWidth: 1,
    },
    labelBgPadding: [6, 4],
    labelBgBorderRadius: 4,
  }));
}

function labelForNode(node: Node) {
  const primaryPort = node.ports[0]?.name ?? "portless";
  return `${node.nodeTypeId}:${primaryPort}`;
}

function deriveSelectedNode(session: SessionDocument, selectedNodeId: string | null): Node | null {
  if (!selectedNodeId) {
    return null;
  }

  return session.nodes.find((node) => node.id === selectedNodeId) ?? null;
}

function buildTopologySignature(session: SessionDocument): string {
  const nodeSignature = session.nodes
    .map((node) => ({
      id: node.id,
      nodeTypeId: node.nodeTypeId,
      enabled: node.enabled,
      ownership: node.ownership.controller,
      ports: node.ports.map((port) => `${port.id}:${port.direction}:${port.signalType}`).join(","),
    }))
    .map((node) => `${node.id}:${node.nodeTypeId}:${node.enabled}:${node.ownership}:${node.ports}`)
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
  const runtimeStatusError =
    session.runtimeStatus.find((runtime) => runtime.runtime === "audio")?.lastError ?? null;
  const visibleError = lastError ?? runtimeStatusError;

  let detail = "Ready for the next transport command.";
  if (visibleError) {
    detail = visibleError;
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
    canStart: lifecycle === "idle" || lifecycle === "failed",
    canStop:
      lifecycle === "booting" ||
      lifecycle === "ready" ||
      lifecycle === "running" ||
      lifecycle === "recovering",
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
  const runtimeStatus = session.runtimeStatus.find((runtime) => runtime.runtime === "visual");
  const connectionStatus = runtimeStatus?.status ?? "unknown";
  const visibleError = lastError ?? runtimeStatus?.lastError ?? null;
  const activeSceneId = session.visualRuntime.activeSceneId;
  const activeScene = activeSceneId
    ? session.scenes.find((scene) => scene.id === activeSceneId)
    : null;
  const activeSceneLabel = activeScene
    ? `${activeScene.name} (${activeScene.id})`
    : activeSceneId ?? "none";
  const rendererLabel = session.visualRuntime.renderer ?? "not attached";
  const fpsLabel = session.visualRuntime.fps != null
    ? `${Math.round(session.visualRuntime.fps)} FPS`
    : "no telemetry";
  const isMissingSidecar = Boolean(
    visibleError?.includes("visual runtime binary not found") ||
      visibleError?.includes("SCRYSYNTH_BEVY_PATH"),
  );
  const isRestartable =
    lifecycle === "idle" ||
    lifecycle === "failed" ||
    lifecycle === "panicked" ||
    connectionStatus === "error";

  let status = `${lifecycle.replace(/_/g, " ")} / ${health.replace(/_/g, " ")}`;
  if (isMissingSidecar) {
    status = "missing sidecar / restartable";
  } else if (lifecycle === "starting" || connectionStatus === "connecting") {
    status = "booting / connecting";
  } else if (lifecycle === "ready" || lifecycle === "rendering") {
    status = `${lifecycle} / ${health}`;
  } else if (lifecycle === "panicked") {
    status = "panicked / restartable";
  } else if (lifecycle === "failed" || health === "degraded" || health === "error") {
    status = "degraded / restartable";
  } else if (connectionStatus === "disconnected") {
    status = "stopped / disconnected";
  }

  let detail = "Stopped. Start launches the visual sidecar and loads the active scene.";
  if (isMissingSidecar && visibleError) {
    detail = `${visibleError}. Build the sidecar or set SCRYSYNTH_BEVY_PATH to its full path, then Start again.`;
  } else if (visibleError) {
    detail = `${visibleError}${isRestartable ? " Start can retry the visual sidecar." : ""}`;
  } else if (lifecycle === "starting" || connectionStatus === "connecting") {
    detail = "Launching sidecar, sending handshake, and waiting for scene load acknowledgement.";
  } else if (lifecycle === "ready" || lifecycle === "rendering") {
    detail = `Scene ${activeSceneLabel} loaded on ${rendererLabel}.`;
  } else if (lifecycle === "panicked") {
    detail = "Panic stop complete. Start will relaunch the sidecar and reload the active scene.";
  }

  return {
    lifecycle,
    health,
    connectionStatus,
    status,
    detail,
    activeSceneLabel,
    rendererLabel,
    fpsLabel,
    canStart: lifecycle === "idle" || lifecycle === "failed" || lifecycle === "panicked",
    canStop: lifecycle === "starting" || lifecycle === "ready" || lifecycle === "rendering",
    canPanic: lifecycle === "starting" || lifecycle === "ready" || lifecycle === "rendering",
    isRestartable,
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

export function projectMacros(session: SessionDocument): MacroProjection[] {
  return session.macros.map((macro) => ({
    id: macro.id,
    name: macro.name,
    targets: macro.targets,
    rangeStart: macro.rangeStart,
    rangeEnd: macro.rangeEnd,
  }));
}
