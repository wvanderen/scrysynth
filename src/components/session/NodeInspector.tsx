import type { Node } from "../../generated/session-types";

type NodeInspectorProps = {
  selectedNode: Node | null;
};

export function NodeInspector({ selectedNode }: NodeInspectorProps) {
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
              <p>{parameter.name}</p>
              <span>
                {parameter.value} {parameter.unit}
              </span>
            </div>
          ))
        ) : (
          <p className="empty-copy">No parameters on this node.</p>
        )}
      </div>
    </aside>
  );
}
