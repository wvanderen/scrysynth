use scrysynth_lib::application::graph_edit::{apply_graph_edit, GraphEditError};
use scrysynth_lib::application::session_store::SessionStore;
use scrysynth_lib::domain::session::{
    AudioEffectNode, AudioEffectType, AudioPrimitive, ChannelMode, ControllerKind,
    GraphEditCommand, Node, NodeType, OwnershipAssignment, ParameterValue, Port, PortDirection,
    Route, SignalType,
};

fn effect_node(id: &str) -> Node {
    Node {
        id: id.to_string(),
        node_type: NodeType::Effect,
        ports: vec![
            Port {
                id: format!("{id}-in"),
                name: "signal_in".to_string(),
                direction: PortDirection::Input,
                signal_type: SignalType::Audio,
            },
            Port {
                id: format!("{id}-out"),
                name: "signal_out".to_string(),
                direction: PortDirection::Output,
                signal_type: SignalType::Audio,
            },
        ],
        parameters: vec![ParameterValue {
            id: format!("{id}-mix"),
            name: "mix".to_string(),
            value: 0.5,
            default_value: 0.5,
            min_value: 0.0,
            max_value: 1.0,
            unit: "ratio".to_string(),
        }],
        runtime_target: Some(format!("audio/effect/{id}")),
        scene_membership: vec![],
        ownership: OwnershipAssignment {
            controller: ControllerKind::Shared,
            is_locked: false,
        },
        enabled: true,
        audio_primitive: Some(AudioPrimitive::Effect(AudioEffectNode {
            effect_type: AudioEffectType::Delay,
            bypassed: false,
            bus_target_id: None,
        })),
    }
}

fn source_node_id(session: &scrysynth_lib::domain::session::SessionDocument) -> String {
    session
        .nodes
        .iter()
        .find(|node| node.node_type == NodeType::Source)
        .expect("default source exists")
        .id
        .clone()
}

fn source_output_port_id(session: &scrysynth_lib::domain::session::SessionDocument) -> String {
    session
        .nodes
        .iter()
        .find(|node| node.node_type == NodeType::Source)
        .and_then(|node| {
            node.ports
                .iter()
                .find(|port| port.direction == PortDirection::Output)
        })
        .expect("source output exists")
        .id
        .clone()
}

#[test]
fn audio_graph_commands_add_source_route_returns_updated_session() {
    let mut store = SessionStore::new_default();
    let added_node = effect_node("fx-delay");
    let initial = store.current();

    let after_node = apply_graph_edit(
        &mut store,
        GraphEditCommand::AddNode {
            node: added_node.clone(),
        },
    )
    .expect("node add succeeds");

    assert!(after_node.nodes.iter().any(|node| node.id == added_node.id));

    let updated = apply_graph_edit(
        &mut store,
        GraphEditCommand::AddRoute {
            route: Route {
                id: "route-source-to-fx".to_string(),
                source_node_id: source_node_id(&initial),
                source_port_id: source_output_port_id(&initial),
                target_node_id: added_node.id.clone(),
                target_port_id: "fx-delay-in".to_string(),
                bus_id: None,
            },
        },
    )
    .expect("route add succeeds");

    assert!(updated
        .routes
        .iter()
        .any(|route| route.id == "route-source-to-fx"));
    assert!(updated.nodes.iter().any(|node| node.id == "fx-delay"));
}

#[test]
fn audio_graph_commands_rejects_cycle_and_store_unchanged() {
    let mut store = SessionStore::new_default();
    let original = store.current();
    let added_node = effect_node("fx-feedback");

    apply_graph_edit(
        &mut store,
        GraphEditCommand::AddNode {
            node: added_node.clone(),
        },
    )
    .expect("node add succeeds");

    let result = apply_graph_edit(
        &mut store,
        GraphEditCommand::AddRoute {
            route: Route {
                id: "route-feedback-self".to_string(),
                source_node_id: added_node.id.clone(),
                source_port_id: "fx-feedback-out".to_string(),
                target_node_id: added_node.id.clone(),
                target_port_id: "fx-feedback-in".to_string(),
                bus_id: None,
            },
        },
    );

    assert!(matches!(
        result,
        Err(GraphEditError::UnsupportedCycle { .. })
    ));
    assert_eq!(store.current().routes, original.routes, "store_unchanged");
}

#[test]
fn audio_graph_commands_remove_node_prunes_dependent_routes() {
    let mut store = SessionStore::new_default();
    let added_node = effect_node("fx-remove");

    apply_graph_edit(
        &mut store,
        GraphEditCommand::AddNode {
            node: added_node.clone(),
        },
    )
    .expect("node add succeeds");

    apply_graph_edit(
        &mut store,
        GraphEditCommand::AddRoute {
            route: Route {
                id: "route-source-to-remove".to_string(),
                source_node_id: source_node_id(&store.current()),
                source_port_id: source_output_port_id(&store.current()),
                target_node_id: added_node.id.clone(),
                target_port_id: "fx-remove-in".to_string(),
                bus_id: None,
            },
        },
    )
    .expect("route add succeeds");

    let updated = apply_graph_edit(
        &mut store,
        GraphEditCommand::RemoveNode {
            node_id: added_node.id.clone(),
        },
    )
    .expect("node removal succeeds");

    assert!(!updated.nodes.iter().any(|node| node.id == added_node.id));
    assert!(!updated
        .routes
        .iter()
        .any(|route| route.id == "route-source-to-remove"));
}
