import { beforeEach, describe, expect, it } from "vitest";

import type { Node, SequencerPattern } from "../generated/session-types";
import {
  applyGraphEdit,
  approvePendingAction,
  createDefaultSession,
  getCurrentSession,
  getNodeCatalog,
  sendAgentMessage,
  startAudioRuntime,
  __testNodeCatalogEntrySchema,
  __testNodeSchema,
  __testSequencerPatternSchema,
} from "./session-client";

describe("session client browser preview", () => {
  beforeEach(async () => {
    await createDefaultSession();
  });

  it("loads a seeded session without a Tauri invoke bridge", async () => {
    const session = await getCurrentSession();

    expect(session.title).toBe("Default Scrysynth Session");
    expect(session.nodes.length).toBeGreaterThan(0);
    expect(session.routes.length).toBeGreaterThan(0);
    expect(session.nodes.map((node) => node.id)).toContain("preview-source");
  });

  it("applies graph edits against the in-memory preview session", async () => {
    const session = await getCurrentSession();
    const node = buildPreviewEffectNode(session.scenes[0]?.id, session.buses[0]?.id);

    const updated = await applyGraphEdit({ type: "addNode", payload: { node } });

    expect(updated.nodes.some((candidate) => candidate.id === node.id)).toBe(true);
  });

  it("supports runtime and pending-action smoke paths in browser preview mode", async () => {
    const running = await startAudioRuntime();

    expect(running.audioRuntime.lifecycle).toBe("running");

    const agentResult = await sendAgentMessage("remove preview-source");
    const pendingAction = agentResult.session.pendingActions[0];

    expect(agentResult.intent.parsedCommands).toHaveLength(1);
    expect(pendingAction.status).toBe("pending");

    const approved = await approvePendingAction(pendingAction.id);

    expect(approved.pendingActions.filter((action) => action.status === "pending")).toHaveLength(0);
    expect(approved.nodes.some((node) => node.id === "preview-source")).toBe(false);
  });

  it("exposes the compiled-in catalog via get_node_catalog (NODES-01 #4)", async () => {
    const catalog = await getNodeCatalog();

    expect(catalog.length).toBeGreaterThan(0);
    // The catalog must cover every family the palette offers (NODES-02/03/04).
    const ids = catalog.map((entry) => entry.id);
    expect(ids).toContain("oscillator");
    expect(ids).toContain("filter");
    expect(ids).toContain("step_sequencer");
    expect(ids).toContain("output");

    // Each entry round-trips through the catalog Zod schema (Pitfall #5 guard).
    for (const entry of catalog) {
      expect(() => __testNodeCatalogEntrySchema.parse(entry)).not.toThrow();
    }

    // The step sequencer is app-driven — no SynthDef (D-06).
    const sequencer = catalog.find((entry) => entry.id === "step_sequencer");
    expect(sequencer?.synthdefName).toBe("");
    expect(sequencer?.synthdefResource).toBe("");
  });

  it("round-trips a v2 catalog-derived session through the relaxed Zod schema (Pitfall #5)", () => {
    // Build a session with a catalog node (oscillator + filter CV route + a
    // step sequencer with a 16-step pattern) and assert it parses. This is
    // the TS-side invariant test (RESEARCH.md:578).
    const pattern: SequencerPattern = {
      gate: Array(16).fill(false) as SequencerPattern["gate"],
      cv: Array(16).fill(0) as SequencerPattern["cv"],
    };
    pattern.gate[0] = true;
    pattern.gate[4] = true;
    pattern.gate[8] = true;
    pattern.gate[12] = true;
    pattern.cv[0] = 0.5;
    pattern.cv[4] = -0.25;

    const sequencerNode: Node = {
      id: "node-seq",
      nodeTypeId: "step_sequencer",
      ports: [
        { id: "port-gate-out", name: "Gate Out", direction: "output", signalType: "control" },
        { id: "port-cv-out", name: "CV Out", direction: "output", signalType: "control" },
      ],
      parameters: [],
      runtimeTarget: "step_sequencer",
      sceneMembership: [],
      ownership: { controller: "shared", isLocked: false },
      enabled: true,
      sequencerPattern: pattern,
    };

    expect(() => __testNodeSchema.parse(sequencerNode)).not.toThrow();
    expect(() => __testSequencerPatternSchema.parse(pattern)).not.toThrow();

    // A malformed pattern (wrong length) must be rejected at the boundary.
    const badPattern = { gate: [false, false], cv: [0] };
    expect(() => __testSequencerPatternSchema.parse(badPattern)).toThrow();
  });
});

function buildPreviewEffectNode(sceneId: string | undefined, busId: string | undefined): Node {
  return {
    id: "preview-test-effect",
    nodeTypeId: "delay",
    ports: [
      { id: "preview-test-effect-in", name: "signal_in", direction: "input", signalType: "audio" },
      { id: "preview-test-effect-out", name: "signal_out", direction: "output", signalType: "audio" },
    ],
    parameters: [
      {
        id: "preview-test-effect-mix",
        name: "mix",
        value: 0.4,
        defaultValue: 0.4,
        minValue: 0,
        maxValue: 1,
        unit: "ratio",
      },
    ],
    runtimeTarget: "delay",
    sceneMembership: sceneId ? [sceneId] : [],
    ownership: { controller: "shared", isLocked: false },
    enabled: true,
    busTargetId: busId ?? null,
    bypassed: false,
  };
}
