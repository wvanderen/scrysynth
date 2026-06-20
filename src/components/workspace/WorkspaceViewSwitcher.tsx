import type { WorkspaceView } from "../../store/sessionStore";

type WorkspaceViewSwitcherProps = {
  currentView: WorkspaceView;
  onViewChange: (view: WorkspaceView) => void;
};

const VIEWS: { id: WorkspaceView; label: string }[] = [
  { id: "graph", label: "Graph" },
  { id: "performance", label: "Performance" },
  { id: "conversation", label: "Conversation" },
  { id: "runtime", label: "Runtime" },
  { id: "hardware", label: "Hardware" },
];

export function WorkspaceViewSwitcher({ currentView, onViewChange }: WorkspaceViewSwitcherProps) {
  return (
    <nav className="mode-rail" aria-label="Workspace mode">
      {VIEWS.map((view) => (
        <button
          key={view.id}
          type="button"
          className={`mode-rail-button ${currentView === view.id ? "mode-rail-button-active" : ""}`}
          onClick={() => onViewChange(view.id)}
          aria-current={currentView === view.id ? "page" : undefined}
        >
          {view.label}
        </button>
      ))}
    </nav>
  );
}
