pub mod application;
pub mod audio;
pub mod domain;
pub mod hardware;
pub mod persistence;
pub mod visual;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use application::agent_command;
use application::graph_edit;
use application::macro_command;
use application::performance_command;
use application::session_store::SessionStore;
use domain::session::{
    write_generated_typescript_contract, ActorRef, AgentRuntimeState, BindingTarget,
    ControllerKind, GraphEditCommand, HardwareRuntimeSettings, HardwareRuntimeStatus, MacroCommand,
    MidiInputPort, PerformanceCommand, SessionDocument,
};
use persistence::session_file;

#[tauri::command]
fn create_default_session(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.replace_current(SessionStore::new_default().current());
    Ok(store.current())
}

#[tauri::command]
fn get_current_session(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let store = state.lock().map_err(|err| err.to_string())?;
    Ok(store.current())
}

#[tauri::command]
fn apply_graph_edit(
    command: GraphEditCommand,
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    graph_edit::apply_graph_edit(&mut store, command).map_err(|err| err.to_string())
}

#[tauri::command]
fn save_session_to_path(
    path: String,
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<(), String> {
    let path = PathBuf::from(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let store = state.lock().map_err(|err| err.to_string())?;
    session_file::save_session_to_path(&store.current(), &path).map_err(|err| err.to_string())
}

#[tauri::command]
fn open_session_from_path(
    path: String,
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let session = session_file::open_session_from_path(&PathBuf::from(path))
        .map_err(|err| err.to_string())?;

    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.replace_current(session.clone());
    Ok(session)
}

#[tauri::command]
fn start_audio_runtime(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.start_audio_runtime().map_err(|err| err.to_string())
}

#[tauri::command]
fn stop_audio_runtime(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.stop_audio_runtime().map_err(|err| err.to_string())
}

#[tauri::command]
fn panic_audio_runtime(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.panic_audio_runtime().map_err(|err| err.to_string())
}

#[tauri::command]
fn apply_performance_command(
    command: PerformanceCommand,
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    performance_command::apply_performance_command(&mut store, command)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn send_agent_message(
    message: String,
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<serde_json::Value, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    let session = store.current();
    let intent = agent_command::parse_agent_intent(&message, &session);
    let actor = ActorRef {
        actor_id: "agent".to_string(),
        correlation_id: domain::session::new_id(),
    };
    let _result = agent_command::apply_agent_command(&mut store, actor.clone(), intent.clone())
        .map_err(|err| err.to_string())?;

    let session = store.current();
    serde_json::to_value(serde_json::json!({
        "session": session,
        "intent": intent,
    }))
    .map_err(|err| err.to_string())
}

#[tauri::command]
fn toggle_agent_freeze(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    agent_command::toggle_agent_freeze(&mut store)
}

#[tauri::command]
fn reclaim_ownership(
    node_ids: Option<Vec<String>>,
    target_controller: Option<ControllerKind>,
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    agent_command::reclaim_ownership(&mut store, node_ids, target_controller)
}

#[tauri::command]
fn approve_pending_action(
    pending_action_id: String,
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    agent_command::approve_pending_action(&mut store, &pending_action_id)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn reject_pending_action(
    pending_action_id: String,
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    agent_command::reject_pending_action(&mut store, &pending_action_id)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn start_visual_runtime(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.start_visual_runtime().map_err(|err| err.to_string())
}

#[tauri::command]
fn stop_visual_runtime(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.stop_visual_runtime().map_err(|err| err.to_string())
}

#[tauri::command]
fn panic_visual_runtime(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.panic_visual_runtime().map_err(|err| err.to_string())
}

#[tauri::command]
fn apply_macro_command(
    command: MacroCommand,
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    macro_command::apply_macro_command(&mut store, command).map_err(|err| err.to_string())
}

#[tauri::command]
fn get_agent_runtime_state(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<AgentRuntimeState, String> {
    let store = state.lock().map_err(|err| err.to_string())?;
    Ok(store.derive_agent_runtime_state())
}

#[tauri::command]
fn start_hardware_learn(
    target: BindingTarget,
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<HardwareRuntimeStatus, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.start_hardware_learn(target)
}

#[tauri::command]
fn stop_hardware_learn(state: tauri::State<'_, Mutex<SessionStore>>) -> Result<(), String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.stop_hardware_learn();
    Ok(())
}

#[tauri::command]
fn poll_hardware_events(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.poll_hardware_events();
    Ok(store.current())
}

#[tauri::command]
fn remove_hardware_binding(
    binding_id: String,
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.remove_hardware_binding(&binding_id);
    Ok(store.current())
}

#[tauri::command]
fn list_midi_input_ports(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<Vec<MidiInputPort>, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.list_midi_input_ports()
}

#[tauri::command]
fn get_hardware_runtime_settings(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<HardwareRuntimeSettings, String> {
    let store = state.lock().map_err(|err| err.to_string())?;
    Ok(store.hardware_runtime_settings())
}

#[tauri::command]
fn update_hardware_runtime_settings(
    settings: HardwareRuntimeSettings,
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<HardwareRuntimeStatus, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.update_hardware_runtime_settings(settings)
}

#[tauri::command]
fn get_hardware_runtime_status(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<HardwareRuntimeStatus, String> {
    let store = state.lock().map_err(|err| err.to_string())?;
    Ok(store.hardware_runtime_status())
}

#[tauri::command]
fn start_hardware_listeners(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<HardwareRuntimeStatus, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    store.start_hardware_listeners()
}

#[tauri::command]
fn stop_hardware_listeners(
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<HardwareRuntimeStatus, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    Ok(store.stop_hardware_listeners())
}

#[tauri::command]
fn drain_hardware_events(
    max_events: Option<u32>,
    state: tauri::State<'_, Mutex<SessionStore>>,
) -> Result<SessionDocument, String> {
    let mut store = state.lock().map_err(|err| err.to_string())?;
    Ok(store.drain_hardware_events(max_events))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(err) = write_generated_typescript_contract() {
        eprintln!("ERROR: Failed to write TypeScript type contract: {err}");
        eprintln!("Run `cargo test write_generated_typescript_contract --manifest-path src-tauri/Cargo.toml` to diagnose.");
        std::process::exit(1);
    }

    tauri::Builder::default()
        .manage(Mutex::new(SessionStore::new_default()))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            create_default_session,
            get_current_session,
            apply_graph_edit,
            apply_performance_command,
            save_session_to_path,
            open_session_from_path,
            start_audio_runtime,
            stop_audio_runtime,
            panic_audio_runtime,
            send_agent_message,
            toggle_agent_freeze,
            reclaim_ownership,
            approve_pending_action,
            reject_pending_action,
            start_visual_runtime,
            stop_visual_runtime,
            panic_visual_runtime,
            apply_macro_command,
            get_agent_runtime_state,
            start_hardware_learn,
            stop_hardware_learn,
            poll_hardware_events,
            remove_hardware_binding,
            list_midi_input_ports,
            get_hardware_runtime_settings,
            update_hardware_runtime_settings,
            get_hardware_runtime_status,
            start_hardware_listeners,
            stop_hardware_listeners,
            drain_hardware_events
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
