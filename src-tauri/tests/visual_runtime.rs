use scrysynth_lib::application::session_store::SessionStore;
use scrysynth_lib::domain::session::{
    new_id, AgentRuntimeState, GraphEditCommand, MacroDefinition, MacroOverride, MacroTarget,
    NodeType, SessionDocument, VisualRuntimeHealth, VisualRuntimeLifecycle,
};
use scrysynth_lib::visual::adapter::{VisualAdapterStatus, VisualRuntimeAdapter};
use scrysynth_lib::visual::compiler::{
    compile_session_to_visual_scene, visual_updates_for_macro_value, VisualParameterUpdate,
};
use scrysynth_lib::visual::runtime_manager::VisualRuntimeManager;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Default)]
struct TestVisualAdapterState {
    loaded_scene_ids: Vec<String>,
    parameter_updates: Vec<Vec<VisualParameterUpdate>>,
}

struct TestVisualAdapter {
    should_fail_start: bool,
    should_fail_scene: bool,
    should_fail_update: bool,
    started: bool,
    state: Rc<RefCell<TestVisualAdapterState>>,
}

impl TestVisualAdapter {
    fn new() -> Self {
        Self {
            should_fail_start: false,
            should_fail_scene: false,
            should_fail_update: false,
            started: false,
            state: Rc::new(RefCell::new(TestVisualAdapterState::default())),
        }
    }

    fn with_state(state: Rc<RefCell<TestVisualAdapterState>>) -> Self {
        Self {
            should_fail_start: false,
            should_fail_scene: false,
            should_fail_update: false,
            started: false,
            state,
        }
    }

    fn with_start_failure() -> Self {
        Self {
            should_fail_start: true,
            should_fail_scene: false,
            should_fail_update: false,
            started: false,
            state: Rc::new(RefCell::new(TestVisualAdapterState::default())),
        }
    }

    fn with_scene_failure() -> Self {
        Self {
            should_fail_start: false,
            should_fail_scene: true,
            should_fail_update: false,
            started: false,
            state: Rc::new(RefCell::new(TestVisualAdapterState::default())),
        }
    }

    fn with_update_failure() -> Self {
        Self {
            should_fail_start: false,
            should_fail_scene: false,
            should_fail_update: true,
            started: false,
            state: Rc::new(RefCell::new(TestVisualAdapterState::default())),
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
        self.state
            .borrow_mut()
            .loaded_scene_ids
            .push(scene.scene_id.clone());
        Ok(VisualAdapterStatus::SceneLoaded {
            scene_id: scene.scene_id.clone(),
        })
    }

    fn update_parameters(
        &mut self,
        params: &[scrysynth_lib::visual::compiler::VisualParameterUpdate],
    ) -> Result<(), String> {
        if self.should_fail_update {
            return Err("test adapter update failure".to_string());
        }
        self.state
            .borrow_mut()
            .parameter_updates
            .push(params.to_vec());
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
fn compile_session_to_visual_scene_uses_active_scene_selection() {
    let mut store = SessionStore::new_default();
    let mut session = store.current();
    let source_id = session
        .nodes
        .iter()
        .find(|node| matches!(node.node_type, NodeType::Source))
        .unwrap()
        .id
        .clone();
    let output_id = session
        .nodes
        .iter()
        .find(|node| matches!(node.node_type, NodeType::Output))
        .unwrap()
        .id
        .clone();

    session
        .scenes
        .push(scrysynth_lib::domain::session::SceneDefinition {
            id: "visual-output-only".to_string(),
            name: "Output Only".to_string(),
            active_node_ids: vec![output_id.clone()],
            macro_overrides: vec![],
        });
    session.visual_runtime.active_scene_id = Some("visual-output-only".to_string());
    store.replace_current(session.clone());

    let scene = compile_session_to_visual_scene(&store.current());

    assert_eq!(scene.scene_id, "visual-output-only");
    assert_eq!(scene.elements.len(), 1);
    assert_eq!(scene.elements[0].element_id, output_id);
    assert!(!scene
        .elements
        .iter()
        .any(|element| element.element_id == source_id));
}

#[test]
fn macro_values_targeting_visual_parameters_compile_to_updates() {
    let mut session = SessionStore::new_default().current();
    let element_id = session.nodes[0].id.clone();
    session.macros.push(MacroDefinition {
        id: "macro-visual".to_string(),
        name: "visual glow".to_string(),
        target_parameter_ids: vec![],
        range_start: 10.0,
        range_end: 20.0,
        targets: vec![MacroTarget::VisualParameter {
            element_id: element_id.clone(),
            parameter_id: "glow".to_string(),
        }],
    });

    let updates = visual_updates_for_macro_value(&session, "macro-visual", 0.25);

    assert_eq!(
        updates,
        vec![VisualParameterUpdate {
            element_id,
            parameter_id: "glow".to_string(),
            value: 12.5,
        }]
    );
}

#[test]
fn active_scene_macro_overrides_are_projected_into_compiled_scene() {
    let mut session = SessionStore::new_default().current();
    let scene_id = session.scenes[0].id.clone();
    let element_id = session.scenes[0].active_node_ids[0].clone();
    session.macros.push(MacroDefinition {
        id: "macro-brightness".to_string(),
        name: "brightness".to_string(),
        target_parameter_ids: vec![],
        range_start: 0.0,
        range_end: 100.0,
        targets: vec![MacroTarget::VisualParameter {
            element_id: element_id.clone(),
            parameter_id: "brightness".to_string(),
        }],
    });
    session.scenes[0].macro_overrides.push(MacroOverride {
        macro_id: "macro-brightness".to_string(),
        value: 0.7,
    });
    session.visual_runtime.active_scene_id = Some(scene_id);

    let scene = compile_session_to_visual_scene(&session);
    let element = scene
        .elements
        .iter()
        .find(|element| element.element_id == element_id)
        .unwrap();

    assert!(element
        .parameters
        .iter()
        .any(|(parameter_id, value)| parameter_id == "brightness" && *value == 70.0));
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
fn visual_runtime_manager_reloads_scene_after_scene_recall() {
    let state = Rc::new(RefCell::new(TestVisualAdapterState::default()));
    let adapter = TestVisualAdapter::with_state(Rc::clone(&state));
    let mut manager = VisualRuntimeManager::new_for_tests(adapter);
    let mut store = SessionStore::new_default();
    let mut session = store.current();
    let output_id = session
        .nodes
        .iter()
        .find(|node| matches!(node.node_type, NodeType::Output))
        .unwrap()
        .id
        .clone();
    session
        .scenes
        .push(scrysynth_lib::domain::session::SceneDefinition {
            id: "visual-output-only".to_string(),
            name: "Output Only".to_string(),
            active_node_ids: vec![output_id],
            macro_overrides: vec![],
        });
    store.replace_current(session);

    manager.start(&mut store).unwrap();
    let _ = store.mutate_current(|session| {
        session.visual_runtime.active_scene_id = Some("visual-output-only".to_string());
        Ok::<(), ()>(())
    });
    let result = manager.reload_scene(&mut store).unwrap();

    assert_eq!(
        result.visual_runtime.active_scene_id,
        Some("visual-output-only".to_string())
    );
    let loaded_scene_ids = &state.borrow().loaded_scene_ids;
    assert_eq!(loaded_scene_ids.len(), 2);
    assert_eq!(
        loaded_scene_ids.last(),
        Some(&"visual-output-only".to_string())
    );
}

#[test]
fn visual_runtime_manager_sends_graph_parameter_edits_as_updates() {
    let state = Rc::new(RefCell::new(TestVisualAdapterState::default()));
    let adapter = TestVisualAdapter::with_state(Rc::clone(&state));
    let mut manager = VisualRuntimeManager::new_for_tests(adapter);
    let mut store = SessionStore::new_default();
    let mut session = store.current();
    let node_id = session.nodes[0].id.clone();
    let parameter_id = session.nodes[0].parameters[0].id.clone();
    let scene_id = session.scenes[0].id.clone();
    session.scenes[0].active_node_ids.push(node_id.clone());
    session.visual_runtime.active_scene_id = Some(scene_id.clone());
    store.replace_current(session);

    manager.start(&mut store).unwrap();
    let _ = store.mutate_current(|session| {
        session.visual_runtime.active_scene_id = Some(scene_id.clone());
        Ok::<(), ()>(())
    });
    let result = manager
        .reconcile_graph_edit(
            &mut store,
            &GraphEditCommand::SetParameterValue {
                node_id: node_id.clone(),
                parameter_id: parameter_id.clone(),
                value: 0.42,
            },
        )
        .unwrap();

    assert_eq!(result.visual_runtime.health, VisualRuntimeHealth::Healthy);
    assert_eq!(
        state.borrow().parameter_updates,
        vec![vec![VisualParameterUpdate {
            element_id: node_id,
            parameter_id,
            value: 0.42,
        }]]
    );
}

#[test]
fn visual_runtime_manager_skips_graph_parameter_updates_outside_active_scene() {
    let state = Rc::new(RefCell::new(TestVisualAdapterState::default()));
    let adapter = TestVisualAdapter::with_state(Rc::clone(&state));
    let mut manager = VisualRuntimeManager::new_for_tests(adapter);
    let mut store = SessionStore::new_default();
    let mut session = store.current();
    let source_id = session
        .nodes
        .iter()
        .find(|node| matches!(node.node_type, NodeType::Source))
        .unwrap()
        .id
        .clone();
    let source_parameter_id = session
        .nodes
        .iter()
        .find(|node| node.id == source_id)
        .unwrap()
        .parameters[0]
        .id
        .clone();
    let output_id = session
        .nodes
        .iter()
        .find(|node| matches!(node.node_type, NodeType::Output))
        .unwrap()
        .id
        .clone();
    session
        .scenes
        .push(scrysynth_lib::domain::session::SceneDefinition {
            id: "visual-output-only".to_string(),
            name: "Output Only".to_string(),
            active_node_ids: vec![output_id],
            macro_overrides: vec![],
        });
    session.visual_runtime.active_scene_id = Some("visual-output-only".to_string());
    store.replace_current(session);

    manager.start(&mut store).unwrap();
    let result = manager
        .reconcile_graph_edit(
            &mut store,
            &GraphEditCommand::SetParameterValue {
                node_id: source_id,
                parameter_id: source_parameter_id,
                value: 0.42,
            },
        )
        .unwrap();

    assert_eq!(
        result.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Ready
    );
    assert_eq!(result.visual_runtime.health, VisualRuntimeHealth::Healthy);
    assert_eq!(
        result.visual_runtime.active_scene_id,
        Some("visual-output-only".to_string())
    );
    assert!(state.borrow().parameter_updates.is_empty());
}

#[test]
fn visual_runtime_manager_sends_macro_visual_targets_as_updates() {
    let state = Rc::new(RefCell::new(TestVisualAdapterState::default()));
    let adapter = TestVisualAdapter::with_state(Rc::clone(&state));
    let mut manager = VisualRuntimeManager::new_for_tests(adapter);
    let mut store = SessionStore::new_default();
    let mut session = store.current();
    let element_id = session.nodes[0].id.clone();
    let parameter_id = session.nodes[0].parameters[0].id.clone();
    session.scenes[0].active_node_ids.push(element_id.clone());
    session.macros.push(MacroDefinition {
        id: "macro-visual".to_string(),
        name: "visual glow".to_string(),
        target_parameter_ids: vec![],
        range_start: 1.0,
        range_end: 3.0,
        targets: vec![MacroTarget::VisualParameter {
            element_id: element_id.clone(),
            parameter_id: parameter_id.clone(),
        }],
    });
    store.replace_current(session);

    manager.start(&mut store).unwrap();
    manager
        .reconcile_macro_value(&mut store, "macro-visual", 0.5)
        .unwrap();

    assert_eq!(
        state.borrow().parameter_updates,
        vec![vec![VisualParameterUpdate {
            element_id,
            parameter_id,
            value: 2.0,
        }]]
    );
}

#[test]
fn visual_runtime_manager_skips_macro_visual_targets_outside_active_scene() {
    let state = Rc::new(RefCell::new(TestVisualAdapterState::default()));
    let adapter = TestVisualAdapter::with_state(Rc::clone(&state));
    let mut manager = VisualRuntimeManager::new_for_tests(adapter);
    let mut store = SessionStore::new_default();
    let mut session = store.current();
    let source_id = session
        .nodes
        .iter()
        .find(|node| matches!(node.node_type, NodeType::Source))
        .unwrap()
        .id
        .clone();
    let output_id = session
        .nodes
        .iter()
        .find(|node| matches!(node.node_type, NodeType::Output))
        .unwrap()
        .id
        .clone();
    session
        .scenes
        .push(scrysynth_lib::domain::session::SceneDefinition {
            id: "visual-output-only".to_string(),
            name: "Output Only".to_string(),
            active_node_ids: vec![output_id],
            macro_overrides: vec![],
        });
    session.visual_runtime.active_scene_id = Some("visual-output-only".to_string());
    session.macros.push(MacroDefinition {
        id: "macro-hidden-visual".to_string(),
        name: "hidden visual glow".to_string(),
        target_parameter_ids: vec![],
        range_start: 1.0,
        range_end: 3.0,
        targets: vec![MacroTarget::VisualParameter {
            element_id: source_id,
            parameter_id: "glow".to_string(),
        }],
    });
    store.replace_current(session);

    manager.start(&mut store).unwrap();
    let result = manager
        .reconcile_macro_value(&mut store, "macro-hidden-visual", 0.5)
        .unwrap();

    assert_eq!(
        result.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Ready
    );
    assert_eq!(result.visual_runtime.health, VisualRuntimeHealth::Healthy);
    assert_eq!(
        result.visual_runtime.active_scene_id,
        Some("visual-output-only".to_string())
    );
    assert!(state.borrow().parameter_updates.is_empty());
}

#[test]
fn visual_runtime_manager_marks_failed_when_parameter_update_fails() {
    let adapter = TestVisualAdapter::with_update_failure();
    let mut manager = VisualRuntimeManager::new_for_tests(adapter);
    let mut store = SessionStore::new_default();
    let session = store.current();
    let node_id = session.nodes[0].id.clone();
    let parameter_id = session.nodes[0].parameters[0].id.clone();
    let scene_id = session.scenes[0].id.clone();
    store.replace_current({
        let mut session = session;
        session.scenes[0].active_node_ids.push(node_id.clone());
        session
    });

    manager.start(&mut store).unwrap();
    let _ = store.mutate_current(|session| {
        session.visual_runtime.active_scene_id = Some(scene_id.clone());
        Ok::<(), ()>(())
    });
    let result = manager
        .reconcile_graph_edit(
            &mut store,
            &GraphEditCommand::SetParameterValue {
                node_id,
                parameter_id,
                value: 0.42,
            },
        )
        .unwrap();

    assert_eq!(
        result.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Failed
    );
    assert_eq!(result.visual_runtime.active_scene_id, Some(scene_id));
    assert_eq!(result.visual_runtime.health, VisualRuntimeHealth::Degraded);
    assert_eq!(
        result.visual_runtime.last_error,
        Some("test adapter update failure".to_string())
    );
    let visual_status = result
        .runtime_status
        .iter()
        .find(|r| r.runtime == scrysynth_lib::domain::session::RuntimeKind::Visual)
        .unwrap();
    assert_eq!(
        visual_status.status,
        scrysynth_lib::domain::session::RuntimeConnectionState::Error
    );
}

#[test]
fn visual_runtime_manager_stop_resets_to_idle() {
    let state = Rc::new(RefCell::new(TestVisualAdapterState::default()));
    let adapter = TestVisualAdapter::with_state(Rc::clone(&state));
    let mut manager = VisualRuntimeManager::new_for_tests(adapter);
    let mut store = SessionStore::new_default();
    let mut session = store.current();
    let output_id = session
        .nodes
        .iter()
        .find(|node| matches!(node.node_type, NodeType::Output))
        .unwrap()
        .id
        .clone();
    session
        .scenes
        .push(scrysynth_lib::domain::session::SceneDefinition {
            id: "visual-output-only".to_string(),
            name: "Output Only".to_string(),
            active_node_ids: vec![output_id.clone()],
            macro_overrides: vec![],
        });
    store.replace_current(session);

    manager.start(&mut store).unwrap();
    let _ = store.mutate_current(|session| {
        session.visual_runtime.active_scene_id = Some("visual-output-only".to_string());
        Ok::<(), ()>(())
    });
    let result = manager.stop(&mut store);

    assert!(result.is_ok());
    let session = result.unwrap();
    assert_eq!(
        session.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Idle
    );
    assert_eq!(session.visual_runtime.health, VisualRuntimeHealth::Unknown);
    assert_eq!(
        session.visual_runtime.active_scene_id,
        Some("visual-output-only".to_string())
    );
    assert_eq!(session.visual_runtime.renderer, None);
    let compiled = compile_session_to_visual_scene(&session);
    assert_eq!(compiled.scene_id, "visual-output-only");
    assert_eq!(compiled.elements.len(), 1);
    assert_eq!(compiled.elements[0].element_id, output_id);

    let visual_status = session
        .runtime_status
        .iter()
        .find(|r| r.runtime == scrysynth_lib::domain::session::RuntimeKind::Visual);
    assert!(visual_status.is_some());
    assert_eq!(
        visual_status.unwrap().status,
        scrysynth_lib::domain::session::RuntimeConnectionState::Disconnected
    );

    let restarted = manager.start(&mut store).unwrap();
    assert_eq!(
        restarted.visual_runtime.active_scene_id,
        Some("visual-output-only".to_string())
    );
    assert_eq!(
        state.borrow().loaded_scene_ids.last(),
        Some(&"visual-output-only".to_string())
    );
}

#[test]
fn visual_runtime_manager_panic_resets_to_idle() {
    let state = Rc::new(RefCell::new(TestVisualAdapterState::default()));
    let adapter = TestVisualAdapter::with_state(Rc::clone(&state));
    let mut manager = VisualRuntimeManager::new_for_tests(adapter);
    let mut store = SessionStore::new_default();
    let mut session = store.current();
    let output_id = session
        .nodes
        .iter()
        .find(|node| matches!(node.node_type, NodeType::Output))
        .unwrap()
        .id
        .clone();
    session
        .scenes
        .push(scrysynth_lib::domain::session::SceneDefinition {
            id: "visual-output-only".to_string(),
            name: "Output Only".to_string(),
            active_node_ids: vec![output_id.clone()],
            macro_overrides: vec![],
        });
    store.replace_current(session);

    manager.start(&mut store).unwrap();
    let _ = store.mutate_current(|session| {
        session.visual_runtime.active_scene_id = Some("visual-output-only".to_string());
        Ok::<(), ()>(())
    });
    let result = manager.panic(&mut store);

    assert!(result.is_ok());
    let session = result.unwrap();
    assert_eq!(
        session.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Idle
    );
    assert_eq!(
        session.visual_runtime.active_scene_id,
        Some("visual-output-only".to_string())
    );
    assert_eq!(session.visual_runtime.renderer, None);
    let compiled = compile_session_to_visual_scene(&session);
    assert_eq!(compiled.scene_id, "visual-output-only");
    assert_eq!(compiled.elements.len(), 1);
    assert_eq!(compiled.elements[0].element_id, output_id);

    let visual_status = session
        .runtime_status
        .iter()
        .find(|r| r.runtime == scrysynth_lib::domain::session::RuntimeKind::Visual);
    assert!(visual_status.is_some());
    assert_eq!(
        visual_status.unwrap().status,
        scrysynth_lib::domain::session::RuntimeConnectionState::Disconnected
    );

    let restarted = manager.start(&mut store).unwrap();
    assert_eq!(
        restarted.visual_runtime.active_scene_id,
        Some("visual-output-only".to_string())
    );
    assert_eq!(
        state.borrow().loaded_scene_ids.last(),
        Some(&"visual-output-only".to_string())
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
