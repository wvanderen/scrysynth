import type { SceneDefinition } from "../../generated/session-types";

type ScenePanelProps = {
  scenes: SceneDefinition[];
  activeSceneId: string | null;
  isLoading: boolean;
  onRecallScene: (sceneId: string) => void;
};

export function ScenePanel({
  scenes,
  activeSceneId,
  isLoading,
  onRecallScene,
}: ScenePanelProps) {
  return (
    <div className="inspector-group">
      <h2>Scenes</h2>
      {scenes.length > 0 ? (
        scenes.map((scene) => {
          const isActive = scene.id === activeSceneId;

          return (
            <div
              key={scene.id}
              className={`list-card scene-card ${isActive ? "scene-card-active" : ""}`}
            >
              <div className="parameter-header">
                <p>
                  {isActive ? "\u25B6 " : ""}
                  {scene.name}
                </p>
                <span>{scene.activeNodeIds.length} nodes</span>
              </div>
              {scene.macroOverrides.length > 0 ? (
                <span className="scene-meta">
                  {scene.macroOverrides.length} macro override(s)
                </span>
              ) : null}
              <button
                type="button"
                disabled={isLoading || isActive}
                onClick={() => onRecallScene(scene.id)}
              >
                {isActive ? "Active" : "Recall"}
              </button>
            </div>
          );
        })
      ) : (
        <p className="empty-copy">No scenes defined in this session.</p>
      )}
    </div>
  );
}
