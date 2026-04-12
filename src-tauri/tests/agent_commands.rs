use scrysynth_lib::application::agent_command;
use scrysynth_lib::application::session_store::{OwnershipGateReason, SessionStore};
use scrysynth_lib::domain::session::{
    new_id, ActorRef, AgentIntent, ControllerKind, GraphEditCommand, Node, NodeType,
    OwnershipAssignment, ParameterValue, PerformanceCommand, Port, PortDirection, SceneDefinition,
    SessionDocument, SignalType, TypedCommand,
};

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
                id: "node-user".to_string(),
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
                    controller: ControllerKind::User,
                    is_locked: false,
                },
                enabled: true,
                audio_primitive: None,
            },
            Node {
                id: "node-agent".to_string(),
                node_type: NodeType::Source,
                ports: vec![],
                parameters: vec![ParameterValue {
                    id: "param-lvl".to_string(),
                    name: "level".to_string(),
                    value: 0.7,
                    default_value: 0.7,
                    min_value: 0.0,
                    max_value: 1.0,
                    unit: "linear".to_string(),
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
            Node {
                id: "node-locked".to_string(),
                node_type: NodeType::Mixer,
                ports: vec![],
                parameters: vec![],
                runtime_target: None,
                scene_membership: vec![],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::Shared,
                    is_locked: true,
                },
                enabled: true,
                audio_primitive: None,
            },
        ],
        scenes: vec![SceneDefinition {
            id: "scene-intro".to_string(),
            name: "intro".to_string(),
            active_node_ids: vec!["node-src".to_string()],
            macro_overrides: vec![],
        }],
        ..SessionDocument::default()
    }
}

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

#[test]
fn intent_parser_add_oscillator_produces_add_node() {
    let session = test_session();
    let intent = agent_command::parse_agent_intent("add oscillator", &session);
    assert_eq!(intent.parsed_commands.len(), 1);
    assert!(intent.confidence > 0.0);
    match &intent.parsed_commands[0] {
        TypedCommand::GraphEdit(GraphEditCommand::AddNode { node }) => {
            assert_eq!(node.node_type, NodeType::Source);
        }
        _ => panic!("expected AddNode"),
    }
}

#[test]
fn intent_parser_add_noise_produces_add_node() {
    let session = test_session();
    let intent = agent_command::parse_agent_intent("add noise", &session);
    assert_eq!(intent.parsed_commands.len(), 1);
}

#[test]
fn intent_parser_remove_by_id() {
    let session = test_session();
    let intent = agent_command::parse_agent_intent("remove node-src", &session);
    assert_eq!(intent.parsed_commands.len(), 1);
    match &intent.parsed_commands[0] {
        TypedCommand::GraphEdit(GraphEditCommand::RemoveNode { node_id }) => {
            assert_eq!(node_id, "node-src");
        }
        _ => panic!("expected RemoveNode"),
    }
}

#[test]
fn intent_parser_set_parameter_by_node_id() {
    let session = test_session();
    let intent = agent_command::parse_agent_intent("set frequency to 880 on node-src", &session);
    assert_eq!(intent.parsed_commands.len(), 1);
    match &intent.parsed_commands[0] {
        TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue { node_id, value, .. }) => {
            assert_eq!(node_id, "node-src");
            assert_eq!(*value, 880.0);
        }
        _ => panic!("expected SetParameterValue"),
    }
}

#[test]
fn intent_parser_recall_scene_by_name() {
    let session = test_session();
    let intent = agent_command::parse_agent_intent("recall scene intro", &session);
    assert_eq!(intent.parsed_commands.len(), 1);
    match &intent.parsed_commands[0] {
        TypedCommand::Performance(PerformanceCommand::RecallScene { scene_id }) => {
            assert_eq!(scene_id, "scene-intro");
        }
        _ => panic!("expected RecallScene"),
    }
}

#[test]
fn intent_parser_unrecognized_returns_empty() {
    let session = test_session();
    let intent = agent_command::parse_agent_intent("what is the meaning of life", &session);
    assert!(intent.parsed_commands.is_empty());
    assert_eq!(intent.confidence, 0.0);
}

#[test]
fn ownership_gate_user_always_passes() {
    let store = SessionStore::new_default();
    let cmd = TypedCommand::GraphEdit(GraphEditCommand::RemoveNode {
        node_id: "anything".to_string(),
    });
    assert!(store.check_ownership(&user_actor(), &cmd).is_ok());
}

#[test]
fn ownership_gate_agent_rejected_on_user_owned_node() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let cmd = TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
        node_id: "node-user".to_string(),
        parameter_id: "param-mix".to_string(),
        value: 0.9,
    });
    let result = store.check_ownership(&agent_actor(), &cmd);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].node_id, "node-user");
    assert_eq!(
        errors[0].reason,
        OwnershipGateReason::AgentBlockedByUserOwnership
    );
}

#[test]
fn ownership_gate_agent_allowed_on_shared_node() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let cmd = TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
        node_id: "node-src".to_string(),
        parameter_id: "param-freq".to_string(),
        value: 880.0,
    });
    assert!(store.check_ownership(&agent_actor(), &cmd).is_ok());
}

#[test]
fn ownership_gate_agent_allowed_on_agent_owned_node() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let cmd = TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
        node_id: "node-agent".to_string(),
        parameter_id: "param-lvl".to_string(),
        value: 0.3,
    });
    assert!(store.check_ownership(&agent_actor(), &cmd).is_ok());
}

#[test]
fn ownership_gate_agent_rejected_on_locked_node() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let cmd = TypedCommand::GraphEdit(GraphEditCommand::RemoveNode {
        node_id: "node-locked".to_string(),
    });
    let result = store.check_ownership(&agent_actor(), &cmd);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err()[0].reason,
        OwnershipGateReason::LockedNode
    );
}

#[test]
fn ownership_gate_user_passes_on_locked_node() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let cmd = TypedCommand::GraphEdit(GraphEditCommand::RemoveNode {
        node_id: "node-locked".to_string(),
    });
    assert!(store.check_ownership(&user_actor(), &cmd).is_ok());
}

#[test]
fn freeze_toggle_blocks_all_agent_commands() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    agent_command::toggle_agent_freeze(&mut store).unwrap();
    assert!(store.current().agent_frozen);

    let cmd = TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
        node_id: "node-agent".to_string(),
        parameter_id: "param-lvl".to_string(),
        value: 0.1,
    });
    let result = store.check_ownership(&agent_actor(), &cmd);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err()[0].reason,
        OwnershipGateReason::AgentFrozen
    );

    agent_command::toggle_agent_freeze(&mut store).unwrap();
    assert!(!store.current().agent_frozen);
    assert!(store.check_ownership(&agent_actor(), &cmd).is_ok());
}

#[test]
fn freeze_does_not_block_user_commands() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    agent_command::toggle_agent_freeze(&mut store).unwrap();

    let cmd = TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
        node_id: "node-user".to_string(),
        parameter_id: "param-mix".to_string(),
        value: 0.9,
    });
    assert!(store.check_ownership(&user_actor(), &cmd).is_ok());
}

#[test]
fn reclaim_ownership_transfers_all_agent_nodes() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let session = agent_command::reclaim_ownership(&mut store, None, None).unwrap();
    for node in &session.nodes {
        assert_ne!(node.ownership.controller, ControllerKind::Agent);
    }
    let agent_node = session.nodes.iter().find(|n| n.id == "node-agent").unwrap();
    assert_eq!(agent_node.ownership.controller, ControllerKind::User);
}

#[test]
fn reclaim_does_not_change_user_or_shared_nodes() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let before = store.current();
    let session = agent_command::reclaim_ownership(&mut store, None, None).unwrap();

    let user_node = session.nodes.iter().find(|n| n.id == "node-user").unwrap();
    let before_user = before.nodes.iter().find(|n| n.id == "node-user").unwrap();
    assert_eq!(
        user_node.ownership.controller,
        before_user.ownership.controller
    );

    let shared_node = session.nodes.iter().find(|n| n.id == "node-src").unwrap();
    let before_shared = before.nodes.iter().find(|n| n.id == "node-src").unwrap();
    assert_eq!(
        shared_node.ownership.controller,
        before_shared.ownership.controller
    );
}

#[test]
fn apply_agent_command_applies_valid_commands() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let actor = agent_actor();
    let intent = AgentIntent {
        raw_input: "set level to 0.1 on node-agent".to_string(),
        parsed_commands: vec![TypedCommand::GraphEdit(
            GraphEditCommand::SetParameterValue {
                node_id: "node-agent".to_string(),
                parameter_id: "param-lvl".to_string(),
                value: 0.1,
            },
        )],
        confidence: 0.9,
    };

    let result = agent_command::apply_agent_command(&mut store, actor, intent).unwrap();
    assert_eq!(result.applied.len(), 1);
    assert!(result.rejected.is_empty());

    let current = store.current();
    let node = current.nodes.iter().find(|n| n.id == "node-agent").unwrap();
    assert_eq!(node.parameters[0].value, 0.1);
}

#[test]
fn apply_agent_command_rejects_user_owned_commands() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let actor = agent_actor();
    let intent = AgentIntent {
        raw_input: "set mix to 0.9 on node-user".to_string(),
        parsed_commands: vec![TypedCommand::GraphEdit(
            GraphEditCommand::SetParameterValue {
                node_id: "node-user".to_string(),
                parameter_id: "param-mix".to_string(),
                value: 0.9,
            },
        )],
        confidence: 0.9,
    };

    let result = agent_command::apply_agent_command(&mut store, actor, intent).unwrap();
    assert!(result.applied.is_empty());
    assert_eq!(result.rejected.len(), 1);
    assert!(result.rejected[0].1.contains("user-owned"));
}

#[test]
fn apply_agent_command_mixed_approved_and_rejected() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let actor = agent_actor();
    let intent = AgentIntent {
        raw_input: "multiple commands".to_string(),
        parsed_commands: vec![
            TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
                node_id: "node-agent".to_string(),
                parameter_id: "param-lvl".to_string(),
                value: 0.2,
            }),
            TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
                node_id: "node-user".to_string(),
                parameter_id: "param-mix".to_string(),
                value: 0.9,
            }),
        ],
        confidence: 0.8,
    };

    let result = agent_command::apply_agent_command(&mut store, actor, intent).unwrap();
    assert_eq!(result.applied.len(), 1);
    assert_eq!(result.rejected.len(), 1);
}

#[test]
fn agent_command_blocked_when_frozen() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());
    agent_command::toggle_agent_freeze(&mut store).unwrap();

    let actor = agent_actor();
    let intent = AgentIntent {
        raw_input: "set level to 0.1 on node-agent".to_string(),
        parsed_commands: vec![TypedCommand::GraphEdit(
            GraphEditCommand::SetParameterValue {
                node_id: "node-agent".to_string(),
                parameter_id: "param-lvl".to_string(),
                value: 0.1,
            },
        )],
        confidence: 0.9,
    };

    let result = agent_command::apply_agent_command(&mut store, actor, intent).unwrap();
    assert!(result.applied.is_empty());
    assert_eq!(result.rejected.len(), 1);
    assert!(result.rejected[0].1.contains("frozen"));
}

#[test]
fn end_to_end_parse_and_apply_agent_add_node() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let session_before = store.current();
    let node_count_before = session_before.nodes.len();

    let actor = agent_actor();
    let intent = agent_command::parse_agent_intent("add oscillator", &store.current());
    let result = agent_command::apply_agent_command(&mut store, actor, intent).unwrap();

    assert!(!result.applied.is_empty());
    assert!(store.current().nodes.len() > node_count_before);
}

#[test]
fn intent_parser_delete_keyword_works_as_remove() {
    let session = test_session();
    let intent = agent_command::parse_agent_intent("delete node-src", &session);
    assert_eq!(intent.parsed_commands.len(), 1);
    match &intent.parsed_commands[0] {
        TypedCommand::GraphEdit(GraphEditCommand::RemoveNode { node_id }) => {
            assert_eq!(node_id, "node-src");
        }
        _ => panic!("expected RemoveNode"),
    }
}
