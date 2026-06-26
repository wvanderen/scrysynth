//! The compiled-in `CATALOG` contents — the single source of truth for node
//! identity, SynthDef mapping, ports, parameters, and visual shape.
//!
//! Granularity follows CONTEXT.md D-01 (param-driven default), D-02 (split only
//! where DSP fundamentally differs), D-04/D-05 (per-parameter CV ports for
//! continuous params only), and the CV-port lists in 12-RESEARCH.md
//! §"New SynthDefs Needed". Adding a node is a data edit here — never a new
//! `match` arm elsewhere.

#![allow(clippy::needless_pass_by_value)]

use super::{CatalogParamSpec, CatalogPortSpec, NodeCatalogEntry, NodeCategory};
use crate::domain::session::{PortDirection, SignalType};

/// Reusable "mod-source output" control port name suffixes are not needed —
/// every CV/output port is declared explicitly per entry below for legibility.

/// Helper: standard audio input port (`audio_in`, stereo, audio-rate).
const AUDIO_IN: CatalogPortSpec = CatalogPortSpec {
    id: "audio_in",
    name: "Audio In",
    direction: PortDirection::Input,
    signal_type: SignalType::Audio,
};

/// Helper: standard audio output port (`audio_out`, stereo, audio-rate).
const AUDIO_OUT: CatalogPortSpec = CatalogPortSpec {
    id: "audio_out",
    name: "Audio Out",
    direction: PortDirection::Output,
    signal_type: SignalType::Audio,
};

/// `level` parameter (linear gain) with a sibling `level_cv` control port.
const fn level_param(cv: bool) -> CatalogParamSpec {
    CatalogParamSpec {
        id: "level",
        sc_arg: "level",
        aliases: &["gain", "amplitude"],
        default_value: 1.0,
        min_value: 0.0,
        max_value: 1.0,
        unit: "linear",
        exposes_cv_port: cv,
        cv_port_id: if cv { Some("level_cv") } else { None },
    }
}

const fn level_cv_port() -> CatalogPortSpec {
    CatalogPortSpec {
        id: "level_cv",
        name: "Level CV",
        direction: PortDirection::Input,
        signal_type: SignalType::Control,
    }
}

const fn mix_param(cv: bool) -> CatalogParamSpec {
    CatalogParamSpec {
        id: "mix",
        sc_arg: "mix",
        aliases: &["wet", "wet_dry"],
        default_value: 1.0,
        min_value: 0.0,
        max_value: 1.0,
        unit: "ratio",
        exposes_cv_port: cv,
        cv_port_id: if cv { Some("mix_cv") } else { None },
    }
}

const fn bypassed_param() -> CatalogParamSpec {
    CatalogParamSpec {
        id: "bypassed",
        sc_arg: "bypassed",
        aliases: &["bypass"],
        default_value: 0.0,
        min_value: 0.0,
        max_value: 1.0,
        unit: "toggle",
        exposes_cv_port: false,
        cv_port_id: None,
    }
}

/// The compiled-in node catalog. Order is stable and human-readable
/// (synthesis chain → effects → utility → sequencer → output).
#[rustfmt::skip]
pub const CATALOG: &[NodeCatalogEntry] = &[
    // ---- Sources (NODES-02) ----
    NodeCatalogEntry {
        id: "oscillator",
        display_name: "Oscillator",
        category: NodeCategory::Source,
        synthdef_name: "scrysynth_v2_oscillator",
        synthdef_resource: "resources/synthdefs/v2/scrysynth_v2_oscillator.scsyndef",
        ports: &[
            CatalogPortSpec { id: "main_out", name: "Main Out", direction: PortDirection::Output, signal_type: SignalType::Audio },
            // D-03 audio-rate CV representative (FM path).
            CatalogPortSpec { id: "frequency_cv", name: "Frequency FM", direction: PortDirection::Input, signal_type: SignalType::Audio },
            CatalogPortSpec { id: "level_cv", name: "Level CV", direction: PortDirection::Input, signal_type: SignalType::Control },
        ],
        parameters: &[
            CatalogParamSpec { id: "frequency", sc_arg: "frequency", aliases: &["freq"], default_value: 220.0, min_value: 20.0, max_value: 20_000.0, unit: "hz", exposes_cv_port: true, cv_port_id: Some("frequency_cv") },
            CatalogParamSpec { id: "wave_shape", sc_arg: "wave_shape", aliases: &["waveShape", "waveform"], default_value: 0.0, min_value: 0.0, max_value: 3.0, unit: "selector", exposes_cv_port: false, cv_port_id: None },
            level_param(true),
        ],
        visual_shape: "sphere",
    },
    NodeCatalogEntry {
        id: "noise",
        display_name: "Noise",
        category: NodeCategory::Source,
        synthdef_name: "scrysynth_v2_noise",
        synthdef_resource: "resources/synthdefs/v2/scrysynth_v2_noise.scsyndef",
        ports: &[
            CatalogPortSpec { id: "main_out", name: "Main Out", direction: PortDirection::Output, signal_type: SignalType::Audio },
            level_cv_port(),
        ],
        parameters: &[
            CatalogParamSpec { id: "noise_color", sc_arg: "noise_color", aliases: &["noiseColor", "color"], default_value: 0.0, min_value: 0.0, max_value: 1.0, unit: "selector", exposes_cv_port: false, cv_port_id: None },
            level_param(true),
        ],
        visual_shape: "sphere",
    },

    // ---- Modulators (NODES-02) ----
    NodeCatalogEntry {
        id: "envelope",
        display_name: "Envelope",
        category: NodeCategory::Modulator,
        synthdef_name: "scrysynth_v2_envelope",
        synthdef_resource: "resources/synthdefs/v2/scrysynth_v2_envelope.scsyndef",
        ports: &[
            CatalogPortSpec { id: "env_out", name: "Envelope Out", direction: PortDirection::Output, signal_type: SignalType::Control },
            // `gate` is a trigger input (driven by the sequencer / performer), not a CV port for a param.
            CatalogPortSpec { id: "gate", name: "Gate", direction: PortDirection::Input, signal_type: SignalType::Control },
        ],
        parameters: &[
            CatalogParamSpec { id: "attack", sc_arg: "attack", aliases: &["att"], default_value: 0.01, min_value: 0.0, max_value: 10.0, unit: "s", exposes_cv_port: false, cv_port_id: None },
            CatalogParamSpec { id: "decay", sc_arg: "decay", aliases: &["dec"], default_value: 0.2, min_value: 0.0, max_value: 10.0, unit: "s", exposes_cv_port: false, cv_port_id: None },
            CatalogParamSpec { id: "sustain", sc_arg: "sustain", aliases: &["sus"], default_value: 0.8, min_value: 0.0, max_value: 1.0, unit: "ratio", exposes_cv_port: false, cv_port_id: None },
            CatalogParamSpec { id: "release", sc_arg: "release", aliases: &["rel"], default_value: 0.3, min_value: 0.0, max_value: 10.0, unit: "s", exposes_cv_port: false, cv_port_id: None },
        ],
        visual_shape: "ring",
    },
    NodeCatalogEntry {
        id: "lfo",
        display_name: "LFO",
        category: NodeCategory::Modulator,
        synthdef_name: "scrysynth_v2_lfo",
        synthdef_resource: "resources/synthdefs/v2/scrysynth_v2_lfo.scsyndef",
        ports: &[
            CatalogPortSpec { id: "lfo_out", name: "LFO Out", direction: PortDirection::Output, signal_type: SignalType::Control },
            CatalogPortSpec { id: "frequency_cv", name: "Frequency CV", direction: PortDirection::Input, signal_type: SignalType::Control },
        ],
        parameters: &[
            CatalogParamSpec { id: "frequency", sc_arg: "frequency", aliases: &["freq", "rate"], default_value: 0.5, min_value: 0.001, max_value: 100.0, unit: "hz", exposes_cv_port: true, cv_port_id: Some("frequency_cv") },
            CatalogParamSpec { id: "wave_shape", sc_arg: "wave_shape", aliases: &["waveShape", "waveform"], default_value: 0.0, min_value: 0.0, max_value: 3.0, unit: "selector", exposes_cv_port: false, cv_port_id: None },
            level_param(false),
        ],
        visual_shape: "ring",
    },

    // ---- Utility (NODES-02) ----
    NodeCatalogEntry {
        id: "vca",
        display_name: "VCA",
        category: NodeCategory::Utility,
        synthdef_name: "scrysynth_v2_vca",
        synthdef_resource: "resources/synthdefs/v2/scrysynth_v2_vca.scsyndef",
        ports: &[AUDIO_IN, AUDIO_OUT, level_cv_port()],
        parameters: &[level_param(true)],
        visual_shape: "box",
    },
    NodeCatalogEntry {
        id: "quantizer",
        display_name: "Quantizer",
        category: NodeCategory::Utility,
        synthdef_name: "scrysynth_v2_quantizer",
        synthdef_resource: "resources/synthdefs/v2/scrysynth_v2_quantizer.scsyndef",
        ports: &[
            CatalogPortSpec { id: "cv_in", name: "CV In", direction: PortDirection::Input, signal_type: SignalType::Control },
            CatalogPortSpec { id: "cv_out", name: "CV Out", direction: PortDirection::Output, signal_type: SignalType::Control },
        ],
        parameters: &[
            CatalogParamSpec { id: "steps", sc_arg: "steps", aliases: &["divisions"], default_value: 12.0, min_value: 1.0, max_value: 48.0, unit: "count", exposes_cv_port: false, cv_port_id: None },
        ],
        visual_shape: "box",
    },
    NodeCatalogEntry {
        id: "mixer",
        display_name: "Mixer",
        category: NodeCategory::Mixer,
        synthdef_name: "scrysynth_v2_mixer",
        synthdef_resource: "resources/synthdefs/v2/scrysynth_v2_mixer.scsyndef",
        ports: &[
            AUDIO_OUT,
            level_cv_port(),
        ],
        parameters: &[
            level_param(true),
            CatalogParamSpec { id: "mix", sc_arg: "mix", aliases: &[], default_value: 1.0, min_value: 0.0, max_value: 1.0, unit: "ratio", exposes_cv_port: false, cv_port_id: None },
        ],
        visual_shape: "ring",
    },

    // ---- Effects (NODES-03) ----
    NodeCatalogEntry {
        id: "filter",
        display_name: "Filter",
        category: NodeCategory::Effect,
        synthdef_name: "scrysynth_v2_filter",
        synthdef_resource: "resources/synthdefs/v2/scrysynth_v2_filter.scsyndef",
        ports: &[
            AUDIO_IN, AUDIO_OUT,
            CatalogPortSpec { id: "cutoff_cv", name: "Cutoff CV", direction: PortDirection::Input, signal_type: SignalType::Control },
            CatalogPortSpec { id: "resonance_cv", name: "Resonance CV", direction: PortDirection::Input, signal_type: SignalType::Control },
        ],
        parameters: &[
            CatalogParamSpec { id: "cutoff", sc_arg: "cutoff_hz", aliases: &["cutoff", "cutoff_hz", "cutoffHz"], default_value: 1200.0, min_value: 20.0, max_value: 20_000.0, unit: "hz", exposes_cv_port: true, cv_port_id: Some("cutoff_cv") },
            CatalogParamSpec { id: "resonance", sc_arg: "resonance", aliases: &["rq"], default_value: 0.5, min_value: 0.05, max_value: 1.0, unit: "ratio", exposes_cv_port: true, cv_port_id: Some("resonance_cv") },
            CatalogParamSpec { id: "filter_mode", sc_arg: "filter_mode", aliases: &["mode"], default_value: 0.0, min_value: 0.0, max_value: 3.0, unit: "selector", exposes_cv_port: false, cv_port_id: None },
            mix_param(false),
            bypassed_param(),
        ],
        visual_shape: "box",
    },
    NodeCatalogEntry {
        id: "delay",
        display_name: "Delay",
        category: NodeCategory::Effect,
        synthdef_name: "scrysynth_v2_delay",
        synthdef_resource: "resources/synthdefs/v2/scrysynth_v2_delay.scsyndef",
        ports: &[
            AUDIO_IN, AUDIO_OUT,
            CatalogPortSpec { id: "delay_time_cv", name: "Delay Time CV", direction: PortDirection::Input, signal_type: SignalType::Control },
            CatalogPortSpec { id: "feedback_cv", name: "Feedback CV", direction: PortDirection::Input, signal_type: SignalType::Control },
        ],
        parameters: &[
            CatalogParamSpec { id: "delay_time", sc_arg: "delay_time_s", aliases: &["delay_time", "delayTime", "delay_time_s"], default_value: 0.25, min_value: 0.0, max_value: 2.0, unit: "s", exposes_cv_port: true, cv_port_id: Some("delay_time_cv") },
            CatalogParamSpec { id: "feedback", sc_arg: "feedback", aliases: &[], default_value: 0.25, min_value: 0.0, max_value: 0.95, unit: "ratio", exposes_cv_port: true, cv_port_id: Some("feedback_cv") },
            mix_param(false),
            bypassed_param(),
        ],
        visual_shape: "box",
    },
    NodeCatalogEntry {
        id: "reverb",
        display_name: "Reverb",
        category: NodeCategory::Effect,
        synthdef_name: "scrysynth_v2_reverb",
        synthdef_resource: "resources/synthdefs/v2/scrysynth_v2_reverb.scsyndef",
        ports: &[
            AUDIO_IN, AUDIO_OUT,
            CatalogPortSpec { id: "room_cv", name: "Room CV", direction: PortDirection::Input, signal_type: SignalType::Control },
            CatalogPortSpec { id: "mix_cv", name: "Mix CV", direction: PortDirection::Input, signal_type: SignalType::Control },
        ],
        parameters: &[
            CatalogParamSpec { id: "room", sc_arg: "room", aliases: &["room_size"], default_value: 0.5, min_value: 0.0, max_value: 1.0, unit: "ratio", exposes_cv_port: true, cv_port_id: Some("room_cv") },
            CatalogParamSpec { id: "damp", sc_arg: "damp", aliases: &["damping"], default_value: 0.5, min_value: 0.0, max_value: 1.0, unit: "ratio", exposes_cv_port: false, cv_port_id: None },
            mix_param(true),
            bypassed_param(),
        ],
        visual_shape: "box",
    },
    NodeCatalogEntry {
        id: "distortion",
        display_name: "Distortion",
        category: NodeCategory::Effect,
        synthdef_name: "scrysynth_v2_distortion",
        synthdef_resource: "resources/synthdefs/v2/scrysynth_v2_distortion.scsyndef",
        ports: &[
            AUDIO_IN, AUDIO_OUT,
            CatalogPortSpec { id: "drive_cv", name: "Drive CV", direction: PortDirection::Input, signal_type: SignalType::Control },
            CatalogPortSpec { id: "mix_cv", name: "Mix CV", direction: PortDirection::Input, signal_type: SignalType::Control },
        ],
        parameters: &[
            CatalogParamSpec { id: "drive", sc_arg: "drive", aliases: &["gain"], default_value: 0.5, min_value: 0.0, max_value: 1.0, unit: "ratio", exposes_cv_port: true, cv_port_id: Some("drive_cv") },
            mix_param(true),
            bypassed_param(),
        ],
        visual_shape: "box",
    },
    NodeCatalogEntry {
        id: "chorus",
        display_name: "Chorus",
        category: NodeCategory::Effect,
        synthdef_name: "scrysynth_v2_chorus",
        synthdef_resource: "resources/synthdefs/v2/scrysynth_v2_chorus.scsyndef",
        ports: &[
            AUDIO_IN, AUDIO_OUT,
            CatalogPortSpec { id: "depth_cv", name: "Depth CV", direction: PortDirection::Input, signal_type: SignalType::Control },
            CatalogPortSpec { id: "rate_cv", name: "Rate CV", direction: PortDirection::Input, signal_type: SignalType::Control },
        ],
        parameters: &[
            CatalogParamSpec { id: "depth", sc_arg: "depth", aliases: &[], default_value: 0.3, min_value: 0.0, max_value: 1.0, unit: "ratio", exposes_cv_port: true, cv_port_id: Some("depth_cv") },
            CatalogParamSpec { id: "rate", sc_arg: "rate", aliases: &["frequency", "freq"], default_value: 0.5, min_value: 0.0, max_value: 10.0, unit: "hz", exposes_cv_port: true, cv_port_id: Some("rate_cv") },
            mix_param(false),
            bypassed_param(),
        ],
        visual_shape: "box",
    },
    NodeCatalogEntry {
        id: "flanger",
        display_name: "Flanger",
        category: NodeCategory::Effect,
        synthdef_name: "scrysynth_v2_flanger",
        synthdef_resource: "resources/synthdefs/v2/scrysynth_v2_flanger.scsyndef",
        ports: &[
            AUDIO_IN, AUDIO_OUT,
            CatalogPortSpec { id: "depth_cv", name: "Depth CV", direction: PortDirection::Input, signal_type: SignalType::Control },
            CatalogPortSpec { id: "rate_cv", name: "Rate CV", direction: PortDirection::Input, signal_type: SignalType::Control },
            CatalogPortSpec { id: "feedback_cv", name: "Feedback CV", direction: PortDirection::Input, signal_type: SignalType::Control },
        ],
        parameters: &[
            CatalogParamSpec { id: "depth", sc_arg: "depth", aliases: &[], default_value: 0.3, min_value: 0.0, max_value: 1.0, unit: "ratio", exposes_cv_port: true, cv_port_id: Some("depth_cv") },
            CatalogParamSpec { id: "rate", sc_arg: "rate", aliases: &["frequency", "freq"], default_value: 0.5, min_value: 0.0, max_value: 10.0, unit: "hz", exposes_cv_port: true, cv_port_id: Some("rate_cv") },
            CatalogParamSpec { id: "feedback", sc_arg: "feedback", aliases: &[], default_value: 0.3, min_value: 0.0, max_value: 0.95, unit: "ratio", exposes_cv_port: true, cv_port_id: Some("feedback_cv") },
            mix_param(false),
            bypassed_param(),
        ],
        visual_shape: "box",
    },

    // ---- Sequencer (NODES-04; D-06 app-driven, no SynthDef) ----
    NodeCatalogEntry {
        id: "step_sequencer",
        display_name: "Step Sequencer",
        category: NodeCategory::Sequencer,
        synthdef_name: "",
        synthdef_resource: "",
        ports: &[
            CatalogPortSpec { id: "gate_out", name: "Gate Out", direction: PortDirection::Output, signal_type: SignalType::Control },
            CatalogPortSpec { id: "cv_out", name: "CV Out", direction: PortDirection::Output, signal_type: SignalType::Control },
        ],
        parameters: &[],
        visual_shape: "ring",
    },

    // ---- Output ----
    NodeCatalogEntry {
        id: "output",
        display_name: "Output",
        category: NodeCategory::Output,
        synthdef_name: "scrysynth_v2_output",
        synthdef_resource: "resources/synthdefs/v2/scrysynth_v2_output.scsyndef",
        ports: &[
            AUDIO_IN,
            level_cv_port(),
        ],
        parameters: &[
            level_param(true),
        ],
        visual_shape: "plane",
    },
];
