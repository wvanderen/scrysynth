// Generated from Rust session contracts.

export type SessionDocument = { schemaVersion: number, sessionId: string, title: string, createdAt: string, updatedAt: string, transport: TransportState, audioRuntime: AudioRuntimeState, nodes: Array<Node>, routes: Array<Route>, buses: Array<Bus>, macros: Array<MacroDefinition>, scenes: Array<SceneDefinition>, variations: Array<VariationDefinition>, ownershipRules: Array<OwnershipRule>, runtimeStatus: Array<RuntimeStatusRef>, };

export type TransportState = { tempoBpm: number, isPlaying: boolean, positionBeats: number, };

export type AudioRuntimeState = { lifecycle: AudioRuntimeLifecycle, health: AudioRuntimeHealth, sampleRateHz: number | null, blockSize: number | null, activePatchId: string | null, lastError: string | null, panicRecoveryCount: number, };

export type AudioRuntimeLifecycle = "idle" | "booting" | "ready" | "running" | "recovering" | "failed";

export type AudioRuntimeHealth = "unknown" | "healthy" | "degraded" | "panic_recovered" | "error";

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

export type MacroDefinition = { id: string, name: string, targetParameterIds: Array<string>, rangeStart: number, rangeEnd: number, };

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
