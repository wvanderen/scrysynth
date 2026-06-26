use std::collections::{BTreeMap, BTreeSet, VecDeque};

use thiserror::Error;

use crate::catalog::find_catalog_entry;
use crate::domain::session::{AudioBusType, Node, OutputKind, Route, SessionDocument, SignalType};

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledTopology {
    pub buses: Vec<CompiledBus>,
    pub group_order: Vec<CompiledGroup>,
    pub node_launch_order: Vec<CompiledNodeLaunch>,
    /// Direct port-to-port modulation routes (no audio `bus_id`). These carry
    /// CV/control-rate modulation and audio-rate FM; the SC resource planner
    /// allocates buses for them separately from the audio signal chain.
    pub cv_routes: Vec<CompiledCvRoute>,
}

/// A modulation route compiled for CV-bus allocation (NODES-05).
#[derive(Clone, Debug, PartialEq)]
pub struct CompiledCvRoute {
    pub route_id: String,
    pub source_node_id: String,
    pub source_port_id: String,
    pub target_node_id: String,
    pub target_port_id: String,
    pub signal_type: SignalType,
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

/// A node's catalog identity plus the per-node config the catalog does not own
/// (bypass / output routing / channel count). Replaces v1's `CompiledNodeKind`
/// closed enum: identity is the catalog `node_type_id`, and category/ports/params
/// come from `find_catalog_entry` at plan time.
#[derive(Clone, Debug, PartialEq)]
pub struct CompiledNodeLaunch {
    pub node_id: String,
    pub node_type_id: String,
    pub runtime_target: String,
    pub bypassed: bool,
    pub output_kind: Option<OutputKind>,
    pub channel_count: u32,
    pub group_id: String,
    pub input_bus_ids: Vec<String>,
    pub output_bus_id: Option<String>,
    pub parameters: Vec<CompiledParameter>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledParameter {
    pub parameter_id: String,
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Error, PartialEq)]
pub enum TopologyCompileError {
    #[error("node `{node_id}` is missing a node_type_id")]
    MissingNodeTypeId { node_id: String },
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
    #[error(
        "route `{route_id}` has a port signal-type mismatch: source `{source_type}` vs target `{target_type}`"
    )]
    PortSignalTypeMismatch {
        route_id: String,
        source_type: String,
        target_type: String,
    },
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
    let cv_routes = compile_cv_routes(session, &nodes);

    Ok(CompiledTopology {
        buses,
        group_order: vec![CompiledGroup {
            group_id: group_id.clone(),
            node_ids: ordered_node_ids,
        }],
        node_launch_order: launches,
        cv_routes,
    })
}

/// Compile direct modulation routes (no audio `bus_id`) into CV routes for the
/// SC resource planner. The route's signal rate is the target CV port's rate
/// (control-rate for LFO/Env/Sequencer → param; audio-rate for oscillator FM).
fn compile_cv_routes(
    session: &SessionDocument,
    nodes: &BTreeMap<String, Node>,
) -> Vec<CompiledCvRoute> {
    session
        .routes
        .iter()
        .filter(|route| route.bus_id.is_none())
        .filter_map(|route| {
            let target = nodes.get(&route.target_node_id)?;
            let target_port = target.ports.iter().find(|p| p.id == route.target_port_id)?;
            Some(CompiledCvRoute {
                route_id: route.id.clone(),
                source_node_id: route.source_node_id.clone(),
                source_port_id: route.source_port_id.clone(),
                target_node_id: route.target_node_id.clone(),
                target_port_id: route.target_port_id.clone(),
                signal_type: target_port.signal_type,
            })
        })
        .collect()
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

        if node.node_type_id.trim().is_empty() {
            return Err(TopologyCompileError::MissingNodeTypeId {
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

        let source_port = validate_port_exists(route, source, &route.source_port_id)?;
        let target_port = validate_port_exists(route, target, &route.target_port_id)?;

        // Pitfall #3: an Audio↔Control CV mismatch is a silent SC failure — reject
        // it at compile time. Same-rate connections (incl. audio-rate FM) are fine.
        if source_port.signal_type != target_port.signal_type {
            return Err(TopologyCompileError::PortSignalTypeMismatch {
                route_id: route.id.clone(),
                source_type: format!("{:?}", source_port.signal_type).to_lowercase(),
                target_type: format!("{:?}", target_port.signal_type).to_lowercase(),
            });
        }
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

fn validate_port_exists<'a>(
    route: &Route,
    node: &'a Node,
    port_id: &str,
) -> Result<&'a crate::domain::session::Port, TopologyCompileError> {
    node.ports
        .iter()
        .find(|port| port.id == port_id)
        .ok_or_else(|| TopologyCompileError::UnknownPortReference {
            route_id: route.id.clone(),
            node_id: node.id.clone(),
            port_id: port_id.to_string(),
        })
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
    let input_bus_ids = incoming_bus_ids(session, &node.id);
    let output_bus_id = node_bus_target_id(node).map(ToString::to_string);

    Ok(CompiledNodeLaunch {
        node_id: node.id.clone(),
        node_type_id: node.node_type_id.clone(),
        runtime_target: node.runtime_target.clone().unwrap_or_default(),
        bypassed: node.bypassed.unwrap_or(false),
        output_kind: node.output_kind,
        channel_count: node.channel_count.unwrap_or(2),
        group_id: group_id.to_string(),
        input_bus_ids,
        output_bus_id,
        parameters: node
            .parameters
            .iter()
            .map(|parameter| CompiledParameter {
                parameter_id: parameter.id.clone(),
                name: parameter.name.clone(),
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
    node.bus_target_id.as_deref()
}

fn bus_sort_key(bus: &crate::domain::session::Bus) -> (u8, &str) {
    let kind_rank = match bus.bus_type {
        AudioBusType::Main => 0,
        AudioBusType::Cue => 1,
        AudioBusType::Auxiliary => 2,
    };
    (kind_rank, bus.id.as_str())
}

/// Deterministic topology-sort rank for a node. Catalog-driven via
/// `find_catalog_entry` — replaces v1's `match node.node_type` dispatch.
/// Unknown `node_type_id`s sort last (rank 255); they fail loudly later in
/// `plan_sc_resources` with `UnknownCatalogEntry` (success criterion #3), so the
/// compiler stays non-fallible here.
fn node_sort_key(node: &Node) -> (u8, &str) {
    let kind_rank = find_catalog_entry(&node.node_type_id)
        .map(|entry| entry.category.rank())
        .unwrap_or(255);
    (kind_rank, node.id.as_str())
}
