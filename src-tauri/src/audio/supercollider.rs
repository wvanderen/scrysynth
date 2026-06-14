use std::env;
use std::fs;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::audio::compiler::CompiledTopology;
use crate::audio::runtime_manager::{AudioRuntimeAdapter, RuntimeAdapterStatus};
use crate::audio::synthdefs::{plan_sc_resources, ScResourcePlan, ScSynthArg};

const SCSYNTH_OVERRIDE_ENV: &str = "SCRYSYNTH_SCSYNTH_PATH";
const SCSYNTH_BIN: &str = "scsynth";
const MACOS_APP_BUNDLE_SCSYNTH: &str = "/Applications/SuperCollider.app/Contents/Resources/scsynth";
const SCSYNTH_HOST: &str = "127.0.0.1";
const SCSYNTH_UDP_PORT: u16 = 57110;
const OSC_SYNC_TIMEOUT: Duration = Duration::from_secs(2);
const SCSYNTH_BOOT_TIMEOUT: Duration = Duration::from_secs(15);
const SCSYNTH_BOOT_RETRY_DELAY: Duration = Duration::from_millis(100);
const OSC_PACKET_BUFFER_SIZE: usize = 1536;
const INITIAL_SYNC_ID: i32 = 1;

#[derive(Debug)]
pub struct SuperColliderAdapter<T = UdpOscTransport> {
    process: Option<Child>,
    osc: Option<ScOscClient<T>>,
    active_patch: Option<ScResourcePlan>,
}

impl AudioRuntimeAdapter for SuperColliderAdapter<UdpOscTransport> {
    fn start(&mut self) -> Result<RuntimeAdapterStatus, String> {
        if self.process.is_some() {
            if let Err(error) = self.sync_scsynth("boot") {
                return Ok(RuntimeAdapterStatus::Failed {
                    message: error,
                    active_patch_id: self.active_patch.as_ref().map(|plan| plan.patch_id.clone()),
                });
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
                    message: missing_scsynth_message(),
                    active_patch_id: None,
                });
            }
        };

        let child = Command::new(&executable)
            .args(["-u", &SCSYNTH_UDP_PORT.to_string(), "-l", "1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| RuntimeAdapterStatus::Failed {
                message: format!(
                    "Failed to spawn scsynth at `{}`: {err}. Set {SCSYNTH_OVERRIDE_ENV} to the full scsynth executable path if this install lives outside PATH or the macOS bundle fallback ({MACOS_APP_BUNDLE_SCSYNTH}).",
                    executable.display()
                ),
                active_patch_id: None,
            });
        let child = match child {
            Ok(child) => child,
            Err(status) => return Ok(status),
        };

        self.process = Some(child);
        self.osc = Some(ScOscClient::connect(
            SocketAddr::from(([127, 0, 0, 1], SCSYNTH_UDP_PORT)),
            OSC_SYNC_TIMEOUT,
        )?);

        if let Err(error) = self.wait_for_scsynth_boot() {
            let _ = terminate_process(&mut self.process);
            self.osc = None;
            return Ok(RuntimeAdapterStatus::Failed {
                message: error,
                active_patch_id: None,
            });
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
                message: "Runtime server error: scsynth process is not running while applying topology. Start audio again; if this repeats, check the SuperCollider server logs.".to_string(),
                active_patch_id: None,
            });
        }

        let plan = match plan_sc_resources(topology) {
            Ok(plan) => plan,
            Err(error) => {
                return Ok(RuntimeAdapterStatus::Failed {
                    message: format!("Topology apply failure before contacting scsynth: {error}"),
                    active_patch_id: self.active_patch.as_ref().map(|plan| plan.patch_id.clone()),
                });
            }
        };

        if let Err(error) = self.apply_resource_plan(&plan) {
            return Ok(RuntimeAdapterStatus::Failed {
                message: error,
                active_patch_id: self.active_patch.as_ref().map(|plan| plan.patch_id.clone()),
            });
        }

        let patch_id = plan.patch_id.clone();
        self.active_patch = Some(plan);
        Ok(RuntimeAdapterStatus::Ready {
            active_patch_id: patch_id,
        })
    }

    fn set_parameter_value(
        &mut self,
        node_id: &str,
        parameter_id: &str,
        value: f64,
    ) -> Result<RuntimeAdapterStatus, String> {
        if !self.is_process_running()? {
            return Ok(RuntimeAdapterStatus::Failed {
                message: "Runtime server error during live parameter update: scsynth process is not running. Start audio again before applying live controls.".to_string(),
                active_patch_id: self.active_patch.as_ref().map(|plan| plan.patch_id.clone()),
            });
        }

        match self.send_live_parameter(node_id, parameter_id, value) {
            Ok(active_patch_id) => Ok(RuntimeAdapterStatus::Ready { active_patch_id }),
            Err(message) => Ok(RuntimeAdapterStatus::Failed {
                message,
                active_patch_id: self.active_patch.as_ref().map(|plan| plan.patch_id.clone()),
            }),
        }
    }

    fn stop(&mut self) -> Result<RuntimeAdapterStatus, String> {
        terminate_process(&mut self.process)?;
        self.osc = None;
        self.active_patch = None;
        Ok(RuntimeAdapterStatus::Stopped)
    }

    fn panic(&mut self) -> Result<RuntimeAdapterStatus, String> {
        terminate_process(&mut self.process)?;
        self.osc = None;
        self.active_patch = None;
        Ok(RuntimeAdapterStatus::Panicked)
    }
}

impl Default for SuperColliderAdapter<UdpOscTransport> {
    fn default() -> Self {
        Self {
            process: None,
            osc: None,
            active_patch: None,
        }
    }
}

impl<T> Drop for SuperColliderAdapter<T> {
    fn drop(&mut self) {
        let _ = terminate_process(&mut self.process);
    }
}

impl<T> SuperColliderAdapter<T>
where
    T: OscTransport,
{
    fn apply_resource_plan(&mut self, plan: &ScResourcePlan) -> Result<(), String> {
        if let Some(active_patch) = self.active_patch.clone() {
            self.free_resource_plan(&active_patch, "topology unload previous patch")?;
            self.active_patch = None;
        }

        {
            let osc = self.osc.as_mut().ok_or_else(|| {
                "Audio runtime app state error during topology apply: OSC client is not connected."
                    .to_string()
            })?;

            for synthdef in &plan.synthdefs {
                let bytes =
                    fs::read(resolve_resource_path(synthdef.relative_path)).map_err(|err| {
                        format!(
                            "SynthDef load failure: failed to read bundled SynthDef resource `{}`: {err}. Reinstall Scrysynth or regenerate checked-in synthdefs before starting audio.",
                            synthdef.relative_path
                        )
                    })?;
                osc.send_message("/d_recv", vec![rosc::OscType::Blob(bytes)])
                    .map_err(|err| {
                        format!(
                            "SynthDef load failure: failed to send SynthDef `{}` to scsynth with /d_recv: {err}",
                            synthdef.name
                        )
                    })?;
            }
        }
        self.sync_scsynth("topology load synthdefs")?;

        let mut created_group_count = 0;
        {
            let osc = self.osc.as_mut().ok_or_else(|| {
                "Audio runtime app state error during topology apply: OSC client is not connected."
                    .to_string()
            })?;
            for group in &plan.groups {
                if let Err(err) = osc.send_message(
                    "/g_new",
                    vec![
                        rosc::OscType::Int(group.node_id),
                        rosc::OscType::Int(1),
                        rosc::OscType::Int(0),
                    ],
                ) {
                    let _ = self.free_created_resources(plan, 0, created_group_count);
                    return Err(format!(
                        "Topology apply failure: failed to create SuperCollider group `{}`: {err}",
                        group.group_key
                    ));
                }
                created_group_count += 1;
            }
        }
        if let Err(error) = self.sync_scsynth("topology load groups") {
            let _ = self.free_created_resources(plan, 0, created_group_count);
            return Err(error);
        }

        let mut created_synth_count = 0;
        {
            let osc = self.osc.as_mut().ok_or_else(|| {
                "Audio runtime app state error during topology apply: OSC client is not connected."
                    .to_string()
            })?;
            for synth in &plan.synths {
                let mut args = vec![
                    rosc::OscType::String(synth.synthdef_name.to_string()),
                    rosc::OscType::Int(synth.node_id),
                    rosc::OscType::Int(1),
                    rosc::OscType::Int(synth.group_node_id),
                ];
                args.extend(synth_args_to_osc(&synth.args));
                if let Err(err) = osc.send_message("/s_new", args) {
                    let _ =
                        self.free_created_resources(plan, created_synth_count, created_group_count);
                    return Err(format!(
                        "Topology apply failure: failed to create SuperCollider synth for node `{}`: {err}",
                        synth.node_key
                    ));
                }
                created_synth_count += 1;
            }
        }
        if let Err(error) = self.sync_scsynth("topology load synths") {
            let _ = self.free_created_resources(plan, created_synth_count, created_group_count);
            return Err(error);
        }

        Ok(())
    }

    fn free_resource_plan(&mut self, plan: &ScResourcePlan, stage: &str) -> Result<(), String> {
        self.free_created_resources(plan, plan.synths.len(), plan.groups.len())?;
        self.sync_scsynth(stage)
    }

    fn send_live_parameter(
        &mut self,
        node_id: &str,
        parameter_id: &str,
        value: f64,
    ) -> Result<String, String> {
        let active_patch = self
            .active_patch
            .as_ref()
            .ok_or_else(|| {
                "Audio runtime app state error during live parameter update: no active SuperCollider patch is registered."
                    .to_string()
            })?;
        let control_key = format!("{node_id}:{parameter_id}");
        let control = active_patch
            .controls
            .iter()
            .find(|control| control.control_key == control_key)
            .ok_or_else(|| {
                format!(
                    "Topology apply failure during live parameter update: no SuperCollider control exists for node `{node_id}` parameter `{parameter_id}` in the active patch."
                )
            })?;
        let active_patch_id = active_patch.patch_id.clone();
        let osc = self
            .osc
            .as_mut()
            .ok_or_else(|| {
                "Audio runtime app state error during live parameter update: OSC client is not connected."
                    .to_string()
            })?;

        osc.send_message(
            "/n_set",
            vec![
                rosc::OscType::Int(control.synth_node_id),
                rosc::OscType::String(control.parameter_name.clone()),
                rosc::OscType::Float(value as f32),
            ],
        )
        .map_err(|err| {
            format!("Runtime server error during live parameter update: failed to send /n_set to scsynth: {err}")
        })?;
        self.sync_scsynth("live parameter update")?;

        Ok(active_patch_id)
    }

    fn free_created_resources(
        &mut self,
        plan: &ScResourcePlan,
        synth_count: usize,
        group_count: usize,
    ) -> Result<(), String> {
        let osc = self
            .osc
            .as_mut()
            .ok_or_else(|| "topology cleanup: OSC client is not connected".to_string())?;

        for synth in plan.synths.iter().take(synth_count).rev() {
            osc.send_message("/n_free", vec![rosc::OscType::Int(synth.node_id)])
                .map_err(|err| {
                    format!(
                        "topology cleanup: failed to free synth for node {}: {err}",
                        synth.node_key
                    )
                })?;
        }

        for group in plan.groups.iter().take(group_count).rev() {
            osc.send_message("/n_free", vec![rosc::OscType::Int(group.node_id)])
                .map_err(|err| {
                    format!(
                        "topology cleanup: failed to free group {}: {err}",
                        group.group_key
                    )
                })?;
        }

        Ok(())
    }

    fn sync_scsynth(&mut self, stage: &str) -> Result<(), String> {
        let osc = self.osc.as_mut().ok_or_else(|| {
            format!("Audio runtime app state error during {stage}: OSC client is not connected.")
        })?;

        osc.send_message("/status", Vec::new()).map_err(|err| {
            format!("Runtime server error during {stage}: failed to send /status to scsynth: {err}")
        })?;
        osc.sync().map(|_| ()).map_err(|err| {
            format!("Runtime server error during {stage}: scsynth did not confirm OSC /sync: {err}")
        })
    }

    fn wait_for_scsynth_boot(&mut self) -> Result<(), String> {
        let deadline = Instant::now() + SCSYNTH_BOOT_TIMEOUT;

        loop {
            match self.sync_scsynth("boot") {
                Ok(()) => return Ok(()),
                Err(error) => {
                    if !self.is_process_running()? {
                        return Err(format!(
                            "Runtime server error during boot: scsynth exited before confirming OSC /sync. Last boot probe: {}",
                            error
                        ));
                    }

                    if Instant::now() >= deadline {
                        return Err(error);
                    }

                    thread::sleep(SCSYNTH_BOOT_RETRY_DELAY);
                }
            }
        }
    }

    fn is_process_running(&mut self) -> Result<bool, String> {
        let Some(child) = self.process.as_mut() else {
            return Ok(false);
        };

        child
            .try_wait()
            .map(|status| status.is_none())
            .map_err(|err| {
                format!("Audio runtime app state error while inspecting scsynth process: {err}")
            })
    }
}

fn missing_scsynth_message() -> String {
    format!(
        "scsynth not found. Install SuperCollider, put `scsynth` on PATH, or set {SCSYNTH_OVERRIDE_ENV} to the full executable path. On macOS Scrysynth also checks the bundle fallback `{MACOS_APP_BUNDLE_SCSYNTH}`."
    )
}

fn synth_args_to_osc(args: &[ScSynthArg]) -> Vec<rosc::OscType> {
    args.iter()
        .flat_map(|arg| {
            [
                rosc::OscType::String(arg.name.clone()),
                rosc::OscType::Float(arg.value),
            ]
        })
        .collect()
}

fn resolve_resource_path(relative_path: &str) -> PathBuf {
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    if dev_path.exists() {
        return dev_path;
    }

    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let local_path = exe_dir.join(relative_path);
            if local_path.exists() {
                return local_path;
            }

            let macos_resource_path = exe_dir.join("../Resources").join(relative_path);
            if macos_resource_path.exists() {
                return macos_resource_path;
            }
        }
    }

    dev_path
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

pub trait OscTransport {
    fn send(&mut self, bytes: &[u8]) -> io::Result<usize>;
    fn recv(&mut self, buffer: &mut [u8], timeout: Duration) -> io::Result<usize>;
}

#[derive(Debug)]
pub struct UdpOscTransport {
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

    #[test]
    fn apply_resource_plan_sends_groups_then_synths_in_plan_order() {
        let (transport, sent_packets) = ScriptedOscTransport::new(vec![
            ScriptedResponse::Packet(synced_packet(INITIAL_SYNC_ID)),
            ScriptedResponse::Packet(synced_packet(INITIAL_SYNC_ID + 1)),
            ScriptedResponse::Packet(synced_packet(INITIAL_SYNC_ID + 2)),
        ]);
        let mut adapter = SuperColliderAdapter {
            process: None,
            osc: Some(ScOscClient::new(transport, Duration::from_millis(250))),
            active_patch: None,
        };
        let plan = test_resource_plan();

        adapter
            .apply_resource_plan(&plan)
            .expect("resource plan applies");

        let sent = sent_packets.lock().expect("sent packets");
        assert_eq!(
            sent.iter()
                .map(|packet| osc_message_addr(packet)
                    .expect("message packet")
                    .to_string())
                .collect::<Vec<_>>(),
            vec![
                "/status", "/sync", "/g_new", "/status", "/sync", "/s_new", "/s_new", "/status",
                "/sync",
            ]
        );
        assert_eq!(osc_message_int_arg(&sent[2]), Some(1000));
        assert_eq!(
            osc_message_string_arg(&sent[5], 0),
            Some("scrysynth_v1_source_oscillator")
        );
        assert_eq!(osc_message_int_arg_at(&sent[5], 1), Some(2000));
        assert_eq!(
            osc_message_string_arg(&sent[6], 0),
            Some("scrysynth_v1_output")
        );
        assert_eq!(osc_message_int_arg_at(&sent[6], 1), Some(2001));
    }

    #[test]
    fn apply_resource_plan_frees_created_nodes_when_final_sync_fails() {
        let (transport, sent_packets) = ScriptedOscTransport::new(vec![
            ScriptedResponse::Packet(synced_packet(INITIAL_SYNC_ID)),
            ScriptedResponse::Packet(synced_packet(INITIAL_SYNC_ID + 1)),
            ScriptedResponse::Timeout,
        ]);
        let mut adapter = SuperColliderAdapter {
            process: None,
            osc: Some(ScOscClient::new(transport, Duration::from_millis(20))),
            active_patch: None,
        };
        let plan = test_resource_plan();

        let error = adapter
            .apply_resource_plan(&plan)
            .expect_err("final sync fails");

        assert!(error.contains("topology load synths"));
        let sent = sent_packets.lock().expect("sent packets");
        assert_eq!(
            sent.iter()
                .map(|packet| osc_message_addr(packet)
                    .expect("message packet")
                    .to_string())
                .collect::<Vec<_>>(),
            vec![
                "/status", "/sync", "/g_new", "/status", "/sync", "/s_new", "/s_new", "/status",
                "/sync", "/n_free", "/n_free", "/n_free",
            ]
        );
        assert_eq!(osc_message_int_arg(&sent[9]), Some(2001));
        assert_eq!(osc_message_int_arg(&sent[10]), Some(2000));
        assert_eq!(osc_message_int_arg(&sent[11]), Some(1000));
    }

    #[test]
    fn apply_resource_plan_preserves_active_patch_when_previous_unload_fails() {
        let (transport, sent_packets) = ScriptedOscTransport::new(vec![ScriptedResponse::Timeout]);
        let active_plan = test_resource_plan();
        let mut next_plan = test_resource_plan();
        next_plan.patch_id = "patch-next".to_string();
        next_plan.groups[0].node_id = 1100;
        next_plan.synths[0].node_id = 2100;
        next_plan.synths[0].group_node_id = 1100;
        next_plan.synths[1].node_id = 2101;
        next_plan.synths[1].group_node_id = 1100;
        let mut adapter = SuperColliderAdapter {
            process: None,
            osc: Some(ScOscClient::new(transport, Duration::from_millis(20))),
            active_patch: Some(active_plan.clone()),
        };

        let error = adapter
            .apply_resource_plan(&next_plan)
            .expect_err("previous unload sync fails");

        assert!(error.contains("topology unload previous patch"));
        assert_eq!(adapter.active_patch, Some(active_plan));
        let sent = sent_packets.lock().expect("sent packets");
        assert_eq!(
            sent.iter()
                .map(|packet| osc_message_addr(packet)
                    .expect("message packet")
                    .to_string())
                .collect::<Vec<_>>(),
            vec!["/n_free", "/n_free", "/n_free", "/status", "/sync"]
        );
    }

    #[test]
    fn send_live_parameter_sends_n_set_to_adapter_owned_synth() {
        let (transport, sent_packets) = ScriptedOscTransport::new(vec![ScriptedResponse::Packet(
            synced_packet(INITIAL_SYNC_ID),
        )]);
        let mut plan = test_resource_plan();
        plan.controls = vec![crate::audio::synthdefs::ScControlPlan {
            control_key: "node-source:param-level".to_string(),
            synth_node_id: 2000,
            parameter_name: "level".to_string(),
        }];
        let mut adapter = SuperColliderAdapter {
            process: None,
            osc: Some(ScOscClient::new(transport, Duration::from_millis(250))),
            active_patch: Some(plan),
        };

        let active_patch_id = adapter
            .send_live_parameter("node-source", "param-level", 0.42)
            .expect("live parameter sends");

        assert_eq!(active_patch_id, "patch-test");
        let sent = sent_packets.lock().expect("sent packets");
        assert_eq!(
            sent.iter()
                .map(|packet| osc_message_addr(packet)
                    .expect("message packet")
                    .to_string())
                .collect::<Vec<_>>(),
            vec!["/n_set", "/status", "/sync"]
        );
        assert_eq!(osc_message_int_arg(&sent[0]), Some(2000));
        assert_eq!(osc_message_string_arg(&sent[0], 1), Some("level"));
        assert!(
            (osc_message_float_arg(&sent[0], 2).expect("float value") - 0.42).abs() < 0.000_001
        );
    }

    #[test]
    fn send_live_parameter_fails_when_control_is_not_in_active_patch() {
        let (transport, _sent_packets) = ScriptedOscTransport::new(vec![]);
        let mut adapter = SuperColliderAdapter {
            process: None,
            osc: Some(ScOscClient::new(transport, Duration::from_millis(20))),
            active_patch: Some(test_resource_plan()),
        };

        let error = adapter
            .send_live_parameter("node-source", "missing-param", 0.42)
            .expect_err("missing control fails");

        assert!(error.contains("no SuperCollider control"));
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
        osc_message_int_arg_at(packet, 0)
    }

    fn osc_message_int_arg_at(packet: &rosc::OscPacket, index: usize) -> Option<i32> {
        match packet {
            rosc::OscPacket::Message(message) => match message.args.get(index) {
                Some(rosc::OscType::Int(value)) => Some(*value),
                _ => None,
            },
            rosc::OscPacket::Bundle(_) => None,
        }
    }

    fn osc_message_string_arg(packet: &rosc::OscPacket, index: usize) -> Option<&str> {
        match packet {
            rosc::OscPacket::Message(message) => match message.args.get(index) {
                Some(rosc::OscType::String(value)) => Some(value.as_str()),
                _ => None,
            },
            rosc::OscPacket::Bundle(_) => None,
        }
    }

    fn osc_message_float_arg(packet: &rosc::OscPacket, index: usize) -> Option<f64> {
        match packet {
            rosc::OscPacket::Message(message) => match message.args.get(index) {
                Some(rosc::OscType::Float(value)) => Some(f64::from(*value)),
                Some(rosc::OscType::Double(value)) => Some(*value),
                _ => None,
            },
            rosc::OscPacket::Bundle(_) => None,
        }
    }

    fn test_resource_plan() -> ScResourcePlan {
        ScResourcePlan {
            patch_id: "patch-test".to_string(),
            synthdefs: Vec::new(),
            groups: vec![crate::audio::synthdefs::ScGroupPlan {
                group_key: "group-main-signal".to_string(),
                node_id: 1000,
            }],
            synths: vec![
                crate::audio::synthdefs::ScSynthPlan {
                    node_key: "node-source".to_string(),
                    node_id: 2000,
                    synthdef_name: "scrysynth_v1_source_oscillator",
                    group_key: "group-main-signal".to_string(),
                    group_node_id: 1000,
                    args: vec![
                        ScSynthArg {
                            name: "out_bus".to_string(),
                            value: 2.0,
                        },
                        ScSynthArg {
                            name: "frequency".to_string(),
                            value: 220.0,
                        },
                    ],
                },
                crate::audio::synthdefs::ScSynthPlan {
                    node_key: "node-output".to_string(),
                    node_id: 2001,
                    synthdef_name: "scrysynth_v1_output",
                    group_key: "group-main-signal".to_string(),
                    group_node_id: 1000,
                    args: vec![
                        ScSynthArg {
                            name: "in_bus".to_string(),
                            value: 2.0,
                        },
                        ScSynthArg {
                            name: "hardware_out".to_string(),
                            value: 0.0,
                        },
                    ],
                },
            ],
            controls: Vec::new(),
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
