import { useMemo } from "react";

import type { ActionHistoryEntry, BindingTarget, HardwareBinding, MacroDefinition, Node, SceneDefinition, VariationDefinition } from "../../generated/session-types";
import { ActivityPanel } from "./ActivityPanel";
import { MacroEditor } from "./MacroEditor";
import { MacroSlider } from "./MacroSlider";
import { MidiLearnOverlay, formatSource, formatTarget } from "./MidiLearnOverlay";
import { ScenePanel } from "./ScenePanel";
import { VariationPanel } from "./VariationPanel";

type PerformanceViewProps = {
  scenes: SceneDefinition[];
  variations: VariationDefinition[];
  enabledNodes: Node[];
  allNodes: Node[];
  macros: MacroDefinition[];
  actionHistory: ActionHistoryEntry[];
  hardwareBindings: HardwareBinding[];
  isLoading: boolean;
  onRecallScene: (sceneId: string) => void;
  onSaveVariation: (name: string, sceneId: string) => void;
  onRestoreVariation: (variationId: string) => void;
  onCreateMacro: (definition: MacroDefinition) => void;
  onUpdateMacro: (macroId: string, updates: { name?: string; targets?: import("../../generated/session-types").MacroTarget[]; rangeStart?: number; rangeEnd?: number }) => void;
  onRemoveMacro: (macroId: string) => void;
  onSetMacroValue: (macroId: string, value: number) => void;
  onStartMidiLearn: (target: BindingTarget) => void;
  onRemoveHardwareBinding: (bindingId: string) => void;
};

export function PerformanceView({
  scenes,
  variations,
  enabledNodes,
  allNodes,
  macros,
  actionHistory,
  hardwareBindings,
  isLoading,
  onRecallScene,
  onSaveVariation,
  onRestoreVariation,
  onCreateMacro,
  onUpdateMacro,
  onRemoveMacro,
  onSetMacroValue,
  onStartMidiLearn,
  onRemoveHardwareBinding,
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

      <div className="inspector-group" style={{ marginTop: 16 }}>
        <h2>Hardware Bindings</h2>
        {macros.length > 0 && (
          <div style={{ display: "flex", flexWrap: "wrap", gap: 8, marginBottom: 12 }}>
            {macros.map((macro) => (
              <button
                key={macro.id}
                onClick={() => onStartMidiLearn({ kind: "macro", config: { macro_id: macro.id } })}
                disabled={isLoading}
                style={{
                  padding: "4px 12px",
                  background: "transparent",
                  border: "1px solid var(--color-accent, #6c63ff)",
                  color: "var(--color-accent, #6c63ff)",
                  borderRadius: 4,
                  cursor: "pointer",
                  fontSize: 12,
                }}
              >
                Learn: {macro.name}
              </button>
            ))}
            {scenes.map((scene) => (
              <button
                key={scene.id}
                onClick={() => onStartMidiLearn({ kind: "sceneRecall", config: { scene_id: scene.id } })}
                disabled={isLoading}
                style={{
                  padding: "4px 12px",
                  background: "transparent",
                  border: "1px solid var(--color-accent, #6c63ff)",
                  color: "var(--color-accent, #6c63ff)",
                  borderRadius: 4,
                  cursor: "pointer",
                  fontSize: 12,
                }}
              >
                Learn: {scene.name}
              </button>
            ))}
          </div>
        )}
        {(hardwareBindings ?? []).length > 0 ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
            {(hardwareBindings ?? []).map((binding) => (
              <div
                key={binding.id}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                  padding: "4px 8px",
                  background: "var(--color-bg-secondary, #2a2a3e)",
                  borderRadius: 4,
                }}
              >
                <span style={{ fontSize: 13 }}>
                  {formatSource(binding)} → {formatTarget(binding.target)}
                </span>
                <button
                  onClick={() => onRemoveHardwareBinding(binding.id)}
                  style={{
                    marginLeft: "auto",
                    padding: "2px 8px",
                    background: "transparent",
                    border: "1px solid #ff6b6b",
                    color: "#ff6b6b",
                    borderRadius: 3,
                    cursor: "pointer",
                    fontSize: 11,
                  }}
                >
                  Remove
                </button>
              </div>
            ))}
          </div>
        ) : (
          <p className="empty-copy">No hardware bindings. Click Learn to bind a hardware control.</p>
        )}
      </div>

      <ActivityPanel actionHistory={actionHistory} />

      <MidiLearnOverlay />
    </section>
  );
}
