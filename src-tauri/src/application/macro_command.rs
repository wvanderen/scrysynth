use thiserror::Error;

use crate::application::session_store::SessionStore;
use crate::domain::session::{MacroCommand, MacroDefinition, MacroTarget, SessionDocument};

#[derive(Debug, Error, PartialEq)]
pub enum MacroCommandError {
    #[error("macro '{macro_id}' already exists")]
    DuplicateMacro { macro_id: String },
    #[error("macro '{macro_id}' not found")]
    MissingMacro { macro_id: String },
    #[error("parameter '{parameter_id}' is out of range on node '{node_id}'")]
    ParameterOutOfRange {
        node_id: String,
        parameter_id: String,
    },
    #[error("visual runtime reconciliation failed: {message}")]
    VisualRuntimeReconcile { message: String },
    #[error("audio runtime reconciliation failed: {message}")]
    AudioRuntimeReconcile { message: String },
}

pub fn apply_macro_command(
    store: &mut SessionStore,
    command: MacroCommand,
) -> Result<SessionDocument, MacroCommandError> {
    let visual_macro_value = match &command {
        MacroCommand::SetMacroValue { macro_id, value } => Some((macro_id.clone(), *value)),
        _ => None,
    };
    let updated = store.mutate_current(|session| match command {
        MacroCommand::CreateMacro { definition } => create_macro(session, definition),
        MacroCommand::UpdateMacro {
            macro_id,
            name,
            targets,
            range_start,
            range_end,
        } => update_macro(session, &macro_id, name, targets, range_start, range_end),
        MacroCommand::RemoveMacro { macro_id } => remove_macro(session, &macro_id),
        MacroCommand::SetMacroValue { macro_id, value } => {
            set_macro_value(session, &macro_id, value)
        }
    })?;

    if let Some((macro_id, value)) = visual_macro_value {
        let _ = store
            .reconcile_audio_macro_value(&macro_id, value)
            .map_err(|err| MacroCommandError::AudioRuntimeReconcile {
                message: err.to_string(),
            })?;
        store
            .reconcile_visual_macro_value(&macro_id, value)
            .map_err(|err| MacroCommandError::VisualRuntimeReconcile {
                message: err.to_string(),
            })
    } else {
        Ok(updated)
    }
}

fn create_macro(
    session: &mut SessionDocument,
    definition: MacroDefinition,
) -> Result<(), MacroCommandError> {
    if session.macros.iter().any(|m| m.id == definition.id) {
        return Err(MacroCommandError::DuplicateMacro {
            macro_id: definition.id.clone(),
        });
    }

    session.macros.push(definition);
    session.macros.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(())
}

fn update_macro(
    session: &mut SessionDocument,
    macro_id: &str,
    name: Option<String>,
    targets: Option<Vec<MacroTarget>>,
    range_start: Option<f64>,
    range_end: Option<f64>,
) -> Result<(), MacroCommandError> {
    let macro_def = session
        .macros
        .iter_mut()
        .find(|m| m.id == macro_id)
        .ok_or_else(|| MacroCommandError::MissingMacro {
            macro_id: macro_id.to_string(),
        })?;

    if let Some(n) = name {
        macro_def.name = n;
    }
    if let Some(t) = targets {
        macro_def.targets = t;
    }
    if let Some(rs) = range_start {
        macro_def.range_start = rs;
    }
    if let Some(re) = range_end {
        macro_def.range_end = re;
    }

    Ok(())
}

fn remove_macro(session: &mut SessionDocument, macro_id: &str) -> Result<(), MacroCommandError> {
    let index = session
        .macros
        .iter()
        .position(|m| m.id == macro_id)
        .ok_or_else(|| MacroCommandError::MissingMacro {
            macro_id: macro_id.to_string(),
        })?;

    session.macros.remove(index);

    for scene in &mut session.scenes {
        scene.macro_overrides.retain(|mo| mo.macro_id != macro_id);
    }

    Ok(())
}

fn set_macro_value(
    session: &mut SessionDocument,
    macro_id: &str,
    value: f64,
) -> Result<(), MacroCommandError> {
    let macro_def = session
        .macros
        .iter()
        .find(|m| m.id == macro_id)
        .ok_or_else(|| MacroCommandError::MissingMacro {
            macro_id: macro_id.to_string(),
        })?
        .clone();

    let clamped_input = value.clamp(0.0, 1.0);
    let scaled_value =
        macro_def.range_start + (clamped_input * (macro_def.range_end - macro_def.range_start));

    if !macro_def.targets.is_empty() {
        for target in &macro_def.targets {
            match target {
                MacroTarget::AudioParameter {
                    node_id,
                    parameter_id,
                } => {
                    apply_audio_parameter(session, node_id, parameter_id, scaled_value)?;
                }
                MacroTarget::VisualParameter {
                    element_id: _,
                    parameter_id: _,
                } => {}
            }
        }
    } else if !macro_def.target_parameter_ids.is_empty() {
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

pub fn apply_audio_parameter(
    session: &mut SessionDocument,
    node_id: &str,
    parameter_id: &str,
    scaled_value: f64,
) -> Result<(), MacroCommandError> {
    for node in &mut session.nodes {
        if node.id == node_id {
            for parameter in &mut node.parameters {
                if parameter.id == parameter_id {
                    let clamped = scaled_value.clamp(parameter.min_value, parameter.max_value);
                    parameter.value = clamped;
                    return Ok(());
                }
            }
            return Err(MacroCommandError::ParameterOutOfRange {
                node_id: node_id.to_string(),
                parameter_id: parameter_id.to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::session::{
        AudioBusType, AudioPrimitive, AudioSourceNode, AudioSourceType, Bus, ChannelMode,
        ControllerKind, MacroOverride, Node, NodeType, OwnershipAssignment, ParameterValue, Port,
        PortDirection, Route, SceneDefinition, SignalType,
    };

    fn test_session() -> SessionDocument {
        SessionDocument {
            title: "Macro Test".to_string(),
            nodes: vec![Node {
                id: "node-src".to_string(),
                node_type: NodeType::Source,
                ports: vec![Port {
                    id: "port-src-out".to_string(),
                    name: "main_out".to_string(),
                    direction: PortDirection::Output,
                    signal_type: SignalType::Audio,
                }],
                parameters: vec![ParameterValue {
                    id: "param-freq".to_string(),
                    name: "frequency".to_string(),
                    value: 440.0,
                    default_value: 440.0,
                    min_value: 20.0,
                    max_value: 20000.0,
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
                    bus_target_id: None,
                })),
            }],
            routes: vec![Route {
                id: "route-1".to_string(),
                source_node_id: "node-src".to_string(),
                source_port_id: "port-src-out".to_string(),
                target_node_id: "node-out".to_string(),
                target_port_id: "port-out-in".to_string(),
                bus_id: None,
            }],
            buses: vec![Bus {
                id: "bus-main".to_string(),
                name: "main".to_string(),
                channels: 2,
                bus_type: AudioBusType::Main,
                is_enabled: true,
            }],
            macros: vec![],
            scenes: vec![SceneDefinition {
                id: "scene-a".to_string(),
                name: "Scene A".to_string(),
                active_node_ids: vec!["node-src".to_string()],
                macro_overrides: vec![],
            }],
            ..SessionDocument::default()
        }
    }

    #[test]
    fn create_macro_adds_to_session() {
        let mut store = SessionStore::new_default();
        store.replace_current(test_session());

        let updated = apply_macro_command(
            &mut store,
            MacroCommand::CreateMacro {
                definition: MacroDefinition {
                    id: "macro-1".to_string(),
                    name: "test".to_string(),
                    target_parameter_ids: vec![],
                    range_start: 0.0,
                    range_end: 1.0,
                    targets: vec![MacroTarget::AudioParameter {
                        node_id: "node-src".to_string(),
                        parameter_id: "param-freq".to_string(),
                    }],
                },
            },
        )
        .expect("create succeeds");

        assert_eq!(updated.macros.len(), 1);
        assert_eq!(updated.macros[0].name, "test");
    }

    #[test]
    fn create_macro_rejects_duplicate_id() {
        let mut store = SessionStore::new_default();
        let mut session = test_session();
        session.macros.push(MacroDefinition {
            id: "macro-1".to_string(),
            name: "existing".to_string(),
            target_parameter_ids: vec![],
            range_start: 0.0,
            range_end: 1.0,
            targets: vec![],
        });
        store.replace_current(session);

        let result = apply_macro_command(
            &mut store,
            MacroCommand::CreateMacro {
                definition: MacroDefinition {
                    id: "macro-1".to_string(),
                    name: "dup".to_string(),
                    target_parameter_ids: vec![],
                    range_start: 0.0,
                    range_end: 1.0,
                    targets: vec![],
                },
            },
        );

        assert!(matches!(
            result,
            Err(MacroCommandError::DuplicateMacro { .. })
        ));
    }

    #[test]
    fn update_macro_changes_fields() {
        let mut store = SessionStore::new_default();
        let mut session = test_session();
        session.macros.push(MacroDefinition {
            id: "macro-1".to_string(),
            name: "old".to_string(),
            target_parameter_ids: vec![],
            range_start: 0.0,
            range_end: 1.0,
            targets: vec![],
        });
        store.replace_current(session);

        let updated = apply_macro_command(
            &mut store,
            MacroCommand::UpdateMacro {
                macro_id: "macro-1".to_string(),
                name: Some("new".to_string()),
                targets: None,
                range_start: Some(0.2),
                range_end: Some(0.8),
            },
        )
        .expect("update succeeds");

        let m = &updated.macros[0];
        assert_eq!(m.name, "new");
        assert!((m.range_start - 0.2).abs() < f64::EPSILON);
        assert!((m.range_end - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn remove_macro_cleans_scene_overrides() {
        let mut store = SessionStore::new_default();
        let mut session = test_session();
        session.macros.push(MacroDefinition {
            id: "macro-1".to_string(),
            name: "to-remove".to_string(),
            target_parameter_ids: vec![],
            range_start: 0.0,
            range_end: 1.0,
            targets: vec![],
        });
        session.scenes[0].macro_overrides.push(MacroOverride {
            macro_id: "macro-1".to_string(),
            value: 0.5,
        });
        store.replace_current(session);

        let updated = apply_macro_command(
            &mut store,
            MacroCommand::RemoveMacro {
                macro_id: "macro-1".to_string(),
            },
        )
        .expect("remove succeeds");

        assert!(updated.macros.is_empty());
        assert!(updated.scenes[0].macro_overrides.is_empty());
    }

    #[test]
    fn set_macro_value_applies_to_audio_target() {
        let mut store = SessionStore::new_default();
        let mut session = test_session();
        session.macros.push(MacroDefinition {
            id: "macro-1".to_string(),
            name: "freq-ctrl".to_string(),
            target_parameter_ids: vec![],
            range_start: 100.0,
            range_end: 1000.0,
            targets: vec![MacroTarget::AudioParameter {
                node_id: "node-src".to_string(),
                parameter_id: "param-freq".to_string(),
            }],
        });
        store.replace_current(session);

        let updated = apply_macro_command(
            &mut store,
            MacroCommand::SetMacroValue {
                macro_id: "macro-1".to_string(),
                value: 0.5,
            },
        )
        .expect("set value succeeds");

        let freq = updated.nodes[0]
            .parameters
            .iter()
            .find(|p| p.id == "param-freq")
            .unwrap();
        assert!((freq.value - 550.0).abs() < 0.01);
    }

    #[test]
    fn set_macro_value_backward_compat_flat_ids() {
        let mut store = SessionStore::new_default();
        let mut session = test_session();
        session.macros.push(MacroDefinition {
            id: "macro-old".to_string(),
            name: "legacy".to_string(),
            target_parameter_ids: vec!["param-freq".to_string()],
            range_start: 200.0,
            range_end: 800.0,
            targets: vec![],
        });
        store.replace_current(session);

        let updated = apply_macro_command(
            &mut store,
            MacroCommand::SetMacroValue {
                macro_id: "macro-old".to_string(),
                value: 1.0,
            },
        )
        .expect("set value succeeds");

        let freq = updated.nodes[0]
            .parameters
            .iter()
            .find(|p| p.id == "param-freq")
            .unwrap();
        assert!((freq.value - 800.0).abs() < 0.01);
    }

    #[test]
    fn set_macro_value_with_multiple_targets() {
        let mut store = SessionStore::new_default();
        let mut session = test_session();
        session.macros.push(MacroDefinition {
            id: "macro-multi".to_string(),
            name: "multi".to_string(),
            target_parameter_ids: vec![],
            range_start: 20.0,
            range_end: 20000.0,
            targets: vec![
                MacroTarget::AudioParameter {
                    node_id: "node-src".to_string(),
                    parameter_id: "param-freq".to_string(),
                },
                MacroTarget::VisualParameter {
                    element_id: "element-1".to_string(),
                    parameter_id: "param-color".to_string(),
                },
            ],
        });
        store.replace_current(session);

        let updated = apply_macro_command(
            &mut store,
            MacroCommand::SetMacroValue {
                macro_id: "macro-multi".to_string(),
                value: 0.5,
            },
        )
        .expect("set value succeeds");

        let freq = updated.nodes[0]
            .parameters
            .iter()
            .find(|p| p.id == "param-freq")
            .unwrap();
        let expected = 20.0 + (0.5 * (20000.0 - 20.0));
        assert!((freq.value - expected).abs() < 0.01);
    }
}
