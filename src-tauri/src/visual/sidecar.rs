use std::collections::HashMap;

use crate::visual::protocol::{
    AppToVisualMessage, AppToVisualPayload, VisualParameterBatchApplied, VisualProtocolError,
    VisualProtocolErrorCode, VisualReady, VisualRuntimeEvent, VisualRuntimeEventLevel,
    VisualSceneLoaded, VisualSceneSnapshot, VisualShutdownComplete, VisualToAppMessage,
    VisualToAppPayload, VISUAL_PROTOCOL_VERSION,
};

pub const SIDECAR_RENDERER: &str = "scrysynth-minimal-visual";
pub const SIDECAR_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Default)]
pub struct MinimalVisualRuntime {
    scene: Option<VisualSceneSnapshot>,
    parameters: HashMap<(String, String), f64>,
    shutdown_requested: bool,
}

impl MinimalVisualRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_message(&mut self, message: AppToVisualMessage) -> Vec<VisualToAppMessage> {
        if message.protocol_version != VISUAL_PROTOCOL_VERSION {
            return vec![VisualToAppMessage::response(
                message.sequence_id,
                VisualToAppPayload::Error(VisualProtocolError {
                    code: VisualProtocolErrorCode::ProtocolMismatch,
                    message: format!(
                        "unsupported visual protocol version {}; expected {}",
                        message.protocol_version, VISUAL_PROTOCOL_VERSION
                    ),
                    recoverable: Some(false),
                }),
            )];
        }

        match message.payload {
            AppToVisualPayload::Handshake(_) => vec![
                VisualToAppMessage::response(
                    message.sequence_id,
                    VisualToAppPayload::Ready(VisualReady {
                        renderer: SIDECAR_RENDERER.to_string(),
                        sidecar_version: SIDECAR_VERSION.to_string(),
                        capabilities: vec![
                            "scene_load".to_string(),
                            "parameter_batch".to_string(),
                            "shutdown".to_string(),
                        ],
                    }),
                ),
                VisualToAppMessage::event(VisualToAppPayload::RuntimeEvent(VisualRuntimeEvent {
                    level: VisualRuntimeEventLevel::Info,
                    message: "minimal visual runtime ready".to_string(),
                    scene_id: self.scene.as_ref().map(|scene| scene.scene_id.clone()),
                })),
            ],
            AppToVisualPayload::LoadScene(load) => {
                self.parameters.clear();
                for element in &load.scene.elements {
                    for parameter in &element.parameters {
                        self.parameters.insert(
                            (element.element_id.clone(), parameter.parameter_id.clone()),
                            parameter.value,
                        );
                    }
                }

                let scene_id = load.scene.scene_id.clone();
                self.scene = Some(load.scene);

                vec![VisualToAppMessage::response(
                    message.sequence_id,
                    VisualToAppPayload::SceneLoaded(VisualSceneLoaded { scene_id }),
                )]
            }
            AppToVisualPayload::UpdateParameters(batch) => {
                let mut applied_count = 0;

                for update in batch.updates {
                    if self
                        .parameters
                        .contains_key(&(update.element_id.clone(), update.parameter_id.clone()))
                    {
                        self.parameters
                            .insert((update.element_id, update.parameter_id), update.value);
                        applied_count += 1;
                    }
                }

                vec![VisualToAppMessage::response(
                    message.sequence_id,
                    VisualToAppPayload::ParameterBatchApplied(VisualParameterBatchApplied {
                        applied_count,
                    }),
                )]
            }
            AppToVisualPayload::Ping(ping) => vec![VisualToAppMessage::response(
                message.sequence_id,
                VisualToAppPayload::Pong(crate::visual::protocol::VisualPong {
                    sent_at_unix_ms: ping.sent_at_unix_ms,
                }),
            )],
            AppToVisualPayload::Shutdown(shutdown) => {
                self.shutdown_requested = true;
                vec![VisualToAppMessage::response(
                    message.sequence_id,
                    VisualToAppPayload::ShutdownComplete(VisualShutdownComplete {
                        mode: shutdown.mode,
                    }),
                )]
            }
        }
    }

    pub fn should_shutdown(&self) -> bool {
        self.shutdown_requested
    }

    #[cfg(test)]
    pub fn parameter_value(&self, element_id: &str, parameter_id: &str) -> Option<f64> {
        self.parameters
            .get(&(element_id.to_string(), parameter_id.to_string()))
            .copied()
    }
}

pub fn error_response(sequence_id: Option<u64>, message: String) -> VisualToAppMessage {
    match sequence_id {
        Some(sequence_id) => VisualToAppMessage::response(
            sequence_id,
            VisualToAppPayload::Error(VisualProtocolError {
                code: VisualProtocolErrorCode::InvalidMessage,
                message,
                recoverable: Some(true),
            }),
        ),
        None => VisualToAppMessage::event(VisualToAppPayload::Error(VisualProtocolError {
            code: VisualProtocolErrorCode::InvalidMessage,
            message,
            recoverable: Some(true),
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visual::protocol::{
        AppToVisualPayload, VisualHandshake, VisualParameterPatch, VisualParameterUpdateBatch,
        VisualSceneElement, VisualSceneLoad, VisualSceneParameter, VisualShutdown,
        VisualShutdownMode,
    };

    #[test]
    fn handshake_returns_ready_without_gpu() {
        let mut runtime = MinimalVisualRuntime::new();
        let responses = runtime.handle_message(AppToVisualMessage::new(
            1,
            AppToVisualPayload::Handshake(VisualHandshake {
                app_name: "scrysynth".to_string(),
                app_version: "0.1.0".to_string(),
                session_id: "session-1".to_string(),
                capabilities: vec![],
            }),
        ));

        assert!(matches!(
            responses.first().map(|response| &response.payload),
            Some(VisualToAppPayload::Ready(ready)) if ready.renderer == SIDECAR_RENDERER
        ));
    }

    #[test]
    fn load_scene_and_update_parameters_are_stateful() {
        let mut runtime = MinimalVisualRuntime::new();
        let responses = runtime.handle_message(AppToVisualMessage::new(
            2,
            AppToVisualPayload::LoadScene(VisualSceneLoad {
                scene: VisualSceneSnapshot {
                    scene_id: "scene-1".to_string(),
                    background_color: [0.0, 0.0, 0.0, 1.0],
                    elements: vec![VisualSceneElement {
                        element_id: "node-1".to_string(),
                        element_type: "sphere".to_string(),
                        position: [0.0, 0.0],
                        scale: 1.0,
                        parameters: vec![VisualSceneParameter {
                            parameter_id: "gain".to_string(),
                            value: 0.25,
                        }],
                    }],
                },
            }),
        ));

        assert!(matches!(
            responses.first().map(|response| &response.payload),
            Some(VisualToAppPayload::SceneLoaded(loaded)) if loaded.scene_id == "scene-1"
        ));
        assert_eq!(runtime.parameter_value("node-1", "gain"), Some(0.25));

        let responses = runtime.handle_message(AppToVisualMessage::new(
            3,
            AppToVisualPayload::UpdateParameters(VisualParameterUpdateBatch {
                updates: vec![VisualParameterPatch {
                    element_id: "node-1".to_string(),
                    parameter_id: "gain".to_string(),
                    value: 0.75,
                }],
            }),
        ));

        assert_eq!(runtime.parameter_value("node-1", "gain"), Some(0.75));
        assert!(matches!(
            responses.first().map(|response| &response.payload),
            Some(VisualToAppPayload::ParameterBatchApplied(applied)) if applied.applied_count == 1
        ));
    }

    #[test]
    fn shutdown_marks_runtime_done() {
        let mut runtime = MinimalVisualRuntime::new();
        let responses = runtime.handle_message(AppToVisualMessage::new(
            4,
            AppToVisualPayload::Shutdown(VisualShutdown {
                mode: VisualShutdownMode::Panic,
                reason: Some("test".to_string()),
            }),
        ));

        assert!(runtime.should_shutdown());
        assert!(matches!(
            responses.first().map(|response| &response.payload),
            Some(VisualToAppPayload::ShutdownComplete(done))
                if done.mode == VisualShutdownMode::Panic
        ));
    }
}
