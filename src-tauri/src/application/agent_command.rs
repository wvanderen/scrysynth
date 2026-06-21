use thiserror::Error;

use crate::application::agent_planner::{
    plan_agent_request, PlannerProposal, PlannerProvider, PlannerProviderError,
    SessionContextBounds,
};
use crate::application::graph_edit::{self, GraphEditError};
use crate::application::performance_command::{self, PerformanceCommandError};
use crate::application::session_store::SessionStore;
use crate::domain::session::{
    new_id, ActorRef, AgentIntent, AudioPrimitive, AudioSourceNode, AudioSourceType, ChannelMode,
    ControllerKind, GraphEditCommand, Node, NodeType, OwnershipAssignment, PendingAction,
    PendingActionStatus, PerformanceCommand, Port, PortDirection, RiskTier, Route, SessionDocument,
    SignalType, TypedCommand,
};

#[derive(Debug, Error, PartialEq)]
pub enum AgentCommandError {
    #[error("ownership gate rejected: {0}")]
    OwnershipBlocked(String),
    #[error("graph edit failed: {0}")]
    GraphEdit(#[from] GraphEditError),
    #[error("performance command failed: {0}")]
    PerformanceCommand(#[from] PerformanceCommandError),
    #[error("planner failed: {0}")]
    Planner(#[from] PlannerProviderError),
    #[error("no session is loaded")]
    NoSession,
}

pub fn parse_agent_intent(input: &str, session: &SessionDocument) -> AgentIntent {
    let lower = input.to_lowercase();
    let mut commands = Vec::new();
    let mut confidence: f64 = 0.0;

    if lower.contains("add") {
        if let Some(cmd) = parse_add_command(&lower, session) {
            commands.push(cmd);
            confidence = 0.9;
        }
    }

    if lower.contains("remove") || lower.contains("delete") {
        if let Some(cmd) = parse_remove_command(&lower, session) {
            commands.push(cmd);
            confidence = confidence.max(0.9);
        }
    }

    if lower.contains("set") {
        if let Some(cmd) = parse_set_parameter_command(&lower, session) {
            commands.push(cmd);
            confidence = confidence.max(0.85);
        }
    }

    if lower.contains("recall scene") {
        if let Some(cmd) = parse_recall_scene_command(&lower, session) {
            commands.push(cmd);
            confidence = confidence.max(0.9);
        }
    }

    if lower.contains("save variation") {
        if let Some(cmd) = parse_save_variation_command(&lower, session) {
            commands.push(cmd);
            confidence = confidence.max(0.9);
        }
    }

    if lower.contains("restore variation") {
        if let Some(cmd) = parse_restore_variation_command(&lower, session) {
            commands.push(cmd);
            confidence = confidence.max(0.9);
        }
    }

    if !commands.is_empty() && confidence == 0.0 {
        confidence = 0.7;
    }

    AgentIntent {
        raw_input: input.to_string(),
        parsed_commands: commands,
        confidence,
    }
}

fn parse_add_command(lower: &str, _session: &SessionDocument) -> Option<TypedCommand> {
    let source_type = if lower.contains("oscillator") || lower.contains("osc") {
        AudioSourceType::Oscillator
    } else if lower.contains("noise") {
        AudioSourceType::Noise
    } else {
        return None;
    };

    let node_id = new_id();
    let port_id = new_id();
    let param_id = new_id();

    let node = Node {
        id: node_id,
        node_type: NodeType::Source,
        ports: vec![Port {
            id: port_id,
            name: "main_out".to_string(),
            direction: PortDirection::Output,
            signal_type: SignalType::Audio,
        }],
        parameters: vec![crate::domain::session::ParameterValue {
            id: param_id,
            name: "level".to_string(),
            value: 0.8,
            default_value: 0.8,
            min_value: 0.0,
            max_value: 1.0,
            unit: "linear".to_string(),
        }],
        runtime_target: Some("audio/source/default".to_string()),
        scene_membership: vec![],
        ownership: OwnershipAssignment {
            controller: ControllerKind::Agent,
            is_locked: false,
        },
        enabled: true,
        audio_primitive: Some(AudioPrimitive::Source(AudioSourceNode {
            source_type,
            channel_mode: ChannelMode::Mono,
            bus_target_id: None,
        })),
    };

    Some(TypedCommand::GraphEdit(GraphEditCommand::AddNode { node }))
}

pub fn classify_risk(command: &TypedCommand) -> RiskTier {
    match command {
        TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue { .. }) => RiskTier::Low,
        TypedCommand::Performance(PerformanceCommand::RestoreVariation { .. }) => RiskTier::Low,
        TypedCommand::GraphEdit(GraphEditCommand::AddNode { .. }) => RiskTier::Medium,
        TypedCommand::GraphEdit(GraphEditCommand::SetNodeEnabled { .. }) => RiskTier::Medium,
        TypedCommand::Performance(PerformanceCommand::RecallScene { .. }) => RiskTier::Medium,
        TypedCommand::GraphEdit(GraphEditCommand::AddRoute { .. }) => RiskTier::Medium,
        TypedCommand::Performance(PerformanceCommand::SaveVariation { .. }) => RiskTier::Low,
        TypedCommand::GraphEdit(GraphEditCommand::RemoveNode { .. }) => RiskTier::High,
        TypedCommand::GraphEdit(GraphEditCommand::RemoveRoute { .. }) => RiskTier::High,
        TypedCommand::GraphEdit(GraphEditCommand::ClearNodeBusAssignment { .. }) => RiskTier::High,
        TypedCommand::GraphEdit(GraphEditCommand::AssignNodeToBus { .. }) => RiskTier::Medium,
    }
}

fn parse_remove_command(lower: &str, session: &SessionDocument) -> Option<TypedCommand> {
    let node_id = extract_node_reference(lower, session)?;
    Some(TypedCommand::GraphEdit(GraphEditCommand::RemoveNode {
        node_id,
    }))
}

fn parse_set_parameter_command(lower: &str, session: &SessionDocument) -> Option<TypedCommand> {
    let node_id = extract_node_reference(lower, session)?;
    let node = session.nodes.iter().find(|n| n.id == node_id)?;

    let value = extract_numeric_value(lower)?;
    let parameter = node.parameters.first()?;
    let parameter_id = parameter.id.clone();

    Some(TypedCommand::GraphEdit(
        GraphEditCommand::SetParameterValue {
            node_id,
            parameter_id,
            value,
        },
    ))
}

fn parse_recall_scene_command(lower: &str, session: &SessionDocument) -> Option<TypedCommand> {
    let scene = find_best_match(lower, &session.scenes, |s| &s.name, "scene")?;
    Some(TypedCommand::Performance(PerformanceCommand::RecallScene {
        scene_id: scene.id.clone(),
    }))
}

fn parse_save_variation_command(lower: &str, session: &SessionDocument) -> Option<TypedCommand> {
    let name = extract_quoted_name(lower).unwrap_or_else(|| "agent-variation".to_string());

    let scene_id = if let Some(scene_name) = extract_after_keyword(lower, "for scene") {
        session
            .scenes
            .iter()
            .find(|s| s.name.to_lowercase().contains(&scene_name.to_lowercase()))
            .map(|s| s.id.clone())
    } else {
        session.scenes.first().map(|s| s.id.clone())
    }?;

    Some(TypedCommand::Performance(
        PerformanceCommand::SaveVariation { name, scene_id },
    ))
}

fn parse_restore_variation_command(
    _lower: &str,
    session: &SessionDocument,
) -> Option<TypedCommand> {
    let variation = session.variations.first()?;
    Some(TypedCommand::Performance(
        PerformanceCommand::RestoreVariation {
            variation_id: variation.id.clone(),
        },
    ))
}

fn extract_node_reference<'a>(lower: &str, session: &'a SessionDocument) -> Option<String> {
    let words: Vec<&str> = lower.split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        if *word == "node" && i + 1 < words.len() {
            let candidate = words[i + 1];
            if let Some(node) = session
                .nodes
                .iter()
                .find(|n| n.id.to_lowercase() == candidate || n.id == candidate)
            {
                return Some(node.id.clone());
            }
        }
    }

    for word in &words {
        if let Some(node) = session
            .nodes
            .iter()
            .find(|n| n.id == *word || n.id.to_lowercase() == word.to_lowercase())
        {
            return Some(node.id.clone());
        }
    }

    None
}

fn extract_numeric_value(lower: &str) -> Option<f64> {
    let words: Vec<&str> = lower.split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        if (*word == "to" || *word == "=") && i + 1 < words.len() {
            if let Ok(value) = words[i + 1].parse::<f64>() {
                return Some(value);
            }
        }
    }
    for word in &words {
        if let Ok(value) = word.parse::<f64>() {
            return Some(value);
        }
    }
    None
}

fn extract_quoted_name(input: &str) -> Option<String> {
    if let Some(start) = input.find('"') {
        if let Some(end) = input[start + 1..].find('"') {
            return Some(input[start + 1..start + 1 + end].to_string());
        }
    }
    if let Some(start) = input.find('\'') {
        if let Some(end) = input[start + 1..].find('\'') {
            return Some(input[start + 1..start + 1 + end].to_string());
        }
    }
    None
}

fn extract_after_keyword<'a>(lower: &str, keyword: &str) -> Option<String> {
    let idx = lower.find(keyword)?;
    let rest = &lower[idx + keyword.len()..].trim();
    if rest.is_empty() {
        return None;
    }
    Some(rest.to_string())
}

fn find_best_match<'a, T>(
    lower: &str,
    items: &'a [T],
    get_name: impl Fn(&T) -> &str,
    _keyword: &str,
) -> Option<&'a T> {
    items
        .iter()
        .find(|item| lower.contains(&get_name(item).to_lowercase()))
}

pub struct AgentCommandResult {
    pub applied: Vec<TypedCommand>,
    pub rejected: Vec<(TypedCommand, String)>,
    pub pending: Vec<PendingAction>,
    pub intent: AgentIntent,
    pub planner_provider_id: Option<String>,
}

pub fn apply_agent_command(
    store: &mut SessionStore,
    actor: ActorRef,
    intent: AgentIntent,
) -> Result<AgentCommandResult, AgentCommandError> {
    let mut applied = Vec::new();
    let mut rejected = Vec::new();
    let mut pending = Vec::new();

    for command in intent.parsed_commands.clone() {
        if let Err(reason) = validate_proposed_command(&store.current(), &command) {
            rejected.push((command, reason));
            continue;
        }

        match store.check_ownership(&actor, &command) {
            Ok(()) => {
                let risk = classify_risk(&command);
                if risk == RiskTier::High && actor.actor_id != "user" {
                    let pa = PendingAction {
                        id: new_id(),
                        correlation_id: actor.correlation_id.clone(),
                        command: command.clone(),
                        risk_tier: RiskTier::High,
                        created_at: chrono_now_string(),
                        status: PendingActionStatus::Pending,
                    };
                    store
                        .mutate_current(|session| {
                            session.pending_actions.push(pa.clone());
                            Ok::<(), String>(())
                        })
                        .map_err(|e| AgentCommandError::OwnershipBlocked(e))?;
                    pending.push(pa);
                } else {
                    match apply_typed_command(store, &command, &actor) {
                        Ok(_) => applied.push(command),
                        Err(e) => rejected.push((command, e.to_string())),
                    }
                }
            }
            Err(errors) => {
                let reason = errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                rejected.push((command, reason));
            }
        }
    }

    Ok(AgentCommandResult {
        applied,
        rejected,
        pending,
        intent,
        planner_provider_id: None,
    })
}

pub fn plan_and_apply_agent_request(
    store: &mut SessionStore,
    actor: ActorRef,
    raw_input: &str,
    provider: &dyn PlannerProvider,
) -> Result<AgentCommandResult, AgentCommandError> {
    let proposal = plan_agent_request(
        &store.current(),
        raw_input,
        provider,
        SessionContextBounds::default(),
    )?;
    apply_planner_proposal(store, actor, proposal)
}

pub fn apply_planner_proposal(
    store: &mut SessionStore,
    actor: ActorRef,
    proposal: PlannerProposal,
) -> Result<AgentCommandResult, AgentCommandError> {
    let provider_id = proposal.provider_id.clone();
    let mut result = apply_agent_command(store, actor, proposal.into_intent())?;
    result.planner_provider_id = Some(provider_id);
    Ok(result)
}

fn chrono_now_string() -> String {
    "2026-04-12T00:00:00Z".to_string()
}

pub fn approve_pending_action(
    store: &mut SessionStore,
    pending_action_id: &str,
) -> Result<SessionDocument, AgentCommandError> {
    let pa = store
        .current()
        .pending_actions
        .iter()
        .find(|pa| pa.id == pending_action_id && pa.status == PendingActionStatus::Pending)
        .cloned()
        .ok_or_else(|| {
            AgentCommandError::OwnershipBlocked(format!(
                "pending action '{}' not found or already resolved",
                pending_action_id
            ))
        })?;

    validate_proposed_command(&store.current(), &pa.command)
        .map_err(AgentCommandError::OwnershipBlocked)?;

    store
        .mutate_current(|session| {
            if let Some(pa) = session
                .pending_actions
                .iter_mut()
                .find(|p| p.id == pending_action_id)
            {
                pa.status = PendingActionStatus::Approved;
            }
            Ok::<(), String>(())
        })
        .map_err(|e| AgentCommandError::OwnershipBlocked(e))?;

    let result = apply_typed_command(
        store,
        &pa.command,
        &ActorRef {
            actor_id: "user".to_string(),
            correlation_id: pa.correlation_id.clone(),
        },
    )?;

    store
        .mutate_current(|session| {
            session
                .pending_actions
                .retain(|p| p.id != pending_action_id);
            Ok::<(), String>(())
        })
        .map_err(|e| AgentCommandError::OwnershipBlocked(e))?;

    store.log_action_with_description(
        &ActorRef {
            actor_id: "user".to_string(),
            correlation_id: pa.correlation_id.clone(),
        },
        &pa.command,
        Some(format!("Approved pending action {}", pending_action_id)),
    );

    Ok(result)
}

pub fn reject_pending_action(
    store: &mut SessionStore,
    pending_action_id: &str,
) -> Result<SessionDocument, AgentCommandError> {
    let rejected_command = store
        .current()
        .pending_actions
        .iter()
        .find(|pa| pa.id == pending_action_id && pa.status == PendingActionStatus::Pending)
        .map(|pa| (pa.command.clone(), pa.correlation_id.clone()));

    store
        .mutate_current(|session| {
            let pa = session
                .pending_actions
                .iter_mut()
                .find(|pa| pa.id == pending_action_id && pa.status == PendingActionStatus::Pending);
            match pa {
                Some(pa) => {
                    pa.status = PendingActionStatus::Rejected;
                    Ok::<(), String>(())
                }
                None => Err(format!(
                    "pending action '{}' not found or already resolved",
                    pending_action_id
                )),
            }
        })
        .map_err(|e| AgentCommandError::OwnershipBlocked(e))?;

    if let Some((command, correlation_id)) = rejected_command {
        store.log_action_with_description(
            &ActorRef {
                actor_id: "user".to_string(),
                correlation_id,
            },
            &command,
            Some(format!("Rejected pending action {}", pending_action_id)),
        );
    }

    let session = store.current();
    Ok(session)
}

pub fn validate_proposed_command(
    session: &SessionDocument,
    command: &TypedCommand,
) -> Result<RiskTier, String> {
    match command {
        TypedCommand::GraphEdit(graph_command) => validate_graph_proposal(session, graph_command)?,
        TypedCommand::Performance(performance_command) => {
            validate_performance_proposal(session, performance_command)?
        }
    }

    Ok(classify_risk(command))
}

fn validate_graph_proposal(
    session: &SessionDocument,
    command: &GraphEditCommand,
) -> Result<(), String> {
    match command {
        GraphEditCommand::AddNode { node } => validate_add_node(session, node),
        GraphEditCommand::RemoveNode { node_id }
        | GraphEditCommand::SetNodeEnabled { node_id, .. } => {
            require_node(session, node_id).map(|_| ())
        }
        GraphEditCommand::SetParameterValue {
            node_id,
            parameter_id,
            value,
        } => {
            let node = require_node(session, node_id)?;
            let parameter = node
                .parameters
                .iter()
                .find(|parameter| &parameter.id == parameter_id)
                .ok_or_else(|| {
                    format!(
                        "parameter '{}' was not found on node '{}'",
                        parameter_id, node_id
                    )
                })?;
            if *value < parameter.min_value || *value > parameter.max_value {
                return Err(format!(
                    "parameter '{}' on node '{}' must be between {} and {}",
                    parameter_id, node_id, parameter.min_value, parameter.max_value
                ));
            }
            Ok(())
        }
        GraphEditCommand::AddRoute { route } => validate_route_proposal(session, route),
        GraphEditCommand::RemoveRoute { route_id } => session
            .routes
            .iter()
            .any(|route| &route.id == route_id)
            .then_some(())
            .ok_or_else(|| format!("route '{}' was not found", route_id)),
        GraphEditCommand::AssignNodeToBus { node_id, bus_id } => {
            let node = require_node(session, node_id)?;
            if node.audio_primitive.is_none() {
                return Err(format!(
                    "node '{}' does not support bus assignment",
                    node_id
                ));
            }
            session
                .buses
                .iter()
                .any(|bus| &bus.id == bus_id)
                .then_some(())
                .ok_or_else(|| format!("bus '{}' was not found", bus_id))
        }
        GraphEditCommand::ClearNodeBusAssignment { node_id } => {
            let node = require_node(session, node_id)?;
            if node.audio_primitive.is_none() {
                return Err(format!(
                    "node '{}' does not support bus assignment",
                    node_id
                ));
            }
            Ok(())
        }
    }
}

fn validate_add_node(session: &SessionDocument, node: &Node) -> Result<(), String> {
    if session.nodes.iter().any(|existing| existing.id == node.id) {
        return Err(format!("node '{}' already exists", node.id));
    }

    for parameter in &node.parameters {
        if parameter.min_value > parameter.max_value {
            return Err(format!(
                "parameter '{}' on node '{}' has an invalid range",
                parameter.id, node.id
            ));
        }
        if parameter.value < parameter.min_value || parameter.value > parameter.max_value {
            return Err(format!(
                "parameter '{}' on node '{}' must be between {} and {}",
                parameter.id, node.id, parameter.min_value, parameter.max_value
            ));
        }
    }

    Ok(())
}

fn validate_route_proposal(session: &SessionDocument, route: &Route) -> Result<(), String> {
    graph_edit::validate_route(session, route).map_err(|err| err.to_string())?;
    if let Some(bus_id) = &route.bus_id {
        session
            .buses
            .iter()
            .any(|bus| &bus.id == bus_id)
            .then_some(())
            .ok_or_else(|| format!("bus '{}' was not found", bus_id))?;
    }
    Ok(())
}

fn validate_performance_proposal(
    session: &SessionDocument,
    command: &PerformanceCommand,
) -> Result<(), String> {
    match command {
        PerformanceCommand::RecallScene { scene_id }
        | PerformanceCommand::SaveVariation { scene_id, .. } => session
            .scenes
            .iter()
            .any(|scene| &scene.id == scene_id)
            .then_some(())
            .ok_or_else(|| format!("scene '{}' was not found", scene_id)),
        PerformanceCommand::RestoreVariation { variation_id } => session
            .variations
            .iter()
            .find(|variation| &variation.id == variation_id)
            .ok_or_else(|| format!("variation '{}' was not found", variation_id))
            .and_then(|variation| {
                for override_param in &variation.parameter_overrides {
                    let parameter = session
                        .nodes
                        .iter()
                        .flat_map(|node| {
                            node.parameters
                                .iter()
                                .map(move |parameter| (node.id.as_str(), parameter))
                        })
                        .find(|(_, parameter)| parameter.id == override_param.parameter_id);
                    if let Some((node_id, parameter)) = parameter {
                        if override_param.value < parameter.min_value
                            || override_param.value > parameter.max_value
                        {
                            return Err(format!(
                                "parameter '{}' is out of range on node '{}'",
                                parameter.id, node_id
                            ));
                        }
                    }
                }
                Ok(())
            }),
    }
}

fn require_node<'a>(session: &'a SessionDocument, node_id: &str) -> Result<&'a Node, String> {
    session
        .nodes
        .iter()
        .find(|node| node.id == node_id)
        .ok_or_else(|| format!("node '{}' was not found", node_id))
}

fn apply_typed_command(
    store: &mut SessionStore,
    command: &TypedCommand,
    actor: &ActorRef,
) -> Result<SessionDocument, AgentCommandError> {
    let result = match command {
        TypedCommand::GraphEdit(gec) => {
            graph_edit::apply_graph_edit(store, gec.clone()).map_err(AgentCommandError::from)
        }
        TypedCommand::Performance(pc) => {
            performance_command::apply_performance_command(store, pc.clone())
                .map_err(AgentCommandError::from)
        }
    };

    if result.is_ok() {
        store.log_action(actor, command);
    }

    result
}

pub fn toggle_agent_freeze(store: &mut SessionStore) -> Result<SessionDocument, String> {
    store
        .mutate_current(|session| {
            session.agent_frozen = !session.agent_frozen;
            Ok::<(), String>(())
        })
        .map_err(|e| e)
}

pub fn reclaim_ownership(
    store: &mut SessionStore,
    node_ids: Option<Vec<String>>,
    target_controller: Option<ControllerKind>,
) -> Result<SessionDocument, String> {
    let target = target_controller.unwrap_or(ControllerKind::User);
    store.mutate_current(|session| {
        for node in &mut session.nodes {
            if let Some(ref ids) = node_ids {
                if ids.contains(&node.id) {
                    node.ownership.controller = target.clone();
                }
            } else {
                if node.ownership.controller == ControllerKind::Agent {
                    node.ownership.controller = target.clone();
                }
            }
        }
        Ok::<(), String>(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::session_store::OwnershipGateReason;
    use crate::domain::session::{ParameterValue, Port, SceneDefinition};

    fn test_session_with_nodes() -> SessionDocument {
        SessionDocument {
            nodes: vec![
                Node {
                    id: "node-1".to_string(),
                    node_type: NodeType::Source,
                    ports: vec![Port {
                        id: "port-1-out".to_string(),
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
                    id: "node-2".to_string(),
                    node_type: NodeType::Output,
                    ports: vec![],
                    parameters: vec![],
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
                    id: "node-3".to_string(),
                    node_type: NodeType::Source,
                    ports: vec![],
                    parameters: vec![ParameterValue {
                        id: "param-lvl".to_string(),
                        name: "level".to_string(),
                        value: 0.5,
                        default_value: 0.5,
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
                    node_type: NodeType::Source,
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
                active_node_ids: vec!["node-1".to_string()],
                macro_overrides: vec![],
            }],
            ..SessionDocument::default()
        }
    }

    #[test]
    fn parse_add_oscillator() {
        let session = test_session_with_nodes();
        let intent = parse_agent_intent("add oscillator", &session);
        assert!(!intent.parsed_commands.is_empty());
        assert!(intent.confidence > 0.0);
        match &intent.parsed_commands[0] {
            TypedCommand::GraphEdit(GraphEditCommand::AddNode { node }) => {
                assert_eq!(node.node_type, NodeType::Source);
            }
            _ => panic!("expected AddNode command"),
        }
    }

    #[test]
    fn parse_add_noise() {
        let session = test_session_with_nodes();
        let intent = parse_agent_intent("add noise", &session);
        assert!(!intent.parsed_commands.is_empty());
    }

    #[test]
    fn parse_remove_node() {
        let session = test_session_with_nodes();
        let intent = parse_agent_intent("remove node-1", &session);
        assert!(!intent.parsed_commands.is_empty());
        match &intent.parsed_commands[0] {
            TypedCommand::GraphEdit(GraphEditCommand::RemoveNode { node_id }) => {
                assert_eq!(node_id, "node-1");
            }
            _ => panic!("expected RemoveNode command"),
        }
    }

    #[test]
    fn parse_set_parameter() {
        let session = test_session_with_nodes();
        let intent = parse_agent_intent("set frequency to 880 on node-1", &session);
        assert!(!intent.parsed_commands.is_empty());
        match &intent.parsed_commands[0] {
            TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
                node_id, value, ..
            }) => {
                assert_eq!(node_id, "node-1");
                assert_eq!(*value, 880.0);
            }
            _ => panic!("expected SetParameterValue command"),
        }
    }

    #[test]
    fn parse_recall_scene() {
        let session = test_session_with_nodes();
        let intent = parse_agent_intent("recall scene intro", &session);
        assert!(!intent.parsed_commands.is_empty());
        match &intent.parsed_commands[0] {
            TypedCommand::Performance(PerformanceCommand::RecallScene { scene_id }) => {
                assert_eq!(scene_id, "scene-intro");
            }
            _ => panic!("expected RecallScene command"),
        }
    }

    #[test]
    fn parse_unrecognized_input() {
        let session = test_session_with_nodes();
        let intent = parse_agent_intent("hello what is the weather", &session);
        assert!(intent.parsed_commands.is_empty());
        assert_eq!(intent.confidence, 0.0);
    }

    #[test]
    fn ownership_gate_allows_user_always() {
        let store = SessionStore::new_default();
        let actor = ActorRef {
            actor_id: "user".to_string(),
            correlation_id: new_id(),
        };
        let cmd = TypedCommand::GraphEdit(GraphEditCommand::RemoveNode {
            node_id: "anything".to_string(),
        });
        assert!(store.check_ownership(&actor, &cmd).is_ok());
    }

    #[test]
    fn ownership_gate_rejects_agent_when_frozen() {
        let mut store = SessionStore::new_default();
        store
            .mutate_current(|s| {
                s.agent_frozen = true;
                Ok::<(), ()>(())
            })
            .unwrap();

        let actor = ActorRef {
            actor_id: "agent".to_string(),
            correlation_id: new_id(),
        };
        let cmd = TypedCommand::GraphEdit(GraphEditCommand::RemoveNode {
            node_id: "anything".to_string(),
        });
        let result = store.check_ownership(&actor, &cmd);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].reason, OwnershipGateReason::AgentFrozen);
    }

    #[test]
    fn ownership_gate_rejects_agent_on_user_owned_node() {
        let mut store = SessionStore::new_default();
        let session = test_session_with_nodes();
        store.replace_current(session);

        let actor = ActorRef {
            actor_id: "agent".to_string(),
            correlation_id: new_id(),
        };
        let cmd = TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
            node_id: "node-2".to_string(),
            parameter_id: "any".to_string(),
            value: 1.0,
        });
        let result = store.check_ownership(&actor, &cmd);
        assert!(result.is_err());
    }

    #[test]
    fn ownership_gate_allows_agent_on_shared_node() {
        let mut store = SessionStore::new_default();
        let session = test_session_with_nodes();
        store.replace_current(session);

        let actor = ActorRef {
            actor_id: "agent".to_string(),
            correlation_id: new_id(),
        };
        let cmd = TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
            node_id: "node-1".to_string(),
            parameter_id: "param-freq".to_string(),
            value: 880.0,
        });
        assert!(store.check_ownership(&actor, &cmd).is_ok());
    }

    #[test]
    fn ownership_gate_rejects_on_locked_node() {
        let mut store = SessionStore::new_default();
        let session = test_session_with_nodes();
        store.replace_current(session);

        let actor = ActorRef {
            actor_id: "agent".to_string(),
            correlation_id: new_id(),
        };
        let cmd = TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
            node_id: "node-locked".to_string(),
            parameter_id: "any".to_string(),
            value: 1.0,
        });
        let result = store.check_ownership(&actor, &cmd);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err()[0].reason,
            OwnershipGateReason::LockedNode
        );
    }

    #[test]
    fn apply_agent_command_respects_ownership() {
        let mut store = SessionStore::new_default();
        let session = test_session_with_nodes();
        store.replace_current(session);

        let actor = ActorRef {
            actor_id: "agent".to_string(),
            correlation_id: new_id(),
        };
        let intent = AgentIntent {
            raw_input: "set frequency to 880 on node-1".to_string(),
            parsed_commands: vec![TypedCommand::GraphEdit(
                GraphEditCommand::SetParameterValue {
                    node_id: "node-1".to_string(),
                    parameter_id: "param-freq".to_string(),
                    value: 880.0,
                },
            )],
            confidence: 0.9,
        };

        let result = apply_agent_command(&mut store, actor, intent).unwrap();
        assert_eq!(result.applied.len(), 1);
        assert!(result.rejected.is_empty());
    }

    #[test]
    fn apply_agent_command_rejects_user_owned() {
        let mut store = SessionStore::new_default();
        let session = test_session_with_nodes();
        store.replace_current(session);

        let actor = ActorRef {
            actor_id: "agent".to_string(),
            correlation_id: new_id(),
        };
        let intent = AgentIntent {
            raw_input: "remove node-2".to_string(),
            parsed_commands: vec![TypedCommand::GraphEdit(GraphEditCommand::RemoveNode {
                node_id: "node-2".to_string(),
            })],
            confidence: 0.9,
        };

        let result = apply_agent_command(&mut store, actor, intent).unwrap();
        assert!(result.applied.is_empty());
        assert_eq!(result.rejected.len(), 1);
    }

    #[test]
    fn toggle_agent_freeze_flips_flag() {
        let mut store = SessionStore::new_default();
        assert!(!store.current().agent_frozen);

        let session = toggle_agent_freeze(&mut store).unwrap();
        assert!(session.agent_frozen);

        let session = toggle_agent_freeze(&mut store).unwrap();
        assert!(!session.agent_frozen);
    }

    #[test]
    fn reclaim_ownership_transfers_agent_nodes_to_user() {
        let mut store = SessionStore::new_default();
        let session = test_session_with_nodes();
        store.replace_current(session);

        let session = reclaim_ownership(&mut store, None, None).unwrap();
        for node in &session.nodes {
            assert_ne!(node.ownership.controller, ControllerKind::Agent);
        }
    }

    #[test]
    fn reclaim_ownership_specific_nodes_with_controller() {
        let mut store = SessionStore::new_default();
        let session = test_session_with_nodes();
        store.replace_current(session);

        let session = reclaim_ownership(
            &mut store,
            Some(vec!["node-1".to_string()]),
            Some(ControllerKind::Agent),
        )
        .unwrap();
        let node1 = session.nodes.iter().find(|n| n.id == "node-1").unwrap();
        assert_eq!(node1.ownership.controller, ControllerKind::Agent);
    }
}
