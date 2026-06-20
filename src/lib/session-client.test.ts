import { beforeEach, describe, expect, it } from "vitest";

import type { Node } from "../generated/session-types";
import {
  applyGraphEdit,
  approvePendingAction,
  createDefaultSession,
  getCurrentSession,
  sendAgentMessage,
  startAudioRuntime,
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
});

function buildPreviewEffectNode(sceneId: string | undefined, busId: string | undefined): Node {
  return {
    id: "preview-test-effect",
    nodeType: "effect",
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
    runtimeTarget: "audio/effect/preview-test-effect",
    sceneMembership: sceneId ? [sceneId] : [],
    ownership: { controller: "shared", isLocked: false },
    enabled: true,
    audioPrimitive: {
      kind: "effect",
      config: { effectType: "delay", bypassed: false, busTargetId: busId ?? null },
    },
  };
}
