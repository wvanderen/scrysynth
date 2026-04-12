import { useEffect } from "react";

import type { Connection } from "@xyflow/react";

import "./App.css";
import { AudioTransportStrip } from "./components/audio/AudioTransportStrip";
import { PrimitivePalette } from "./components/audio/PrimitivePalette";
import { GraphViewport } from "./components/session/GraphViewport";
import { NodeInspector } from "./components/session/NodeInspector";
import { SessionToolbar } from "./components/session/SessionToolbar";
import { ConversationView } from "./components/workspace/ConversationView";
import { PerformanceView } from "./components/workspace/PerformanceView";
import { WorkspaceViewSwitcher } from "./components/workspace/WorkspaceViewSwitcher";
import { useSessionStore } from "./store/sessionStore";

const DEFAULT_SAVE_PATH = "./scrysynth-session.json";

function App() {
  const {
    session,
    selectedNode,
    graphNodes,
    graphEdges,
    audioRuntime,
    isLoading,
    error,
    workspaceView,
    bootstrapSession,
    newSession,
    saveSession,
    openSession,
    selectNode,
    addNode,
    removeNode,
    connectNodes,
    assignNodeToBus,
    clearNodeBusAssignment,
    updateNodeParameter,
    toggleNodeEnabled,
    startAudio,
    stopAudio,
    panicAudio,
    setWorkspaceView,
    recallScene,
    saveVariation,
    restoreVariation,
  } = useSessionStore();

  useEffect(() => {
    void bootstrapSession();
  }, [bootstrapSession]);

  const handleSaveSession = () => {
    const path = window.prompt("Save session to path", DEFAULT_SAVE_PATH);
    if (!path) {
      return;
    }

    void saveSession(path);
  };

  const handleOpenSession = () => {
    const path = window.prompt("Open session from path", DEFAULT_SAVE_PATH);
    if (!path) {
      return;
    }

    void openSession(path);
  };

  const handleConnect = (connection: Connection) => {
    if (!connection.source || !connection.target) {
      return;
    }

    void connectNodes(connection.source, connection.target);
  };

  return (
    <main className="workspace-shell">
      <SessionToolbar
        title={session?.title ?? "Loading Session"}
        isLoading={isLoading}
        onNewSession={() => void newSession()}
        onSaveSession={handleSaveSession}
        onOpenSession={handleOpenSession}
      />

      {error ? <div className="error-banner">{error}</div> : null}

      <AudioTransportStrip
        runtime={audioRuntime}
        isLoading={isLoading}
        onStart={() => void startAudio()}
        onStop={() => void stopAudio()}
        onPanic={() => void panicAudio()}
      />

      <WorkspaceViewSwitcher currentView={workspaceView} onViewChange={setWorkspaceView} />

      {workspaceView === "graph" ? (
        <section className="workspace-grid">
          <div className="workspace-main-column">
            <GraphViewport
              graphNodes={graphNodes}
              graphEdges={graphEdges}
              onSelectNode={selectNode}
              onConnect={handleConnect}
            />
            <PrimitivePalette
              session={session}
              selectedNode={selectedNode}
              isLoading={isLoading}
              onAddNode={(node) => void addNode(node)}
              onRemoveNode={(nodeId) => void removeNode(nodeId)}
            />
          </div>
          <NodeInspector
            selectedNode={selectedNode}
            buses={session?.buses ?? []}
            isLoading={isLoading}
            onToggleEnabled={(nodeId, enabled) => void toggleNodeEnabled(nodeId, enabled)}
            onUpdateParameter={(nodeId, parameterId, value) =>
              void updateNodeParameter(nodeId, parameterId, value)
            }
            onAssignNodeToBus={(nodeId, busId) => void assignNodeToBus(nodeId, busId)}
            onClearNodeBus={(nodeId) => void clearNodeBusAssignment(nodeId)}
          />
        </section>
      ) : null}

      {workspaceView === "conversation" ? (
        <ConversationView sessionTitle={session?.title ?? "No session"} />
      ) : null}

      {workspaceView === "performance" ? (
        <PerformanceView
          scenes={session?.scenes ?? []}
          variations={session?.variations ?? []}
          enabledNodes={session?.nodes ?? []}
          isLoading={isLoading}
          onRecallScene={(sceneId) => void recallScene(sceneId)}
          onSaveVariation={(name, sceneId) => void saveVariation(name, sceneId)}
          onRestoreVariation={(variationId) => void restoreVariation(variationId)}
        />
      ) : null}

      <footer className="runtime-strip">
        {(session?.runtimeStatus ?? []).map((runtime) => (
          <div key={runtime.id} className="runtime-pill">
            <strong>{runtime.runtime}</strong>
            <span>{runtime.status}</span>
            <small>{runtime.targetId ?? "no target"}</small>
          </div>
        ))}
      </footer>
    </main>
  );
}

export default App;
