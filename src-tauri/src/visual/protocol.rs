use serde::{Deserialize, Serialize};

use crate::visual::compiler::{CompiledVisualScene, VisualParameterUpdate};

pub const VISUAL_PROTOCOL_VERSION: u32 = 1;
pub const DEFAULT_READY_TIMEOUT_MS: u64 = 3_000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppToVisualMessage {
    pub protocol_version: u32,
    pub sequence_id: u64,
    pub payload: AppToVisualPayload,
}

impl AppToVisualMessage {
    pub fn new(sequence_id: u64, payload: AppToVisualPayload) -> Self {
        Self {
            protocol_version: VISUAL_PROTOCOL_VERSION,
            sequence_id,
            payload,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum AppToVisualPayload {
    Handshake(VisualHandshake),
    LoadScene(VisualSceneLoad),
    UpdateParameters(VisualParameterUpdateBatch),
    Ping(VisualPing),
    Shutdown(VisualShutdown),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualToAppMessage {
    pub protocol_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence_id: Option<u64>,
    pub payload: VisualToAppPayload,
}

impl VisualToAppMessage {
    pub fn response(sequence_id: u64, payload: VisualToAppPayload) -> Self {
        Self {
            protocol_version: VISUAL_PROTOCOL_VERSION,
            sequence_id: Some(sequence_id),
            payload,
        }
    }

    pub fn event(payload: VisualToAppPayload) -> Self {
        Self {
            protocol_version: VISUAL_PROTOCOL_VERSION,
            sequence_id: None,
            payload,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum VisualToAppPayload {
    Ready(VisualReady),
    SceneLoaded(VisualSceneLoaded),
    ParameterBatchApplied(VisualParameterBatchApplied),
    RuntimeEvent(VisualRuntimeEvent),
    Telemetry(VisualTelemetry),
    Error(VisualProtocolError),
    Pong(VisualPong),
    ShutdownComplete(VisualShutdownComplete),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualHandshake {
    pub app_name: String,
    pub app_version: String,
    pub session_id: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualReady {
    pub renderer: String,
    pub sidecar_version: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualSceneLoad {
    pub scene: VisualSceneSnapshot,
}

impl From<&CompiledVisualScene> for VisualSceneLoad {
    fn from(scene: &CompiledVisualScene) -> Self {
        Self {
            scene: VisualSceneSnapshot::from(scene),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualSceneSnapshot {
    pub scene_id: String,
    pub background_color: [f32; 4],
    pub elements: Vec<VisualSceneElement>,
}

impl From<&CompiledVisualScene> for VisualSceneSnapshot {
    fn from(scene: &CompiledVisualScene) -> Self {
        Self {
            scene_id: scene.scene_id.clone(),
            background_color: scene.background_color,
            elements: scene
                .elements
                .iter()
                .map(VisualSceneElement::from)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualSceneElement {
    pub element_id: String,
    pub element_type: String,
    pub position: [f32; 2],
    pub scale: f32,
    pub parameters: Vec<VisualSceneParameter>,
}

impl From<&crate::visual::compiler::CompiledVisualElement> for VisualSceneElement {
    fn from(element: &crate::visual::compiler::CompiledVisualElement) -> Self {
        Self {
            element_id: element.element_id.clone(),
            element_type: element.element_type.clone(),
            position: element.position,
            scale: element.scale,
            parameters: element
                .parameters
                .iter()
                .map(|(parameter_id, value)| VisualSceneParameter {
                    parameter_id: parameter_id.clone(),
                    value: *value,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualSceneParameter {
    pub parameter_id: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualSceneLoaded {
    pub scene_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualParameterUpdateBatch {
    pub updates: Vec<VisualParameterPatch>,
}

impl From<&[VisualParameterUpdate]> for VisualParameterUpdateBatch {
    fn from(updates: &[VisualParameterUpdate]) -> Self {
        Self {
            updates: updates.iter().map(VisualParameterPatch::from).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualParameterPatch {
    pub element_id: String,
    pub parameter_id: String,
    pub value: f64,
}

impl From<&VisualParameterUpdate> for VisualParameterPatch {
    fn from(update: &VisualParameterUpdate) -> Self {
        Self {
            element_id: update.element_id.clone(),
            parameter_id: update.parameter_id.clone(),
            value: update.value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualParameterBatchApplied {
    pub applied_count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualRuntimeEvent {
    pub level: VisualRuntimeEventLevel,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisualRuntimeEventLevel {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualTelemetry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_id: Option<String>,
    pub fps: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_time_ms: Option<f32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualProtocolError {
    pub code: VisualProtocolErrorCode,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recoverable: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisualProtocolErrorCode {
    ProtocolMismatch,
    InvalidMessage,
    SceneRejected,
    ParameterRejected,
    RendererUnavailable,
    InternalError,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualPing {
    pub sent_at_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualPong {
    pub sent_at_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualShutdown {
    pub mode: VisualShutdownMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisualShutdownMode {
    Graceful,
    Panic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualShutdownComplete {
    pub mode: VisualShutdownMode,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visual::compiler::{CompiledVisualElement, CompiledVisualScene};

    #[test]
    fn app_to_visual_handshake_round_trips_as_json() {
        let message = AppToVisualMessage::new(
            7,
            AppToVisualPayload::Handshake(VisualHandshake {
                app_name: "scrysynth".to_string(),
                app_version: "0.1.0".to_string(),
                session_id: "session-1".to_string(),
                capabilities: vec!["scene_load".to_string(), "parameter_batch".to_string()],
            }),
        );

        let json = serde_json::to_string(&message).expect("handshake serializes");
        assert!(json.contains("\"protocolVersion\":1"));
        assert!(json.contains("\"sequenceId\":7"));
        assert!(json.contains("\"type\":\"handshake\""));

        let restored: AppToVisualMessage =
            serde_json::from_str(&json).expect("handshake deserializes");
        assert_eq!(restored, message);
    }

    #[test]
    fn compiled_scene_maps_to_protocol_scene_load() {
        let compiled = CompiledVisualScene {
            scene_id: "scene-a".to_string(),
            background_color: [0.1, 0.2, 0.3, 1.0],
            elements: vec![CompiledVisualElement {
                element_id: "node-1".to_string(),
                element_type: "sphere".to_string(),
                position: [2.0, 4.0],
                scale: 1.25,
                parameters: vec![("energy".to_string(), 0.75)],
            }],
        };

        let message = AppToVisualMessage::new(
            11,
            AppToVisualPayload::LoadScene(VisualSceneLoad::from(&compiled)),
        );
        let json = serde_json::to_string(&message).expect("scene load serializes");
        let restored: AppToVisualMessage =
            serde_json::from_str(&json).expect("scene load deserializes");

        assert_eq!(restored, message);
        match restored.payload {
            AppToVisualPayload::LoadScene(load) => {
                assert_eq!(load.scene.scene_id, "scene-a");
                assert_eq!(load.scene.elements[0].parameters[0].parameter_id, "energy");
                assert_eq!(load.scene.elements[0].parameters[0].value, 0.75);
            }
            _ => panic!("expected scene load payload"),
        }
    }

    #[test]
    fn parameter_update_batch_uses_transport_owned_sequence_id() {
        let updates = vec![
            VisualParameterUpdate {
                element_id: "element-1".to_string(),
                parameter_id: "opacity".to_string(),
                value: 0.5,
            },
            VisualParameterUpdate {
                element_id: "element-2".to_string(),
                parameter_id: "scale".to_string(),
                value: 2.0,
            },
        ];

        let message = AppToVisualMessage::new(
            12,
            AppToVisualPayload::UpdateParameters(VisualParameterUpdateBatch::from(
                updates.as_slice(),
            )),
        );
        let restored: AppToVisualMessage =
            serde_json::from_str(&serde_json::to_string(&message).unwrap()).unwrap();

        assert_eq!(restored.sequence_id, 12);
        match restored.payload {
            AppToVisualPayload::UpdateParameters(batch) => {
                assert_eq!(batch.updates.len(), 2);
                assert_eq!(batch.updates[0].element_id, "element-1");
            }
            _ => panic!("expected update batch payload"),
        }
    }

    #[test]
    fn visual_to_app_ready_error_pong_and_shutdown_round_trip() {
        let messages = vec![
            VisualToAppMessage::response(
                1,
                VisualToAppPayload::Ready(VisualReady {
                    renderer: "bevy".to_string(),
                    sidecar_version: "0.1.0".to_string(),
                    capabilities: vec!["telemetry".to_string()],
                }),
            ),
            VisualToAppMessage::event(VisualToAppPayload::Error(VisualProtocolError {
                code: VisualProtocolErrorCode::RendererUnavailable,
                message: "no adapter".to_string(),
                recoverable: Some(false),
            })),
            VisualToAppMessage::response(
                2,
                VisualToAppPayload::Pong(VisualPong {
                    sent_at_unix_ms: 42,
                }),
            ),
            VisualToAppMessage::response(
                3,
                VisualToAppPayload::ShutdownComplete(VisualShutdownComplete {
                    mode: VisualShutdownMode::Graceful,
                }),
            ),
        ];

        for message in messages {
            let restored: VisualToAppMessage =
                serde_json::from_str(&serde_json::to_string(&message).unwrap()).unwrap();
            assert_eq!(restored, message);
        }
    }
}
