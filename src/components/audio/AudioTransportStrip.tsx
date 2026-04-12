import type { AudioRuntimeProjection } from "../../store/session-projections";

type AudioTransportStripProps = {
  runtime: AudioRuntimeProjection | null;
  isLoading: boolean;
  onStart: () => void;
  onStop: () => void;
  onPanic: () => void;
};

export function AudioTransportStrip({
  runtime,
  isLoading,
  onStart,
  onStop,
  onPanic,
}: AudioTransportStripProps) {
  return (
    <section className="transport-strip">
      <div>
        <p className="eyebrow">Audio transport</p>
        <h2>Play the graph, then steer it live.</h2>
        <p className="transport-detail">{runtime?.detail ?? "Waiting for audio runtime status."}</p>
      </div>

      <div className="transport-readout">
        <span className="transport-status">{runtime?.status ?? "idle / unknown"}</span>
        <div className="transport-actions">
          <button type="button" onClick={onStart} disabled={isLoading || runtime?.canStart === false}>
            Play
          </button>
          <button type="button" onClick={onStop} disabled={isLoading || runtime?.canStop === false}>
            Stop
          </button>
          <button type="button" className="panic-button" onClick={onPanic} disabled={isLoading}>
            Panic
          </button>
        </div>
      </div>
    </section>
  );
}
