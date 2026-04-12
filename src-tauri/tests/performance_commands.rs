use scrysynth_lib::application::performance_command::{
    apply_performance_command, PerformanceCommandError,
};
use scrysynth_lib::application::session_store::SessionStore;
use scrysynth_lib::domain::session::{
    AudioBusType, AudioOutputNode, AudioOutputType, AudioPrimitive, AudioSourceNode,
    AudioSourceType, Bus, ChannelMode, ControllerKind, MacroDefinition, MacroOverride, Node,
    NodeType, OwnershipAssignment, ParameterOverride, ParameterValue, Port, PortDirection, Route,
    SceneDefinition, SessionDocument, SignalType, VariationDefinition,
};

fn performance_test_session() -> SessionDocument {
    SessionDocument {
        title: "Performance Integration".to_string(),
        nodes: vec![
            Node {
                id: "source-1".to_string(),
                node_type: NodeType::Source,
                ports: vec![Port {
                    id: "source-1-out".to_string(),
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
                scene_membership: vec!["scene-intro".to_string()],
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
                id: "output-1".to_string(),
                node_type: NodeType::Output,
                ports: vec![Port {
                    id: "output-1-in".to_string(),
                    name: "master_in".to_string(),
                    direction: PortDirection::Input,
                    signal_type: SignalType::Audio,
                }],
                parameters: vec![ParameterValue {
                    id: "param-master-level".to_string(),
                    name: "level".to_string(),
                    value: 0.9,
                    default_value: 0.9,
                    min_value: 0.0,
                    max_value: 1.0,
                    unit: "linear".to_string(),
                }],
                runtime_target: Some("audio/output/master".to_string()),
                scene_membership: vec!["scene-intro".to_string(), "scene-outro".to_string()],
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
            source_node_id: "source-1".to_string(),
            source_port_id: "source-1-out".to_string(),
            target_node_id: "output-1".to_string(),
            target_port_id: "output-1-in".to_string(),
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
            id: "macro-energy".to_string(),
            name: "energy".to_string(),
            target_parameter_ids: vec!["param-freq".to_string()],
            range_start: 100.0,
            range_end: 2000.0,
        }],
        scenes: vec![
            SceneDefinition {
                id: "scene-intro".to_string(),
                name: "Intro".to_string(),
                active_node_ids: vec!["source-1".to_string(), "output-1".to_string()],
                macro_overrides: vec![MacroOverride {
                    macro_id: "macro-energy".to_string(),
                    value: 0.22,
                }],
            },
            SceneDefinition {
                id: "scene-outro".to_string(),
                name: "Outro".to_string(),
                active_node_ids: vec!["output-1".to_string()],
                macro_overrides: vec![],
            },
        ],
        variations: vec![VariationDefinition {
            id: "var-soft".to_string(),
            name: "Soft Intro".to_string(),
            scene_id: "scene-intro".to_string(),
            parameter_overrides: vec![ParameterOverride {
                parameter_id: "param-freq".to_string(),
                value: 220.0,
            }],
        }],
        ..SessionDocument::default()
    }
}

#[test]
fn performance_recall_scene_enables_correct_nodes() {
    let mut store = SessionStore::new_default();
    store.replace_current(performance_test_session());

    let updated = apply_performance_command(
        &mut store,
        scrysynth_lib::domain::session::PerformanceCommand::RecallScene {
            scene_id: "scene-outro".to_string(),
        },
    )
    .expect("recall outro");

    let source_enabled = updated
        .nodes
        .iter()
        .find(|n| n.id == "source-1")
        .unwrap()
        .enabled;
    let output_enabled = updated
        .nodes
        .iter()
        .find(|n| n.id == "output-1")
        .unwrap()
        .enabled;

    assert!(!source_enabled, "source should be disabled in outro scene");
    assert!(output_enabled, "output should be enabled in outro scene");
}

#[test]
fn performance_recall_scene_applies_macro_overrides() {
    let mut store = SessionStore::new_default();
    store.replace_current(performance_test_session());

    let updated = apply_performance_command(
        &mut store,
        scrysynth_lib::domain::session::PerformanceCommand::RecallScene {
            scene_id: "scene-intro".to_string(),
        },
    )
    .expect("recall intro");

    let freq_param = updated
        .nodes
        .iter()
        .find(|n| n.id == "source-1")
        .and_then(|n| n.parameters.iter().find(|p| p.id == "param-freq"))
        .unwrap();

    let expected = 100.0 + (0.22 * (2000.0 - 100.0));
    assert!(
        (freq_param.value - expected).abs() < 0.01,
        "expected {}, got {}",
        expected,
        freq_param.value
    );
}

#[test]
fn performance_recall_scene_rejects_unknown_scene() {
    let mut store = SessionStore::new_default();
    store.replace_current(performance_test_session());
    let original = store.current();

    let result = apply_performance_command(
        &mut store,
        scrysynth_lib::domain::session::PerformanceCommand::RecallScene {
            scene_id: "no-such-scene".to_string(),
        },
    );

    assert!(matches!(
        result,
        Err(PerformanceCommandError::MissingScene { .. })
    ));
    assert_eq!(store.current().nodes, original.nodes);
}

#[test]
fn performance_save_variation_snapshots_parameters() {
    let mut store = SessionStore::new_default();
    store.replace_current(performance_test_session());

    let updated = apply_performance_command(
        &mut store,
        scrysynth_lib::domain::session::PerformanceCommand::SaveVariation {
            name: "current state".to_string(),
            scene_id: "scene-intro".to_string(),
        },
    )
    .expect("save succeeds");

    assert_eq!(updated.variations.len(), 2);
    let new_variation = updated
        .variations
        .iter()
        .find(|v| v.name == "current state")
        .unwrap();
    assert_eq!(new_variation.scene_id, "scene-intro");
    assert!(new_variation
        .parameter_overrides
        .iter()
        .any(|p| p.parameter_id == "param-freq" && (p.value - 440.0).abs() < 0.01));
}

#[test]
fn performance_save_variation_rejects_unknown_scene() {
    let mut store = SessionStore::new_default();
    store.replace_current(performance_test_session());

    let result = apply_performance_command(
        &mut store,
        scrysynth_lib::domain::session::PerformanceCommand::SaveVariation {
            name: "orphan".to_string(),
            scene_id: "unknown".to_string(),
        },
    );

    assert!(matches!(
        result,
        Err(PerformanceCommandError::MissingScene { .. })
    ));
}

#[test]
fn performance_restore_variation_applies_overrides() {
    let mut store = SessionStore::new_default();
    store.replace_current(performance_test_session());

    let updated = apply_performance_command(
        &mut store,
        scrysynth_lib::domain::session::PerformanceCommand::RestoreVariation {
            variation_id: "var-soft".to_string(),
        },
    )
    .expect("restore succeeds");

    let freq_param = updated
        .nodes
        .iter()
        .find(|n| n.id == "source-1")
        .and_then(|n| n.parameters.iter().find(|p| p.id == "param-freq"))
        .unwrap();

    assert_eq!(freq_param.value, 220.0);
}

#[test]
fn performance_restore_variation_rejects_unknown() {
    let mut store = SessionStore::new_default();
    store.replace_current(performance_test_session());

    let result = apply_performance_command(
        &mut store,
        scrysynth_lib::domain::session::PerformanceCommand::RestoreVariation {
            variation_id: "nonexistent".to_string(),
        },
    );

    assert!(matches!(
        result,
        Err(PerformanceCommandError::MissingVariation { .. })
    ));
}

#[test]
fn performance_full_scene_recall_save_restore_cycle() {
    let mut store = SessionStore::new_default();
    store.replace_current(performance_test_session());

    apply_performance_command(
        &mut store,
        scrysynth_lib::domain::session::PerformanceCommand::RecallScene {
            scene_id: "scene-intro".to_string(),
        },
    )
    .expect("recall intro");

    let with_snapshot = apply_performance_command(
        &mut store,
        scrysynth_lib::domain::session::PerformanceCommand::SaveVariation {
            name: "intro baseline".to_string(),
            scene_id: "scene-intro".to_string(),
        },
    )
    .expect("save");
    let snapshot_id = with_snapshot
        .variations
        .iter()
        .find(|v| v.name == "intro baseline")
        .unwrap()
        .id
        .clone();

    apply_performance_command(
        &mut store,
        scrysynth_lib::domain::session::PerformanceCommand::RecallScene {
            scene_id: "scene-outro".to_string(),
        },
    )
    .expect("switch to outro");

    let after_outro = store.current();
    let source = after_outro
        .nodes
        .iter()
        .find(|n| n.id == "source-1")
        .unwrap();
    assert!(!source.enabled);

    apply_performance_command(
        &mut store,
        scrysynth_lib::domain::session::PerformanceCommand::RecallScene {
            scene_id: "scene-intro".to_string(),
        },
    )
    .expect("back to intro");

    let restored = apply_performance_command(
        &mut store,
        scrysynth_lib::domain::session::PerformanceCommand::RestoreVariation {
            variation_id: snapshot_id,
        },
    )
    .expect("restore snapshot");

    let source = restored.nodes.iter().find(|n| n.id == "source-1").unwrap();
    assert!(source.enabled);

    let expected_freq = 100.0 + (0.22 * (2000.0 - 100.0));
    let freq = source
        .parameters
        .iter()
        .find(|p| p.id == "param-freq")
        .unwrap();
    assert!(
        (freq.value - expected_freq).abs() < 0.01,
        "snapshot should restore the macro-overridden value"
    );
}
