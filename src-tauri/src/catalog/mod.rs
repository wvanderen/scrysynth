//! Data-driven node catalog — the single source of truth for node identity,
//! SynthDef mapping, ports, parameters, and visual shape.
//!
//! Adding a new node type is a data edit in [`entries::CATALOG`]; it must NEVER
//! require a new `match` arm in the compiler, synthdefs planner, or visual
//! compiler. An unknown `node_type_id` resolves to
//! [`ScResourcePlanError::UnknownCatalogEntry`] via [`find_catalog_entry`],
//! replacing the v1 `unreachable!()` panic (success criterion #3).
//!
//! Design notes (see `.planning/phases/12-node-catalog-foundation/12-RESEARCH.md`):
//! - Compiled-in `&'static` const table; linear scan over ~16 entries is trivial.
//!   No `phf` / `BTreeMap` builder (project favors stable primitives).
//! - [`CatalogPortSpec`] / [`CatalogParamSpec`] reuse [`PortDirection`] and
//!   [`SignalType`] from [`crate::domain::session`] (never redefined here).
//! - Per-parameter CV-input ports are declared in the catalog (D-04/D-05):
//!   every continuous parameter has a sibling CV-input port and
//!   `exposes_cv_port: true`; discrete selectors and toggles do not.

mod entries;

use serde::Serialize;
use ts_rs::TS;

use crate::audio::synthdefs::ScResourcePlanError;
use crate::domain::session::{PortDirection, SignalType};
pub use entries::CATALOG;

/// Semantic node category used for topology sort order and visual grouping.
///
/// This is NOT the closed identity enum (node identity is the string
/// `node_type_id`). It replaces v1's `NodeType` rank logic in the compiler.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum NodeCategory {
    Source,
    Modulator,
    Effect,
    Utility,
    Sequencer,
    Mixer,
    Output,
}

impl NodeCategory {
    /// Stable rank for deterministic topology sort (matches the v1
    /// source→effect→mixer→output launch order, extended for the new
    /// modulator/utility/sequencer families).
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Source => 0,
            Self::Modulator => 1,
            Self::Effect => 2,
            Self::Utility => 3,
            Self::Sequencer => 4,
            Self::Mixer => 5,
            Self::Output => 6,
        }
    }
}

/// A port declared by a catalog entry (audio in/out or per-parameter CV input).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CatalogPortSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub direction: PortDirection,
    pub signal_type: SignalType,
}

/// A parameter declared by a catalog entry.
///
/// `exposes_cv_port` follows D-05: `true` for continuous parameters
/// (frequency, cutoff, level, …), `false` for discrete selectors and toggles.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CatalogParamSpec {
    pub id: &'static str,
    /// SuperCollider synth arg name — replaces v1's `normalize_parameter_name`.
    pub sc_arg: &'static str,
    /// Backward-compatible parameter aliases accepted at the command boundary.
    pub aliases: &'static [&'static str],
    pub default_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub unit: &'static str,
    /// D-05: continuous params expose a CV-input port; selectors/toggles do not.
    pub exposes_cv_port: bool,
    /// sibling CV-input port id (e.g. `"cutoff_cv"`) when `exposes_cv_port`.
    pub cv_port_id: Option<&'static str>,
}

/// One entry in the compiled-in node catalog — the single source of truth.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct NodeCatalogEntry {
    /// Canonical node identity (e.g. `"oscillator"`, `"filter"`, `"step_sequencer"`).
    pub id: &'static str,
    pub display_name: &'static str,
    pub category: NodeCategory,
    /// SuperCollider SynthDef name (empty for app-driven nodes like the sequencer).
    pub synthdef_name: &'static str,
    /// Resource path relative to the crate manifest
    /// (e.g. `"resources/synthdefs/v2/scrysynth_v2_oscillator.scsyndef"`).
    pub synthdef_resource: &'static str,
    /// audio in/out + per-parameter CV-input ports (D-04/D-05).
    pub ports: &'static [CatalogPortSpec],
    pub parameters: &'static [CatalogParamSpec],
    /// Visual shape consumed by the visual compiler (Phase 15).
    pub visual_shape: &'static str,
}

impl NodeCatalogEntry {
    /// Returns the SC arg name for a parameter id/alias, or `None` if unknown.
    ///
    /// Replaces v1's `normalize_parameter_name` allowlist.
    #[must_use]
    pub fn resolve_sc_arg(&self, name: &str) -> Option<&'static str> {
        self.parameters
            .iter()
            .find(|param| param.id == name || param.aliases.contains(&name))
            .map(|param| param.sc_arg)
    }

    /// Returns the CV-input port declared for a continuous parameter, if any.
    #[must_use]
    pub fn cv_port_for_param(&self, param_id: &str) -> Option<&'static CatalogPortSpec> {
        let cv_port_id = self
            .parameters
            .iter()
            .find(|param| param.id == param_id && param.exposes_cv_port)
            .and_then(|param| param.cv_port_id)?;
        self.ports.iter().find(|port| port.id == cv_port_id)
    }
}

/// Resolve a `node_type_id` to its catalog entry.
///
/// Returns [`ScResourcePlanError::UnknownCatalogEntry`] on miss — this is the
/// single lookup that replaces every former closed-enum `match` dispatch in the
/// audio compiler, synthdef planner, and visual compiler. An unknown id becomes
/// a real `Err`, never a panic (success criterion #3).
///
/// # Errors
/// `UnknownCatalogEntry { node_type_id }` when `id` is not in [`CATALOG`].
pub fn find_catalog_entry(id: &str) -> Result<&'static NodeCatalogEntry, ScResourcePlanError> {
    CATALOG
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| ScResourcePlanError::UnknownCatalogEntry {
            node_type_id: id.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_entries_have_unique_ids() {
        let mut ids: Vec<&str> = CATALOG.iter().map(|entry| entry.id).collect();
        ids.sort_unstable();
        let duplicates: Vec<&str> = ids
            .windows(2)
            .filter(|window| window[0] == window[1])
            .map(|window| window[0])
            .collect();
        assert!(duplicates.is_empty(), "duplicate catalog ids: {duplicates:?}");
    }

    #[test]
    fn find_catalog_entry_returns_entry_for_known_id() {
        let entry = find_catalog_entry("oscillator").expect("oscillator is cataloged");
        assert_eq!(entry.id, "oscillator");
        assert_eq!(entry.category, NodeCategory::Source);
    }

    #[test]
    fn find_catalog_entry_errors_for_unknown_id() {
        let error = find_catalog_entry("definitely-not-a-node").expect_err("unknown id errors");
        assert!(matches!(
            error,
            ScResourcePlanError::UnknownCatalogEntry { node_type_id } if node_type_id == "definitely-not-a-node"
        ));
    }

    #[test]
    fn continuous_parameters_declare_a_cv_port() {
        // D-04/D-05 invariant: every param with exposes_cv_port must reference a
        // real CV-input port id that exists on the entry's port list.
        for entry in CATALOG {
            for param in entry.parameters {
                if param.exposes_cv_port {
                    let cv_port_id = param
                        .cv_port_id
                        .expect("exposes_cv_port params must name a cv_port_id");
                    assert!(
                        entry.ports.iter().any(|port| port.id == cv_port_id),
                        "entry `{}` param `{}` declares cv_port_id `{cv_port_id}` not in its ports",
                        entry.id,
                        param.id,
                    );
                    assert!(
                        entry
                            .ports
                            .iter()
                            .find(|port| port.id == cv_port_id)
                            .is_some_and(|port| port.direction == PortDirection::Input),
                        "cv port `{cv_port_id}` on entry `{}` must be an input",
                        entry.id,
                    );
                }
            }
        }
    }

    #[test]
    fn step_sequencer_has_no_synthdef() {
        // D-06: SC stays dumb — the sequencer is app-driven, no SynthDef.
        let entry = find_catalog_entry("step_sequencer").expect("sequencer cataloged");
        assert_eq!(entry.category, NodeCategory::Sequencer);
        assert!(entry.synthdef_name.is_empty());
        assert!(entry.synthdef_resource.is_empty());
    }
}
