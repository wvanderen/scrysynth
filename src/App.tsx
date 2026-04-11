import { useEffect } from "react";

import "./App.css";
import { GraphViewport } from "./components/session/GraphViewport";
import { NodeInspector } from "./components/session/NodeInspector";
import { SessionToolbar } from "./components/session/SessionToolbar";
import { useSessionStore } from "./store/sessionStore";

const DEFAULT_SAVE_PATH = "./scrysynth-session.json";

function App() {
  const {
    session,
    selectedNode,
    graphNodes,
    graphEdges,
    isLoading,
    error,
    bootstrapSession,
    newSession,
    saveSession,
    openSession,
    selectNode,
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

      <section className="workspace-grid">
        <GraphViewport
          graphNodes={graphNodes}
          graphEdges={graphEdges}
          onSelectNode={selectNode}
        />
        <NodeInspector selectedNode={selectedNode} />
      </section>

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
