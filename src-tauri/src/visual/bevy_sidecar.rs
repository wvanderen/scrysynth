use std::env;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

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

#[derive(Debug)]
pub struct BevySidecarAdapter {
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    responses: Option<Receiver<Result<VisualToAppMessage, String>>>,
    next_sequence_id: u64,
    active_scene_id: Option<String>,
    timeout: Duration,
    executable_override: Option<PathBuf>,
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
        }
    }
}

impl BevySidecarAdapter {
    #[cfg(test)]
    fn new_for_tests(executable: PathBuf, timeout: Duration) -> Self {
        let mut adapter = Self::default();
        adapter.executable_override = Some(executable);
        adapter.timeout = timeout;
        adapter
    }
}

impl VisualRuntimeAdapter for BevySidecarAdapter {
    fn start(&mut self) -> Result<VisualAdapterStatus, String> {
        if self.process.is_some() {
            return Ok(VisualAdapterStatus::Booted {
                renderer: "bevy".to_string(),
            });
        }

        let executable = match self
            .executable_override
            .clone()
            .or_else(resolve_bevy_executable)
        {
            Some(path) => path,
            None => {
                return Ok(VisualAdapterStatus::Failed {
                    message: format!(
                        "visual runtime binary not found; install scrysynth-visual or set {BEVY_OVERRIDE_ENV}"
                    ),
                });
            }
        };

        let mut child = Command::new(executable)
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

        self.process = Some(child);
        self.stdin = Some(stdin);
        self.responses = Some(responses);

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

impl BevySidecarAdapter {
    fn next_sequence_id(&mut self) -> u64 {
        let sequence_id = self.next_sequence_id;
        self.next_sequence_id += 1;
        sequence_id
    }

    fn ensure_running(&mut self) -> Result<(), String> {
        if let Some(child) = self.process.as_mut() {
            if let Some(status) = child
                .try_wait()
                .map_err(|err| format!("failed to poll visual runtime: {err}"))?
            {
                self.clear_runtime();
                return Err(format!("visual runtime exited with status {status}"));
            }
            return Ok(());
        }

        Err("visual runtime process is not running".to_string())
    }

    fn send_and_wait(
        &mut self,
        message: AppToVisualMessage,
        sequence_id: u64,
    ) -> Result<VisualToAppPayload, String> {
        self.ensure_running()?;
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| "visual runtime stdin is not connected".to_string())?;

        serde_json::to_writer(&mut *stdin, &message)
            .map_err(|err| format!("failed to encode visual runtime message: {err}"))?;
        stdin
            .write_all(b"\n")
            .map_err(|err| format!("failed to write visual runtime message: {err}"))?;
        stdin
            .flush()
            .map_err(|err| format!("failed to flush visual runtime message: {err}"))?;

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

fn terminate_process(process: &mut Option<Child>) -> Result<(), String> {
    if let Some(child) = process.as_mut() {
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
        let mut adapter = BevySidecarAdapter::new_for_tests(script, Duration::from_secs(2));

        assert_eq!(
            adapter.start().unwrap(),
            VisualAdapterStatus::Booted {
                renderer: "fake-renderer".to_string()
            }
        );
        assert_eq!(
            adapter.load_scene(&test_scene()).unwrap(),
            VisualAdapterStatus::SceneLoaded {
                scene_id: "scene-1".to_string()
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
        let mut adapter = BevySidecarAdapter::new_for_tests(script, Duration::from_millis(50));

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
        let mut adapter = BevySidecarAdapter::new_for_tests(script, Duration::from_secs(2));

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
        let mut adapter = BevySidecarAdapter::new_for_tests(script, Duration::from_secs(2));
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
        let mut adapter = BevySidecarAdapter::new_for_tests(script, Duration::from_secs(2));

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
        let mut adapter = BevySidecarAdapter::new_for_tests(script, Duration::from_secs(2));
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
