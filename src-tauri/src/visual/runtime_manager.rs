use std::collections::HashSet;

use thiserror::Error;

use crate::application::session_store::SessionStore;
use crate::domain::session::{
    GraphEditCommand, RuntimeConnectionState, RuntimeKind, VisualRuntimeHealth,
    VisualRuntimeLifecycle,
};
use crate::visual::bevy_sidecar::BevySidecarAdapter;
use crate::visual::compiler::{
    compile_session_to_visual_scene, visual_updates_for_macro_value, VisualParameterUpdate,
};

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
                session.visual_runtime.fps = None;
                session.visual_runtime.renderer = None;
                session.visual_runtime.last_error = error.clone();
                set_visual_runtime_status(session, RuntimeConnectionState::Disconnected, error);
                Ok::<(), VisualRuntimeManagerError>(())
            })
            .map_err(Into::into)
    }

    pub fn reload_scene(
        &mut self,
        store: &mut SessionStore,
    ) -> Result<crate::domain::session::SessionDocument, VisualRuntimeManagerError> {
        if !visual_runtime_is_ready(&store.current()) {
            return Ok(store.current());
        }

        let scene = compile_session_to_visual_scene(&store.current());
        let status = match self.adapter.load_scene(&scene) {
            Ok(status) => status,
            Err(message) => {
                return Ok(mark_visual_failed(
                    store,
                    VisualRuntimeHealth::Degraded,
                    message,
                )?);
            }
        };

        match status {
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
                "unexpected visual scene reload status".to_string(),
            )?),
        }
    }

    pub fn reconcile_graph_edit(
        &mut self,
        store: &mut SessionStore,
        command: &GraphEditCommand,
    ) -> Result<crate::domain::session::SessionDocument, VisualRuntimeManagerError> {
        if !visual_runtime_is_ready(&store.current()) {
            return Ok(store.current());
        }

        match command {
            GraphEditCommand::SetParameterValue {
                node_id,
                parameter_id,
                value,
            } => self.update_parameters(
                store,
                vec![VisualParameterUpdate {
                    element_id: node_id.clone(),
                    parameter_id: parameter_id.clone(),
                    value: *value,
                }],
            ),
            GraphEditCommand::AddNode { .. }
            | GraphEditCommand::RemoveNode { .. }
            | GraphEditCommand::SetNodeEnabled { .. }
            | GraphEditCommand::AddRoute { .. }
            | GraphEditCommand::RemoveRoute { .. }
            | GraphEditCommand::AssignNodeToBus { .. }
            | GraphEditCommand::ClearNodeBusAssignment { .. } => self.reload_scene(store),
        }
    }

    pub fn reconcile_macro_value(
        &mut self,
        store: &mut SessionStore,
        macro_id: &str,
        value: f64,
    ) -> Result<crate::domain::session::SessionDocument, VisualRuntimeManagerError> {
        if !visual_runtime_is_ready(&store.current()) {
            return Ok(store.current());
        }

        let updates = visual_updates_for_macro_value(&store.current(), macro_id, value);
        self.update_parameters(store, updates)
    }

    fn update_parameters(
        &mut self,
        store: &mut SessionStore,
        updates: Vec<VisualParameterUpdate>,
    ) -> Result<crate::domain::session::SessionDocument, VisualRuntimeManagerError> {
        let updates = updates_for_active_visual_scene(&store.current(), updates);
        if updates.is_empty() {
            return Ok(store.current());
        }

        match self.adapter.update_parameters(&updates) {
            Ok(()) => store
                .mutate_current(|session| {
                    session.visual_runtime.health = VisualRuntimeHealth::Healthy;
                    session.visual_runtime.last_error = None;
                    set_visual_runtime_status(session, RuntimeConnectionState::Ready, None);
                    Ok::<(), VisualRuntimeManagerError>(())
                })
                .map_err(Into::into),
            Err(message) => Ok(mark_visual_failed(
                store,
                VisualRuntimeHealth::Degraded,
                message,
            )?),
        }
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

fn visual_runtime_is_ready(session: &crate::domain::session::SessionDocument) -> bool {
    matches!(
        session.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Ready | VisualRuntimeLifecycle::Rendering
    )
}

fn updates_for_active_visual_scene(
    session: &crate::domain::session::SessionDocument,
    updates: Vec<VisualParameterUpdate>,
) -> Vec<VisualParameterUpdate> {
    let scene = compile_session_to_visual_scene(session);
    let active_element_ids = scene
        .elements
        .iter()
        .map(|element| element.element_id.as_str())
        .collect::<HashSet<_>>();

    updates
        .into_iter()
        .filter(|update| active_element_ids.contains(update.element_id.as_str()))
        .collect()
}
