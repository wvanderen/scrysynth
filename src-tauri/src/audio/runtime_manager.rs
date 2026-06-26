use thiserror::Error;

use crate::application::session_store::SessionStore;
use crate::audio::compiler::{compile_session_to_topology, TopologyCompileError};
use crate::audio::supercollider::SuperColliderAdapter;
use crate::domain::session::{
    AudioRuntimeHealth, AudioRuntimeLifecycle, GraphEditCommand, MacroTarget,
    RuntimeConnectionState, RuntimeKind, SessionDocument,
};

pub trait AudioRuntimeAdapter {
    fn start(&mut self) -> Result<RuntimeAdapterStatus, String>;
    fn load_topology(
        &mut self,
        topology: &crate::audio::compiler::CompiledTopology,
    ) -> Result<RuntimeAdapterStatus, String>;
    fn set_parameter_value(
        &mut self,
        node_id: &str,
        parameter_id: &str,
        value: f64,
    ) -> Result<RuntimeAdapterStatus, String>;
    fn stop(&mut self) -> Result<RuntimeAdapterStatus, String>;
    fn panic(&mut self) -> Result<RuntimeAdapterStatus, String>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeAdapterStatus {
    Booted {
        sample_rate_hz: u32,
        block_size: u32,
    },
    Ready {
        active_patch_id: String,
    },
    Stopped,
    Panicked,
    Failed {
        message: String,
        active_patch_id: Option<String>,
    },
}

#[derive(Debug)]
pub struct AudioRuntimeManager<A = SuperColliderAdapter> {
    adapter: A,
}

#[derive(Debug, Error)]
pub enum AudioRuntimeManagerError {
    #[error("failed to compile audio topology: {0}")]
    Compile(#[from] TopologyCompileError),
    #[error("audio runtime adapter error: {0}")]
    Adapter(String),
}

impl Default for AudioRuntimeManager<SuperColliderAdapter> {
    fn default() -> Self {
        Self {
            adapter: SuperColliderAdapter::default(),
        }
    }
}

impl<A> AudioRuntimeManager<A>
where
    A: AudioRuntimeAdapter,
{
    pub fn new_for_tests(adapter: A) -> Self {
        Self { adapter }
    }

    pub fn start(
        &mut self,
        store: &mut SessionStore,
    ) -> Result<SessionDocument, AudioRuntimeManagerError> {
        let _ = store.mutate_current(|session| {
            set_runtime_connecting(session);
            Ok::<(), AudioRuntimeManagerError>(())
        })?;

        let topology = match compile_session_to_topology(&store.current()) {
            Ok(topology) => topology,
            Err(error) => {
                return Ok(mark_runtime_failed(
                    store,
                    AudioRuntimeHealth::Degraded,
                    format!("failed to compile audio topology: {error}"),
                    store.current().audio_runtime.active_patch_id.clone(),
                )?);
            }
        };

        let boot_status =
            self.adapter
                .start()
                .unwrap_or_else(|message| RuntimeAdapterStatus::Failed {
                    message: format!(
                        "Audio runtime app state error while starting adapter: {message}"
                    ),
                    active_patch_id: store.current().audio_runtime.active_patch_id.clone(),
                });
        match boot_status {
            RuntimeAdapterStatus::Booted {
                sample_rate_hz,
                block_size,
            } => {
                let _ = store.mutate_current(|session| {
                    session.audio_runtime.sample_rate_hz = Some(sample_rate_hz);
                    session.audio_runtime.block_size = Some(block_size);
                    Ok::<(), AudioRuntimeManagerError>(())
                })?;
            }
            RuntimeAdapterStatus::Failed {
                message,
                active_patch_id,
            } => {
                return Ok(mark_runtime_failed(
                    store,
                    AudioRuntimeHealth::Degraded,
                    message,
                    active_patch_id,
                )?);
            }
            _ => {
                return Ok(mark_runtime_failed(
                    store,
                    AudioRuntimeHealth::Degraded,
                    "unexpected runtime boot status".to_string(),
                    None,
                )?);
            }
        }

        let ready_status = self
            .adapter
            .load_topology(&topology)
            .unwrap_or_else(|message| RuntimeAdapterStatus::Failed {
                message: format!(
                    "Audio runtime app state error while applying topology: {message}"
                ),
                active_patch_id: store.current().audio_runtime.active_patch_id.clone(),
            });

        match ready_status {
            RuntimeAdapterStatus::Ready { active_patch_id } => store
                .mutate_current(|session| {
                    session.audio_runtime.lifecycle = AudioRuntimeLifecycle::Ready;
                    session.audio_runtime.health = AudioRuntimeHealth::Healthy;
                    session.audio_runtime.active_patch_id = Some(active_patch_id);
                    session.audio_runtime.last_error = None;
                    set_runtime_status(session, RuntimeConnectionState::Ready, None);
                    Ok::<(), AudioRuntimeManagerError>(())
                })
                .map_err(Into::into),
            RuntimeAdapterStatus::Failed {
                message,
                active_patch_id,
            } => Ok(mark_runtime_failed(
                store,
                AudioRuntimeHealth::Degraded,
                message,
                active_patch_id,
            )?),
            _ => Ok(mark_runtime_failed(
                store,
                AudioRuntimeHealth::Degraded,
                "unexpected runtime topology load status".to_string(),
                store.current().audio_runtime.active_patch_id.clone(),
            )?),
        }
    }

    pub fn reconcile_graph_edit(
        &mut self,
        store: &mut SessionStore,
        command: &GraphEditCommand,
    ) -> Result<SessionDocument, AudioRuntimeManagerError> {
        if store.current().audio_runtime.lifecycle != AudioRuntimeLifecycle::Ready {
            return Ok(store.current());
        }

        match command {
            GraphEditCommand::SetParameterValue {
                node_id,
                parameter_id,
                value,
            } => self.set_live_parameter(store, node_id, parameter_id, *value),
            GraphEditCommand::AddNode { .. }
            | GraphEditCommand::RemoveNode { .. }
            | GraphEditCommand::SetNodeEnabled { .. }
            | GraphEditCommand::AddRoute { .. }
            | GraphEditCommand::RemoveRoute { .. }
            | GraphEditCommand::AssignNodeToBus { .. }
            | GraphEditCommand::ClearNodeBusAssignment { .. } => self.reapply_live_topology(store),
            // Sequencer step edits change the 16-step pattern the app-driven
            // transport tick plays; they do not alter the audio topology, so no
            // SC reapply is needed (the tick loop reads canonical state).
            GraphEditCommand::SetStepValue { .. } => Ok(store.current().clone()),
        }
    }

    pub fn reconcile_macro_value(
        &mut self,
        store: &mut SessionStore,
        macro_id: &str,
        value: f64,
    ) -> Result<SessionDocument, AudioRuntimeManagerError> {
        if store.current().audio_runtime.lifecycle != AudioRuntimeLifecycle::Ready {
            return Ok(store.current());
        }

        let Some(macro_def) = store
            .current()
            .macros
            .iter()
            .find(|macro_def| macro_def.id == macro_id)
            .cloned()
        else {
            return Ok(store.current());
        };

        let clamped_input = value.clamp(0.0, 1.0);
        let scaled_value =
            macro_def.range_start + (clamped_input * (macro_def.range_end - macro_def.range_start));

        for target in &macro_def.targets {
            if let MacroTarget::AudioParameter {
                node_id,
                parameter_id,
            } = target
            {
                self.set_live_parameter(store, node_id, parameter_id, scaled_value)?;
            }
        }

        Ok(store.current())
    }

    pub fn reload_topology_if_ready(
        &mut self,
        store: &mut SessionStore,
    ) -> Result<SessionDocument, AudioRuntimeManagerError> {
        if store.current().audio_runtime.lifecycle != AudioRuntimeLifecycle::Ready {
            return Ok(store.current());
        }

        self.reapply_live_topology(store)
    }

    fn set_live_parameter(
        &mut self,
        store: &mut SessionStore,
        node_id: &str,
        parameter_id: &str,
        value: f64,
    ) -> Result<SessionDocument, AudioRuntimeManagerError> {
        let status = self
            .adapter
            .set_parameter_value(node_id, parameter_id, value)
            .unwrap_or_else(|message| RuntimeAdapterStatus::Failed {
                message,
                active_patch_id: store.current().audio_runtime.active_patch_id.clone(),
            });

        match status {
            RuntimeAdapterStatus::Ready { active_patch_id } => {
                mark_runtime_ready(store, active_patch_id)
            }
            RuntimeAdapterStatus::Failed {
                message,
                active_patch_id,
            } => Ok(mark_runtime_failed(
                store,
                AudioRuntimeHealth::Degraded,
                message,
                active_patch_id,
            )?),
            _ => Ok(mark_runtime_failed(
                store,
                AudioRuntimeHealth::Degraded,
                "unexpected live parameter update status".to_string(),
                store.current().audio_runtime.active_patch_id.clone(),
            )?),
        }
    }

    fn reapply_live_topology(
        &mut self,
        store: &mut SessionStore,
    ) -> Result<SessionDocument, AudioRuntimeManagerError> {
        let topology = match compile_session_to_topology(&store.current()) {
            Ok(topology) => topology,
            Err(error) => {
                return Ok(mark_runtime_failed(
                    store,
                    AudioRuntimeHealth::Degraded,
                    format!("failed to compile live audio topology reapply: {error}"),
                    store.current().audio_runtime.active_patch_id.clone(),
                )?);
            }
        };

        let status = self
            .adapter
            .load_topology(&topology)
            .unwrap_or_else(|message| RuntimeAdapterStatus::Failed {
                message,
                active_patch_id: store.current().audio_runtime.active_patch_id.clone(),
            });

        match status {
            RuntimeAdapterStatus::Ready { active_patch_id } => {
                mark_runtime_ready(store, active_patch_id)
            }
            RuntimeAdapterStatus::Failed {
                message,
                active_patch_id,
            } => Ok(mark_runtime_failed(
                store,
                AudioRuntimeHealth::Degraded,
                message,
                active_patch_id,
            )?),
            _ => Ok(mark_runtime_failed(
                store,
                AudioRuntimeHealth::Degraded,
                "unexpected live topology reapply status".to_string(),
                store.current().audio_runtime.active_patch_id.clone(),
            )?),
        }
    }

    pub fn stop(
        &mut self,
        store: &mut SessionStore,
    ) -> Result<SessionDocument, AudioRuntimeManagerError> {
        let status = self
            .adapter
            .stop()
            .map_err(AudioRuntimeManagerError::Adapter)?;
        let error = match status {
            RuntimeAdapterStatus::Failed { message, .. } => Some(message),
            _ => None,
        };

        store
            .mutate_current(|session| {
                session.audio_runtime.lifecycle = AudioRuntimeLifecycle::Idle;
                session.audio_runtime.health = AudioRuntimeHealth::Unknown;
                session.audio_runtime.active_patch_id = None;
                session.audio_runtime.sample_rate_hz = None;
                session.audio_runtime.block_size = None;
                session.audio_runtime.last_error = error.clone();
                set_runtime_status(session, RuntimeConnectionState::Disconnected, error.clone());
                Ok::<(), AudioRuntimeManagerError>(())
            })
            .map_err(Into::into)
    }

    pub fn panic(
        &mut self,
        store: &mut SessionStore,
    ) -> Result<SessionDocument, AudioRuntimeManagerError> {
        let panic_result = self.adapter.panic();
        let error = match panic_result {
            Ok(RuntimeAdapterStatus::Failed { message, .. }) => Some(format!(
                "Panic recovery completed, but audio runtime reported: {message}"
            )),
            Err(message) => Some(format!(
                "Panic recovery completed after adapter error: {message}"
            )),
            _ => Some(
                "Panic recovery completed: stopped scsynth, cleared the active patch, and returned audio to a restartable idle state."
                    .to_string(),
            ),
        };

        store
            .mutate_current(|session| {
                session.audio_runtime.lifecycle = AudioRuntimeLifecycle::Idle;
                session.audio_runtime.health = AudioRuntimeHealth::PanicRecovered;
                session.audio_runtime.active_patch_id = None;
                session.audio_runtime.sample_rate_hz = None;
                session.audio_runtime.block_size = None;
                session.audio_runtime.last_error = error.clone();
                session.audio_runtime.panic_recovery_count += 1;
                set_runtime_status(session, RuntimeConnectionState::Disconnected, error.clone());
                Ok::<(), AudioRuntimeManagerError>(())
            })
            .map_err(Into::into)
    }
}

fn set_runtime_connecting(session: &mut SessionDocument) {
    session.audio_runtime.lifecycle = AudioRuntimeLifecycle::Booting;
    session.audio_runtime.health = AudioRuntimeHealth::Unknown;
    session.audio_runtime.last_error = None;
    set_runtime_status(session, RuntimeConnectionState::Connecting, None);
}

fn mark_runtime_failed(
    store: &mut SessionStore,
    health: AudioRuntimeHealth,
    message: String,
    active_patch_id: Option<String>,
) -> Result<SessionDocument, AudioRuntimeManagerError> {
    store
        .mutate_current(|session| {
            session.audio_runtime.lifecycle = AudioRuntimeLifecycle::Failed;
            session.audio_runtime.health = health.clone();
            session.audio_runtime.active_patch_id = active_patch_id.clone();
            session.audio_runtime.last_error = Some(message.clone());
            set_runtime_status(
                session,
                RuntimeConnectionState::Error,
                Some(message.clone()),
            );
            Ok::<(), AudioRuntimeManagerError>(())
        })
        .map_err(Into::into)
}

fn mark_runtime_ready(
    store: &mut SessionStore,
    active_patch_id: String,
) -> Result<SessionDocument, AudioRuntimeManagerError> {
    store
        .mutate_current(|session| {
            session.audio_runtime.lifecycle = AudioRuntimeLifecycle::Ready;
            session.audio_runtime.health = AudioRuntimeHealth::Healthy;
            session.audio_runtime.active_patch_id = Some(active_patch_id);
            session.audio_runtime.last_error = None;
            set_runtime_status(session, RuntimeConnectionState::Ready, None);
            Ok::<(), AudioRuntimeManagerError>(())
        })
        .map_err(Into::into)
}

fn set_runtime_status(
    session: &mut SessionDocument,
    status: RuntimeConnectionState,
    error: Option<String>,
) {
    if let Some(runtime) = session
        .runtime_status
        .iter_mut()
        .find(|runtime| runtime.runtime == RuntimeKind::Audio)
    {
        runtime.status = status;
        runtime.last_error = error;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct FakeAdapter {
        parameter_status: RuntimeAdapterStatus,
        topology_status: RuntimeAdapterStatus,
        parameter_updates: Vec<(String, String, f64)>,
        topology_load_count: usize,
    }

    impl FakeAdapter {
        fn ready() -> Self {
            Self {
                parameter_status: RuntimeAdapterStatus::Ready {
                    active_patch_id: "patch-live".to_string(),
                },
                topology_status: RuntimeAdapterStatus::Ready {
                    active_patch_id: "patch-reapplied".to_string(),
                },
                parameter_updates: Vec::new(),
                topology_load_count: 0,
            }
        }
    }

    impl AudioRuntimeAdapter for FakeAdapter {
        fn start(&mut self) -> Result<RuntimeAdapterStatus, String> {
            Ok(RuntimeAdapterStatus::Booted {
                sample_rate_hz: 48_000,
                block_size: 64,
            })
        }

        fn load_topology(
            &mut self,
            _topology: &crate::audio::compiler::CompiledTopology,
        ) -> Result<RuntimeAdapterStatus, String> {
            self.topology_load_count += 1;
            Ok(self.topology_status.clone())
        }

        fn set_parameter_value(
            &mut self,
            node_id: &str,
            parameter_id: &str,
            value: f64,
        ) -> Result<RuntimeAdapterStatus, String> {
            self.parameter_updates
                .push((node_id.to_string(), parameter_id.to_string(), value));
            Ok(self.parameter_status.clone())
        }

        fn stop(&mut self) -> Result<RuntimeAdapterStatus, String> {
            Ok(RuntimeAdapterStatus::Stopped)
        }

        fn panic(&mut self) -> Result<RuntimeAdapterStatus, String> {
            Ok(RuntimeAdapterStatus::Panicked)
        }
    }

    #[test]
    fn live_parameter_update_sends_adapter_control_and_keeps_runtime_ready() {
        let mut store = ready_store();
        let node_id = store.current().nodes[0].id.clone();
        let parameter_id = store.current().nodes[0].parameters[0].id.clone();
        let mut manager = AudioRuntimeManager::new_for_tests(FakeAdapter::ready());

        let session = manager
            .reconcile_graph_edit(
                &mut store,
                &GraphEditCommand::SetParameterValue {
                    node_id: node_id.clone(),
                    parameter_id: parameter_id.clone(),
                    value: 0.42,
                },
            )
            .expect("live update reconciles");

        assert_eq!(
            manager.adapter.parameter_updates,
            vec![(node_id, parameter_id, 0.42)]
        );
        assert_eq!(
            session.audio_runtime.lifecycle,
            AudioRuntimeLifecycle::Ready
        );
        assert_eq!(session.audio_runtime.health, AudioRuntimeHealth::Healthy);
        assert_eq!(
            session.audio_runtime.active_patch_id,
            Some("patch-live".to_string())
        );
        assert_eq!(session.audio_runtime.last_error, None);
    }

    #[test]
    fn live_parameter_failure_marks_runtime_degraded_with_error() {
        let mut store = ready_store();
        let node_id = store.current().nodes[0].id.clone();
        let parameter_id = store.current().nodes[0].parameters[0].id.clone();
        let mut adapter = FakeAdapter::ready();
        adapter.parameter_status = RuntimeAdapterStatus::Failed {
            message: "live parameter update: missing SC control".to_string(),
            active_patch_id: Some("patch-live".to_string()),
        };
        let mut manager = AudioRuntimeManager::new_for_tests(adapter);

        let session = manager
            .reconcile_graph_edit(
                &mut store,
                &GraphEditCommand::SetParameterValue {
                    node_id,
                    parameter_id,
                    value: 0.21,
                },
            )
            .expect("failure is reflected in session");

        assert_eq!(
            session.audio_runtime.lifecycle,
            AudioRuntimeLifecycle::Failed
        );
        assert_eq!(session.audio_runtime.health, AudioRuntimeHealth::Degraded);
        assert_eq!(
            session.audio_runtime.last_error,
            Some("live parameter update: missing SC control".to_string())
        );
    }

    #[test]
    fn live_topology_edit_reapplies_full_topology() {
        let mut store = ready_store();
        let route_id = store.current().routes[0].id.clone();
        let mut manager = AudioRuntimeManager::new_for_tests(FakeAdapter::ready());

        let session = manager
            .reconcile_graph_edit(&mut store, &GraphEditCommand::RemoveRoute { route_id })
            .expect("topology reapply reconciles");

        assert_eq!(manager.adapter.topology_load_count, 1);
        assert_eq!(
            session.audio_runtime.active_patch_id,
            Some("patch-reapplied".to_string())
        );
    }

    fn ready_store() -> SessionStore {
        let mut store = SessionStore::new_default();
        store
            .mutate_current(|session| {
                session.audio_runtime.lifecycle = AudioRuntimeLifecycle::Ready;
                session.audio_runtime.health = AudioRuntimeHealth::Healthy;
                session.audio_runtime.active_patch_id = Some("patch-live".to_string());
                set_runtime_status(session, RuntimeConnectionState::Ready, None);
                Ok::<(), AudioRuntimeManagerError>(())
            })
            .expect("store marks ready");
        store
    }
}
