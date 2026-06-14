use thiserror::Error;

use crate::application::session_store::SessionStore;
use crate::domain::session::{
    RuntimeConnectionState, RuntimeKind, VisualRuntimeHealth, VisualRuntimeLifecycle,
};
use crate::visual::bevy_sidecar::BevySidecarAdapter;
use crate::visual::compiler::compile_session_to_visual_scene;

use super::adapter::VisualAdapterStatus;
use super::adapter::VisualRuntimeAdapter;

#[derive(Debug)]
pub struct VisualRuntimeManager<A = BevySidecarAdapter> {
    adapter: A,
}

#[derive(Debug, Error)]
pub enum VisualRuntimeManagerError {
    #[error("visual runtime adapter error: {0}")]
    Adapter(String),
}

impl Default for VisualRuntimeManager<BevySidecarAdapter> {
    fn default() -> Self {
        Self {
            adapter: BevySidecarAdapter::default(),
        }
    }
}

impl<A> VisualRuntimeManager<A>
where
    A: VisualRuntimeAdapter,
{
    pub fn new_for_tests(adapter: A) -> Self {
        Self { adapter }
    }

    pub fn start(
        &mut self,
        store: &mut SessionStore,
    ) -> Result<crate::domain::session::SessionDocument, VisualRuntimeManagerError> {
        let scene = compile_session_to_visual_scene(&store.current());

        let _ = store.mutate_current(|session| {
            set_visual_connecting(session);
            Ok::<(), VisualRuntimeManagerError>(())
        });

        let boot_status = match self.adapter.start() {
            Ok(status) => status,
            Err(message) => {
                return Ok(mark_visual_failed(
                    store,
                    VisualRuntimeHealth::Degraded,
                    message,
                )?);
            }
        };

        match boot_status {
            VisualAdapterStatus::Booted { renderer } => {
                let _ = store.mutate_current(|session| {
                    session.visual_runtime.renderer = Some(renderer);
                    Ok::<(), VisualRuntimeManagerError>(())
                });
            }
            VisualAdapterStatus::Failed { message } => {
                return Ok(mark_visual_failed(
                    store,
                    VisualRuntimeHealth::Degraded,
                    message,
                )?);
            }
            _ => {
                return Ok(mark_visual_failed(
                    store,
                    VisualRuntimeHealth::Degraded,
                    "unexpected visual runtime boot status".to_string(),
                )?);
            }
        }

        let ready_status = match self.adapter.load_scene(&scene) {
            Ok(status) => status,
            Err(message) => {
                return Ok(mark_visual_failed(
                    store,
                    VisualRuntimeHealth::Degraded,
                    message,
                )?);
            }
        };

        match ready_status {
            VisualAdapterStatus::SceneLoaded { scene_id } => store
                .mutate_current(|session| {
                    session.visual_runtime.lifecycle = VisualRuntimeLifecycle::Ready;
                    session.visual_runtime.health = VisualRuntimeHealth::Healthy;
                    session.visual_runtime.active_scene_id = Some(scene_id);
                    session.visual_runtime.last_error = None;
                    set_visual_runtime_status(session, RuntimeConnectionState::Ready, None);
                    Ok::<(), VisualRuntimeManagerError>(())
                })
                .map_err(Into::into),
            VisualAdapterStatus::Failed { message } => Ok(mark_visual_failed(
                store,
                VisualRuntimeHealth::Degraded,
                message,
            )?),
            _ => Ok(mark_visual_failed(
                store,
                VisualRuntimeHealth::Degraded,
                "unexpected visual scene load status".to_string(),
            )?),
        }
    }

    pub fn stop(
        &mut self,
        store: &mut SessionStore,
    ) -> Result<crate::domain::session::SessionDocument, VisualRuntimeManagerError> {
        let status = self
            .adapter
            .stop()
            .map_err(VisualRuntimeManagerError::Adapter)?;
        let error = match status {
            VisualAdapterStatus::Failed { message } => Some(message),
            _ => None,
        };

        store
            .mutate_current(|session| {
                session.visual_runtime.lifecycle = VisualRuntimeLifecycle::Idle;
                session.visual_runtime.health = VisualRuntimeHealth::Unknown;
                session.visual_runtime.active_scene_id = None;
                session.visual_runtime.fps = None;
                session.visual_runtime.renderer = None;
                session.visual_runtime.last_error = error.clone();
                set_visual_runtime_status(
                    session,
                    RuntimeConnectionState::Disconnected,
                    error.clone(),
                );
                Ok::<(), VisualRuntimeManagerError>(())
            })
            .map_err(Into::into)
    }

    pub fn panic(
        &mut self,
        store: &mut SessionStore,
    ) -> Result<crate::domain::session::SessionDocument, VisualRuntimeManagerError> {
        let panic_result = self.adapter.panic();
        let error = match panic_result {
            Ok(VisualAdapterStatus::Failed { message }) => Some(message),
            Err(message) => Some(message),
            _ => None,
        };

        store
            .mutate_current(|session| {
                session.visual_runtime.lifecycle = VisualRuntimeLifecycle::Idle;
                session.visual_runtime.health = VisualRuntimeHealth::Unknown;
                session.visual_runtime.active_scene_id = None;
                session.visual_runtime.fps = None;
                session.visual_runtime.renderer = None;
                session.visual_runtime.last_error = error.clone();
                set_visual_runtime_status(session, RuntimeConnectionState::Disconnected, error);
                Ok::<(), VisualRuntimeManagerError>(())
            })
            .map_err(Into::into)
    }
}

fn set_visual_connecting(session: &mut crate::domain::session::SessionDocument) {
    session.visual_runtime.lifecycle = VisualRuntimeLifecycle::Starting;
    session.visual_runtime.health = VisualRuntimeHealth::Unknown;
    session.visual_runtime.last_error = None;
    set_visual_runtime_status(session, RuntimeConnectionState::Connecting, None);
}

fn mark_visual_failed(
    store: &mut SessionStore,
    health: VisualRuntimeHealth,
    message: String,
) -> Result<crate::domain::session::SessionDocument, VisualRuntimeManagerError> {
    store
        .mutate_current(|session| {
            session.visual_runtime.lifecycle = VisualRuntimeLifecycle::Failed;
            session.visual_runtime.health = health.clone();
            session.visual_runtime.active_scene_id = None;
            session.visual_runtime.last_error = Some(message.clone());
            set_visual_runtime_status(
                session,
                RuntimeConnectionState::Error,
                Some(message.clone()),
            );
            Ok::<(), VisualRuntimeManagerError>(())
        })
        .map_err(Into::into)
}

fn set_visual_runtime_status(
    session: &mut crate::domain::session::SessionDocument,
    status: RuntimeConnectionState,
    error: Option<String>,
) {
    if let Some(runtime) = session
        .runtime_status
        .iter_mut()
        .find(|runtime| runtime.runtime == RuntimeKind::Visual)
    {
        runtime.status = status;
        runtime.last_error = error;
    }
}
