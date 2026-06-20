import type { Node, SessionDocument } from "../../generated/session-types";

type PrimitivePaletteProps = {
  session: SessionDocument | null;
  selectedNode: Node | null;
  isLoading: boolean;
  onAddNode: (node: Node) => void;
  onRemoveNode: (nodeId: string) => void;
};

type PrimitiveKind = "source" | "effect" | "mixer";

export function PrimitivePalette({
  session,
  selectedNode,
  isLoading,
  onAddNode,
  onRemoveNode,
}: PrimitivePaletteProps) {
  const addPrimitive = (kind: PrimitiveKind) => {
    onAddNode(buildPrimitiveNode(kind, session));
  };

  return (
    <section className="primitive-palette">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Primitive palette</p>
          <h2>Add nodes</h2>
        </div>
        <span>{session?.nodes.length ?? 0} voices</span>
      </div>

      <div className="palette-actions">
        <button type="button" onClick={() => addPrimitive("source")} disabled={isLoading}>
          Add Source
        </button>
        <button type="button" onClick={() => addPrimitive("effect")} disabled={isLoading}>
          Add Effect
        </button>
        <button type="button" onClick={() => addPrimitive("mixer")} disabled={isLoading}>
          Add Mixer
        </button>
      </div>

      <p className="palette-caption">
        Drag a node handle onto another to reroute the live graph without leaving the instrument.
      </p>

      <button
        type="button"
        className="destructive-button"
        onClick={() => selectedNode && onRemoveNode(selectedNode.id)}
        disabled={isLoading || !selectedNode}
      >
        Remove Selected
      </button>
    </section>
  );
}

function buildPrimitiveNode(kind: PrimitiveKind, session: SessionDocument | null): Node {
  const suffix = globalThis.crypto.randomUUID().slice(0, 8);
  const id = `${kind}-${suffix}`;
  const busId = session?.buses[0]?.id ?? null;
  const sceneId = session?.scenes[0]?.id;

  if (kind === "source") {
    return {
      id,
      nodeType: "source",
      ports: [{ id: `${id}-out`, name: "main_out", direction: "output", signalType: "audio" }],
      parameters: [
        {
          id: `${id}-level`,
          name: "level",
          value: 0.75,
          defaultValue: 0.75,
          minValue: 0,
          maxValue: 1,
          unit: "linear",
        },
      ],
      runtimeTarget: `audio/source/${id}`,
      sceneMembership: sceneId ? [sceneId] : [],
      ownership: { controller: "shared", isLocked: false },
      enabled: true,
      audioPrimitive: {
        kind: "source",
        config: { sourceType: "oscillator", channelMode: "mono", busTargetId: busId },
      },
    };
  }

  if (kind === "effect") {
    return {
      id,
      nodeType: "effect",
      ports: [
        { id: `${id}-in`, name: "signal_in", direction: "input", signalType: "audio" },
        { id: `${id}-out`, name: "signal_out", direction: "output", signalType: "audio" },
      ],
      parameters: [
        {
          id: `${id}-mix`,
          name: "mix",
          value: 0.4,
          defaultValue: 0.4,
          minValue: 0,
          maxValue: 1,
          unit: "ratio",
        },
      ],
      runtimeTarget: `audio/effect/${id}`,
      sceneMembership: sceneId ? [sceneId] : [],
      ownership: { controller: "shared", isLocked: false },
      enabled: true,
      audioPrimitive: {
        kind: "effect",
        config: { effectType: "delay", bypassed: false, busTargetId: busId },
      },
    };
  }

  return {
    id,
    nodeType: "mixer",
    ports: [
      { id: `${id}-in`, name: "mix_in", direction: "input", signalType: "audio" },
      { id: `${id}-out`, name: "mix_out", direction: "output", signalType: "audio" },
    ],
    parameters: [
      {
        id: `${id}-level`,
        name: "level",
        value: 0.85,
        defaultValue: 0.85,
        minValue: 0,
        maxValue: 1,
        unit: "linear",
      },
    ],
    runtimeTarget: `audio/mixer/${id}`,
    sceneMembership: sceneId ? [sceneId] : [],
    ownership: { controller: "shared", isLocked: false },
    enabled: true,
    audioPrimitive: {
      kind: "mixer",
      config: { channelMode: "stereo", busTargetId: busId },
    },
  };
}
