use scrysynth_lib::domain::session::{
    AudioBusType, AudioOutputNode, AudioOutputType, AudioPrimitive, AudioSourceNode,
    AudioSourceType, Bus, ChannelMode, ControllerKind, Node, NodeType, OwnershipAssignment,
    ParameterValue, Port, PortDirection, Route, SessionDocument, SignalType,
};

use scrysynth_lib::audio::compiler::compile_session_to_topology;

fn deterministic_session() -> SessionDocument {
    SessionDocument {
        title: "Deterministic Runtime".to_string(),
        nodes: vec![
            Node {
                id: "node-output".to_string(),
                node_type: NodeType::Output,
                ports: vec![Port {
                    id: "port-output-in".to_string(),
                    name: "master_in".to_string(),
                    direction: PortDirection::Input,
                    signal_type: SignalType::Audio,
                }],
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
            Node {
                id: "node-source".to_string(),
                node_type: NodeType::Source,
                ports: vec![Port {
                    id: "port-source-out".to_string(),
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
        ],
        routes: vec![Route {
            id: "route-source-output".to_string(),
            source_node_id: "node-source".to_string(),
            source_port_id: "port-source-out".to_string(),
            target_node_id: "node-output".to_string(),
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
        ..SessionDocument::default()
    }
}

mod audio_runtime {
    mod compiler {
        use super::super::*;

        #[test]
        fn deterministic_topology_compilation_preserves_ordering() {
            let session = deterministic_session();

            let first = compile_session_to_topology(&session).expect("first compile succeeds");
            let second = compile_session_to_topology(&session).expect("second compile succeeds");

            assert_eq!(first, second, "deterministic");
            assert_eq!(first.buses.len(), 1);
            assert_eq!(first.node_launch_order.len(), 2);
            assert_eq!(
                first.group_order[0].node_ids,
                vec!["node-source", "node-output"]
            );
        }

        #[test]
        fn compile_error_rejects_missing_bus_reference() {
            let mut session = deterministic_session();
            session.buses.clear();

            let error = compile_session_to_topology(&session).expect_err("compile_error");

            assert!(error.to_string().contains("bus-main"));
        }
    }
}
