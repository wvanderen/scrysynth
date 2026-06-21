use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::application::agent_command::parse_agent_intent;
use crate::domain::session::{
    ActionHistoryEntry, AgentIntent, AudioRuntimeState, Bus, MacroDefinition, Node, OwnershipRule,
    PendingAction, Route, RuntimeStatusRef, SceneDefinition, SessionDocument, TypedCommand,
    VisualRuntimeState,
};

pub const DEFAULT_CONTEXT_NODE_LIMIT: usize = 48;
pub const DEFAULT_CONTEXT_ROUTE_LIMIT: usize = 96;
pub const DEFAULT_CONTEXT_BUS_LIMIT: usize = 32;
pub const DEFAULT_CONTEXT_MACRO_LIMIT: usize = 32;
pub const DEFAULT_CONTEXT_SCENE_LIMIT: usize = 24;
pub const DEFAULT_CONTEXT_OWNERSHIP_RULE_LIMIT: usize = 32;
pub const DEFAULT_CONTEXT_PENDING_ACTION_LIMIT: usize = 16;
pub const DEFAULT_CONTEXT_HISTORY_LIMIT: usize = 12;
pub const MAX_PROVIDER_ERROR_DETAIL_CHARS: usize = 512;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionContextBounds {
    pub max_nodes: usize,
    pub max_routes: usize,
    pub max_buses: usize,
    pub max_macros: usize,
    pub max_scenes: usize,
    pub max_ownership_rules: usize,
    pub max_pending_actions: usize,
    pub max_history_entries: usize,
}

impl Default for SessionContextBounds {
    fn default() -> Self {
        Self {
            max_nodes: DEFAULT_CONTEXT_NODE_LIMIT,
            max_routes: DEFAULT_CONTEXT_ROUTE_LIMIT,
            max_buses: DEFAULT_CONTEXT_BUS_LIMIT,
            max_macros: DEFAULT_CONTEXT_MACRO_LIMIT,
            max_scenes: DEFAULT_CONTEXT_SCENE_LIMIT,
            max_ownership_rules: DEFAULT_CONTEXT_OWNERSHIP_RULE_LIMIT,
            max_pending_actions: DEFAULT_CONTEXT_PENDING_ACTION_LIMIT,
            max_history_entries: DEFAULT_CONTEXT_HISTORY_LIMIT,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionContextPacket {
    pub session_id: String,
    pub title: String,
    pub graph: GraphContext,
    pub scenes: Vec<SceneDefinition>,
    pub macros: Vec<MacroDefinition>,
    pub ownership: OwnershipContext,
    pub pending_actions: Vec<PendingAction>,
    pub runtime_health: RuntimeHealthContext,
    pub history: HistoryContext,
    pub truncation: ContextTruncation,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphContext {
    pub nodes: Vec<Node>,
    pub routes: Vec<Route>,
    pub buses: Vec<Bus>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnershipContext {
    pub rules: Vec<OwnershipRule>,
    pub locked_node_ids: Vec<String>,
    pub user_owned_node_ids: Vec<String>,
    pub agent_owned_node_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeHealthContext {
    pub audio: AudioRuntimeState,
    pub visual: VisualRuntimeState,
    pub runtime_status: Vec<RuntimeStatusRef>,
    pub agent_available: bool,
    pub agent_frozen: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryContext {
    pub summaries: Vec<ActionHistorySummary>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionHistorySummary {
    pub id: String,
    pub timestamp: String,
    pub actor_id: String,
    pub description: String,
    pub affected_node_ids: Vec<String>,
}

impl From<&ActionHistoryEntry> for ActionHistorySummary {
    fn from(entry: &ActionHistoryEntry) -> Self {
        Self {
            id: entry.id.clone(),
            timestamp: entry.timestamp.clone(),
            actor_id: entry.actor.actor_id.clone(),
            description: entry.diff.description.clone(),
            affected_node_ids: entry.diff.affected_node_ids.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextTruncation {
    pub total_nodes: usize,
    pub included_nodes: usize,
    pub total_routes: usize,
    pub included_routes: usize,
    pub total_buses: usize,
    pub included_buses: usize,
    pub total_macros: usize,
    pub included_macros: usize,
    pub total_scenes: usize,
    pub included_scenes: usize,
    pub total_ownership_rules: usize,
    pub included_ownership_rules: usize,
    pub total_pending_actions: usize,
    pub included_pending_actions: usize,
    pub total_history_entries: usize,
    pub included_history_entries: usize,
}

pub fn derive_session_context_packet(
    session: &SessionDocument,
    bounds: SessionContextBounds,
) -> SessionContextPacket {
    let nodes = take_bounded(&session.nodes, bounds.max_nodes);
    let routes = take_bounded(&session.routes, bounds.max_routes);
    let buses = take_bounded(&session.buses, bounds.max_buses);
    let macros = take_bounded(&session.macros, bounds.max_macros);
    let scenes = take_bounded(&session.scenes, bounds.max_scenes);
    let ownership_rules = take_bounded(&session.ownership_rules, bounds.max_ownership_rules);
    let pending_actions = take_recent_bounded(&session.pending_actions, bounds.max_pending_actions);
    let history_entries = take_recent_bounded(&session.action_history, bounds.max_history_entries);
    let history_summaries = history_entries
        .iter()
        .map(ActionHistorySummary::from)
        .collect::<Vec<_>>();

    SessionContextPacket {
        session_id: session.session_id.clone(),
        title: session.title.clone(),
        graph: GraphContext {
            nodes,
            routes,
            buses,
        },
        scenes,
        macros,
        ownership: OwnershipContext {
            rules: ownership_rules,
            locked_node_ids: session
                .nodes
                .iter()
                .filter(|node| node.ownership.is_locked)
                .map(|node| node.id.clone())
                .collect(),
            user_owned_node_ids: session
                .nodes
                .iter()
                .filter(|node| {
                    node.ownership.controller == crate::domain::session::ControllerKind::User
                })
                .map(|node| node.id.clone())
                .collect(),
            agent_owned_node_ids: session
                .nodes
                .iter()
                .filter(|node| {
                    node.ownership.controller == crate::domain::session::ControllerKind::Agent
                })
                .map(|node| node.id.clone())
                .collect(),
        },
        pending_actions,
        runtime_health: RuntimeHealthContext {
            audio: session.audio_runtime.clone(),
            visual: session.visual_runtime.clone(),
            runtime_status: session.runtime_status.clone(),
            agent_available: session.agent_runtime.is_available,
            agent_frozen: session.agent_frozen || session.agent_runtime.is_frozen,
        },
        history: HistoryContext {
            summaries: history_summaries,
        },
        truncation: ContextTruncation {
            total_nodes: session.nodes.len(),
            included_nodes: session.nodes.len().min(bounds.max_nodes),
            total_routes: session.routes.len(),
            included_routes: session.routes.len().min(bounds.max_routes),
            total_buses: session.buses.len(),
            included_buses: session.buses.len().min(bounds.max_buses),
            total_macros: session.macros.len(),
            included_macros: session.macros.len().min(bounds.max_macros),
            total_scenes: session.scenes.len(),
            included_scenes: session.scenes.len().min(bounds.max_scenes),
            total_ownership_rules: session.ownership_rules.len(),
            included_ownership_rules: session
                .ownership_rules
                .len()
                .min(bounds.max_ownership_rules),
            total_pending_actions: session.pending_actions.len(),
            included_pending_actions: session
                .pending_actions
                .len()
                .min(bounds.max_pending_actions),
            total_history_entries: session.action_history.len(),
            included_history_entries: session.action_history.len().min(bounds.max_history_entries),
        },
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlannerRequest {
    pub raw_input: String,
    pub context: SessionContextPacket,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlannerProposal {
    pub raw_input: String,
    pub commands: Vec<TypedCommand>,
    pub confidence: f64,
    pub provider_id: String,
}

impl PlannerProposal {
    pub fn into_intent(self) -> AgentIntent {
        AgentIntent {
            raw_input: self.raw_input,
            parsed_commands: self.commands,
            confidence: self.confidence,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlannerProviderOutput {
    Typed(PlannerProposal),
    Json(String),
}

pub trait PlannerProvider {
    fn provider_id(&self) -> &str;
    fn is_available(&self) -> bool;
    fn plan(&self, request: &PlannerRequest)
        -> Result<PlannerProviderOutput, PlannerProviderError>;
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum PlannerProviderError {
    #[error("planner provider '{provider_id}' is unavailable: {reason}")]
    Unavailable { provider_id: String, reason: String },
    #[error("planner provider '{provider_id}' failed: {detail}")]
    ProviderFailed { provider_id: String, detail: String },
    #[error("planner provider '{provider_id}' returned invalid output: {detail}")]
    InvalidOutput { provider_id: String, detail: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlannerProposalWire {
    raw_input: Option<String>,
    commands: Vec<TypedCommand>,
    confidence: f64,
}

pub fn plan_agent_request(
    session: &SessionDocument,
    raw_input: &str,
    provider: &dyn PlannerProvider,
    bounds: SessionContextBounds,
) -> Result<PlannerProposal, PlannerProviderError> {
    if !session.agent_runtime.is_available {
        return Err(PlannerProviderError::Unavailable {
            provider_id: provider.provider_id().to_string(),
            reason: "session agent runtime is unavailable".to_string(),
        });
    }

    if !provider.is_available() {
        return Err(PlannerProviderError::Unavailable {
            provider_id: provider.provider_id().to_string(),
            reason: "provider is not available".to_string(),
        });
    }

    let context = derive_session_context_packet(session, bounds);
    let request = PlannerRequest {
        raw_input: raw_input.to_string(),
        context,
    };

    match provider.plan(&request)? {
        PlannerProviderOutput::Typed(proposal) => Ok(proposal),
        PlannerProviderOutput::Json(json) => {
            parse_planner_json(provider.provider_id(), raw_input, &json)
        }
    }
}

fn parse_planner_json(
    provider_id: &str,
    fallback_raw_input: &str,
    json: &str,
) -> Result<PlannerProposal, PlannerProviderError> {
    let parsed: PlannerProposalWire =
        serde_json::from_str(json).map_err(|err| PlannerProviderError::InvalidOutput {
            provider_id: provider_id.to_string(),
            detail: trim_error_detail(&err.to_string()),
        })?;

    Ok(PlannerProposal {
        raw_input: parsed
            .raw_input
            .unwrap_or_else(|| fallback_raw_input.to_string()),
        commands: parsed.commands,
        confidence: parsed.confidence,
        provider_id: provider_id.to_string(),
    })
}

fn trim_error_detail(detail: &str) -> String {
    detail
        .chars()
        .take(MAX_PROVIDER_ERROR_DETAIL_CHARS)
        .collect()
}

fn take_bounded<T: Clone>(items: &[T], limit: usize) -> Vec<T> {
    items.iter().take(limit).cloned().collect()
}

fn take_recent_bounded<T: Clone>(items: &[T], limit: usize) -> Vec<T> {
    items
        .iter()
        .skip(items.len().saturating_sub(limit))
        .cloned()
        .collect()
}

#[derive(Clone, Debug)]
pub struct ParserPlannerProvider {
    provider_id: String,
    available: bool,
}

impl Default for ParserPlannerProvider {
    fn default() -> Self {
        Self {
            provider_id: "local-parser".to_string(),
            available: true,
        }
    }
}

impl ParserPlannerProvider {
    pub fn unavailable(provider_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            available: false,
        }
    }
}

impl PlannerProvider for ParserPlannerProvider {
    fn provider_id(&self) -> &str {
        &self.provider_id
    }

    fn is_available(&self) -> bool {
        self.available
    }

    fn plan(
        &self,
        request: &PlannerRequest,
    ) -> Result<PlannerProviderOutput, PlannerProviderError> {
        let session = SessionDocument {
            session_id: request.context.session_id.clone(),
            title: request.context.title.clone(),
            nodes: request.context.graph.nodes.clone(),
            routes: request.context.graph.routes.clone(),
            buses: request.context.graph.buses.clone(),
            macros: request.context.macros.clone(),
            scenes: request.context.scenes.clone(),
            ownership_rules: request.context.ownership.rules.clone(),
            runtime_status: request.context.runtime_health.runtime_status.clone(),
            audio_runtime: request.context.runtime_health.audio.clone(),
            visual_runtime: request.context.runtime_health.visual.clone(),
            agent_frozen: request.context.runtime_health.agent_frozen,
            pending_actions: request.context.pending_actions.clone(),
            action_history: Vec::new(),
            ..SessionDocument::default()
        };
        let intent = parse_agent_intent(&request.raw_input, &session);
        Ok(PlannerProviderOutput::Typed(PlannerProposal {
            raw_input: intent.raw_input,
            commands: intent.parsed_commands,
            confidence: intent.confidence,
            provider_id: self.provider_id.clone(),
        }))
    }
}
