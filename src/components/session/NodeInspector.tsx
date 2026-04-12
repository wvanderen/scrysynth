import type { Bus, Node } from "../../generated/session-types";

type NodeInspectorProps = {
  selectedNode: Node | null;
  buses: Bus[];
  isLoading: boolean;
  onToggleEnabled: (nodeId: string, enabled: boolean) => void;
  onUpdateParameter: (nodeId: string, parameterId: string, value: number) => void;
  onAssignNodeToBus: (nodeId: string, busId: string) => void;
  onClearNodeBus: (nodeId: string) => void;
};

export function NodeInspector({
  selectedNode,
  buses,
  isLoading,
  onToggleEnabled,
  onUpdateParameter,
  onAssignNodeToBus,
  onClearNodeBus,
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

  const assignedBusId = getAssignedBusId(selectedNode);

  return (
    <aside className="inspector-panel">
      <div className="panel-heading">
        <p className="eyebrow">Node inspector</p>
        <span>{selectedNode.nodeType}</span>
      </div>

      <div className="inspector-group">
        <h2>Identity</h2>
        <p><strong>id</strong> {selectedNode.id}</p>
        <p><strong>node type</strong> {selectedNode.nodeType}</p>
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
        <p><strong>controller</strong> {selectedNode.ownership.controller}</p>
        <p><strong>locked</strong> {selectedNode.ownership.isLocked ? "yes" : "no"}</p>
      </div>

      <div className="inspector-group">
        <h2>Ports</h2>
        {selectedNode.ports.map((port) => (
          <div key={port.id} className="list-card">
            <p>{port.name}</p>
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

function getAssignedBusId(node: Node): string | null {
  if (!node.audioPrimitive) {
    return null;
  }

  return node.audioPrimitive.config.busTargetId;
}
