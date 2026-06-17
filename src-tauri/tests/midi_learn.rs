use std::sync::mpsc;

use scrysynth_lib::application::midi_learn::{
    scale_value_exposed, HardwareInputRouter, HardwareLearnState,
};
use scrysynth_lib::application::session_store::SessionStore;
use scrysynth_lib::domain::session::{
    new_id, BindingTarget, HardwareBinding, HardwareLearnLifecycle, HardwareListenerLifecycle,
    HardwareRuntimeSettings, HardwareRuntimeStatus, HardwareSource, MacroDefinition, MacroTarget,
    Node, NodeType, OwnershipAssignment, ParameterValue, Port, PortDirection, SessionDocument,
    SignalType, ValueTransform,
};
use scrysynth_lib::hardware::midi_input::{parse_midi_message, MidiLearnEvent};

fn hardware_test_session() -> SessionDocument {
    SessionDocument {
        title: "Hardware Test".to_string(),
        nodes: vec![Node {
            id: "node-src".to_string(),
            node_type: NodeType::Source,
            ports: vec![Port {
                id: "port-out".to_string(),
                name: "out".to_string(),
                direction: PortDirection::Output,
                signal_type: SignalType::Audio,
            }],
            parameters: vec![ParameterValue {
                id: "param-gain".to_string(),
                name: "gain".to_string(),
                value: 0.5,
                default_value: 0.5,
                min_value: 0.0,
                max_value: 1.0,
                unit: "linear".to_string(),
            }],
            runtime_target: None,
            scene_membership: vec![],
            ownership: OwnershipAssignment {
                controller: scrysynth_lib::domain::session::ControllerKind::Shared,
                is_locked: false,
            },
            enabled: true,
            audio_primitive: None,
        }],
        macros: vec![MacroDefinition {
            id: "macro-energy".to_string(),
            name: "energy".to_string(),
            target_parameter_ids: vec![],
            range_start: 0.0,
            range_end: 1.0,
            targets: vec![MacroTarget::AudioParameter {
                node_id: "node-src".to_string(),
                parameter_id: "param-gain".to_string(),
            }],
        }],
        ..SessionDocument::default()
    }
}

#[test]
fn hardware_binding_midi_cc_round_trip() {
    let binding = HardwareBinding {
        id: new_id(),
        source: HardwareSource::MidiCc {
            channel: 1,
            controller: 7,
        },
        target: BindingTarget::Macro {
            macro_id: "macro-energy".to_string(),
        },
        transform: ValueTransform::default(),
    };

    let json = serde_json::to_string(&binding).expect("serialize");
    let restored: HardwareBinding = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored, binding);
}

#[test]
fn hardware_binding_osc_address_round_trip() {
    let binding = HardwareBinding {
        id: new_id(),
        source: HardwareSource::OscAddress {
            address: "/scrysynth/energy".to_string(),
        },
        target: BindingTarget::Macro {
            macro_id: "macro-energy".to_string(),
        },
        transform: ValueTransform {
            input_min: 0.0,
            input_max: 1.0,
            output_min: 0.0,
            output_max: 1.0,
        },
    };

    let json = serde_json::to_string(&binding).expect("serialize");
    let restored: HardwareBinding = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored, binding);
}

#[test]
fn all_binding_target_variants_serialize() {
    let targets = vec![
        BindingTarget::Macro {
            macro_id: "m1".to_string(),
        },
        BindingTarget::SceneRecall {
            scene_id: "s1".to_string(),
        },
        BindingTarget::TransportPlay,
        BindingTarget::TransportStop,
        BindingTarget::TransportPanic,
    ];

    for target in &targets {
        let json = serde_json::to_string(target).expect("serialize target");
        let restored: BindingTarget = serde_json::from_str(&json).expect("deserialize target");
        assert_eq!(&restored, target);
    }
}

#[test]
fn learn_state_machine_transitions() {
    let mut router = HardwareInputRouter::new();
    assert_eq!(router.learn_state, HardwareLearnState::Idle);

    let target = BindingTarget::Macro {
        macro_id: "m1".to_string(),
    };
    router.start_learn(target.clone());
    match &router.learn_state {
        HardwareLearnState::Learning { target: t } => {
            assert_eq!(
                t,
                &BindingTarget::Macro {
                    macro_id: "m1".to_string(),
                }
            );
        }
        _ => panic!("expected Learning state"),
    }

    let (tx, rx) = mpsc::channel();
    router.midi_rx = Some(rx);
    tx.send(MidiLearnEvent::MidiCc {
        channel: 0,
        controller: 7,
        value: 100,
    })
    .unwrap();

    let mut session = hardware_test_session();
    let binding = router.poll_and_route(&mut session);
    assert!(binding.is_some());

    match &router.learn_state {
        HardwareLearnState::Captured { source, target: t } => {
            assert_eq!(
                source,
                &HardwareSource::MidiCc {
                    channel: 0,
                    controller: 7,
                }
            );
            assert_eq!(
                t,
                &BindingTarget::Macro {
                    macro_id: "m1".to_string(),
                }
            );
        }
        _ => panic!("expected Captured state"),
    }

    router.stop_learn();
    assert_eq!(router.learn_state, HardwareLearnState::Idle);
}

#[test]
fn learn_state_start_stop_returns_idle() {
    let mut router = HardwareInputRouter::new();
    router.start_learn(BindingTarget::TransportPlay);
    assert!(matches!(
        &router.learn_state,
        HardwareLearnState::Learning { .. }
    ));

    router.stop_learn();
    assert_eq!(router.learn_state, HardwareLearnState::Idle);
}

#[test]
fn hardware_binding_persists_in_session() {
    let mut store = SessionStore::new_default();
    store.replace_current(hardware_test_session());

    let binding = HardwareBinding {
        id: "hb-1".to_string(),
        source: HardwareSource::MidiCc {
            channel: 0,
            controller: 7,
        },
        target: BindingTarget::Macro {
            macro_id: "macro-energy".to_string(),
        },
        transform: ValueTransform::default(),
    };

    let _ = store.mutate_current(|session| {
        session.hardware_bindings.push(binding);
        Ok::<(), ()>(())
    });

    let session = store.current();
    assert_eq!(session.hardware_bindings.len(), 1);
    assert_eq!(session.hardware_bindings[0].id, "hb-1");
}

#[test]
fn remove_hardware_binding() {
    let mut store = SessionStore::new_default();
    store.replace_current(hardware_test_session());

    let _ = store.mutate_current(|session| {
        session.hardware_bindings.push(HardwareBinding {
            id: "hb-1".to_string(),
            source: HardwareSource::MidiCc {
                channel: 0,
                controller: 7,
            },
            target: BindingTarget::Macro {
                macro_id: "macro-energy".to_string(),
            },
            transform: ValueTransform::default(),
        });
        Ok::<(), ()>(())
    });

    assert_eq!(store.current().hardware_bindings.len(), 1);

    let removed = store.remove_hardware_binding("hb-1");
    assert!(removed);
    assert!(store.current().hardware_bindings.is_empty());

    let removed_again = store.remove_hardware_binding("hb-1");
    assert!(!removed_again);
}

#[test]
fn midi_parsing_cc() {
    let msg = [0xB3u8, 7, 100];
    let event = parse_midi_message(&msg).unwrap();
    assert_eq!(
        event,
        MidiLearnEvent::MidiCc {
            channel: 3,
            controller: 7,
            value: 100,
        }
    );
}

#[test]
fn midi_parsing_note_on() {
    let msg = [0x90u8, 60, 127];
    let event = parse_midi_message(&msg).unwrap();
    assert_eq!(
        event,
        MidiLearnEvent::MidiNote {
            channel: 0,
            note: 60,
            velocity: 127,
        }
    );
}

#[test]
fn midi_parsing_pitch_bend() {
    let msg = [0xE0u8, 0x00, 0x40];
    let event = parse_midi_message(&msg).unwrap();
    assert_eq!(
        event,
        MidiLearnEvent::MidiPitchBend {
            channel: 0,
            value: 0x2000,
        }
    );
}

#[test]
fn midi_parsing_edge_cases() {
    assert!(parse_midi_message(&[]).is_none());
    assert!(parse_midi_message(&[0xF0, 0x00]).is_none());
    assert!(parse_midi_message(&[0xB0, 7]).is_none());
}

#[test]
fn value_transform_scaling() {
    let transform = ValueTransform {
        input_min: 0.0,
        input_max: 127.0,
        output_min: 0.0,
        output_max: 1.0,
    };

    let result = scale_value_exposed(0.0, &transform);
    assert!((result - 0.0).abs() < f64::EPSILON);

    let result = scale_value_exposed(127.0, &transform);
    assert!((result - 1.0).abs() < f64::EPSILON);

    let result = scale_value_exposed(63.0, &transform);
    assert!((result - 0.496).abs() < 0.01);
}

#[test]
fn session_document_backward_compat_no_hardware_bindings() {
    let json = r#"{
        "schemaVersion": 1,
        "sessionId": "s1",
        "title": "Old Session",
        "createdAt": "2026-04-11T00:00:00Z",
        "updatedAt": "2026-04-11T00:00:00Z",
        "transport": { "tempoBpm": 120, "isPlaying": false, "positionBeats": 0 },
        "audioRuntime": { "lifecycle": "idle", "health": "unknown", "sampleRateHz": null, "blockSize": null, "activePatchId": null, "lastError": null, "panicRecoveryCount": 0 },
        "visualRuntime": { "lifecycle": "idle", "health": "unknown", "activeSceneId": null, "fps": null, "lastError": null, "renderer": null },
        "agentRuntime": { "isAvailable": true, "pendingActionCount": 0, "isFrozen": false },
        "nodes": [],
        "routes": [],
        "buses": [],
        "macros": [],
        "scenes": [],
        "variations": [],
        "ownershipRules": [],
        "runtimeStatus": [],
        "agentFrozen": false,
        "pendingActions": [],
        "actionHistory": []
    }"#;

    let session: SessionDocument = serde_json::from_str(json).expect("deserialize old session");
    assert!(session.hardware_bindings.is_empty());
}

#[test]
fn typescript_contract_includes_hardware_types() {
    use std::fs;
    use std::path::PathBuf;

    scrysynth_lib::domain::session::write_generated_typescript_contract().expect("write contract");

    let file_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/generated/session-types.ts");
    let generated = fs::read_to_string(file_path).expect("read generated types");

    assert!(generated.contains("export type HardwareBinding"));
    assert!(generated.contains("export type HardwareSource"));
    assert!(generated.contains("export type BindingTarget"));
    assert!(generated.contains("export type ValueTransform"));
    assert!(generated.contains("export type HardwareRuntimeSettings"));
    assert!(generated.contains("export type HardwareRuntimeStatus"));
    assert!(generated.contains("export type MidiInputPort"));
}

#[test]
fn hardware_runtime_contract_stays_outside_session_document() {
    let settings = HardwareRuntimeSettings::default();
    assert_eq!(settings.midi.selected_input_id, None);
    assert!(!settings.midi.auto_start);
    assert_eq!(settings.osc.bind_host, "127.0.0.1");
    assert_eq!(settings.osc.listen_port, 9000);

    let status = HardwareRuntimeStatus::default();
    assert_eq!(status.midi.lifecycle, HardwareListenerLifecycle::Stopped);
    assert_eq!(status.osc.lifecycle, HardwareListenerLifecycle::Stopped);
    assert_eq!(status.learn.lifecycle, HardwareLearnLifecycle::Idle);

    let session_json = serde_json::to_value(SessionDocument::default()).expect("serialize session");
    assert!(session_json.get("hardwareBindings").is_some());
    assert!(session_json.get("hardwareRuntimeSettings").is_none());
    assert!(session_json.get("hardwareRuntimeStatus").is_none());
}

#[test]
fn live_routing_midi_cc_to_macro() {
    let (tx, rx) = mpsc::channel();
    let mut router = HardwareInputRouter::new();
    router.midi_rx = Some(rx);

    let mut session = hardware_test_session();
    session.hardware_bindings.push(HardwareBinding {
        id: "hb-1".to_string(),
        source: HardwareSource::MidiCc {
            channel: 0,
            controller: 7,
        },
        target: BindingTarget::Macro {
            macro_id: "macro-energy".to_string(),
        },
        transform: ValueTransform {
            input_min: 0.0,
            input_max: 127.0,
            output_min: 0.0,
            output_max: 1.0,
        },
    });

    tx.send(MidiLearnEvent::MidiCc {
        channel: 0,
        controller: 7,
        value: 63,
    })
    .unwrap();

    let result = router.poll_and_route(&mut session);
    assert!(result.is_none());

    let gain = session.nodes[0]
        .parameters
        .iter()
        .find(|p| p.id == "param-gain")
        .unwrap();
    let expected = 63.0 / 127.0;
    assert!((gain.value - expected).abs() < 0.01);
}

#[test]
fn live_routing_transport_play() {
    let (tx, rx) = mpsc::channel();
    let mut router = HardwareInputRouter::new();
    router.midi_rx = Some(rx);

    let mut session = hardware_test_session();
    assert!(!session.transport.is_playing);

    session.hardware_bindings.push(HardwareBinding {
        id: "hb-play".to_string(),
        source: HardwareSource::MidiNote {
            channel: 0,
            note: 60,
        },
        target: BindingTarget::TransportPlay,
        transform: ValueTransform {
            input_min: 0.0,
            input_max: 127.0,
            output_min: 0.0,
            output_max: 1.0,
        },
    });

    tx.send(MidiLearnEvent::MidiNote {
        channel: 0,
        note: 60,
        velocity: 127,
    })
    .unwrap();

    router.poll_and_route(&mut session);
    assert!(session.transport.is_playing);
}

#[test]
fn live_routing_transport_stop() {
    let (tx, rx) = mpsc::channel();
    let mut router = HardwareInputRouter::new();
    router.midi_rx = Some(rx);

    let mut session = hardware_test_session();
    session.transport.is_playing = true;

    session.hardware_bindings.push(HardwareBinding {
        id: "hb-stop".to_string(),
        source: HardwareSource::MidiNote {
            channel: 0,
            note: 61,
        },
        target: BindingTarget::TransportStop,
        transform: ValueTransform::default(),
    });

    tx.send(MidiLearnEvent::MidiNote {
        channel: 0,
        note: 61,
        velocity: 127,
    })
    .unwrap();

    router.poll_and_route(&mut session);
    assert!(!session.transport.is_playing);
}
