use std::env;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::audio::compiler::CompiledTopology;
use crate::audio::runtime_manager::{AudioRuntimeAdapter, RuntimeAdapterStatus};

const SCSYNTH_OVERRIDE_ENV: &str = "SCRYSYNTH_SCSYNTH_PATH";
const SCSYNTH_BIN: &str = "scsynth";
const MACOS_APP_BUNDLE_SCSYNTH: &str = "/Applications/SuperCollider.app/Contents/Resources/scsynth";
const SCSYNTH_HOST: &str = "127.0.0.1";
const SCSYNTH_UDP_PORT: u16 = 57110;
const OSC_SYNC_TIMEOUT: Duration = Duration::from_secs(2);
const OSC_PACKET_BUFFER_SIZE: usize = 1536;
const INITIAL_SYNC_ID: i32 = 1;

#[derive(Debug)]
pub struct SuperColliderAdapter {
    process: Option<Child>,
    osc: Option<ScOscClient>,
}

impl AudioRuntimeAdapter for SuperColliderAdapter {
    fn start(&mut self) -> Result<RuntimeAdapterStatus, String> {
        if self.process.is_some() {
            if let Err(error) = self.sync_scsynth("boot") {
                return Ok(RuntimeAdapterStatus::Failed { message: error });
            }

            return Ok(RuntimeAdapterStatus::Booted {
                sample_rate_hz: 48_000,
                block_size: 64,
            });
        }

        let executable = match resolve_scsynth_executable() {
            Some(path) => path,
            None => {
                return Ok(RuntimeAdapterStatus::Failed {
                    message: format!(
                        "scsynth not found on PATH; install SuperCollider or set {SCSYNTH_OVERRIDE_ENV}"
                    ),
                });
            }
        };

        let child = Command::new(executable)
            .args(["-u", &SCSYNTH_UDP_PORT.to_string(), "-l", "1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| format!("failed to launch scsynth: {err}"))?;

        self.process = Some(child);
        self.osc = Some(ScOscClient::connect(
            SocketAddr::from(([127, 0, 0, 1], SCSYNTH_UDP_PORT)),
            OSC_SYNC_TIMEOUT,
        )?);

        if let Err(error) = self.sync_scsynth("boot") {
            let _ = terminate_process(&mut self.process);
            self.osc = None;
            return Ok(RuntimeAdapterStatus::Failed { message: error });
        }

        Ok(RuntimeAdapterStatus::Booted {
            sample_rate_hz: 48_000,
            block_size: 64,
        })
    }

    fn load_topology(
        &mut self,
        topology: &CompiledTopology,
    ) -> Result<RuntimeAdapterStatus, String> {
        if !self.is_process_running()? {
            return Ok(RuntimeAdapterStatus::Failed {
                message: "scsynth process is not running".to_string(),
            });
        }

        if let Err(error) = self.sync_scsynth("topology load") {
            return Ok(RuntimeAdapterStatus::Failed { message: error });
        }

        Ok(RuntimeAdapterStatus::Ready {
            active_patch_id: format!("patch-{}", topology.node_launch_order.len()),
        })
    }

    fn stop(&mut self) -> Result<RuntimeAdapterStatus, String> {
        terminate_process(&mut self.process)?;
        self.osc = None;
        Ok(RuntimeAdapterStatus::Stopped)
    }

    fn panic(&mut self) -> Result<RuntimeAdapterStatus, String> {
        terminate_process(&mut self.process)?;
        self.osc = None;
        Ok(RuntimeAdapterStatus::Panicked)
    }
}

impl Default for SuperColliderAdapter {
    fn default() -> Self {
        Self {
            process: None,
            osc: None,
        }
    }
}

impl Drop for SuperColliderAdapter {
    fn drop(&mut self) {
        let _ = terminate_process(&mut self.process);
    }
}

impl SuperColliderAdapter {
    fn sync_scsynth(&mut self, stage: &str) -> Result<(), String> {
        let osc = self
            .osc
            .as_mut()
            .ok_or_else(|| format!("{stage}: OSC client is not connected"))?;

        osc.send_message("/status", Vec::new())
            .map_err(|err| format!("{stage}: failed to send /status: {err}"))?;
        osc.sync()
            .map(|_| ())
            .map_err(|err| format!("{stage}: scsynth /sync failed: {err}"))
    }

    fn is_process_running(&mut self) -> Result<bool, String> {
        let Some(child) = self.process.as_mut() else {
            return Ok(false);
        };

        child
            .try_wait()
            .map(|status| status.is_none())
            .map_err(|err| format!("failed to inspect scsynth process: {err}"))
    }
}

fn resolve_scsynth_executable() -> Option<PathBuf> {
    if let Some(override_path) = env::var_os(SCSYNTH_OVERRIDE_ENV) {
        let path = PathBuf::from(override_path);
        if path.is_file() {
            return Some(path);
        }
    }

    env::var_os("PATH")
        .and_then(|path_var| {
            env::split_paths(&path_var)
                .map(|entry| entry.join(SCSYNTH_BIN))
                .find(|candidate| is_executable(candidate))
        })
        .or_else(|| {
            let app_bundle_path = PathBuf::from(MACOS_APP_BUNDLE_SCSYNTH);
            is_executable(&app_bundle_path).then_some(app_bundle_path)
        })
}

fn is_executable(path: &Path) -> bool {
    path.is_file()
}

fn terminate_process(process: &mut Option<Child>) -> Result<(), String> {
    if let Some(child) = process.as_mut() {
        child
            .kill()
            .map_err(|err| format!("failed to stop scsynth: {err}"))?;
        let _ = child.wait();
    }

    *process = None;
    Ok(())
}

#[derive(Debug)]
struct ScOscClient<T = UdpOscTransport> {
    transport: T,
    next_sync_id: i32,
    sync_timeout: Duration,
}

impl ScOscClient<UdpOscTransport> {
    fn connect(server_addr: SocketAddr, sync_timeout: Duration) -> Result<Self, String> {
        let local_addr = format!("{SCSYNTH_HOST}:0");
        let socket = UdpSocket::bind(local_addr)
            .map_err(|err| format!("failed to bind OSC UDP socket: {err}"))?;
        socket
            .connect(server_addr)
            .map_err(|err| format!("failed to connect OSC UDP socket to {server_addr}: {err}"))?;

        Ok(Self::new(UdpOscTransport { socket }, sync_timeout))
    }
}

impl<T> ScOscClient<T>
where
    T: OscTransport,
{
    fn new(transport: T, sync_timeout: Duration) -> Self {
        Self {
            transport,
            next_sync_id: INITIAL_SYNC_ID,
            sync_timeout,
        }
    }

    fn send_message(&mut self, addr: &str, args: Vec<rosc::OscType>) -> Result<(), ScOscError> {
        self.send_packet(rosc::OscPacket::Message(rosc::OscMessage {
            addr: addr.to_string(),
            args,
        }))
    }

    fn send_packet(&mut self, packet: rosc::OscPacket) -> Result<(), ScOscError> {
        let bytes =
            rosc::encoder::encode(&packet).map_err(|err| ScOscError::Encode(err.to_string()))?;
        self.transport.send(&bytes).map_err(ScOscError::from)?;
        Ok(())
    }

    fn sync(&mut self) -> Result<i32, ScOscError> {
        let sync_id = self.next_sync_id;
        self.next_sync_id = self.next_sync_id.saturating_add(1);
        self.send_message("/sync", vec![rosc::OscType::Int(sync_id)])?;
        self.wait_for_synced(sync_id)?;
        Ok(sync_id)
    }

    fn wait_for_synced(&mut self, sync_id: i32) -> Result<(), ScOscError> {
        let deadline = Instant::now() + self.sync_timeout;
        let mut buffer = [0; OSC_PACKET_BUFFER_SIZE];

        loop {
            let now = Instant::now();
            if now >= deadline {
                return Err(ScOscError::SyncTimeout {
                    sync_id,
                    timeout: self.sync_timeout,
                });
            }

            let remaining = deadline.saturating_duration_since(now);
            let size = match self.transport.recv(&mut buffer, remaining) {
                Ok(size) => size,
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) =>
                {
                    return Err(ScOscError::SyncTimeout {
                        sync_id,
                        timeout: self.sync_timeout,
                    });
                }
                Err(error) => return Err(ScOscError::from(error)),
            };
            let (_, packet) = rosc::decoder::decode_udp(&buffer[..size])
                .map_err(|err| ScOscError::Decode(err.to_string()))?;

            if has_matching_synced(&packet, sync_id)? {
                return Ok(());
            }
        }
    }
}

fn has_matching_synced(packet: &rosc::OscPacket, sync_id: i32) -> Result<bool, ScOscError> {
    match packet {
        rosc::OscPacket::Message(message) if message.addr == "/synced" => {
            match message.args.as_slice() {
                [rosc::OscType::Int(response_id)] if *response_id == sync_id => Ok(true),
                [rosc::OscType::Int(_)] => Ok(false),
                _ => Err(ScOscError::MalformedSynced {
                    sync_id,
                    packet: format!("{packet:?}"),
                }),
            }
        }
        rosc::OscPacket::Message(_) => Ok(false),
        rosc::OscPacket::Bundle(bundle) => {
            for packet in &bundle.content {
                if has_matching_synced(packet, sync_id)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
    }
}

trait OscTransport {
    fn send(&mut self, bytes: &[u8]) -> io::Result<usize>;
    fn recv(&mut self, buffer: &mut [u8], timeout: Duration) -> io::Result<usize>;
}

#[derive(Debug)]
struct UdpOscTransport {
    socket: UdpSocket,
}

impl OscTransport for UdpOscTransport {
    fn send(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.socket.send(bytes)
    }

    fn recv(&mut self, buffer: &mut [u8], timeout: Duration) -> io::Result<usize> {
        self.socket.set_read_timeout(Some(timeout))?;
        self.socket.recv(buffer)
    }
}

#[derive(Debug, PartialEq)]
enum ScOscError {
    Encode(String),
    Decode(String),
    Io(String),
    SyncTimeout { sync_id: i32, timeout: Duration },
    MalformedSynced { sync_id: i32, packet: String },
}

impl From<io::Error> for ScOscError {
    fn from(error: io::Error) -> Self {
        ScOscError::Io(error.to_string())
    }
}

impl std::fmt::Display for ScOscError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScOscError::Encode(message) => write!(formatter, "OSC encode error: {message}"),
            ScOscError::Decode(message) => write!(formatter, "OSC decode error: {message}"),
            ScOscError::Io(message) => write!(formatter, "OSC IO error: {message}"),
            ScOscError::SyncTimeout { sync_id, timeout } => {
                write!(formatter, "/sync {sync_id} timed out after {timeout:?}")
            }
            ScOscError::MalformedSynced { sync_id, packet } => {
                write!(
                    formatter,
                    "malformed /synced response for sync {sync_id}: {packet}"
                )
            }
        }
    }
}

impl std::error::Error for ScOscError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    #[test]
    fn osc_sync_sends_correlation_id_and_accepts_matching_synced() {
        let (transport, sent_packets) = ScriptedOscTransport::new(vec![ScriptedResponse::Packet(
            synced_packet(INITIAL_SYNC_ID),
        )]);
        let mut client = ScOscClient::new(transport, Duration::from_millis(250));

        let sync_id = client.sync().expect("sync succeeds");

        assert_eq!(sync_id, INITIAL_SYNC_ID);
        let sent = sent_packets.lock().expect("sent packets");
        assert_eq!(sent.len(), 1);
        assert_eq!(osc_message_addr(&sent[0]), Some("/sync"));
        assert_eq!(osc_message_int_arg(&sent[0]), Some(INITIAL_SYNC_ID));
    }

    #[test]
    fn osc_sync_accepts_matching_synced_inside_bundle() {
        let (transport, _sent_packets) = ScriptedOscTransport::new(vec![ScriptedResponse::Packet(
            rosc::OscPacket::Bundle(rosc::OscBundle {
                timetag: rosc::OscTime::from((0, 1)),
                content: vec![
                    rosc::OscPacket::Message(rosc::OscMessage {
                        addr: "/status.reply".to_string(),
                        args: vec![],
                    }),
                    synced_packet(INITIAL_SYNC_ID),
                ],
            }),
        )]);
        let mut client = ScOscClient::new(transport, Duration::from_millis(250));

        assert_eq!(client.sync().expect("sync succeeds"), INITIAL_SYNC_ID);
    }

    #[test]
    fn osc_sync_times_out_when_no_synced_response_arrives() {
        let (transport, _sent_packets) = ScriptedOscTransport::new(vec![ScriptedResponse::Timeout]);
        let mut client = ScOscClient::new(transport, Duration::from_millis(20));

        let error = client.sync().expect_err("sync times out");

        assert!(matches!(
            error,
            ScOscError::SyncTimeout {
                sync_id: INITIAL_SYNC_ID,
                ..
            }
        ));
    }

    #[test]
    fn osc_sync_fails_on_malformed_synced_response() {
        let (transport, _sent_packets) = ScriptedOscTransport::new(vec![ScriptedResponse::Packet(
            rosc::OscPacket::Message(rosc::OscMessage {
                addr: "/synced".to_string(),
                args: vec![rosc::OscType::String("not-an-int".to_string())],
            }),
        )]);
        let mut client = ScOscClient::new(transport, Duration::from_millis(250));

        let error = client.sync().expect_err("malformed sync fails");

        assert!(matches!(
            error,
            ScOscError::MalformedSynced {
                sync_id: INITIAL_SYNC_ID,
                ..
            }
        ));
    }

    #[test]
    fn osc_client_can_send_bundles() {
        let (transport, sent_packets) = ScriptedOscTransport::new(vec![]);
        let mut client = ScOscClient::new(transport, Duration::from_millis(20));

        client
            .send_packet(rosc::OscPacket::Bundle(rosc::OscBundle {
                timetag: rosc::OscTime::from((0, 1)),
                content: vec![rosc::OscPacket::Message(rosc::OscMessage {
                    addr: "/n_set".to_string(),
                    args: vec![rosc::OscType::Int(2000)],
                })],
            }))
            .expect("bundle sends");

        let sent = sent_packets.lock().expect("sent packets");
        assert_eq!(sent.len(), 1);
        assert!(matches!(sent[0], rosc::OscPacket::Bundle(_)));
    }

    fn synced_packet(sync_id: i32) -> rosc::OscPacket {
        rosc::OscPacket::Message(rosc::OscMessage {
            addr: "/synced".to_string(),
            args: vec![rosc::OscType::Int(sync_id)],
        })
    }

    fn osc_message_addr(packet: &rosc::OscPacket) -> Option<&str> {
        match packet {
            rosc::OscPacket::Message(message) => Some(message.addr.as_str()),
            rosc::OscPacket::Bundle(_) => None,
        }
    }

    fn osc_message_int_arg(packet: &rosc::OscPacket) -> Option<i32> {
        match packet {
            rosc::OscPacket::Message(message) => match message.args.as_slice() {
                [rosc::OscType::Int(value)] => Some(*value),
                _ => None,
            },
            rosc::OscPacket::Bundle(_) => None,
        }
    }

    struct ScriptedOscTransport {
        sent_packets: Arc<Mutex<Vec<rosc::OscPacket>>>,
        responses: VecDeque<ScriptedResponse>,
    }

    impl ScriptedOscTransport {
        fn new(responses: Vec<ScriptedResponse>) -> (Self, Arc<Mutex<Vec<rosc::OscPacket>>>) {
            let sent_packets = Arc::new(Mutex::new(Vec::new()));

            (
                Self {
                    sent_packets: Arc::clone(&sent_packets),
                    responses: responses.into(),
                },
                sent_packets,
            )
        }
    }

    impl OscTransport for ScriptedOscTransport {
        fn send(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let (_, packet) = rosc::decoder::decode_udp(bytes).map_err(|err| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("failed to decode sent packet: {err}"),
                )
            })?;
            self.sent_packets.lock().expect("sent packets").push(packet);
            Ok(bytes.len())
        }

        fn recv(&mut self, buffer: &mut [u8], _timeout: Duration) -> io::Result<usize> {
            let response = self
                .responses
                .pop_front()
                .unwrap_or(ScriptedResponse::Timeout);
            match response {
                ScriptedResponse::Packet(packet) => {
                    let bytes = rosc::encoder::encode(&packet).map_err(|err| {
                        io::Error::new(io::ErrorKind::InvalidData, err.to_string())
                    })?;
                    buffer[..bytes.len()].copy_from_slice(&bytes);
                    Ok(bytes.len())
                }
                ScriptedResponse::Timeout => Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "scripted OSC receive timeout",
                )),
            }
        }
    }

    enum ScriptedResponse {
        Packet(rosc::OscPacket),
        Timeout,
    }
}
