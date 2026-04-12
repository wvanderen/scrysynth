use std::env;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use crate::audio::compiler::CompiledTopology;
use crate::audio::runtime_manager::{AudioRuntimeAdapter, RuntimeAdapterStatus};

const SCSYNTH_OVERRIDE_ENV: &str = "SCRYSYNTH_SCSYNTH_PATH";
const SCSYNTH_BIN: &str = "scsynth";

#[derive(Debug, Default)]
pub struct SuperColliderAdapter {
    process: Option<Child>,
}

impl AudioRuntimeAdapter for SuperColliderAdapter {
    fn start(&mut self) -> Result<RuntimeAdapterStatus, String> {
        if self.process.is_some() {
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
            .args(["-u", "57110", "-l", "1"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| format!("failed to launch scsynth: {err}"))?;

        self.process = Some(child);

        Ok(RuntimeAdapterStatus::Booted {
            sample_rate_hz: 48_000,
            block_size: 64,
        })
    }

    fn load_topology(
        &mut self,
        topology: &CompiledTopology,
    ) -> Result<RuntimeAdapterStatus, String> {
        Ok(RuntimeAdapterStatus::Ready {
            active_patch_id: format!("patch-{}", topology.node_launch_order.len()),
        })
    }

    fn stop(&mut self) -> Result<RuntimeAdapterStatus, String> {
        terminate_process(&mut self.process)?;
        Ok(RuntimeAdapterStatus::Stopped)
    }

    fn panic(&mut self) -> Result<RuntimeAdapterStatus, String> {
        terminate_process(&mut self.process)?;
        Ok(RuntimeAdapterStatus::Panicked)
    }
}

impl Drop for SuperColliderAdapter {
    fn drop(&mut self) {
        let _ = terminate_process(&mut self.process);
    }
}

fn resolve_scsynth_executable() -> Option<PathBuf> {
    if let Some(override_path) = env::var_os(SCSYNTH_OVERRIDE_ENV) {
        let path = PathBuf::from(override_path);
        if path.is_file() {
            return Some(path);
        }
    }

    env::var_os("PATH").and_then(|path_var| {
        env::split_paths(&path_var)
            .map(|entry| entry.join(SCSYNTH_BIN))
            .find(|candidate| is_executable(candidate))
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
