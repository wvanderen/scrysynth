use crate::domain::session::{
    new_id, AudioBusType, AudioOutputNode, AudioOutputType, AudioPrimitive, AudioRuntimeHealth,
    AudioRuntimeLifecycle, AudioRuntimeState, AudioSourceNode, AudioSourceType, Bus, ChannelMode,
    ControllerKind, MacroDefinition, MacroOverride, Node, NodeType, OwnershipAssignment,
    OwnershipRule, ParameterOverride, ParameterValue, Port, PortDirection, Route,
    RuntimeConnectionState, RuntimeKind, RuntimeStatusRef, SceneDefinition, SessionDocument,
    SignalType, VariationDefinition,
};

#[derive(Clone, Debug)]
pub struct SessionStore {
    current: SessionDocument,
}

impl SessionStore {
    pub fn new_default() -> Self {
        Self {
            current: build_default_session(),
        }
    }

    pub fn current(&self) -> SessionDocument {
        self.current.clone()
    }

    pub fn replace_current(&mut self, session: SessionDocument) {
        self.current = session;
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
