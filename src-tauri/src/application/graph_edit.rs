use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::application::session_store::SessionStore;
use crate::domain::session::{
    AudioPrimitive, GraphEditCommand, Node, ParameterValue, PortDirection, Route, SessionDocument,
};

#[derive(Debug, Error, PartialEq)]
pub enum GraphEditError {
    #[error("node '{node_id}' already exists")]
    DuplicateNode { node_id: String },
    #[error("node '{node_id}' was not found")]
    MissingNode { node_id: String },
    #[error("route '{route_id}' already exists")]
    DuplicateRoute { route_id: String },
    #[error("route '{route_id}' was not found")]
    MissingRoute { route_id: String },
    #[error("port '{port_id}' was not found on node '{node_id}'")]
    MissingPort { node_id: String, port_id: String },
    #[error("route '{route_id}' has invalid port direction")]
    InvalidRouteDirection { route_id: String },
    #[error("route '{route_id}' has mismatched signal types")]
    InvalidSignalType { route_id: String },
    #[error("route '{route_id}' creates an unsupported cycle")]
    UnsupportedCycle { route_id: String },
    #[error("parameter '{parameter_id}' was not found on node '{node_id}'")]
    MissingParameter {
        node_id: String,
        parameter_id: String,
    },
    #[error("parameter '{parameter_id}' on node '{node_id}' must be between {min} and {max}")]
    ParameterOutOfRange {
        node_id: String,
        parameter_id: String,
        min: f64,
        max: f64,
    },
    #[error("bus '{bus_id}' was not found")]
    MissingBus { bus_id: String },
    #[error("node '{node_id}' does not support bus assignment")]
    MissingAudioPrimitive { node_id: String },
}

pub fn apply_graph_edit(
    store: &mut SessionStore,
    command: GraphEditCommand,
) -> Result<SessionDocument, GraphEditError> {
    store.mutate_current(|session| match command {
        GraphEditCommand::AddNode { node } => add_node(session, node),
        GraphEditCommand::RemoveNode { node_id } => remove_node(session, &node_id),
        GraphEditCommand::SetNodeEnabled { node_id, enabled } => {
            set_node_enabled(session, &node_id, enabled)
        }
        GraphEditCommand::SetParameterValue {
            node_id,
            parameter_id,
            value,
        } => set_parameter_value(session, &node_id, &parameter_id, value),
        GraphEditCommand::AddRoute { route } => add_route(session, route),
        GraphEditCommand::RemoveRoute { route_id } => remove_route(session, &route_id),
        GraphEditCommand::AssignNodeToBus { node_id, bus_id } => {
            assign_node_to_bus(session, &node_id, &bus_id)
        }
        GraphEditCommand::ClearNodeBusAssignment { node_id } => {
            clear_node_bus_assignment(session, &node_id)
        }
    })
}

fn add_node(session: &mut SessionDocument, node: Node) -> Result<(), GraphEditError> {
    if session.nodes.iter().any(|existing| existing.id == node.id) {
        return Err(GraphEditError::DuplicateNode {
            node_id: node.id.clone(),
        });
    }

    session.nodes.push(node);
    session.nodes.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(())
}

fn remove_node(session: &mut SessionDocument, node_id: &str) -> Result<(), GraphEditError> {
    let original_len = session.nodes.len();
    session.nodes.retain(|node| node.id != node_id);
    if session.nodes.len() == original_len {
        return Err(GraphEditError::MissingNode {
            node_id: node_id.to_string(),
        });
    }

    session
        .routes
        .retain(|route| route.source_node_id != node_id && route.target_node_id != node_id);
    Ok(())
}

fn set_node_enabled(
    session: &mut SessionDocument,
    node_id: &str,
    enabled: bool,
) -> Result<(), GraphEditError> {
    let node = session
        .nodes
        .iter_mut()
        .find(|node| node.id == node_id)
        .ok_or_else(|| GraphEditError::MissingNode {
            node_id: node_id.to_string(),
        })?;
    node.enabled = enabled;
    Ok(())
}

fn set_parameter_value(
    session: &mut SessionDocument,
    node_id: &str,
    parameter_id: &str,
    value: f64,
) -> Result<(), GraphEditError> {
    let node = session
        .nodes
        .iter_mut()
        .find(|node| node.id == node_id)
        .ok_or_else(|| GraphEditError::MissingNode {
            node_id: node_id.to_string(),
        })?;
    let parameter = node
        .parameters
        .iter_mut()
        .find(|parameter| parameter.id == parameter_id)
        .ok_or_else(|| GraphEditError::MissingParameter {
            node_id: node_id.to_string(),
            parameter_id: parameter_id.to_string(),
        })?;

    validate_parameter_range(node_id, parameter_id, parameter, value)?;
    parameter.value = value;
    Ok(())
}

fn add_route(session: &mut SessionDocument, route: Route) -> Result<(), GraphEditError> {
    if session
        .routes
        .iter()
        .any(|existing| existing.id == route.id)
    {
        return Err(GraphEditError::DuplicateRoute {
            route_id: route.id.clone(),
        });
    }

    validate_route(session, &route)?;
    session.routes.push(route);
    session.routes.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(())
}

fn remove_route(session: &mut SessionDocument, route_id: &str) -> Result<(), GraphEditError> {
    let original_len = session.routes.len();
    session.routes.retain(|route| route.id != route_id);
    if session.routes.len() == original_len {
        return Err(GraphEditError::MissingRoute {
            route_id: route_id.to_string(),
        });
    }

    Ok(())
}

fn assign_node_to_bus(
    session: &mut SessionDocument,
    node_id: &str,
    bus_id: &str,
) -> Result<(), GraphEditError> {
    if !session.buses.iter().any(|bus| bus.id == bus_id) {
        return Err(GraphEditError::MissingBus {
            bus_id: bus_id.to_string(),
        });
    }

    let node = session
        .nodes
        .iter_mut()
        .find(|node| node.id == node_id)
        .ok_or_else(|| GraphEditError::MissingNode {
            node_id: node_id.to_string(),
        })?;
    let primitive =
        node.audio_primitive
            .as_mut()
            .ok_or_else(|| GraphEditError::MissingAudioPrimitive {
                node_id: node_id.to_string(),
            })?;
    set_bus_target(primitive, Some(bus_id.to_string()));
    Ok(())
}

fn clear_node_bus_assignment(
    session: &mut SessionDocument,
    node_id: &str,
) -> Result<(), GraphEditError> {
    let node = session
        .nodes
        .iter_mut()
        .find(|node| node.id == node_id)
        .ok_or_else(|| GraphEditError::MissingNode {
            node_id: node_id.to_string(),
        })?;
    let primitive =
        node.audio_primitive
            .as_mut()
            .ok_or_else(|| GraphEditError::MissingAudioPrimitive {
                node_id: node_id.to_string(),
            })?;
    set_bus_target(primitive, None);
    Ok(())
}

pub fn validate_route(session: &SessionDocument, route: &Route) -> Result<(), GraphEditError> {
    let source = session
        .nodes
        .iter()
        .find(|node| node.id == route.source_node_id)
        .ok_or_else(|| GraphEditError::MissingNode {
            node_id: route.source_node_id.clone(),
        })?;
    let target = session
        .nodes
        .iter()
        .find(|node| node.id == route.target_node_id)
        .ok_or_else(|| GraphEditError::MissingNode {
            node_id: route.target_node_id.clone(),
        })?;
    let source_port = source
        .ports
        .iter()
        .find(|port| port.id == route.source_port_id)
        .ok_or_else(|| GraphEditError::MissingPort {
            node_id: source.id.clone(),
            port_id: route.source_port_id.clone(),
        })?;
    let target_port = target
        .ports
        .iter()
        .find(|port| port.id == route.target_port_id)
        .ok_or_else(|| GraphEditError::MissingPort {
            node_id: target.id.clone(),
            port_id: route.target_port_id.clone(),
        })?;

    if source_port.direction != PortDirection::Output
        || target_port.direction != PortDirection::Input
    {
        return Err(GraphEditError::InvalidRouteDirection {
            route_id: route.id.clone(),
        });
    }

    if source_port.signal_type != target_port.signal_type {
        return Err(GraphEditError::InvalidSignalType {
            route_id: route.id.clone(),
        });
    }

    if route.source_node_id == route.target_node_id
        || path_exists(session, &route.target_node_id, &route.source_node_id)
    {
        return Err(GraphEditError::UnsupportedCycle {
            route_id: route.id.clone(),
        });
    }

    Ok(())
}

fn validate_parameter_range(
    node_id: &str,
    parameter_id: &str,
    parameter: &ParameterValue,
    value: f64,
) -> Result<(), GraphEditError> {
    if value < parameter.min_value || value > parameter.max_value {
        return Err(GraphEditError::ParameterOutOfRange {
            node_id: node_id.to_string(),
            parameter_id: parameter_id.to_string(),
            min: parameter.min_value,
            max: parameter.max_value,
        });
    }

    Ok(())
}

fn set_bus_target(primitive: &mut AudioPrimitive, bus_target_id: Option<String>) {
    match primitive {
        AudioPrimitive::Source(node) => node.bus_target_id = bus_target_id,
        AudioPrimitive::Effect(node) => node.bus_target_id = bus_target_id,
        AudioPrimitive::Mixer(node) => node.bus_target_id = bus_target_id,
        AudioPrimitive::Output(node) => node.bus_target_id = bus_target_id,
    }
}

fn path_exists(session: &SessionDocument, start_node_id: &str, target_node_id: &str) -> bool {
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for route in &session.routes {
        adjacency
            .entry(route.source_node_id.as_str())
            .or_default()
            .push(route.target_node_id.as_str());
    }

    let mut stack = vec![start_node_id];
    let mut visited: HashSet<&str> = HashSet::new();
    while let Some(node_id) = stack.pop() {
        if node_id == target_node_id {
            return true;
        }
        if !visited.insert(node_id) {
            continue;
        }
        if let Some(next_nodes) = adjacency.get(node_id) {
            stack.extend(next_nodes.iter().copied());
        }
    }

    false
}
