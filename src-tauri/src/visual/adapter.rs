#[derive(Clone, Debug, PartialEq)]
pub enum VisualAdapterStatus {
    Booted { renderer: String },
    SceneLoaded { scene_id: String, rendering: bool },
    Stopped,
    Panicked,
    Failed { message: String },
}

pub trait VisualRuntimeAdapter {
    fn start(&mut self) -> Result<VisualAdapterStatus, String>;
    fn load_scene(
        &mut self,
        scene: &crate::visual::compiler::CompiledVisualScene,
    ) -> Result<VisualAdapterStatus, String>;
    fn update_parameters(
        &mut self,
        params: &[crate::visual::compiler::VisualParameterUpdate],
    ) -> Result<(), String>;
    fn stop(&mut self) -> Result<VisualAdapterStatus, String>;
    fn panic(&mut self) -> Result<VisualAdapterStatus, String>;

    /// Provide the Tauri `AppHandle` so the adapter can launch a bundled
    /// sidecar through `tauri_plugin_shell`'s `app.shell().sidecar()` API.
    /// The default is a no-op so that adapters used in unit tests or dev runs
    /// without a real Tauri runtime (which spawn via raw `std::process::Command`)
    /// keep compiling unchanged.
    #[allow(unused_variables)]
    fn set_app_handle(&mut self, handle: tauri::AppHandle) {}
}
