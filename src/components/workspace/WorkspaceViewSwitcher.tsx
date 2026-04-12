import type { WorkspaceView } from "../../store/sessionStore";

type WorkspaceViewSwitcherProps = {
  currentView: WorkspaceView;
  onViewChange: (view: WorkspaceView) => void;
};

const VIEWS: { id: WorkspaceView; label: string }[] = [
  { id: "graph", label: "Graph" },
  { id: "conversation", label: "Conversation" },
  { id: "performance", label: "Performance" },
];

export function WorkspaceViewSwitcher({ currentView, onViewChange }: WorkspaceViewSwitcherProps) {
  return (
    <nav className="view-switcher">
      {VIEWS.map((view) => (
        <button
          key={view.id}
          type="button"
          className={`view-tab ${currentView === view.id ? "view-tab-active" : ""}`}
          onClick={() => onViewChange(view.id)}
        >
          {view.label}
        </button>
      ))}
    </nav>
  );
}
