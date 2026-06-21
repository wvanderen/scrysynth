use scrysynth_lib::application::agent_command;
use scrysynth_lib::application::session_store::SessionStore;
use scrysynth_lib::domain::session::{
    new_id, ActorRef, ControllerKind, GraphEditCommand, Node, NodeType, OwnershipAssignment,
    ParameterValue, PendingActionStatus, PerformanceCommand, Port, PortDirection, RiskTier, Route,
    SceneDefinition, SessionDocument, SignalType, TypedCommand,
};

fn agent_actor() -> ActorRef {
    ActorRef {
        actor_id: "agent".to_string(),
        correlation_id: new_id(),
    }
}

fn user_actor() -> ActorRef {
    ActorRef {
        actor_id: "user".to_string(),
        correlation_id: new_id(),
    }
}

fn test_session() -> SessionDocument {
    SessionDocument {
        nodes: vec![
            Node {
                id: "node-src".to_string(),
                node_type: NodeType::Source,
                ports: vec![Port {
                    id: "port-src-out".to_string(),
                    name: "out".to_string(),
                    direction: PortDirection::Output,
                    signal_type: SignalType::Audio,
                }],
                parameters: vec![ParameterValue {
                    id: "param-freq".to_string(),
                    name: "frequency".to_string(),
                    value: 440.0,
                    default_value: 440.0,
                    min_value: 20.0,
                    max_value: 20000.0,
                    unit: "hz".to_string(),
                }],
                runtime_target: None,
                scene_membership: vec![],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::Shared,
                    is_locked: false,
                },
                enabled: true,
                audio_primitive: None,
            },
            Node {
                id: "node-agent".to_string(),
                node_type: NodeType::Effect,
                ports: vec![],
                parameters: vec![ParameterValue {
                    id: "param-mix".to_string(),
                    name: "mix".to_string(),
                    value: 0.5,
                    default_value: 0.5,
                    min_value: 0.0,
                    max_value: 1.0,
                    unit: "ratio".to_string(),
                }],
                runtime_target: None,
                scene_membership: vec![],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::Agent,
                    is_locked: false,
                },
                enabled: true,
                audio_primitive: None,
            },
        ],
        routes: vec![Route {
            id: "route-1".to_string(),
            source_node_id: "node-src".to_string(),
            source_port_id: "port-src-out".to_string(),
            target_node_id: "node-agent".to_string(),
            target_port_id: "port-agent-in".to_string(),
            bus_id: None,
        }],
        scenes: vec![SceneDefinition {
            id: "scene-1".to_string(),
            name: "intro".to_string(),
            active_node_ids: vec!["node-src".to_string()],
            macro_overrides: vec![],
        }],
        ..SessionDocument::default()
    }
}

#[test]
fn classify_risk_set_parameter_value_is_low() {
    let cmd = TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
        node_id: "node-1".to_string(),
        parameter_id: "param-1".to_string(),
        value: 0.5,
    });
    assert_eq!(agent_command::classify_risk(&cmd), RiskTier::Low);
}

#[test]
fn classify_risk_restore_variation_is_low() {
    let cmd = TypedCommand::Performance(PerformanceCommand::RestoreVariation {
        variation_id: "var-1".to_string(),
    });
    assert_eq!(agent_command::classify_risk(&cmd), RiskTier::Low);
}

#[test]
fn classify_risk_add_node_is_medium() {
    let cmd = TypedCommand::GraphEdit(GraphEditCommand::AddNode {
        node: Node {
            id: "new-node".to_string(),
            node_type: NodeType::Source,
            ports: vec![],
            parameters: vec![],
            runtime_target: None,
            scene_membership: vec![],
            ownership: OwnershipAssignment {
                controller: ControllerKind::Agent,
                is_locked: false,
            },
            enabled: true,
            audio_primitive: None,
        },
    });
    assert_eq!(agent_command::classify_risk(&cmd), RiskTier::Medium);
}

#[test]
fn classify_risk_set_node_enabled_is_medium() {
    let cmd = TypedCommand::GraphEdit(GraphEditCommand::SetNodeEnabled {
        node_id: "node-1".to_string(),
        enabled: false,
    });
    assert_eq!(agent_command::classify_risk(&cmd), RiskTier::Medium);
}

#[test]
fn classify_risk_add_route_is_medium() {
    let cmd = TypedCommand::GraphEdit(GraphEditCommand::AddRoute {
        route: Route {
            id: "r-1".to_string(),
            source_node_id: "a".to_string(),
            source_port_id: "pa".to_string(),
            target_node_id: "b".to_string(),
            target_port_id: "pb".to_string(),
            bus_id: None,
        },
    });
    assert_eq!(agent_command::classify_risk(&cmd), RiskTier::Medium);
}

#[test]
fn classify_risk_recall_scene_is_medium() {
    let cmd = TypedCommand::Performance(PerformanceCommand::RecallScene {
        scene_id: "scene-1".to_string(),
    });
    assert_eq!(agent_command::classify_risk(&cmd), RiskTier::Medium);
}

#[test]
fn classify_risk_remove_node_is_high() {
    let cmd = TypedCommand::GraphEdit(GraphEditCommand::RemoveNode {
        node_id: "node-1".to_string(),
    });
    assert_eq!(agent_command::classify_risk(&cmd), RiskTier::High);
}

#[test]
fn classify_risk_remove_route_is_high() {
    let cmd = TypedCommand::GraphEdit(GraphEditCommand::RemoveRoute {
        route_id: "route-1".to_string(),
    });
    assert_eq!(agent_command::classify_risk(&cmd), RiskTier::High);
}

#[test]
fn classify_risk_clear_bus_assignment_is_high() {
    let cmd = TypedCommand::GraphEdit(GraphEditCommand::ClearNodeBusAssignment {
        node_id: "node-1".to_string(),
    });
    assert_eq!(agent_command::classify_risk(&cmd), RiskTier::High);
}

#[test]
fn high_risk_agent_command_creates_pending_action() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let actor = agent_actor();
    let intent = scrysynth_lib::domain::session::AgentIntent {
        raw_input: "remove node-src".to_string(),
        parsed_commands: vec![TypedCommand::GraphEdit(GraphEditCommand::RemoveNode {
            node_id: "node-src".to_string(),
        })],
        confidence: 0.9,
    };

    let result = agent_command::apply_agent_command(&mut store, actor, intent).unwrap();
    assert!(result.applied.is_empty());
    assert!(result.rejected.is_empty());
    assert_eq!(result.pending.len(), 1);
    assert_eq!(result.pending[0].risk_tier, RiskTier::High);
    assert_eq!(result.pending[0].status, PendingActionStatus::Pending);

    let session = store.current();
    assert_eq!(session.pending_actions.len(), 1);
    assert!(session.nodes.iter().any(|n| n.id == "node-src"));
}

#[test]
fn low_risk_agent_command_auto_applies() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let actor = agent_actor();
    let intent = scrysynth_lib::domain::session::AgentIntent {
        raw_input: "set frequency to 880 on node-src".to_string(),
        parsed_commands: vec![TypedCommand::GraphEdit(
            GraphEditCommand::SetParameterValue {
                node_id: "node-src".to_string(),
                parameter_id: "param-freq".to_string(),
                value: 880.0,
            },
        )],
        confidence: 0.9,
    };

    let result = agent_command::apply_agent_command(&mut store, actor, intent).unwrap();
    assert_eq!(result.applied.len(), 1);
    assert!(result.pending.is_empty());

    let session = store.current();
    let node = session.nodes.iter().find(|n| n.id == "node-src").unwrap();
    assert_eq!(node.parameters[0].value, 880.0);
}

#[test]
fn medium_risk_agent_command_auto_applies() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let actor = agent_actor();
    let intent = scrysynth_lib::domain::session::AgentIntent {
        raw_input: "disable node-agent".to_string(),
        parsed_commands: vec![TypedCommand::GraphEdit(GraphEditCommand::SetNodeEnabled {
            node_id: "node-agent".to_string(),
            enabled: false,
        })],
        confidence: 0.9,
    };

    let result = agent_command::apply_agent_command(&mut store, actor, intent).unwrap();
    assert_eq!(result.applied.len(), 1);
    assert!(result.pending.is_empty());

    let session = store.current();
    let node = session.nodes.iter().find(|n| n.id == "node-agent").unwrap();
    assert!(!node.enabled);
}

#[test]
fn approve_pending_action_applies_command() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let actor = agent_actor();
    let intent = scrysynth_lib::domain::session::AgentIntent {
        raw_input: "remove node-src".to_string(),
        parsed_commands: vec![TypedCommand::GraphEdit(GraphEditCommand::RemoveNode {
            node_id: "node-src".to_string(),
        })],
        confidence: 0.9,
    };

    let result = agent_command::apply_agent_command(&mut store, actor, intent).unwrap();
    let pending_id = result.pending[0].id.clone();

    agent_command::approve_pending_action(&mut store, &pending_id).unwrap();

    let session = store.current();
    assert!(session.pending_actions.is_empty());
    assert!(!session.nodes.iter().any(|n| n.id == "node-src"));
    assert!(session
        .action_history
        .iter()
        .any(|entry| entry.diff.description.contains("Approved pending action")));
}

#[test]
fn reject_pending_action_discards_command() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let actor = agent_actor();
    let intent = scrysynth_lib::domain::session::AgentIntent {
        raw_input: "remove node-src".to_string(),
        parsed_commands: vec![TypedCommand::GraphEdit(GraphEditCommand::RemoveNode {
            node_id: "node-src".to_string(),
        })],
        confidence: 0.9,
    };

    let result = agent_command::apply_agent_command(&mut store, actor, intent).unwrap();
    let pending_id = result.pending[0].id.clone();

    let session = agent_command::reject_pending_action(&mut store, &pending_id).unwrap();

    assert!(!session
        .pending_actions
        .iter()
        .any(|pa| pa.id == pending_id && pa.status == PendingActionStatus::Pending));
    assert!(session.nodes.iter().any(|n| n.id == "node-src"));
    assert!(session
        .action_history
        .iter()
        .any(|entry| entry.diff.description.contains("Rejected pending action")));
}

#[test]
fn approve_nonexistent_pending_action_fails() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let result = agent_command::approve_pending_action(&mut store, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn approve_stale_pending_action_fails_without_resolving_pending() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let actor = agent_actor();
    let intent = scrysynth_lib::domain::session::AgentIntent {
        raw_input: "remove node-src".to_string(),
        parsed_commands: vec![TypedCommand::GraphEdit(GraphEditCommand::RemoveNode {
            node_id: "node-src".to_string(),
        })],
        confidence: 0.9,
    };

    let result = agent_command::apply_agent_command(&mut store, actor, intent).unwrap();
    let pending_id = result.pending[0].id.clone();
    store
        .mutate_current(|session| {
            session.nodes.retain(|node| node.id != "node-src");
            Ok::<(), ()>(())
        })
        .unwrap();

    let err = agent_command::approve_pending_action(&mut store, &pending_id).unwrap_err();
    assert!(err.to_string().contains("node 'node-src' was not found"));

    let session = store.current();
    assert_eq!(session.pending_actions.len(), 1);
    assert_eq!(
        session.pending_actions[0].status,
        PendingActionStatus::Pending
    );
}

#[test]
fn reject_nonexistent_pending_action_fails() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let result = agent_command::reject_pending_action(&mut store, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn action_history_logs_entries() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let actor = user_actor();
    let cmd = TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
        node_id: "node-src".to_string(),
        parameter_id: "param-freq".to_string(),
        value: 880.0,
    });

    store.log_action(&actor, &cmd);

    let session = store.current();
    assert_eq!(session.action_history.len(), 1);
    assert_eq!(session.action_history[0].actor.actor_id, "user");
    assert_eq!(
        session.action_history[0].diff.affected_node_ids,
        vec!["node-src"]
    );
}

#[test]
fn action_history_cap_at_200_entries() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let actor = user_actor();
    let cmd = TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
        node_id: "node-src".to_string(),
        parameter_id: "param-freq".to_string(),
        value: 440.0,
    });

    for _ in 0..250 {
        store.log_action(&actor, &cmd);
    }

    let session = store.current();
    assert_eq!(session.action_history.len(), 200);
}

#[test]
fn user_high_risk_command_auto_applies() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let actor = user_actor();
    let intent = scrysynth_lib::domain::session::AgentIntent {
        raw_input: "remove node-src".to_string(),
        parsed_commands: vec![TypedCommand::GraphEdit(GraphEditCommand::RemoveNode {
            node_id: "node-src".to_string(),
        })],
        confidence: 0.9,
    };

    let result = agent_command::apply_agent_command(&mut store, actor, intent).unwrap();
    assert_eq!(result.applied.len(), 1);
    assert!(result.pending.is_empty());

    let session = store.current();
    assert!(!session.nodes.iter().any(|n| n.id == "node-src"));
}
