import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { z } from "zod";

import type {
  ActionHistoryEntry,
  AgentIntent,
  AgentRuntimeState,
  BindingTarget,
  GraphEditCommand,
  HardwareBinding,
  HardwareRuntimeSettings,
  HardwareRuntimeStatus,
  MacroCommand,
  MidiInputPort,
  NodeCatalogEntry,
  PerformanceCommand,
  PendingAction,
  SequencerPattern,
  SessionDocument,
  TypedCommand,
} from "../generated/session-types";
import { invokeBrowserPreview } from "./browser-preview-session";

type TauriWindow = Window & {
  __TAURI_INTERNALS__?: {
    invoke?: unknown;
  };
};

async function invokeCommand(command: string, args?: Record<string, unknown>) {
  if (hasTauriInvokeBridge()) {
    return tauriInvoke(command, args);
  }

  return invokeBrowserPreview(command, args);
}

function hasTauriInvokeBridge(): boolean {
  return (
    typeof window !== "undefined" &&
    typeof (window as TauriWindow).__TAURI_INTERNALS__?.invoke === "function"
  );
}
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

// D-07/D-08: fixed 16-step mono gate+cv pattern. The tuple length is enforced
// at runtime (`.length(16)`) so a malformed payload is rejected at the
// boundary (Pitfall #5 guard). Zod v4 does not refine `.length(N)` to a
// fixed-length tuple at the type level, so the schema is cast through
// `unknown` to the generated 16-tuple `SequencerPattern` shape.
const sequencerPatternSchema = z.object({
  gate: z.array(z.boolean()).length(16),
  cv: z.array(z.number()).length(16),
}) as unknown as z.ZodType<SequencerPattern>;

// Pitfall #5 fix: the v1 closed-enum `nodeType` + `audioPrimitive` union is
// gone. Node identity is the catalog `nodeTypeId` (validated as a free string
// — the catalog is the authority, not the Zod schema). Per-node config flows
// as flat optional fields; the catalog is the single source of truth.
const nodeSchema = z.object({
  id: z.string(),
  nodeTypeId: z.string(),
  ports: z.array(portSchema),
  parameters: z.array(parameterValueSchema),
  runtimeTarget: z.string().nullable(),
  sceneMembership: z.array(z.string()),
  ownership: ownershipAssignmentSchema,
  enabled: z.boolean(),
  busTargetId: z.string().nullable().optional(),
  outputKind: z.enum(["master", "cue"]).nullable().optional(),
  channelCount: z.number().nullable().optional(),
  bypassed: z.boolean().nullable().optional(),
  channelMode: z.enum(["mono", "stereo"]).nullable().optional(),
  sequencerPattern: sequencerPatternSchema.nullable().optional(),
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
  lifecycle: z.enum(["idle", "starting", "ready", "rendering", "panicked", "failed"]),
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
  z.object({
    type: z.literal("setStepValue"),
    payload: z.object({
      node_id: z.string(),
      step_index: z.number(),
      gate: z.boolean().nullable(),
      cv: z.number().nullable(),
    }),
  }),
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

const midiInputPortSchema: z.ZodType<MidiInputPort> = z.object({
  id: z.string(),
  displayName: z.string(),
  isSelected: z.boolean(),
});

const hardwareRuntimeSettingsSchema: z.ZodType<HardwareRuntimeSettings> = z.object({
  midi: z.object({
    selectedInputId: z.string().nullable(),
    autoStart: z.boolean(),
  }),
  osc: z.object({
    bindHost: z.string(),
    listenPort: z.number(),
    autoStart: z.boolean(),
  }),
});

const hardwareListenerLifecycleSchema = z.enum(["unavailable", "stopped", "starting", "listening", "restarting", "error"]);
const hardwareLearnLifecycleSchema = z.enum(["idle", "learning", "captured"]);
const hardwareRuntimeDiagnosticCodeSchema = z.enum([
  "no_midi_ports",
  "invalid_midi_port_selection",
  "midi_enumeration_failed",
  "osc_bind_failed",
  "osc_port_in_use",
  "listener_restart_required",
  "listener_restarted",
  "listener_stopped",
  "listener_start_pending",
  "route_apply_failed",
]);

const hardwareRuntimeDiagnosticSchema = z.object({
  code: hardwareRuntimeDiagnosticCodeSchema,
  message: z.string(),
  recoverable: z.boolean(),
  detail: z.string().nullable(),
});

const hardwareRuntimeStatusSchema: z.ZodType<HardwareRuntimeStatus> = z.object({
  midi: z.object({
    lifecycle: hardwareListenerLifecycleSchema,
    selectedInputId: z.string().nullable(),
    selectedDisplayName: z.string().nullable(),
    availableInputCount: z.number().nullable(),
    lastError: z.string().nullable(),
  }),
  osc: z.object({
    lifecycle: hardwareListenerLifecycleSchema,
    bindHost: z.string(),
    listenPort: z.number(),
    lastError: z.string().nullable(),
  }),
  learn: z.object({
    lifecycle: hardwareLearnLifecycleSchema,
    target: bindingTargetSchema.nullable(),
    source: hardwareSourceSchema.nullable(),
  }),
  diagnostics: z.array(hardwareRuntimeDiagnosticSchema),
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

// Catalog — the compiled-in single source of truth (NODES-01 #4). Validated
// at the boundary so a malformed catalog payload never reaches the palette.
const catalogPortSpecSchema = z.object({
  id: z.string(),
  name: z.string(),
  direction: z.enum(["input", "output"]),
  signalType: z.enum(["audio", "control"]),
});

const catalogParamSpecSchema = z.object({
  id: z.string(),
  scArg: z.string(),
  aliases: z.array(z.string()),
  defaultValue: z.number(),
  minValue: z.number(),
  maxValue: z.number(),
  unit: z.string(),
  exposesCvPort: z.boolean(),
  cvPortId: z.string().nullable(),
});

const nodeCatalogEntrySchema: z.ZodType<NodeCatalogEntry> = z.object({
  id: z.string(),
  displayName: z.string(),
  category: z.enum(["source", "modulator", "effect", "utility", "sequencer", "mixer", "output"]),
  synthdefName: z.string(),
  synthdefResource: z.string(),
  ports: z.array(catalogPortSpecSchema),
  parameters: z.array(catalogParamSpecSchema),
  visualShape: z.string(),
});

async function invokeSession(command: string, args?: Record<string, unknown>) {
  const payload = await invokeCommand(command, args);
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
  await invokeCommand("save_session_to_path", { path });
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
  const payload = await invokeCommand("send_agent_message", { message });
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
  const payload = await invokeCommand("get_agent_runtime_state");
  return agentRuntimeStateResponseSchema.parse(payload);
}

export async function startHardwareLearn(target: BindingTarget): Promise<HardwareRuntimeStatus> {
  bindingTargetSchema.parse(target);
  const payload = await invokeCommand("start_hardware_learn", { target });
  return hardwareRuntimeStatusSchema.parse(payload);
}

export async function stopHardwareLearn(): Promise<void> {
  await invokeCommand("stop_hardware_learn");
}

export async function pollHardwareEvents(): Promise<SessionDocument> {
  return invokeSession("poll_hardware_events");
}

export async function removeHardwareBinding(bindingId: string): Promise<SessionDocument> {
  return invokeSession("remove_hardware_binding", { bindingId });
}

export async function listMidiInputPorts(): Promise<MidiInputPort[]> {
  const payload = await invokeCommand("list_midi_input_ports");
  return z.array(midiInputPortSchema).parse(payload);
}

export async function getHardwareRuntimeSettings(): Promise<HardwareRuntimeSettings> {
  const payload = await invokeCommand("get_hardware_runtime_settings");
  return hardwareRuntimeSettingsSchema.parse(payload);
}

export async function updateHardwareRuntimeSettings(settings: HardwareRuntimeSettings): Promise<HardwareRuntimeStatus> {
  hardwareRuntimeSettingsSchema.parse(settings);
  const payload = await invokeCommand("update_hardware_runtime_settings", { settings });
  return hardwareRuntimeStatusSchema.parse(payload);
}

export async function getHardwareRuntimeStatus(): Promise<HardwareRuntimeStatus> {
  const payload = await invokeCommand("get_hardware_runtime_status");
  return hardwareRuntimeStatusSchema.parse(payload);
}

export async function startHardwareListeners(): Promise<HardwareRuntimeStatus> {
  const payload = await invokeCommand("start_hardware_listeners");
  return hardwareRuntimeStatusSchema.parse(payload);
}

export async function stopHardwareListeners(): Promise<HardwareRuntimeStatus> {
  const payload = await invokeCommand("stop_hardware_listeners");
  return hardwareRuntimeStatusSchema.parse(payload);
}

export async function drainHardwareEvents(maxEvents?: number): Promise<SessionDocument> {
  return invokeSession("drain_hardware_events", { maxEvents: maxEvents ?? null });
}

/**
 * Fetch the compiled-in node catalog (NODES-01 success criterion #4). The
 * palette/inspector read this single source of truth instead of a hand-
 * maintained mirror. Falls through to the browser-preview catalog when no
 * Tauri bridge is present (dev/preview mode).
 */
export async function getNodeCatalog(): Promise<NodeCatalogEntry[]> {
  const payload = await invokeCommand("get_node_catalog");
  return z.array(nodeCatalogEntrySchema).parse(payload);
}

// Test-only schema exports (used by session-client.test.ts round-trip tests).
export const __testNodeCatalogEntrySchema = nodeCatalogEntrySchema;
export const __testNodeSchema = nodeSchema;
export const __testSequencerPatternSchema = sequencerPatternSchema;
