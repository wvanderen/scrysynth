use scrysynth_lib::application::macro_command::{apply_macro_command, MacroCommandError};
use scrysynth_lib::application::session_store::SessionStore;
use scrysynth_lib::domain::session::{
    AudioBusType, AudioPrimitive, AudioSourceNode, AudioSourceType, Bus, ChannelMode,
    ControllerKind, MacroCommand, MacroDefinition, MacroOverride, MacroTarget, Node, NodeType,
    OwnershipAssignment, ParameterValue, Port, PortDirection, Route, SceneDefinition,
    SessionDocument, SignalType,
};

fn macro_test_session() -> SessionDocument {
    SessionDocument {
        title: "Macro Integration".to_string(),
        nodes: vec![
            Node {
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
                scene_membership: vec!["scene-a".to_string()],
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
            },
            Node {
                id: "node-fx".to_string(),
                node_type: NodeType::Effect,
                ports: vec![],
                parameters: vec![ParameterValue {
                    id: "param-mix".to_string(),
                    name: "mix".to_string(),
                    value: 0.5,
                    default_value: 0.5,
                    min_value: 0.0,
                    max_value: 1.0,
                    unit: "ratio".to_string(),
                }],
                runtime_target: None,
                scene_membership: vec![],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::Shared,
                    is_locked: false,
                },
                enabled: true,
                audio_primitive: None,
            },
        ],
        routes: vec![Route {
            id: "route-1".to_string(),
            source_node_id: "node-src".to_string(),
            source_port_id: "port-src-out".to_string(),
            target_node_id: "node-fx".to_string(),
            target_port_id: "port-fx-in".to_string(),
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
fn macro_create_adds_audio_parameter_targets() {
    let mut store = SessionStore::new_default();
    store.replace_current(macro_test_session());

    let updated = apply_macro_command(
        &mut store,
        MacroCommand::CreateMacro {
            definition: MacroDefinition {
                id: "macro-1".to_string(),
                name: "freq ctrl".to_string(),
                target_parameter_ids: vec![],
                range_start: 100.0,
                range_end: 5000.0,
                targets: vec![MacroTarget::AudioParameter {
                    node_id: "node-src".to_string(),
                    parameter_id: "param-freq".to_string(),
                }],
            },
        },
    )
    .expect("create succeeds");

    assert_eq!(updated.macros.len(), 1);
    assert_eq!(updated.macros[0].name, "freq ctrl");
    assert_eq!(updated.macros[0].targets.len(), 1);
}

#[test]
fn macro_create_rejects_duplicate_id() {
    let mut store = SessionStore::new_default();
    let mut session = macro_test_session();
    session.macros.push(MacroDefinition {
        id: "macro-dup".to_string(),
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
                id: "macro-dup".to_string(),
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
    assert_eq!(store.current().macros.len(), 1);
}

#[test]
fn macro_update_changes_name_and_range() {
    let mut store = SessionStore::new_default();
    let mut session = macro_test_session();
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
            name: Some("new name".to_string()),
            targets: None,
            range_start: Some(0.2),
            range_end: Some(0.8),
        },
    )
    .expect("update succeeds");

    assert_eq!(updated.macros[0].name, "new name");
    assert!((updated.macros[0].range_start - 0.2).abs() < f64::EPSILON);
    assert!((updated.macros[0].range_end - 0.8).abs() < f64::EPSILON);
}

#[test]
fn macro_update_rejects_missing_id() {
    let mut store = SessionStore::new_default();
    store.replace_current(macro_test_session());

    let result = apply_macro_command(
        &mut store,
        MacroCommand::UpdateMacro {
            macro_id: "nonexistent".to_string(),
            name: Some("x".to_string()),
            targets: None,
            range_start: None,
            range_end: None,
        },
    );

    assert!(matches!(
        result,
        Err(MacroCommandError::MissingMacro { .. })
    ));
}

#[test]
fn macro_remove_cleans_scene_overrides() {
    let mut store = SessionStore::new_default();
    let mut session = macro_test_session();
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
fn macro_set_value_audio_parameter_target() {
    let mut store = SessionStore::new_default();
    let mut session = macro_test_session();
    session.macros.push(MacroDefinition {
        id: "macro-freq".to_string(),
        name: "freq".to_string(),
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
            macro_id: "macro-freq".to_string(),
            value: 0.5,
        },
    )
    .expect("set value");

    let freq = updated
        .nodes
        .iter()
        .find(|n| n.id == "node-src")
        .and_then(|n| n.parameters.iter().find(|p| p.id == "param-freq"))
        .unwrap();

    let expected = 100.0 + (0.5 * (1000.0 - 100.0));
    assert!((freq.value - expected).abs() < 0.01);
}

#[test]
fn macro_set_value_visual_parameter_target() {
    let mut store = SessionStore::new_default();
    let mut session = macro_test_session();
    session.macros.push(MacroDefinition {
        id: "macro-vis".to_string(),
        name: "color".to_string(),
        target_parameter_ids: vec![],
        range_start: 0.0,
        range_end: 1.0,
        targets: vec![MacroTarget::VisualParameter {
            element_id: "element-sphere".to_string(),
            parameter_id: "param-hue".to_string(),
        }],
    });
    store.replace_current(session);

    let result = apply_macro_command(
        &mut store,
        MacroCommand::SetMacroValue {
            macro_id: "macro-vis".to_string(),
            value: 0.75,
        },
    );

    assert!(result.is_ok());
}

#[test]
fn macro_set_value_multiple_targets_audio_and_visual() {
    let mut store = SessionStore::new_default();
    let mut session = macro_test_session();
    session.macros.push(MacroDefinition {
        id: "macro-cross".to_string(),
        name: "cross".to_string(),
        target_parameter_ids: vec![],
        range_start: 0.0,
        range_end: 1.0,
        targets: vec![
            MacroTarget::AudioParameter {
                node_id: "node-fx".to_string(),
                parameter_id: "param-mix".to_string(),
            },
            MacroTarget::VisualParameter {
                element_id: "element-1".to_string(),
                parameter_id: "param-opacity".to_string(),
            },
        ],
    });
    store.replace_current(session);

    let updated = apply_macro_command(
        &mut store,
        MacroCommand::SetMacroValue {
            macro_id: "macro-cross".to_string(),
            value: 0.6,
        },
    )
    .expect("set value");

    let mix = updated
        .nodes
        .iter()
        .find(|n| n.id == "node-fx")
        .and_then(|n| n.parameters.iter().find(|p| p.id == "param-mix"))
        .unwrap();

    assert!((mix.value - 0.6).abs() < 0.01);
}

#[test]
fn macro_backward_compat_flat_target_parameter_ids() {
    let mut store = SessionStore::new_default();
    let mut session = macro_test_session();
    session.macros.push(MacroDefinition {
        id: "macro-legacy".to_string(),
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
            macro_id: "macro-legacy".to_string(),
            value: 1.0,
        },
    )
    .expect("set value");

    let freq = updated
        .nodes
        .iter()
        .find(|n| n.id == "node-src")
        .and_then(|n| n.parameters.iter().find(|p| p.id == "param-freq"))
        .unwrap();

    assert!((freq.value - 800.0).abs() < 0.01);
}

#[test]
fn macro_scene_recall_with_macro_targets() {
    use scrysynth_lib::application::performance_command::apply_performance_command;
    use scrysynth_lib::domain::session::PerformanceCommand;

    let mut store = SessionStore::new_default();
    let mut session = macro_test_session();
    session.macros.push(MacroDefinition {
        id: "macro-1".to_string(),
        name: "freq".to_string(),
        target_parameter_ids: vec![],
        range_start: 100.0,
        range_end: 5000.0,
        targets: vec![MacroTarget::AudioParameter {
            node_id: "node-src".to_string(),
            parameter_id: "param-freq".to_string(),
        }],
    });
    session.scenes[0].macro_overrides.push(MacroOverride {
        macro_id: "macro-1".to_string(),
        value: 0.3,
    });
    store.replace_current(session);

    let updated = apply_performance_command(
        &mut store,
        PerformanceCommand::RecallScene {
            scene_id: "scene-a".to_string(),
        },
    )
    .expect("recall scene");

    let freq = updated
        .nodes
        .iter()
        .find(|n| n.id == "node-src")
        .and_then(|n| n.parameters.iter().find(|p| p.id == "param-freq"))
        .unwrap();

    let expected = 100.0 + (0.3 * (5000.0 - 100.0));
    assert!(
        (freq.value - expected).abs() < 0.01,
        "expected {}, got {}",
        expected,
        freq.value
    );
}

#[test]
fn macro_value_scaling_range() {
    let mut store = SessionStore::new_default();
    let mut session = macro_test_session();
    session.macros.push(MacroDefinition {
        id: "macro-scale".to_string(),
        name: "scale".to_string(),
        target_parameter_ids: vec![],
        range_start: 0.2,
        range_end: 0.8,
        targets: vec![MacroTarget::AudioParameter {
            node_id: "node-fx".to_string(),
            parameter_id: "param-mix".to_string(),
        }],
    });
    store.replace_current(session);

    let at_zero = apply_macro_command(
        &mut store,
        MacroCommand::SetMacroValue {
            macro_id: "macro-scale".to_string(),
            value: 0.0,
        },
    )
    .expect("value 0.0");
    let mix_zero = at_zero
        .nodes
        .iter()
        .find(|n| n.id == "node-fx")
        .and_then(|n| n.parameters.iter().find(|p| p.id == "param-mix"))
        .unwrap();
    assert!(
        (mix_zero.value - 0.2).abs() < 0.01,
        "value at 0.0 should be range_start"
    );

    let at_half = apply_macro_command(
        &mut store,
        MacroCommand::SetMacroValue {
            macro_id: "macro-scale".to_string(),
            value: 0.5,
        },
    )
    .expect("value 0.5");
    let mix_half = at_half
        .nodes
        .iter()
        .find(|n| n.id == "node-fx")
        .and_then(|n| n.parameters.iter().find(|p| p.id == "param-mix"))
        .unwrap();
    assert!(
        (mix_half.value - 0.5).abs() < 0.01,
        "value at 0.5 should be midpoint"
    );

    let at_one = apply_macro_command(
        &mut store,
        MacroCommand::SetMacroValue {
            macro_id: "macro-scale".to_string(),
            value: 1.0,
        },
    )
    .expect("value 1.0");
    let mix_one = at_one
        .nodes
        .iter()
        .find(|n| n.id == "node-fx")
        .and_then(|n| n.parameters.iter().find(|p| p.id == "param-mix"))
        .unwrap();
    assert!(
        (mix_one.value - 0.8).abs() < 0.01,
        "value at 1.0 should be range_end"
    );
}
