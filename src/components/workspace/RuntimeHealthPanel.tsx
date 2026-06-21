import { useSessionStore } from "../../store/sessionStore";

type StatusDotColor = "green" | "yellow" | "red" | "gray";

function statusDotColor(
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

export function RuntimeHealthPanel() {
  const audioRuntime = useSessionStore((s) => s.audioRuntime);
  const visualRuntime = useSessionStore((s) => s.visualRuntime);
  const agentRuntime = useSessionStore((s) => s.agentRuntime);
  const startAudio = useSessionStore((s) => s.startAudio);
  const stopAudio = useSessionStore((s) => s.stopAudio);
  const panicAudio = useSessionStore((s) => s.panicAudio);
  const startVisual = useSessionStore((s) => s.startVisual);
  const stopVisual = useSessionStore((s) => s.stopVisual);
  const panicVisual = useSessionStore((s) => s.panicVisual);

  const audioDot = statusDotColor(audioRuntime?.lifecycle ?? "idle", audioRuntime?.health ?? "unknown");
  const visualDot = statusDotColor(visualRuntime?.lifecycle ?? "idle", visualRuntime?.health ?? "unknown");
  const agentDot: StatusDotColor = agentRuntime?.isFrozen ? "yellow" : agentRuntime?.isAvailable ? "green" : "gray";

  return (
    <div className="runtime-health-grid">
      <div className="runtime-health-card">
        <div className="runtime-card-summary">
          <div className="runtime-health-label">
            <span className={`status-dot status-dot-${audioDot}`} />
            Audio Runtime
          </div>
          <strong>{audioRuntime?.status ?? "idle / unknown"}</strong>
        </div>
        <details className="runtime-health-details">
          <summary>Details</summary>
          <div className="runtime-health-detail">{audioRuntime?.detail ?? "No audio runtime detail reported."}</div>
        </details>
        <div className="dense-control-row">
          {audioRuntime?.canStart ? <button type="button" className="compact-button" onClick={startAudio}>Start</button> : null}
          {audioRuntime?.canStop ? <button type="button" className="compact-button" onClick={stopAudio}>Stop</button> : null}
          {audioRuntime?.canStop ? <button type="button" className="compact-button destructive-button" onClick={panicAudio}>Panic</button> : null}
        </div>
      </div>

      <div className="runtime-health-card">
        <div className="runtime-card-summary">
          <div className="runtime-health-label">
            <span className={`status-dot status-dot-${visualDot}`} />
            Visual Runtime
          </div>
          <strong>{visualRuntime?.status ?? "idle / unknown"}</strong>
        </div>
        <details className="runtime-health-details">
          <summary>Details</summary>
          {visualRuntime?.detail ? <div className="runtime-health-detail">{visualRuntime.detail}</div> : null}
          {visualRuntime ? (
            <div className="runtime-health-metadata">
              <span>Connection</span>
              <strong>{visualRuntime.connectionStatus}</strong>
              <span>Scene</span>
              <strong>{visualRuntime.activeSceneLabel}</strong>
              <span>Renderer</span>
              <strong>{visualRuntime.rendererLabel}</strong>
              <span>Telemetry</span>
              <strong>{visualRuntime.fpsLabel}</strong>
            </div>
          ) : null}
        </details>
        <div className="dense-control-row">
          {visualRuntime?.canStart ? <button type="button" className="compact-button" onClick={startVisual}>Start</button> : null}
          {visualRuntime?.canStop ? <button type="button" className="compact-button" onClick={stopVisual}>Stop</button> : null}
          {visualRuntime?.canPanic ? <button type="button" className="compact-button destructive-button" onClick={panicVisual}>Panic</button> : null}
        </div>
      </div>

      <div className="runtime-health-card">
        <div className="runtime-card-summary">
          <div className="runtime-health-label">
            <span className={`status-dot status-dot-${agentDot}`} />
            Agent System
          </div>
          <strong>{agentRuntime?.status ?? "Available"}</strong>
        </div>
        <details className="runtime-health-details">
          <summary>Details</summary>
          <div className="runtime-health-detail">
            {agentRuntime?.isAvailable ? "Online" : "Offline"}
            {agentRuntime?.isFrozen ? " · Frozen" : ""}
            {agentRuntime?.pendingActionCount ? ` · ${agentRuntime.pendingActionCount} pending` : ""}
          </div>
        </details>
      </div>
    </div>
  );
}
