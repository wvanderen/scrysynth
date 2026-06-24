use std::env;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

use tauri::AppHandle;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

use crate::visual::adapter::VisualAdapterStatus;
use crate::visual::compiler::{CompiledVisualScene, VisualParameterUpdate};
use crate::visual::protocol::{
    AppToVisualMessage, AppToVisualPayload, VisualHandshake, VisualParameterUpdateBatch,
    VisualProtocolError, VisualSceneLoad, VisualShutdown, VisualShutdownMode, VisualToAppMessage,
    VisualToAppPayload, DEFAULT_READY_TIMEOUT_MS,
};

use super::adapter::VisualRuntimeAdapter;

const BEVY_OVERRIDE_ENV: &str = "SCRYSYNTH_BEVY_PATH";
const BEVY_BIN: &str = "scrysynth-visual";
/// Bare sidecar name passed to `app.shell().sidecar(...)`. The shell plugin
/// resolves the host-triple-suffixed binary that Tauri's `externalBin` placed
/// next to the main executable; callers must NOT pre-suffix it.
const SIDECAR_NAME: &str = "scrysynth-visual";

/// Which spawn mechanism produced (or attempted to produce) the live child.
/// Drives missing-binary message wording so a packaged-app end user gets
/// reinstall guidance instead of a dev-only env-var instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpawnKind {
    /// Packaged path: `app.shell().sidecar()` against the bundled binary.
    Bundled,
    /// Dev/test path: raw `std::process::Command` against an override/PATH binary.
    DevOverride,
}

/// Live child handle, normalized across the two spawn mechanisms.
#[derive(Debug)]
enum SidecarChild {
    /// Raw `std::process` spawn — dev/test override path (no Tauri runtime).
    Std(Child),
    /// Tauri shell-plugin sidecar — packaged-app path (`externalBin` binary).
    Plugin(CommandChild),
}

#[derive(Debug)]
pub struct BevySidecarAdapter {
    process: Option<SidecarChild>,
    stdin: Option<ChildStdin>,
    responses: Option<Receiver<Result<VisualToAppMessage, String>>>,
    next_sequence_id: u64,
    active_scene_id: Option<String>,
    timeout: Duration,
    executable_override: Option<PathBuf>,
    executable_args: Vec<String>,
    app_handle: Option<AppHandle>,
    /// Set by the sidecar forwarder thread when the child terminates; lets
    /// `ensure_running` detect exit on the Plugin path, which has no
    /// `try_wait` equivalent.
    terminated: Arc<AtomicBool>,
    /// Tracks which resolution path was last attempted so missing-binary
    /// messages branch correctly (bundled vs dev override).
    last_spawn_kind: SpawnKind,
}

impl Default for BevySidecarAdapter {
    fn default() -> Self {
        Self {
            process: None,
            stdin: None,
            responses: None,
            next_sequence_id: 1,
            active_scene_id: None,
            timeout: Duration::from_millis(DEFAULT_READY_TIMEOUT_MS),
            executable_override: None,
            executable_args: Vec::new(),
            app_handle: None,
            terminated: Arc::new(AtomicBool::new(false)),
            last_spawn_kind: SpawnKind::DevOverride,
        }
    }
}

impl BevySidecarAdapter {
    pub fn with_executable_override(executable: PathBuf, timeout: Duration) -> Self {
        Self::with_executable_override_and_args(executable, timeout, Vec::new())
    }

    pub fn with_executable_override_and_args(
        executable: PathBuf,
        timeout: Duration,
        executable_args: Vec<String>,
    ) -> Self {
        let mut adapter = Self::default();
        adapter.executable_override = Some(executable);
        adapter.timeout = timeout;
        adapter.executable_args = executable_args;
        adapter
    }
}

impl VisualRuntimeAdapter for BevySidecarAdapter {
    fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }

    fn start(&mut self) -> Result<VisualAdapterStatus, String> {
        if self.process.is_some() {
            return Ok(VisualAdapterStatus::Booted {
                renderer: "bevy".to_string(),
            });
        }

        // Precedence (Pitfall 6 + D-06): an explicit executable override
        // (tests) or a set SCRYSYNTH_BEVY_PATH (dev override pointing at a
        // built/debug binary) takes precedence over the bundled sidecar so a
        // developer can target a non-packaged build. Otherwise, when an
        // AppHandle is present we launch via the shell-plugin sidecar API
        // (the packaged-app path). With neither override nor AppHandle we
        // fall back to raw Command + PATH lookup (unit tests / dev without
        // the plugin registered).
        let dev_override_requested = self.executable_override.is_some()
            || env::var_os(BEVY_OVERRIDE_ENV).is_some();

        if dev_override_requested || self.app_handle.is_none() {
            self.start_via_command()
        } else {
            self.start_via_sidecar()
        }
    }

    fn load_scene(&mut self, scene: &CompiledVisualScene) -> Result<VisualAdapterStatus, String> {
        self.ensure_running()?;
        let sequence_id = self.next_sequence_id();
        let response = self.send_and_wait(
            AppToVisualMessage::new(
                sequence_id,
                AppToVisualPayload::LoadScene(VisualSceneLoad::from(scene)),
            ),
            sequence_id,
        );

        match response {
            Ok(VisualToAppPayload::SceneLoaded(loaded)) => {
                self.active_scene_id = Some(loaded.scene_id.clone());
                Ok(VisualAdapterStatus::SceneLoaded {
                    scene_id: loaded.scene_id,
                    rendering: loaded.rendering,
                })
            }
            Ok(VisualToAppPayload::Error(error)) => Ok(VisualAdapterStatus::Failed {
                message: protocol_error_message("visual scene load failed", &error),
            }),
            Ok(payload) => Ok(VisualAdapterStatus::Failed {
                message: format!("visual scene load returned unexpected payload {payload:?}"),
            }),
            Err(message) => Ok(VisualAdapterStatus::Failed { message }),
        }
    }

    fn update_parameters(&mut self, params: &[VisualParameterUpdate]) -> Result<(), String> {
        self.ensure_running()?;
        if self.active_scene_id.is_none() {
            return Err(
                "cannot update visual parameters before a scene has been loaded".to_string(),
            );
        }

        let sequence_id = self.next_sequence_id();
        match self.send_and_wait(
            AppToVisualMessage::new(
                sequence_id,
                AppToVisualPayload::UpdateParameters(VisualParameterUpdateBatch::from(params)),
            ),
            sequence_id,
        )? {
            VisualToAppPayload::ParameterBatchApplied(applied) => {
                let requested_count = params.len() as u32;
                if applied.applied_count == requested_count {
                    Ok(())
                } else {
                    Err(format!(
                        "visual parameter update applied {} of {} requested patches",
                        applied.applied_count, requested_count
                    ))
                }
            }
            VisualToAppPayload::Error(error) => Err(protocol_error_message(
                "visual parameter update failed",
                &error,
            )),
            payload => Err(format!(
                "visual parameter update returned unexpected payload {payload:?}"
            )),
        }
    }

    fn stop(&mut self) -> Result<VisualAdapterStatus, String> {
        let error = self.request_shutdown(VisualShutdownMode::Graceful, "visual runtime stopped");
        terminate_process(&mut self.process)?;
        self.clear_runtime_handles();
        if let Err(message) = error {
            return Ok(VisualAdapterStatus::Failed { message });
        }
        Ok(VisualAdapterStatus::Stopped)
    }

    fn panic(&mut self) -> Result<VisualAdapterStatus, String> {
        let _ = self.request_shutdown(VisualShutdownMode::Panic, "visual runtime panic requested");
        terminate_process(&mut self.process)?;
        self.clear_runtime_handles();
        Ok(VisualAdapterStatus::Panicked)
    }
}

impl Drop for BevySidecarAdapter {
    fn drop(&mut self) {
        let _ = terminate_process(&mut self.process);
    }
}

fn resolve_bevy_executable() -> Option<PathBuf> {
    if let Some(override_path) = env::var_os(BEVY_OVERRIDE_ENV) {
        let path = PathBuf::from(override_path);
        if path.is_file() {
            return Some(path);
        }
    }

    env::var_os("PATH").and_then(|path_var| {
        env::split_paths(&path_var)
            .map(|entry| entry.join(BEVY_BIN))
            .find(|candidate| is_executable(candidate))
    })
}

fn is_executable(path: &Path) -> bool {
    path.is_file()
}

/// Missing-binary message for the dev/test override path (`SCRYSYNTH_BEVY_PATH`
/// or `with_executable_override_and_args`). Preserves the historical substrings
/// ("visual runtime binary not found", `BEVY_OVERRIDE_ENV`, "scrysynth-visual")
/// so the Phase 8 missing-sidecar test keeps asserting the same intent.
fn missing_sidecar_message_dev(path: Option<&Path>) -> String {
    match path {
        Some(path) => format!(
            "visual runtime binary not found at {}; build scrysynth-visual or set the {BEVY_OVERRIDE_ENV} environment variable to a built binary.",
            path.display()
        ),
        None => format!(
            "visual runtime binary not found; build scrysynth-visual or set the {BEVY_OVERRIDE_ENV} environment variable to a built binary."
        ),
    }
}

/// Missing-binary message for the packaged path, where the bundled sidecar
/// (placed via Tauri `externalBin`) could not be launched. Speaks to an end
/// user (reinstall guidance), not to a developer env-var. `detail` carries the
/// plugin/spawn-layer error for diagnostics.
fn missing_sidecar_message_bundled(detail: impl std::fmt::Display) -> String {
    format!(
        "The bundled visual sidecar ({BEVY_BIN}) could not be launched: {detail}. Reinstall Scrysynth to restore it. (Developers running outside a packaged build can instead set {BEVY_OVERRIDE_ENV} to a built scrysynth-visual binary.)"
    )
}

impl BevySidecarAdapter {
    /// Dev/test path: spawn the visual runtime via raw `std::process::Command`
    /// against either an explicit `executable_override` (tests) or the
    /// `SCRYSYNTH_BEVY_PATH`/PATH-discovered binary (dev). Kept intact so the
    /// Phase 8 `visual_sidecar_uat` integration test and override callers work
    /// without a real Tauri runtime.
    fn start_via_command(&mut self) -> Result<VisualAdapterStatus, String> {
        self.last_spawn_kind = SpawnKind::DevOverride;

        let executable = match self.executable_override.clone() {
            Some(path) if is_executable(&path) => path,
            Some(path) => {
                return Ok(VisualAdapterStatus::Failed {
                    message: missing_sidecar_message_dev(Some(&path)),
                });
            }
            None => match resolve_bevy_executable() {
                Some(path) => path,
                None => {
                    return Ok(VisualAdapterStatus::Failed {
                        message: missing_sidecar_message_dev(None),
                    });
                }
            },
        };

        let mut command = Command::new(executable);
        command.args(&self.executable_args);
        if let Some(mode) = env::var_os("SCRYSYNTH_VISUAL_MODE") {
            if mode == "minimal" {
                command.arg("--minimal");
            }
            command.env("SCRYSYNTH_VISUAL_MODE", mode);
        }

        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| format!("failed to launch visual runtime: {err}"))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "visual runtime stdin was not available".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "visual runtime stdout was not available".to_string())?;
        let responses = spawn_response_reader(stdout);

        self.process = Some(SidecarChild::Std(child));
        self.stdin = Some(stdin);
        self.responses = Some(responses);
        self.terminated.store(false, Ordering::SeqCst);

        self.complete_handshake()
    }

    /// Packaged path: spawn the bundled visual sidecar through Tauri's official
    /// `app.shell().sidecar()` API. The shell plugin resolves the
    /// host-triple-suffixed binary placed by `externalBin`. `--minimal` is
    /// passed explicitly so the GPU-free minimal runtime is the packaged
    /// default regardless of any inherited `SCRYSYNTH_VISUAL_MODE` (D-04,
    /// Pitfall 6). Only the spawn mechanism changes; the JSON-lines protocol
    /// reader/writer is reused unchanged.
    fn start_via_sidecar(&mut self) -> Result<VisualAdapterStatus, String> {
        self.last_spawn_kind = SpawnKind::Bundled;

        let Some(app_handle) = self.app_handle.clone() else {
            return Ok(VisualAdapterStatus::Failed {
                message: missing_sidecar_message_bundled(
                    "no Tauri AppHandle available to resolve the sidecar",
                ),
            });
        };

        let sidecar = app_handle.shell().sidecar(SIDECAR_NAME).map_err(|err| {
            missing_sidecar_message_bundled(format!("failed to resolve sidecar: {err}"))
        })?;
        let (mut rx, child) = sidecar.args(["--minimal"]).spawn().map_err(|err| {
            missing_sidecar_message_bundled(format!("failed to launch bundled sidecar: {err}"))
        })?;

        let (sender, receiver) = mpsc::channel();
        let terminated = self.terminated.clone();
        thread::spawn(move || {
            loop {
                let event = tauri::async_runtime::block_on(async { rx.recv().await });
                match event {
                    Some(CommandEvent::Stdout(bytes)) => {
                        let line = String::from_utf8_lossy(&bytes);
                        if line.trim().is_empty() {
                            continue;
                        }
                        let message = serde_json::from_str::<VisualToAppMessage>(&line)
                            .map_err(|err| format!("invalid visual runtime response: {err}"));
                        if sender.send(message).is_err() {
                            break;
                        }
                    }
                    Some(CommandEvent::Stderr(_)) => {
                        // Sidecar stderr is intentionally ignored at the protocol layer.
                    }
                    Some(CommandEvent::Error(message)) => {
                        let _ = sender.send(Err(format!("visual runtime error: {message}")));
                        terminated.store(true, Ordering::SeqCst);
                        break;
                    }
                    Some(CommandEvent::Terminated(payload)) => {
                        let _ = sender.send(Err(format!(
                            "visual runtime exited with status code {:?}",
                            payload.code
                        )));
                        terminated.store(true, Ordering::SeqCst);
                        break;
                    }
                    // `CommandEvent` is `#[non_exhaustive]`; treat any future
                    // variant as a termination so the adapter stops cleanly.
                    None => {
                        terminated.store(true, Ordering::SeqCst);
                        break;
                    }
                    _ => {
                        terminated.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            }
        });

        self.process = Some(SidecarChild::Plugin(child));
        self.responses = Some(receiver);
        self.stdin = None;
        self.terminated.store(false, Ordering::SeqCst);

        self.complete_handshake()
    }

    /// Handshake against the freshly-spawned child (shared by both spawn
    /// paths). Sends the typed handshake and maps the first response to a
    /// `VisualAdapterStatus`.
    fn complete_handshake(&mut self) -> Result<VisualAdapterStatus, String> {
        let sequence_id = self.next_sequence_id();
        let response = self.send_and_wait(
            AppToVisualMessage::new(
                sequence_id,
                AppToVisualPayload::Handshake(VisualHandshake {
                    app_name: "scrysynth".to_string(),
                    app_version: env!("CARGO_PKG_VERSION").to_string(),
                    session_id: "local-session".to_string(),
                    capabilities: vec![
                        "scene_load".to_string(),
                        "parameter_batch".to_string(),
                        "rendering_status".to_string(),
                        "shutdown".to_string(),
                    ],
                }),
            ),
            sequence_id,
        );

        match response {
            Ok(VisualToAppPayload::Ready(ready)) => Ok(VisualAdapterStatus::Booted {
                renderer: ready.renderer,
            }),
            Ok(VisualToAppPayload::Error(error)) => {
                let message = protocol_error_message("visual runtime handshake failed", &error);
                self.clear_runtime();
                Ok(VisualAdapterStatus::Failed { message })
            }
            Ok(payload) => {
                let message =
                    format!("visual runtime handshake returned unexpected payload {payload:?}");
                self.clear_runtime();
                Ok(VisualAdapterStatus::Failed { message })
            }
            Err(message) => {
                self.clear_runtime();
                Ok(VisualAdapterStatus::Failed { message })
            }
        }
    }

    fn next_sequence_id(&mut self) -> u64 {
        let sequence_id = self.next_sequence_id;
        self.next_sequence_id += 1;
        sequence_id
    }

    fn ensure_running(&mut self) -> Result<(), String> {
        match self.process.as_mut() {
            Some(SidecarChild::Std(child)) => {
                if let Some(status) = child
                    .try_wait()
                    .map_err(|err| format!("failed to poll visual runtime: {err}"))?
                {
                    self.clear_runtime();
                    return Err(format!("visual runtime exited with status {status}"));
                }
                Ok(())
            }
            Some(SidecarChild::Plugin(_)) => {
                if self.terminated.load(Ordering::SeqCst) {
                    self.clear_runtime();
                    return Err("visual runtime process exited".to_string());
                }
                Ok(())
            }
            None => Err("visual runtime process is not running".to_string()),
        }
    }

    /// Write a pre-encoded JSON-lines message to the live child's stdin,
    /// branching on the spawn mechanism. The std path writes through the
    /// captured `ChildStdin`; the sidecar path writes through the shell
    /// plugin's `CommandChild::write`.
    fn write_stdin(&mut self, bytes: &[u8]) -> Result<(), String> {
        match self.process.as_mut() {
            Some(SidecarChild::Plugin(child)) => child
                .write(bytes)
                .map_err(|err| format!("failed to write visual runtime message: {err}")),
            Some(SidecarChild::Std(_)) => {
                let stdin = self
                    .stdin
                    .as_mut()
                    .ok_or_else(|| "visual runtime stdin is not connected".to_string())?;
                stdin
                    .write_all(bytes)
                    .map_err(|err| format!("failed to write visual runtime message: {err}"))?;
                stdin
                    .flush()
                    .map_err(|err| format!("failed to flush visual runtime message: {err}"))
            }
            None => Err("visual runtime process is not running".to_string()),
        }
    }

    fn send_and_wait(
        &mut self,
        message: AppToVisualMessage,
        sequence_id: u64,
    ) -> Result<VisualToAppPayload, String> {
        self.ensure_running()?;

        let mut bytes = serde_json::to_vec(&message)
            .map_err(|err| format!("failed to encode visual runtime message: {err}"))?;
        bytes.push(b'\n');
        self.write_stdin(&bytes)?;

        self.wait_for_response(sequence_id)
    }

    fn wait_for_response(&mut self, sequence_id: u64) -> Result<VisualToAppPayload, String> {
        let deadline = Instant::now() + self.timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                self.clear_runtime();
                return Err(format!(
                    "timed out waiting for visual runtime acknowledgement {sequence_id}"
                ));
            }

            let response = self
                .responses
                .as_ref()
                .ok_or_else(|| "visual runtime stdout is not connected".to_string())?
                .recv_timeout(remaining)
                .map_err(|err| {
                    self.clear_runtime();
                    format!("visual runtime acknowledgement channel closed: {err}")
                })??;

            if response.sequence_id == Some(sequence_id) {
                return Ok(response.payload);
            }
        }
    }

    fn request_shutdown(&mut self, mode: VisualShutdownMode, reason: &str) -> Result<(), String> {
        if self.process.is_none() {
            return Ok(());
        }
        let sequence_id = self.next_sequence_id();
        match self.send_and_wait(
            AppToVisualMessage::new(
                sequence_id,
                AppToVisualPayload::Shutdown(VisualShutdown {
                    mode: mode.clone(),
                    reason: Some(reason.to_string()),
                }),
            ),
            sequence_id,
        )? {
            VisualToAppPayload::ShutdownComplete(done) if done.mode == mode => Ok(()),
            VisualToAppPayload::Error(error) => {
                Err(protocol_error_message("visual shutdown failed", &error))
            }
            payload => Err(format!(
                "visual shutdown returned unexpected payload {payload:?}"
            )),
        }
    }

    fn clear_runtime(&mut self) {
        let _ = terminate_process(&mut self.process);
        self.clear_runtime_handles();
    }

    fn clear_runtime_handles(&mut self) {
        self.stdin = None;
        self.responses = None;
        self.active_scene_id = None;
    }
}

fn spawn_response_reader(
    stdout: std::process::ChildStdout,
) -> Receiver<Result<VisualToAppMessage, String>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let line = match line {
                Ok(line) => line,
                Err(err) => {
                    let _ =
                        sender.send(Err(format!("failed reading visual runtime stdout: {err}")));
                    break;
                }
            };

            if line.trim().is_empty() {
                continue;
            }

            let message = serde_json::from_str::<VisualToAppMessage>(&line)
                .map_err(|err| format!("invalid visual runtime response: {err}"));
            if sender.send(message).is_err() {
                break;
            }
        }
    });
    receiver
}

fn protocol_error_message(context: &str, error: &VisualProtocolError) -> String {
    format!("{context}: {:?}: {}", error.code, error.message)
}

fn terminate_process(process: &mut Option<SidecarChild>) -> Result<(), String> {
    if let Some(child) = process.take() {
        match child {
            SidecarChild::Std(mut child) => {
                if child
                    .try_wait()
                    .map_err(|err| format!("failed to poll visual runtime: {err}"))?
                    .is_none()
                {
                    child
                        .kill()
                        .map_err(|err| format!("failed to stop visual runtime: {err}"))?;
                }
                let _ = child.wait();
            }
            // `CommandChild::kill` consumes and sends the signal; the shell
            // plugin's background waiter observes termination on its own.
            SidecarChild::Plugin(child) => {
                let _ = child.kill();
            }
        }
    }

    *process = None;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visual::compiler::{
        CompiledVisualElement, CompiledVisualScene, VisualParameterUpdate,
    };
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn adapter_handshakes_loads_scene_updates_and_stops() {
        let script = write_sidecar_script(
            r#"#!/usr/bin/env python3
import sys
responses = [
    '{"protocolVersion":1,"sequenceId":1,"payload":{"type":"ready","payload":{"renderer":"fake-renderer","sidecarVersion":"0.1.0","capabilities":["scene_load","parameter_batch","shutdown"]}}}',
    '{"protocolVersion":1,"sequenceId":2,"payload":{"type":"sceneLoaded","payload":{"sceneId":"scene-1"}}}',
    '{"protocolVersion":1,"sequenceId":3,"payload":{"type":"parameterBatchApplied","payload":{"appliedCount":1}}}',
    '{"protocolVersion":1,"sequenceId":4,"payload":{"type":"shutdownComplete","payload":{"mode":"graceful"}}}',
]
for response in responses:
    line = sys.stdin.readline()
    if not line:
        break
    print(response, flush=True)
"#,
        );
        let mut adapter =
            BevySidecarAdapter::with_executable_override(script, Duration::from_secs(2));

        assert_eq!(
            adapter.start().unwrap(),
            VisualAdapterStatus::Booted {
                renderer: "fake-renderer".to_string()
            }
        );
        assert_eq!(
            adapter.load_scene(&test_scene()).unwrap(),
            VisualAdapterStatus::SceneLoaded {
                scene_id: "scene-1".to_string(),
                rendering: false,
            }
        );
        adapter
            .update_parameters(&[VisualParameterUpdate {
                element_id: "node-1".to_string(),
                parameter_id: "gain".to_string(),
                value: 0.75,
            }])
            .unwrap();
        assert_eq!(adapter.stop().unwrap(), VisualAdapterStatus::Stopped);
    }

    #[test]
    fn adapter_reports_handshake_timeout() {
        let script = write_sidecar_script(
            r#"#!/usr/bin/env python3
import time
time.sleep(1)
"#,
        );
        let mut adapter =
            BevySidecarAdapter::with_executable_override(script, Duration::from_millis(50));

        let status = adapter.start().unwrap();

        assert!(matches!(
            status,
            VisualAdapterStatus::Failed { message } if message.contains("timed out waiting")
        ));
    }

    #[test]
    fn adapter_reports_failure_acknowledgement() {
        let script = write_sidecar_script(
            r#"#!/usr/bin/env python3
import sys
sys.stdin.readline()
print('{"protocolVersion":1,"sequenceId":1,"payload":{"type":"error","payload":{"code":"renderer_unavailable","message":"no renderer","recoverable":false}}}', flush=True)
"#,
        );
        let mut adapter =
            BevySidecarAdapter::with_executable_override(script, Duration::from_secs(2));

        let status = adapter.start().unwrap();

        assert!(
            matches!(
                &status,
                VisualAdapterStatus::Failed { message }
                    if message.contains("visual runtime handshake failed")
                        && message.contains("no renderer")
            ),
            "{status:?}"
        );
    }

    #[test]
    fn adapter_reports_missing_configured_sidecar_with_setup_guidance() {
        let missing_path = std::env::temp_dir().join("missing-scrysynth-visual-for-test");
        let mut adapter =
            BevySidecarAdapter::with_executable_override(missing_path, Duration::from_secs(2));

        let status = adapter.start().unwrap();

        assert!(
            matches!(
                &status,
                VisualAdapterStatus::Failed { message }
                    if message.contains("visual runtime binary not found")
                        && message.contains(BEVY_OVERRIDE_ENV)
                        && message.contains("scrysynth-visual")
            ),
            "{status:?}"
        );
    }

    #[test]
    fn adapter_fails_parameter_update_without_loaded_scene() {
        let script = write_sidecar_script(
            r#"#!/usr/bin/env python3
import sys
sys.stdin.readline()
print('{"protocolVersion":1,"sequenceId":1,"payload":{"type":"ready","payload":{"renderer":"fake-renderer","sidecarVersion":"0.1.0","capabilities":[]}}}', flush=True)
for line in sys.stdin:
    pass
"#,
        );
        let mut adapter =
            BevySidecarAdapter::with_executable_override(script, Duration::from_secs(2));
        adapter.start().unwrap();

        let error = adapter
            .update_parameters(&[VisualParameterUpdate {
                element_id: "node-1".to_string(),
                parameter_id: "gain".to_string(),
                value: 0.75,
            }])
            .unwrap_err();

        assert!(error.contains("before a scene has been loaded"));
        let _ = adapter.panic();
    }

    #[test]
    fn adapter_rejects_unapplied_parameter_updates() {
        let script = write_sidecar_script(
            r#"#!/usr/bin/env python3
import sys
responses = [
    '{"protocolVersion":1,"sequenceId":1,"payload":{"type":"ready","payload":{"renderer":"fake-renderer","sidecarVersion":"0.1.0","capabilities":["scene_load","parameter_batch"]}}}',
    '{"protocolVersion":1,"sequenceId":2,"payload":{"type":"sceneLoaded","payload":{"sceneId":"scene-1"}}}',
    '{"protocolVersion":1,"sequenceId":3,"payload":{"type":"parameterBatchApplied","payload":{"appliedCount":0}}}',
]
for response in responses:
    line = sys.stdin.readline()
    if not line:
        break
    print(response, flush=True)
"#,
        );
        let mut adapter =
            BevySidecarAdapter::with_executable_override(script, Duration::from_secs(2));

        adapter.start().unwrap();
        adapter.load_scene(&test_scene()).unwrap();
        let error = adapter
            .update_parameters(&[VisualParameterUpdate {
                element_id: "node-1".to_string(),
                parameter_id: "glow".to_string(),
                value: 0.75,
            }])
            .unwrap_err();

        assert!(error.contains("applied 0 of 1 requested patches"));
        let _ = adapter.panic();
    }

    #[test]
    fn adapter_reports_terminated_process_on_scene_load() {
        let script = write_sidecar_script(
            r#"#!/usr/bin/env python3
import sys
sys.stdin.readline()
print('{"protocolVersion":1,"sequenceId":1,"payload":{"type":"ready","payload":{"renderer":"fake-renderer","sidecarVersion":"0.1.0","capabilities":[]}}}', flush=True)
"#,
        );
        let mut adapter =
            BevySidecarAdapter::with_executable_override(script, Duration::from_secs(2));
        adapter.start().unwrap();

        let result = adapter.load_scene(&test_scene());

        match result {
            Ok(VisualAdapterStatus::Failed { message }) => {
                assert!(
                    message.contains("exited") || message.contains("closed"),
                    "{message}"
                );
            }
            Err(message) => {
                assert!(
                    message.contains("exited")
                        || message.contains("closed")
                        || message.contains("not running"),
                    "{message}"
                );
            }
            other => panic!("expected terminated process failure, got {other:?}"),
        }
    }

    fn test_scene() -> CompiledVisualScene {
        CompiledVisualScene {
            scene_id: "scene-1".to_string(),
            background_color: [0.0, 0.0, 0.0, 1.0],
            elements: vec![CompiledVisualElement {
                element_id: "node-1".to_string(),
                element_type: "sphere".to_string(),
                position: [0.0, 0.0],
                scale: 1.0,
                parameters: vec![("gain".to_string(), 0.5)],
            }],
        }
    }

    fn write_sidecar_script(contents: &str) -> PathBuf {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("fake-scrysynth-visual");
        fs::write(&path, contents).expect("script is written");
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        std::mem::forget(dir);
        path
    }
}
