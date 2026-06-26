use std::collections::{BTreeMap, BTreeSet};

use crate::audio::compiler::CompiledTopology;
use crate::catalog::{find_catalog_entry, NodeCatalogEntry, NodeCategory};
use crate::domain::session::{OutputKind, SignalType};

/// First audio bus index handed out to session buses (0/1 are the hardware outs).
pub const HARDWARE_AUDIO_BUS_OFFSET: u32 = 2;
/// Control buses live in a clearly separated high range so they can never
/// collide with audio bus indices (Pitfall #2). scsynth's bus index space is
/// shared; the rate is chosen by `In.ar`/`In.kr`, so disjoint ranges prevent a
/// control write from clobbering an audio signal (or vice versa).
pub const FIRST_CONTROL_BUS_OFFSET: u32 = 1024;
pub const FIRST_GROUP_ID: i32 = 1000;
pub const FIRST_SYNTH_ID: i32 = 2000;

#[derive(Clone, Debug, PartialEq)]
pub struct SynthDefResource {
    pub name: &'static str,
    pub relative_path: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScResourcePlan {
    pub patch_id: String,
    pub synthdefs: Vec<SynthDefResource>,
    pub groups: Vec<ScGroupPlan>,
    pub synths: Vec<ScSynthPlan>,
    pub controls: Vec<ScControlPlan>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScGroupPlan {
    pub group_key: String,
    pub node_id: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScSynthPlan {
    pub node_key: String,
    pub node_id: i32,
    pub synthdef_name: &'static str,
    pub group_key: String,
    pub group_node_id: i32,
    pub args: Vec<ScSynthArg>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScSynthArg {
    pub name: String,
    pub value: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScControlPlan {
    pub control_key: String,
    pub synth_node_id: i32,
    pub parameter_name: String,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ScResourcePlanError {
    #[error("duplicate bus id `{bus_id}`")]
    DuplicateBus { bus_id: String },
    #[error("bus `{bus_id}` must have a positive channel count")]
    InvalidBusChannels { bus_id: String },
    #[error("duplicate group id `{group_id}`")]
    DuplicateGroup { group_id: String },
    #[error("node `{node_id}` references unknown group `{group_id}`")]
    UnknownGroup { node_id: String, group_id: String },
    #[error("node `{node_id}` references unknown output bus `{bus_id}`")]
    UnknownOutputBus { node_id: String, bus_id: String },
    #[error("node `{node_id}` references unknown input bus `{bus_id}`")]
    UnknownInputBus { node_id: String, bus_id: String },
    #[error("node `{node_id}` requires an output bus")]
    MissingOutputBus { node_id: String },
    #[error("node `{node_id}` requires an input bus")]
    MissingInputBus { node_id: String },
    #[error("node `{node_id}` mixer supports at most 8 input buses, got {input_count}")]
    TooManyMixerInputs { node_id: String, input_count: usize },
    #[error("node references unknown catalog entry `{node_type_id}`")]
    UnknownCatalogEntry {
        node_type_id: String,
    },
    #[error(
        "node `{node_id}` parameter `{parameter_id}` uses unsupported parameter `{parameter_name}`"
    )]
    UnsupportedParameter {
        node_id: String,
        parameter_id: String,
        parameter_name: String,
    },
}

/// Map from a mod-source output `(node_id, port_id)` to its allocated control
/// bus index. One bus per source output port is shared by every outgoing CV
/// route (a mod source writes once; many targets may read). Compiler artifact —
/// control buses are NEVER canonical `Bus` records (RESEARCH.md anti-pattern).
type CvBusMap = BTreeMap<(String, String), u32>;

pub fn plan_sc_resources(
    topology: &CompiledTopology,
) -> Result<ScResourcePlan, ScResourcePlanError> {
    let bus_map = plan_buses(topology)?;
    let group_map = plan_groups(topology)?;
    let node_out_bus = build_node_output_bus_map(topology, &bus_map);
    let node_cv_args = plan_cv_buses(topology, &node_out_bus)?;

    let mut synthdefs: BTreeSet<&'static str> = BTreeSet::new();
    let mut synths = Vec::new();
    let mut controls = Vec::new();
    let mut next_synth_id = FIRST_SYNTH_ID;

    for node in &topology.node_launch_order {
        // THE catalog lookup — replaces v1's `match &node.node_kind` dispatch.
        // An unknown `node_type_id` becomes a real `Err`, never a panic
        // (success criterion #3).
        let entry = find_catalog_entry(&node.node_type_id)?;
        let group_node_id = *group_map.get(&node.group_id).ok_or_else(|| {
            ScResourcePlanError::UnknownGroup {
                node_id: node.node_id.clone(),
                group_id: node.group_id.clone(),
            }
        })?;
        let synth_node_id = next_synth_id;
        let mut args = Vec::new();

        match entry.category {
            NodeCategory::Source => {
                let out_bus =
                    required_output_bus(&node.node_id, node.output_bus_id.as_deref(), &bus_map)?;
                args.push(arg("out_bus", out_bus));
                apply_parameters(
                    &node.node_id,
                    entry,
                    &node.parameters,
                    &mut args,
                    &mut controls,
                    synth_node_id,
                )?;
            }
            NodeCategory::Effect => {
                let in_bus = first_input_bus(&node.node_id, &node.input_bus_ids, &bus_map)?;
                let out_bus =
                    required_output_bus(&node.node_id, node.output_bus_id.as_deref(), &bus_map)?;
                args.push(arg("in_bus", in_bus));
                args.push(arg("out_bus", out_bus));
                args.push(arg("bypassed", if node.bypassed { 1.0 } else { 0.0 }));
                apply_parameters(
                    &node.node_id,
                    entry,
                    &node.parameters,
                    &mut args,
                    &mut controls,
                    synth_node_id,
                )?;
            }
            NodeCategory::Mixer => {
                if node.input_bus_ids.len() > 8 {
                    return Err(ScResourcePlanError::TooManyMixerInputs {
                        node_id: node.node_id.clone(),
                        input_count: node.input_bus_ids.len(),
                    });
                }
                let out_bus =
                    required_output_bus(&node.node_id, node.output_bus_id.as_deref(), &bus_map)?;
                args.push(arg("out_bus", out_bus));
                args.push(arg("input_count", node.input_bus_ids.len() as f32));
                for input_index in 0..8 {
                    let value = match node.input_bus_ids.get(input_index) {
                        Some(bus_id) => lookup_input_bus(&node.node_id, bus_id, &bus_map)?,
                        None => -1.0,
                    };
                    args.push(arg(&format!("in_bus_{}", input_index + 1), value));
                }
                apply_parameters(
                    &node.node_id,
                    entry,
                    &node.parameters,
                    &mut args,
                    &mut controls,
                    synth_node_id,
                )?;
            }
            NodeCategory::Output => {
                let in_bus = node
                    .input_bus_ids
                    .first()
                    .map(|bus_id| lookup_input_bus(&node.node_id, bus_id, &bus_map))
                    .transpose()?
                    .or_else(|| {
                        node.output_bus_id
                            .as_deref()
                            .and_then(|bus_id| bus_map.get(bus_id).copied())
                    })
                    .ok_or_else(|| ScResourcePlanError::MissingInputBus {
                        node_id: node.node_id.clone(),
                    })?;
                args.push(arg("in_bus", in_bus));
                args.push(arg(
                    "hardware_out",
                    match node.output_kind {
                        Some(OutputKind::Cue) => node.channel_count as f32,
                        _ => 0.0,
                    },
                ));
                args.push(arg("channels", node.channel_count as f32));
                apply_parameters(
                    &node.node_id,
                    entry,
                    &node.parameters,
                    &mut args,
                    &mut controls,
                    synth_node_id,
                )?;
            }
            // Modulators (envelope/lfo) are control-rate sources with no audio
            // bus args; their `out_cv_bus` is attached below from `node_cv_args`.
            NodeCategory::Modulator | NodeCategory::Sequencer => {
                apply_parameters(
                    &node.node_id,
                    entry,
                    &node.parameters,
                    &mut args,
                    &mut controls,
                    synth_node_id,
                )?;
            }
            // Utility nodes split by signal rate: VCA is audio-rate (in/out bus
            // args like an effect, no bypass); Quantizer is control-rate (no
            // audio bus args — its in_cv_bus/out_cv_bus come from node_cv_args).
            NodeCategory::Utility => {
                let has_audio_input = entry
                    .ports
                    .iter()
                    .any(|port| port.direction == crate::domain::session::PortDirection::Input
                        && port.signal_type == SignalType::Audio);
                if has_audio_input {
                    let in_bus = first_input_bus(&node.node_id, &node.input_bus_ids, &bus_map)?;
                    let out_bus =
                        required_output_bus(&node.node_id, node.output_bus_id.as_deref(), &bus_map)?;
                    args.push(arg("in_bus", in_bus));
                    args.push(arg("out_bus", out_bus));
                }
                apply_parameters(
                    &node.node_id,
                    entry,
                    &node.parameters,
                    &mut args,
                    &mut controls,
                    synth_node_id,
                )?;
            }
        }

        // Attach CV-bus args (out_cv_bus on mod sources; <param>_cv_bus on targets).
        args.extend(node_cv_args.get(&node.node_id).cloned().unwrap_or_default());

        // App-driven nodes (empty synthdef_name, e.g. the step sequencer) launch
        // no SuperCollider synth — skip synthdef/synth allocation for them.
        if entry.synthdef_name.is_empty() {
            continue;
        }

        let synthdef_name = entry.synthdef_name;
        synthdefs.insert(synthdef_name);
        next_synth_id += 1;
        synths.push(ScSynthPlan {
            node_key: node.node_id.clone(),
            node_id: synth_node_id,
            synthdef_name,
            group_key: node.group_id.clone(),
            group_node_id,
            args,
        });
    }

    Ok(ScResourcePlan {
        patch_id: format!("patch-v2-{}", topology_fingerprint(topology)),
        synthdefs: synthdefs
            .into_iter()
            .map(|name| synthdef_resource_for_name(name))
            .collect(),
        groups: topology
            .group_order
            .iter()
            .map(|group| ScGroupPlan {
                group_key: group.group_id.clone(),
                node_id: group_map[&group.group_id],
            })
            .collect(),
        synths,
        controls,
    })
}

fn plan_buses(topology: &CompiledTopology) -> Result<BTreeMap<String, f32>, ScResourcePlanError> {
    let mut bus_map = BTreeMap::new();
    for bus in &topology.buses {
        if bus.channels == 0 {
            return Err(ScResourcePlanError::InvalidBusChannels {
                bus_id: bus.bus_id.clone(),
            });
        }
        if bus_map
            .insert(
                bus.bus_id.clone(),
                (HARDWARE_AUDIO_BUS_OFFSET + bus.index) as f32,
            )
            .is_some()
        {
            return Err(ScResourcePlanError::DuplicateBus {
                bus_id: bus.bus_id.clone(),
            });
        }
    }
    Ok(bus_map)
}

fn plan_groups(topology: &CompiledTopology) -> Result<BTreeMap<String, i32>, ScResourcePlanError> {
    let mut group_map = BTreeMap::new();
    for (index, group) in topology.group_order.iter().enumerate() {
        if group_map
            .insert(group.group_id.clone(), FIRST_GROUP_ID + index as i32)
            .is_some()
        {
            return Err(ScResourcePlanError::DuplicateGroup {
                group_id: group.group_id.clone(),
            });
        }
    }
    Ok(group_map)
}

/// Resolve each node's audio output bus index (for audio-rate CV reuse + bus args).
fn build_node_output_bus_map(
    topology: &CompiledTopology,
    bus_map: &BTreeMap<String, f32>,
) -> BTreeMap<String, f32> {
    let mut out = BTreeMap::new();
    for node in &topology.node_launch_order {
        if let Some(bus_id) = node.output_bus_id.as_deref() {
            if let Some(idx) = bus_map.get(bus_id) {
                out.insert(node.node_id.clone(), *idx);
            }
        }
    }
    out
}

/// Allocate control buses for CV routes and build the per-node CV-bus args.
///
/// - Control-rate CV (LFO/Env/Sequencer → param): allocate ONE control bus per
///   mod-source output port (shared by all its outgoing routes); the source
///   synth gets `out_cv_bus`/`<port>_bus` and writes `Out.kr`; each target synth
///   gets `<cv_port>_bus` and reads `In.kr`.
/// - Audio-rate CV (oscillator FM): reuse the source's existing audio out bus;
///   the target gets `<cv_port>_bus` = source out bus index (no new allocation).
fn plan_cv_buses(
    topology: &CompiledTopology,
    node_out_bus: &BTreeMap<String, f32>,
) -> Result<BTreeMap<String, Vec<ScSynthArg>>, ScResourcePlanError> {
    let mut cv_args: BTreeMap<String, Vec<ScSynthArg>> = BTreeMap::new();
    let mut control_buses: CvBusMap = BTreeMap::new();
    let mut next_control_bus = FIRST_CONTROL_BUS_OFFSET;

    for route in &topology.cv_routes {
        match route.signal_type {
            SignalType::Control => {
                // One control bus per mod-source output port (shared by readers).
                let key = (route.source_node_id.clone(), route.source_port_id.clone());
                let bus_index = *control_buses.entry(key).or_insert_with(|| {
                    let allocated = next_control_bus;
                    next_control_bus += 1;
                    allocated
                }) as f32;
                // Mod-source writes its signal to the bus.
                cv_args
                    .entry(route.source_node_id.clone())
                    .or_default()
                    .push(arg("out_cv_bus", bus_index));
                // Target reads the bus into its modulated parameter.
                cv_args
                    .entry(route.target_node_id.clone())
                    .or_default()
                    .push(arg(&format!("{}_bus", route.target_port_id), bus_index));
            }
            SignalType::Audio => {
                // Audio-rate CV (FM): the source already writes its audio out
                // bus; the target reads that bus via In.ar.
                if let Some(source_out) = node_out_bus.get(&route.source_node_id) {
                    cv_args
                        .entry(route.target_node_id.clone())
                        .or_default()
                        .push(arg(&format!("{}_bus", route.target_port_id), *source_out));
                }
            }
        }
    }

    Ok(cv_args)
}

fn required_output_bus(
    node_id: &str,
    bus_id: Option<&str>,
    bus_map: &BTreeMap<String, f32>,
) -> Result<f32, ScResourcePlanError> {
    let bus_id = bus_id.ok_or_else(|| ScResourcePlanError::MissingOutputBus {
        node_id: node_id.to_string(),
    })?;
    bus_map.get(bus_id).copied().ok_or_else(|| {
        ScResourcePlanError::UnknownOutputBus {
            node_id: node_id.to_string(),
            bus_id: bus_id.to_string(),
        }
    })
}

fn first_input_bus(
    node_id: &str,
    bus_ids: &[String],
    bus_map: &BTreeMap<String, f32>,
) -> Result<f32, ScResourcePlanError> {
    let bus_id = bus_ids.first().ok_or_else(|| ScResourcePlanError::MissingInputBus {
        node_id: node_id.to_string(),
    })?;
    lookup_input_bus(node_id, bus_id, bus_map)
}

fn lookup_input_bus(
    node_id: &str,
    bus_id: &str,
    bus_map: &BTreeMap<String, f32>,
) -> Result<f32, ScResourcePlanError> {
    bus_map.get(bus_id).copied().ok_or_else(|| {
        ScResourcePlanError::UnknownInputBus {
            node_id: node_id.to_string(),
            bus_id: bus_id.to_string(),
        }
    })
}

/// Bind each parameter to its SuperCollider synth arg via the catalog entry
/// (replaces v1's `normalize_parameter_name` allowlist). The base value is
/// still live-`/n_set`-able; CV modulation is additive inside the synth graph.
fn apply_parameters(
    node_id: &str,
    entry: &NodeCatalogEntry,
    parameters: &[crate::audio::compiler::CompiledParameter],
    args: &mut Vec<ScSynthArg>,
    controls: &mut Vec<ScControlPlan>,
    synth_node_id: i32,
) -> Result<(), ScResourcePlanError> {
    for parameter in parameters {
        let Some(sc_arg) = entry.resolve_sc_arg(&parameter.name) else {
            return Err(ScResourcePlanError::UnsupportedParameter {
                node_id: node_id.to_string(),
                parameter_id: parameter.parameter_id.clone(),
                parameter_name: parameter.name.clone(),
            });
        };
        args.push(arg(sc_arg, parameter.value as f32));
        controls.push(ScControlPlan {
            control_key: format!("{node_id}:{}", parameter.parameter_id),
            synth_node_id,
            parameter_name: sc_arg.to_string(),
        });
    }
    Ok(())
}

fn arg(name: &str, value: f32) -> ScSynthArg {
    ScSynthArg {
        name: name.to_string(),
        value,
    }
}

/// Resolve a SynthDef name (drawn from a catalog entry) to its resource path.
/// Infallible: `name` always originates from a resolved catalog entry, so this
/// never panics on user input (the v1 `unreachable!()` is gone — success
/// criterion #3; unknown node ids error earlier via `find_catalog_entry`).
fn synthdef_resource_for_name(name: &str) -> SynthDefResource {
    let entry = crate::catalog::CATALOG
        .iter()
        .find(|entry| entry.synthdef_name == name)
        .expect("synthdef name originates from a catalog entry");
    SynthDefResource {
        name: entry.synthdef_name,
        relative_path: entry.synthdef_resource,
    }
}

fn topology_fingerprint(topology: &CompiledTopology) -> String {
    let mut hash = FNV_OFFSET_BASIS;

    hash_bytes(&mut hash, b"buses");
    for bus in &topology.buses {
        hash_str(&mut hash, &bus.bus_id);
        hash_u32(&mut hash, bus.index);
        hash_u32(&mut hash, bus.channels);
        hash_str(&mut hash, &format!("{:?}", bus.bus_type));
    }

    hash_bytes(&mut hash, b"groups");
    for group in &topology.group_order {
        hash_str(&mut hash, &group.group_id);
        for node_id in &group.node_ids {
            hash_str(&mut hash, node_id);
        }
    }

    hash_bytes(&mut hash, b"nodes");
    for node in &topology.node_launch_order {
        hash_str(&mut hash, &node.node_id);
        hash_str(&mut hash, &node.node_type_id);
        hash_str(&mut hash, &node.runtime_target);
        hash_str(&mut hash, &node.group_id);
        for bus_id in &node.input_bus_ids {
            hash_str(&mut hash, bus_id);
        }
        if let Some(bus_id) = &node.output_bus_id {
            hash_str(&mut hash, bus_id);
        }
        hash_u8(&mut hash, u8::from(node.bypassed));
        hash_str(&mut hash, &format!("{:?}", node.output_kind));
        hash_u32(&mut hash, node.channel_count);
        for parameter in &node.parameters {
            hash_str(&mut hash, &parameter.parameter_id);
            hash_str(&mut hash, &parameter.name);
            hash_u64(&mut hash, parameter.value.to_bits());
        }
    }

    hash_bytes(&mut hash, b"cv_routes");
    for route in &topology.cv_routes {
        hash_str(&mut hash, &route.route_id);
        hash_str(&mut hash, &route.source_node_id);
        hash_str(&mut hash, &route.source_port_id);
        hash_str(&mut hash, &route.target_node_id);
        hash_str(&mut hash, &route.target_port_id);
        hash_str(&mut hash, &format!("{:?}", route.signal_type));
    }

    format!("{hash:016x}")
}

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001b3;

fn hash_str(hash: &mut u64, value: &str) {
    hash_u64(hash, value.len() as u64);
    hash_bytes(hash, value.as_bytes());
}

fn hash_u8(hash: &mut u64, value: u8) {
    hash_bytes(hash, &[value]);
}

fn hash_u32(hash: &mut u64, value: u32) {
    hash_bytes(hash, &value.to_le_bytes());
}

fn hash_u64(hash: &mut u64, value: u64) {
    hash_bytes(hash, &value.to_le_bytes());
}

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}
