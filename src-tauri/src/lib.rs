mod application;
mod domain;

use std::sync::Mutex;

use application::session_store::SessionStore;
use domain::session::{write_generated_typescript_contract, SessionDocument};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = write_generated_typescript_contract();

    tauri::Builder::default()
        .manage(Mutex::new(SessionStore::new_default()))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            create_default_session,
            get_current_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
