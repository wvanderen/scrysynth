// Generated from Rust session contracts.

export type SessionDocument = { schemaVersion: number, sessionId: string, title: string, createdAt: string, updatedAt: string, transport: TransportState, audioRuntime: AudioRuntimeState, nodes: Array<Node>, routes: Array<Route>, buses: Array<Bus>, macros: Array<MacroDefinition>, scenes: Array<SceneDefinition>, variations: Array<VariationDefinition>, ownershipRules: Array<OwnershipRule>, runtimeStatus: Array<RuntimeStatusRef>, visualRuntime: VisualRuntimeState, agentRuntime: AgentRuntimeState, agentFrozen: boolean, pendingActions: Array<PendingAction>, actionHistory: Array<ActionHistoryEntry>, hardwareBindings: Array<HardwareBinding>, };

export type TransportState = { tempoBpm: number, isPlaying: boolean, positionBeats: number, };

export type AudioRuntimeState = { lifecycle: AudioRuntimeLifecycle, health: AudioRuntimeHealth, sampleRateHz: number | null, blockSize: number | null, activePatchId: string | null, lastError: string | null, panicRecoveryCount: number, };

export type AudioRuntimeLifecycle = "idle" | "booting" | "ready" | "running" | "recovering" | "failed";

export type AudioRuntimeHealth = "unknown" | "healthy" | "degraded" | "panic_recovered" | "error";

export type VisualRuntimeLifecycle = "idle" | "starting" | "ready" | "rendering" | "panicked" | "failed";

export type VisualRuntimeHealth = "unknown" | "healthy" | "degraded" | "error";

export type VisualRuntimeState = { lifecycle: VisualRuntimeLifecycle, health: VisualRuntimeHealth, activeSceneId: string | null, fps: number | null, lastError: string | null, renderer: string | null, };

export type AgentRuntimeState = { isAvailable: boolean, pendingActionCount: number, isFrozen: boolean, };

export type Node = { id: string, nodeType: NodeType, ports: Array<Port>, parameters: Array<ParameterValue>, runtimeTarget: string | null, sceneMembership: Array<string>, ownership: OwnershipAssignment, enabled: boolean, audioPrimitive: AudioPrimitive | null, };

export type NodeType = "source" | "effect" | "mixer" | "output";

export type AudioPrimitive = { "kind": "source", "config": AudioSourceNode } | { "kind": "effect", "config": AudioEffectNode } | { "kind": "mixer", "config": AudioMixerNode } | { "kind": "output", "config": AudioOutputNode };

export type AudioSourceNode = { sourceType: AudioSourceType, channelMode: ChannelMode, busTargetId: string | null, };

export type AudioSourceType = "oscillator" | "noise";

export type AudioEffectNode = { effectType: AudioEffectType, bypassed: boolean, busTargetId: string | null, };

export type AudioEffectType = "low_pass_filter" | "delay";

export type AudioMixerNode = { channelMode: ChannelMode, busTargetId: string | null, };

export type AudioOutputNode = { outputType: AudioOutputType, channels: number, busTargetId: string | null, };

export type AudioOutputType = "master" | "cue";

export type ChannelMode = "mono" | "stereo";

export type Port = { id: string, name: string, direction: PortDirection, signalType: SignalType, };

export type PortDirection = "input" | "output";

export type SignalType = "audio" | "control";

export type ParameterValue = { id: string, name: string, value: number, defaultValue: number, minValue: number, maxValue: number, unit: string, };

export type Route = { id: string, sourceNodeId: string, sourcePortId: string, targetNodeId: string, targetPortId: string, busId: string | null, };

export type Bus = { id: string, name: string, channels: number, busType: AudioBusType, isEnabled: boolean, };

export type AudioBusType = "auxiliary" | "main" | "cue";

export type MacroTarget = { "kind": "audioParameter", "config": { node_id: string, parameter_id: string, } } | { "kind": "visualParameter", "config": { element_id: string, parameter_id: string, } };

export type MacroDefinition = { id: string, name: string, targetParameterIds?: Array<string>, rangeStart: number, rangeEnd: number, targets: Array<MacroTarget>, };

export type MacroCommand = { "type": "createMacro", "payload": { definition: MacroDefinition, } } | { "type": "updateMacro", "payload": { macro_id: string, name: string | null, targets: Array<MacroTarget> | null, range_start: number | null, range_end: number | null, } } | { "type": "removeMacro", "payload": { macro_id: string, } } | { "type": "setMacroValue", "payload": { macro_id: string, value: number, } };

export type SceneDefinition = { id: string, name: string, activeNodeIds: Array<string>, macroOverrides: Array<MacroOverride>, };

export type MacroOverride = { macroId: string, value: number, };

export type VariationDefinition = { id: string, name: string, sceneId: string, parameterOverrides: Array<ParameterOverride>, };

export type ParameterOverride = { parameterId: string, value: number, };

export type OwnershipRule = { id: string, scope: string, controller: ControllerKind, canOverride: boolean, };

export type ControllerKind = "user" | "agent" | "shared";

export type OwnershipAssignment = { controller: ControllerKind, isLocked: boolean, };

export type RuntimeStatusRef = { id: string, runtime: RuntimeKind, status: RuntimeConnectionState, targetId: string | null, lastError: string | null, };

export type RuntimeKind = "audio" | "visual" | "agent";

export type RuntimeConnectionState = "disconnected" | "connecting" | "ready" | "error";

export type GraphEditCommand = { "type": "addNode", "payload": { node: Node, } } | { "type": "removeNode", "payload": { node_id: string, } } | { "type": "setNodeEnabled", "payload": { node_id: string, enabled: boolean, } } | { "type": "setParameterValue", "payload": { node_id: string, parameter_id: string, value: number, } } | { "type": "addRoute", "payload": { route: Route, } } | { "type": "removeRoute", "payload": { route_id: string, } } | { "type": "assignNodeToBus", "payload": { node_id: string, bus_id: string, } } | { "type": "clearNodeBusAssignment", "payload": { node_id: string, } };

export type PerformanceCommand = { "type": "recallScene", "payload": { scene_id: string, } } | { "type": "saveVariation", "payload": { name: string, scene_id: string, } } | { "type": "restoreVariation", "payload": { variation_id: string, } };

export type ActorRef = { actorId: string, correlationId: string, };

export type TypedCommand = { "type": "graphEdit", "payload": GraphEditCommand } | { "type": "performance", "payload": PerformanceCommand };

export type AgentIntent = { rawInput: string, parsedCommands: Array<TypedCommand>, confidence: number, };

export type RiskTier = "low" | "medium" | "high";

export type DiffSummary = { description: string, affectedNodeIds: Array<string>, beforeSnippet: string, afterSnippet: string, };

export type PendingActionStatus = "pending" | "approved" | "rejected";

export type PendingAction = { id: string, correlationId: string, command: TypedCommand, riskTier: RiskTier, createdAt: string, status: PendingActionStatus, };

export type ActionHistoryEntry = { id: string, timestamp: string, actor: ActorRef, command: TypedCommand, diff: DiffSummary, };

export type HardwareSource = { "kind": "midiCc", "config": { channel: number, controller: number, } } | { "kind": "midiNote", "config": { channel: number, note: number, } } | { "kind": "midiPitchBend", "config": { channel: number, } } | { "kind": "oscAddress", "config": { address: string, } };

export type BindingTarget = { "kind": "macro", "config": { macro_id: string, } } | { "kind": "sceneRecall", "config": { scene_id: string, } } | { "kind": "transportPlay" } | { "kind": "transportStop" } | { "kind": "transportPanic" };

export type ValueTransform = { inputMin: number, inputMax: number, outputMin: number, outputMax: number, };

export type HardwareBinding = { id: string, source: HardwareSource, target: BindingTarget, transform: ValueTransform, };

export type MidiInputPort = { id: string, displayName: string, isSelected: boolean, };

export type MidiInputSettings = { selectedInputId: string | null, autoStart: boolean, };

export type OscInputSettings = { bindHost: string, listenPort: number, autoStart: boolean, };

export type HardwareRuntimeSettings = { midi: MidiInputSettings, osc: OscInputSettings, };

export type HardwareListenerLifecycle = "unavailable" | "stopped" | "starting" | "listening" | "restarting" | "error";

export type MidiRuntimeStatus = { lifecycle: HardwareListenerLifecycle, selectedInputId: string | null, selectedDisplayName: string | null, availableInputCount: number | null, lastError: string | null, };

export type OscRuntimeStatus = { lifecycle: HardwareListenerLifecycle, bindHost: string, listenPort: number, lastError: string | null, };

export type HardwareLearnLifecycle = "idle" | "learning" | "captured";

export type HardwareLearnStatus = { lifecycle: HardwareLearnLifecycle, target: BindingTarget | null, source: HardwareSource | null, };

export type HardwareRuntimeDiagnosticCode = "no_midi_ports" | "invalid_midi_port_selection" | "midi_enumeration_failed" | "osc_bind_failed" | "osc_port_in_use" | "listener_restart_required" | "listener_restarted" | "listener_stopped" | "listener_start_pending";

export type HardwareRuntimeDiagnostic = { code: HardwareRuntimeDiagnosticCode, message: string, recoverable: boolean, detail: string | null, };

export type HardwareRuntimeStatus = { midi: MidiRuntimeStatus, osc: OscRuntimeStatus, learn: HardwareLearnStatus, diagnostics: Array<HardwareRuntimeDiagnostic>, };
