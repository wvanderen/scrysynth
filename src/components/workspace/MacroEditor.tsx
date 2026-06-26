import { useCallback, useState } from "react";

import type { MacroDefinition, MacroTarget, Node } from "../../generated/session-types";

type MacroEditorProps = {
  macros: MacroDefinition[];
  nodes: Node[];
  isLoading: boolean;
  onCreateMacro: (definition: MacroDefinition) => void;
  onUpdateMacro: (macroId: string, updates: { name?: string; targets?: MacroTarget[]; rangeStart?: number; rangeEnd?: number }) => void;
  onRemoveMacro: (macroId: string) => void;
};

export function MacroEditor({
  macros,
  nodes,
  isLoading,
  onCreateMacro,
  onUpdateMacro,
  onRemoveMacro,
}: MacroEditorProps) {
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const [editRangeStart, setEditRangeStart] = useState(0);
  const [editRangeEnd, setEditRangeEnd] = useState(1);

  const handleAdd = useCallback(() => {
    const id = `macro-${globalThis.crypto.randomUUID()}`;
    onCreateMacro({
      id,
      name: `Macro ${macros.length + 1}`,
      targetParameterIds: [],
      rangeStart: 0,
      rangeEnd: 1,
      targets: [],
    });
  }, [macros.length, onCreateMacro]);

  const handleStartEdit = useCallback((macro: MacroDefinition) => {
    setEditingId(macro.id);
    setEditName(macro.name);
    setEditRangeStart(macro.rangeStart);
    setEditRangeEnd(macro.rangeEnd);
  }, []);

  const handleSaveEdit = useCallback(
    (macroId: string) => {
      onUpdateMacro(macroId, {
        name: editName,
        rangeStart: editRangeStart,
        rangeEnd: editRangeEnd,
      });
      setEditingId(null);
    },
    [editName, editRangeStart, editRangeEnd, onUpdateMacro],
  );

  const handleAddAudioTarget = useCallback(
    (macroId: string, nodeId: string, parameterId: string) => {
      const macro = macros.find((m) => m.id === macroId);
      if (!macro) return;
      const existing = macro.targets ?? [];
      onUpdateMacro(macroId, {
        targets: [...existing, { kind: "audioParameter", config: { node_id: nodeId, parameter_id: parameterId } }],
      });
    },
    [macros, onUpdateMacro],
  );

  const handleAddVisualTarget = useCallback(
    (macroId: string, elementId: string, parameterId: string) => {
      const macro = macros.find((m) => m.id === macroId);
      if (!macro) return;
      const existing = macro.targets ?? [];
      onUpdateMacro(macroId, {
        targets: [...existing, { kind: "visualParameter", config: { element_id: elementId, parameter_id: parameterId } }],
      });
    },
    [macros, onUpdateMacro],
  );

  const handleRemoveTarget = useCallback(
    (macroId: string, targetIndex: number) => {
      const macro = macros.find((m) => m.id === macroId);
      if (!macro) return;
      const updated = [...(macro.targets ?? [])];
      updated.splice(targetIndex, 1);
      onUpdateMacro(macroId, { targets: updated });
    },
    [macros, onUpdateMacro],
  );

  return (
    <div className="inspector-group">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h2>Macros</h2>
        <button type="button" disabled={isLoading} onClick={handleAdd}>
          + Add Macro
        </button>
      </div>

      {macros.length === 0 ? (
        <p className="empty-copy">No macros defined.</p>
      ) : (
        macros.map((macro) => {
          const isEditing = editingId === macro.id;
          const targets = macro.targets ?? [];

          return (
            <div key={macro.id} className="list-card" style={{ padding: 12 }}>
              {isEditing ? (
                <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                  <input
                    type="text"
                    value={editName}
                    onChange={(e) => setEditName(e.target.value)}
                    style={{ background: "#1a2a28", color: "#f2eee5", border: "1px solid #2d4442", borderRadius: 6, padding: "4px 8px" }}
                  />
                  <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                    <label style={{ color: "#d9c8a0", fontSize: 12 }}>Range</label>
                    <input
                      type="number"
                      value={editRangeStart}
                      onChange={(e) => setEditRangeStart(Number(e.target.value))}
                      style={{ background: "#1a2a28", color: "#f2eee5", border: "1px solid #2d4442", borderRadius: 6, padding: "4px 8px", width: 60 }}
                    />
                    <span style={{ color: "#d9c8a0" }}>—</span>
                    <input
                      type="number"
                      value={editRangeEnd}
                      onChange={(e) => setEditRangeEnd(Number(e.target.value))}
                      style={{ background: "#1a2a28", color: "#f2eee5", border: "1px solid #2d4442", borderRadius: 6, padding: "4px 8px", width: 60 }}
                    />
                  </div>
                  <div style={{ display: "flex", gap: 4 }}>
                    <button type="button" onClick={() => handleSaveEdit(macro.id)}>Save</button>
                    <button type="button" onClick={() => setEditingId(null)}>Cancel</button>
                  </div>
                </div>
              ) : (
                <div>
                  <div className="parameter-header" style={{ marginBottom: 4 }}>
                    <p style={{ fontWeight: 600 }}>{macro.name}</p>
                    <span style={{ color: "#d9c8a0", fontSize: 12 }}>
                      {macro.rangeStart} — {macro.rangeEnd}
                    </span>
                  </div>

                  {targets.length > 0 && (
                    <div style={{ marginBottom: 8 }}>
                      {targets.map((target, idx) => (
                        <div
                          key={idx}
                          style={{
                            display: "flex",
                            justifyContent: "space-between",
                            alignItems: "center",
                            padding: "2px 0",
                            fontSize: 12,
                            color: "#a0b0aa",
                          }}
                        >
                          <span>
                            {target.kind === "audioParameter"
                              ? `Audio: ${target.config.node_id}/${target.config.parameter_id}`
                              : `Visual: ${target.config.element_id}/${target.config.parameter_id}`}
                          </span>
                          <button
                            type="button"
                            onClick={() => handleRemoveTarget(macro.id, idx)}
                            style={{ background: "none", border: "none", color: "#e05050", cursor: "pointer", fontSize: 12, padding: 0 }}
                          >
                            ×
                          </button>
                        </div>
                      ))}
                    </div>
                  )}

                  {targetParameterIds(macro).length > 0 && (
                    <div style={{ marginBottom: 8 }}>
                      {targetParameterIds(macro).map((pid) => (
                        <div key={pid} style={{ fontSize: 12, color: "#808080" }}>
                          Legacy: {pid}
                        </div>
                      ))}
                    </div>
                  )}

                  <div style={{ display: "flex", gap: 4, flexWrap: "wrap" }}>
                    <AudioTargetSelector nodes={nodes} macroId={macro.id} onSelect={handleAddAudioTarget} />
                    <VisualTargetSelector macroId={macro.id} onSelect={handleAddVisualTarget} />
                    <button type="button" disabled={isLoading} onClick={() => handleStartEdit(macro)}>
                      Edit
                    </button>
                    <button type="button" disabled={isLoading} onClick={() => onRemoveMacro(macro.id)}>
                      Delete
                    </button>
                  </div>
                </div>
              )}
            </div>
          );
        })
      )}
    </div>
  );
}

function targetParameterIds(macro: MacroDefinition): string[] {
  return macro.targetParameterIds ?? [];
}

type AudioTargetSelectorProps = {
  nodes: Node[];
  macroId: string;
  onSelect: (macroId: string, nodeId: string, parameterId: string) => void;
};

function AudioTargetSelector({ nodes, macroId, onSelect }: AudioTargetSelectorProps) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div style={{ position: "relative", display: "inline-block" }}>
      <button type="button" onClick={() => setExpanded(!expanded)} style={{ fontSize: 12 }}>
        + Audio
      </button>
      {expanded && (
        <div
          style={{
            position: "absolute",
            top: "100%",
            left: 0,
            background: "#112725",
            border: "1px solid #2d4442",
            borderRadius: 6,
            padding: 8,
            zIndex: 10,
            minWidth: 200,
            maxHeight: 200,
            overflow: "auto",
          }}
        >
          {nodes.flatMap((node) =>
            node.parameters.map((param) => (
              <button
                key={`${node.id}-${param.id}`}
                type="button"
                onClick={() => {
                  onSelect(macroId, node.id, param.id);
                  setExpanded(false);
                }}
                style={{
                  display: "block",
                  width: "100%",
                  textAlign: "left",
                  background: "none",
                  border: "none",
                  color: "#f2eee5",
                  padding: "4px 0",
                  fontSize: 12,
                  cursor: "pointer",
                }}
              >
                {node.nodeTypeId}: {param.name}
              </button>
            )),
          )}
        </div>
      )}
    </div>
  );
}

type VisualTargetSelectorProps = {
  macroId: string;
  onSelect: (macroId: string, elementId: string, parameterId: string) => void;
};

function VisualTargetSelector({ macroId, onSelect }: VisualTargetSelectorProps) {
  const [elementId, setElementId] = useState("");
  const [parameterId, setParameterId] = useState("");

  return (
    <div style={{ display: "inline-flex", gap: 4, alignItems: "center" }}>
      <input
        type="text"
        placeholder="element id"
        value={elementId}
        onChange={(e) => setElementId(e.target.value)}
        style={{ background: "#1a2a28", color: "#f2eee5", border: "1px solid #2d4442", borderRadius: 6, padding: "2px 6px", fontSize: 12, width: 80 }}
      />
      <input
        type="text"
        placeholder="param id"
        value={parameterId}
        onChange={(e) => setParameterId(e.target.value)}
        style={{ background: "#1a2a28", color: "#f2eee5", border: "1px solid #2d4442", borderRadius: 6, padding: "2px 6px", fontSize: 12, width: 80 }}
      />
      <button
        type="button"
        disabled={!elementId || !parameterId}
        onClick={() => {
          onSelect(macroId, elementId, parameterId);
          setElementId("");
          setParameterId("");
        }}
        style={{ fontSize: 12 }}
      >
        + Visual
      </button>
    </div>
  );
}
