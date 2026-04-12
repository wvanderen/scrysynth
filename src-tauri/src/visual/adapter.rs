#[derive(Clone, Debug, PartialEq)]
pub enum VisualAdapterStatus {
    Booted { renderer: String },
    SceneLoaded { scene_id: String },
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
}
