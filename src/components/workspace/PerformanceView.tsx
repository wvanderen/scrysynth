import { useMemo } from "react";

import type { ActionHistoryEntry, MacroDefinition, Node, SceneDefinition, VariationDefinition } from "../../generated/session-types";
import { ActivityPanel } from "./ActivityPanel";
import { MacroEditor } from "./MacroEditor";
import { MacroSlider } from "./MacroSlider";
import { ScenePanel } from "./ScenePanel";
import { VariationPanel } from "./VariationPanel";

type PerformanceViewProps = {
  scenes: SceneDefinition[];
  variations: VariationDefinition[];
  enabledNodes: Node[];
  allNodes: Node[];
  macros: MacroDefinition[];
  actionHistory: ActionHistoryEntry[];
  isLoading: boolean;
  onRecallScene: (sceneId: string) => void;
  onSaveVariation: (name: string, sceneId: string) => void;
  onRestoreVariation: (variationId: string) => void;
  onCreateMacro: (definition: MacroDefinition) => void;
  onUpdateMacro: (macroId: string, updates: { name?: string; targets?: import("../../generated/session-types").MacroTarget[]; rangeStart?: number; rangeEnd?: number }) => void;
  onRemoveMacro: (macroId: string) => void;
  onSetMacroValue: (macroId: string, value: number) => void;
};

export function PerformanceView({
  scenes,
  variations,
  enabledNodes,
  allNodes,
  macros,
  actionHistory,
  isLoading,
  onRecallScene,
  onSaveVariation,
  onRestoreVariation,
  onCreateMacro,
  onUpdateMacro,
  onRemoveMacro,
  onSetMacroValue,
}: PerformanceViewProps) {
  const activeSceneId = useMemo(() => {
    const enabledIds = new Set(
      enabledNodes.filter((n) => n.enabled).map((n) => n.id),
    );

    if (scenes.length === 0 || enabledIds.size === 0) return null;

    let best: { sceneId: string; score: number } | null = null;
    for (const scene of scenes) {
      const sceneIds = new Set(scene.activeNodeIds);
      let matchCount = 0;
      let mismatchCount = 0;

      for (const id of enabledIds) {
        if (sceneIds.has(id)) matchCount++;
        else mismatchCount++;
      }
      for (const id of sceneIds) {
        if (!enabledIds.has(id)) mismatchCount++;
      }

      const score = matchCount - mismatchCount;
      if (!best || score > best.score) {
        best = { sceneId: scene.id, score };
      }
    }

    return best?.sceneId ?? null;
  }, [enabledNodes, scenes]);

  return (
    <section className="performance-view">
      <div className="panel-heading">
        <p className="eyebrow">Performance</p>
        <h2>Shape the live set with scenes and variations</h2>
      </div>

      <div className="performance-grid">
        <div className="performance-column">
          <ScenePanel
            scenes={scenes}
            activeSceneId={activeSceneId}
            isLoading={isLoading}
            onRecallScene={onRecallScene}
          />
        </div>

        <div className="performance-column">
          <VariationPanel
            variations={variations}
            scenes={scenes}
            activeSceneId={activeSceneId}
            isLoading={isLoading}
            onRestoreVariation={onRestoreVariation}
            onSaveVariation={onSaveVariation}
          />
        </div>
      </div>

      <div className="inspector-group" style={{ marginTop: 16 }}>
        <h2>Macro Controls</h2>
        {macros.length > 0 ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            {macros.map((macro) => (
              <MacroSlider
                key={macro.id}
                macroId={macro.id}
                macroName={macro.name}
                rangeStart={macro.rangeStart}
                rangeEnd={macro.rangeEnd}
                onValueChange={onSetMacroValue}
              />
            ))}
          </div>
        ) : (
          <p className="empty-copy">No macros defined.</p>
        )}
      </div>

      <MacroEditor
        macros={macros}
        nodes={allNodes}
        isLoading={isLoading}
        onCreateMacro={onCreateMacro}
        onUpdateMacro={onUpdateMacro}
        onRemoveMacro={onRemoveMacro}
      />

      <ActivityPanel actionHistory={actionHistory} />
    </section>
  );
}
