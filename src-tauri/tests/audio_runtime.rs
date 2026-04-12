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
