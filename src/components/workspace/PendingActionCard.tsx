import type { PendingAction, TypedCommand } from "../../generated/session-types";

function describeCommand(cmd: TypedCommand): string {
  if (cmd.type === "graphEdit") {
    return cmd.payload.type;
  }
  return cmd.payload.type;
}

type PendingActionCardProps = {
  action: PendingAction;
  isLoading: boolean;
  onApprove: () => void;
  onReject: () => void;
};

export function PendingActionCard({ action, isLoading, onApprove, onReject }: PendingActionCardProps) {
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
      <p className="pending-action-command">
        {action.command.type}: {describeCommand(action.command)}
      </p>
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
    </div>
  );
}
