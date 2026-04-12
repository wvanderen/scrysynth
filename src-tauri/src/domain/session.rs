use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use ts_rs::{Config, TS};
use uuid::Uuid;

const GENERATED_TYPES_PATH: &str = "../src/generated/session-types.ts";
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SessionDocument {
    pub schema_version: u32,
    pub session_id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub transport: TransportState,
    #[serde(default)]
    pub audio_runtime: AudioRuntimeState,
    pub nodes: Vec<Node>,
    pub routes: Vec<Route>,
    pub buses: Vec<Bus>,
    pub macros: Vec<MacroDefinition>,
    pub scenes: Vec<SceneDefinition>,
    pub variations: Vec<VariationDefinition>,
    pub ownership_rules: Vec<OwnershipRule>,
    pub runtime_status: Vec<RuntimeStatusRef>,
}

impl Default for SessionDocument {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            session_id: new_id(),
            title: "Untitled Session".to_string(),
            created_at: "2026-04-11T00:00:00Z".to_string(),
            updated_at: "2026-04-11T00:00:00Z".to_string(),
            transport: TransportState::default(),
            audio_runtime: AudioRuntimeState::default(),
            nodes: Vec::new(),
            routes: Vec::new(),
            buses: Vec::new(),
            macros: Vec::new(),
            scenes: Vec::new(),
            variations: Vec::new(),
            ownership_rules: Vec::new(),
            runtime_status: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AudioRuntimeState {
    pub lifecycle: AudioRuntimeLifecycle,
    pub health: AudioRuntimeHealth,
    pub sample_rate_hz: Option<u32>,
    pub block_size: Option<u32>,
    pub active_patch_id: Option<String>,
    pub last_error: Option<String>,
    pub panic_recovery_count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AudioRuntimeLifecycle {
    #[default]
    Idle,
    Booting,
    Ready,
    Running,
    Recovering,
    Failed,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AudioRuntimeHealth {
    #[default]
    Unknown,
    Healthy,
    Degraded,
    PanicRecovered,
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TransportState {
    pub tempo_bpm: f32,
    pub is_playing: bool,
    pub position_beats: f32,
}

impl Default for TransportState {
    fn default() -> Self {
        Self {
            tempo_bpm: 120.0,
            is_playing: false,
            position_beats: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub ports: Vec<Port>,
    pub parameters: Vec<ParameterValue>,
    pub runtime_target: Option<String>,
    pub scene_membership: Vec<String>,
    pub ownership: OwnershipAssignment,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub audio_primitive: Option<AudioPrimitive>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Source,
    Effect,
    Mixer,
    Output,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "config", rename_all = "camelCase")]
pub enum AudioPrimitive {
    Source(AudioSourceNode),
    Effect(AudioEffectNode),
    Mixer(AudioMixerNode),
    Output(AudioOutputNode),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AudioSourceNode {
    pub source_type: AudioSourceType,
    pub channel_mode: ChannelMode,
    pub bus_target_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AudioSourceType {
    Oscillator,
    Noise,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AudioEffectNode {
    pub effect_type: AudioEffectType,
    pub bypassed: bool,
    pub bus_target_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AudioEffectType {
    LowPassFilter,
    Delay,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AudioMixerNode {
    pub channel_mode: ChannelMode,
    pub bus_target_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AudioOutputNode {
    pub output_type: AudioOutputType,
    pub channels: u32,
    pub bus_target_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AudioOutputType {
    Master,
    Cue,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ChannelMode {
    Mono,
    Stereo,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Port {
    pub id: String,
    pub name: String,
    pub direction: PortDirection,
    pub signal_type: SignalType,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    Audio,
    Control,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ParameterValue {
    pub id: String,
    pub name: String,
    pub value: f64,
    #[serde(default = "parameter_default_value")]
    pub default_value: f64,
    #[serde(default = "parameter_default_min")]
    pub min_value: f64,
    #[serde(default = "parameter_default_max")]
    pub max_value: f64,
    pub unit: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    pub id: String,
    pub source_node_id: String,
    pub source_port_id: String,
    pub target_node_id: String,
    pub target_port_id: String,
    pub bus_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Bus {
    pub id: String,
    pub name: String,
    pub channels: u32,
    #[serde(default)]
    pub bus_type: AudioBusType,
    #[serde(default = "default_enabled")]
    pub is_enabled: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AudioBusType {
    #[default]
    Auxiliary,
    Main,
    Cue,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MacroDefinition {
    pub id: String,
    pub name: String,
    pub target_parameter_ids: Vec<String>,
    pub range_start: f64,
    pub range_end: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SceneDefinition {
    pub id: String,
    pub name: String,
    pub active_node_ids: Vec<String>,
    pub macro_overrides: Vec<MacroOverride>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MacroOverride {
    pub macro_id: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct VariationDefinition {
    pub id: String,
    pub name: String,
    pub scene_id: String,
    pub parameter_overrides: Vec<ParameterOverride>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ParameterOverride {
    pub parameter_id: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct OwnershipRule {
    pub id: String,
    pub scope: String,
    pub controller: ControllerKind,
    pub can_override: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ControllerKind {
    User,
    Agent,
    Shared,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct OwnershipAssignment {
    pub controller: ControllerKind,
    pub is_locked: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatusRef {
    pub id: String,
    pub runtime: RuntimeKind,
    pub status: RuntimeConnectionState,
    pub target_id: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeKind {
    Audio,
    Visual,
    Agent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeConnectionState {
    Disconnected,
    Connecting,
    Ready,
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum GraphEditCommand {
    AddNode {
        node: Node,
    },
    RemoveNode {
        node_id: String,
    },
    SetNodeEnabled {
        node_id: String,
        enabled: bool,
    },
    SetParameterValue {
        node_id: String,
        parameter_id: String,
        value: f64,
    },
    AddRoute {
        route: Route,
    },
    RemoveRoute {
        route_id: String,
    },
    AssignNodeToBus {
        node_id: String,
        bus_id: String,
    },
    ClearNodeBusAssignment {
        node_id: String,
    },
}

fn default_enabled() -> bool {
    true
}

fn parameter_default_value() -> f64 {
    0.0
}

fn parameter_default_min() -> f64 {
    0.0
}

fn parameter_default_max() -> f64 {
    1.0
}

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn write_generated_typescript_contract() -> std::io::Result<()> {
    let cfg = Config::default();
    let declarations = [
        SessionDocument::decl(&cfg),
        TransportState::decl(&cfg),
        AudioRuntimeState::decl(&cfg),
        AudioRuntimeLifecycle::decl(&cfg),
        AudioRuntimeHealth::decl(&cfg),
        Node::decl(&cfg),
        NodeType::decl(&cfg),
        AudioPrimitive::decl(&cfg),
        AudioSourceNode::decl(&cfg),
        AudioSourceType::decl(&cfg),
        AudioEffectNode::decl(&cfg),
        AudioEffectType::decl(&cfg),
        AudioMixerNode::decl(&cfg),
        AudioOutputNode::decl(&cfg),
        AudioOutputType::decl(&cfg),
        ChannelMode::decl(&cfg),
        Port::decl(&cfg),
        PortDirection::decl(&cfg),
        SignalType::decl(&cfg),
        ParameterValue::decl(&cfg),
        Route::decl(&cfg),
        Bus::decl(&cfg),
        AudioBusType::decl(&cfg),
        MacroDefinition::decl(&cfg),
        SceneDefinition::decl(&cfg),
        MacroOverride::decl(&cfg),
        VariationDefinition::decl(&cfg),
        ParameterOverride::decl(&cfg),
        OwnershipRule::decl(&cfg),
        ControllerKind::decl(&cfg),
        OwnershipAssignment::decl(&cfg),
        RuntimeStatusRef::decl(&cfg),
        RuntimeKind::decl(&cfg),
        RuntimeConnectionState::decl(&cfg),
        GraphEditCommand::decl(&cfg),
    ]
    .join("\n\n");

    let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GENERATED_TYPES_PATH);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let generated = declarations
        .replace("\ntype ", "\nexport type ")
        .replace("\ninterface ", "\nexport interface ");
    let generated = if let Some(rest) = generated.strip_prefix("type ") {
        format!("export type {rest}")
    } else if let Some(rest) = generated.strip_prefix("interface ") {
        format!("export interface {rest}")
    } else {
        generated
    };

    fs::write(
        file_path,
        format!(
            "// Generated from Rust session contracts.\n\n{}\n",
            generated
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_document_default_round_trip_preserves_required_collections() {
        let session = SessionDocument::default();
        let json = serde_json::to_string(&session).expect("session serializes");
        let restored: SessionDocument = serde_json::from_str(&json).expect("session deserializes");

        assert_eq!(restored.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(restored.audio_runtime, session.audio_runtime);
        assert_eq!(restored.nodes, session.nodes);
        assert_eq!(restored.routes, session.routes);
        assert_eq!(restored.buses, session.buses);
        assert_eq!(restored.macros, session.macros);
        assert_eq!(restored.scenes, session.scenes);
        assert_eq!(restored.variations, session.variations);
        assert_eq!(restored.ownership_rules, session.ownership_rules);
        assert_eq!(restored.runtime_status, session.runtime_status);
    }

    #[test]
    fn session_document_node_exposes_required_fields() {
        let node = Node {
            id: new_id(),
            node_type: NodeType::Source,
            ports: vec![Port {
                id: new_id(),
                name: "out".to_string(),
                direction: PortDirection::Output,
                signal_type: SignalType::Audio,
            }],
            parameters: vec![ParameterValue {
                id: new_id(),
                name: "gain".to_string(),
                value: 0.75,
                default_value: 0.75,
                min_value: 0.0,
                max_value: 1.0,
                unit: "linear".to_string(),
            }],
            runtime_target: Some("audio:source".to_string()),
            scene_membership: vec![new_id()],
            ownership: OwnershipAssignment {
                controller: ControllerKind::Shared,
                is_locked: false,
            },
            enabled: true,
            audio_primitive: Some(AudioPrimitive::Source(AudioSourceNode {
                source_type: AudioSourceType::Oscillator,
                channel_mode: ChannelMode::Mono,
                bus_target_id: None,
            })),
        };

        assert!(!node.id.is_empty());
        assert_eq!(node.node_type, NodeType::Source);
        assert_eq!(node.ports.len(), 1);
        assert_eq!(node.parameters.len(), 1);
        assert_eq!(node.runtime_target.as_deref(), Some("audio:source"));
        assert_eq!(node.scene_membership.len(), 1);
        assert_eq!(node.ownership.controller, ControllerKind::Shared);
        assert!(node.enabled);
        assert!(matches!(
            node.audio_primitive,
            Some(AudioPrimitive::Source(_))
        ));
    }

    #[test]
    fn session_document_exports_typescript_contracts() {
        write_generated_typescript_contract().expect("typescript contract is written");

        let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GENERATED_TYPES_PATH);
        let generated = fs::read_to_string(file_path).expect("generated types are readable");

        assert!(generated.contains("export type SessionDocument"));
        assert!(generated.contains("export type Node"));
        assert!(generated.contains("export type OwnershipRule"));
    }

    #[test]
    fn audio_graph_schema_round_trips_supported_v1_primitives() {
        let session = SessionDocument {
            audio_runtime: AudioRuntimeState {
                lifecycle: AudioRuntimeLifecycle::Ready,
                health: AudioRuntimeHealth::Healthy,
                sample_rate_hz: Some(48_000),
                block_size: Some(64),
                active_patch_id: Some("patch-main".to_string()),
                last_error: None,
                panic_recovery_count: 0,
            },
            nodes: vec![
                Node {
                    id: "node-source".to_string(),
                    node_type: NodeType::Source,
                    ports: vec![Port {
                        id: "node-source-out".to_string(),
                        name: "main_out".to_string(),
                        direction: PortDirection::Output,
                        signal_type: SignalType::Audio,
                    }],
                    parameters: vec![ParameterValue {
                        id: "param-frequency".to_string(),
                        name: "frequency".to_string(),
                        value: 220.0,
                        default_value: 220.0,
                        min_value: 20.0,
                        max_value: 20_000.0,
                        unit: "hz".to_string(),
                    }],
                    runtime_target: Some("audio/source/oscillator".to_string()),
                    scene_membership: vec![],
                    ownership: OwnershipAssignment {
                        controller: ControllerKind::Shared,
                        is_locked: false,
                    },
                    enabled: true,
                    audio_primitive: Some(AudioPrimitive::Source(AudioSourceNode {
                        source_type: AudioSourceType::Oscillator,
                        channel_mode: ChannelMode::Mono,
                        bus_target_id: Some("bus-main".to_string()),
                    })),
                },
                Node {
                    id: "node-fx".to_string(),
                    node_type: NodeType::Effect,
                    ports: vec![
                        Port {
                            id: "node-fx-in".to_string(),
                            name: "signal_in".to_string(),
                            direction: PortDirection::Input,
                            signal_type: SignalType::Audio,
                        },
                        Port {
                            id: "node-fx-out".to_string(),
                            name: "signal_out".to_string(),
                            direction: PortDirection::Output,
                            signal_type: SignalType::Audio,
                        },
                    ],
                    parameters: vec![ParameterValue {
                        id: "param-mix".to_string(),
                        name: "mix".to_string(),
                        value: 0.35,
                        default_value: 0.35,
                        min_value: 0.0,
                        max_value: 1.0,
                        unit: "ratio".to_string(),
                    }],
                    runtime_target: Some("audio/effect/filter".to_string()),
                    scene_membership: vec![],
                    ownership: OwnershipAssignment {
                        controller: ControllerKind::User,
                        is_locked: false,
                    },
                    enabled: false,
                    audio_primitive: Some(AudioPrimitive::Effect(AudioEffectNode {
                        effect_type: AudioEffectType::LowPassFilter,
                        bypassed: true,
                        bus_target_id: Some("bus-main".to_string()),
                    })),
                },
                Node {
                    id: "node-mix".to_string(),
                    node_type: NodeType::Mixer,
                    ports: vec![],
                    parameters: vec![],
                    runtime_target: Some("audio/mixer/submix".to_string()),
                    scene_membership: vec![],
                    ownership: OwnershipAssignment {
                        controller: ControllerKind::Shared,
                        is_locked: false,
                    },
                    enabled: true,
                    audio_primitive: Some(AudioPrimitive::Mixer(AudioMixerNode {
                        channel_mode: ChannelMode::Stereo,
                        bus_target_id: Some("bus-main".to_string()),
                    })),
                },
                Node {
                    id: "node-out".to_string(),
                    node_type: NodeType::Output,
                    ports: vec![],
                    parameters: vec![],
                    runtime_target: Some("audio/output/master".to_string()),
                    scene_membership: vec![],
                    ownership: OwnershipAssignment {
                        controller: ControllerKind::User,
                        is_locked: false,
                    },
                    enabled: true,
                    audio_primitive: Some(AudioPrimitive::Output(AudioOutputNode {
                        output_type: AudioOutputType::Master,
                        channels: 2,
                        bus_target_id: Some("bus-main".to_string()),
                    })),
                },
            ],
            buses: vec![Bus {
                id: "bus-main".to_string(),
                name: "main".to_string(),
                channels: 2,
                bus_type: AudioBusType::Main,
                is_enabled: true,
            }],
            ..SessionDocument::default()
        };

        let json = serde_json::to_string(&session).expect("session serializes");
        let restored: SessionDocument = serde_json::from_str(&json).expect("session deserializes");

        assert_eq!(restored.audio_runtime, session.audio_runtime);
        assert_eq!(restored.nodes, session.nodes);
        assert_eq!(restored.buses, session.buses);
    }

    #[test]
    fn audio_graph_schema_exports_typescript_contracts() {
        write_generated_typescript_contract().expect("typescript contract is written");

        let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GENERATED_TYPES_PATH);
        let generated = fs::read_to_string(file_path).expect("generated types are readable");

        assert!(generated.contains("export type GraphEditCommand"));
        assert!(generated.contains("export type AudioRuntimeState"));
        assert!(generated.contains("export type AudioPrimitive"));
        assert!(generated.contains("export type AudioBusType"));
    }
}
