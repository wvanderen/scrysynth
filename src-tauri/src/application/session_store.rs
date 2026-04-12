use crate::audio::runtime_manager::{AudioRuntimeManager, AudioRuntimeManagerError};
use crate::domain::session::{
    new_id, ActionHistoryEntry, ActorRef, AgentRuntimeState, AudioBusType, AudioOutputNode,
    AudioOutputType, AudioPrimitive, AudioRuntimeHealth, AudioRuntimeLifecycle, AudioRuntimeState,
    AudioSourceNode, AudioSourceType, Bus, ChannelMode, ControllerKind, DiffSummary,
    GraphEditCommand, MacroDefinition, MacroOverride, Node, NodeType, OwnershipAssignment,
    OwnershipRule, ParameterOverride, ParameterValue, PerformanceCommand, Port, PortDirection,
    Route, RuntimeConnectionState, RuntimeKind, RuntimeStatusRef, SceneDefinition, SessionDocument,
    SignalType, TypedCommand, VariationDefinition,
};
use crate::visual::runtime_manager::{VisualRuntimeManager, VisualRuntimeManagerError};

#[derive(Debug, Clone, PartialEq)]
pub struct OwnershipGateError {
    pub node_id: String,
    pub reason: OwnershipGateReason,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipGateReason {
    AgentFrozen,
    LockedNode,
    AgentBlockedByUserOwnership,
}

impl std::fmt::Display for OwnershipGateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.reason {
            OwnershipGateReason::AgentFrozen => write!(f, "agent is frozen"),
            OwnershipGateReason::LockedNode => write!(f, "node '{}' is locked", self.node_id),
            OwnershipGateReason::AgentBlockedByUserOwnership => {
                write!(f, "node '{}' is user-owned", self.node_id)
            }
        }
    }
}

impl std::error::Error for OwnershipGateError {}

#[derive(Debug)]
pub struct SessionStore {
    current: SessionDocument,
    audio_runtime_manager: AudioRuntimeManager,
    visual_runtime_manager: VisualRuntimeManager,
}

impl SessionStore {
    pub fn new_default() -> Self {
        Self {
            current: build_default_session(),
            audio_runtime_manager: AudioRuntimeManager::default(),
            visual_runtime_manager: VisualRuntimeManager::default(),
        }
    }

    pub fn current(&self) -> SessionDocument {
        self.current.clone()
    }

    pub fn replace_current(&mut self, session: SessionDocument) {
        self.current = session;
    }

    pub fn start_audio_runtime(&mut self) -> Result<SessionDocument, AudioRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.audio_runtime_manager);
        let result = manager.start(self);
        self.audio_runtime_manager = manager;
        result
    }

    pub fn stop_audio_runtime(&mut self) -> Result<SessionDocument, AudioRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.audio_runtime_manager);
        let result = manager.stop(self);
        self.audio_runtime_manager = manager;
        result
    }

    pub fn panic_audio_runtime(&mut self) -> Result<SessionDocument, AudioRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.audio_runtime_manager);
        let result = manager.panic(self);
        self.audio_runtime_manager = manager;
        result
    }

    pub fn start_visual_runtime(&mut self) -> Result<SessionDocument, VisualRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.visual_runtime_manager);
        let result = manager.start(self);
        self.visual_runtime_manager = manager;
        result
    }

    pub fn stop_visual_runtime(&mut self) -> Result<SessionDocument, VisualRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.visual_runtime_manager);
        let result = manager.stop(self);
        self.visual_runtime_manager = manager;
        result
    }

    pub fn panic_visual_runtime(&mut self) -> Result<SessionDocument, VisualRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.visual_runtime_manager);
        let result = manager.panic(self);
        self.visual_runtime_manager = manager;
        result
    }

    pub fn derive_agent_runtime_state(&self) -> AgentRuntimeState {
        AgentRuntimeState {
            is_available: true,
            pending_action_count: self.current.pending_actions.len() as u32,
            is_frozen: self.current.agent_frozen,
        }
    }

    pub fn mutate_current<F, E>(&mut self, mutate: F) -> Result<SessionDocument, E>
    where
        F: FnOnce(&mut SessionDocument) -> Result<(), E>,
    {
        let mut next = self.current.clone();
        mutate(&mut next)?;
        self.current = next.clone();
        Ok(next)
    }

    pub fn check_ownership(
        &self,
        actor: &ActorRef,
        command: &TypedCommand,
    ) -> Result<(), Vec<OwnershipGateError>> {
        if actor.actor_id == "user" {
            return Ok(());
        }

        if self.current.agent_frozen {
            return Err(vec![OwnershipGateError {
                node_id: String::new(),
                reason: OwnershipGateReason::AgentFrozen,
            }]);
        }

        let target_ids = extract_target_node_ids(command);
        let mut errors = Vec::new();

        for node_id in &target_ids {
            if let Some(node) = self.current.nodes.iter().find(|n| &n.id == node_id) {
                if node.ownership.is_locked {
                    errors.push(OwnershipGateError {
                        node_id: node_id.clone(),
                        reason: OwnershipGateReason::LockedNode,
                    });
                } else if node.ownership.controller == ControllerKind::User {
                    errors.push(OwnershipGateError {
                        node_id: node_id.clone(),
                        reason: OwnershipGateReason::AgentBlockedByUserOwnership,
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn log_action(&mut self, actor: &ActorRef, command: &TypedCommand) {
        let _ = self.mutate_current(|session| {
            let diff = generate_diff_summary(command, session);
            let entry = ActionHistoryEntry {
                id: new_id(),
                timestamp: "2026-04-12T00:00:00Z".to_string(),
                actor: actor.clone(),
                command: command.clone(),
                diff,
            };
            session.action_history.push(entry);
            if session.action_history.len() > 200 {
                session
                    .action_history
                    .drain(0..session.action_history.len() - 200);
            }
            Ok::<(), ()>(())
        });
    }
}

fn extract_target_node_ids(command: &TypedCommand) -> Vec<String> {
    match command {
        TypedCommand::GraphEdit(gec) => match gec {
            GraphEditCommand::AddNode { .. } => vec![],
            GraphEditCommand::RemoveNode { node_id } => vec![node_id.clone()],
            GraphEditCommand::SetNodeEnabled { node_id, .. } => vec![node_id.clone()],
            GraphEditCommand::SetParameterValue { node_id, .. } => vec![node_id.clone()],
            GraphEditCommand::AddRoute { route } => {
                vec![route.source_node_id.clone(), route.target_node_id.clone()]
            }
            GraphEditCommand::RemoveRoute { .. } => vec![],
            GraphEditCommand::AssignNodeToBus { node_id, .. } => vec![node_id.clone()],
            GraphEditCommand::ClearNodeBusAssignment { node_id } => vec![node_id.clone()],
        },
        TypedCommand::Performance(_) => vec![],
    }
}

fn generate_diff_summary(command: &TypedCommand, _session: &SessionDocument) -> DiffSummary {
    let (description, affected_node_ids) = match command {
        TypedCommand::GraphEdit(GraphEditCommand::AddNode { node }) => (
            format!(
                "Added {} node",
                format!("{:?}", node.node_type).to_lowercase()
            ),
            vec![node.id.clone()],
        ),
        TypedCommand::GraphEdit(GraphEditCommand::RemoveNode { node_id }) => {
            (format!("Removed node {}", node_id), vec![node_id.clone()])
        }
        TypedCommand::GraphEdit(GraphEditCommand::SetNodeEnabled { node_id, enabled }) => (
            format!(
                "Set node {} {}",
                node_id,
                if *enabled { "enabled" } else { "disabled" }
            ),
            vec![node_id.clone()],
        ),
        TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
            node_id,
            parameter_id,
            value,
        }) => (
            format!("Set parameter {} to {} on {}", parameter_id, value, node_id),
            vec![node_id.clone()],
        ),
        TypedCommand::GraphEdit(GraphEditCommand::AddRoute { route }) => (
            format!(
                "Added route from {} to {}",
                route.source_node_id, route.target_node_id
            ),
            vec![route.source_node_id.clone(), route.target_node_id.clone()],
        ),
        TypedCommand::GraphEdit(GraphEditCommand::RemoveRoute { route_id }) => {
            (format!("Removed route {}", route_id), vec![])
        }
        TypedCommand::GraphEdit(GraphEditCommand::AssignNodeToBus { node_id, bus_id }) => (
            format!("Assigned node {} to bus {}", node_id, bus_id),
            vec![node_id.clone()],
        ),
        TypedCommand::GraphEdit(GraphEditCommand::ClearNodeBusAssignment { node_id }) => (
            format!("Cleared bus assignment for node {}", node_id),
            vec![node_id.clone()],
        ),
        TypedCommand::Performance(PerformanceCommand::RecallScene { scene_id }) => {
            (format!("Recalled scene {}", scene_id), vec![])
        }
        TypedCommand::Performance(PerformanceCommand::SaveVariation { name, .. }) => {
            (format!("Saved variation '{}'", name), vec![])
        }
        TypedCommand::Performance(PerformanceCommand::RestoreVariation { variation_id }) => {
            (format!("Restored variation {}", variation_id), vec![])
        }
    };

    DiffSummary {
        description,
        affected_node_ids,
        before_snippet: String::new(),
        after_snippet: String::new(),
    }
}

fn build_default_session() -> SessionDocument {
    let scene_id = new_id();
    let source_node_id = new_id();
    let source_out_port_id = new_id();
    let master_node_id = new_id();
    let master_in_port_id = new_id();
    let bus_id = new_id();
    let parameter_id = new_id();
    let macro_id = new_id();

    SessionDocument {
        title: "Default Scrysynth Session".to_string(),
        audio_runtime: AudioRuntimeState {
            lifecycle: AudioRuntimeLifecycle::Idle,
            health: AudioRuntimeHealth::Unknown,
            sample_rate_hz: None,
            block_size: None,
            active_patch_id: None,
            last_error: None,
            panic_recovery_count: 0,
        },
        nodes: vec![
            Node {
                id: source_node_id.clone(),
                node_type: NodeType::Source,
                ports: vec![Port {
                    id: source_out_port_id.clone(),
                    name: "main_out".to_string(),
                    direction: PortDirection::Output,
                    signal_type: SignalType::Audio,
                }],
                parameters: vec![ParameterValue {
                    id: parameter_id.clone(),
                    name: "level".to_string(),
                    value: 0.8,
                    default_value: 0.8,
                    min_value: 0.0,
                    max_value: 1.0,
                    unit: "linear".to_string(),
                }],
                runtime_target: Some("audio/source/default".to_string()),
                scene_membership: vec![scene_id.clone()],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::Shared,
                    is_locked: false,
                },
                enabled: true,
                audio_primitive: Some(AudioPrimitive::Source(AudioSourceNode {
                    source_type: AudioSourceType::Oscillator,
                    channel_mode: ChannelMode::Mono,
                    bus_target_id: Some(bus_id.clone()),
                })),
            },
            Node {
                id: master_node_id.clone(),
                node_type: NodeType::Output,
                ports: vec![Port {
                    id: master_in_port_id.clone(),
                    name: "master_in".to_string(),
                    direction: PortDirection::Input,
                    signal_type: SignalType::Audio,
                }],
                parameters: vec![],
                runtime_target: Some("audio/output/master".to_string()),
                scene_membership: vec![scene_id.clone()],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::User,
                    is_locked: false,
                },
                enabled: true,
                audio_primitive: Some(AudioPrimitive::Output(AudioOutputNode {
                    output_type: AudioOutputType::Master,
                    channels: 2,
                    bus_target_id: Some(bus_id.clone()),
                })),
            },
        ],
        routes: vec![Route {
            id: new_id(),
            source_node_id,
            source_port_id: source_out_port_id,
            target_node_id: master_node_id.clone(),
            target_port_id: master_in_port_id,
            bus_id: Some(bus_id.clone()),
        }],
        buses: vec![Bus {
            id: bus_id,
            name: "master_bus".to_string(),
            channels: 2,
            bus_type: AudioBusType::Main,
            is_enabled: true,
        }],
        macros: vec![MacroDefinition {
            id: macro_id.clone(),
            name: "energy".to_string(),
            target_parameter_ids: vec![parameter_id.clone()],
            range_start: 0.0,
            range_end: 1.0,
            targets: vec![],
        }],
        scenes: vec![SceneDefinition {
            id: scene_id.clone(),
            name: "intro".to_string(),
            active_node_ids: vec![master_node_id],
            macro_overrides: vec![MacroOverride {
                macro_id: macro_id.clone(),
                value: 0.65,
            }],
        }],
        variations: vec![VariationDefinition {
            id: new_id(),
            name: "intro-alt".to_string(),
            scene_id,
            parameter_overrides: vec![ParameterOverride {
                parameter_id,
                value: 0.55,
            }],
        }],
        ownership_rules: vec![OwnershipRule {
            id: new_id(),
            scope: "graph:master".to_string(),
            controller: ControllerKind::Shared,
            can_override: true,
        }],
        runtime_status: vec![
            RuntimeStatusRef {
                id: new_id(),
                runtime: RuntimeKind::Audio,
                status: RuntimeConnectionState::Disconnected,
                target_id: Some("audio-runtime".to_string()),
                last_error: None,
            },
            RuntimeStatusRef {
                id: new_id(),
                runtime: RuntimeKind::Visual,
                status: RuntimeConnectionState::Disconnected,
                target_id: Some("visual-runtime".to_string()),
                last_error: None,
            },
            RuntimeStatusRef {
                id: new_id(),
                runtime: RuntimeKind::Agent,
                status: RuntimeConnectionState::Disconnected,
                target_id: Some("agent-runtime".to_string()),
                last_error: None,
            },
        ],
        ..SessionDocument::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_store_create_default_session_returns_seeded_graph() {
        let store = SessionStore::new_default();
        let session = store.current();

        assert!(!session.nodes.is_empty());
        assert!(!session.routes.is_empty());
        assert!(!session.buses.is_empty());
        assert!(!session.macros.is_empty());
        assert!(!session.scenes.is_empty());
        assert!(!session.variations.is_empty());
        assert!(!session.ownership_rules.is_empty());
        assert!(!session.runtime_status.is_empty());
    }

    #[test]
    fn session_store_get_current_session_returns_same_session_after_replace() {
        let mut store = SessionStore::new_default();
        let mut replacement = SessionDocument::default();
        replacement.title = "Replacement Session".to_string();
        store.replace_current(replacement.clone());

        assert_eq!(store.current(), replacement);
    }
}
