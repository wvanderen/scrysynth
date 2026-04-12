use std::env;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use crate::visual::adapter::VisualAdapterStatus;
use crate::visual::compiler::{CompiledVisualScene, VisualParameterUpdate};

use super::adapter::VisualRuntimeAdapter;

const BEVY_OVERRIDE_ENV: &str = "SCRYSYNTH_BEVY_PATH";
const BEVY_BIN: &str = "scrysynth-visual";

#[derive(Debug, Default)]
pub struct BevySidecarAdapter {
    process: Option<Child>,
}

impl VisualRuntimeAdapter for BevySidecarAdapter {
    fn start(&mut self) -> Result<VisualAdapterStatus, String> {
        if self.process.is_some() {
            return Ok(VisualAdapterStatus::Booted {
                renderer: "bevy".to_string(),
            });
        }

        let executable = match resolve_bevy_executable() {
            Some(path) => path,
            None => {
                return Ok(VisualAdapterStatus::Failed {
                    message: format!(
                        "visual runtime binary not found; install scrysynth-visual or set {BEVY_OVERRIDE_ENV}"
                    ),
                });
            }
        };

        let child = Command::new(executable)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| format!("failed to launch visual runtime: {err}"))?;

        self.process = Some(child);

        Ok(VisualAdapterStatus::Booted {
            renderer: "bevy".to_string(),
        })
    }

    fn load_scene(&mut self, scene: &CompiledVisualScene) -> Result<VisualAdapterStatus, String> {
        Ok(VisualAdapterStatus::SceneLoaded {
            scene_id: scene.scene_id.clone(),
        })
    }

    fn update_parameters(&mut self, _params: &[VisualParameterUpdate]) -> Result<(), String> {
        Ok(())
    }

    fn stop(&mut self) -> Result<VisualAdapterStatus, String> {
        terminate_process(&mut self.process)?;
        Ok(VisualAdapterStatus::Stopped)
    }

    fn panic(&mut self) -> Result<VisualAdapterStatus, String> {
        terminate_process(&mut self.process)?;
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

fn terminate_process(process: &mut Option<Child>) -> Result<(), String> {
    if let Some(child) = process.as_mut() {
        child
            .kill()
            .map_err(|err| format!("failed to stop visual runtime: {err}"))?;
        let _ = child.wait();
    }

    *process = None;
    Ok(())
}
