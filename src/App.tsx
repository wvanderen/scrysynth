import { useEffect } from "react";

import type { Connection } from "@xyflow/react";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";

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
    actionHistory,
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
    reclaimOwnership,
    setNodeOwnership,
    createMacro,
    updateMacro,
    removeMacro,
    setMacroValue,
    hardwareBindings,
    hardwareSettings,
    hardwareStatus,
    midiInputPorts,
    refreshHardwareRuntime,
    updateHardwareSettings,
    startHardwareRuntime,
    stopHardwareRuntime,
    startMidiLearn,
    stopMidiLearn,
    removeHardwareBinding,
  } = useSessionStore();

  useEffect(() => {
    void bootstrapSession();
  }, [bootstrapSession]);

  const handleSaveSession = async () => {
    const path = await saveDialog({
      defaultPath: DEFAULT_SAVE_PATH,
      filters: [{ name: "Session", extensions: ["json"] }],
    });
    if (!path) {
      return;
    }

    void saveSession(path);
  };

  const handleOpenSession = async () => {
    const path = await openDialog({
      filters: [{ name: "Session", extensions: ["json"] }],
      multiple: false,
    });
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
            onReclaimOwnership={(nodeIds) => void reclaimOwnership(nodeIds)}
            onSetNodeOwnership={(nodeIds, controller) => void setNodeOwnership(nodeIds, controller)}
          />
        </section>
      ) : null}

      {workspaceView === "conversation" ? (
        <ConversationView />
      ) : null}

      {workspaceView === "performance" ? (
        <PerformanceView
          scenes={session?.scenes ?? []}
          variations={session?.variations ?? []}
          enabledNodes={session?.nodes ?? []}
          allNodes={session?.nodes ?? []}
          macros={session?.macros ?? []}
          actionHistory={actionHistory}
          isLoading={isLoading}
          onRecallScene={(sceneId) => void recallScene(sceneId)}
          onSaveVariation={(name, sceneId) => void saveVariation(name, sceneId)}
          onRestoreVariation={(variationId) => void restoreVariation(variationId)}
          onCreateMacro={(definition) => void createMacro(definition)}
          onUpdateMacro={(macroId, updates) => void updateMacro(macroId, updates)}
          onRemoveMacro={(macroId) => void removeMacro(macroId)}
          onSetMacroValue={(macroId, value) => void setMacroValue(macroId, value)}
          hardwareBindings={hardwareBindings ?? []}
          hardwareSettings={hardwareSettings}
          hardwareStatus={hardwareStatus}
          midiInputPorts={midiInputPorts}
          onRefreshHardware={() => void refreshHardwareRuntime()}
          onUpdateHardwareSettings={(settings) => void updateHardwareSettings(settings)}
          onStartHardwareRuntime={() => void startHardwareRuntime()}
          onStopHardwareRuntime={() => void stopHardwareRuntime()}
          onStartMidiLearn={(target) => void startMidiLearn(target)}
          onStopMidiLearn={() => void stopMidiLearn()}
          onRemoveHardwareBinding={(bindingId) => void removeHardwareBinding(bindingId)}
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
