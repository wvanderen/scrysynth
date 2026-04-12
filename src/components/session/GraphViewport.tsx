import {
  Background,
  type Connection,
  Controls,
  MiniMap,
  Panel,
  ReactFlow,
  type Edge,
  type Node,
  type OnConnect,
  type NodeMouseHandler,
} from "@xyflow/react";

import "@xyflow/react/dist/style.css";

type GraphViewportProps = {
  graphNodes: Node[];
  graphEdges: Edge[];
  onSelectNode: (nodeId: string | null) => void;
  onConnect: (connection: Connection) => void;
};

export function GraphViewport({ graphNodes, graphEdges, onSelectNode, onConnect }: GraphViewportProps) {
  const handleNodeClick: NodeMouseHandler = (_, node) => {
    onSelectNode(node.id);
  };

  const handleConnect: OnConnect = (connection) => {
    onConnect(connection);
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
          nodesConnectable
          elementsSelectable
          onNodeClick={handleNodeClick}
          onConnect={handleConnect}
          onPaneClick={() => onSelectNode(null)}
        >
          <Panel position="top-right" className="graph-hint">
            Patch source and effect nodes directly in the graph.
          </Panel>
          <MiniMap pannable zoomable />
          <Controls showInteractive={false} />
          <Background color="#28514d" gap={24} />
        </ReactFlow>
      </div>
    </section>
  );
}
