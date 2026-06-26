use std::collections::HashSet;

use crate::domain::session::{MacroTarget, SessionDocument};

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
    let active_scene = active_visual_scene(session);
    let scene_id = active_scene
        .map(|scene| scene.id.clone())
        .or_else(|| session.scenes.first().map(|scene| scene.id.clone()))
        .unwrap_or_default();
    let active_node_ids = active_scene.map(|scene| {
        scene
            .active_node_ids
            .iter()
            .map(|id| id.as_str())
            .collect::<HashSet<_>>()
    });
    let visual_overrides = active_scene
        .map(|scene| visual_updates_for_macro_overrides(session, &scene.macro_overrides))
        .unwrap_or_default();

    let elements: Vec<CompiledVisualElement> = session
        .nodes
        .iter()
        .filter(|node| {
            active_node_ids
                .as_ref()
                .map(|ids| ids.contains(node.id.as_str()))
                .unwrap_or(node.enabled)
        })
        .enumerate()
        .map(|(index, node)| {
            // Catalog-driven visual shape (replaces v1's `match node.node_type`).
            // Graceful default keeps `compile_session_to_visual_scene` non-Result
            // (its current signature); unknown ids render as a plain box.
            let element_type = crate::catalog::find_catalog_entry(&node.node_type_id)
                .map(|entry| entry.visual_shape)
                .unwrap_or("box");

            CompiledVisualElement {
                element_id: node.id.clone(),
                element_type: element_type.to_string(),
                position: [index as f32 * 2.0, 0.0],
                scale: 1.0,
                parameters: merged_parameters(
                    node.parameters
                        .iter()
                        .map(|p| (p.id.clone(), p.value))
                        .collect(),
                    visual_overrides
                        .iter()
                        .filter(|update| update.element_id == node.id)
                        .cloned()
                        .collect(),
                ),
            }
        })
        .collect();

    CompiledVisualScene {
        scene_id,
        background_color: [0.0, 0.0, 0.0, 1.0],
        elements,
    }
}

pub fn visual_updates_for_macro_value(
    session: &SessionDocument,
    macro_id: &str,
    value: f64,
) -> Vec<VisualParameterUpdate> {
    let Some(macro_def) = session
        .macros
        .iter()
        .find(|macro_def| macro_def.id == macro_id)
    else {
        return Vec::new();
    };
    let clamped_input = value.clamp(0.0, 1.0);
    let scaled_value =
        macro_def.range_start + (clamped_input * (macro_def.range_end - macro_def.range_start));

    macro_def
        .targets
        .iter()
        .filter_map(|target| match target {
            MacroTarget::VisualParameter {
                element_id,
                parameter_id,
            } => Some(VisualParameterUpdate {
                element_id: element_id.clone(),
                parameter_id: parameter_id.clone(),
                value: scaled_value,
            }),
            MacroTarget::AudioParameter { .. } => None,
        })
        .collect()
}

fn visual_updates_for_macro_overrides(
    session: &SessionDocument,
    overrides: &[crate::domain::session::MacroOverride],
) -> Vec<VisualParameterUpdate> {
    overrides
        .iter()
        .flat_map(|macro_override| {
            visual_updates_for_macro_value(session, &macro_override.macro_id, macro_override.value)
        })
        .collect()
}

fn active_visual_scene(
    session: &SessionDocument,
) -> Option<&crate::domain::session::SceneDefinition> {
    if let Some(active_scene_id) = &session.visual_runtime.active_scene_id {
        if let Some(scene) = session
            .scenes
            .iter()
            .find(|scene| scene.id == *active_scene_id)
        {
            return Some(scene);
        }
    }
    None
}

fn merged_parameters(
    mut parameters: Vec<(String, f64)>,
    updates: Vec<VisualParameterUpdate>,
) -> Vec<(String, f64)> {
    for update in updates {
        if let Some((_, value)) = parameters
            .iter_mut()
            .find(|(parameter_id, _)| *parameter_id == update.parameter_id)
        {
            *value = update.value;
        } else {
            parameters.push((update.parameter_id, update.value));
        }
    }
    parameters
}
