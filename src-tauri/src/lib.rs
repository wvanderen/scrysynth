pub mod application;
pub mod audio;
pub mod domain;
pub mod persistence;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use application::agent_command;
use application::graph_edit;
use application::performance_command;
use application::session_store::SessionStore;
use domain::session::{
    write_generated_typescript_contract, ActorRef, ControllerKind, GraphEditCommand,
    PerformanceCommand, SessionDocument,
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = write_generated_typescript_contract();

    tauri::Builder::default()
        .manage(Mutex::new(SessionStore::new_default()))
        .plugin(tauri_plugin_opener::init())
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
            reject_pending_action
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
