import { useEffect, useState } from "react";
import type { ReactNode } from "react";

import type {
  BindingTarget,
  HardwareBinding,
  HardwareRuntimeSettings,
  HardwareRuntimeStatus,
  MacroDefinition,
  MidiInputPort,
  SceneDefinition,
} from "../../generated/session-types";
import { formatSource, formatTarget } from "./MidiLearnOverlay";

type HardwarePanelProps = {
  bindings: HardwareBinding[];
  settings: HardwareRuntimeSettings | null;
  status: HardwareRuntimeStatus | null;
  midiInputPorts: MidiInputPort[];
  macros: MacroDefinition[];
  scenes: SceneDefinition[];
  isLoading: boolean;
  onRefresh: () => void;
  onUpdateSettings: (settings: HardwareRuntimeSettings) => void;
  onStartListeners: () => void;
  onStopListeners: () => void;
  onStartLearn: (target: BindingTarget) => void;
  onCancelLearn: () => void;
  onRemoveBinding: (bindingId: string) => void;
};

const DEFAULT_SETTINGS: HardwareRuntimeSettings = {
  midi: { selectedInputId: null, autoStart: false },
  osc: { bindHost: "127.0.0.1", listenPort: 9001, autoStart: false },
};

const TRANSPORT_TARGETS: Array<{ label: string; target: BindingTarget }> = [
  { label: "Play", target: { kind: "transportPlay" } },
  { label: "Stop", target: { kind: "transportStop" } },
  { label: "Panic", target: { kind: "transportPanic" } },
];

export function listenerLabel(lifecycle: string | undefined): string {
  switch (lifecycle) {
    case "listening":
      return "Listening";
    case "learning":
      return "Learning";
    case "captured":
      return "Captured";
    case "starting":
      return "Starting";
    case "restarting":
      return "Restarting";
    case "error":
      return "Error";
    case "unavailable":
      return "Unavailable";
    case "stopped":
      return "Stopped";
    default:
      return "Unknown";
  }
}

export function statusTone(lifecycle: string | undefined): "ok" | "warn" | "error" | "idle" {
  if (lifecycle === "listening" || lifecycle === "captured") return "ok";
  if (lifecycle === "learning" || lifecycle === "starting" || lifecycle === "restarting") return "warn";
  if (lifecycle === "error" || lifecycle === "unavailable") return "error";
  return "idle";
}

export function bindingSourceKind(binding: HardwareBinding): string {
  return binding.source.kind === "oscAddress" ? "OSC" : "MIDI";
}

export function bindingTargetLabel(
  target: BindingTarget,
  macros: MacroDefinition[],
  scenes: SceneDefinition[],
): string {
  if (target.kind === "macro") {
    return macros.find((macro) => macro.id === target.config.macro_id)?.name ?? formatTarget(target);
  }
  if (target.kind === "sceneRecall") {
    return scenes.find((scene) => scene.id === target.config.scene_id)?.name ?? formatTarget(target);
  }
  return formatTarget(target);
}

export function HardwarePanel({
  bindings,
  settings,
  status,
  midiInputPorts,
  macros,
  scenes,
  isLoading,
  onRefresh,
  onUpdateSettings,
  onStartListeners,
  onStopListeners,
  onStartLearn,
  onCancelLearn,
  onRemoveBinding,
}: HardwarePanelProps) {
  const [draft, setDraft] = useState<HardwareRuntimeSettings>(settings ?? DEFAULT_SETTINGS);

  useEffect(() => {
    setDraft(settings ?? DEFAULT_SETTINGS);
  }, [settings]);

  const applySettings = () => onUpdateSettings(draft);
  const learnState = status?.learn.lifecycle ?? "idle";

  return (
    <div className="inspector-group hardware-panel" style={{ marginTop: 16 }}>
      <div className="hardware-panel-header">
        <div>
          <h2>Hardware</h2>
          <p className="empty-copy">MIDI and OSC controls for live bindings.</p>
        </div>
        <div className="hardware-actions">
          <button type="button" onClick={onRefresh} disabled={isLoading}>Refresh</button>
          <button type="button" onClick={onStartListeners} disabled={isLoading}>Start</button>
          <button type="button" onClick={onStopListeners} disabled={isLoading}>Stop</button>
        </div>
      </div>

      <div className="hardware-status-grid">
        <StatusPill label="MIDI" lifecycle={status?.midi.lifecycle} detail={status?.midi.selectedDisplayName ?? status?.midi.lastError ?? "No input selected"} />
        <StatusPill label="OSC" lifecycle={status?.osc.lifecycle} detail={`${status?.osc.bindHost ?? draft.osc.bindHost}:${status?.osc.listenPort ?? draft.osc.listenPort}`} />
        <StatusPill label="Learn" lifecycle={learnState} detail={status?.learn.target ? formatTarget(status.learn.target) : "Idle"} />
      </div>

      <div className="hardware-settings-grid">
        <label className="field-stack">
          <span>MIDI input</span>
          <select
            className="bus-select"
            value={draft.midi.selectedInputId ?? ""}
            onChange={(event) =>
              setDraft({
                ...draft,
                midi: { ...draft.midi, selectedInputId: event.target.value || null },
              })
            }
          >
            <option value="">No MIDI input</option>
            {midiInputPorts.map((port) => (
              <option key={port.id} value={port.id}>{port.displayName}</option>
            ))}
          </select>
        </label>

        <label className="field-stack">
          <span>OSC host</span>
          <input
            className="hardware-input"
            value={draft.osc.bindHost}
            onChange={(event) =>
              setDraft({ ...draft, osc: { ...draft.osc, bindHost: event.target.value } })
            }
          />
        </label>

        <label className="field-stack">
          <span>OSC port</span>
          <input
            className="hardware-input"
            type="number"
            min={1}
            max={65535}
            value={draft.osc.listenPort}
            onChange={(event) =>
              setDraft({ ...draft, osc: { ...draft.osc, listenPort: Number(event.target.value) } })
            }
          />
        </label>

        <button type="button" onClick={applySettings} disabled={isLoading}>Apply</button>
      </div>

      <div className="hardware-learn-section">
        <div className="hardware-section-title">
          <h3>Learn Targets</h3>
          {learnState === "learning" ? (
            <button type="button" onClick={onCancelLearn} disabled={isLoading}>Cancel Learn</button>
          ) : null}
        </div>

        <div className="hardware-learn-groups">
          <LearnGroup title="Macros">
            {macros.length > 0 ? macros.map((macro) => (
              <button
                key={macro.id}
                type="button"
                onClick={() => onStartLearn({ kind: "macro", config: { macro_id: macro.id } })}
                disabled={isLoading}
              >
                {macro.name}
              </button>
            )) : <span className="empty-copy">No macros</span>}
          </LearnGroup>

          <LearnGroup title="Scenes">
            {scenes.map((scene) => (
              <button
                key={scene.id}
                type="button"
                onClick={() => onStartLearn({ kind: "sceneRecall", config: { scene_id: scene.id } })}
                disabled={isLoading}
              >
                {scene.name}
              </button>
            ))}
          </LearnGroup>

          <LearnGroup title="Transport">
            {TRANSPORT_TARGETS.map((item) => (
              <button
                key={item.label}
                type="button"
                onClick={() => onStartLearn(item.target)}
                disabled={isLoading}
                className={item.target.kind === "transportPanic" ? "panic-button" : undefined}
              >
                {item.label}
              </button>
            ))}
          </LearnGroup>
        </div>
      </div>

      {status?.diagnostics.length ? (
        <div className="hardware-diagnostics">
          {status.diagnostics.map((diagnostic) => (
            <div key={`${diagnostic.code}-${diagnostic.message}`} className="hardware-diagnostic">
              <strong>{diagnostic.message}</strong>
              {diagnostic.detail ? <span>{diagnostic.detail}</span> : null}
            </div>
          ))}
        </div>
      ) : null}

      <div className="hardware-bindings">
        <h3>Bindings</h3>
        {bindings.length > 0 ? (
          <div className="hardware-binding-list">
            {bindings.map((binding) => (
              <div key={binding.id} className="hardware-binding-row">
                <span className="source-badge">{bindingSourceKind(binding)}</span>
                <span>{formatSource(binding)}</span>
                <span className="binding-arrow">to</span>
                <strong>{bindingTargetLabel(binding.target, macros, scenes)}</strong>
                <button type="button" onClick={() => onRemoveBinding(binding.id)} disabled={isLoading}>
                  Remove
                </button>
              </div>
            ))}
          </div>
        ) : (
          <p className="empty-copy">No hardware bindings yet.</p>
        )}
      </div>
    </div>
  );
}

function StatusPill({ label, lifecycle, detail }: { label: string; lifecycle?: string; detail: string }) {
  const tone = statusTone(lifecycle);
  return (
    <div className={`hardware-status-pill hardware-status-${tone}`}>
      <span>{label}</span>
      <strong>{listenerLabel(lifecycle)}</strong>
      <small>{detail}</small>
    </div>
  );
}

function LearnGroup({ title, children }: { title: string; children: ReactNode }) {
  return (
    <div className="hardware-learn-group">
      <span>{title}</span>
      <div>{children}</div>
    </div>
  );
}
