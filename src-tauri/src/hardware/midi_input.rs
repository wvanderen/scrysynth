use std::sync::mpsc::{channel, Sender};

#[derive(Clone, Debug, PartialEq)]
pub enum MidiLearnEvent {
    MidiCc {
        channel: u8,
        controller: u8,
        value: u8,
    },
    MidiNote {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    MidiPitchBend {
        channel: u8,
        value: u16,
    },
}

pub struct MidiInputManager {
    connection: Option<midir::MidiInputConnection<()>>,
    event_sender: Sender<MidiLearnEvent>,
}

impl MidiInputManager {
    pub fn new() -> (Self, std::sync::mpsc::Receiver<MidiLearnEvent>) {
        let (tx, rx) = channel();
        (
            Self {
                connection: None,
                event_sender: tx,
            },
            rx,
        )
    }

    pub fn list_devices() -> Result<Vec<String>, String> {
        let midi_in = midir::MidiInput::new("scrysynth-list").map_err(|e| e.to_string())?;
        let ports = midi_in.ports();
        let mut names = Vec::with_capacity(ports.len());
        for port in &ports {
            let name = midi_in.port_name(port).map_err(|e| e.to_string())?;
            names.push(name);
        }
        Ok(names)
    }

    pub fn start_listening(&mut self, port_index: Option<usize>) -> Result<(), String> {
        self.stop_listening();

        let midi_in = midir::MidiInput::new("scrysynth").map_err(|e| e.to_string())?;

        let ports = midi_in.ports();
        if ports.is_empty() {
            return Err("No MIDI input ports available".to_string());
        }

        let idx = port_index.unwrap_or(0);
        if idx >= ports.len() {
            return Err(format!(
                "Port index {} out of range (0-{})",
                idx,
                ports.len() - 1
            ));
        }

        let port = ports[idx].clone();
        let sender = self.event_sender.clone();
        let connection = midi_in
            .connect(
                &port,
                "scrysynth-midi-in",
                move |_stamp: u64, message: &[u8], _data: &mut ()| {
                    if let Some(event) = parse_midi_message(message) {
                        let _ = sender.send(event);
                    }
                },
                (),
            )
            .map_err(|e| e.to_string())?;

        self.connection = Some(connection);
        Ok(())
    }

    pub fn stop_listening(&mut self) {
        self.connection = None;
    }

    pub fn is_listening(&self) -> bool {
        self.connection.is_some()
    }
}

pub fn parse_midi_message(message: &[u8]) -> Option<MidiLearnEvent> {
    if message.is_empty() {
        return None;
    }

    let status = message[0];
    let command = status & 0xF0;
    let channel = status & 0x0F;

    match command {
        0xB0 => {
            if message.len() >= 3 {
                Some(MidiLearnEvent::MidiCc {
                    channel,
                    controller: message[1],
                    value: message[2],
                })
            } else {
                None
            }
        }
        0x90 => {
            if message.len() >= 3 {
                Some(MidiLearnEvent::MidiNote {
                    channel,
                    note: message[1],
                    velocity: message[2],
                })
            } else {
                None
            }
        }
        0xE0 => {
            if message.len() >= 3 {
                let lsb = message[1] as u16;
                let msb = message[2] as u16;
                Some(MidiLearnEvent::MidiPitchBend {
                    channel,
                    value: (msb << 7) | lsb,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

impl Default for MidiInputManager {
    fn default() -> Self {
        let (manager, _rx) = Self::new();
        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_midi_cc_message() {
        let msg = [0xB3u8, 7, 100];
        let event = parse_midi_message(&msg).unwrap();
        assert_eq!(
            event,
            MidiLearnEvent::MidiCc {
                channel: 3,
                controller: 7,
                value: 100,
            }
        );
    }

    #[test]
    fn parse_midi_note_on_message() {
        let msg = [0x90u8, 60, 127];
        let event = parse_midi_message(&msg).unwrap();
        assert_eq!(
            event,
            MidiLearnEvent::MidiNote {
                channel: 0,
                note: 60,
                velocity: 127,
            }
        );
    }

    #[test]
    fn parse_midi_pitch_bend_message() {
        let msg = [0xE2u8, 0x00, 0x40];
        let event = parse_midi_message(&msg).unwrap();
        assert_eq!(
            event,
            MidiLearnEvent::MidiPitchBend {
                channel: 2,
                value: 0x2000,
            }
        );
    }

    #[test]
    fn parse_empty_message_returns_none() {
        assert!(parse_midi_message(&[]).is_none());
    }

    #[test]
    fn parse_unknown_status_returns_none() {
        assert!(parse_midi_message(&[0xF0, 0x00]).is_none());
    }

    #[test]
    fn parse_truncated_cc_returns_none() {
        assert!(parse_midi_message(&[0xB0, 7]).is_none());
    }
}
