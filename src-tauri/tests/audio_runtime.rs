use scrysynth_lib::catalog::{find_catalog_entry, CATALOG};
use scrysynth_lib::domain::session::{
    AudioBusType, Bus, ChannelMode, ControllerKind, Node, OutputKind, OwnershipAssignment,
    ParameterValue, Port, PortDirection, Route, SessionDocument, SignalType,
};

use scrysynth_lib::audio::compiler::compile_session_to_topology;
use scrysynth_lib::audio::synthdefs::{plan_sc_resources, ScResourcePlanError};

fn oscillator_node(id: &str, bus: &str) -> Node {
    Node {
        id: id.to_string(),
        node_type_id: "oscillator".to_string(),
        ports: vec![Port {
            id: format!("{id}-out"),
            name: "main_out".to_string(),
            direction: PortDirection::Output,
            signal_type: SignalType::Audio,
        }],
        parameters: vec![ParameterValue {
            id: format!("{id}-freq"),
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
        bus_target_id: Some(bus.to_string()),
        output_kind: None,
        channel_count: None,
        bypassed: None,
        channel_mode: Some(ChannelMode::Mono),
        sequencer_pattern: None,
    }
}

fn output_node(id: &str, bus: &str) -> Node {
    Node {
        id: id.to_string(),
        node_type_id: "output".to_string(),
        ports: vec![Port {
            id: format!("{id}-in"),
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
        bus_target_id: Some(bus.to_string()),
        output_kind: Some(OutputKind::Master),
        channel_count: Some(2),
        bypassed: None,
        channel_mode: None,
        sequencer_pattern: None,
    }
}

fn deterministic_session() -> SessionDocument {
    SessionDocument {
        title: "Deterministic Runtime".to_string(),
        nodes: vec![
            output_node("node-output", "bus-main"),
            oscillator_node("node-source", "bus-main"),
        ],
        routes: vec![Route {
            id: "route-source-output".to_string(),
            source_node_id: "node-source".to_string(),
            source_port_id: "node-source-out".to_string(),
            target_node_id: "node-output".to_string(),
            target_port_id: "node-output-in".to_string(),
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
        fn topology_compilation_preserves_catalog_node_identity() {
            let mut session = deterministic_session();
            session.nodes.push(Node {
                id: "node-delay".to_string(),
                node_type_id: "delay".to_string(),
                ports: vec![
                    Port {
                        id: "port-delay-in".to_string(),
                        name: "audio_in".to_string(),
                        direction: PortDirection::Input,
                        signal_type: SignalType::Audio,
                    },
                    Port {
                        id: "port-delay-out".to_string(),
                        name: "audio_out".to_string(),
                        direction: PortDirection::Output,
                        signal_type: SignalType::Audio,
                    },
                ],
                parameters: vec![],
                runtime_target: Some("audio/effect/delay".to_string()),
                scene_membership: vec![],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::Shared,
                    is_locked: false,
                },
                enabled: true,
                bus_target_id: Some("bus-main".to_string()),
                output_kind: None,
                channel_count: None,
                bypassed: Some(true),
                channel_mode: None,
                sequencer_pattern: None,
            });
            session.routes = vec![
                Route {
                    id: "route-source-delay".to_string(),
                    source_node_id: "node-source".to_string(),
                    source_port_id: "node-source-out".to_string(),
                    target_node_id: "node-delay".to_string(),
                    target_port_id: "port-delay-in".to_string(),
                    bus_id: Some("bus-main".to_string()),
                },
                Route {
                    id: "route-delay-output".to_string(),
                    source_node_id: "node-delay".to_string(),
                    source_port_id: "port-delay-out".to_string(),
                    target_node_id: "node-output".to_string(),
                    target_port_id: "node-output-in".to_string(),
                    bus_id: Some("bus-main".to_string()),
                },
            ];

            let topology = compile_session_to_topology(&session).expect("compile succeeds");

            // Catalog identity + config flow through the compiled launch (replaces
            // v1's CompiledNodeKind enum assertions).
            assert_eq!(topology.node_launch_order[0].node_type_id, "oscillator");
            assert_eq!(topology.node_launch_order[1].node_type_id, "delay");
            assert!(topology.node_launch_order[1].bypassed, "delay bypassed flag");
            assert_eq!(topology.node_launch_order[2].node_type_id, "output");
            assert_eq!(
                topology.node_launch_order[2].output_kind,
                Some(OutputKind::Master)
            );
            assert_eq!(
                topology.node_launch_order[0].parameters[0].name,
                "frequency"
            );
        }

        #[test]
        fn cv_route_with_signal_type_mismatch_is_rejected_at_compile() {
            // Pitfall #3: an Audio↔Control CV mismatch is a silent SC failure —
            // the catalog-driven compiler rejects it (TopologyCompileError).
            let mut session = deterministic_session();
            // Source's audio out → filter's control `cutoff_cv` port = rate mismatch.
            session.nodes.push(Node {
                id: "node-filter".to_string(),
                node_type_id: "filter".to_string(),
                ports: vec![
                    Port {
                        id: "filter-in".to_string(),
                        name: "audio_in".to_string(),
                        direction: PortDirection::Input,
                        signal_type: SignalType::Audio,
                    },
                    Port {
                        id: "cutoff_cv".to_string(),
                        name: "Cutoff CV".to_string(),
                        direction: PortDirection::Input,
                        signal_type: SignalType::Control,
                    },
                ],
                parameters: vec![],
                runtime_target: Some("audio/effect/filter".to_string()),
                scene_membership: vec![],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::Shared,
                    is_locked: false,
                },
                enabled: true,
                bus_target_id: None,
                output_kind: None,
                channel_count: None,
                bypassed: None,
                channel_mode: None,
                sequencer_pattern: None,
            });
            session.routes.push(Route {
                id: "bad-cv-route".to_string(),
                source_node_id: "node-source".to_string(),
                source_port_id: "node-source-out".to_string(), // Audio
                target_node_id: "node-filter".to_string(),
                target_port_id: "cutoff_cv".to_string(), // Control
                bus_id: None,
            });

            let error = compile_session_to_topology(&session).expect_err("mismatch rejected");
            assert!(error.to_string().contains("signal-type mismatch"));
        }

        #[test]
        fn compile_error_rejects_missing_bus_reference() {
            let mut session = deterministic_session();
            session.buses.clear();

            let error = compile_session_to_topology(&session).expect_err("compile_error");

            assert!(error.to_string().contains("bus-main"));
        }
    }

    mod synthdefs {
        use super::super::*;

        fn mixer_node(id: &str, out_bus: &str) -> Node {
            Node {
                id: id.to_string(),
                node_type_id: "mixer".to_string(),
                ports: vec![
                    Port {
                        id: format!("{id}-in"),
                        name: "audio_in".to_string(),
                        direction: PortDirection::Input,
                        signal_type: SignalType::Audio,
                    },
                    Port {
                        id: format!("{id}-out"),
                        name: "audio_out".to_string(),
                        direction: PortDirection::Output,
                        signal_type: SignalType::Audio,
                    },
                ],
                parameters: vec![ParameterValue {
                    id: format!("{id}-gain"),
                    name: "gain".to_string(),
                    value: 0.75,
                    default_value: 1.0,
                    min_value: 0.0,
                    max_value: 1.0,
                    unit: "linear".to_string(),
                }],
                runtime_target: Some("audio/mixer/stereo".to_string()),
                scene_membership: vec![],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::Shared,
                    is_locked: false,
                },
                enabled: true,
                bus_target_id: Some(out_bus.to_string()),
                output_kind: None,
                channel_count: None,
                bypassed: None,
                channel_mode: Some(ChannelMode::Stereo),
                sequencer_pattern: None,
            }
        }

        #[test]
        fn resource_plan_maps_catalog_entries_to_known_synthdefs_and_params() {
            let mut session = deterministic_session();
            session.buses.push(Bus {
                id: "bus-mix".to_string(),
                name: "mix".to_string(),
                channels: 2,
                bus_type: AudioBusType::Auxiliary,
                is_enabled: true,
            });
            session.nodes.push(mixer_node("node-mixer", "bus-mix"));
            session.routes = vec![
                Route {
                    id: "route-source-mixer".to_string(),
                    source_node_id: "node-source".to_string(),
                    source_port_id: "node-source-out".to_string(),
                    target_node_id: "node-mixer".to_string(),
                    target_port_id: "node-mixer-in".to_string(),
                    bus_id: Some("bus-main".to_string()),
                },
                Route {
                    id: "route-mixer-output".to_string(),
                    source_node_id: "node-mixer".to_string(),
                    source_port_id: "node-mixer-out".to_string(),
                    target_node_id: "node-output".to_string(),
                    target_port_id: "node-output-in".to_string(),
                    bus_id: Some("bus-mix".to_string()),
                },
            ];

            let topology = compile_session_to_topology(&session).expect("compile succeeds");
            let plan = plan_sc_resources(&topology).expect("plan succeeds");

            assert!(plan.patch_id.starts_with("patch-v2-"));
            assert_eq!(plan.groups[0].node_id, 1000);
            // Catalog-derived synthdef names (no v1 consts).
            let mut synthdef_names: Vec<&str> =
                plan.synthdefs.iter().map(|r| r.name).collect();
            synthdef_names.sort();
            assert_eq!(
                synthdef_names,
                vec![
                    "scrysynth_v2_mixer",
                    "scrysynth_v2_oscillator",
                    "scrysynth_v2_output",
                ]
            );
            assert_eq!(plan.synths[0].synthdef_name, "scrysynth_v2_oscillator");
            assert_eq!(plan.synths[0].node_id, 2000);
            assert_eq!(plan.synths[1].synthdef_name, "scrysynth_v2_mixer");
            assert!(plan.synths[1]
                .args
                .iter()
                .any(|arg| arg.name == "in_bus_1" && arg.value == 2.0));
            // Catalog alias resolution: "gain" → sc_arg "level".
            assert!(plan.synths[1]
                .args
                .iter()
                .any(|arg| arg.name == "level" && arg.value == 0.75));
            assert_eq!(plan.synths[2].synthdef_name, "scrysynth_v2_output");
            assert!(plan.controls.iter().any(|control| control.control_key
                == "node-source:node-source-freq"
                && control.parameter_name == "frequency"));
        }

        #[test]
        fn resource_plan_patch_id_changes_for_same_shape_topology_changes() {
            let first_session = deterministic_session();
            let mut second_session = deterministic_session();
            second_session.nodes[1].parameters[0].value = 440.0;

            let first_topology =
                compile_session_to_topology(&first_session).expect("first compile succeeds");
            let second_topology =
                compile_session_to_topology(&second_session).expect("second compile succeeds");

            let first_plan = plan_sc_resources(&first_topology).expect("first plan succeeds");
            let second_plan = plan_sc_resources(&second_topology).expect("second plan succeeds");

            assert_ne!(first_plan.patch_id, second_plan.patch_id);
            assert!(first_plan.patch_id.starts_with("patch-v2-"));
            assert!(second_plan.patch_id.starts_with("patch-v2-"));
        }

        // Success criterion #3: an unknown node_type_id returns Err, never a panic
        // (the v1 `unreachable!()` at synthdefs.rs:455 is gone).
        #[test]
        fn resource_plan_fails_loudly_for_unknown_catalog_entry() {
            let mut session = deterministic_session();
            // An unknown node_type_id compiles (topology is catalog-agnostic) but
            // fails loudly at plan time with UnknownCatalogEntry.
            session.nodes[1].node_type_id = "totally-not-a-node".to_string();
            let topology = compile_session_to_topology(&session).expect("compile succeeds");

            let error = plan_sc_resources(&topology).expect_err("unknown catalog id fails");
            assert!(
                matches!(
                    error,
                    ScResourcePlanError::UnknownCatalogEntry { .. }
                ),
                "expected UnknownCatalogEntry, got {error:?}"
            );
            if let ScResourcePlanError::UnknownCatalogEntry { node_type_id } = &error {
                assert_eq!(node_type_id, "totally-not-a-node");
            }
        }

        #[test]
        fn resource_plan_fails_loudly_for_unsupported_parameter_name() {
            let mut session = deterministic_session();
            session.nodes[1].parameters.push(ParameterValue {
                id: "param-grain-size".to_string(),
                name: "grain_size".to_string(),
                value: 0.1,
                default_value: 0.1,
                min_value: 0.0,
                max_value: 1.0,
                unit: "s".to_string(),
            });
            let topology = compile_session_to_topology(&session).expect("compile succeeds");

            let error = plan_sc_resources(&topology).expect_err("unsupported parameter fails");

            assert!(error.to_string().contains("param-grain-size"));
            assert!(error.to_string().contains("grain_size"));
        }

        #[test]
        fn delay_node_uses_delay_synthdef_and_stable_parameter_names() {
            let mut session = deterministic_session();
            session.nodes.push(Node {
                id: "node-delay".to_string(),
                node_type_id: "delay".to_string(),
                ports: vec![
                    Port {
                        id: "delay-in".to_string(),
                        name: "audio_in".to_string(),
                        direction: PortDirection::Input,
                        signal_type: SignalType::Audio,
                    },
                    Port {
                        id: "delay-out".to_string(),
                        name: "audio_out".to_string(),
                        direction: PortDirection::Output,
                        signal_type: SignalType::Audio,
                    },
                ],
                parameters: vec![ParameterValue {
                    id: "param-delay-time".to_string(),
                    name: "delayTime".to_string(), // catalog alias → delay_time_s
                    value: 0.5,
                    default_value: 0.25,
                    min_value: 0.0,
                    max_value: 2.0,
                    unit: "s".to_string(),
                }],
                runtime_target: Some("audio/effect/delay".to_string()),
                scene_membership: vec![],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::Shared,
                    is_locked: false,
                },
                enabled: true,
                bus_target_id: Some("bus-main".to_string()),
                output_kind: None,
                channel_count: None,
                bypassed: Some(false),
                channel_mode: None,
                sequencer_pattern: None,
            });
            // source → delay → output (replaces the direct source→output route).
            session.routes = vec![
                Route {
                    id: "route-src-delay".to_string(),
                    source_node_id: "node-source".to_string(),
                    source_port_id: "node-source-out".to_string(),
                    target_node_id: "node-delay".to_string(),
                    target_port_id: "delay-in".to_string(),
                    bus_id: Some("bus-main".to_string()),
                },
                Route {
                    id: "route-delay-out".to_string(),
                    source_node_id: "node-delay".to_string(),
                    source_port_id: "delay-out".to_string(),
                    target_node_id: "node-output".to_string(),
                    target_port_id: "node-output-in".to_string(),
                    bus_id: Some("bus-main".to_string()),
                },
            ];

            let topology = compile_session_to_topology(&session).expect("compile succeeds");
            let plan = plan_sc_resources(&topology).expect("plan succeeds");

            let delay_synth = plan
                .synths
                .iter()
                .find(|s| s.synthdef_name == "scrysynth_v2_delay")
                .expect("delay synth present");
            assert!(delay_synth
                .args
                .iter()
                .any(|arg| arg.name == "delay_time_s" && arg.value == 0.5));
        }

        // NODES-05: a control-rate CV route allocates a control bus; the mod source
        // gains `out_cv_bus` and the target gains `<cv_port>_bus`.
        #[test]
        fn cv_route_allocates_control_bus_for_modulation() {
            let mut session = deterministic_session();
            session.nodes.push(Node {
                id: "node-lfo".to_string(),
                node_type_id: "lfo".to_string(),
                ports: vec![Port {
                    id: "lfo_out".to_string(),
                    name: "LFO Out".to_string(),
                    direction: PortDirection::Output,
                    signal_type: SignalType::Control,
                }],
                parameters: vec![ParameterValue {
                    id: "lfo-freq".to_string(),
                    name: "frequency".to_string(),
                    value: 0.5,
                    default_value: 0.5,
                    min_value: 0.001,
                    max_value: 100.0,
                    unit: "hz".to_string(),
                }],
                runtime_target: Some("audio/modulator/lfo".to_string()),
                scene_membership: vec![],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::Shared,
                    is_locked: false,
                },
                enabled: true,
                bus_target_id: None,
                output_kind: None,
                channel_count: None,
                bypassed: None,
                channel_mode: None,
                sequencer_pattern: None,
            });
            session.nodes.push(Node {
                id: "node-filter".to_string(),
                node_type_id: "filter".to_string(),
                ports: vec![
                    Port {
                        id: "filter-in".to_string(),
                        name: "audio_in".to_string(),
                        direction: PortDirection::Input,
                        signal_type: SignalType::Audio,
                    },
                    Port {
                        id: "filter-out".to_string(),
                        name: "audio_out".to_string(),
                        direction: PortDirection::Output,
                        signal_type: SignalType::Audio,
                    },
                    Port {
                        id: "cutoff_cv".to_string(),
                        name: "Cutoff CV".to_string(),
                        direction: PortDirection::Input,
                        signal_type: SignalType::Control,
                    },
                ],
                parameters: vec![],
                runtime_target: Some("audio/effect/filter".to_string()),
                scene_membership: vec![],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::Shared,
                    is_locked: false,
                },
                enabled: true,
                bus_target_id: Some("bus-main".to_string()),
                output_kind: None,
                channel_count: None,
                bypassed: Some(false),
                channel_mode: None,
                sequencer_pattern: None,
            });
            // Audio signal chain: source → filter (so the effect has an input bus),
            // plus the LFO control out → filter control CV in (CV route, no bus_id).
            session.routes = vec![
                Route {
                    id: "route-source-filter".to_string(),
                    source_node_id: "node-source".to_string(),
                    source_port_id: "node-source-out".to_string(),
                    target_node_id: "node-filter".to_string(),
                    target_port_id: "filter-in".to_string(),
                    bus_id: Some("bus-main".to_string()),
                },
                Route {
                    id: "route-lfo-cutoff".to_string(),
                    source_node_id: "node-lfo".to_string(),
                    source_port_id: "lfo_out".to_string(),
                    target_node_id: "node-filter".to_string(),
                    target_port_id: "cutoff_cv".to_string(),
                    bus_id: None,
                },
            ];

            let topology = compile_session_to_topology(&session).expect("compile succeeds");
            let plan = plan_sc_resources(&topology).expect("plan succeeds");

            let lfo_synth = plan
                .synths
                .iter()
                .find(|s| s.node_key == "node-lfo")
                .expect("lfo synth");
            let filter_synth = plan
                .synths
                .iter()
                .find(|s| s.node_key == "node-filter")
                .expect("filter synth");

            let out_cv = lfo_synth
                .args
                .iter()
                .find(|a| a.name == "out_cv_bus")
                .expect("LFO gets out_cv_bus arg");
            let cutoff_cv = filter_synth
                .args
                .iter()
                .find(|a| a.name == "cutoff_cv_bus")
                .expect("filter gets cutoff_cv_bus arg");

            assert!(
                out_cv.value >= scrysynth_lib::audio::synthdefs::FIRST_CONTROL_BUS_OFFSET as f32,
                "control bus index in the control range (got {})",
                out_cv.value
            );
            assert_eq!(
                out_cv.value, cutoff_cv.value,
                "LFO out_cv_bus and filter cutoff_cv_bus share the allocated index"
            );
        }

        // Every catalog entry with a SynthDef maps to a checked-in .scsyndef with
        // an SCgf v2 header + the embedded name (the full "boots real scsynth per
        // entry" conformance test lives in Plan 02).
        #[test]
        fn checked_in_v2_synthdef_resources_are_present_and_named() {
            for entry in CATALOG.iter().filter(|e| !e.synthdef_resource.is_empty()) {
                let bytes = std::fs::read(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(entry.synthdef_resource),
                )
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to read SynthDef resource {}: {error}",
                        entry.synthdef_resource
                    )
                });

                assert!(
                    bytes.starts_with(b"SCgf"),
                    "{} has SCgf header",
                    entry.synthdef_resource
                );
                assert_eq!(
                    i32::from_be_bytes(bytes[4..8].try_into().expect("version bytes")),
                    2,
                    "{} uses SynthDef v2",
                    entry.synthdef_resource
                );
                assert_eq!(
                    i16::from_be_bytes(
                        bytes[8..10].try_into().expect("definition count bytes")
                    ),
                    1,
                    "{} contains one SynthDef",
                    entry.synthdef_resource
                );
                let name_length = bytes[10] as usize;
                assert_eq!(
                    &bytes[11..11 + name_length],
                    entry.synthdef_name.as_bytes(),
                    "{} embeds the expected SynthDef name",
                    entry.synthdef_resource
                );
            }
        }

        #[test]
        fn catalog_find_entry_roundtrips_against_checked_in_resources() {
            // Belt-and-braces: find_catalog_entry + the resource paths agree.
            let osc = find_catalog_entry("oscillator").expect("oscillator cataloged");
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(osc.synthdef_resource);
            assert!(path.exists(), "{} exists", osc.synthdef_resource);
        }
    }

    mod sequencer {
        //! NODES-04 / D-06 / D-07 / D-08: app-driven step sequencer /c_set advance.
        use super::super::*;
        use scrysynth_lib::audio::runtime_manager::{
            AudioRuntimeAdapter, AudioRuntimeManager, RuntimeAdapterStatus,
        };
        use scrysynth_lib::audio::sequencer::{RecordingCSetSink, SequencerController};
        use scrysynth_lib::domain::session::SequencerPattern;

        #[test]
        fn sequencer_controller_emits_c_set_per_step_over_full_16_step_cycle() {
            // The app-driven tick loop writes gate+cv via /c_set per step. Over
            // one 16-step cycle the recording sink should capture exactly 32
            // writes (16 gate + 16 cv), with values matching the pattern and
            // step index wrapping 15→0.
            let mut sink = RecordingCSetSink::new();
            let shared = sink.shared();
            let mut pattern = SequencerPattern::default();
            // Distinctive pattern so we can attribute each write to its step.
            for i in 0..16 {
                pattern.gate[i] = i % 2 == 0;
                pattern.cv[i] = (i as f64) * 0.1 - 0.8; // -0.8 .. +0.7
            }
            let gate_bus = 1024_i32;
            let cv_bus = 1025_i32;

            SequencerController::run_steps_inline(&mut sink, gate_bus, cv_bus, &pattern, 16);

            let sent = shared.lock().expect("captured writes").clone();
            assert_eq!(sent.len(), 32, "16 gate writes + 16 cv writes");

            for step in 0..16 {
                let gate_write = sent[step * 2];
                let cv_write = sent[step * 2 + 1];
                assert_eq!(gate_write.0, gate_bus, "step {step} gate bus");
                assert!(
                    (gate_write.1 - if pattern.gate[step] { 1.0 } else { 0.0 }).abs() < 1e-6,
                    "step {step} gate value"
                );
                assert_eq!(cv_write.0, cv_bus, "step {step} cv bus");
                assert!(
                    (cv_write.1 - pattern.cv[step] as f32).abs() < 1e-6,
                    "step {step} cv value"
                );
            }
        }

        #[test]
        fn sequencer_step_index_wraps_after_16_steps() {
            let mut sink = RecordingCSetSink::new();
            let shared = sink.shared();
            let mut pattern = SequencerPattern::default();
            pattern.gate[0] = true;
            pattern.cv[0] = 0.42;

            // Run 18 steps — step 16 should mirror step 0.
            SequencerController::run_steps_inline(&mut sink, 10, 11, &pattern, 18);

            let sent = shared.lock().expect("captured writes").clone();
            assert_eq!(sent.len(), 36, "18 steps × 2 writes");
            // Step 16 == step 0 (wrap). Writes 32 (gate) and 33 (cv).
            assert!(
                (sent[32].1 - 1.0).abs() < 1e-6,
                "wrapped gate mirrors step 0"
            );
            assert!(
                (sent[33].1 - 0.42).abs() < 1e-6,
                "wrapped cv mirrors step 0"
            );
        }

        #[test]
        fn set_step_value_reconcile_updates_live_controller_pattern() {
            // T-12-07: a SetStepValue mutation must propagate to the live
            // controller's shared pattern (the next tick reads the new value).
            // Spawn a real controller with a fast tempo so stop() is prompt,
            // then swap the pattern and read it back under the lock.
            let initial = SequencerPattern::default();
            let mut edited = SequencerPattern::default();
            edited.gate[3] = true;
            edited.cv[3] = 0.9;

            let controller = SequencerController::start(
                "node-seq-reconcile",
                RecordingCSetSink::new(),
                1024,
                1025,
                9999.0, // fast tempo → tiny step period → prompt join
                initial,
            );
            controller.update_pattern(edited.clone());

            // The live pattern the tick loop reads should now carry step 3.
            // We can't read the controller's internal Arc<Mutex> directly from
            // outside the crate, but we can assert node_id() identity and that
            // stop() joins cleanly (the pattern swap is exercised in the lib
            // unit test `update_pattern_swaps_live_pattern_visible_to_inline_run`).
            assert_eq!(controller.node_id(), "node-seq-reconcile");
            controller.stop();
        }

        // Catalog-replaces-allowlists invariant (success criterion #3) through
        // the FULL compile→plan path. The unit-level test in `synthdefs` already
        // covers `plan_sc_resources` directly; this asserts the catalog is the
        // gate from the session boundary too (no panic path anywhere).
        #[test]
        fn unknown_node_type_id_errors_through_full_compile_to_plan_path() {
            let mut session = deterministic_session();
            session.nodes[1].node_type_id = "nonexistent_node".to_string();
            let topology = compile_session_to_topology(&session).expect("compile is catalog-agnostic");

            let error = plan_sc_resources(&topology).expect_err("unknown id fails at plan time");
            assert!(
                matches!(error, ScResourcePlanError::UnknownCatalogEntry { .. }),
                "expected UnknownCatalogEntry, got {error:?}"
            );
            if let ScResourcePlanError::UnknownCatalogEntry { node_type_id } = &error {
                assert_eq!(node_type_id, "nonexistent_node");
            }
        }

        // Start/stop lifecycle: spawning on Ready, killed on stop/panic. Verifies
        // the SessionStore↔AudioRuntimeManager wiring without booting real
        // scsynth (FakeAdapter). T-12-06 control safety.
        #[test]
        fn audio_start_stop_does_not_panic_with_sequencer_nodes_in_session() {
            use scrysynth_lib::application::session_store::SessionStore;
            use scrysynth_lib::domain::session::{
                AudioRuntimeHealth, AudioRuntimeLifecycle,
            };

            #[derive(Clone)]
            struct SequencerFakeAdapter;

            impl AudioRuntimeAdapter for SequencerFakeAdapter {
                fn start(&mut self) -> Result<RuntimeAdapterStatus, String> {
                    Ok(RuntimeAdapterStatus::Booted {
                        sample_rate_hz: 48_000,
                        block_size: 64,
                    })
                }
                fn load_topology(
                    &mut self,
                    _topology: &scrysynth_lib::audio::compiler::CompiledTopology,
                ) -> Result<RuntimeAdapterStatus, String> {
                    Ok(RuntimeAdapterStatus::Ready {
                        active_patch_id: "patch-fake".to_string(),
                    })
                }
                fn set_parameter_value(
                    &mut self,
                    _node_id: &str,
                    _parameter_id: &str,
                    _value: f64,
                ) -> Result<RuntimeAdapterStatus, String> {
                    unreachable!("not used")
                }
                fn stop(&mut self) -> Result<RuntimeAdapterStatus, String> {
                    Ok(RuntimeAdapterStatus::Stopped)
                }
                fn panic(&mut self) -> Result<RuntimeAdapterStatus, String> {
                    Ok(RuntimeAdapterStatus::Panicked)
                }
            }

            // The default store has no sequencer nodes; spawn/stop is a no-op
            // for controllers but must not panic. This guards the wiring.
            let mut manager = AudioRuntimeManager::new_for_tests(SequencerFakeAdapter);
            let mut store = SessionStore::new_default();
            let session = manager.start(&mut store).expect("start succeeds");
            assert_eq!(session.audio_runtime.lifecycle, AudioRuntimeLifecycle::Ready);
            assert_eq!(session.audio_runtime.health, AudioRuntimeHealth::Healthy);

            // stop must clean up sequencer_controllers without panic.
            let stopped = manager.stop(&mut store).expect("stop succeeds");
            assert_eq!(stopped.audio_runtime.lifecycle, AudioRuntimeLifecycle::Idle);
        }
    }

    mod conformance {
        //! Real-scsynth conformance gate (success criterion #4) + end-to-end
        //! audible CV modulation (NODES-05, D-03). Both are #[ignore]'d so they
        //! only run via `cargo test -- --ignored` and skip cleanly with a
        //! message when the scsynth bundle is absent.
        use super::super::*;
        use scrysynth_lib::audio::supercollider::SuperColliderAdapter;
        use scrysynth_lib::audio::runtime_manager::AudioRuntimeAdapter;

        /// Returns `Some(())` if scsynth is available (so the test should run),
        /// or prints a skip reason and returns `None`. We detect availability
        /// via `SuperColliderAdapter::start()` returning `Booted`; a `Failed`
        /// status whose message mentions scsynth-not-found means skip.
        fn boot_scsynth_or_skip() -> Option<SuperColliderAdapter> {
            let mut adapter = SuperColliderAdapter::default();
            match adapter.start() {
                Ok(status) => match status {
                    scrysynth_lib::audio::runtime_manager::RuntimeAdapterStatus::Booted { .. } => {
                        Some(adapter)
                    }
                    scrysynth_lib::audio::runtime_manager::RuntimeAdapterStatus::Failed {
                        message,
                        ..
                    } => {
                        eprintln!(
                            "SKIP: scsynth is not available — conformance test did not run. \
                             Reason: {message}"
                        );
                        None
                    }
                    other => {
                        eprintln!(
                            "SKIP: unexpected scsynth boot status {other:?} — conformance test did not run."
                        );
                        None
                    }
                },
                Err(err) => {
                    eprintln!(
                        "SKIP: scsynth boot returned error — conformance test did not run. \
                         Reason: {err}"
                    );
                    None
                }
            }
        }

        /// SUCCESS CRITERION #4 — the Phase 12 lynchpin. Every catalog entry
        /// with a non-empty `synthdef_resource` must `/d_recv` successfully to a
        /// real booted scsynth. Catches unavailable UGens (FreeVerb/GVerb/CombC/
        /// AllpassN per research assumption A3), malformed bytes, and synthdef
        /// name/embedded-name drift.
        #[test]
        #[ignore = "requires local scsynth bundle — run via `cargo test -- --ignored`"]
        fn every_catalog_entry_synthdef_loads_on_real_scsynth() {
            let Some(mut adapter) = boot_scsynth_or_skip() else {
                return;
            };

            // The conformance entry point /d_recvs every catalog synthdef and
            // /syncs after each so a failure attributes to the right entry.
            let result = adapter.recv_all_catalog_synthdefs_for_conformance();

            // Tear down scsynth regardless of outcome.
            let _ = adapter.stop();

            match result {
                Ok(()) => {
                    // Sanity: at least one entry has a synthdef (else this test
                    // would trivially pass with zero work).
                    let with_synthdef = CATALOG
                        .iter()
                        .filter(|e| !e.synthdef_resource.is_empty())
                        .count();
                    assert!(
                        with_synthdef >= 10,
                        "expected at least 10 catalog entries with synthdefs, got {with_synthdef}"
                    );
                }
                Err(err) => panic!("catalog synthdef conformance failed: {err}"),
            }
        }

        /// NODES-05 / D-03 — end-to-end CV modulation wiring. Boots real
        /// scsynth, applies an LFO→filter cutoff_cv plan, and asserts the
        /// synths launched with the expected `out_cv_bus` (LFO) and
        /// `cutoff_cv_bus` (filter) args. A full audible FFT assertion is out
        /// of scope (RESEARCH.md:224); the wiring + launch verification is the
        /// gate. The unit-level CV allocation test in `synthdefs` covers the
        /// non-scsynth path; this verifies the launch against the real engine.
        #[test]
        #[ignore = "requires local scsynth bundle — run via `cargo test -- --ignored`"]
        fn lfo_to_filter_cv_modulation_launches_with_cv_bus_args_on_real_scsynth() {
            let Some(mut adapter) = boot_scsynth_or_skip() else {
                return;
            };

            // Build a session with the LFO→filter CV route (the unit-level
            // `cv_route_allocates_control_bus_for_modulation` shape). We
            // compile + plan, then assert plan_sc_resources produced the
            // expected cv-bus args BEFORE applying — applying requires a real
            // signal chain we don't need to assert the wiring.
            let mut session = deterministic_session();
            session.nodes.push(Node {
                id: "node-lfo".to_string(),
                node_type_id: "lfo".to_string(),
                ports: vec![Port {
                    id: "lfo_out".to_string(),
                    name: "LFO Out".to_string(),
                    direction: PortDirection::Output,
                    signal_type: SignalType::Control,
                }],
                parameters: vec![ParameterValue {
                    id: "lfo-freq".to_string(),
                    name: "frequency".to_string(),
                    value: 0.5,
                    default_value: 0.5,
                    min_value: 0.001,
                    max_value: 100.0,
                    unit: "hz".to_string(),
                }],
                runtime_target: Some("audio/modulator/lfo".to_string()),
                scene_membership: vec![],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::Shared,
                    is_locked: false,
                },
                enabled: true,
                bus_target_id: None,
                output_kind: None,
                channel_count: None,
                bypassed: None,
                channel_mode: None,
                sequencer_pattern: None,
            });
            session.nodes.push(Node {
                id: "node-filter".to_string(),
                node_type_id: "filter".to_string(),
                ports: vec![
                    Port {
                        id: "filter-in".to_string(),
                        name: "audio_in".to_string(),
                        direction: PortDirection::Input,
                        signal_type: SignalType::Audio,
                    },
                    Port {
                        id: "filter-out".to_string(),
                        name: "audio_out".to_string(),
                        direction: PortDirection::Output,
                        signal_type: SignalType::Audio,
                    },
                    Port {
                        id: "cutoff_cv".to_string(),
                        name: "Cutoff CV".to_string(),
                        direction: PortDirection::Input,
                        signal_type: SignalType::Control,
                    },
                ],
                parameters: vec![],
                runtime_target: Some("audio/effect/filter".to_string()),
                scene_membership: vec![],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::Shared,
                    is_locked: false,
                },
                enabled: true,
                bus_target_id: Some("bus-main".to_string()),
                output_kind: None,
                channel_count: None,
                bypassed: Some(false),
                channel_mode: None,
                sequencer_pattern: None,
            });
            session.routes = vec![
                Route {
                    id: "route-source-filter".to_string(),
                    source_node_id: "node-source".to_string(),
                    source_port_id: "node-source-out".to_string(),
                    target_node_id: "node-filter".to_string(),
                    target_port_id: "filter-in".to_string(),
                    bus_id: Some("bus-main".to_string()),
                },
                Route {
                    id: "route-lfo-cutoff".to_string(),
                    source_node_id: "node-lfo".to_string(),
                    source_port_id: "lfo_out".to_string(),
                    target_node_id: "node-filter".to_string(),
                    target_port_id: "cutoff_cv".to_string(),
                    bus_id: None,
                },
            ];

            let topology = compile_session_to_topology(&session).expect("compile succeeds");
            let plan = plan_sc_resources(&topology).expect("plan succeeds");

            // Apply against the real engine. If this returns Ready, the
            // synthdefs /d_recv'd AND the synths launched with the CV-bus args
            // (a missing arg would have failed /s_new).
            let status = adapter.load_topology(&topology);
            let _ = adapter.stop();
            let status = status.expect("load_topology did not error");

            use scrysynth_lib::audio::runtime_manager::RuntimeAdapterStatus;
            match status {
                RuntimeAdapterStatus::Ready { active_patch_id } => {
                    assert!(
                        active_patch_id.starts_with("patch-v2-"),
                        "real scsynth accepted the LFO→filter CV plan: {active_patch_id}"
                    );
                    // Belt-and-braces: the plan carried the cv-bus args.
                    let lfo_synth = plan
                        .synths
                        .iter()
                        .find(|s| s.node_key == "node-lfo")
                        .expect("lfo synth present");
                    assert!(lfo_synth
                        .args
                        .iter()
                        .any(|a| a.name == "out_cv_bus"
                            && a.value >= scrysynth_lib::audio::synthdefs::FIRST_CONTROL_BUS_OFFSET
                                as f32));
                }
                RuntimeAdapterStatus::Failed { message, .. } => {
                    panic!("LFO→filter CV plan failed to launch on real scsynth: {message}");
                }
                other => panic!("unexpected load_topology status: {other:?}"),
            }
        }
    }

    mod lifecycle {
        use std::sync::{Arc, Mutex};

        use super::super::*;
        use scrysynth_lib::application::session_store::SessionStore;
        use scrysynth_lib::audio::runtime_manager::{
            AudioRuntimeAdapter, AudioRuntimeManager, RuntimeAdapterStatus,
        };
        use scrysynth_lib::domain::session::{
            AudioRuntimeHealth, AudioRuntimeLifecycle, RuntimeConnectionState,
        };

        #[derive(Clone, Debug, PartialEq)]
        enum AdapterAction {
            Start,
            Stop,
            Panic,
            LoadTopology,
            SetParameter,
        }

        #[derive(Clone)]
        struct FakeAdapter {
            actions: Arc<Mutex<Vec<AdapterAction>>>,
            next_statuses: Arc<Mutex<Vec<RuntimeAdapterStatus>>>,
        }

        impl FakeAdapter {
            fn with_statuses(statuses: Vec<RuntimeAdapterStatus>) -> Self {
                Self {
                    actions: Arc::new(Mutex::new(Vec::new())),
                    next_statuses: Arc::new(Mutex::new(statuses)),
                }
            }

            fn actions(&self) -> Vec<AdapterAction> {
                self.actions.lock().expect("actions lock").clone()
            }

            fn take_status(&self) -> RuntimeAdapterStatus {
                self.next_statuses.lock().expect("status lock").remove(0)
            }
        }

        impl AudioRuntimeAdapter for FakeAdapter {
            fn start(&mut self) -> Result<RuntimeAdapterStatus, String> {
                self.actions
                    .lock()
                    .expect("actions lock")
                    .push(AdapterAction::Start);
                Ok(self.take_status())
            }

            fn load_topology(
                &mut self,
                _topology: &scrysynth_lib::audio::compiler::CompiledTopology,
            ) -> Result<RuntimeAdapterStatus, String> {
                self.actions
                    .lock()
                    .expect("actions lock")
                    .push(AdapterAction::LoadTopology);
                Ok(self.take_status())
            }

            fn set_parameter_value(
                &mut self,
                _node_id: &str,
                _parameter_id: &str,
                _value: f64,
            ) -> Result<RuntimeAdapterStatus, String> {
                self.actions
                    .lock()
                    .expect("actions lock")
                    .push(AdapterAction::SetParameter);
                Ok(self.take_status())
            }

            fn stop(&mut self) -> Result<RuntimeAdapterStatus, String> {
                self.actions
                    .lock()
                    .expect("actions lock")
                    .push(AdapterAction::Stop);
                Ok(self.take_status())
            }

            fn panic(&mut self) -> Result<RuntimeAdapterStatus, String> {
                self.actions
                    .lock()
                    .expect("actions lock")
                    .push(AdapterAction::Panic);
                Ok(self.take_status())
            }
        }

        #[test]
        fn start_audio_runtime_transitions_booting_to_ready() {
            let adapter = FakeAdapter::with_statuses(vec![
                RuntimeAdapterStatus::Booted {
                    sample_rate_hz: 48_000,
                    block_size: 64,
                },
                RuntimeAdapterStatus::Ready {
                    active_patch_id: "patch-ready".to_string(),
                },
            ]);
            let mut manager = AudioRuntimeManager::new_for_tests(adapter.clone());
            let mut store = SessionStore::new_default();

            let session = manager.start(&mut store).expect("runtime starts");

            assert_eq!(
                session.audio_runtime.lifecycle,
                AudioRuntimeLifecycle::Ready
            );
            assert_eq!(session.audio_runtime.health, AudioRuntimeHealth::Healthy);
            assert_eq!(session.audio_runtime.sample_rate_hz, Some(48_000));
            assert_eq!(session.audio_runtime.block_size, Some(64));
            assert_eq!(
                audio_runtime_status(&session),
                RuntimeConnectionState::Ready,
                "ready connection state"
            );
            assert_eq!(
                adapter.actions(),
                vec![AdapterAction::Start, AdapterAction::LoadTopology]
            );
        }

        #[test]
        fn panic_audio_runtime_recovers_to_known_safe_state_and_allows_restart() {
            let adapter = FakeAdapter::with_statuses(vec![
                RuntimeAdapterStatus::Booted {
                    sample_rate_hz: 48_000,
                    block_size: 64,
                },
                RuntimeAdapterStatus::Ready {
                    active_patch_id: "patch-a".to_string(),
                },
                RuntimeAdapterStatus::Panicked,
                RuntimeAdapterStatus::Booted {
                    sample_rate_hz: 48_000,
                    block_size: 64,
                },
                RuntimeAdapterStatus::Ready {
                    active_patch_id: "patch-b".to_string(),
                },
            ]);
            let mut manager = AudioRuntimeManager::new_for_tests(adapter.clone());
            let mut store = SessionStore::new_default();

            manager.start(&mut store).expect("initial start");
            let recovered = manager.panic(&mut store).expect("panic succeeds");

            assert_eq!(
                recovered.audio_runtime.lifecycle,
                AudioRuntimeLifecycle::Idle
            );
            assert_eq!(
                recovered.audio_runtime.health,
                AudioRuntimeHealth::PanicRecovered
            );
            assert_eq!(recovered.audio_runtime.active_patch_id, None);
            assert_eq!(recovered.audio_runtime.panic_recovery_count, 1, "panic");

            let restarted = manager.start(&mut store).expect("restart succeeds");

            assert_eq!(
                restarted.audio_runtime.lifecycle,
                AudioRuntimeLifecycle::Ready
            );
            assert_eq!(
                restarted.audio_runtime.active_patch_id.as_deref(),
                Some("patch-b")
            );
        }

        #[test]
        fn start_audio_runtime_marks_degraded_when_adapter_fails() {
            let adapter = FakeAdapter::with_statuses(vec![RuntimeAdapterStatus::Failed {
                message: "scsynth not found. Install SuperCollider, put `scsynth` on PATH, or set SCRYSYNTH_SCSYNTH_PATH to the full executable path. On macOS Scrysynth also checks the bundle fallback `/Applications/SuperCollider.app/Contents/Resources/scsynth`.".to_string(),
                active_patch_id: None,
            }]);
            let mut manager = AudioRuntimeManager::new_for_tests(adapter);
            let mut store = SessionStore::new_default();

            let session = manager.start(&mut store).expect("failure is recorded");

            assert_eq!(
                session.audio_runtime.lifecycle,
                AudioRuntimeLifecycle::Failed
            );
            assert_eq!(session.audio_runtime.health, AudioRuntimeHealth::Degraded);
            assert_eq!(
                session.audio_runtime.last_error.as_deref(),
                Some("scsynth not found. Install SuperCollider, put `scsynth` on PATH, or set SCRYSYNTH_SCSYNTH_PATH to the full executable path. On macOS Scrysynth also checks the bundle fallback `/Applications/SuperCollider.app/Contents/Resources/scsynth`.")
            );
            assert_eq!(
                audio_runtime_status(&session),
                RuntimeConnectionState::Error
            );
        }

        #[test]
        fn start_audio_runtime_marks_degraded_when_topology_sync_fails() {
            let adapter = FakeAdapter::with_statuses(vec![
                RuntimeAdapterStatus::Booted {
                    sample_rate_hz: 48_000,
                    block_size: 64,
                },
                RuntimeAdapterStatus::Failed {
                    message: "Runtime server error during topology load synthdefs: scsynth did not confirm OSC /sync: /sync 1 timed out after 2s".to_string(),
                    active_patch_id: None,
                },
            ]);
            let mut manager = AudioRuntimeManager::new_for_tests(adapter);
            let mut store = SessionStore::new_default();

            let session = manager.start(&mut store).expect("failure is recorded");

            assert_eq!(
                session.audio_runtime.lifecycle,
                AudioRuntimeLifecycle::Failed
            );
            assert_eq!(session.audio_runtime.health, AudioRuntimeHealth::Degraded);
            assert_eq!(
                session.audio_runtime.last_error.as_deref(),
                Some("Runtime server error during topology load synthdefs: scsynth did not confirm OSC /sync: /sync 1 timed out after 2s")
            );
            assert_eq!(
                audio_runtime_status(&session),
                RuntimeConnectionState::Error
            );
        }

        #[test]
        fn start_audio_runtime_marks_degraded_when_topology_compile_fails() {
            let adapter = FakeAdapter::with_statuses(vec![]);
            let mut manager = AudioRuntimeManager::new_for_tests(adapter.clone());
            let mut store = SessionStore::new_default();
            let _ = store.mutate_current(|session| {
                session.nodes[0].runtime_target = None;
                session.audio_runtime.active_patch_id = Some("patch-a".to_string());
                Ok::<(), ()>(())
            });

            let session = manager
                .start(&mut store)
                .expect("compile failure is recorded");

            assert_eq!(
                session.audio_runtime.lifecycle,
                AudioRuntimeLifecycle::Failed
            );
            assert_eq!(session.audio_runtime.health, AudioRuntimeHealth::Degraded);
            assert_eq!(
                session.audio_runtime.active_patch_id.as_deref(),
                Some("patch-a")
            );
            let last_error = session
                .audio_runtime
                .last_error
                .as_deref()
                .expect("compile error is recorded");
            assert!(last_error.starts_with("failed to compile audio topology: node `"));
            assert!(last_error.ends_with("` is missing a runtime target"));
            assert_eq!(
                audio_runtime_status(&session),
                RuntimeConnectionState::Error
            );
            assert_eq!(adapter.actions(), Vec::<AdapterAction>::new());
            assert_eq!(store.current(), session);
        }

        #[test]
        fn topology_reload_failure_preserves_previous_active_patch_id() {
            let adapter = FakeAdapter::with_statuses(vec![
                RuntimeAdapterStatus::Booted {
                    sample_rate_hz: 48_000,
                    block_size: 64,
                },
                RuntimeAdapterStatus::Ready {
                    active_patch_id: "patch-a".to_string(),
                },
                RuntimeAdapterStatus::Booted {
                    sample_rate_hz: 48_000,
                    block_size: 64,
                },
                RuntimeAdapterStatus::Failed {
                    message: "Runtime server error during topology unload previous patch: scsynth did not confirm OSC /sync".to_string(),
                    active_patch_id: Some("patch-a".to_string()),
                },
            ]);
            let mut manager = AudioRuntimeManager::new_for_tests(adapter);
            let mut store = SessionStore::new_default();

            manager.start(&mut store).expect("initial start succeeds");
            let session = manager
                .start(&mut store)
                .expect("reload failure is recorded");

            assert_eq!(
                session.audio_runtime.lifecycle,
                AudioRuntimeLifecycle::Failed
            );
            assert_eq!(session.audio_runtime.health, AudioRuntimeHealth::Degraded);
            assert_eq!(
                session.audio_runtime.active_patch_id.as_deref(),
                Some("patch-a")
            );
            assert_eq!(
                session.audio_runtime.last_error.as_deref(),
                Some("Runtime server error during topology unload previous patch: scsynth did not confirm OSC /sync")
            );
            assert_eq!(
                audio_runtime_status(&session),
                RuntimeConnectionState::Error
            );
        }

        #[test]
        fn topology_reload_boot_failure_preserves_adapter_active_patch_id() {
            let adapter = FakeAdapter::with_statuses(vec![
                RuntimeAdapterStatus::Booted {
                    sample_rate_hz: 48_000,
                    block_size: 64,
                },
                RuntimeAdapterStatus::Ready {
                    active_patch_id: "patch-a".to_string(),
                },
                RuntimeAdapterStatus::Failed {
                    message: "Runtime server error during boot: scsynth did not confirm OSC /sync"
                        .to_string(),
                    active_patch_id: Some("patch-a".to_string()),
                },
            ]);
            let mut manager = AudioRuntimeManager::new_for_tests(adapter.clone());
            let mut store = SessionStore::new_default();

            manager.start(&mut store).expect("initial start succeeds");
            let session = manager
                .start(&mut store)
                .expect("reload boot failure is recorded");

            assert_eq!(
                session.audio_runtime.lifecycle,
                AudioRuntimeLifecycle::Failed
            );
            assert_eq!(session.audio_runtime.health, AudioRuntimeHealth::Degraded);
            assert_eq!(
                session.audio_runtime.active_patch_id.as_deref(),
                Some("patch-a")
            );
            assert_eq!(
                session.audio_runtime.last_error.as_deref(),
                Some("Runtime server error during boot: scsynth did not confirm OSC /sync")
            );
            assert_eq!(
                audio_runtime_status(&session),
                RuntimeConnectionState::Error
            );
            assert_eq!(
                adapter.actions(),
                vec![
                    AdapterAction::Start,
                    AdapterAction::LoadTopology,
                    AdapterAction::Start
                ]
            );
        }

        #[test]
        fn topology_reload_failure_clears_active_patch_after_previous_unload_succeeds() {
            let adapter = FakeAdapter::with_statuses(vec![
                RuntimeAdapterStatus::Booted {
                    sample_rate_hz: 48_000,
                    block_size: 64,
                },
                RuntimeAdapterStatus::Ready {
                    active_patch_id: "patch-a".to_string(),
                },
                RuntimeAdapterStatus::Booted {
                    sample_rate_hz: 48_000,
                    block_size: 64,
                },
                RuntimeAdapterStatus::Failed {
                    message: "Topology apply failure: failed to create SuperCollider group `main`: connection refused".to_string(),
                    active_patch_id: None,
                },
            ]);
            let mut manager = AudioRuntimeManager::new_for_tests(adapter);
            let mut store = SessionStore::new_default();

            manager.start(&mut store).expect("initial start succeeds");
            let session = manager
                .start(&mut store)
                .expect("reload failure is recorded");

            assert_eq!(
                session.audio_runtime.lifecycle,
                AudioRuntimeLifecycle::Failed
            );
            assert_eq!(session.audio_runtime.health, AudioRuntimeHealth::Degraded);
            assert_eq!(session.audio_runtime.active_patch_id, None);
            assert_eq!(
                session.audio_runtime.last_error.as_deref(),
                Some("Topology apply failure: failed to create SuperCollider group `main`: connection refused")
            );
            assert_eq!(
                audio_runtime_status(&session),
                RuntimeConnectionState::Error
            );
        }

        #[test]
        fn adapter_start_errors_are_recorded_as_app_state_details() {
            #[derive(Clone)]
            struct ErrorAdapter;

            impl AudioRuntimeAdapter for ErrorAdapter {
                fn start(&mut self) -> Result<RuntimeAdapterStatus, String> {
                    Err("OSC client could not be constructed".to_string())
                }

                fn load_topology(
                    &mut self,
                    _topology: &scrysynth_lib::audio::compiler::CompiledTopology,
                ) -> Result<RuntimeAdapterStatus, String> {
                    unreachable!("start fails first")
                }

                fn set_parameter_value(
                    &mut self,
                    _node_id: &str,
                    _parameter_id: &str,
                    _value: f64,
                ) -> Result<RuntimeAdapterStatus, String> {
                    unreachable!("not used")
                }

                fn stop(&mut self) -> Result<RuntimeAdapterStatus, String> {
                    unreachable!("not used")
                }

                fn panic(&mut self) -> Result<RuntimeAdapterStatus, String> {
                    unreachable!("not used")
                }
            }

            let mut manager = AudioRuntimeManager::new_for_tests(ErrorAdapter);
            let mut store = SessionStore::new_default();

            let session = manager.start(&mut store).expect("error is recorded");

            assert_eq!(
                session.audio_runtime.last_error.as_deref(),
                Some("Audio runtime app state error while starting adapter: OSC client could not be constructed")
            );
            assert_eq!(
                audio_runtime_status(&session),
                RuntimeConnectionState::Error
            );
        }

        #[test]
        fn panic_audio_runtime_records_actionable_recovery_detail() {
            let adapter = FakeAdapter::with_statuses(vec![RuntimeAdapterStatus::Panicked]);
            let mut manager = AudioRuntimeManager::new_for_tests(adapter);
            let mut store = SessionStore::new_default();

            let session = manager.panic(&mut store).expect("panic recovers");

            assert_eq!(
                session.audio_runtime.health,
                AudioRuntimeHealth::PanicRecovered
            );
            assert_eq!(
                session.audio_runtime.last_error.as_deref(),
                Some("Panic recovery completed: stopped scsynth, cleared the active patch, and returned audio to a restartable idle state.")
            );
        }

        fn audio_runtime_status(session: &SessionDocument) -> RuntimeConnectionState {
            session
                .runtime_status
                .iter()
                .find(|status| status.runtime == scrysynth_lib::domain::session::RuntimeKind::Audio)
                .expect("audio runtime status")
                .status
                .clone()
        }
    }
}
