import { useEffect } from "react";

import type { BindingTarget, HardwareBinding } from "../../generated/session-types";
import {
  ensureHardwarePolling,
  useSessionStore,
} from "../../store/sessionStore";

function formatSource(binding: HardwareBinding): string {
  const s = binding.source;
  switch (s.kind) {
    case "midiCc":
      return `MIDI CC Ch${s.config.channel} #${s.config.controller}`;
    case "midiNote":
      return `MIDI Note Ch${s.config.channel} ${s.config.note}`;
    case "midiPitchBend":
      return `MIDI Pitch Bend Ch${s.config.channel}`;
    case "oscAddress":
      return `OSC ${s.config.address}`;
    default:
      return "Unknown";
  }
}

function formatTarget(target: BindingTarget): string {
  switch (target.kind) {
    case "macro":
      return `macro`;
    case "sceneRecall":
      return `scene recall`;
    case "transportPlay":
      return "transport play";
    case "transportStop":
      return "transport stop";
    case "transportPanic":
      return "transport panic";
    default:
      return "target";
  }
}

export function MidiLearnOverlay() {
  const midiLearnActive = useSessionStore((s) => s.midiLearnActive);
  const midiLearnTarget = useSessionStore((s) => s.midiLearnTarget);
  const hardwareBindings = useSessionStore((s) => s.hardwareBindings);
  const stopMidiLearn = useSessionStore((s) => s.stopMidiLearn);

  useEffect(() => {
    ensureHardwarePolling();
  }, [midiLearnActive, hardwareBindings]);

  if (!midiLearnActive) return null;

  const targetLabel = midiLearnTarget ? formatTarget(midiLearnTarget) : "target";

  return (
    <div
      style={{
        position: "fixed",
        bottom: 0,
        left: 0,
        right: 0,
        zIndex: 1000,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        gap: 16,
        padding: "12px 24px",
        background: "var(--color-bg-elevated, #1a1a2e)",
        borderTop: "2px solid var(--color-accent, #6c63ff)",
      }}
    >
      <div
        style={{
          width: 10,
          height: 10,
          borderRadius: "50%",
          background: "#ff6b6b",
          animation: "pulse 1s ease-in-out infinite",
        }}
      />
      <span style={{ color: "var(--color-text, #e0e0e0)", fontSize: 14 }}>
        Learning — move a hardware control to bind to {targetLabel}...
      </span>
      <button
        onClick={stopMidiLearn}
        style={{
          padding: "4px 16px",
          background: "transparent",
          border: "1px solid var(--color-text, #e0e0e0)",
          color: "var(--color-text, #e0e0e0)",
          borderRadius: 4,
          cursor: "pointer",
          fontSize: 13,
        }}
      >
        Cancel
      </button>
      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.3; }
        }
      `}</style>
    </div>
  );
}

export { formatSource, formatTarget };
