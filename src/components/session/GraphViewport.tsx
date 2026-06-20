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

function minimapNodeColor(node: Node) {
  if (node.id.startsWith("source")) {
    return "#38bdf8";
  }

  if (node.id.startsWith("output")) {
    return "#34d399";
  }

  if (node.id.startsWith("effect")) {
    return "#a78bfa";
  }

  if (node.id.startsWith("mixer")) {
    return "#f59e0b";
  }

  return "#f59e0b";
}

export function GraphViewport({ graphNodes, graphEdges, onSelectNode, onConnect }: GraphViewportProps) {
  const handleNodeClick: NodeMouseHandler = (_, node) => {
    onSelectNode(node.id);
  };

  const handleConnect: OnConnect = (connection) => {
    onConnect(connection);
  };

  return (
    <section className="graph-panel">
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
          <Panel position="top-left" className="graph-count">
            {graphNodes.length} nodes
          </Panel>
          <Panel position="top-right" className="graph-hint">
            Patch handles
          </Panel>
          <MiniMap
            pannable
            zoomable
            bgColor="#10141d"
            maskColor="rgba(9, 11, 16, 0.72)"
            nodeColor={minimapNodeColor}
            nodeStrokeColor="#eef2f7"
            nodeStrokeWidth={3}
            nodeBorderRadius={8}
          />
          <Controls showInteractive={false} />
          <Background color="#2f3847" gap={24} />
        </ReactFlow>
      </div>
    </section>
  );
}
