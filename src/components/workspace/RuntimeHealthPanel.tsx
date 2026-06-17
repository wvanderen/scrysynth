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

function dotStyle(color: StatusDotColor): React.CSSProperties {
  const colors: Record<StatusDotColor, string> = {
    green: "#4ade80",
    yellow: "#facc15",
    red: "#f87171",
    gray: "#6b7280",
  };
  return {
    width: 10,
    height: 10,
    borderRadius: "50%",
    backgroundColor: colors[color],
    display: "inline-block",
    marginRight: 8,
    flexShrink: 0,
  };
}

const sectionStyle: React.CSSProperties = {
  padding: "10px 14px",
  borderRadius: 8,
  background: "#112725",
  border: "1px solid #2d4442",
  minWidth: 200,
};

const labelStyle: React.CSSProperties = {
  fontSize: 11,
  textTransform: "uppercase",
  letterSpacing: "0.05em",
  color: "#8ba8a4",
  marginBottom: 4,
};

const detailStyle: React.CSSProperties = {
  fontSize: 12,
  color: "#c4d8d4",
  marginTop: 4,
};

const metadataStyle: React.CSSProperties = {
  display: "grid",
  gridTemplateColumns: "88px minmax(0, 1fr)",
  gap: "3px 10px",
  marginTop: 8,
  fontSize: 11,
  color: "#8ba8a4",
};

const metadataValueStyle: React.CSSProperties = {
  color: "#f2eee5",
  minWidth: 0,
  overflowWrap: "anywhere",
};

const buttonStyle: React.CSSProperties = {
  fontSize: 11,
  padding: "4px 10px",
  borderRadius: 4,
  border: "1px solid #2d4442",
  background: "#1a3533",
  color: "#c4d8d4",
  cursor: "pointer",
  marginRight: 4,
  marginTop: 6,
};

const buttonDangerStyle: React.CSSProperties = {
  ...buttonStyle,
  borderColor: "#7f1d1d",
  color: "#f87171",
};

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
    <div style={{ display: "flex", gap: 12, padding: "8px 0", flexWrap: "wrap" }}>
      <div style={sectionStyle}>
        <div style={labelStyle}>
          <span style={dotStyle(audioDot)} />
          Audio Runtime
        </div>
        <div style={detailStyle}>{audioRuntime?.status ?? "idle / unknown"}</div>
        {audioRuntime?.detail ? <div style={detailStyle}>{audioRuntime.detail}</div> : null}
        <div>
          {audioRuntime?.canStart ? <button type="button" style={buttonStyle} onClick={startAudio}>Start</button> : null}
          {audioRuntime?.canStop ? <button type="button" style={buttonStyle} onClick={stopAudio}>Stop</button> : null}
          {audioRuntime?.canStop ? <button type="button" style={buttonDangerStyle} onClick={panicAudio}>Panic</button> : null}
        </div>
      </div>

      <div style={sectionStyle}>
        <div style={labelStyle}>
          <span style={dotStyle(visualDot)} />
          Visual Runtime
        </div>
        <div style={detailStyle}>{visualRuntime?.status ?? "idle / unknown"}</div>
        {visualRuntime?.detail ? <div style={detailStyle}>{visualRuntime.detail}</div> : null}
        {visualRuntime ? (
          <div style={metadataStyle}>
            <span>Connection</span>
            <strong style={metadataValueStyle}>{visualRuntime.connectionStatus}</strong>
            <span>Scene</span>
            <strong style={metadataValueStyle}>{visualRuntime.activeSceneLabel}</strong>
            <span>Renderer</span>
            <strong style={metadataValueStyle}>{visualRuntime.rendererLabel}</strong>
            <span>Telemetry</span>
            <strong style={metadataValueStyle}>{visualRuntime.fpsLabel}</strong>
          </div>
        ) : null}
        <div>
          {visualRuntime?.canStart ? <button type="button" style={buttonStyle} onClick={startVisual}>Start</button> : null}
          {visualRuntime?.canStop ? <button type="button" style={buttonStyle} onClick={stopVisual}>Stop</button> : null}
          {visualRuntime?.canPanic ? <button type="button" style={buttonDangerStyle} onClick={panicVisual}>Panic</button> : null}
        </div>
      </div>

      <div style={sectionStyle}>
        <div style={labelStyle}>
          <span style={dotStyle(agentDot)} />
          Agent System
        </div>
        <div style={detailStyle}>{agentRuntime?.status ?? "Available"}</div>
        <div style={detailStyle}>
          {agentRuntime?.isAvailable ? "Online" : "Offline"}
          {agentRuntime?.isFrozen ? " · Frozen" : ""}
          {agentRuntime?.pendingActionCount ? ` · ${agentRuntime.pendingActionCount} pending` : ""}
        </div>
      </div>
    </div>
  );
}
