pub mod application;
pub mod audio;
pub mod domain;
pub mod persistence;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use application::graph_edit;
use application::session_store::SessionStore;
use domain::session::{write_generated_typescript_contract, GraphEditCommand, SessionDocument};
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
            save_session_to_path,
            open_session_from_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
