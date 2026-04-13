import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";

import type {
  ActionHistoryEntry,
  AgentIntent,
  AgentRuntimeState,
  BindingTarget,
  GraphEditCommand,
  HardwareBinding,
  MacroCommand,
  PerformanceCommand,
  PendingAction,
  SessionDocument,
  TypedCommand,
} from "../generated/session-types";
const transportStateSchema = z.object({
  tempoBpm: z.number(),
  isPlaying: z.boolean(),
  positionBeats: z.number(),
});

const parameterValueSchema = z.object({
  id: z.string(),
  name: z.string(),
  value: z.number(),
  defaultValue: z.number(),
  minValue: z.number(),
  maxValue: z.number(),
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
  nodeType: z.enum(["source", "effect", "mixer", "output"]),
  ports: z.array(portSchema),
  parameters: z.array(parameterValueSchema),
  runtimeTarget: z.string().nullable(),
  sceneMembership: z.array(z.string()),
  ownership: ownershipAssignmentSchema,
  enabled: z.boolean(),
  audioPrimitive: z
    .discriminatedUnion("kind", [
      z.object({
        kind: z.literal("source"),
        config: z.object({
          sourceType: z.enum(["oscillator", "noise"]),
          channelMode: z.enum(["mono", "stereo"]),
          busTargetId: z.string().nullable(),
        }),
      }),
      z.object({
        kind: z.literal("effect"),
        config: z.object({
          effectType: z.enum(["low_pass_filter", "delay"]),
          bypassed: z.boolean(),
          busTargetId: z.string().nullable(),
        }),
      }),
      z.object({
        kind: z.literal("mixer"),
        config: z.object({
          channelMode: z.enum(["mono", "stereo"]),
          busTargetId: z.string().nullable(),
        }),
      }),
      z.object({
        kind: z.literal("output"),
        config: z.object({
          outputType: z.enum(["master", "cue"]),
          channels: z.number(),
          busTargetId: z.string().nullable(),
        }),
      }),
    ])
    .nullable(),
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
  busType: z.enum(["auxiliary", "main", "cue"]),
  isEnabled: z.boolean(),
});

const macroTargetSchema = z.discriminatedUnion("kind", [
  z.object({
    kind: z.literal("audioParameter"),
    config: z.object({
      node_id: z.string(),
      parameter_id: z.string(),
    }),
  }),
  z.object({
    kind: z.literal("visualParameter"),
    config: z.object({
      element_id: z.string(),
      parameter_id: z.string(),
    }),
  }),
]);

const macroDefinitionSchema = z.object({
  id: z.string(),
  name: z.string(),
  targetParameterIds: z.array(z.string()).optional().default([]),
  rangeStart: z.number(),
  rangeEnd: z.number(),
  targets: z.array(macroTargetSchema).default([]),
});

const macroCommandSchema: z.ZodType<MacroCommand> = z.discriminatedUnion("type", [
  z.object({ type: z.literal("createMacro"), payload: z.object({ definition: macroDefinitionSchema }) }),
  z.object({
    type: z.literal("updateMacro"),
    payload: z.object({
      macro_id: z.string(),
      name: z.string().nullable(),
      targets: z.array(macroTargetSchema).nullable(),
      range_start: z.number().nullable(),
      range_end: z.number().nullable(),
    }),
  }),
  z.object({ type: z.literal("removeMacro"), payload: z.object({ macro_id: z.string() }) }),
  z.object({ type: z.literal("setMacroValue"), payload: z.object({ macro_id: z.string(), value: z.number() }) }),
]);

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

const audioRuntimeSchema = z.object({
  lifecycle: z.enum(["idle", "booting", "ready", "running", "recovering", "failed"]),
  health: z.enum(["unknown", "healthy", "degraded", "panic_recovered", "error"]),
  sampleRateHz: z.number().nullable(),
  blockSize: z.number().nullable(),
  activePatchId: z.string().nullable(),
  lastError: z.string().nullable(),
  panicRecoveryCount: z.number(),
});

const visualRuntimeStateSchema = z.object({
  lifecycle: z.enum(["idle", "starting", "ready", "rendering", "failed"]),
  health: z.enum(["unknown", "healthy", "degraded", "error"]),
  activeSceneId: z.string().nullable(),
  fps: z.number().nullable(),
  lastError: z.string().nullable(),
  renderer: z.string().nullable(),
});

const agentRuntimeStateSchema = z.object({
  isAvailable: z.boolean(),
  pendingActionCount: z.number(),
  isFrozen: z.boolean(),
});

const graphEditCommandSchema: z.ZodType<GraphEditCommand> = z.discriminatedUnion("type", [
  z.object({ type: z.literal("addNode"), payload: z.object({ node: nodeSchema }) }),
  z.object({ type: z.literal("removeNode"), payload: z.object({ node_id: z.string() }) }),
  z.object({ type: z.literal("setNodeEnabled"), payload: z.object({ node_id: z.string(), enabled: z.boolean() }) }),
  z.object({ type: z.literal("setParameterValue"), payload: z.object({ node_id: z.string(), parameter_id: z.string(), value: z.number() }) }),
  z.object({ type: z.literal("addRoute"), payload: z.object({ route: routeSchema }) }),
  z.object({ type: z.literal("removeRoute"), payload: z.object({ route_id: z.string() }) }),
  z.object({ type: z.literal("assignNodeToBus"), payload: z.object({ node_id: z.string(), bus_id: z.string() }) }),
  z.object({ type: z.literal("clearNodeBusAssignment"), payload: z.object({ node_id: z.string() }) }),
]);

const performanceCommandSchema: z.ZodType<PerformanceCommand> = z.discriminatedUnion("type", [
  z.object({ type: z.literal("recallScene"), payload: z.object({ scene_id: z.string() }) }),
  z.object({ type: z.literal("saveVariation"), payload: z.object({ name: z.string(), scene_id: z.string() }) }),
  z.object({ type: z.literal("restoreVariation"), payload: z.object({ variation_id: z.string() }) }),
]);

const typedCommandSchema: z.ZodType<TypedCommand> = z.discriminatedUnion("type", [
  z.object({ type: z.literal("graphEdit"), payload: graphEditCommandSchema }),
  z.object({ type: z.literal("performance"), payload: performanceCommandSchema }),
]);

const actorRefSchema = z.object({
  actorId: z.string(),
  correlationId: z.string(),
});

const diffSummarySchema = z.object({
  description: z.string(),
  affectedNodeIds: z.array(z.string()),
  beforeSnippet: z.string(),
  afterSnippet: z.string(),
});

const pendingActionStatusSchema = z.enum(["pending", "approved", "rejected"]);

const pendingActionSchema: z.ZodType<PendingAction> = z.object({
  id: z.string(),
  correlationId: z.string(),
  command: typedCommandSchema,
  riskTier: z.enum(["low", "medium", "high"]),
  createdAt: z.string(),
  status: pendingActionStatusSchema,
});

const actionHistoryEntrySchema: z.ZodType<ActionHistoryEntry> = z.object({
  id: z.string(),
  timestamp: z.string(),
  actor: actorRefSchema,
  command: typedCommandSchema,
  diff: diffSummarySchema,
});

const agentIntentSchema: z.ZodType<AgentIntent> = z.object({
  rawInput: z.string(),
  parsedCommands: z.array(typedCommandSchema),
  confidence: z.number(),
});

const hardwareSourceSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("midiCc"), config: z.object({ channel: z.number(), controller: z.number() }) }),
  z.object({ kind: z.literal("midiNote"), config: z.object({ channel: z.number(), note: z.number() }) }),
  z.object({ kind: z.literal("midiPitchBend"), config: z.object({ channel: z.number() }) }),
  z.object({ kind: z.literal("oscAddress"), config: z.object({ address: z.string() }) }),
]);

const bindingTargetSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("macro"), config: z.object({ macro_id: z.string() }) }),
  z.object({ kind: z.literal("sceneRecall"), config: z.object({ scene_id: z.string() }) }),
  z.object({ kind: z.literal("transportPlay") }),
  z.object({ kind: z.literal("transportStop") }),
  z.object({ kind: z.literal("transportPanic") }),
]);

const valueTransformSchema = z.object({
  inputMin: z.number(),
  inputMax: z.number(),
  outputMin: z.number(),
  outputMax: z.number(),
});

const hardwareBindingSchema: z.ZodType<HardwareBinding> = z.object({
  id: z.string(),
  source: hardwareSourceSchema,
  target: bindingTargetSchema,
  transform: valueTransformSchema,
});

const sessionDocumentSchema: z.ZodType<SessionDocument> = z.object({
  schemaVersion: z.number(),
  sessionId: z.string(),
  title: z.string(),
  createdAt: z.string(),
  updatedAt: z.string(),
  transport: transportStateSchema,
  audioRuntime: audioRuntimeSchema,
  visualRuntime: visualRuntimeStateSchema,
  agentRuntime: agentRuntimeStateSchema,
  nodes: z.array(nodeSchema),
  routes: z.array(routeSchema),
  buses: z.array(busSchema),
  macros: z.array(macroDefinitionSchema),
  scenes: z.array(sceneDefinitionSchema),
  variations: z.array(variationDefinitionSchema),
  ownershipRules: z.array(ownershipRuleSchema),
  runtimeStatus: z.array(runtimeStatusSchema),
  agentFrozen: z.boolean(),
  pendingActions: z.array(pendingActionSchema),
  actionHistory: z.array(actionHistoryEntrySchema),
  hardwareBindings: z.array(hardwareBindingSchema).default([]),
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

export async function applyGraphEdit(command: GraphEditCommand): Promise<SessionDocument> {
  return invokeSession("apply_graph_edit", { command });
}

export async function startAudioRuntime(): Promise<SessionDocument> {
  return invokeSession("start_audio_runtime");
}

export async function stopAudioRuntime(): Promise<SessionDocument> {
  return invokeSession("stop_audio_runtime");
}

export async function panicAudioRuntime(): Promise<SessionDocument> {
  return invokeSession("panic_audio_runtime");
}

export async function applyPerformanceCommand(command: PerformanceCommand): Promise<SessionDocument> {
  return invokeSession("apply_performance_command", { command });
}

export async function applyMacroCommand(command: MacroCommand): Promise<SessionDocument> {
  macroCommandSchema.parse(command);
  return invokeSession("apply_macro_command", { command });
}

const agentMessageResponseSchema = z.object({
  session: sessionDocumentSchema,
  intent: agentIntentSchema,
});

export async function sendAgentMessage(message: string): Promise<{ session: SessionDocument; intent: AgentIntent }> {
  const payload = await invoke("send_agent_message", { message });
  return agentMessageResponseSchema.parse(payload);
}

export async function toggleAgentFreeze(): Promise<SessionDocument> {
  return invokeSession("toggle_agent_freeze");
}

export async function reclaimOwnership(nodeIds?: string[], targetController?: string): Promise<SessionDocument> {
  return invokeSession("reclaim_ownership", {
    nodeIds: nodeIds ?? null,
    targetController: targetController ?? null,
  });
}

export async function approvePendingAction(actionId: string): Promise<SessionDocument> {
  return invokeSession("approve_pending_action", { actionId });
}

export async function rejectPendingAction(actionId: string): Promise<SessionDocument> {
  return invokeSession("reject_pending_action", { actionId });
}

export async function startVisualRuntime(): Promise<SessionDocument> {
  return invokeSession("start_visual_runtime");
}

export async function stopVisualRuntime(): Promise<SessionDocument> {
  return invokeSession("stop_visual_runtime");
}

export async function panicVisualRuntime(): Promise<SessionDocument> {
  return invokeSession("panic_visual_runtime");
}

const agentRuntimeStateResponseSchema: z.ZodType<AgentRuntimeState> = z.object({
  isAvailable: z.boolean(),
  pendingActionCount: z.number(),
  isFrozen: z.boolean(),
});

export async function getAgentRuntimeState(): Promise<AgentRuntimeState> {
  const payload = await invoke("get_agent_runtime_state");
  return agentRuntimeStateResponseSchema.parse(payload);
}

export async function startHardwareLearn(target: BindingTarget): Promise<void> {
  bindingTargetSchema.parse(target);
  await invoke("start_hardware_learn", { target });
}

export async function stopHardwareLearn(): Promise<void> {
  await invoke("stop_hardware_learn");
}

export async function pollHardwareEvents(): Promise<SessionDocument> {
  return invokeSession("poll_hardware_events");
}

export async function removeHardwareBinding(bindingId: string): Promise<SessionDocument> {
  return invokeSession("remove_hardware_binding", { bindingId });
}
