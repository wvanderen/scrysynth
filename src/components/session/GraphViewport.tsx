import {
  Background,
  Controls,
  MiniMap,
  ReactFlow,
  type Edge,
  type Node,
  type NodeMouseHandler,
} from "@xyflow/react";

import "@xyflow/react/dist/style.css";

type GraphViewportProps = {
  graphNodes: Node[];
  graphEdges: Edge[];
  onSelectNode: (nodeId: string | null) => void;
};

export function GraphViewport({ graphNodes, graphEdges, onSelectNode }: GraphViewportProps) {
  const handleNodeClick: NodeMouseHandler = (_, node) => {
    onSelectNode(node.id);
  };

  return (
    <section className="graph-panel">
      <div className="panel-heading">
        <p className="eyebrow">Graph viewport</p>
        <span>{graphNodes.length} nodes</span>
      </div>
      <div className="graph-surface">
        <ReactFlow
          nodes={graphNodes}
          edges={graphEdges}
          fitView
          nodesDraggable={false}
          nodesConnectable={false}
          elementsSelectable
          onNodeClick={handleNodeClick}
          onPaneClick={() => onSelectNode(null)}
        >
          <MiniMap pannable zoomable />
          <Controls showInteractive={false} />
          <Background color="#28514d" gap={24} />
        </ReactFlow>
      </div>
    </section>
  );
}
