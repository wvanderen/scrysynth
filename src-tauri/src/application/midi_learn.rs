use std::sync::mpsc::Receiver;

use crate::domain::session::{
    new_id, BindingTarget, HardwareBinding, HardwareSource, SessionDocument, ValueTransform,
};
use crate::hardware::midi_input::MidiLearnEvent;
use crate::hardware::osc_input::OscLearnEvent;

#[derive(Clone, Debug, PartialEq)]
pub enum HardwareLearnState {
    Idle,
    Learning {
        target: BindingTarget,
    },
    Captured {
        source: HardwareSource,
        target: BindingTarget,
    },
}

pub struct HardwareInputRouter {
    pub learn_state: HardwareLearnState,
    pub midi_rx: Option<Receiver<MidiLearnEvent>>,
    pub osc_rx: Option<Receiver<OscLearnEvent>>,
}

impl std::fmt::Debug for HardwareInputRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HardwareInputRouter")
            .field("learn_state", &self.learn_state)
            .field(
                "midi_rx",
                &self.midi_rx.as_ref().map(|_| "Receiver<MidiLearnEvent>"),
            )
            .field(
                "osc_rx",
                &self.osc_rx.as_ref().map(|_| "Receiver<OscLearnEvent>"),
            )
            .finish()
    }
}

impl HardwareInputRouter {
    pub fn new() -> Self {
        Self {
            learn_state: HardwareLearnState::Idle,
            midi_rx: None,
            osc_rx: None,
        }
    }

    pub fn start_learn(&mut self, target: BindingTarget) {
        self.learn_state = HardwareLearnState::Learning { target };
    }

    pub fn stop_learn(&mut self) {
        self.learn_state = HardwareLearnState::Idle;
    }

    pub fn attach_midi_receiver(&mut self, rx: Receiver<MidiLearnEvent>) {
        self.midi_rx = Some(rx);
    }

    pub fn detach_midi_receiver(&mut self) {
        self.midi_rx = None;
    }

    pub fn attach_osc_receiver(&mut self, rx: Receiver<OscLearnEvent>) {
        self.osc_rx = Some(rx);
    }

    pub fn detach_osc_receiver(&mut self) {
        self.osc_rx = None;
    }

    pub fn poll_and_route(&mut self, session: &mut SessionDocument) -> Option<HardwareBinding> {
        let midi_event = self.midi_rx.as_ref().and_then(|rx| rx.try_recv().ok());

        let osc_event = if midi_event.is_none() {
            self.osc_rx.as_ref().and_then(|rx| rx.try_recv().ok())
        } else {
            None
        };

        if let Some(ref event) = midi_event {
            match &self.learn_state {
                HardwareLearnState::Learning { target } => {
                    let source = midi_event_to_source(event);
                    let transform = default_midi_transform();
                    let binding = HardwareBinding {
                        id: new_id(),
                        source,
                        target: target.clone(),
                        transform,
                    };
                    let captured_source = binding.source.clone();
                    let captured_target = binding.target.clone();
                    session.hardware_bindings.push(binding.clone());
                    self.learn_state = HardwareLearnState::Captured {
                        source: captured_source,
                        target: captured_target,
                    };
                    return Some(binding);
                }
                HardwareLearnState::Idle => {
                    let source = midi_event_to_source(event);
                    route_live_event(session, &source, event_value_from_midi(event));
                }
                HardwareLearnState::Captured { .. } => {}
            }
        }

        if let Some(ref event) = osc_event {
            match &self.learn_state {
                HardwareLearnState::Learning { target } => {
                    let source = HardwareSource::OscAddress {
                        address: event.address.clone(),
                    };
                    let transform = default_osc_transform();
                    let binding = HardwareBinding {
                        id: new_id(),
                        source,
                        target: target.clone(),
                        transform,
                    };
                    let captured_source = binding.source.clone();
                    let captured_target = binding.target.clone();
                    session.hardware_bindings.push(binding.clone());
                    self.learn_state = HardwareLearnState::Captured {
                        source: captured_source,
                        target: captured_target,
                    };
                    return Some(binding);
                }
                HardwareLearnState::Idle => {
                    let osc_value = event
                        .args
                        .first()
                        .and_then(|a| match a {
                            rosc::OscType::Float(f) => Some(*f as f64),
                            rosc::OscType::Int(i) => Some(*i as f64),
                            rosc::OscType::Double(d) => Some(*d),
                            _ => None,
                        })
                        .unwrap_or(0.0);
                    let source = HardwareSource::OscAddress {
                        address: event.address.clone(),
                    };
                    route_live_event(session, &source, osc_value);
                }
                HardwareLearnState::Captured { .. } => {}
            }
        }

        None
    }
}

impl Default for HardwareInputRouter {
    fn default() -> Self {
        Self::new()
    }
}

fn midi_event_to_source(event: &MidiLearnEvent) -> HardwareSource {
    match event {
        MidiLearnEvent::MidiCc {
            channel,
            controller,
            ..
        } => HardwareSource::MidiCc {
            channel: *channel,
            controller: *controller,
        },
        MidiLearnEvent::MidiNote { channel, note, .. } => HardwareSource::MidiNote {
            channel: *channel,
            note: *note,
        },
        MidiLearnEvent::MidiPitchBend { channel, .. } => {
            HardwareSource::MidiPitchBend { channel: *channel }
        }
    }
}

fn event_value_from_midi(event: &MidiLearnEvent) -> f64 {
    match event {
        MidiLearnEvent::MidiCc { value, .. } => *value as f64,
        MidiLearnEvent::MidiNote { velocity, .. } => *velocity as f64,
        MidiLearnEvent::MidiPitchBend { value, .. } => *value as f64,
    }
}

fn default_midi_transform() -> ValueTransform {
    ValueTransform {
        input_min: 0.0,
        input_max: 127.0,
        output_min: 0.0,
        output_max: 1.0,
    }
}

fn default_osc_transform() -> ValueTransform {
    ValueTransform {
        input_min: 0.0,
        input_max: 1.0,
        output_min: 0.0,
        output_max: 1.0,
    }
}

fn route_live_event(session: &mut SessionDocument, source: &HardwareSource, raw_value: f64) {
    let matching_bindings: Vec<(usize, HardwareBinding)> = session
        .hardware_bindings
        .iter()
        .enumerate()
        .filter(|(_, b)| &b.source == source)
        .map(|(i, b)| (i, b.clone()))
        .collect();

    for (_, binding) in matching_bindings {
        let scaled = scale_value(raw_value, &binding.transform);
        apply_binding_target(session, &binding.target, scaled);
    }
}

fn scale_value(raw: f64, transform: &ValueTransform) -> f64 {
    if (transform.input_max - transform.input_min).abs() < f64::EPSILON {
        return transform.output_min;
    }
    let normalized = (raw - transform.input_min) / (transform.input_max - transform.input_min);
    let clamped = normalized.clamp(0.0, 1.0);
    transform.output_min + clamped * (transform.output_max - transform.output_min)
}

fn apply_binding_target(session: &mut SessionDocument, target: &BindingTarget, value: f64) {
    match target {
        BindingTarget::Macro { macro_id } => {
            let macro_id_owned = macro_id.clone();
            let info = session
                .macros
                .iter()
                .find(|m| m.id == macro_id_owned)
                .map(|m| (m.range_start, m.range_end, m.targets.clone()));
            if let Some((range_start, range_end, targets)) = info {
                let scaled = range_start + (value * (range_end - range_start));
                for t in &targets {
                    if let crate::domain::session::MacroTarget::AudioParameter {
                        node_id,
                        parameter_id,
                    } = t
                    {
                        let _ = crate::application::macro_command::apply_audio_parameter(
                            session,
                            node_id,
                            parameter_id,
                            scaled,
                        );
                    }
                }
            }
        }
        BindingTarget::SceneRecall { scene_id } => {
            let active_ids = session
                .scenes
                .iter()
                .find(|s| s.id == *scene_id)
                .map(|s| s.active_node_ids.clone());
            if let Some(ids) = active_ids {
                for node in &mut session.nodes {
                    node.enabled = ids.contains(&node.id);
                }
            }
        }
        BindingTarget::TransportPlay => {
            session.transport.is_playing = true;
        }
        BindingTarget::TransportStop => {
            session.transport.is_playing = false;
        }
        BindingTarget::TransportPanic => {
            session.transport.is_playing = false;
        }
    }
}

pub fn scale_value_exposed(raw: f64, transform: &ValueTransform) -> f64 {
    scale_value(raw, transform)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::session::{
        AudioBusType, AudioPrimitive, AudioSourceNode, AudioSourceType, Bus, ChannelMode,
        ControllerKind, MacroTarget, Node, NodeType, OwnershipAssignment, ParameterValue, Port,
        PortDirection, SceneDefinition, SignalType,
    };

    fn test_session() -> SessionDocument {
        SessionDocument {
            title: "Hardware Test".to_string(),
            nodes: vec![Node {
                id: "node-src".to_string(),
                node_type: NodeType::Source,
                ports: vec![Port {
                    id: "port-out".to_string(),
                    name: "out".to_string(),
                    direction: PortDirection::Output,
                    signal_type: SignalType::Audio,
                }],
                parameters: vec![ParameterValue {
                    id: "param-gain".to_string(),
                    name: "gain".to_string(),
                    value: 0.5,
                    default_value: 0.5,
                    min_value: 0.0,
                    max_value: 1.0,
                    unit: "linear".to_string(),
                }],
                runtime_target: None,
                scene_membership: vec![],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::Shared,
                    is_locked: false,
                },
                enabled: true,
                audio_primitive: Some(AudioPrimitive::Source(AudioSourceNode {
                    source_type: AudioSourceType::Oscillator,
                    channel_mode: ChannelMode::Mono,
                    bus_target_id: None,
                })),
            }],
            routes: vec![],
            buses: vec![Bus {
                id: "bus-1".to_string(),
                name: "main".to_string(),
                channels: 2,
                bus_type: AudioBusType::Main,
                is_enabled: true,
            }],
            macros: vec![crate::domain::session::MacroDefinition {
                id: "macro-energy".to_string(),
                name: "energy".to_string(),
                target_parameter_ids: vec![],
                range_start: 0.0,
                range_end: 1.0,
                targets: vec![MacroTarget::AudioParameter {
                    node_id: "node-src".to_string(),
                    parameter_id: "param-gain".to_string(),
                }],
            }],
            scenes: vec![SceneDefinition {
                id: "scene-a".to_string(),
                name: "intro".to_string(),
                active_node_ids: vec!["node-src".to_string()],
                macro_overrides: vec![],
            }],
            ..SessionDocument::default()
        }
    }

    #[test]
    fn learn_state_transitions_idle_to_learning() {
        let mut router = HardwareInputRouter::new();
        router.start_learn(BindingTarget::Macro {
            macro_id: "m1".to_string(),
        });
        assert!(matches!(
            &router.learn_state,
            HardwareLearnState::Learning { .. }
        ));
    }

    #[test]
    fn learn_state_stop_returns_to_idle() {
        let mut router = HardwareInputRouter::new();
        router.start_learn(BindingTarget::Macro {
            macro_id: "m1".to_string(),
        });
        router.stop_learn();
        assert_eq!(router.learn_state, HardwareLearnState::Idle);
    }

    #[test]
    fn learn_captures_midi_event_and_creates_binding() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut router = HardwareInputRouter::new();
        router.midi_rx = Some(rx);
        router.start_learn(BindingTarget::Macro {
            macro_id: "macro-energy".to_string(),
        });

        tx.send(MidiLearnEvent::MidiCc {
            channel: 1,
            controller: 7,
            value: 100,
        })
        .unwrap();

        let mut session = test_session();
        let binding = router.poll_and_route(&mut session);

        assert!(binding.is_some());
        assert_eq!(session.hardware_bindings.len(), 1);
        let b = &session.hardware_bindings[0];
        assert_eq!(
            b.source,
            HardwareSource::MidiCc {
                channel: 1,
                controller: 7,
            }
        );
        assert!(matches!(
            &router.learn_state,
            HardwareLearnState::Captured { .. }
        ));
    }

    #[test]
    fn live_routing_applies_binding_to_macro() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut router = HardwareInputRouter::new();
        router.midi_rx = Some(rx);

        let mut session = test_session();
        session.hardware_bindings.push(HardwareBinding {
            id: "hb-1".to_string(),
            source: HardwareSource::MidiCc {
                channel: 0,
                controller: 7,
            },
            target: BindingTarget::Macro {
                macro_id: "macro-energy".to_string(),
            },
            transform: ValueTransform {
                input_min: 0.0,
                input_max: 127.0,
                output_min: 0.0,
                output_max: 1.0,
            },
        });

        tx.send(MidiLearnEvent::MidiCc {
            channel: 0,
            controller: 7,
            value: 63,
        })
        .unwrap();

        router.poll_and_route(&mut session);

        let gain = session.nodes[0]
            .parameters
            .iter()
            .find(|p| p.id == "param-gain")
            .unwrap();
        let expected = 63.0 / 127.0;
        assert!((gain.value - expected).abs() < 0.01);
    }

    #[test]
    fn value_transform_scaling() {
        let transform = ValueTransform {
            input_min: 0.0,
            input_max: 127.0,
            output_min: 0.0,
            output_max: 1.0,
        };
        let result = scale_value_exposed(0.0, &transform);
        assert!((result - 0.0).abs() < f64::EPSILON);

        let result = scale_value_exposed(127.0, &transform);
        assert!((result - 1.0).abs() < f64::EPSILON);

        let result = scale_value_exposed(63.0, &transform);
        assert!((result - 0.496).abs() < 0.01);
    }
}
