import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";

import type { SessionDocument } from "../generated/session-types";

const transportStateSchema = z.object({
  tempoBpm: z.number(),
  isPlaying: z.boolean(),
  positionBeats: z.number(),
});

const parameterValueSchema = z.object({
  id: z.string(),
  name: z.string(),
  value: z.number(),
  unit: z.string(),
});

const portSchema = z.object({
  id: z.string(),
  name: z.string(),
  direction: z.enum(["input", "output"]),
  signalType: z.enum(["audio", "control"]),
});

const ownershipAssignmentSchema = z.object({
  controller: z.enum(["user", "agent", "shared"]),
  isLocked: z.boolean(),
});

const nodeSchema = z.object({
  id: z.string(),
  nodeType: z.enum(["source", "mixer", "output"]),
  ports: z.array(portSchema),
  parameters: z.array(parameterValueSchema),
  runtimeTarget: z.string().nullable(),
  sceneMembership: z.array(z.string()),
  ownership: ownershipAssignmentSchema,
});

const routeSchema = z.object({
  id: z.string(),
  sourceNodeId: z.string(),
  sourcePortId: z.string(),
  targetNodeId: z.string(),
  targetPortId: z.string(),
  busId: z.string().nullable(),
});

const busSchema = z.object({
  id: z.string(),
  name: z.string(),
  channels: z.number(),
});

const macroDefinitionSchema = z.object({
  id: z.string(),
  name: z.string(),
  targetParameterIds: z.array(z.string()),
  rangeStart: z.number(),
  rangeEnd: z.number(),
});

const macroOverrideSchema = z.object({
  macroId: z.string(),
  value: z.number(),
});

const sceneDefinitionSchema = z.object({
  id: z.string(),
  name: z.string(),
  activeNodeIds: z.array(z.string()),
  macroOverrides: z.array(macroOverrideSchema),
});

const parameterOverrideSchema = z.object({
  parameterId: z.string(),
  value: z.number(),
});

const variationDefinitionSchema = z.object({
  id: z.string(),
  name: z.string(),
  sceneId: z.string(),
  parameterOverrides: z.array(parameterOverrideSchema),
});

const ownershipRuleSchema = z.object({
  id: z.string(),
  scope: z.string(),
  controller: z.enum(["user", "agent", "shared"]),
  canOverride: z.boolean(),
});

const runtimeStatusSchema = z.object({
  id: z.string(),
  runtime: z.enum(["audio", "visual", "agent"]),
  status: z.enum(["disconnected", "connecting", "ready", "error"]),
  targetId: z.string().nullable(),
  lastError: z.string().nullable(),
});

const sessionDocumentSchema: z.ZodType<SessionDocument> = z.object({
  schemaVersion: z.number(),
  sessionId: z.string(),
  title: z.string(),
  createdAt: z.string(),
  updatedAt: z.string(),
  transport: transportStateSchema,
  nodes: z.array(nodeSchema),
  routes: z.array(routeSchema),
  buses: z.array(busSchema),
  macros: z.array(macroDefinitionSchema),
  scenes: z.array(sceneDefinitionSchema),
  variations: z.array(variationDefinitionSchema),
  ownershipRules: z.array(ownershipRuleSchema),
  runtimeStatus: z.array(runtimeStatusSchema),
});

async function invokeSession(command: string, args?: Record<string, unknown>) {
  const payload = await invoke(command, args);
  return sessionDocumentSchema.parse(payload);
}

export function parseSessionDocument(payload: unknown): SessionDocument {
  return sessionDocumentSchema.parse(payload);
}

export async function createDefaultSession(): Promise<SessionDocument> {
  return invokeSession("create_default_session");
}

export async function getCurrentSession(): Promise<SessionDocument> {
  return invokeSession("get_current_session");
}

export async function saveSessionToPath(path: string): Promise<void> {
  await invoke("save_session_to_path", { path });
}

export async function openSessionFromPath(path: string): Promise<SessionDocument> {
  return invokeSession("open_session_from_path", { path });
}
