import type {
  AgentRuntimeProjection,
  AudioRuntimeProjection,
  VisualRuntimeProjection,
} from "../../store/session-projections";

type StatusDotColor = "green" | "yellow" | "red" | "gray";

type GlobalSafetyChromeProps = {
  audioRuntime: AudioRuntimeProjection | null;
  visualRuntime: VisualRuntimeProjection | null;
  agentRuntime: AgentRuntimeProjection | null;
  pendingActionCount: number;
  isLoading: boolean;
  onFreezeAgent: () => void;
  onReclaimAll: () => void;
  onPanic: () => void;
};

function runtimeDotColor(
  lifecycle: string,
  health: string,
): StatusDotColor {
  if (health === "error" || health === "panic_recovered" || lifecycle === "failed") {
    return "red";
  }
  if (lifecycle === "ready" || lifecycle === "running" || lifecycle === "rendering") {
    return "green";
  }
  if (
    lifecycle === "booting" ||
    lifecycle === "starting" ||
    lifecycle === "recovering" ||
    lifecycle === "panicked" ||
    health === "degraded"
  ) {
    return "yellow";
  }
  return "gray";
}

function StatusBadge({
  label,
  value,
  dot,
}: {
  label: string;
  value: string;
  dot: StatusDotColor;
}) {
  return (
    <div className="global-status-badge">
      <span className={`status-dot status-dot-${dot}`} />
      <span className="global-status-label">{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

export function GlobalSafetyChrome({
  audioRuntime,
  visualRuntime,
  agentRuntime,
  pendingActionCount,
  isLoading,
  onFreezeAgent,
  onReclaimAll,
  onPanic,
}: GlobalSafetyChromeProps) {
  const audioDot = runtimeDotColor(
    audioRuntime?.lifecycle ?? "idle",
    audioRuntime?.health ?? "unknown",
  );
  const visualDot = runtimeDotColor(
    visualRuntime?.lifecycle ?? "idle",
    visualRuntime?.health ?? "unknown",
  );
  const agentDot: StatusDotColor = agentRuntime?.isFrozen
    ? "yellow"
    : agentRuntime?.isAvailable
      ? "green"
      : "gray";

  return (
    <section className="global-safety-chrome" aria-label="Global runtime and safety controls">
      <div className="global-status-grid">
        <StatusBadge label="Audio" value={audioRuntime?.status ?? "idle"} dot={audioDot} />
        <StatusBadge label="Visual" value={visualRuntime?.status ?? "idle"} dot={visualDot} />
        <StatusBadge label="Agent" value={agentRuntime?.status ?? "available"} dot={agentDot} />
        <div className="global-status-badge pending-status-badge">
          <span className="pending-count">{pendingActionCount}</span>
          <span className="global-status-label">Pending</span>
          <strong>{pendingActionCount === 1 ? "action" : "actions"}</strong>
        </div>
      </div>

      <div className="global-safety-actions">
        <button
          type="button"
          className={agentRuntime?.isFrozen ? "compact-button freeze-button-active" : "compact-button"}
          onClick={onFreezeAgent}
          disabled={isLoading}
        >
          {agentRuntime?.isFrozen ? "Unfreeze" : "Freeze Agent"}
        </button>
        <button
          type="button"
          className="compact-button"
          onClick={onReclaimAll}
          disabled={isLoading}
        >
          Reclaim All
        </button>
        <button
          type="button"
          className="compact-button panic-button global-panic-button"
          onClick={onPanic}
          disabled={isLoading}
        >
          Panic
        </button>
      </div>
    </section>
  );
}
