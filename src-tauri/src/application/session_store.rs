use crate::application::midi_learn::{HardwareInputRouter, HardwareLearnState};
use crate::audio::runtime_manager::{AudioRuntimeManager, AudioRuntimeManagerError};
use crate::domain::session::{
    new_id, ActionHistoryEntry, ActorRef, AgentRuntimeState, AudioBusType, AudioOutputNode,
    AudioOutputType, AudioPrimitive, AudioRuntimeHealth, AudioRuntimeLifecycle, AudioRuntimeState,
    AudioSourceNode, AudioSourceType, BindingTarget, Bus, ChannelMode, ControllerKind, DiffSummary,
    GraphEditCommand, HardwareBinding, HardwareLearnLifecycle, HardwareLearnStatus,
    HardwareListenerLifecycle, HardwareRuntimeDiagnostic, HardwareRuntimeDiagnosticCode,
    HardwareRuntimeSettings, HardwareRuntimeStatus, MacroDefinition, MacroOverride, MidiInputPort,
    Node, NodeType, OscRuntimeStatus, OwnershipAssignment, OwnershipRule, ParameterOverride,
    ParameterValue, PerformanceCommand, Port, PortDirection, Route, RuntimeConnectionState,
    RuntimeKind, RuntimeStatusRef, SceneDefinition, SessionDocument, SignalType, TypedCommand,
    VariationDefinition,
};
use crate::hardware::midi_input::MidiInputManager;
use crate::hardware::osc_input::OscInputManager;
use crate::visual::runtime_manager::{VisualRuntimeManager, VisualRuntimeManagerError};

#[derive(Debug, Clone, PartialEq)]
pub struct OwnershipGateError {
    pub node_id: String,
    pub reason: OwnershipGateReason,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipGateReason {
    AgentFrozen,
    LockedNode,
    AgentBlockedByUserOwnership,
}

impl std::fmt::Display for OwnershipGateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.reason {
            OwnershipGateReason::AgentFrozen => write!(f, "agent is frozen"),
            OwnershipGateReason::LockedNode => write!(f, "node '{}' is locked", self.node_id),
            OwnershipGateReason::AgentBlockedByUserOwnership => {
                write!(f, "node '{}' is user-owned", self.node_id)
            }
        }
    }
}

impl std::error::Error for OwnershipGateError {}

#[derive(Debug)]
pub struct SessionStore {
    current: SessionDocument,
    audio_runtime_manager: AudioRuntimeManager,
    visual_runtime_manager: VisualRuntimeManager,
    #[allow(dead_code)]
    hardware_router: HardwareInputRouter,
    midi_input_manager: MidiInputManager,
    osc_input_manager: OscInputManager,
    hardware_settings: HardwareRuntimeSettings,
    hardware_status: HardwareRuntimeStatus,
}

impl SessionStore {
    pub fn new_default() -> Self {
        let (midi_input_manager, midi_rx) = MidiInputManager::new();
        let mut hardware_router = HardwareInputRouter::new();
        hardware_router.attach_midi_receiver(midi_rx);

        Self {
            current: build_default_session(),
            audio_runtime_manager: AudioRuntimeManager::default(),
            visual_runtime_manager: VisualRuntimeManager::default(),
            hardware_router,
            midi_input_manager,
            osc_input_manager: OscInputManager::default(),
            hardware_settings: HardwareRuntimeSettings::default(),
            hardware_status: HardwareRuntimeStatus::default(),
        }
    }

    pub fn current(&self) -> SessionDocument {
        self.current.clone()
    }

    pub fn replace_current(&mut self, session: SessionDocument) {
        self.current = session;
    }

    pub fn start_audio_runtime(&mut self) -> Result<SessionDocument, AudioRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.audio_runtime_manager);
        let result = manager.start(self);
        self.audio_runtime_manager = manager;
        result
    }

    pub fn stop_audio_runtime(&mut self) -> Result<SessionDocument, AudioRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.audio_runtime_manager);
        let result = manager.stop(self);
        self.audio_runtime_manager = manager;
        result
    }

    pub fn panic_audio_runtime(&mut self) -> Result<SessionDocument, AudioRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.audio_runtime_manager);
        let result = manager.panic(self);
        self.audio_runtime_manager = manager;
        result
    }

    pub fn reconcile_audio_graph_edit(
        &mut self,
        command: &GraphEditCommand,
    ) -> Result<SessionDocument, AudioRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.audio_runtime_manager);
        let result = manager.reconcile_graph_edit(self, command);
        self.audio_runtime_manager = manager;
        result
    }

    pub fn start_visual_runtime(&mut self) -> Result<SessionDocument, VisualRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.visual_runtime_manager);
        let result = manager.start(self);
        self.visual_runtime_manager = manager;
        result
    }

    pub fn stop_visual_runtime(&mut self) -> Result<SessionDocument, VisualRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.visual_runtime_manager);
        let result = manager.stop(self);
        self.visual_runtime_manager = manager;
        result
    }

    pub fn panic_visual_runtime(&mut self) -> Result<SessionDocument, VisualRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.visual_runtime_manager);
        let result = manager.panic(self);
        self.visual_runtime_manager = manager;
        result
    }

    pub fn reload_visual_scene(&mut self) -> Result<SessionDocument, VisualRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.visual_runtime_manager);
        let result = manager.reload_scene(self);
        self.visual_runtime_manager = manager;
        result
    }

    pub fn reconcile_visual_graph_edit(
        &mut self,
        command: &GraphEditCommand,
    ) -> Result<SessionDocument, VisualRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.visual_runtime_manager);
        let result = manager.reconcile_graph_edit(self, command);
        self.visual_runtime_manager = manager;
        result
    }

    pub fn reconcile_visual_macro_value(
        &mut self,
        macro_id: &str,
        value: f64,
    ) -> Result<SessionDocument, VisualRuntimeManagerError> {
        let mut manager = std::mem::take(&mut self.visual_runtime_manager);
        let result = manager.reconcile_macro_value(self, macro_id, value);
        self.visual_runtime_manager = manager;
        result
    }

    pub fn start_hardware_learn(
        &mut self,
        target: BindingTarget,
    ) -> Result<HardwareRuntimeStatus, String> {
        if !self.midi_input_manager.is_listening() {
            self.start_midi_listener()?;
        }
        self.hardware_router.start_learn(target);
        Ok(self.hardware_runtime_status())
    }

    pub fn stop_hardware_learn(&mut self) {
        self.hardware_router.stop_learn();
    }

    pub fn poll_hardware_events(&mut self) -> Option<HardwareBinding> {
        self.hardware_router.poll_and_route(&mut self.current)
    }

    pub fn remove_hardware_binding(&mut self, binding_id: &str) -> bool {
        let index = self
            .current
            .hardware_bindings
            .iter()
            .position(|b| b.id == binding_id);
        if let Some(i) = index {
            self.current.hardware_bindings.remove(i);
            true
        } else {
            false
        }
    }

    pub fn list_midi_input_ports(&mut self) -> Result<Vec<MidiInputPort>, String> {
        match MidiInputManager::list_devices() {
            Ok(names) => {
                let selected = self.hardware_settings.midi.selected_input_id.clone();
                let ports: Vec<MidiInputPort> = names
                    .iter()
                    .enumerate()
                    .map(|(index, display_name)| {
                        let id = midi_port_id(index);
                        MidiInputPort {
                            is_selected: selected.as_deref() == Some(id.as_str()),
                            id,
                            display_name: display_name.clone(),
                        }
                    })
                    .collect();

                self.hardware_status.midi.available_input_count = Some(ports.len() as u32);
                self.hardware_status.midi.selected_input_id = selected.clone();
                self.hardware_status.midi.selected_display_name = ports
                    .iter()
                    .find(|port| port.is_selected)
                    .map(|port| port.display_name.clone());
                self.hardware_status.midi.last_error = None;
                self.hardware_status.diagnostics.clear();

                if ports.is_empty() {
                    self.hardware_status.midi.lifecycle = HardwareListenerLifecycle::Unavailable;
                    self.record_hardware_diagnostic(HardwareRuntimeDiagnostic {
                        code: HardwareRuntimeDiagnosticCode::NoMidiPorts,
                        message: "No MIDI input ports are available.".to_string(),
                        recoverable: true,
                        detail: Some(
                            "Connect or enable a MIDI device, then refresh hardware inputs."
                                .to_string(),
                        ),
                    });
                } else if selected
                    .as_deref()
                    .is_some_and(|id| !ports.iter().any(|port| port.id == id))
                {
                    self.hardware_status.midi.lifecycle = HardwareListenerLifecycle::Error;
                    self.hardware_status.midi.last_error =
                        Some("Selected MIDI input is no longer available.".to_string());
                    self.record_hardware_diagnostic(invalid_midi_selection_diagnostic(
                        selected.as_deref().unwrap_or_default(),
                    ));
                } else if self.hardware_status.midi.lifecycle
                    == HardwareListenerLifecycle::Unavailable
                    || self.hardware_status.midi.lifecycle == HardwareListenerLifecycle::Error
                {
                    self.hardware_status.midi.lifecycle = HardwareListenerLifecycle::Stopped;
                }

                Ok(ports)
            }
            Err(err) => {
                self.hardware_status.midi.lifecycle = HardwareListenerLifecycle::Error;
                self.hardware_status.midi.last_error = Some(err.clone());
                self.hardware_status.diagnostics.clear();
                self.record_hardware_diagnostic(HardwareRuntimeDiagnostic {
                    code: HardwareRuntimeDiagnosticCode::MidiEnumerationFailed,
                    message: "Could not enumerate MIDI input ports.".to_string(),
                    recoverable: true,
                    detail: Some(err.clone()),
                });
                Err(err)
            }
        }
    }

    pub fn hardware_runtime_settings(&self) -> HardwareRuntimeSettings {
        self.hardware_settings.clone()
    }

    pub fn update_hardware_runtime_settings(
        &mut self,
        settings: HardwareRuntimeSettings,
    ) -> Result<HardwareRuntimeStatus, String> {
        validate_osc_settings(&settings)?;
        self.validate_selected_midi_input(&settings)?;

        let midi_selection_changed =
            self.hardware_settings.midi.selected_input_id != settings.midi.selected_input_id;
        let midi_was_listening =
            self.hardware_status.midi.lifecycle == HardwareListenerLifecycle::Listening;
        let osc_settings_changed = self.hardware_settings.osc != settings.osc;
        let osc_needs_restart = osc_settings_changed
            && self.hardware_status.osc.lifecycle == HardwareListenerLifecycle::Listening;

        self.hardware_settings = settings.clone();
        self.hardware_status.midi.selected_input_id = settings.midi.selected_input_id.clone();
        self.hardware_status.osc = OscRuntimeStatus {
            lifecycle: if osc_needs_restart {
                HardwareListenerLifecycle::Restarting
            } else {
                self.hardware_status.osc.lifecycle.clone()
            },
            bind_host: settings.osc.bind_host.clone(),
            listen_port: settings.osc.listen_port,
            last_error: None,
        };
        self.hardware_status.diagnostics.clear();

        if midi_selection_changed && midi_was_listening {
            self.restart_midi_listener()?;
            self.record_hardware_diagnostic(HardwareRuntimeDiagnostic {
                code: HardwareRuntimeDiagnosticCode::ListenerRestarted,
                message: "MIDI listener restarted for the selected input.".to_string(),
                recoverable: true,
                detail: None,
            });
        }

        if osc_needs_restart {
            self.restart_osc_listener()?;
            self.record_hardware_diagnostic(HardwareRuntimeDiagnostic {
                code: HardwareRuntimeDiagnosticCode::ListenerRestarted,
                message: "OSC listener restarted for the updated bind settings.".to_string(),
                recoverable: true,
                detail: None,
            });
        }

        Ok(self.hardware_runtime_status())
    }

    pub fn hardware_runtime_status(&self) -> HardwareRuntimeStatus {
        let mut status = self.hardware_status.clone();
        status.learn = match &self.hardware_router.learn_state {
            HardwareLearnState::Idle => HardwareLearnStatus {
                lifecycle: HardwareLearnLifecycle::Idle,
                target: None,
                source: None,
            },
            HardwareLearnState::Learning { target } => HardwareLearnStatus {
                lifecycle: HardwareLearnLifecycle::Learning,
                target: Some(target.clone()),
                source: None,
            },
            HardwareLearnState::Captured { source, target } => HardwareLearnStatus {
                lifecycle: HardwareLearnLifecycle::Captured,
                target: Some(target.clone()),
                source: Some(source.clone()),
            },
        };
        status.midi.selected_input_id = self.hardware_settings.midi.selected_input_id.clone();
        status.osc.bind_host = self.hardware_settings.osc.bind_host.clone();
        status.osc.listen_port = self.hardware_settings.osc.listen_port;
        status
    }

    pub fn start_hardware_listeners(&mut self) -> Result<HardwareRuntimeStatus, String> {
        self.hardware_status.diagnostics.clear();
        self.start_midi_listener()?;
        if self.hardware_settings.osc.auto_start {
            self.start_osc_listener()?;
        }
        Ok(self.hardware_runtime_status())
    }

    pub fn stop_hardware_listeners(&mut self) -> HardwareRuntimeStatus {
        self.midi_input_manager.stop_listening();
        self.stop_osc_listener();
        self.hardware_status.midi.lifecycle = HardwareListenerLifecycle::Stopped;
        self.hardware_status.osc.lifecycle = HardwareListenerLifecycle::Stopped;
        self.hardware_status.midi.last_error = None;
        self.hardware_status.osc.last_error = None;
        self.hardware_status.diagnostics.clear();
        self.record_hardware_diagnostic(HardwareRuntimeDiagnostic {
            code: HardwareRuntimeDiagnosticCode::ListenerStopped,
            message: "Hardware listeners are stopped.".to_string(),
            recoverable: true,
            detail: None,
        });
        self.hardware_runtime_status()
    }

    pub fn drain_hardware_events(&mut self, max_events: Option<u32>) -> SessionDocument {
        let limit = max_events.unwrap_or(16).clamp(1, 128);
        for _ in 0..limit {
            if self.poll_hardware_events().is_none() {
                break;
            }
        }
        self.current()
    }

    pub fn start_osc_listener(&mut self) -> Result<HardwareRuntimeStatus, String> {
        self.hardware_status.diagnostics.clear();
        self.start_osc_listener_inner()?;
        Ok(self.hardware_runtime_status())
    }

    pub fn stop_osc_listener(&mut self) {
        self.osc_input_manager.stop_listening();
        self.hardware_router.detach_osc_receiver();
        self.hardware_status.osc.lifecycle = HardwareListenerLifecycle::Stopped;
        self.hardware_status.osc.last_error = None;
    }

    fn start_midi_listener(&mut self) -> Result<(), String> {
        self.hardware_status.midi.lifecycle = HardwareListenerLifecycle::Starting;
        let ports = self.list_midi_input_ports()?;
        if ports.is_empty() {
            return Err("No MIDI input ports are available. Connect or enable a MIDI device, then refresh hardware inputs.".to_string());
        }

        let port_index = self
            .hardware_settings
            .midi
            .selected_input_id
            .as_deref()
            .map(|id| {
                parse_midi_port_id(id).ok_or_else(|| invalid_midi_selection_diagnostic(id).message)
            })
            .transpose()?;

        if let Some(index) = port_index {
            if index >= ports.len() {
                let selected = self
                    .hardware_settings
                    .midi
                    .selected_input_id
                    .clone()
                    .unwrap_or_default();
                let diagnostic = invalid_midi_selection_diagnostic(&selected);
                let message = diagnostic.message.clone();
                self.hardware_status.diagnostics.clear();
                self.record_hardware_diagnostic(diagnostic);
                self.hardware_status.midi.lifecycle = HardwareListenerLifecycle::Error;
                self.hardware_status.midi.last_error = Some(message.clone());
                return Err(message);
            }
        }

        match self.midi_input_manager.start_listening(port_index) {
            Ok(()) => {
                let selected_port = port_index
                    .and_then(|index| ports.get(index))
                    .or_else(|| ports.first());
                self.hardware_status.midi.lifecycle = HardwareListenerLifecycle::Listening;
                self.hardware_status.midi.last_error = None;
                self.hardware_status.midi.available_input_count = Some(ports.len() as u32);
                self.hardware_status.midi.selected_input_id =
                    self.hardware_settings.midi.selected_input_id.clone();
                self.hardware_status.midi.selected_display_name =
                    selected_port.map(|port| port.display_name.clone());
                Ok(())
            }
            Err(err) => {
                self.hardware_status.midi.lifecycle = HardwareListenerLifecycle::Error;
                self.hardware_status.midi.last_error = Some(err.clone());
                self.hardware_status.diagnostics.clear();
                self.record_hardware_diagnostic(HardwareRuntimeDiagnostic {
                    code: HardwareRuntimeDiagnosticCode::MidiEnumerationFailed,
                    message: "Could not start MIDI input listening.".to_string(),
                    recoverable: true,
                    detail: Some(err.clone()),
                });
                Err(err)
            }
        }
    }

    fn restart_midi_listener(&mut self) -> Result<(), String> {
        self.hardware_status.midi.lifecycle = HardwareListenerLifecycle::Restarting;
        self.midi_input_manager.stop_listening();
        self.start_midi_listener()
    }

    fn start_osc_listener_inner(&mut self) -> Result<(), String> {
        let settings = self.hardware_settings.osc.clone();
        self.hardware_status.osc.lifecycle = HardwareListenerLifecycle::Starting;
        self.hardware_status.osc.bind_host = settings.bind_host.clone();
        self.hardware_status.osc.listen_port = settings.listen_port;

        match self
            .osc_input_manager
            .start_listening(&settings.bind_host, settings.listen_port)
        {
            Ok(rx) => {
                self.hardware_router.attach_osc_receiver(rx);
                self.hardware_status.osc.lifecycle = HardwareListenerLifecycle::Listening;
                self.hardware_status.osc.last_error = None;
                Ok(())
            }
            Err(err) => {
                self.hardware_router.detach_osc_receiver();
                self.hardware_status.osc.lifecycle = HardwareListenerLifecycle::Error;
                self.hardware_status.osc.last_error = Some(err.clone());
                self.record_hardware_diagnostic(osc_bind_diagnostic(
                    &settings.bind_host,
                    settings.listen_port,
                    &err,
                ));
                Err(err)
            }
        }
    }

    fn restart_osc_listener(&mut self) -> Result<(), String> {
        self.hardware_status.osc.lifecycle = HardwareListenerLifecycle::Restarting;
        self.osc_input_manager.stop_listening();
        self.hardware_router.detach_osc_receiver();
        self.start_osc_listener_inner()
    }

    fn validate_selected_midi_input(
        &mut self,
        settings: &HardwareRuntimeSettings,
    ) -> Result<(), String> {
        let Some(selected_input_id) = settings.midi.selected_input_id.as_deref() else {
            return Ok(());
        };
        let Some(index) = parse_midi_port_id(selected_input_id) else {
            let diagnostic = invalid_midi_selection_diagnostic(selected_input_id);
            let message = diagnostic.message.clone();
            self.hardware_status.diagnostics.clear();
            self.record_hardware_diagnostic(diagnostic);
            return Err(message);
        };

        let ports = MidiInputManager::list_devices().map_err(|err| {
            self.hardware_status.diagnostics.clear();
            self.record_hardware_diagnostic(HardwareRuntimeDiagnostic {
                code: HardwareRuntimeDiagnosticCode::MidiEnumerationFailed,
                message: "Could not validate the selected MIDI input.".to_string(),
                recoverable: true,
                detail: Some(err.clone()),
            });
            err
        })?;

        if ports.is_empty() {
            let diagnostic = HardwareRuntimeDiagnostic {
                code: HardwareRuntimeDiagnosticCode::NoMidiPorts,
                message: "No MIDI input ports are available.".to_string(),
                recoverable: true,
                detail: Some(
                    "Connect or enable a MIDI device before selecting a MIDI input.".to_string(),
                ),
            };
            let message = diagnostic.message.clone();
            self.hardware_status.diagnostics.clear();
            self.record_hardware_diagnostic(diagnostic);
            return Err(message);
        }

        if index >= ports.len() {
            let diagnostic = invalid_midi_selection_diagnostic(selected_input_id);
            let message = diagnostic.message.clone();
            self.hardware_status.diagnostics.clear();
            self.record_hardware_diagnostic(diagnostic);
            return Err(message);
        }

        Ok(())
    }

    fn record_hardware_diagnostic(&mut self, diagnostic: HardwareRuntimeDiagnostic) {
        self.hardware_status.diagnostics.push(diagnostic);
    }

    pub fn derive_agent_runtime_state(&self) -> AgentRuntimeState {
        AgentRuntimeState {
            is_available: true,
            pending_action_count: self.current.pending_actions.len() as u32,
            is_frozen: self.current.agent_frozen,
        }
    }

    pub fn mutate_current<F, E>(&mut self, mutate: F) -> Result<SessionDocument, E>
    where
        F: FnOnce(&mut SessionDocument) -> Result<(), E>,
    {
        let mut next = self.current.clone();
        mutate(&mut next)?;
        self.current = next.clone();
        Ok(next)
    }

    pub fn check_ownership(
        &self,
        actor: &ActorRef,
        command: &TypedCommand,
    ) -> Result<(), Vec<OwnershipGateError>> {
        if actor.actor_id == "user" {
            return Ok(());
        }

        if self.current.agent_frozen {
            return Err(vec![OwnershipGateError {
                node_id: String::new(),
                reason: OwnershipGateReason::AgentFrozen,
            }]);
        }

        let target_ids = extract_target_node_ids(command);
        let mut errors = Vec::new();

        for node_id in &target_ids {
            if let Some(node) = self.current.nodes.iter().find(|n| &n.id == node_id) {
                if node.ownership.is_locked {
                    errors.push(OwnershipGateError {
                        node_id: node_id.clone(),
                        reason: OwnershipGateReason::LockedNode,
                    });
                } else if node.ownership.controller == ControllerKind::User {
                    errors.push(OwnershipGateError {
                        node_id: node_id.clone(),
                        reason: OwnershipGateReason::AgentBlockedByUserOwnership,
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn log_action(&mut self, actor: &ActorRef, command: &TypedCommand) {
        let _ = self.mutate_current(|session| {
            let diff = generate_diff_summary(command, session);
            let entry = ActionHistoryEntry {
                id: new_id(),
                timestamp: "2026-04-12T00:00:00Z".to_string(),
                actor: actor.clone(),
                command: command.clone(),
                diff,
            };
            session.action_history.push(entry);
            if session.action_history.len() > 200 {
                session
                    .action_history
                    .drain(0..session.action_history.len() - 200);
            }
            Ok::<(), ()>(())
        });
    }
}

fn midi_port_id(index: usize) -> String {
    format!("midi-input-{index}")
}

fn parse_midi_port_id(id: &str) -> Option<usize> {
    id.strip_prefix("midi-input-")?.parse().ok()
}

fn validate_osc_settings(settings: &HardwareRuntimeSettings) -> Result<(), String> {
    if settings.osc.bind_host.trim().is_empty() {
        return Err("OSC bind host cannot be empty".to_string());
    }
    if settings.osc.listen_port == 0 {
        return Err("OSC listen port must be between 1 and 65535".to_string());
    }
    Ok(())
}

fn invalid_midi_selection_diagnostic(selected_input_id: &str) -> HardwareRuntimeDiagnostic {
    HardwareRuntimeDiagnostic {
        code: HardwareRuntimeDiagnosticCode::InvalidMidiPortSelection,
        message: "Selected MIDI input is not available.".to_string(),
        recoverable: true,
        detail: Some(format!(
            "The selected input id '{selected_input_id}' was not found in the current MIDI port list."
        )),
    }
}

fn osc_bind_diagnostic(bind_host: &str, listen_port: u16, err: &str) -> HardwareRuntimeDiagnostic {
    let port_in_use = err.contains("Address already in use")
        || err.contains("addr in use")
        || err.contains("os error 48")
        || err.contains("os error 98");
    HardwareRuntimeDiagnostic {
        code: if port_in_use {
            HardwareRuntimeDiagnosticCode::OscPortInUse
        } else {
            HardwareRuntimeDiagnosticCode::OscBindFailed
        },
        message: if port_in_use {
            "OSC listen port is already in use.".to_string()
        } else {
            "Could not start OSC listener.".to_string()
        },
        recoverable: true,
        detail: Some(format!(
            "Tried to bind OSC listener on {bind_host}:{listen_port}. {err}"
        )),
    }
}

fn extract_target_node_ids(command: &TypedCommand) -> Vec<String> {
    match command {
        TypedCommand::GraphEdit(gec) => match gec {
            GraphEditCommand::AddNode { .. } => vec![],
            GraphEditCommand::RemoveNode { node_id } => vec![node_id.clone()],
            GraphEditCommand::SetNodeEnabled { node_id, .. } => vec![node_id.clone()],
            GraphEditCommand::SetParameterValue { node_id, .. } => vec![node_id.clone()],
            GraphEditCommand::AddRoute { route } => {
                vec![route.source_node_id.clone(), route.target_node_id.clone()]
            }
            GraphEditCommand::RemoveRoute { .. } => vec![],
            GraphEditCommand::AssignNodeToBus { node_id, .. } => vec![node_id.clone()],
            GraphEditCommand::ClearNodeBusAssignment { node_id } => vec![node_id.clone()],
        },
        TypedCommand::Performance(_) => vec![],
    }
}

fn generate_diff_summary(command: &TypedCommand, _session: &SessionDocument) -> DiffSummary {
    let (description, affected_node_ids) = match command {
        TypedCommand::GraphEdit(GraphEditCommand::AddNode { node }) => (
            format!(
                "Added {} node",
                format!("{:?}", node.node_type).to_lowercase()
            ),
            vec![node.id.clone()],
        ),
        TypedCommand::GraphEdit(GraphEditCommand::RemoveNode { node_id }) => {
            (format!("Removed node {}", node_id), vec![node_id.clone()])
        }
        TypedCommand::GraphEdit(GraphEditCommand::SetNodeEnabled { node_id, enabled }) => (
            format!(
                "Set node {} {}",
                node_id,
                if *enabled { "enabled" } else { "disabled" }
            ),
            vec![node_id.clone()],
        ),
        TypedCommand::GraphEdit(GraphEditCommand::SetParameterValue {
            node_id,
            parameter_id,
            value,
        }) => (
            format!("Set parameter {} to {} on {}", parameter_id, value, node_id),
            vec![node_id.clone()],
        ),
        TypedCommand::GraphEdit(GraphEditCommand::AddRoute { route }) => (
            format!(
                "Added route from {} to {}",
                route.source_node_id, route.target_node_id
            ),
            vec![route.source_node_id.clone(), route.target_node_id.clone()],
        ),
        TypedCommand::GraphEdit(GraphEditCommand::RemoveRoute { route_id }) => {
            (format!("Removed route {}", route_id), vec![])
        }
        TypedCommand::GraphEdit(GraphEditCommand::AssignNodeToBus { node_id, bus_id }) => (
            format!("Assigned node {} to bus {}", node_id, bus_id),
            vec![node_id.clone()],
        ),
        TypedCommand::GraphEdit(GraphEditCommand::ClearNodeBusAssignment { node_id }) => (
            format!("Cleared bus assignment for node {}", node_id),
            vec![node_id.clone()],
        ),
        TypedCommand::Performance(PerformanceCommand::RecallScene { scene_id }) => {
            (format!("Recalled scene {}", scene_id), vec![])
        }
        TypedCommand::Performance(PerformanceCommand::SaveVariation { name, .. }) => {
            (format!("Saved variation '{}'", name), vec![])
        }
        TypedCommand::Performance(PerformanceCommand::RestoreVariation { variation_id }) => {
            (format!("Restored variation {}", variation_id), vec![])
        }
    };

    DiffSummary {
        description,
        affected_node_ids,
        before_snippet: String::new(),
        after_snippet: String::new(),
    }
}

fn build_default_session() -> SessionDocument {
    let scene_id = new_id();
    let source_node_id = new_id();
    let source_out_port_id = new_id();
    let master_node_id = new_id();
    let master_in_port_id = new_id();
    let bus_id = new_id();
    let parameter_id = new_id();
    let macro_id = new_id();

    SessionDocument {
        title: "Default Scrysynth Session".to_string(),
        audio_runtime: AudioRuntimeState {
            lifecycle: AudioRuntimeLifecycle::Idle,
            health: AudioRuntimeHealth::Unknown,
            sample_rate_hz: None,
            block_size: None,
            active_patch_id: None,
            last_error: None,
            panic_recovery_count: 0,
        },
        nodes: vec![
            Node {
                id: source_node_id.clone(),
                node_type: NodeType::Source,
                ports: vec![Port {
                    id: source_out_port_id.clone(),
                    name: "main_out".to_string(),
                    direction: PortDirection::Output,
                    signal_type: SignalType::Audio,
                }],
                parameters: vec![ParameterValue {
                    id: parameter_id.clone(),
                    name: "level".to_string(),
                    value: 0.8,
                    default_value: 0.8,
                    min_value: 0.0,
                    max_value: 1.0,
                    unit: "linear".to_string(),
                }],
                runtime_target: Some("audio/source/default".to_string()),
                scene_membership: vec![scene_id.clone()],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::Shared,
                    is_locked: false,
                },
                enabled: true,
                audio_primitive: Some(AudioPrimitive::Source(AudioSourceNode {
                    source_type: AudioSourceType::Oscillator,
                    channel_mode: ChannelMode::Mono,
                    bus_target_id: Some(bus_id.clone()),
                })),
            },
            Node {
                id: master_node_id.clone(),
                node_type: NodeType::Output,
                ports: vec![Port {
                    id: master_in_port_id.clone(),
                    name: "master_in".to_string(),
                    direction: PortDirection::Input,
                    signal_type: SignalType::Audio,
                }],
                parameters: vec![],
                runtime_target: Some("audio/output/master".to_string()),
                scene_membership: vec![scene_id.clone()],
                ownership: OwnershipAssignment {
                    controller: ControllerKind::User,
                    is_locked: false,
                },
                enabled: true,
                audio_primitive: Some(AudioPrimitive::Output(AudioOutputNode {
                    output_type: AudioOutputType::Master,
                    channels: 2,
                    bus_target_id: Some(bus_id.clone()),
                })),
            },
        ],
        routes: vec![Route {
            id: new_id(),
            source_node_id,
            source_port_id: source_out_port_id,
            target_node_id: master_node_id.clone(),
            target_port_id: master_in_port_id,
            bus_id: Some(bus_id.clone()),
        }],
        buses: vec![Bus {
            id: bus_id,
            name: "master_bus".to_string(),
            channels: 2,
            bus_type: AudioBusType::Main,
            is_enabled: true,
        }],
        macros: vec![MacroDefinition {
            id: macro_id.clone(),
            name: "energy".to_string(),
            target_parameter_ids: vec![parameter_id.clone()],
            range_start: 0.0,
            range_end: 1.0,
            targets: vec![],
        }],
        scenes: vec![SceneDefinition {
            id: scene_id.clone(),
            name: "intro".to_string(),
            active_node_ids: vec![master_node_id],
            macro_overrides: vec![MacroOverride {
                macro_id: macro_id.clone(),
                value: 0.65,
            }],
        }],
        variations: vec![VariationDefinition {
            id: new_id(),
            name: "intro-alt".to_string(),
            scene_id,
            parameter_overrides: vec![ParameterOverride {
                parameter_id,
                value: 0.55,
            }],
        }],
        ownership_rules: vec![OwnershipRule {
            id: new_id(),
            scope: "graph:master".to_string(),
            controller: ControllerKind::Shared,
            can_override: true,
        }],
        runtime_status: vec![
            RuntimeStatusRef {
                id: new_id(),
                runtime: RuntimeKind::Audio,
                status: RuntimeConnectionState::Disconnected,
                target_id: Some("audio-runtime".to_string()),
                last_error: None,
            },
            RuntimeStatusRef {
                id: new_id(),
                runtime: RuntimeKind::Visual,
                status: RuntimeConnectionState::Disconnected,
                target_id: Some("visual-runtime".to_string()),
                last_error: None,
            },
            RuntimeStatusRef {
                id: new_id(),
                runtime: RuntimeKind::Agent,
                status: RuntimeConnectionState::Disconnected,
                target_id: Some("agent-runtime".to_string()),
                last_error: None,
            },
        ],
        ..SessionDocument::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::midi_input::MidiLearnEvent;
    use std::net::UdpSocket;
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn session_store_create_default_session_returns_seeded_graph() {
        let store = SessionStore::new_default();
        let session = store.current();

        assert!(!session.nodes.is_empty());
        assert!(!session.routes.is_empty());
        assert!(!session.buses.is_empty());
        assert!(!session.macros.is_empty());
        assert!(!session.scenes.is_empty());
        assert!(!session.variations.is_empty());
        assert!(!session.ownership_rules.is_empty());
        assert!(!session.runtime_status.is_empty());
    }

    #[test]
    fn session_store_get_current_session_returns_same_session_after_replace() {
        let mut store = SessionStore::new_default();
        let mut replacement = SessionDocument::default();
        replacement.title = "Replacement Session".to_string();
        store.replace_current(replacement.clone());

        assert_eq!(store.current(), replacement);
    }

    #[test]
    fn session_store_wires_midi_receiver_on_creation() {
        let store = SessionStore::new_default();
        assert!(store.hardware_router.midi_rx.is_some());
        assert!(!store.midi_input_manager.is_listening());
    }

    #[test]
    fn invalid_midi_port_selection_returns_diagnostic_without_enumerating() {
        let mut store = SessionStore::new_default();
        let mut settings = store.hardware_runtime_settings();
        settings.midi.selected_input_id = Some("not-a-midi-port".to_string());

        let err = store
            .update_hardware_runtime_settings(settings)
            .expect_err("invalid stable port id should fail");

        assert!(err.contains("Selected MIDI input is not available"));
        let status = store.hardware_runtime_status();
        assert_eq!(status.diagnostics.len(), 1);
        assert_eq!(
            status.diagnostics[0].code,
            HardwareRuntimeDiagnosticCode::InvalidMidiPortSelection
        );
    }

    #[test]
    fn app_level_store_path_captures_midi_learn_event() {
        let (tx, rx) = mpsc::channel();
        let mut store = SessionStore::new_default();
        store.hardware_router.attach_midi_receiver(rx);
        store
            .hardware_router
            .start_learn(BindingTarget::TransportPlay);

        tx.send(MidiLearnEvent::MidiCc {
            channel: 0,
            controller: 7,
            value: 127,
        })
        .unwrap();

        let binding = store
            .poll_hardware_events()
            .expect("learn should capture a binding through SessionStore");
        assert_eq!(binding.target, BindingTarget::TransportPlay);
        assert_eq!(store.current().hardware_bindings.len(), 1);
        assert!(matches!(
            store.hardware_runtime_status().learn.lifecycle,
            HardwareLearnLifecycle::Captured
        ));
    }

    #[test]
    fn changing_invalid_midi_port_preserves_existing_bindings() {
        let mut store = SessionStore::new_default();
        store.current.hardware_bindings.push(HardwareBinding {
            id: "hb-1".to_string(),
            source: crate::domain::session::HardwareSource::MidiCc {
                channel: 0,
                controller: 7,
            },
            target: BindingTarget::TransportPlay,
            transform: crate::domain::session::ValueTransform::default(),
        });

        let mut settings = store.hardware_runtime_settings();
        settings.midi.selected_input_id = Some("not-a-midi-port".to_string());
        let _ = store.update_hardware_runtime_settings(settings);

        assert_eq!(store.current().hardware_bindings.len(), 1);
        assert_eq!(store.current().hardware_bindings[0].id, "hb-1");
    }

    #[test]
    fn app_level_store_path_captures_osc_learn_event() {
        let port = unused_local_udp_port();
        let mut store = SessionStore::new_default();
        let mut settings = store.hardware_runtime_settings();
        settings.osc.listen_port = port;
        store
            .update_hardware_runtime_settings(settings)
            .expect("valid OSC settings");
        store
            .start_osc_listener()
            .expect("OSC listener should start");
        store
            .hardware_router
            .start_learn(BindingTarget::TransportPlay);

        send_osc_message(port, "/scrysynth/learn", vec![rosc::OscType::Float(1.0)]);

        let binding = wait_for_hardware_binding(&mut store);
        assert_eq!(binding.target, BindingTarget::TransportPlay);
        assert_eq!(
            binding.source,
            crate::domain::session::HardwareSource::OscAddress {
                address: "/scrysynth/learn".to_string()
            }
        );
        assert!(matches!(
            store.hardware_runtime_status().learn.lifecycle,
            HardwareLearnLifecycle::Captured
        ));

        store.stop_osc_listener();
    }

    #[test]
    fn osc_listener_restarts_on_same_port() {
        let port = unused_local_udp_port();
        let mut store = SessionStore::new_default();
        let mut settings = store.hardware_runtime_settings();
        settings.osc.listen_port = port;
        store
            .update_hardware_runtime_settings(settings)
            .expect("valid OSC settings");

        store
            .start_osc_listener()
            .expect("first OSC listener start");
        assert!(store.osc_input_manager.is_listening());
        store.stop_osc_listener();
        assert!(!store.osc_input_manager.is_listening());
        store
            .start_osc_listener()
            .expect("OSC listener should restart on same port");
        assert!(store.osc_input_manager.is_listening());

        store.stop_osc_listener();
    }

    #[test]
    fn osc_listener_shutdown_releases_udp_port() {
        let port = unused_local_udp_port();
        let mut store = SessionStore::new_default();
        let mut settings = store.hardware_runtime_settings();
        settings.osc.listen_port = port;
        store
            .update_hardware_runtime_settings(settings)
            .expect("valid OSC settings");
        store.start_osc_listener().expect("OSC listener start");

        store.stop_osc_listener();

        UdpSocket::bind(("127.0.0.1", port)).expect("stopped listener should release UDP port");
    }

    #[test]
    fn osc_port_in_use_returns_clear_diagnostic() {
        let blocker = UdpSocket::bind(("127.0.0.1", 0)).expect("bind blocker");
        let port = blocker.local_addr().unwrap().port();
        let mut store = SessionStore::new_default();
        let mut settings = store.hardware_runtime_settings();
        settings.osc.listen_port = port;
        store
            .update_hardware_runtime_settings(settings)
            .expect("valid OSC settings");

        let err = store
            .start_osc_listener()
            .expect_err("port in use should fail");
        assert!(err.contains("Failed to bind OSC listener"));

        let status = store.hardware_runtime_status();
        assert_eq!(status.osc.lifecycle, HardwareListenerLifecycle::Error);
        assert!(status.osc.last_error.is_some());
        assert_eq!(status.diagnostics.len(), 1);
        assert_eq!(
            status.diagnostics[0].code,
            HardwareRuntimeDiagnosticCode::OscPortInUse
        );
        assert!(status.diagnostics[0]
            .detail
            .as_deref()
            .unwrap_or_default()
            .contains(&format!("127.0.0.1:{port}")));
    }

    fn unused_local_udp_port() -> u16 {
        let socket = UdpSocket::bind(("127.0.0.1", 0)).expect("bind ephemeral port");
        socket.local_addr().unwrap().port()
    }

    fn send_osc_message(port: u16, address: &str, args: Vec<rosc::OscType>) {
        let packet = rosc::OscPacket::Message(rosc::OscMessage {
            addr: address.to_string(),
            args,
        });
        let bytes = rosc::encoder::encode(&packet).expect("encode OSC packet");
        let socket = UdpSocket::bind(("127.0.0.1", 0)).expect("bind sender");
        socket
            .send_to(&bytes, ("127.0.0.1", port))
            .expect("send OSC packet");
    }

    fn wait_for_hardware_binding(store: &mut SessionStore) -> HardwareBinding {
        let deadline = Instant::now() + Duration::from_millis(500);
        loop {
            if let Some(binding) = store.poll_hardware_events() {
                return binding;
            }
            assert!(Instant::now() < deadline, "timed out waiting for OSC event");
            thread::sleep(Duration::from_millis(10));
        }
    }
}
