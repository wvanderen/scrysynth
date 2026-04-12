import type { SceneDefinition, VariationDefinition } from "../../generated/session-types";

type VariationPanelProps = {
  variations: VariationDefinition[];
  scenes: SceneDefinition[];
  activeSceneId: string | null;
  isLoading: boolean;
  onRestoreVariation: (variationId: string) => void;
  onSaveVariation: (name: string, sceneId: string) => void;
};

export function VariationPanel({
  variations,
  scenes,
  activeSceneId,
  isLoading,
  onRestoreVariation,
  onSaveVariation,
}: VariationPanelProps) {
  const activeScene = scenes.find((scene) => scene.id === activeSceneId);
  const activeSceneName = activeScene?.name ?? "No active scene";
  const filteredVariations = activeScene
    ? variations.filter((v) => v.sceneId === activeScene.id)
    : variations;

  return (
    <>
      <div className="inspector-group">
        <h2>Variations for {activeSceneName}</h2>
        {filteredVariations.length > 0 ? (
          filteredVariations.map((variation) => (
            <div key={variation.id} className="list-card">
              <div className="parameter-header">
                <p>{variation.name}</p>
                <span>{variation.parameterOverrides.length} params</span>
              </div>
              <button
                type="button"
                disabled={isLoading}
                onClick={() => onRestoreVariation(variation.id)}
              >
                Restore
              </button>
            </div>
          ))
        ) : (
          <p className="empty-copy">No variations for this scene.</p>
        )}
      </div>

      <div className="inspector-group">
        <h2>Save Variation</h2>
        {activeScene ? (
          <div className="save-variation-row">
            <p className="variation-save-hint">
              Snapshot current parameters as a variation of <strong>{activeSceneName}</strong>.
            </p>
            <button
              type="button"
              disabled={isLoading}
              onClick={() => {
                const name = window.prompt("Variation name");
                if (name) {
                  onSaveVariation(name, activeScene.id);
                }
              }}
            >
              Save Current State
            </button>
          </div>
        ) : (
          <p className="empty-copy">Recall a scene first to save variations.</p>
        )}
      </div>
    </>
  );
}
