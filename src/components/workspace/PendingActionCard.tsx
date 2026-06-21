import type { PendingAction, SessionDocument, TypedCommand } from "../../generated/session-types";

function describeCommand(cmd: TypedCommand): string {
  if (cmd.type === "graphEdit") {
    return cmd.payload.type;
  }
  return cmd.payload.type;
}

function commandObjectIds(cmd: TypedCommand): string[] {
  if (cmd.type === "performance") {
    switch (cmd.payload.type) {
      case "recallScene":
        return [cmd.payload.payload.scene_id];
      case "saveVariation":
        return [cmd.payload.payload.scene_id];
      case "restoreVariation":
        return [cmd.payload.payload.variation_id];
    }
  }

  switch (cmd.payload.type) {
    case "addNode":
      return [cmd.payload.payload.node.id];
    case "removeNode":
    case "setNodeEnabled":
    case "setParameterValue":
    case "assignNodeToBus":
    case "clearNodeBusAssignment":
      return [cmd.payload.payload.node_id];
    case "addRoute":
      return [
        cmd.payload.payload.route.id,
        cmd.payload.payload.route.sourceNodeId,
        cmd.payload.payload.route.targetNodeId,
      ];
    case "removeRoute":
      return [cmd.payload.payload.route_id];
  }
}

function findObjectSnippet(session: SessionDocument | null, objectId: string): string {
  if (!session) return "not loaded";

  const node = session.nodes.find((candidate) => candidate.id === objectId);
  if (node) {
    const params = node.parameters
      .map((parameter) => `${parameter.name}=${parameter.value}${parameter.unit === "linear" ? "" : parameter.unit}`)
      .join(", ");
    return `${node.id} ${node.nodeType} ${node.enabled ? "enabled" : "muted"} ${node.ownership.controller}${params ? ` / ${params}` : ""}`;
  }

  const route = session.routes.find((candidate) => candidate.id === objectId);
  if (route) {
    return `${route.id} ${route.sourceNodeId} -> ${route.targetNodeId}`;
  }

  const scene = session.scenes.find((candidate) => candidate.id === objectId);
  if (scene) {
    return `${scene.name} / ${scene.activeNodeIds.length} active node(s)`;
  }

  const variation = session.variations.find((candidate) => candidate.id === objectId);
  if (variation) {
    return `${variation.name} / scene ${variation.sceneId}`;
  }

  return "not currently present";
}

function afterSnippet(cmd: TypedCommand): string {
  if (cmd.type === "performance") {
    switch (cmd.payload.type) {
      case "recallScene":
        return `Recall scene ${cmd.payload.payload.scene_id}`;
      case "saveVariation":
        return `Save variation "${cmd.payload.payload.name}" for ${cmd.payload.payload.scene_id}`;
      case "restoreVariation":
        return `Restore variation ${cmd.payload.payload.variation_id}`;
    }
  }

  switch (cmd.payload.type) {
    case "addNode":
      return `${cmd.payload.payload.node.id} ${cmd.payload.payload.node.nodeType} added`;
    case "removeNode":
      return `${cmd.payload.payload.node_id} removed`;
    case "setNodeEnabled":
      return `${cmd.payload.payload.node_id} ${cmd.payload.payload.enabled ? "enabled" : "muted"}`;
    case "setParameterValue":
      return `${cmd.payload.payload.node_id}.${cmd.payload.payload.parameter_id} = ${cmd.payload.payload.value}`;
    case "addRoute":
      return `${cmd.payload.payload.route.sourceNodeId} -> ${cmd.payload.payload.route.targetNodeId}`;
    case "removeRoute":
      return `${cmd.payload.payload.route_id} removed`;
    case "assignNodeToBus":
      return `${cmd.payload.payload.node_id} assigned to ${cmd.payload.payload.bus_id}`;
    case "clearNodeBusAssignment":
      return `${cmd.payload.payload.node_id} bus assignment cleared`;
  }
}

function riskLabel(riskTier: PendingAction["riskTier"]): string {
  if (riskTier === "high") return "High risk: approval required before graph or performance changes.";
  if (riskTier === "medium") return "Medium risk: review routing, ownership, or live-control impact.";
  return "Low risk: scoped change with limited live-session impact.";
}

function statusDetail(action: PendingAction): string {
  if (action.status === "pending") return "Waiting for performer approval.";
  if (action.status === "approved") return "Approved and applied to the session.";
  return "Rejected by performer; no session diff applied.";
}

type PendingActionCardProps = {
  action: PendingAction;
  session: SessionDocument | null;
  isLoading: boolean;
  onApprove: () => void;
  onReject: () => void;
};

export function PendingActionCard({ action, session, isLoading, onApprove, onReject }: PendingActionCardProps) {
  const objectIds = Array.from(new Set(commandObjectIds(action.command)));
  const beforeSnippets = objectIds.map((id) => `${id}: ${findObjectSnippet(session, id)}`);

  return (
    <div className={`pending-action-card risk-${action.riskTier}`}>
      <div className="pending-action-header">
        <span className={`risk-badge risk-${action.riskTier}`}>
          {action.riskTier}
        </span>
        <span className="pending-action-time">
          {new Date(action.createdAt).toLocaleTimeString()}
        </span>
      </div>
      <div className="proposal-summary">
        <p className="pending-action-command">
          {action.command.type}: {describeCommand(action.command)}
        </p>
        <p className="proposal-status">{statusDetail(action)}</p>
      </div>

      <dl className="proposal-review-grid">
        <div>
          <dt>Plan</dt>
          <dd>{afterSnippet(action.command)}</dd>
        </div>
        <div>
          <dt>Risk</dt>
          <dd>{riskLabel(action.riskTier)}</dd>
        </div>
        <div>
          <dt>Affected</dt>
          <dd>{objectIds.length > 0 ? objectIds.join(", ") : "session"}</dd>
        </div>
      </dl>

      <div className="proposal-diff">
        <div>
          <span>Before</span>
          {beforeSnippets.length > 0 ? (
            beforeSnippets.map((snippet) => <code key={snippet}>{snippet}</code>)
          ) : (
            <code>current session state</code>
          )}
        </div>
        <div>
          <span>After</span>
          <code>{afterSnippet(action.command)}</code>
        </div>
      </div>

      {action.status === "pending" ? (
        <div className="pending-action-actions">
          <button
            type="button"
            onClick={onApprove}
            disabled={isLoading}
          >
            Approve
          </button>
          <button
            type="button"
            className="destructive-button"
            onClick={onReject}
            disabled={isLoading}
          >
            Reject
          </button>
        </div>
      ) : null}
    </div>
  );
}
