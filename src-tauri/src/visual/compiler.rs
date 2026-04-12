use crate::domain::session::SessionDocument;

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledVisualScene {
    pub scene_id: String,
    pub background_color: [f32; 4],
    pub elements: Vec<CompiledVisualElement>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledVisualElement {
    pub element_id: String,
    pub element_type: String,
    pub position: [f32; 2],
    pub scale: f32,
    pub parameters: Vec<(String, f64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VisualParameterUpdate {
    pub element_id: String,
    pub parameter_id: String,
    pub value: f64,
}

pub fn compile_session_to_visual_scene(session: &SessionDocument) -> CompiledVisualScene {
    let scene_id = session
        .scenes
        .first()
        .map(|s| s.id.clone())
        .unwrap_or_default();

    let elements: Vec<CompiledVisualElement> = session
        .nodes
        .iter()
        .filter(|node| node.enabled)
        .enumerate()
        .map(|(index, node)| {
            let element_type = match node.node_type {
                crate::domain::session::NodeType::Source => "sphere",
                crate::domain::session::NodeType::Effect => "box",
                crate::domain::session::NodeType::Mixer => "ring",
                crate::domain::session::NodeType::Output => "plane",
            };

            CompiledVisualElement {
                element_id: node.id.clone(),
                element_type: element_type.to_string(),
                position: [index as f32 * 2.0, 0.0],
                scale: 1.0,
                parameters: node
                    .parameters
                    .iter()
                    .map(|p| (p.id.clone(), p.value))
                    .collect(),
            }
        })
        .collect();

    CompiledVisualScene {
        scene_id,
        background_color: [0.0, 0.0, 0.0, 1.0],
        elements,
    }
}
