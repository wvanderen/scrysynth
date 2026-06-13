use scrysynth_lib::domain::session::{
    AudioBusType, AudioEffectNode, AudioEffectType, AudioMixerNode, AudioOutputNode,
    AudioOutputType, AudioPrimitive, AudioSourceNode, AudioSourceType, Bus, ChannelMode,
    ControllerKind, Node, NodeType, OwnershipAssignment, ParameterValue, Port, PortDirection,
    Route, SessionDocument, SignalType,
};

use scrysynth_lib::audio::compiler::{compile_session_to_topology, CompiledNodeKind};

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
        fn topology_compilation_preserves_concrete_audio_primitives() {
            let mut session = deterministic_session();
            session.nodes.push(Node {
                id: "node-delay".to_string(),
                node_type: NodeType::Effect,
                ports: vec![
                    Port {
                        id: "port-delay-in".to_string(),
                        name: "delay_in".to_string(),
                        direction: PortDirection::Input,
                        signal_type: SignalType::Audio,
                    },
                    Port {
                        id: "port-delay-out".to_string(),
                        name: "delay_out".to_string(),
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
                audio_primitive: Some(AudioPrimitive::Effect(AudioEffectNode {
                    effect_type: AudioEffectType::Delay,
                    bypassed: true,
                    bus_target_id: Some("bus-main".to_string()),
                })),
            });
            session.routes = vec![
                Route {
                    id: "route-source-delay".to_string(),
                    source_node_id: "node-source".to_string(),
                    source_port_id: "port-source-out".to_string(),
                    target_node_id: "node-delay".to_string(),
                    target_port_id: "port-delay-in".to_string(),
                    bus_id: Some("bus-main".to_string()),
                },
                Route {
                    id: "route-delay-output".to_string(),
                    source_node_id: "node-delay".to_string(),
                    source_port_id: "port-delay-out".to_string(),
                    target_node_id: "node-output".to_string(),
                    target_port_id: "port-output-in".to_string(),
                    bus_id: Some("bus-main".to_string()),
                },
            ];

            let topology = compile_session_to_topology(&session).expect("compile succeeds");

            assert!(matches!(
                topology.node_launch_order[0].node_kind,
                CompiledNodeKind::Source {
                    source_type: AudioSourceType::Oscillator,
                    channel_mode: ChannelMode::Mono
                }
            ));
            assert!(matches!(
                topology.node_launch_order[1].node_kind,
                CompiledNodeKind::Effect {
                    effect_type: AudioEffectType::Delay,
                    bypassed: true
                }
            ));
            assert!(matches!(
                topology.node_launch_order[2].node_kind,
                CompiledNodeKind::Output {
                    output_type: AudioOutputType::Master,
                    channels: 2
                }
            ));
            assert_eq!(
                topology.node_launch_order[0].parameters[0].name,
                "frequency"
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

    mod synthdefs {
        use super::super::*;
        use scrysynth_lib::audio::synthdefs::{
            plan_sc_resources, FX_DELAY_SYNTHDEF, FX_LOWPASS_SYNTHDEF, MIXER_SYNTHDEF,
            OUTPUT_SYNTHDEF, SOURCE_NOISE_SYNTHDEF, SOURCE_OSCILLATOR_SYNTHDEF,
        };

        #[test]
        fn resource_plan_maps_supported_primitives_to_known_synthdefs_and_params() {
            let mut session = deterministic_session();
            session.buses.push(Bus {
                id: "bus-mix".to_string(),
                name: "mix".to_string(),
                channels: 2,
                bus_type: AudioBusType::Auxiliary,
                is_enabled: true,
            });
            session.nodes.push(Node {
                id: "node-mixer".to_string(),
                node_type: NodeType::Mixer,
                ports: vec![
                    Port {
                        id: "port-mixer-in".to_string(),
                        name: "mix_in".to_string(),
                        direction: PortDirection::Input,
                        signal_type: SignalType::Audio,
                    },
                    Port {
                        id: "port-mixer-out".to_string(),
                        name: "mix_out".to_string(),
                        direction: PortDirection::Output,
                        signal_type: SignalType::Audio,
                    },
                ],
                parameters: vec![ParameterValue {
                    id: "param-gain".to_string(),
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
                audio_primitive: Some(AudioPrimitive::Mixer(AudioMixerNode {
                    channel_mode: ChannelMode::Stereo,
                    bus_target_id: Some("bus-mix".to_string()),
                })),
            });
            session.routes = vec![
                Route {
                    id: "route-source-mixer".to_string(),
                    source_node_id: "node-source".to_string(),
                    source_port_id: "port-source-out".to_string(),
                    target_node_id: "node-mixer".to_string(),
                    target_port_id: "port-mixer-in".to_string(),
                    bus_id: Some("bus-main".to_string()),
                },
                Route {
                    id: "route-mixer-output".to_string(),
                    source_node_id: "node-mixer".to_string(),
                    source_port_id: "port-mixer-out".to_string(),
                    target_node_id: "node-output".to_string(),
                    target_port_id: "port-output-in".to_string(),
                    bus_id: Some("bus-mix".to_string()),
                },
            ];

            let topology = compile_session_to_topology(&session).expect("compile succeeds");
            let plan = plan_sc_resources(&topology).expect("plan succeeds");

            assert_eq!(plan.patch_id, "patch-v1-2-1-3");
            assert_eq!(plan.groups[0].node_id, 1000);
            assert_eq!(
                plan.synthdefs
                    .iter()
                    .map(|resource| resource.name)
                    .collect::<Vec<_>>(),
                vec![MIXER_SYNTHDEF, OUTPUT_SYNTHDEF, SOURCE_OSCILLATOR_SYNTHDEF]
            );
            assert_eq!(plan.synths[0].synthdef_name, SOURCE_OSCILLATOR_SYNTHDEF);
            assert_eq!(plan.synths[0].node_id, 2000);
            assert_eq!(plan.synths[1].synthdef_name, MIXER_SYNTHDEF);
            assert!(plan.synths[1]
                .args
                .iter()
                .any(|arg| arg.name == "in_bus_1" && arg.value == 2.0));
            assert!(plan.synths[1]
                .args
                .iter()
                .any(|arg| arg.name == "level" && arg.value == 0.75));
            assert_eq!(plan.synths[2].synthdef_name, OUTPUT_SYNTHDEF);
            assert!(plan.controls.iter().any(|control| control.control_key
                == "node-source:param-frequency"
                && control.parameter_name == "frequency"));
            assert!(plan
                .controls
                .iter()
                .any(|control| control.control_key == "node-mixer:param-gain"
                    && control.parameter_name == "level"));
        }

        #[test]
        fn resource_plan_fails_loudly_for_unsupported_runtime_target() {
            let mut session = deterministic_session();
            session.nodes[0].runtime_target = Some("visual/output/master".to_string());
            let topology = compile_session_to_topology(&session).expect("compile succeeds");

            let error = plan_sc_resources(&topology).expect_err("unsupported target fails");

            assert!(error.to_string().contains("visual/output/master"));
        }

        #[test]
        fn resource_plan_accepts_known_legacy_runtime_target_aliases() {
            let mut session = deterministic_session();
            session.nodes[1].runtime_target = Some("audio/source/default".to_string());
            let topology = compile_session_to_topology(&session).expect("compile succeeds");

            let plan = plan_sc_resources(&topology).expect("legacy target alias succeeds");

            assert_eq!(plan.synths[0].synthdef_name, SOURCE_OSCILLATOR_SYNTHDEF);
        }

        #[test]
        fn resource_plan_fails_loudly_for_unknown_audio_runtime_target() {
            let mut session = deterministic_session();
            session.nodes[1].runtime_target = Some("audio/source/granular".to_string());
            let topology = compile_session_to_topology(&session).expect("compile succeeds");

            let error = plan_sc_resources(&topology).expect_err("unsupported audio target fails");

            assert!(error.to_string().contains("audio/source/granular"));
        }

        #[test]
        fn resource_plan_fails_loudly_for_mismatched_runtime_target_and_primitive() {
            let mut session = deterministic_session();
            session.nodes[1].runtime_target = Some("audio/source/noise".to_string());
            let topology = compile_session_to_topology(&session).expect("compile succeeds");

            let error = plan_sc_resources(&topology).expect_err("mismatched target fails");

            assert!(error.to_string().contains("audio/source/noise"));
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
        fn delay_primitive_uses_delay_synthdef_and_stable_parameter_names() {
            let mut session = deterministic_session();
            session.nodes[0].audio_primitive = Some(AudioPrimitive::Effect(AudioEffectNode {
                effect_type: AudioEffectType::Delay,
                bypassed: false,
                bus_target_id: Some("bus-main".to_string()),
            }));
            session.nodes[0].node_type = NodeType::Effect;
            session.nodes[0].runtime_target = Some("audio/effect/delay".to_string());
            session.nodes[0].parameters = vec![ParameterValue {
                id: "param-delay-time".to_string(),
                name: "delayTime".to_string(),
                value: 0.5,
                default_value: 0.25,
                min_value: 0.0,
                max_value: 2.0,
                unit: "s".to_string(),
            }];

            let topology = compile_session_to_topology(&session).expect("compile succeeds");
            let plan = plan_sc_resources(&topology).expect("plan succeeds");

            assert_eq!(plan.synths[1].synthdef_name, FX_DELAY_SYNTHDEF);
            assert!(plan.synths[1]
                .args
                .iter()
                .any(|arg| arg.name == "delay_time_s" && arg.value == 0.5));
        }

        #[test]
        fn checked_in_v1_synthdef_resources_are_present_and_named() {
            let resources = [
                (
                    SOURCE_OSCILLATOR_SYNTHDEF,
                    "resources/synthdefs/v1/scrysynth_v1_source_oscillator.scsyndef",
                ),
                (
                    SOURCE_NOISE_SYNTHDEF,
                    "resources/synthdefs/v1/scrysynth_v1_source_noise.scsyndef",
                ),
                (
                    FX_LOWPASS_SYNTHDEF,
                    "resources/synthdefs/v1/scrysynth_v1_fx_lowpass.scsyndef",
                ),
                (
                    FX_DELAY_SYNTHDEF,
                    "resources/synthdefs/v1/scrysynth_v1_fx_delay.scsyndef",
                ),
                (
                    MIXER_SYNTHDEF,
                    "resources/synthdefs/v1/scrysynth_v1_mixer.scsyndef",
                ),
                (
                    OUTPUT_SYNTHDEF,
                    "resources/synthdefs/v1/scrysynth_v1_output.scsyndef",
                ),
            ];

            for (name, relative_path) in resources {
                let bytes = std::fs::read(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path),
                )
                .unwrap_or_else(|error| {
                    panic!("failed to read SynthDef resource {relative_path}: {error}")
                });

                assert!(
                    bytes.starts_with(b"SCgf"),
                    "{relative_path} has SCgf header"
                );
                assert_eq!(
                    i32::from_be_bytes(bytes[4..8].try_into().expect("version bytes")),
                    2,
                    "{relative_path} uses SynthDef v2"
                );
                assert_eq!(
                    i16::from_be_bytes(bytes[8..10].try_into().expect("definition count bytes")),
                    1,
                    "{relative_path} contains one SynthDef"
                );
                let name_length = bytes[10] as usize;
                assert_eq!(
                    &bytes[11..11 + name_length],
                    name.as_bytes(),
                    "{relative_path} embeds the expected SynthDef name"
                );
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
                message: "scsynth not found".to_string(),
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
                Some("scsynth not found")
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
                    message: "topology load: scsynth /sync failed: /sync 1 timed out".to_string(),
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
                Some("topology load: scsynth /sync failed: /sync 1 timed out")
            );
            assert_eq!(
                audio_runtime_status(&session),
                RuntimeConnectionState::Error
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
