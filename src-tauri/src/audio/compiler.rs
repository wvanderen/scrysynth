use std::collections::{BTreeMap, BTreeSet, VecDeque};

use thiserror::Error;

use crate::domain::session::{
    AudioBusType, AudioEffectNode, AudioMixerNode, AudioOutputNode, AudioPrimitive,
    AudioSourceNode, Node, NodeType, Route, SessionDocument,
};

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledTopology {
    pub buses: Vec<CompiledBus>,
    pub group_order: Vec<CompiledGroup>,
    pub node_launch_order: Vec<CompiledNodeLaunch>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledBus {
    pub bus_id: String,
    pub index: u32,
    pub channels: u32,
    pub bus_type: AudioBusType,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledGroup {
    pub group_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledNodeLaunch {
    pub node_id: String,
    pub runtime_target: String,
    pub node_kind: CompiledNodeKind,
    pub group_id: String,
    pub input_bus_ids: Vec<String>,
    pub output_bus_id: Option<String>,
    pub parameters: Vec<CompiledParameter>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledParameter {
    pub parameter_id: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CompiledNodeKind {
    Source,
    Effect { bypassed: bool },
    Mixer,
    Output { channels: u32 },
}

#[derive(Debug, Error, PartialEq)]
pub enum TopologyCompileError {
    #[error("node `{node_id}` is missing an audio primitive")]
    MissingAudioPrimitive { node_id: String },
    #[error("node `{node_id}` is missing a runtime target")]
    MissingRuntimeTarget { node_id: String },
    #[error("node `{node_id}` references unknown bus `{bus_id}`")]
    UnknownBusReference { node_id: String, bus_id: String },
    #[error("route `{route_id}` references unknown bus `{bus_id}`")]
    UnknownRouteBus { route_id: String, bus_id: String },
    #[error("route `{route_id}` references unknown node `{node_id}`")]
    UnknownNodeReference { route_id: String, node_id: String },
    #[error("route `{route_id}` references unknown port `{port_id}` on node `{node_id}`")]
    UnknownPortReference {
        route_id: String,
        node_id: String,
        port_id: String,
    },
    #[error("route `{route_id}` connects disabled node `{node_id}`")]
    DisabledNodeRoute { route_id: String, node_id: String },
    #[error("canonical audio graph contains a cycle or unresolved dependency")]
    CyclicGraph,
}

pub fn compile_session_to_topology(
    session: &SessionDocument,
) -> Result<CompiledTopology, TopologyCompileError> {
    let buses = compile_buses(session);
    let bus_lookup: BTreeSet<&str> = buses.iter().map(|bus| bus.bus_id.as_str()).collect();

    let nodes = enabled_audio_nodes(session)?;
    validate_node_bus_references(&nodes, &bus_lookup)?;
    validate_routes(session, &nodes, &bus_lookup)?;

    let ordered_node_ids = topo_sort_nodes(session, &nodes)?;
    let group_id = "group-main-signal".to_string();
    let launches = ordered_node_ids
        .iter()
        .map(|node_id| {
            let node = nodes.get(node_id).expect("node exists in topology");
            compile_node_launch(node, session, &group_id)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CompiledTopology {
        buses,
        group_order: vec![CompiledGroup {
            group_id: group_id.clone(),
            node_ids: ordered_node_ids,
        }],
        node_launch_order: launches,
    })
}

fn compile_buses(session: &SessionDocument) -> Vec<CompiledBus> {
    let mut buses: Vec<_> = session.buses.iter().filter(|bus| bus.is_enabled).collect();
    buses.sort_by(|left, right| bus_sort_key(left).cmp(&bus_sort_key(right)));

    let mut next_index = 0;
    buses
        .into_iter()
        .map(|bus| {
            let compiled = CompiledBus {
                bus_id: bus.id.clone(),
                index: next_index,
                channels: bus.channels,
                bus_type: bus.bus_type.clone(),
            };
            next_index += bus.channels;
            compiled
        })
        .collect()
}

fn enabled_audio_nodes(
    session: &SessionDocument,
) -> Result<BTreeMap<String, Node>, TopologyCompileError> {
    let mut nodes = BTreeMap::new();

    for node in &session.nodes {
        if !node.enabled {
            continue;
        }

        if node.audio_primitive.is_none() {
            return Err(TopologyCompileError::MissingAudioPrimitive {
                node_id: node.id.clone(),
            });
        }

        if node
            .runtime_target
            .as_deref()
            .unwrap_or_default()
            .is_empty()
        {
            return Err(TopologyCompileError::MissingRuntimeTarget {
                node_id: node.id.clone(),
            });
        }

        nodes.insert(node.id.clone(), node.clone());
    }

    Ok(nodes)
}

fn validate_node_bus_references(
    nodes: &BTreeMap<String, Node>,
    bus_lookup: &BTreeSet<&str>,
) -> Result<(), TopologyCompileError> {
    for node in nodes.values() {
        if let Some(bus_id) = node_bus_target_id(node) {
            if !bus_lookup.contains(bus_id) {
                return Err(TopologyCompileError::UnknownBusReference {
                    node_id: node.id.clone(),
                    bus_id: bus_id.to_string(),
                });
            }
        }
    }

    Ok(())
}

fn validate_routes(
    session: &SessionDocument,
    nodes: &BTreeMap<String, Node>,
    bus_lookup: &BTreeSet<&str>,
) -> Result<(), TopologyCompileError> {
    for route in &session.routes {
        if let Some(bus_id) = route.bus_id.as_deref() {
            if !bus_lookup.contains(bus_id) {
                return Err(TopologyCompileError::UnknownRouteBus {
                    route_id: route.id.clone(),
                    bus_id: bus_id.to_string(),
                });
            }
        }

        let source = nodes
            .get(&route.source_node_id)
            .ok_or_else(|| missing_node_error(route, route.source_node_id.clone(), session))?;
        let target = nodes
            .get(&route.target_node_id)
            .ok_or_else(|| missing_node_error(route, route.target_node_id.clone(), session))?;

        validate_port_exists(route, source, &route.source_port_id)?;
        validate_port_exists(route, target, &route.target_port_id)?;
    }

    Ok(())
}

fn missing_node_error(
    route: &Route,
    node_id: String,
    session: &SessionDocument,
) -> TopologyCompileError {
    if session
        .nodes
        .iter()
        .any(|node| node.id == node_id && !node.enabled)
    {
        TopologyCompileError::DisabledNodeRoute {
            route_id: route.id.clone(),
            node_id,
        }
    } else {
        TopologyCompileError::UnknownNodeReference {
            route_id: route.id.clone(),
            node_id,
        }
    }
}

fn validate_port_exists(
    route: &Route,
    node: &Node,
    port_id: &str,
) -> Result<(), TopologyCompileError> {
    if node.ports.iter().any(|port| port.id == port_id) {
        Ok(())
    } else {
        Err(TopologyCompileError::UnknownPortReference {
            route_id: route.id.clone(),
            node_id: node.id.clone(),
            port_id: port_id.to_string(),
        })
    }
}

fn topo_sort_nodes(
    session: &SessionDocument,
    nodes: &BTreeMap<String, Node>,
) -> Result<Vec<String>, TopologyCompileError> {
    let mut indegree: BTreeMap<String, usize> = nodes.keys().map(|id| (id.clone(), 0)).collect();
    let mut adjacency: BTreeMap<String, Vec<String>> =
        nodes.keys().map(|id| (id.clone(), Vec::new())).collect();

    let mut routes: Vec<_> = session
        .routes
        .iter()
        .filter(|route| {
            nodes.contains_key(&route.source_node_id) && nodes.contains_key(&route.target_node_id)
        })
        .collect();
    routes.sort_by(|left, right| left.id.cmp(&right.id));

    for route in routes {
        adjacency
            .get_mut(&route.source_node_id)
            .expect("source in adjacency")
            .push(route.target_node_id.clone());
        *indegree
            .get_mut(&route.target_node_id)
            .expect("target in indegree") += 1;
    }

    for targets in adjacency.values_mut() {
        targets.sort();
    }

    let mut ready: Vec<_> = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(node_id, _)| node_id.clone())
        .collect();
    ready.sort_by(|left, right| node_sort_key(&nodes[left]).cmp(&node_sort_key(&nodes[right])));
    let mut queue: VecDeque<String> = ready.into();
    let mut ordered = Vec::with_capacity(nodes.len());

    while let Some(node_id) = queue.pop_front() {
        ordered.push(node_id.clone());
        for target_id in adjacency.get(&node_id).into_iter().flatten() {
            let degree = indegree.get_mut(target_id).expect("target indegree exists");
            *degree -= 1;
            if *degree == 0 {
                queue.push_back(target_id.clone());
                let mut sorted: Vec<_> = queue.drain(..).collect();
                sorted.sort_by(|left, right| {
                    node_sort_key(&nodes[left]).cmp(&node_sort_key(&nodes[right]))
                });
                queue = sorted.into();
            }
        }
    }

    if ordered.len() != nodes.len() {
        return Err(TopologyCompileError::CyclicGraph);
    }

    Ok(ordered)
}

fn compile_node_launch(
    node: &Node,
    session: &SessionDocument,
    group_id: &str,
) -> Result<CompiledNodeLaunch, TopologyCompileError> {
    let primitive = node.audio_primitive.as_ref().ok_or_else(|| {
        TopologyCompileError::MissingAudioPrimitive {
            node_id: node.id.clone(),
        }
    })?;

    let node_kind = match primitive {
        AudioPrimitive::Source(AudioSourceNode { .. }) => CompiledNodeKind::Source,
        AudioPrimitive::Effect(AudioEffectNode { bypassed, .. }) => CompiledNodeKind::Effect {
            bypassed: *bypassed,
        },
        AudioPrimitive::Mixer(AudioMixerNode { .. }) => CompiledNodeKind::Mixer,
        AudioPrimitive::Output(AudioOutputNode { channels, .. }) => CompiledNodeKind::Output {
            channels: *channels,
        },
    };

    let input_bus_ids = incoming_bus_ids(session, &node.id);
    let output_bus_id = node_bus_target_id(node).map(ToString::to_string);

    Ok(CompiledNodeLaunch {
        node_id: node.id.clone(),
        runtime_target: node.runtime_target.clone().unwrap_or_default(),
        node_kind,
        group_id: group_id.to_string(),
        input_bus_ids,
        output_bus_id,
        parameters: node
            .parameters
            .iter()
            .map(|parameter| CompiledParameter {
                parameter_id: parameter.id.clone(),
                value: parameter.value,
            })
            .collect(),
    })
}

fn incoming_bus_ids(session: &SessionDocument, node_id: &str) -> Vec<String> {
    let mut bus_ids: Vec<_> = session
        .routes
        .iter()
        .filter(|route| route.target_node_id == node_id)
        .filter_map(|route| route.bus_id.clone())
        .collect();
    bus_ids.sort();
    bus_ids.dedup();
    bus_ids
}

fn node_bus_target_id(node: &Node) -> Option<&str> {
    match node.audio_primitive.as_ref() {
        Some(AudioPrimitive::Source(primitive)) => primitive.bus_target_id.as_deref(),
        Some(AudioPrimitive::Effect(primitive)) => primitive.bus_target_id.as_deref(),
        Some(AudioPrimitive::Mixer(primitive)) => primitive.bus_target_id.as_deref(),
        Some(AudioPrimitive::Output(primitive)) => primitive.bus_target_id.as_deref(),
        None => None,
    }
}

fn bus_sort_key(bus: &crate::domain::session::Bus) -> (u8, &str) {
    let kind_rank = match bus.bus_type {
        AudioBusType::Main => 0,
        AudioBusType::Cue => 1,
        AudioBusType::Auxiliary => 2,
    };
    (kind_rank, bus.id.as_str())
}

fn node_sort_key(node: &Node) -> (u8, &str) {
    let kind_rank = match node.node_type {
        NodeType::Source => 0,
        NodeType::Effect => 1,
        NodeType::Mixer => 2,
        NodeType::Output => 3,
    };
    (kind_rank, node.id.as_str())
}
