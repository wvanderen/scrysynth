use scrysynth_lib::application::agent_command;
use scrysynth_lib::application::agent_planner::{
    derive_session_context_packet, plan_agent_request, ParserPlannerProvider, PlannerProvider,
    PlannerProviderError, PlannerProviderOutput, PlannerRequest, SessionContextBounds,
};
use scrysynth_lib::application::session_store::{OwnershipGateReason, SessionStore};
use scrysynth_lib::domain::session::{
    new_id, ActionHistoryEntry, ActorRef, AgentIntent, AgentRuntimeState, ControllerKind,
    DiffSummary, GraphEditCommand, Node, NodeType, OwnershipAssignment, OwnershipRule,
    ParameterValue, PendingAction, PendingActionStatus, PerformanceCommand, Port, PortDirection,
    RiskTier, Route, SceneDefinition, SessionDocument, SignalType, TypedCommand,
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

#[derive(Clone)]
struct JsonPlannerProvider {
    output: Result<PlannerProviderOutput, PlannerProviderError>,
    available: bool,
}

impl PlannerProvider for JsonPlannerProvider {
    fn provider_id(&self) -> &str {
        "json-test-provider"
    }

    fn is_available(&self) -> bool {
        self.available
    }

    fn plan(
        &self,
        _request: &PlannerRequest,
    ) -> Result<PlannerProviderOutput, PlannerProviderError> {
        self.output.clone()
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
fn session_context_packet_contains_bounded_planner_context() {
    let mut session = test_session();
    session.ownership_rules.push(OwnershipRule {
        id: "rule-1".to_string(),
        scope: "graph".to_string(),
        controller: ControllerKind::Shared,
        can_override: true,
    });
    session.pending_actions.push(PendingAction {
        id: "pending-old".to_string(),
        correlation_id: "corr-old".to_string(),
        command: TypedCommand::GraphEdit(GraphEditCommand::RemoveNode {
            node_id: "node-src".to_string(),
        }),
        risk_tier: RiskTier::High,
        created_at: "2026-04-12T00:00:00Z".to_string(),
        status: PendingActionStatus::Pending,
    });
    session.pending_actions.push(PendingAction {
        id: "pending-new".to_string(),
        correlation_id: "corr-new".to_string(),
        command: TypedCommand::GraphEdit(GraphEditCommand::RemoveNode {
            node_id: "node-agent".to_string(),
        }),
        risk_tier: RiskTier::High,
        created_at: "2026-04-13T00:00:00Z".to_string(),
        status: PendingActionStatus::Pending,
    });
    session.action_history.push(ActionHistoryEntry {
        id: "hist-old".to_string(),
        timestamp: "2026-04-12T00:00:00Z".to_string(),
        actor: user_actor(),
        command: TypedCommand::Performance(PerformanceCommand::RecallScene {
            scene_id: "scene-intro".to_string(),
        }),
        diff: DiffSummary {
            description: "old action".to_string(),
            affected_node_ids: vec!["node-src".to_string()],
            before_snippet: "before".to_string(),
            after_snippet: "after".to_string(),
        },
    });
    session.action_history.push(ActionHistoryEntry {
        id: "hist-new".to_string(),
        timestamp: "2026-04-13T00:00:00Z".to_string(),
        actor: agent_actor(),
        command: TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
            node_id: "node-agent".to_string(),
            parameter_id: "param-lvl".to_string(),
            value: 0.2,
        }),
        diff: DiffSummary {
            description: "new action".to_string(),
            affected_node_ids: vec!["node-agent".to_string()],
            before_snippet: "before".to_string(),
            after_snippet: "after".to_string(),
        },
    });

    let packet = derive_session_context_packet(
        &session,
        SessionContextBounds {
            max_nodes: 2,
            max_routes: 10,
            max_buses: 10,
            max_macros: 10,
            max_scenes: 10,
            max_ownership_rules: 10,
            max_pending_actions: 1,
            max_history_entries: 1,
        },
    );

    assert_eq!(packet.graph.nodes.len(), 2);
    assert_eq!(packet.scenes.len(), 1);
    assert_eq!(packet.ownership.rules.len(), 1);
    assert!(packet
        .ownership
        .locked_node_ids
        .contains(&"node-locked".to_string()));
    assert!(packet
        .ownership
        .user_owned_node_ids
        .contains(&"node-user".to_string()));
    assert!(packet
        .ownership
        .agent_owned_node_ids
        .contains(&"node-agent".to_string()));
    assert_eq!(packet.pending_actions[0].id, "pending-new");
    assert_eq!(packet.history.summaries[0].id, "hist-new");
    assert_eq!(packet.truncation.total_nodes, 4);
    assert_eq!(packet.truncation.included_nodes, 2);
}

#[test]
fn planner_json_output_is_parsed_into_typed_proposal_commands() {
    let session = test_session();
    let json = serde_json::json!({
        "rawInput": "set level",
        "commands": [
            {
                "type": "graphEdit",
                "payload": {
                    "type": "setParameterValue",
                    "payload": {
                        "node_id": "node-agent",
                        "parameter_id": "param-lvl",
                        "value": 0.25
                    }
                }
            }
        ],
        "confidence": 0.81
    })
    .to_string();
    let provider = JsonPlannerProvider {
        output: Ok(PlannerProviderOutput::Json(json)),
        available: true,
    };

    let proposal = plan_agent_request(
        &session,
        "set level",
        &provider,
        SessionContextBounds::default(),
    )
    .unwrap();

    assert_eq!(proposal.provider_id, "json-test-provider");
    assert_eq!(proposal.commands.len(), 1);
    match &proposal.commands[0] {
        TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue { node_id, value, .. }) => {
            assert_eq!(node_id, "node-agent");
            assert_eq!(*value, 0.25);
        }
        _ => panic!("expected typed SetParameterValue proposal"),
    }
}

#[test]
fn planner_provider_unavailable_is_explicit() {
    let session = test_session();
    let provider = ParserPlannerProvider::unavailable("offline-provider");

    let err = plan_agent_request(
        &session,
        "add oscillator",
        &provider,
        SessionContextBounds::default(),
    )
    .unwrap_err();

    assert_eq!(
        err,
        PlannerProviderError::Unavailable {
            provider_id: "offline-provider".to_string(),
            reason: "provider is not available".to_string()
        }
    );
}

#[test]
fn planner_session_unavailable_is_explicit() {
    let mut session = test_session();
    session.agent_runtime = AgentRuntimeState {
        is_available: false,
        pending_action_count: 0,
        is_frozen: false,
    };
    let provider = ParserPlannerProvider::default();

    let err = plan_agent_request(
        &session,
        "add oscillator",
        &provider,
        SessionContextBounds::default(),
    )
    .unwrap_err();

    assert_eq!(
        err,
        PlannerProviderError::Unavailable {
            provider_id: "local-parser".to_string(),
            reason: "session agent runtime is unavailable".to_string()
        }
    );
}

#[test]
fn invalid_planner_json_is_explicit() {
    let session = test_session();
    let provider = JsonPlannerProvider {
        output: Ok(PlannerProviderOutput::Json("{\"commands\":}".to_string())),
        available: true,
    };

    let err = plan_agent_request(
        &session,
        "bad output",
        &provider,
        SessionContextBounds::default(),
    )
    .unwrap_err();

    assert!(matches!(
        err,
        PlannerProviderError::InvalidOutput {
            provider_id,
            detail: _
        } if provider_id == "json-test-provider"
    ));
}

#[test]
fn plan_and_apply_routes_through_planner_before_validation() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());
    let provider = ParserPlannerProvider::default();

    let result = agent_command::plan_and_apply_agent_request(
        &mut store,
        agent_actor(),
        "set level to 0.2 on node-agent",
        &provider,
    )
    .unwrap();

    assert_eq!(result.planner_provider_id.as_deref(), Some("local-parser"));
    assert_eq!(result.applied.len(), 1);
    let current = store.current();
    let node = current.nodes.iter().find(|n| n.id == "node-agent").unwrap();
    assert_eq!(node.parameters[0].value, 0.2);
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
fn apply_agent_command_rejects_invalid_target_before_mutation() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let intent = AgentIntent {
        raw_input: "set missing".to_string(),
        parsed_commands: vec![TypedCommand::GraphEdit(
            GraphEditCommand::SetParameterValue {
                node_id: "missing-node".to_string(),
                parameter_id: "param-lvl".to_string(),
                value: 0.2,
            },
        )],
        confidence: 0.9,
    };

    let result = agent_command::apply_agent_command(&mut store, agent_actor(), intent).unwrap();
    assert!(result.applied.is_empty());
    assert!(result.pending.is_empty());
    assert_eq!(result.rejected.len(), 1);
    assert!(result.rejected[0]
        .1
        .contains("node 'missing-node' was not found"));
    assert!(store.current().action_history.is_empty());
}

#[test]
fn apply_agent_command_rejects_out_of_range_parameter_with_diagnostic() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let intent = AgentIntent {
        raw_input: "set level too hot".to_string(),
        parsed_commands: vec![TypedCommand::GraphEdit(
            GraphEditCommand::SetParameterValue {
                node_id: "node-agent".to_string(),
                parameter_id: "param-lvl".to_string(),
                value: 2.0,
            },
        )],
        confidence: 0.9,
    };

    let result = agent_command::apply_agent_command(&mut store, agent_actor(), intent).unwrap();
    assert!(result.applied.is_empty());
    assert_eq!(result.rejected.len(), 1);
    assert!(result.rejected[0].1.contains("must be between 0 and 1"));
}

#[test]
fn apply_agent_command_rejects_invalid_route_with_diagnostic() {
    let mut store = SessionStore::new_default();
    store.replace_current(test_session());

    let intent = AgentIntent {
        raw_input: "route bad target".to_string(),
        parsed_commands: vec![TypedCommand::GraphEdit(GraphEditCommand::AddRoute {
            route: Route {
                id: "bad-route".to_string(),
                source_node_id: "node-src".to_string(),
                source_port_id: "port-src-out".to_string(),
                target_node_id: "missing-node".to_string(),
                target_port_id: "missing-port".to_string(),
                bus_id: None,
            },
        })],
        confidence: 0.9,
    };

    let result = agent_command::apply_agent_command(&mut store, agent_actor(), intent).unwrap();
    assert!(result.applied.is_empty());
    assert_eq!(result.rejected.len(), 1);
    assert!(result.rejected[0].1.contains("missing-node"));
}

#[test]
fn remove_route_respects_endpoint_ownership() {
    let mut session = test_session();
    session.routes.push(Route {
        id: "route-user-owned".to_string(),
        source_node_id: "node-src".to_string(),
        source_port_id: "port-src-out".to_string(),
        target_node_id: "node-user".to_string(),
        target_port_id: "port-user-in".to_string(),
        bus_id: None,
    });

    let mut store = SessionStore::new_default();
    store.replace_current(session);

    let intent = AgentIntent {
        raw_input: "remove user route".to_string(),
        parsed_commands: vec![TypedCommand::GraphEdit(GraphEditCommand::RemoveRoute {
            route_id: "route-user-owned".to_string(),
        })],
        confidence: 0.9,
    };

    let result = agent_command::apply_agent_command(&mut store, agent_actor(), intent).unwrap();
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
