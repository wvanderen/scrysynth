use scrysynth_lib::application::session_store::SessionStore;
use scrysynth_lib::domain::session::{
    new_id, AgentRuntimeState, NodeType, SessionDocument, VisualRuntimeHealth,
    VisualRuntimeLifecycle,
};
use scrysynth_lib::visual::adapter::{VisualAdapterStatus, VisualRuntimeAdapter};
use scrysynth_lib::visual::compiler::compile_session_to_visual_scene;
use scrysynth_lib::visual::runtime_manager::VisualRuntimeManager;

use std::path::PathBuf;

struct TestVisualAdapter {
    should_fail_start: bool,
    should_fail_scene: bool,
    started: bool,
}

impl TestVisualAdapter {
    fn new() -> Self {
        Self {
            should_fail_start: false,
            should_fail_scene: false,
            started: false,
        }
    }

    fn with_start_failure() -> Self {
        Self {
            should_fail_start: true,
            should_fail_scene: false,
            started: false,
        }
    }

    fn with_scene_failure() -> Self {
        Self {
            should_fail_start: false,
            should_fail_scene: true,
            started: false,
        }
    }
}

impl VisualRuntimeAdapter for TestVisualAdapter {
    fn start(&mut self) -> Result<VisualAdapterStatus, String> {
        if self.should_fail_start {
            return Ok(VisualAdapterStatus::Failed {
                message: "test adapter start failure".to_string(),
            });
        }
        self.started = true;
        Ok(VisualAdapterStatus::Booted {
            renderer: "test".to_string(),
        })
    }

    fn load_scene(
        &mut self,
        scene: &scrysynth_lib::visual::compiler::CompiledVisualScene,
    ) -> Result<VisualAdapterStatus, String> {
        if self.should_fail_scene {
            return Ok(VisualAdapterStatus::Failed {
                message: "test adapter scene failure".to_string(),
            });
        }
        Ok(VisualAdapterStatus::SceneLoaded {
            scene_id: scene.scene_id.clone(),
        })
    }

    fn update_parameters(
        &mut self,
        _params: &[scrysynth_lib::visual::compiler::VisualParameterUpdate],
    ) -> Result<(), String> {
        Ok(())
    }

    fn stop(&mut self) -> Result<VisualAdapterStatus, String> {
        self.started = false;
        Ok(VisualAdapterStatus::Stopped)
    }

    fn panic(&mut self) -> Result<VisualAdapterStatus, String> {
        self.started = false;
        Ok(VisualAdapterStatus::Panicked)
    }
}

#[test]
fn visual_runtime_state_defaults_correctly() {
    let state = scrysynth_lib::domain::session::VisualRuntimeState::default();

    assert_eq!(state.lifecycle, VisualRuntimeLifecycle::Idle);
    assert_eq!(state.health, VisualRuntimeHealth::Unknown);
    assert_eq!(state.active_scene_id, None);
    assert_eq!(state.fps, None);
    assert_eq!(state.last_error, None);
    assert_eq!(state.renderer, None);
}

#[test]
fn session_document_serialization_round_trip_with_new_runtime_fields() {
    let mut session = SessionDocument::default();
    session.visual_runtime = scrysynth_lib::domain::session::VisualRuntimeState {
        lifecycle: VisualRuntimeLifecycle::Ready,
        health: VisualRuntimeHealth::Healthy,
        active_scene_id: Some("scene-1".to_string()),
        fps: Some(60.0),
        last_error: None,
        renderer: Some("bevy".to_string()),
    };
    session.agent_runtime = AgentRuntimeState {
        is_available: true,
        pending_action_count: 3,
        is_frozen: false,
    };

    let json = serde_json::to_string(&session).expect("session serializes");
    let restored: SessionDocument = serde_json::from_str(&json).expect("session deserializes");

    assert_eq!(
        restored.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Ready
    );
    assert_eq!(restored.visual_runtime.health, VisualRuntimeHealth::Healthy);
    assert_eq!(
        restored.visual_runtime.active_scene_id,
        Some("scene-1".to_string())
    );
    assert_eq!(restored.visual_runtime.fps, Some(60.0));
    assert_eq!(restored.visual_runtime.renderer, Some("bevy".to_string()));
    assert_eq!(restored.agent_runtime.is_available, true);
    assert_eq!(restored.agent_runtime.pending_action_count, 3);
    assert_eq!(restored.agent_runtime.is_frozen, false);
}

#[test]
fn compile_session_to_visual_scene_produces_scene_from_enabled_nodes() {
    let store = SessionStore::new_default();
    let session = store.current();

    let scene = compile_session_to_visual_scene(&session);

    assert!(!scene.scene_id.is_empty());
    assert_eq!(scene.background_color, [0.0, 0.0, 0.0, 1.0]);

    let enabled_count = session.nodes.iter().filter(|n| n.enabled).count();
    assert_eq!(scene.elements.len(), enabled_count);

    for element in &scene.elements {
        let node = session.nodes.iter().find(|n| n.id == element.element_id);
        assert!(node.is_some());
        let node = node.unwrap();
        let expected_type = match node.node_type {
            NodeType::Source => "sphere",
            NodeType::Effect => "box",
            NodeType::Mixer => "ring",
            NodeType::Output => "plane",
        };
        assert_eq!(element.element_type, expected_type);
    }
}

#[test]
fn compile_session_to_visual_scene_handles_empty_sessions() {
    let session = SessionDocument::default();

    let scene = compile_session_to_visual_scene(&session);

    assert_eq!(scene.scene_id, "");
    assert!(scene.elements.is_empty());
    assert_eq!(scene.background_color, [0.0, 0.0, 0.0, 1.0]);
}

#[test]
fn visual_runtime_manager_start_succeeds() {
    let adapter = TestVisualAdapter::new();
    let mut manager = VisualRuntimeManager::new_for_tests(adapter);
    let mut store = SessionStore::new_default();

    let result = manager.start(&mut store);

    assert!(result.is_ok());
    let session = result.unwrap();
    assert_eq!(
        session.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Ready
    );
    assert_eq!(session.visual_runtime.health, VisualRuntimeHealth::Healthy);
    assert!(session.visual_runtime.active_scene_id.is_some());
    assert_eq!(session.visual_runtime.renderer, Some("test".to_string()));

    let visual_status = session
        .runtime_status
        .iter()
        .find(|r| r.runtime == scrysynth_lib::domain::session::RuntimeKind::Visual);
    assert!(visual_status.is_some());
    assert_eq!(
        visual_status.unwrap().status,
        scrysynth_lib::domain::session::RuntimeConnectionState::Ready
    );
}

#[test]
fn visual_runtime_manager_start_with_adapter_failure() {
    let adapter = TestVisualAdapter::with_start_failure();
    let mut manager = VisualRuntimeManager::new_for_tests(adapter);
    let mut store = SessionStore::new_default();

    let result = manager.start(&mut store);

    assert!(result.is_ok());
    let session = result.unwrap();
    assert_eq!(
        session.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Failed
    );
    assert_eq!(session.visual_runtime.health, VisualRuntimeHealth::Degraded);
    assert_eq!(
        session.visual_runtime.last_error,
        Some("test adapter start failure".to_string())
    );
}

#[test]
fn visual_runtime_manager_start_with_scene_failure() {
    let adapter = TestVisualAdapter::with_scene_failure();
    let mut manager = VisualRuntimeManager::new_for_tests(adapter);
    let mut store = SessionStore::new_default();

    let result = manager.start(&mut store);

    assert!(result.is_ok());
    let session = result.unwrap();
    assert_eq!(
        session.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Failed
    );
    assert_eq!(
        session.visual_runtime.last_error,
        Some("test adapter scene failure".to_string())
    );
}

#[test]
fn visual_runtime_manager_stop_resets_to_idle() {
    let adapter = TestVisualAdapter::new();
    let mut manager = VisualRuntimeManager::new_for_tests(adapter);
    let mut store = SessionStore::new_default();

    manager.start(&mut store).unwrap();
    let result = manager.stop(&mut store);

    assert!(result.is_ok());
    let session = result.unwrap();
    assert_eq!(
        session.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Idle
    );
    assert_eq!(session.visual_runtime.health, VisualRuntimeHealth::Unknown);
    assert_eq!(session.visual_runtime.active_scene_id, None);
    assert_eq!(session.visual_runtime.renderer, None);

    let visual_status = session
        .runtime_status
        .iter()
        .find(|r| r.runtime == scrysynth_lib::domain::session::RuntimeKind::Visual);
    assert!(visual_status.is_some());
    assert_eq!(
        visual_status.unwrap().status,
        scrysynth_lib::domain::session::RuntimeConnectionState::Disconnected
    );
}

#[test]
fn visual_runtime_manager_panic_resets_to_idle() {
    let adapter = TestVisualAdapter::new();
    let mut manager = VisualRuntimeManager::new_for_tests(adapter);
    let mut store = SessionStore::new_default();

    manager.start(&mut store).unwrap();
    let result = manager.panic(&mut store);

    assert!(result.is_ok());
    let session = result.unwrap();
    assert_eq!(
        session.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Idle
    );
    assert_eq!(session.visual_runtime.active_scene_id, None);
    assert_eq!(session.visual_runtime.renderer, None);

    let visual_status = session
        .runtime_status
        .iter()
        .find(|r| r.runtime == scrysynth_lib::domain::session::RuntimeKind::Visual);
    assert!(visual_status.is_some());
    assert_eq!(
        visual_status.unwrap().status,
        scrysynth_lib::domain::session::RuntimeConnectionState::Disconnected
    );
}

#[test]
fn agent_runtime_state_derivation() {
    let mut store = SessionStore::new_default();

    let state = store.derive_agent_runtime_state();
    assert!(state.is_available);
    assert_eq!(state.pending_action_count, 0);
    assert!(!state.is_frozen);

    let _ = store.mutate_current(|session| {
        session.agent_frozen = true;
        session
            .pending_actions
            .push(scrysynth_lib::domain::session::PendingAction {
                id: new_id(),
                correlation_id: new_id(),
                command: scrysynth_lib::domain::session::TypedCommand::GraphEdit(
                    scrysynth_lib::domain::session::GraphEditCommand::RemoveNode {
                        node_id: "test".to_string(),
                    },
                ),
                risk_tier: scrysynth_lib::domain::session::RiskTier::Low,
                created_at: "2026-04-12T00:00:00Z".to_string(),
                status: scrysynth_lib::domain::session::PendingActionStatus::Pending,
            });
        Ok::<(), ()>(())
    });

    let state = store.derive_agent_runtime_state();
    assert!(state.is_available);
    assert_eq!(state.pending_action_count, 1);
    assert!(state.is_frozen);
}

#[test]
fn typescript_contract_generation_includes_visual_and_agent_types() {
    let file_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/generated/session-types.ts");
    let generated = std::fs::read_to_string(&file_path).expect("generated types are readable");

    assert!(generated.contains("export type VisualRuntimeState"));
    assert!(generated.contains("export type VisualRuntimeLifecycle"));
    assert!(generated.contains("export type VisualRuntimeHealth"));
    assert!(generated.contains("export type AgentRuntimeState"));
}
