import type {
  Bus,
  ControllerKind,
  Node,
} from "../../generated/session-types";

type NodeInspectorProps = {
  selectedNode: Node | null;
  buses: Bus[];
  isLoading: boolean;
  onToggleEnabled: (nodeId: string, enabled: boolean) => void;
  onUpdateParameter: (nodeId: string, parameterId: string, value: number) => void;
  onAssignNodeToBus: (nodeId: string, busId: string) => void;
  onClearNodeBus: (nodeId: string) => void;
  onReclaimOwnership: (nodeIds: string[]) => void;
  onSetNodeOwnership: (nodeIds: string[], controller: ControllerKind) => void;
  /** D-06/D-07/D-08: edit one step of a sequencer node's 16-step pattern. */
  onUpdateStep?: (
    nodeId: string,
    stepIndex: number,
    gate?: boolean,
    cv?: number,
  ) => void;
};

export function NodeInspector({
  selectedNode,
  buses,
  isLoading,
  onToggleEnabled,
  onUpdateParameter,
  onAssignNodeToBus,
  onClearNodeBus,
  onReclaimOwnership: _onReclaimOwnership,
  onSetNodeOwnership,
  onUpdateStep,
}: NodeInspectorProps) {
  if (!selectedNode) {
    return (
      <aside className="inspector-panel">
        <div className="panel-heading">
          <p className="eyebrow">Node inspector</p>
        </div>
        <p className="empty-state">Select a node to inspect its canonical metadata.</p>
      </aside>
    );
  }

  const assignedBusId = selectedNode.busTargetId ?? null;
  const displayName = displayNameFor(selectedNode);

  return (
    <aside className="inspector-panel">
      <div className="panel-heading">
        <p className="eyebrow">Node inspector</p>
        <span>{displayName}</span>
      </div>

      <div className="inspector-group">
        <h2>Identity</h2>
        <p><strong>id</strong> {selectedNode.id}</p>
        <p><strong>node type</strong> {selectedNode.nodeTypeId}</p>
        <label className="toggle-row">
          <span>Enabled</span>
          <input
            type="checkbox"
            checked={selectedNode.enabled}
            disabled={isLoading}
            onChange={(event) => onToggleEnabled(selectedNode.id, event.target.checked)}
          />
        </label>
        <p><strong>runtime target</strong> {selectedNode.runtimeTarget ?? "disconnected"}</p>
        <p><strong>scene membership</strong> {selectedNode.sceneMembership.join(", ") || "none"}</p>
      </div>

      <div className="inspector-group">
        <h2>Ownership</h2>
        <div className="ownership-row">
          <span className={`ownership-badge badge-${selectedNode.ownership.controller}`}>
            {selectedNode.ownership.controller}
          </span>
          {selectedNode.ownership.isLocked ? (
            <span className="lock-indicator">locked</span>
          ) : null}
        </div>
        <div className="ownership-quick-set">
          {(["user", "agent", "shared"] as const)
            .filter((c) => c !== selectedNode.ownership.controller)
            .map((controller: ControllerKind) => (
              <button
                key={controller}
                type="button"
                className="ownership-set-button"
                disabled={isLoading}
                onClick={() => onSetNodeOwnership([selectedNode.id], controller)}
              >
                Set {controller}
              </button>
            ))}
        </div>
      </div>

      <div className="inspector-group">
        <h2>Ports</h2>
        {selectedNode.ports.map((port) => (
          <div key={port.id} className="list-card">
            <p>
              {port.name}
              {port.signalType === "control" ? (
                <span className="port-badge port-badge-cv">CV</span>
              ) : null}
            </p>
            <span>{port.direction} / {port.signalType}</span>
          </div>
        ))}
      </div>

      <div className="inspector-group">
        <h2>Parameters</h2>
        {selectedNode.parameters.length > 0 ? (
          selectedNode.parameters.map((parameter) => (
            <div key={parameter.id} className="list-card">
              <div className="parameter-header">
                <p>{parameter.name}</p>
                <span>
                  {parameter.value.toFixed(2)} {parameter.unit}
                </span>
              </div>
              <input
                type="range"
                min={parameter.minValue}
                max={parameter.maxValue}
                step={(parameter.maxValue - parameter.minValue) / 100 || 0.01}
                value={parameter.value}
                disabled={isLoading}
                onChange={(event) =>
                  onUpdateParameter(selectedNode.id, parameter.id, Number(event.target.value))
                }
              />
            </div>
          ))
        ) : (
          <p className="empty-copy">No parameters on this node.</p>
        )}
      </div>

      {selectedNode.sequencerPattern ? (
        <SequencerStepEditor
          nodeId={selectedNode.id}
          pattern={selectedNode.sequencerPattern}
          isLoading={isLoading}
          onUpdateStep={onUpdateStep}
        />
      ) : null}

      <div className="inspector-group">
        <h2>Bus path</h2>
        {buses.length > 0 ? (
          <>
            <select
              className="bus-select"
              value={assignedBusId ?? ""}
              disabled={isLoading}
              onChange={(event) => {
                if (!event.target.value) {
                  onClearNodeBus(selectedNode.id);
                  return;
                }

                onAssignNodeToBus(selectedNode.id, event.target.value);
              }}
            >
              <option value="">Direct output</option>
              {buses.map((bus) => (
                <option key={bus.id} value={bus.id}>
                  {bus.name}
                </option>
              ))}
            </select>
            <p><strong>current</strong> {assignedBusId ?? "none"}</p>
          </>
        ) : (
          <p className="empty-copy">No buses available in this session.</p>
        )}
      </div>
    </aside>
  );
}

/**
 * 16-step gate+cv editor for sequencer nodes (NODES-04; D-07 mono gate+CV,
 * D-08 fixed 16 steps). Each step renders a gate toggle and a CV slider;
 * edits dispatch `GraphEditCommand::SetStepValue` via the parent.
 */
function SequencerStepEditor({
  nodeId,
  pattern,
  isLoading,
  onUpdateStep,
}: {
  nodeId: string;
  pattern: NonNullable<Node["sequencerPattern"]>;
  isLoading: boolean;
  onUpdateStep?: NodeInspectorProps["onUpdateStep"];
}) {
  return (
    <div className="inspector-group">
      <h2>Sequencer — 16 steps</h2>
      <div className="sequencer-grid">
        {pattern.gate.map((gate, index) => (
          <div key={index} className="sequencer-step">
            <label className="sequencer-step-index">#{index + 1}</label>
            <label className="toggle-row">
              <span>gate</span>
              <input
                type="checkbox"
                checked={gate}
                disabled={isLoading || !onUpdateStep}
                onChange={(event) =>
                  onUpdateStep?.(nodeId, index, event.target.checked, undefined)
                }
              />
            </label>
            <label className="sequencer-step-cv">
              <span>cv</span>
              <input
                type="range"
                min={-1}
                max={1}
                step={0.01}
                value={pattern.cv[index]}
                disabled={isLoading || !onUpdateStep}
                onChange={(event) =>
                  onUpdateStep?.(nodeId, index, undefined, Number(event.target.value))
                }
              />
              <span className="sequencer-step-cv-value">{pattern.cv[index].toFixed(2)}</span>
            </label>
          </div>
        ))}
      </div>
    </div>
  );
}

/** Derive a human-readable display name for the selected node. */
function displayNameFor(node: Node): string {
  // Catalog identity is the source — prettify the snake_case id into Title Case
  // (e.g. "step_sequencer" → "Step Sequencer"). The catalog's displayName is
  // already the canonical pretty name; if we ever expose it on the Node we can
  // short-circuit here. For now derive from nodeTypeId.
  if (!node.nodeTypeId) {
    return "Node";
  }
  return node.nodeTypeId
    .split("_")
    .map((part) => (part.length > 0 ? part[0].toUpperCase() + part.slice(1) : part))
    .join(" ");
}
