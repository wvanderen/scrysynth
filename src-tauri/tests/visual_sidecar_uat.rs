use std::{path::PathBuf, time::Duration};

use scrysynth_lib::application::session_store::SessionStore;
use scrysynth_lib::domain::session::{
    GraphEditCommand, RuntimeConnectionState, RuntimeKind, VisualRuntimeHealth,
    VisualRuntimeLifecycle,
};
use scrysynth_lib::visual::bevy_sidecar::BevySidecarAdapter;
use scrysynth_lib::visual::runtime_manager::VisualRuntimeManager;
use scrysynth_lib::visual::sidecar::SIDECAR_RENDERER;

#[test]
fn visual_manager_drives_real_minimal_sidecar_lifecycle() {
    let adapter = BevySidecarAdapter::with_executable_override_and_args(
        PathBuf::from(env!("CARGO_BIN_EXE_scrysynth-visual")),
        Duration::from_secs(15),
        vec!["--minimal".to_string()],
    );
    let mut manager = VisualRuntimeManager::new_for_tests(adapter);

    run_visual_sidecar_lifecycle_uat(&mut manager);
}

fn run_visual_sidecar_lifecycle_uat(manager: &mut VisualRuntimeManager<BevySidecarAdapter>) {
    let mut store = SessionStore::new_default();
    let mut session = store.current();
    let scene_id = session.scenes[0].id.clone();
    let visual_node = session
        .nodes
        .iter()
        .find(|node| !node.parameters.is_empty())
        .expect("seed session has a parameterized node");
    let node_id = visual_node.id.clone();
    let parameter_id = visual_node.parameters[0].id.clone();

    if !session.scenes[0].active_node_ids.contains(&node_id) {
        session.scenes[0].active_node_ids.push(node_id.clone());
    }
    session.visual_runtime.active_scene_id = Some(scene_id.clone());
    store.replace_current(session);

    let started = manager.start(&mut store).expect("visual sidecar starts");
    assert_eq!(
        started.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Ready,
        "{:?}",
        started.visual_runtime
    );
    assert_eq!(started.visual_runtime.health, VisualRuntimeHealth::Healthy);
    assert_eq!(
        started.visual_runtime.renderer,
        Some(SIDECAR_RENDERER.to_string())
    );
    assert_eq!(
        started.visual_runtime.active_scene_id,
        Some(scene_id.clone())
    );
    assert_eq!(
        visual_connection_state(&started),
        RuntimeConnectionState::Ready
    );

    let updated = manager
        .reconcile_graph_edit(
            &mut store,
            &GraphEditCommand::SetParameterValue {
                node_id: node_id.clone(),
                parameter_id: parameter_id.clone(),
                value: 0.42,
            },
        )
        .expect("live visual parameter update succeeds");
    assert_eq!(updated.visual_runtime.health, VisualRuntimeHealth::Healthy);
    assert_eq!(updated.visual_runtime.last_error, None);

    let stopped = manager.stop(&mut store).expect("visual sidecar stops");
    assert_eq!(
        stopped.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Idle
    );
    assert_eq!(
        visual_connection_state(&stopped),
        RuntimeConnectionState::Disconnected
    );

    let restarted = manager
        .start(&mut store)
        .expect("visual sidecar restarts after stop");
    assert_eq!(
        restarted.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Ready
    );
    assert_eq!(
        restarted.visual_runtime.active_scene_id,
        Some(scene_id.clone())
    );

    let panicked = manager
        .panic(&mut store)
        .expect("visual sidecar panic shutdown succeeds");
    assert_eq!(
        panicked.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Panicked
    );
    assert_eq!(
        panicked.visual_runtime.health,
        VisualRuntimeHealth::Degraded
    );
    assert_eq!(
        visual_connection_state(&panicked),
        RuntimeConnectionState::Disconnected
    );

    let restarted_after_panic = manager
        .start(&mut store)
        .expect("visual sidecar restarts after panic");
    assert_eq!(
        restarted_after_panic.visual_runtime.lifecycle,
        VisualRuntimeLifecycle::Ready
    );
    assert_eq!(
        restarted_after_panic.visual_runtime.active_scene_id,
        Some(scene_id)
    );
    assert_eq!(
        restarted_after_panic.visual_runtime.renderer,
        Some(SIDECAR_RENDERER.to_string())
    );
}

fn visual_connection_state(
    session: &scrysynth_lib::domain::session::SessionDocument,
) -> RuntimeConnectionState {
    session
        .runtime_status
        .iter()
        .find(|runtime| runtime.runtime == RuntimeKind::Visual)
        .expect("visual runtime status exists")
        .status
        .clone()
}
