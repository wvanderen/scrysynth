import { useEffect, useState } from "react";

import type {
  Node,
  NodeCatalogEntry,
  SessionDocument,
  SequencerPattern,
} from "../../generated/session-types";
import { getNodeCatalog } from "../../lib/session-client";

type PrimitivePaletteProps = {
  session: SessionDocument | null;
  selectedNode: Node | null;
  isLoading: boolean;
  onAddNode: (node: Node) => void;
  onRemoveNode: (nodeId: string) => void;
};

/**
 * Catalog-driven node palette (NODES-01 success criterion #4).
 *
 * Iterates the compiled-in `NodeCatalogEntry[]` (fetched once via the
 * `get_node_catalog` Tauri command and cached) and renders one button per
 * entry. Clicking builds a `Node` directly from the entry's ports/parameters/
 * defaults — replacing v1's hardcoded `buildPrimitiveNode` factory and fixing
 * the v1 `runtimeTarget: audio/source/${id}` quirk (T-12-09: runtimeTarget is
 * now the catalog entry id, never a node-id-templated string).
 */
export function PrimitivePalette({
  session,
  selectedNode,
  isLoading,
  onAddNode,
  onRemoveNode,
}: PrimitivePaletteProps) {
  const [catalog, setCatalog] = useState<NodeCatalogEntry[]>([]);

  useEffect(() => {
    let cancelled = false;
    getNodeCatalog()
      .then((entries) => {
        if (!cancelled) setCatalog(entries);
      })
      .catch(() => {
        // Schema/transport failure leaves the palette empty rather than
        // crashing the workspace; the inspector still works against the
        // current session.
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const addCatalogNode = (entry: NodeCatalogEntry) => {
    onAddNode(buildNodeFromCatalogEntry(entry, session));
  };

  return (
    <section className="primitive-palette">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Palette</p>
          <h2>Nodes</h2>
        </div>
        <span>{session?.nodes.length ?? 0} voices</span>
      </div>

      <div className="palette-actions">
        {catalog.length === 0 ? (
          <p className="palette-caption">Loading catalog…</p>
        ) : (
          catalog.map((entry) => (
            <button
              key={entry.id}
              type="button"
              onClick={() => addCatalogNode(entry)}
              disabled={isLoading}
              title={entry.id}
            >
              Add {entry.displayName}
            </button>
          ))
        )}
      </div>

      <p className="palette-caption">
        Drag a node handle onto another to reroute the live graph without leaving the instrument.
      </p>

      <button
        type="button"
        className="destructive-button"
        onClick={() => selectedNode && onRemoveNode(selectedNode.id)}
        disabled={isLoading || !selectedNode}
      >
        Remove Selected
      </button>
    </section>
  );
}

/**
 * Build a canonical `Node` from a catalog entry. Identity is the catalog
 * `node_type_id`; ports come straight from the entry; parameters carry the
 * entry's default/min/max/unit; `runtimeTarget` is the entry id (NOT a
 * node-id-templated string — fixes the v1 quirk).
 *
 * `step_sequencer` nodes additionally seed a default 16-step pattern so the
 * controller has something to tick through (D-07/D-08).
 */
export function buildNodeFromCatalogEntry(
  entry: NodeCatalogEntry,
  session: SessionDocument | null,
): Node {
  const suffix = (globalThis.crypto?.randomUUID?.() ?? String(Math.random())).slice(0, 8);
  const id = `${entry.id}-${suffix}`;
  const sceneId = session?.scenes[0]?.id;
  const firstBusId = session?.buses[0]?.id ?? null;

  // Map CatalogPortSpec → Port (same shape, but the runtime Node carries its
  // own port copies so live port edits don't mutate the static catalog).
  const ports = entry.ports.map((port) => ({
    id: `${id}-${port.id}`,
    name: port.name,
    direction: port.direction,
    signalType: port.signalType,
  }));

  // Catalog params carry defaults; the node snapshots them at creation time.
  const parameters = entry.parameters.map((param) => ({
    id: `${id}-${param.id}`,
    name: param.id,
    value: param.defaultValue,
    defaultValue: param.defaultValue,
    minValue: param.minValue,
    maxValue: param.maxValue,
    unit: param.unit,
  }));

  // Per-node config: sources/effects/mixers/output default onto the first
  // session bus so they produce sound immediately; the user can re-route
  // later. Modulators/sequencer/utility (control-rate or app-driven) skip
  // the bus assignment.
  const needsBusTarget =
    entry.category === "source" ||
    entry.category === "effect" ||
    entry.category === "mixer" ||
    entry.category === "output";

  const node: Node = {
    id,
    nodeTypeId: entry.id,
    ports,
    parameters,
    // T-12-09: runtimeTarget is the catalog entry id, NOT `audio/<kind>/<id>`.
    runtimeTarget: entry.id,
    sceneMembership: sceneId ? [sceneId] : [],
    ownership: { controller: "shared", isLocked: false },
    enabled: true,
    busTargetId: needsBusTarget ? firstBusId : null,
    // Output-kind/channel config only meaningful for `output` nodes.
    outputKind: entry.category === "output" ? "master" : null,
    channelCount: entry.category === "output" ? 2 : null,
    bypassed: entry.category === "effect" ? false : null,
    channelMode:
      entry.category === "source" || entry.category === "mixer" ? "mono" : null,
    // D-07/D-08: seed the sequencer pattern with a silent default so the
    // app-driven tick loop has something to play; the inspector editor
    // mutates it via SetStepValue.
    sequencerPattern:
      entry.id === "step_sequencer"
        ? {
            gate: Array(16).fill(false) as SequencerPattern["gate"],
            cv: Array(16).fill(0) as SequencerPattern["cv"],
          }
        : null,
  };

  return node;
}
