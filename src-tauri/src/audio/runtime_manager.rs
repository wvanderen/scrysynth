use thiserror::Error;

use crate::application::session_store::SessionStore;
use crate::audio::compiler::{compile_session_to_topology, TopologyCompileError};
use crate::audio::supercollider::SuperColliderAdapter;
use crate::domain::session::{
    AudioRuntimeHealth, AudioRuntimeLifecycle, RuntimeConnectionState, RuntimeKind, SessionDocument,
};

pub trait AudioRuntimeAdapter {
    fn start(&mut self) -> Result<RuntimeAdapterStatus, String>;
    fn load_topology(
        &mut self,
        topology: &crate::audio::compiler::CompiledTopology,
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
        let topology = compile_session_to_topology(&store.current())?;

        let _ = store.mutate_current(|session| {
            set_runtime_connecting(session);
            Ok::<(), AudioRuntimeManagerError>(())
        })?;

        let boot_status = self
            .adapter
            .start()
            .map_err(AudioRuntimeManagerError::Adapter)?;
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
            RuntimeAdapterStatus::Failed { message } => {
                return Ok(mark_runtime_failed(
                    store,
                    AudioRuntimeHealth::Degraded,
                    message,
                )?);
            }
            _ => {
                return Ok(mark_runtime_failed(
                    store,
                    AudioRuntimeHealth::Degraded,
                    "unexpected runtime boot status".to_string(),
                )?);
            }
        }

        let ready_status = self
            .adapter
            .load_topology(&topology)
            .map_err(AudioRuntimeManagerError::Adapter)?;

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
            RuntimeAdapterStatus::Failed { message } => Ok(mark_runtime_failed(
                store,
                AudioRuntimeHealth::Degraded,
                message,
            )?),
            _ => Ok(mark_runtime_failed(
                store,
                AudioRuntimeHealth::Degraded,
                "unexpected runtime topology load status".to_string(),
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
            RuntimeAdapterStatus::Failed { message } => Some(message),
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
            Ok(RuntimeAdapterStatus::Failed { message }) => Some(message),
            Err(message) => Some(message),
            _ => None,
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
) -> Result<SessionDocument, AudioRuntimeManagerError> {
    store
        .mutate_current(|session| {
            session.audio_runtime.lifecycle = AudioRuntimeLifecycle::Failed;
            session.audio_runtime.health = health.clone();
            session.audio_runtime.active_patch_id = None;
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
