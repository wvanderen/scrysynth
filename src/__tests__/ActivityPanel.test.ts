import { describe, expect, it } from "vitest";

import type { ActionHistoryEntry, SessionDocument, TypedCommand } from "../generated/session-types";

function createSession(overrides: Partial<SessionDocument> = {}): SessionDocument {
  return {
    schemaVersion: 1,
    sessionId: "session-1",
    title: "Activity Test Session",
    createdAt: "2026-04-12T00:00:00Z",
    updatedAt: "2026-04-12T00:00:00Z",
    transport: { tempoBpm: 120, isPlaying: false, positionBeats: 0 },
    audioRuntime: {
      lifecycle: "idle", health: "unknown", sampleRateHz: null, blockSize: null,
      activePatchId: null, lastError: null, panicRecoveryCount: 0,
    },
    visualRuntime: {
      lifecycle: "idle", health: "unknown", activeSceneId: null, fps: null,
      lastError: null, renderer: null,
    },
    agentRuntime: {
      isAvailable: true, pendingActionCount: 0, isFrozen: false,
    },
    nodes: [],
    routes: [],
    buses: [],
    macros: [],
    scenes: [],
    variations: [],
    ownershipRules: [],
    runtimeStatus: [],
    agentFrozen: false,
    pendingActions: [],
    actionHistory: [],
    hardwareBindings: [],
    ...overrides,
  };
}

function createActionHistoryEntry(
  actorId: string,
  cmd: TypedCommand,
  description: string,
): ActionHistoryEntry {
  return {
    id: `entry-${Math.random().toString(36).slice(2, 8)}`,
    timestamp: "2026-04-12T01:00:00Z",
    actor: { actorId, correlationId: "corr-1" },
    command: cmd,
    diff: {
      description,
      affectedNodeIds: ["node-1"],
      beforeSnippet: "",
      afterSnippet: "changed",
    },
  };
}

describe("activity panel data structure", () => {
  it("action history entries have correct actor identification", () => {
    const cmd: TypedCommand = {
      type: "graphEdit",
      payload: { type: "addNode", payload: { node: { id: "node-1", nodeType: "source", ports: [], parameters: [], runtimeTarget: null, sceneMembership: [], ownership: { controller: "user", isLocked: false }, enabled: true, audioPrimitive: null } } },
    };

    const userEntry = createActionHistoryEntry("user", cmd, "Added source node");
    const agentEntry = createActionHistoryEntry("agent", cmd, "Added source node");

    expect(userEntry.actor.actorId).toBe("user");
    expect(agentEntry.actor.actorId).toBe("agent");
  });

  it("session with mixed action history can be filtered by actor", () => {
    const cmd: TypedCommand = {
      type: "graphEdit",
      payload: { type: "addNode", payload: { node: { id: "node-1", nodeType: "source", ports: [], parameters: [], runtimeTarget: null, sceneMembership: [], ownership: { controller: "user", isLocked: false }, enabled: true, audioPrimitive: null } } },
    };

    const history = [
      createActionHistoryEntry("user", cmd, "User added node"),
      createActionHistoryEntry("agent", cmd, "Agent added node"),
      createActionHistoryEntry("user", cmd, "User removed node"),
    ];

    const session = createSession({ actionHistory: history });

    const userActions = session.actionHistory.filter((e) => e.actor.actorId === "user");
    const agentActions = session.actionHistory.filter((e) => e.actor.actorId === "agent");

    expect(userActions).toHaveLength(2);
    expect(agentActions).toHaveLength(1);
  });

  it("diff summaries contain expected fields", () => {
    const cmd: TypedCommand = {
      type: "performance",
      payload: { type: "recallScene", payload: { scene_id: "scene-1" } },
    };

    const entry = createActionHistoryEntry("agent", cmd, "Recalled scene Main");

    expect(entry.diff.description).toBe("Recalled scene Main");
    expect(entry.diff.affectedNodeIds).toEqual(["node-1"]);
    expect(entry.command.type).toBe("performance");
  });
});
