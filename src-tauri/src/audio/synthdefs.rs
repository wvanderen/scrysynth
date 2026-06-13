use std::collections::{BTreeMap, BTreeSet};

use crate::audio::compiler::{CompiledNodeKind, CompiledTopology};
use crate::domain::session::{AudioEffectType, AudioOutputType, AudioSourceType};

pub const HARDWARE_AUDIO_BUS_OFFSET: u32 = 2;
pub const FIRST_GROUP_ID: i32 = 1000;
pub const FIRST_SYNTH_ID: i32 = 2000;

pub const SOURCE_OSCILLATOR_SYNTHDEF: &str = "scrysynth_v1_source_oscillator";
pub const SOURCE_NOISE_SYNTHDEF: &str = "scrysynth_v1_source_noise";
pub const FX_LOWPASS_SYNTHDEF: &str = "scrysynth_v1_fx_lowpass";
pub const FX_DELAY_SYNTHDEF: &str = "scrysynth_v1_fx_delay";
pub const MIXER_SYNTHDEF: &str = "scrysynth_v1_mixer";
pub const OUTPUT_SYNTHDEF: &str = "scrysynth_v1_output";

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
    #[error("node `{node_id}` targets unsupported runtime `{runtime_target}`")]
    UnsupportedRuntimeTarget {
        node_id: String,
        runtime_target: String,
    },
    #[error(
        "node `{node_id}` parameter `{parameter_id}` uses unsupported v1 SuperCollider parameter `{parameter_name}`"
    )]
    UnsupportedParameter {
        node_id: String,
        parameter_id: String,
        parameter_name: String,
    },
}

pub fn plan_sc_resources(
    topology: &CompiledTopology,
) -> Result<ScResourcePlan, ScResourcePlanError> {
    let bus_map = plan_buses(topology)?;
    let group_map = plan_groups(topology)?;
    let mut synthdefs = BTreeSet::new();
    let mut synths = Vec::new();
    let mut controls = Vec::new();

    for (index, node) in topology.node_launch_order.iter().enumerate() {
        validate_runtime_target(&node.node_id, &node.runtime_target, &node.node_kind)?;
        let group_node_id =
            *group_map
                .get(&node.group_id)
                .ok_or_else(|| ScResourcePlanError::UnknownGroup {
                    node_id: node.node_id.clone(),
                    group_id: node.group_id.clone(),
                })?;
        let synth_node_id = FIRST_SYNTH_ID + index as i32;
        let mut args = Vec::new();
        let synthdef_name = match &node.node_kind {
            CompiledNodeKind::Source { source_type, .. } => {
                let out_bus =
                    required_output_bus(&node.node_id, node.output_bus_id.as_deref(), &bus_map)?;
                args.push(arg("out_bus", out_bus));
                apply_parameters(
                    &node.node_id,
                    &node.parameters,
                    &mut args,
                    &mut controls,
                    synth_node_id,
                )?;
                match source_type {
                    AudioSourceType::Oscillator => SOURCE_OSCILLATOR_SYNTHDEF,
                    AudioSourceType::Noise => SOURCE_NOISE_SYNTHDEF,
                }
            }
            CompiledNodeKind::Effect {
                effect_type,
                bypassed,
            } => {
                let in_bus = first_input_bus(&node.node_id, &node.input_bus_ids, &bus_map)?;
                let out_bus =
                    required_output_bus(&node.node_id, node.output_bus_id.as_deref(), &bus_map)?;
                args.push(arg("in_bus", in_bus));
                args.push(arg("out_bus", out_bus));
                args.push(arg("bypassed", if *bypassed { 1.0 } else { 0.0 }));
                apply_parameters(
                    &node.node_id,
                    &node.parameters,
                    &mut args,
                    &mut controls,
                    synth_node_id,
                )?;
                match effect_type {
                    AudioEffectType::LowPassFilter => FX_LOWPASS_SYNTHDEF,
                    AudioEffectType::Delay => FX_DELAY_SYNTHDEF,
                }
            }
            CompiledNodeKind::Mixer { .. } => {
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
                    &node.parameters,
                    &mut args,
                    &mut controls,
                    synth_node_id,
                )?;
                MIXER_SYNTHDEF
            }
            CompiledNodeKind::Output {
                output_type,
                channels,
            } => {
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
                    match output_type {
                        AudioOutputType::Master => 0.0,
                        AudioOutputType::Cue => *channels as f32,
                    },
                ));
                args.push(arg("channels", *channels as f32));
                apply_parameters(
                    &node.node_id,
                    &node.parameters,
                    &mut args,
                    &mut controls,
                    synth_node_id,
                )?;
                OUTPUT_SYNTHDEF
            }
        };

        synthdefs.insert(synthdef_name);
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
        patch_id: format!(
            "patch-v1-{}-{}-{}",
            topology.buses.len(),
            topology.group_order.len(),
            topology.node_launch_order.len()
        ),
        synthdefs: synthdefs.into_iter().map(synthdef_resource).collect(),
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

fn validate_runtime_target(
    node_id: &str,
    runtime_target: &str,
    node_kind: &CompiledNodeKind,
) -> Result<(), ScResourcePlanError> {
    let supported = match node_kind {
        CompiledNodeKind::Source {
            source_type: AudioSourceType::Oscillator,
            ..
        } => matches!(
            runtime_target,
            "audio/source/oscillator" | "audio/source/default"
        ),
        CompiledNodeKind::Source {
            source_type: AudioSourceType::Noise,
            ..
        } => matches!(
            runtime_target,
            "audio/source/noise" | "audio/source/default"
        ),
        CompiledNodeKind::Effect {
            effect_type: AudioEffectType::LowPassFilter,
            ..
        } => matches!(
            runtime_target,
            "audio/effect/low_pass_filter" | "audio/effect/filter"
        ),
        CompiledNodeKind::Effect {
            effect_type: AudioEffectType::Delay,
            ..
        } => runtime_target == "audio/effect/delay",
        CompiledNodeKind::Mixer { .. } => {
            matches!(runtime_target, "audio/mixer/stereo" | "audio/mixer/submix")
        }
        CompiledNodeKind::Output {
            output_type: AudioOutputType::Master,
            ..
        } => runtime_target == "audio/output/master",
        CompiledNodeKind::Output {
            output_type: AudioOutputType::Cue,
            ..
        } => runtime_target == "audio/output/cue",
    };

    if supported {
        Ok(())
    } else {
        Err(ScResourcePlanError::UnsupportedRuntimeTarget {
            node_id: node_id.to_string(),
            runtime_target: runtime_target.to_string(),
        })
    }
}

fn required_output_bus(
    node_id: &str,
    bus_id: Option<&str>,
    bus_map: &BTreeMap<String, f32>,
) -> Result<f32, ScResourcePlanError> {
    let bus_id = bus_id.ok_or_else(|| ScResourcePlanError::MissingOutputBus {
        node_id: node_id.to_string(),
    })?;
    bus_map
        .get(bus_id)
        .copied()
        .ok_or_else(|| ScResourcePlanError::UnknownOutputBus {
            node_id: node_id.to_string(),
            bus_id: bus_id.to_string(),
        })
}

fn first_input_bus(
    node_id: &str,
    bus_ids: &[String],
    bus_map: &BTreeMap<String, f32>,
) -> Result<f32, ScResourcePlanError> {
    let bus_id = bus_ids
        .first()
        .ok_or_else(|| ScResourcePlanError::MissingInputBus {
            node_id: node_id.to_string(),
        })?;
    lookup_input_bus(node_id, bus_id, bus_map)
}

fn lookup_input_bus(
    node_id: &str,
    bus_id: &str,
    bus_map: &BTreeMap<String, f32>,
) -> Result<f32, ScResourcePlanError> {
    bus_map
        .get(bus_id)
        .copied()
        .ok_or_else(|| ScResourcePlanError::UnknownInputBus {
            node_id: node_id.to_string(),
            bus_id: bus_id.to_string(),
        })
}

fn apply_parameters(
    node_id: &str,
    parameters: &[crate::audio::compiler::CompiledParameter],
    args: &mut Vec<ScSynthArg>,
    controls: &mut Vec<ScControlPlan>,
    synth_node_id: i32,
) -> Result<(), ScResourcePlanError> {
    for parameter in parameters {
        let Some(name) = normalize_parameter_name(&parameter.name) else {
            return Err(ScResourcePlanError::UnsupportedParameter {
                node_id: node_id.to_string(),
                parameter_id: parameter.parameter_id.clone(),
                parameter_name: parameter.name.clone(),
            });
        };
        args.push(arg(name, parameter.value as f32));
        controls.push(ScControlPlan {
            control_key: format!("{node_id}:{}", parameter.parameter_id),
            synth_node_id,
            parameter_name: name.to_string(),
        });
    }
    Ok(())
}

fn normalize_parameter_name(name: &str) -> Option<&'static str> {
    match name {
        "level" | "gain" | "amplitude" => Some("level"),
        "frequency" | "freq" => Some("frequency"),
        "wave_shape" | "waveShape" => Some("wave_shape"),
        "noise_color" | "noiseColor" => Some("noise_color"),
        "cutoff" | "cutoff_hz" | "cutoffHz" => Some("cutoff_hz"),
        "resonance" | "rq" => Some("resonance"),
        "delay_time" | "delay_time_s" | "delayTime" => Some("delay_time_s"),
        "feedback" => Some("feedback"),
        "mix" => Some("mix"),
        _ => None,
    }
}

fn arg(name: &str, value: f32) -> ScSynthArg {
    ScSynthArg {
        name: name.to_string(),
        value,
    }
}

fn synthdef_resource(name: &str) -> SynthDefResource {
    match name {
        SOURCE_OSCILLATOR_SYNTHDEF => SynthDefResource {
            name: SOURCE_OSCILLATOR_SYNTHDEF,
            relative_path: "resources/synthdefs/v1/scrysynth_v1_source_oscillator.scsyndef",
        },
        SOURCE_NOISE_SYNTHDEF => SynthDefResource {
            name: SOURCE_NOISE_SYNTHDEF,
            relative_path: "resources/synthdefs/v1/scrysynth_v1_source_noise.scsyndef",
        },
        FX_LOWPASS_SYNTHDEF => SynthDefResource {
            name: FX_LOWPASS_SYNTHDEF,
            relative_path: "resources/synthdefs/v1/scrysynth_v1_fx_lowpass.scsyndef",
        },
        FX_DELAY_SYNTHDEF => SynthDefResource {
            name: FX_DELAY_SYNTHDEF,
            relative_path: "resources/synthdefs/v1/scrysynth_v1_fx_delay.scsyndef",
        },
        MIXER_SYNTHDEF => SynthDefResource {
            name: MIXER_SYNTHDEF,
            relative_path: "resources/synthdefs/v1/scrysynth_v1_mixer.scsyndef",
        },
        OUTPUT_SYNTHDEF => SynthDefResource {
            name: OUTPUT_SYNTHDEF,
            relative_path: "resources/synthdefs/v1/scrysynth_v1_output.scsyndef",
        },
        _ => unreachable!("unknown v1 synthdef name"),
    }
}
