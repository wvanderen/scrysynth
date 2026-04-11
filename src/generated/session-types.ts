// Generated from Rust session contracts.

export type SessionDocument = { schema_version: number, session_id: string, title: string, created_at: string, updated_at: string, transport: TransportState, nodes: Array<Node>, routes: Array<Route>, buses: Array<Bus>, macros: Array<MacroDefinition>, scenes: Array<SceneDefinition>, variations: Array<VariationDefinition>, ownership_rules: Array<OwnershipRule>, runtime_status: Array<RuntimeStatusRef>, };

export type TransportState = { tempo_bpm: number, is_playing: boolean, position_beats: number, };

export type Node = { id: string, node_type: NodeType, ports: Array<Port>, parameters: Array<ParameterValue>, runtime_target: string | null, scene_membership: Array<string>, ownership: OwnershipAssignment, };

export type NodeType = "source" | "mixer" | "output";

export type Port = { id: string, name: string, direction: PortDirection, signal_type: SignalType, };

export type PortDirection = "input" | "output";

export type SignalType = "audio" | "control";

export type ParameterValue = { id: string, name: string, value: number, unit: string, };

export type Route = { id: string, source_node_id: string, source_port_id: string, target_node_id: string, target_port_id: string, bus_id: string | null, };

export type Bus = { id: string, name: string, channels: number, };

export type MacroDefinition = { id: string, name: string, target_parameter_ids: Array<string>, range_start: number, range_end: number, };

export type SceneDefinition = { id: string, name: string, active_node_ids: Array<string>, macro_overrides: Array<MacroOverride>, };

export type MacroOverride = { macro_id: string, value: number, };

export type VariationDefinition = { id: string, name: string, scene_id: string, parameter_overrides: Array<ParameterOverride>, };

export type ParameterOverride = { parameter_id: string, value: number, };

export type OwnershipRule = { id: string, scope: string, controller: ControllerKind, can_override: boolean, };

export type ControllerKind = "user" | "agent" | "shared";

export type OwnershipAssignment = { controller: ControllerKind, is_locked: boolean, };

export type RuntimeStatusRef = { id: string, runtime: RuntimeKind, status: RuntimeConnectionState, target_id: string | null, last_error: string | null, };

export type RuntimeKind = "audio" | "visual" | "agent";

export type RuntimeConnectionState = "disconnected" | "connecting" | "ready" | "error";
