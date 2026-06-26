import type {
  ActionHistoryEntry,
  AgentIntent,
  BindingTarget,
  ControllerKind,
  GraphEditCommand,
  HardwareRuntimeSettings,
  HardwareRuntimeStatus,
  MacroCommand,
  MidiInputPort,
  Node,
  NodeCatalogEntry,
  PendingAction,
  PerformanceCommand,
  SessionDocument,
  TypedCommand,
} from "../generated/session-types";

type PreviewArgs = Record<string, unknown> | undefined;

let previewSession = buildPreviewSession();
let previewHardwareSettings: HardwareRuntimeSettings = {
  midi: { selectedInputId: null, autoStart: false },
  osc: { bindHost: "127.0.0.1", listenPort: 9001, autoStart: false },
};
let previewHardwareStatus: HardwareRuntimeStatus = buildHardwareStatus();
let idCounter = 0;

export async function invokeBrowserPreview(command: string, args?: PreviewArgs): Promise<unknown> {
  switch (command) {
    case "create_default_session":
      previewSession = buildPreviewSession();
      return cloneSession();
    case "get_current_session":
      return cloneSession();
    case "save_session_to_path":
      return undefined;
    case "open_session_from_path":
      previewSession = buildPreviewSession("Preview Loaded Session");
      return cloneSession();
    case "apply_graph_edit":
      applyGraphEdit(readArg<GraphEditCommand>(args, "command"));
      return cloneSession();
    case "start_audio_runtime":
      previewSession.audioRuntime = {
        lifecycle: "running",
        health: "healthy",
        sampleRateHz: 48000,
        blockSize: 64,
        activePatchId: "preview-patch",
        lastError: null,
        panicRecoveryCount: previewSession.audioRuntime.panicRecoveryCount,
      };
      setRuntimeStatus("audio", "ready", null);
      return cloneSession();
    case "stop_audio_runtime":
      previewSession.audioRuntime = {
        ...previewSession.audioRuntime,
        lifecycle: "idle",
        health: "healthy",
        activePatchId: null,
        lastError: null,
      };
      setRuntimeStatus("audio", "disconnected", null);
      return cloneSession();
    case "panic_audio_runtime":
      previewSession.audioRuntime = {
        ...previewSession.audioRuntime,
        lifecycle: "recovering",
        health: "panic_recovered",
        activePatchId: null,
        lastError: null,
        panicRecoveryCount: previewSession.audioRuntime.panicRecoveryCount + 1,
      };
      setRuntimeStatus("audio", "ready", null);
      return cloneSession();
    case "apply_performance_command":
      applyPerformanceCommand(readArg<PerformanceCommand>(args, "command"));
      return cloneSession();
    case "apply_macro_command":
      applyMacroCommand(readArg<MacroCommand>(args, "command"));
      return cloneSession();
    case "send_agent_message":
      return sendAgentMessage(readArg<string>(args, "message"));
    case "toggle_agent_freeze":
      previewSession.agentFrozen = !previewSession.agentFrozen;
      return cloneSession();
    case "reclaim_ownership":
      reclaimOwnership(
        readOptionalArg<string[]>(args, "nodeIds"),
        readOptionalArg<ControllerKind>(args, "targetController"),
      );
      return cloneSession();
    case "approve_pending_action":
      approvePendingAction(readArg<string>(args, "actionId"));
      return cloneSession();
    case "reject_pending_action":
      rejectPendingAction(readArg<string>(args, "actionId"));
      return cloneSession();
    case "start_visual_runtime":
      previewSession.visualRuntime = {
        lifecycle: "rendering",
        health: "healthy",
        activeSceneId: previewSession.scenes[0]?.id ?? null,
        fps: 60,
        lastError: null,
        renderer: "browser preview",
      };
      setRuntimeStatus("visual", "ready", null);
      return cloneSession();
    case "stop_visual_runtime":
      previewSession.visualRuntime = {
        lifecycle: "idle",
        health: "unknown",
        activeSceneId: null,
        fps: null,
        lastError: null,
        renderer: null,
      };
      setRuntimeStatus("visual", "disconnected", null);
      return cloneSession();
    case "panic_visual_runtime":
      previewSession.visualRuntime = {
        ...previewSession.visualRuntime,
        lifecycle: "panicked",
        health: "degraded",
        fps: null,
        lastError: null,
      };
      setRuntimeStatus("visual", "error", null);
      return cloneSession();
    case "get_agent_runtime_state":
      syncDerivedState();
      return clone(previewSession.agentRuntime);
    case "start_hardware_learn":
      previewHardwareStatus = {
        ...previewHardwareStatus,
        learn: {
          lifecycle: "learning",
          target: readArg<BindingTarget>(args, "target"),
          source: null,
        },
      };
      return clone(previewHardwareStatus);
    case "stop_hardware_learn":
      previewHardwareStatus = {
        ...previewHardwareStatus,
        learn: { lifecycle: "idle", target: null, source: null },
      };
      return undefined;
    case "poll_hardware_events":
    case "drain_hardware_events":
      return cloneSession();
    case "remove_hardware_binding":
      previewSession.hardwareBindings = previewSession.hardwareBindings.filter(
        (binding) => binding.id !== readArg<string>(args, "bindingId"),
      );
      return cloneSession();
    case "list_midi_input_ports":
      return [] satisfies MidiInputPort[];
    case "get_hardware_runtime_settings":
      return clone(previewHardwareSettings);
    case "update_hardware_runtime_settings":
      previewHardwareSettings = readArg<HardwareRuntimeSettings>(args, "settings");
      previewHardwareStatus = {
        ...previewHardwareStatus,
        osc: {
          ...previewHardwareStatus.osc,
          bindHost: previewHardwareSettings.osc.bindHost,
          listenPort: previewHardwareSettings.osc.listenPort,
        },
      };
      return clone(previewHardwareStatus);
    case "get_hardware_runtime_status":
      return clone(previewHardwareStatus);
    case "start_hardware_listeners":
      previewHardwareStatus = {
        ...previewHardwareStatus,
        midi: {
          ...previewHardwareStatus.midi,
          lifecycle: "listening",
          availableInputCount: 0,
        },
        osc: {
          ...previewHardwareStatus.osc,
          lifecycle: "listening",
          bindHost: previewHardwareSettings.osc.bindHost,
          listenPort: previewHardwareSettings.osc.listenPort,
        },
      };
      return clone(previewHardwareStatus);
    case "stop_hardware_listeners":
      previewHardwareStatus = {
        ...previewHardwareStatus,
        midi: { ...previewHardwareStatus.midi, lifecycle: "stopped" },
        osc: { ...previewHardwareStatus.osc, lifecycle: "stopped" },
        learn: { lifecycle: "idle", target: null, source: null },
      };
      return clone(previewHardwareStatus);
    case "get_node_catalog":
      return clone(PREVIEW_CATALOG);
    default:
      throw new Error(`Preview mode does not implement Tauri command '${command}'.`);
  }
}

function buildPreviewSession(title = "Default Scrysynth Session"): SessionDocument {
  const sceneId = "preview-scene-intro";
  const busId = "preview-bus-main";
  const sourceId = "preview-source";
  const effectId = "preview-delay";
  const outputId = "preview-output";
  const sourceLevelId = "preview-source-level";
  const effectMixId = "preview-delay-mix";
  const macroId = "preview-macro-energy";

  return {
    schemaVersion: 2,
    sessionId: "preview-session",
    title,
    createdAt: "2026-04-11T00:00:00Z",
    updatedAt: "2026-04-11T00:00:00Z",
    transport: { tempoBpm: 120, isPlaying: false, positionBeats: 0 },
    audioRuntime: {
      lifecycle: "idle",
      health: "unknown",
      sampleRateHz: null,
      blockSize: null,
      activePatchId: null,
      lastError: null,
      panicRecoveryCount: 0,
    },
    nodes: [
      {
        id: sourceId,
        nodeTypeId: "oscillator",
        ports: [
          { id: `${sourceId}-main_out`, name: "Main Out", direction: "output", signalType: "audio" },
          {
            id: `${sourceId}-frequency_cv`,
            name: "Frequency FM",
            direction: "input",
            signalType: "audio",
          },
          { id: `${sourceId}-level_cv`, name: "Level CV", direction: "input", signalType: "control" },
        ],
        parameters: [
          {
            id: sourceLevelId,
            name: "level",
            value: 0.8,
            defaultValue: 0.8,
            minValue: 0,
            maxValue: 1,
            unit: "linear",
          },
        ],
        runtimeTarget: "oscillator",
        sceneMembership: [sceneId],
        ownership: { controller: "shared", isLocked: false },
        enabled: true,
        busTargetId: busId,
        channelMode: "mono",
      },
      {
        id: effectId,
        nodeTypeId: "delay",
        ports: [
          { id: `${effectId}-audio_in`, name: "Audio In", direction: "input", signalType: "audio" },
          { id: `${effectId}-audio_out`, name: "Audio Out", direction: "output", signalType: "audio" },
        ],
        parameters: [
          {
            id: effectMixId,
            name: "mix",
            value: 0.35,
            defaultValue: 0.35,
            minValue: 0,
            maxValue: 1,
            unit: "ratio",
          },
        ],
        runtimeTarget: "delay",
        sceneMembership: [sceneId],
        ownership: { controller: "shared", isLocked: false },
        enabled: true,
        busTargetId: busId,
        bypassed: false,
      },
      {
        id: outputId,
        nodeTypeId: "output",
        ports: [{ id: `${outputId}-audio_in`, name: "Audio In", direction: "input", signalType: "audio" }],
        parameters: [],
        runtimeTarget: "output",
        sceneMembership: [sceneId],
        ownership: { controller: "user", isLocked: false },
        enabled: true,
        busTargetId: busId,
        outputKind: "master",
        channelCount: 2,
      },
    ],
    routes: [
      {
        id: "preview-route-source-delay",
        sourceNodeId: sourceId,
        sourcePortId: `${sourceId}-main_out`,
        targetNodeId: effectId,
        targetPortId: `${effectId}-audio_in`,
        busId,
      },
      {
        id: "preview-route-delay-output",
        sourceNodeId: effectId,
        sourcePortId: `${effectId}-audio_out`,
        targetNodeId: outputId,
        targetPortId: `${outputId}-audio_in`,
        busId,
      },
    ],
    buses: [{ id: busId, name: "master_bus", channels: 2, busType: "main", isEnabled: true }],
    macros: [
      {
        id: macroId,
        name: "energy",
        targetParameterIds: [sourceLevelId, effectMixId],
        rangeStart: 0,
        rangeEnd: 1,
        targets: [],
      },
    ],
    scenes: [
      {
        id: sceneId,
        name: "intro",
        activeNodeIds: [sourceId, effectId, outputId],
        macroOverrides: [{ macroId, value: 0.65 }],
      },
    ],
    variations: [
      {
        id: "preview-variation-intro-alt",
        name: "intro-alt",
        sceneId,
        parameterOverrides: [
          { parameterId: sourceLevelId, value: 0.55 },
          { parameterId: effectMixId, value: 0.25 },
        ],
      },
    ],
    ownershipRules: [
      { id: "preview-rule-master", scope: "graph:master", controller: "shared", canOverride: true },
    ],
    runtimeStatus: [
      { id: "preview-runtime-audio", runtime: "audio", status: "disconnected", targetId: "audio-runtime", lastError: null },
      { id: "preview-runtime-visual", runtime: "visual", status: "disconnected", targetId: "visual-runtime", lastError: null },
      { id: "preview-runtime-agent", runtime: "agent", status: "ready", targetId: "agent-runtime", lastError: null },
    ],
    visualRuntime: {
      lifecycle: "idle",
      health: "unknown",
      activeSceneId: null,
      fps: null,
      lastError: null,
      renderer: null,
    },
    agentRuntime: { isAvailable: true, pendingActionCount: 0, isFrozen: false },
    agentFrozen: false,
    pendingActions: [],
    actionHistory: [],
    hardwareBindings: [],
  };
}

/**
 * Minimal catalog projection for browser-preview mode. The real app reads the
 * compiled-in Rust `CATALOG` via `get_node_catalog`; in preview/dev without a
 * Tauri bridge we ship a curated subset so the palette renders at least one
 * button per family. Sufficient for UI smoke tests, not authoritative.
 */
const PREVIEW_CATALOG: NodeCatalogEntry[] = [
  {
    id: "oscillator",
    displayName: "Oscillator",
    category: "source",
    synthdefName: "scrysynth_v2_oscillator",
    synthdefResource: "resources/synthdefs/v2/scrysynth_v2_oscillator.scsyndef",
    ports: [
      { id: "main_out", name: "Main Out", direction: "output", signalType: "audio" },
      { id: "frequency_cv", name: "Frequency FM", direction: "input", signalType: "audio" },
      { id: "level_cv", name: "Level CV", direction: "input", signalType: "control" },
    ],
    parameters: [
      { id: "frequency", scArg: "frequency", aliases: ["freq"], defaultValue: 220, minValue: 20, maxValue: 20000, unit: "hz", exposesCvPort: true, cvPortId: "frequency_cv" },
      { id: "wave_shape", scArg: "wave_shape", aliases: [], defaultValue: 0, minValue: 0, maxValue: 3, unit: "selector", exposesCvPort: false, cvPortId: null },
      { id: "level", scArg: "level", aliases: ["gain"], defaultValue: 1, minValue: 0, maxValue: 1, unit: "linear", exposesCvPort: true, cvPortId: "level_cv" },
    ],
    visualShape: "sphere",
  },
  {
    id: "filter",
    displayName: "Filter",
    category: "effect",
    synthdefName: "scrysynth_v2_filter",
    synthdefResource: "resources/synthdefs/v2/scrysynth_v2_filter.scsyndef",
    ports: [
      { id: "audio_in", name: "Audio In", direction: "input", signalType: "audio" },
      { id: "audio_out", name: "Audio Out", direction: "output", signalType: "audio" },
      { id: "cutoff_cv", name: "Cutoff CV", direction: "input", signalType: "control" },
    ],
    parameters: [
      { id: "cutoff", scArg: "cutoff_hz", aliases: ["cutoff"], defaultValue: 1200, minValue: 20, maxValue: 20000, unit: "hz", exposesCvPort: true, cvPortId: "cutoff_cv" },
    ],
    visualShape: "box",
  },
  {
    id: "step_sequencer",
    displayName: "Step Sequencer",
    category: "sequencer",
    synthdefName: "",
    synthdefResource: "",
    ports: [
      { id: "gate_out", name: "Gate Out", direction: "output", signalType: "control" },
      { id: "cv_out", name: "CV Out", direction: "output", signalType: "control" },
    ],
    parameters: [],
    visualShape: "ring",
  },
  {
    id: "output",
    displayName: "Output",
    category: "output",
    synthdefName: "scrysynth_v2_output",
    synthdefResource: "resources/synthdefs/v2/scrysynth_v2_output.scsyndef",
    ports: [
      { id: "audio_in", name: "Audio In", direction: "input", signalType: "audio" },
      { id: "level_cv", name: "Level CV", direction: "input", signalType: "control" },
    ],
    parameters: [
      { id: "level", scArg: "level", aliases: ["gain"], defaultValue: 1, minValue: 0, maxValue: 1, unit: "linear", exposesCvPort: true, cvPortId: "level_cv" },
    ],
    visualShape: "plane",
  },
];

function buildHardwareStatus(): HardwareRuntimeStatus {
  return {
    midi: {
      lifecycle: "stopped",
      selectedInputId: null,
      selectedDisplayName: null,
      availableInputCount: 0,
      lastError: null,
    },
    osc: {
      lifecycle: "stopped",
      bindHost: previewHardwareSettings.osc.bindHost,
      listenPort: previewHardwareSettings.osc.listenPort,
      lastError: null,
    },
    learn: { lifecycle: "idle", target: null, source: null },
    diagnostics: [],
  };
}

function applyGraphEdit(command: GraphEditCommand): void {
  switch (command.type) {
    case "addNode":
      if (!previewSession.nodes.some((node) => node.id === command.payload.node.id)) {
        previewSession.nodes.push(clone(command.payload.node));
        previewSession.nodes.sort((left, right) => left.id.localeCompare(right.id));
      }
      break;
    case "removeNode":
      previewSession.nodes = previewSession.nodes.filter((node) => node.id !== command.payload.node_id);
      previewSession.routes = previewSession.routes.filter(
        (route) => route.sourceNodeId !== command.payload.node_id && route.targetNodeId !== command.payload.node_id,
      );
      break;
    case "setNodeEnabled":
      updateNode(command.payload.node_id, (node) => {
        node.enabled = command.payload.enabled;
      });
      break;
    case "setParameterValue":
      updateNode(command.payload.node_id, (node) => {
        const parameter = node.parameters.find((candidate) => candidate.id === command.payload.parameter_id);
        if (parameter) {
          parameter.value = command.payload.value;
        }
      });
      break;
    case "addRoute":
      if (!previewSession.routes.some((route) => route.id === command.payload.route.id)) {
        previewSession.routes.push(clone(command.payload.route));
        previewSession.routes.sort((left, right) => left.id.localeCompare(right.id));
      }
      break;
    case "removeRoute":
      previewSession.routes = previewSession.routes.filter((route) => route.id !== command.payload.route_id);
      break;
    case "assignNodeToBus":
      setNodeBus(command.payload.node_id, command.payload.bus_id);
      break;
    case "clearNodeBusAssignment":
      setNodeBus(command.payload.node_id, null);
      break;
  }
  touchSession();
}

function applyPerformanceCommand(command: PerformanceCommand): void {
  switch (command.type) {
    case "recallScene": {
      const scene = previewSession.scenes.find((candidate) => candidate.id === command.payload.scene_id);
      if (!scene) return;
      const activeIds = new Set(scene.activeNodeIds);
      previewSession.nodes = previewSession.nodes.map((node) => ({
        ...node,
        enabled: activeIds.has(node.id),
      }));
      for (const macroOverride of scene.macroOverrides) {
        applyMacroValue(macroOverride.macroId, macroOverride.value);
      }
      previewSession.visualRuntime.activeSceneId = scene.id;
      break;
    }
    case "saveVariation": {
      const scene = previewSession.scenes.find((candidate) => candidate.id === command.payload.scene_id);
      if (!scene) return;
      const activeIds = new Set(scene.activeNodeIds);
      previewSession.variations.push({
        id: nextId("preview-variation"),
        name: command.payload.name,
        sceneId: scene.id,
        parameterOverrides: previewSession.nodes
          .filter((node) => activeIds.has(node.id))
          .flatMap((node) => node.parameters.map((parameter) => ({ parameterId: parameter.id, value: parameter.value }))),
      });
      break;
    }
    case "restoreVariation": {
      const variation = previewSession.variations.find((candidate) => candidate.id === command.payload.variation_id);
      if (!variation) return;
      for (const override of variation.parameterOverrides) {
        updateParameterById(override.parameterId, override.value);
      }
      break;
    }
  }
  touchSession();
}

function applyMacroCommand(command: MacroCommand): void {
  switch (command.type) {
    case "createMacro":
      if (!previewSession.macros.some((macro) => macro.id === command.payload.definition.id)) {
        previewSession.macros.push(clone(command.payload.definition));
      }
      break;
    case "updateMacro":
      previewSession.macros = previewSession.macros.map((macro) =>
        macro.id === command.payload.macro_id
          ? {
              ...macro,
              name: command.payload.name ?? macro.name,
              targets: command.payload.targets ?? macro.targets,
              rangeStart: command.payload.range_start ?? macro.rangeStart,
              rangeEnd: command.payload.range_end ?? macro.rangeEnd,
            }
          : macro,
      );
      break;
    case "removeMacro":
      previewSession.macros = previewSession.macros.filter((macro) => macro.id !== command.payload.macro_id);
      previewSession.scenes = previewSession.scenes.map((scene) => ({
        ...scene,
        macroOverrides: scene.macroOverrides.filter((override) => override.macroId !== command.payload.macro_id),
      }));
      break;
    case "setMacroValue":
      applyMacroValue(command.payload.macro_id, command.payload.value);
      break;
  }
  touchSession();
}

function sendAgentMessage(message: string): { session: SessionDocument; intent: AgentIntent } {
  const intent = parsePreviewIntent(message);

  if (!previewSession.agentFrozen) {
    for (const command of intent.parsedCommands) {
      if (classifyRisk(command) === "high") {
        const pending: PendingAction = {
          id: nextId("preview-pending"),
          correlationId: nextId("preview-correlation"),
          command,
          riskTier: "high",
          createdAt: new Date().toISOString(),
          status: "pending",
        };
        previewSession.pendingActions.push(pending);
      } else {
        applyTypedCommand(command);
        logAction("agent-preview", command);
      }
    }
  }

  touchSession();
  return { session: cloneSession(), intent };
}

function parsePreviewIntent(message: string): AgentIntent {
  const lower = message.toLowerCase();
  const parsedCommands: TypedCommand[] = [];

  if (lower.includes("add") && (lower.includes("osc") || lower.includes("source") || lower.includes("noise"))) {
    parsedCommands.push({
      type: "graphEdit",
      payload: { type: "addNode", payload: { node: buildAgentSourceNode(lower.includes("noise") ? "noise" : "oscillator") } },
    });
  } else if (lower.includes("remove") || lower.includes("delete")) {
    const node = findMentionedNode(lower) ?? previewSession.nodes.find((candidate) => candidate.nodeTypeId !== "output");
    if (node) {
      parsedCommands.push({ type: "graphEdit", payload: { type: "removeNode", payload: { node_id: node.id } } });
    }
  } else if (lower.includes("set")) {
    const node = findMentionedNode(lower) ?? previewSession.nodes.find((candidate) => candidate.parameters.length > 0);
    const parameter = node?.parameters[0];
    const value = extractNumericValue(lower);
    if (node && parameter && value !== null) {
      parsedCommands.push({
        type: "graphEdit",
        payload: { type: "setParameterValue", payload: { node_id: node.id, parameter_id: parameter.id, value } },
      });
    }
  }

  if (lower.includes("recall scene")) {
    const scene = previewSession.scenes.find((candidate) => lower.includes(candidate.name.toLowerCase()));
    if (scene) {
      parsedCommands.push({ type: "performance", payload: { type: "recallScene", payload: { scene_id: scene.id } } });
    }
  }

  if (lower.includes("save variation")) {
    const scene = previewSession.scenes[0];
    if (scene) {
      parsedCommands.push({
        type: "performance",
        payload: { type: "saveVariation", payload: { name: "agent-variation", scene_id: scene.id } },
      });
    }
  }

  if (lower.includes("restore variation")) {
    const variation = previewSession.variations[0];
    if (variation) {
      parsedCommands.push({
        type: "performance",
        payload: { type: "restoreVariation", payload: { variation_id: variation.id } },
      });
    }
  }

  return {
    rawInput: message,
    parsedCommands,
    confidence: parsedCommands.length > 0 ? 0.85 : 0.1,
  };
}

function buildAgentSourceNode(sourceType: "oscillator" | "noise"): Node {
  const id = nextId(`preview-${sourceType}`);
  return {
    id,
    nodeTypeId: sourceType,
    ports: [{ id: `${id}-main_out`, name: "Main Out", direction: "output", signalType: "audio" }],
    parameters: [
      {
        id: `${id}-level`,
        name: "level",
        value: 0.7,
        defaultValue: 0.7,
        minValue: 0,
        maxValue: 1,
        unit: "linear",
      },
    ],
    runtimeTarget: sourceType,
    sceneMembership: previewSession.scenes[0] ? [previewSession.scenes[0].id] : [],
    ownership: { controller: "agent", isLocked: false },
    enabled: true,
    busTargetId: previewSession.buses[0]?.id ?? null,
    channelMode: "mono",
  };
}

function approvePendingAction(actionId: string): void {
  const action = previewSession.pendingActions.find(
    (candidate) => candidate.id === actionId && candidate.status === "pending",
  );
  if (!action) {
    throw new Error(`pending action '${actionId}' not found or already resolved`);
  }

  applyTypedCommand(action.command);
  logAction("user", action.command);
  previewSession.pendingActions = previewSession.pendingActions.filter((candidate) => candidate.id !== actionId);
  touchSession();
}

function rejectPendingAction(actionId: string): void {
  previewSession.pendingActions = previewSession.pendingActions.map((candidate) =>
    candidate.id === actionId ? { ...candidate, status: "rejected" } : candidate,
  );
  touchSession();
}

function reclaimOwnership(nodeIds?: string[] | null, targetController?: ControllerKind | null): void {
  const idSet = nodeIds ? new Set(nodeIds) : null;
  const target = targetController ?? "user";
  previewSession.nodes = previewSession.nodes.map((node) =>
    (!idSet && node.ownership.controller === "agent") || idSet?.has(node.id)
      ? { ...node, ownership: { ...node.ownership, controller: target } }
      : node,
  );
  touchSession();
}

function applyTypedCommand(command: TypedCommand): void {
  if (command.type === "graphEdit") {
    applyGraphEdit(command.payload);
  } else {
    applyPerformanceCommand(command.payload);
  }
}

function classifyRisk(command: TypedCommand): "low" | "medium" | "high" {
  if (command.type === "graphEdit") {
    if (
      command.payload.type === "removeNode" ||
      command.payload.type === "removeRoute" ||
      command.payload.type === "clearNodeBusAssignment"
    ) {
      return "high";
    }
    if (command.payload.type === "setParameterValue") {
      return "low";
    }
  }
  return "medium";
}

function applyMacroValue(macroId: string, value: number): void {
  const macro = previewSession.macros.find((candidate) => candidate.id === macroId);
  if (!macro) return;

  const scaledValue = macro.rangeStart + value * (macro.rangeEnd - macro.rangeStart);
  if (macro.targets.length > 0) {
    for (const target of macro.targets) {
      if (target.kind === "audioParameter") {
        updateNode(target.config.node_id, (node) => {
          const parameter = node.parameters.find((candidate) => candidate.id === target.config.parameter_id);
          if (parameter) parameter.value = clamp(scaledValue, parameter.minValue, parameter.maxValue);
        });
      }
    }
  } else {
    for (const parameterId of macro.targetParameterIds ?? []) {
      updateParameterById(parameterId, scaledValue);
    }
  }
}

function updateNode(nodeId: string, mutate: (node: Node) => void): void {
  previewSession.nodes = previewSession.nodes.map((node) => {
    if (node.id !== nodeId) return node;
    const next = clone(node);
    mutate(next);
    return next;
  });
}

function updateParameterById(parameterId: string, value: number): void {
  previewSession.nodes = previewSession.nodes.map((node) => ({
    ...node,
    parameters: node.parameters.map((parameter) =>
      parameter.id === parameterId
        ? { ...parameter, value: clamp(value, parameter.minValue, parameter.maxValue) }
        : parameter,
    ),
  }));
}

function setNodeBus(nodeId: string, busId: string | null): void {
  updateNode(nodeId, (node) => {
    node.busTargetId = busId;
  });
}

function findMentionedNode(lower: string): Node | null {
  return previewSession.nodes.find((node) => lower.includes(node.id.toLowerCase())) ?? null;
}

function extractNumericValue(lower: string): number | null {
  const match = lower.match(/(?:to|=)\s*(-?\d+(?:\.\d+)?)/) ?? lower.match(/-?\d+(?:\.\d+)?/);
  return match ? Number(match[1] ?? match[0]) : null;
}

function logAction(actorId: string, command: TypedCommand): void {
  const entry: ActionHistoryEntry = {
    id: nextId("preview-action"),
    timestamp: new Date().toISOString(),
    actor: { actorId, correlationId: nextId("preview-correlation") },
    command,
    diff: {
      description: command.type === "graphEdit" ? command.payload.type : command.payload.type,
      affectedNodeIds: command.type === "graphEdit" && "node_id" in command.payload.payload
        ? [command.payload.payload.node_id]
        : [],
      beforeSnippet: "",
      afterSnippet: "",
    },
  };
  previewSession.actionHistory.push(entry);
}

function setRuntimeStatus(
  runtime: SessionDocument["runtimeStatus"][number]["runtime"],
  status: SessionDocument["runtimeStatus"][number]["status"],
  lastError: string | null,
): void {
  previewSession.runtimeStatus = previewSession.runtimeStatus.map((candidate) =>
    candidate.runtime === runtime ? { ...candidate, status, lastError } : candidate,
  );
  touchSession();
}

function cloneSession(): SessionDocument {
  syncDerivedState();
  return clone(previewSession);
}

function syncDerivedState(): void {
  previewSession.agentRuntime = {
    isAvailable: true,
    pendingActionCount: previewSession.pendingActions.filter((action) => action.status === "pending").length,
    isFrozen: previewSession.agentFrozen,
  };
}

function touchSession(): void {
  previewSession.updatedAt = new Date().toISOString();
  syncDerivedState();
}

function readArg<T>(args: PreviewArgs, key: string): T {
  if (!args || !(key in args)) {
    throw new Error(`Missing preview argument '${key}'.`);
  }
  return args[key] as T;
}

function readOptionalArg<T>(args: PreviewArgs, key: string): T | null {
  if (!args || !(key in args)) {
    return null;
  }
  return args[key] as T | null;
}

function nextId(prefix: string): string {
  idCounter += 1;
  const randomId = globalThis.crypto?.randomUUID?.().slice(0, 8) ?? idCounter.toString(36);
  return `${prefix}-${randomId}`;
}

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
