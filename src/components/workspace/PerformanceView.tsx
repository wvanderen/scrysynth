import { useMemo } from "react";

import type {
  ActionHistoryEntry,
  BindingTarget,
  HardwareBinding,
  HardwareRuntimeSettings,
  HardwareRuntimeStatus,
  MacroDefinition,
  MidiInputPort,
  Node,
  SceneDefinition,
  VariationDefinition,
} from "../../generated/session-types";
import { ActivityPanel } from "./ActivityPanel";
import { HardwarePanel } from "./HardwarePanel";
import { MacroEditor } from "./MacroEditor";
import { MacroSlider } from "./MacroSlider";
import { MidiLearnOverlay } from "./MidiLearnOverlay";
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
  hardwareSettings: HardwareRuntimeSettings | null;
  hardwareStatus: HardwareRuntimeStatus | null;
  midiInputPorts: MidiInputPort[];
  isLoading: boolean;
  onRecallScene: (sceneId: string) => void;
  onSaveVariation: (name: string, sceneId: string) => void;
  onRestoreVariation: (variationId: string) => void;
  onCreateMacro: (definition: MacroDefinition) => void;
  onUpdateMacro: (macroId: string, updates: { name?: string; targets?: import("../../generated/session-types").MacroTarget[]; rangeStart?: number; rangeEnd?: number }) => void;
  onRemoveMacro: (macroId: string) => void;
  onSetMacroValue: (macroId: string, value: number) => void;
  onRefreshHardware: () => void;
  onUpdateHardwareSettings: (settings: HardwareRuntimeSettings) => void;
  onStartHardwareRuntime: (settings: HardwareRuntimeSettings) => void;
  onStopHardwareRuntime: () => void;
  onStartMidiLearn: (target: BindingTarget) => void;
  onStopMidiLearn: () => void;
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
  hardwareSettings,
  hardwareStatus,
  midiInputPorts,
  isLoading,
  onRecallScene,
  onSaveVariation,
  onRestoreVariation,
  onCreateMacro,
  onUpdateMacro,
  onRemoveMacro,
  onSetMacroValue,
  onRefreshHardware,
  onUpdateHardwareSettings,
  onStartHardwareRuntime,
  onStopHardwareRuntime,
  onStartMidiLearn,
  onStopMidiLearn,
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
      <div className="panel-heading performance-heading">
        <div>
          <p className="eyebrow">Performance</p>
          <h2>Live control surface</h2>
        </div>
        <span className="queue-count-pill">{macros.length} macros</span>
      </div>

      <div className="performance-cockpit-grid">
        <div className="performance-pad-grid">
          <ScenePanel
            scenes={scenes}
            activeSceneId={activeSceneId}
            isLoading={isLoading}
            onRecallScene={onRecallScene}
          />

          <VariationPanel
            variations={variations}
            scenes={scenes}
            activeSceneId={activeSceneId}
            isLoading={isLoading}
            onRestoreVariation={onRestoreVariation}
            onSaveVariation={onSaveVariation}
          />
        </div>

        <div className="performance-macro-strip">
          <div className="performance-section-heading">
            <h2>Macro Controls</h2>
            <span>{macros.length} assigned</span>
          </div>
          {macros.length > 0 ? (
            <div className="macro-control-grid">
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

        <div className="performance-editor-drawer">
          <MacroEditor
            macros={macros}
            nodes={allNodes}
            isLoading={isLoading}
            onCreateMacro={onCreateMacro}
            onUpdateMacro={onUpdateMacro}
            onRemoveMacro={onRemoveMacro}
          />
        </div>

        <div className="performance-hardware-drawer">
          <HardwarePanel
            bindings={hardwareBindings ?? []}
            settings={hardwareSettings}
            status={hardwareStatus}
            midiInputPorts={midiInputPorts}
            macros={macros}
            scenes={scenes}
            isLoading={isLoading}
            onRefresh={onRefreshHardware}
            onUpdateSettings={onUpdateHardwareSettings}
            onStartListeners={onStartHardwareRuntime}
            onStopListeners={onStopHardwareRuntime}
            onStartLearn={onStartMidiLearn}
            onCancelLearn={onStopMidiLearn}
            onRemoveBinding={onRemoveHardwareBinding}
          />
        </div>
      </div>

      <div className="performance-activity-strip">
        <ActivityPanel actionHistory={actionHistory} />
      </div>

      <MidiLearnOverlay />
    </section>
  );
}
