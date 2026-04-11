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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Source,
    Mixer,
    Output,
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

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn write_generated_typescript_contract() -> std::io::Result<()> {
    let cfg = Config::default();
    let declarations = [
        SessionDocument::decl(&cfg),
        TransportState::decl(&cfg),
        Node::decl(&cfg),
        NodeType::decl(&cfg),
        Port::decl(&cfg),
        PortDirection::decl(&cfg),
        SignalType::decl(&cfg),
        ParameterValue::decl(&cfg),
        Route::decl(&cfg),
        Bus::decl(&cfg),
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
                unit: "linear".to_string(),
            }],
            runtime_target: Some("audio:source".to_string()),
            scene_membership: vec![new_id()],
            ownership: OwnershipAssignment {
                controller: ControllerKind::Shared,
                is_locked: false,
            },
        };

        assert!(!node.id.is_empty());
        assert_eq!(node.node_type, NodeType::Source);
        assert_eq!(node.ports.len(), 1);
        assert_eq!(node.parameters.len(), 1);
        assert_eq!(node.runtime_target.as_deref(), Some("audio:source"));
        assert_eq!(node.scene_membership.len(), 1);
        assert_eq!(node.ownership.controller, ControllerKind::Shared);
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
}
