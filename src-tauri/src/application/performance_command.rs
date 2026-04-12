use thiserror::Error;

use crate::application::session_store::SessionStore;
use crate::domain::session::{
    new_id, MacroOverride, MacroTarget, ParameterOverride, PerformanceCommand, SessionDocument,
    VariationDefinition,
};

#[derive(Debug, Error, PartialEq)]
pub enum PerformanceCommandError {
    #[error("scene '{scene_id}' was not found")]
    MissingScene { scene_id: String },
    #[error("variation '{variation_id}' was not found")]
    MissingVariation { variation_id: String },
    #[error("parameter '{parameter_id}' is out of range on node '{node_id}'")]
    ParameterOutOfRange {
        node_id: String,
        parameter_id: String,
    },
}

pub fn apply_performance_command(
    store: &mut SessionStore,
    command: PerformanceCommand,
) -> Result<SessionDocument, PerformanceCommandError> {
    store.mutate_current(|session| match command {
        PerformanceCommand::RecallScene { scene_id } => recall_scene(session, &scene_id),
        PerformanceCommand::SaveVariation { name, scene_id } => {
            save_variation(session, &name, &scene_id)
        }
        PerformanceCommand::RestoreVariation { variation_id } => {
            restore_variation(session, &variation_id)
        }
    })
}

fn recall_scene(
    session: &mut SessionDocument,
    scene_id: &str,
) -> Result<(), PerformanceCommandError> {
    let scene = session
        .scenes
        .iter()
        .find(|scene| scene.id == scene_id)
        .ok_or_else(|| PerformanceCommandError::MissingScene {
            scene_id: scene_id.to_string(),
        })?
        .clone();

    let active_ids: std::collections::HashSet<&str> =
        scene.active_node_ids.iter().map(|id| id.as_str()).collect();

    for node in &mut session.nodes {
        node.enabled = active_ids.contains(node.id.as_str());
    }

    for macro_override in &scene.macro_overrides {
        apply_macro_override(session, macro_override)?;
    }

    Ok(())
}

fn apply_macro_override(
    session: &mut SessionDocument,
    macro_override: &MacroOverride,
) -> Result<(), PerformanceCommandError> {
    let macro_def = match session
        .macros
        .iter()
        .find(|m| m.id == macro_override.macro_id)
    {
        Some(m) => m.clone(),
        None => return Ok(()),
    };

    let scaled_value = macro_def.range_start
        + (macro_override.value * (macro_def.range_end - macro_def.range_start));

    if !macro_def.targets.is_empty() {
        for target in &macro_def.targets {
            match target {
                MacroTarget::AudioParameter {
                    node_id,
                    parameter_id,
                } => {
                    for node in &mut session.nodes {
                        if node.id == *node_id {
                            for parameter in &mut node.parameters {
                                if parameter.id == *parameter_id {
                                    let clamped = scaled_value
                                        .clamp(parameter.min_value, parameter.max_value);
                                    parameter.value = clamped;
                                }
                            }
                        }
                    }
                }
                MacroTarget::VisualParameter {
                    element_id: _,
                    parameter_id: _,
                } => {}
            }
        }
    } else {
        for target_param_id in &macro_def.target_parameter_ids {
            for node in &mut session.nodes {
                for parameter in &mut node.parameters {
                    if parameter.id == *target_param_id {
                        let clamped = scaled_value.clamp(parameter.min_value, parameter.max_value);
                        parameter.value = clamped;
                    }
                }
            }
        }
    }

    Ok(())
}

fn save_variation(
    session: &mut SessionDocument,
    name: &str,
    scene_id: &str,
) -> Result<(), PerformanceCommandError> {
    if !session.scenes.iter().any(|scene| scene.id == scene_id) {
        return Err(PerformanceCommandError::MissingScene {
            scene_id: scene_id.to_string(),
        });
    }

    let scene = session
        .scenes
        .iter()
        .find(|scene| scene.id == scene_id)
        .expect("scene existence checked above");

    let mut parameter_overrides = Vec::new();

    for node_id in &scene.active_node_ids {
        if let Some(node) = session.nodes.iter().find(|n| &n.id == node_id) {
            for parameter in &node.parameters {
                parameter_overrides.push(ParameterOverride {
                    parameter_id: parameter.id.clone(),
                    value: parameter.value,
                });
            }
        }
    }

    parameter_overrides.sort_by(|a, b| a.parameter_id.cmp(&b.parameter_id));

    let variation = VariationDefinition {
        id: new_id(),
        name: name.to_string(),
        scene_id: scene_id.to_string(),
        parameter_overrides,
    };

    session.variations.push(variation);
    session
        .variations
        .sort_by(|a, b| a.scene_id.cmp(&b.scene_id).then(a.name.cmp(&b.name)));

    Ok(())
}

fn restore_variation(
    session: &mut SessionDocument,
    variation_id: &str,
) -> Result<(), PerformanceCommandError> {
    let variation = session
        .variations
        .iter()
        .find(|v| v.id == variation_id)
        .ok_or_else(|| PerformanceCommandError::MissingVariation {
            variation_id: variation_id.to_string(),
        })?
        .clone();

    for override_param in &variation.parameter_overrides {
        let mut found = false;
        for node in &mut session.nodes {
            for parameter in &mut node.parameters {
                if parameter.id == override_param.parameter_id {
                    if override_param.value < parameter.min_value
                        || override_param.value > parameter.max_value
                    {
                        return Err(PerformanceCommandError::ParameterOutOfRange {
                            node_id: node.id.clone(),
                            parameter_id: parameter.id.clone(),
                        });
                    }
                    parameter.value = override_param.value;
                    found = true;
                }
            }
        }
        let _ = found;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::session::{
        AudioBusType, AudioOutputNode, AudioOutputType, AudioPrimitive, AudioSourceNode,
        AudioSourceType, Bus, ChannelMode, ControllerKind, MacroDefinition, MacroOverride, Node,
        NodeType, OwnershipAssignment, ParameterValue, Port, PortDirection, Route, SceneDefinition,
        SignalType,
    };

    fn test_session() -> SessionDocument {
        let scene_a_id = "scene-a";
        let scene_b_id = "scene-b";
        let source_id = "node-source";
        let output_id = "node-output";
        let param_id = "param-level";
        let macro_id = "macro-energy";

        SessionDocument {
            title: "Performance Test".to_string(),
            nodes: vec![
                Node {
                    id: source_id.to_string(),
                    node_type: NodeType::Source,
                    ports: vec![Port {
                        id: "port-source-out".to_string(),
                        name: "main_out".to_string(),
                        direction: PortDirection::Output,
                        signal_type: SignalType::Audio,
                    }],
                    parameters: vec![ParameterValue {
                        id: param_id.to_string(),
                        name: "level".to_string(),
                        value: 0.8,
                        default_value: 0.8,
                        min_value: 0.0,
                        max_value: 1.0,
                        unit: "linear".to_string(),
                    }],
                    runtime_target: Some("audio/source/oscillator".to_string()),
                    scene_membership: vec![scene_a_id.to_string()],
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
                    id: output_id.to_string(),
                    node_type: NodeType::Output,
                    ports: vec![Port {
                        id: "port-output-in".to_string(),
                        name: "master_in".to_string(),
                        direction: PortDirection::Input,
                        signal_type: SignalType::Audio,
                    }],
                    parameters: vec![],
                    runtime_target: Some("audio/output/master".to_string()),
                    scene_membership: vec![scene_a_id.to_string(), scene_b_id.to_string()],
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
            routes: vec![Route {
                id: "route-1".to_string(),
                source_node_id: source_id.to_string(),
                source_port_id: "port-source-out".to_string(),
                target_node_id: output_id.to_string(),
                target_port_id: "port-output-in".to_string(),
                bus_id: Some("bus-main".to_string()),
            }],
            buses: vec![Bus {
                id: "bus-main".to_string(),
                name: "main".to_string(),
                channels: 2,
                bus_type: AudioBusType::Main,
                is_enabled: true,
            }],
            macros: vec![MacroDefinition {
                id: macro_id.to_string(),
                name: "energy".to_string(),
                target_parameter_ids: vec![param_id.to_string()],
                range_start: 0.0,
                range_end: 1.0,
                targets: vec![],
            }],
            scenes: vec![
                SceneDefinition {
                    id: scene_a_id.to_string(),
                    name: "Scene A".to_string(),
                    active_node_ids: vec![source_id.to_string(), output_id.to_string()],
                    macro_overrides: vec![MacroOverride {
                        macro_id: macro_id.to_string(),
                        value: 0.5,
                    }],
                },
                SceneDefinition {
                    id: scene_b_id.to_string(),
                    name: "Scene B".to_string(),
                    active_node_ids: vec![output_id.to_string()],
                    macro_overrides: vec![],
                },
            ],
            variations: vec![],
            ..SessionDocument::default()
        }
    }

    #[test]
    fn recall_scene_enables_scene_nodes_and_disables_others() {
        let mut store = SessionStore::new_default();
        store.replace_current(test_session());

        let updated = apply_performance_command(
            &mut store,
            PerformanceCommand::RecallScene {
                scene_id: "scene-b".to_string(),
            },
        )
        .expect("recall succeeds");

        assert!(!updated
            .nodes
            .iter()
            .any(|n| n.id == "node-source" && n.enabled));
        assert!(updated
            .nodes
            .iter()
            .any(|n| n.id == "node-output" && n.enabled));
    }

    #[test]
    fn recall_scene_applies_macro_overrides() {
        let mut store = SessionStore::new_default();
        store.replace_current(test_session());

        let updated = apply_performance_command(
            &mut store,
            PerformanceCommand::RecallScene {
                scene_id: "scene-a".to_string(),
            },
        )
        .expect("recall succeeds");

        let source = updated
            .nodes
            .iter()
            .find(|n| n.id == "node-source")
            .unwrap();
        let level = &source.parameters[0];
        assert_eq!(level.value, 0.5);
    }

    #[test]
    fn recall_scene_rejects_missing_scene() {
        let mut store = SessionStore::new_default();
        store.replace_current(test_session());
        let original = store.current();

        let result = apply_performance_command(
            &mut store,
            PerformanceCommand::RecallScene {
                scene_id: "nonexistent".to_string(),
            },
        );

        assert!(result.is_err());
        assert_eq!(store.current().nodes, original.nodes);
    }

    #[test]
    fn save_variation_snapshots_current_parameters() {
        let mut store = SessionStore::new_default();
        store.replace_current(test_session());

        let updated = apply_performance_command(
            &mut store,
            PerformanceCommand::SaveVariation {
                name: "soft".to_string(),
                scene_id: "scene-a".to_string(),
            },
        )
        .expect("save succeeds");

        assert_eq!(updated.variations.len(), 1);
        let variation = &updated.variations[0];
        assert_eq!(variation.name, "soft");
        assert_eq!(variation.scene_id, "scene-a");
        assert!(variation
            .parameter_overrides
            .iter()
            .any(|p| p.parameter_id == "param-level" && p.value == 0.8));
    }

    #[test]
    fn save_variation_rejects_missing_scene() {
        let mut store = SessionStore::new_default();
        store.replace_current(test_session());

        let result = apply_performance_command(
            &mut store,
            PerformanceCommand::SaveVariation {
                name: "ghost".to_string(),
                scene_id: "nonexistent".to_string(),
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn restore_variation_applies_parameter_overrides() {
        let mut store = SessionStore::new_default();
        let mut session = test_session();
        session.variations.push(VariationDefinition {
            id: "var-quiet".to_string(),
            name: "quiet".to_string(),
            scene_id: "scene-a".to_string(),
            parameter_overrides: vec![ParameterOverride {
                parameter_id: "param-level".to_string(),
                value: 0.3,
            }],
        });
        store.replace_current(session);

        let updated = apply_performance_command(
            &mut store,
            PerformanceCommand::RestoreVariation {
                variation_id: "var-quiet".to_string(),
            },
        )
        .expect("restore succeeds");

        let source = updated
            .nodes
            .iter()
            .find(|n| n.id == "node-source")
            .unwrap();
        assert_eq!(source.parameters[0].value, 0.3);
    }

    #[test]
    fn restore_variation_rejects_missing_variation() {
        let mut store = SessionStore::new_default();
        store.replace_current(test_session());
        let original = store.current();

        let result = apply_performance_command(
            &mut store,
            PerformanceCommand::RestoreVariation {
                variation_id: "nonexistent".to_string(),
            },
        );

        assert!(result.is_err());
        let current = store.current();
        for (cn, on) in current.nodes.iter().zip(original.nodes.iter()) {
            assert_eq!(cn.parameters, on.parameters);
        }
    }

    #[test]
    fn recall_and_variation_round_trip() {
        let mut store = SessionStore::new_default();
        store.replace_current(test_session());

        apply_performance_command(
            &mut store,
            PerformanceCommand::RecallScene {
                scene_id: "scene-a".to_string(),
            },
        )
        .expect("recall scene-a");

        let with_variation = apply_performance_command(
            &mut store,
            PerformanceCommand::SaveVariation {
                name: "baseline".to_string(),
                scene_id: "scene-a".to_string(),
            },
        )
        .expect("save variation");

        let variation_id = with_variation.variations[0].id.clone();

        apply_performance_command(
            &mut store,
            PerformanceCommand::RestoreVariation {
                variation_id: "nonexistent".to_string(),
            },
        )
        .unwrap_err();

        apply_performance_command(
            &mut store,
            PerformanceCommand::RecallScene {
                scene_id: "scene-b".to_string(),
            },
        )
        .expect("recall scene-b");

        let after_b = store.current();
        assert!(!after_b
            .nodes
            .iter()
            .any(|n| n.id == "node-source" && n.enabled));

        apply_performance_command(
            &mut store,
            PerformanceCommand::RecallScene {
                scene_id: "scene-a".to_string(),
            },
        )
        .expect("recall scene-a again");

        let restored = apply_performance_command(
            &mut store,
            PerformanceCommand::RestoreVariation { variation_id },
        )
        .expect("restore variation");

        let source = restored
            .nodes
            .iter()
            .find(|n| n.id == "node-source")
            .unwrap();
        assert_eq!(source.parameters[0].value, 0.5);
    }
}
