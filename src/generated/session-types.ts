// Generated from Rust session contracts.

export type SessionDocument = { schemaVersion: number, sessionId: string, title: string, createdAt: string, updatedAt: string, transport: TransportState, nodes: Array<Node>, routes: Array<Route>, buses: Array<Bus>, macros: Array<MacroDefinition>, scenes: Array<SceneDefinition>, variations: Array<VariationDefinition>, ownershipRules: Array<OwnershipRule>, runtimeStatus: Array<RuntimeStatusRef>, };

export type TransportState = { tempoBpm: number, isPlaying: boolean, positionBeats: number, };

export type Node = { id: string, nodeType: NodeType, ports: Array<Port>, parameters: Array<ParameterValue>, runtimeTarget: string | null, sceneMembership: Array<string>, ownership: OwnershipAssignment, };

export type NodeType = "source" | "mixer" | "output";

export type Port = { id: string, name: string, direction: PortDirection, signalType: SignalType, };

export type PortDirection = "input" | "output";

export type SignalType = "audio" | "control";

export type ParameterValue = { id: string, name: string, value: number, unit: string, };

export type Route = { id: string, sourceNodeId: string, sourcePortId: string, targetNodeId: string, targetPortId: string, busId: string | null, };

export type Bus = { id: string, name: string, channels: number, };

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
