use std::fs;

use scrysynth_lib::application::session_store::SessionStore;
use scrysynth_lib::domain::session::SessionDocument;
use scrysynth_lib::persistence::session_file::{
    open_session_from_path, save_session_to_path, SessionFileError,
};
use tempfile::tempdir;

#[test]
fn session_persistence_round_trip_preserves_canonical_collections() {
    let temp_dir = tempdir().expect("tempdir");
    let path = temp_dir.path().join("session.json");
    let store = SessionStore::new_default();
    let original = store.current();

    save_session_to_path(&original, &path).expect("save succeeds");
    let saved_json = fs::read_to_string(&path).expect("saved session is readable");
    let restored = open_session_from_path(&path).expect("open succeeds");

    assert!(saved_json.contains("\"schemaVersion\""));
    assert!(saved_json.contains("\"nodes\""));
    assert!(saved_json.contains("\"routes\""));
    assert!(saved_json.contains("\"buses\""));
    assert!(saved_json.contains("\"macros\""));
    assert!(saved_json.contains("\"scenes\""));
    assert!(saved_json.contains("\"variations\""));
    assert!(saved_json.contains("\"ownershipRules\""));
    assert!(saved_json.contains("\"runtimeStatus\""));
    assert_eq!(restored.nodes, original.nodes);
    assert_eq!(restored.routes, original.routes);
    assert_eq!(restored.buses, original.buses);
    assert_eq!(restored.macros, original.macros);
    assert_eq!(restored.scenes, original.scenes);
    assert_eq!(restored.variations, original.variations);
    assert_eq!(restored.ownership_rules, original.ownership_rules);
    assert_eq!(restored.runtime_status, original.runtime_status);
}

#[test]
fn session_persistence_rejects_corrupt_json() {
    let temp_dir = tempdir().expect("tempdir");
    let path = temp_dir.path().join("corrupt-session.json");

    fs::write(&path, "{ definitely not valid json }").expect("writes corrupt fixture");

    let result = open_session_from_path(&path);

    assert!(
        matches!(result, Err(SessionFileError::Deserialize(_))),
        "corrupt JSON should fail cleanly"
    );
}

#[test]
fn session_persistence_rejects_unsupported_schema_version() {
    let temp_dir = tempdir().expect("tempdir");
    let path = temp_dir.path().join("unsupported-schema-version.json");
    let store = SessionStore::new_default();
    let session = store.current();

    let mut value = serde_json::to_value(session).expect("session serializes");
    value["schemaVersion"] = serde_json::json!(99);
    fs::write(
        &path,
        serde_json::to_string_pretty(&value).expect("fixture serializes"),
    )
    .expect("fixture writes");

    let result = open_session_from_path(&path);

    assert!(matches!(
        result,
        Err(SessionFileError::UnsupportedSchemaVersion {
            expected: 1,
            found: 99,
        })
    ));
}

#[test]
fn session_persistence_open_failure_does_not_replace_store() {
    let temp_dir = tempdir().expect("tempdir");
    let path = temp_dir.path().join("bad-session.json");
    let mut store = SessionStore::new_default();
    let original = store.current();

    fs::write(&path, "{ broken json }").expect("writes invalid fixture");

    let result = open_session_from_path(&path);
    if let Ok(session) = result.as_ref() {
        store.replace_current(session.clone());
    }

    assert!(result.is_err());
    assert_eq!(store.current(), original);
}

#[test]
fn session_persistence_open_replaces_store_with_saved_document() {
    let temp_dir = tempdir().expect("tempdir");
    let path = temp_dir.path().join("saved-session.json");
    let original = SessionStore::new_default().current();
    let mut store = SessionStore::new_default();
    store.replace_current(SessionDocument::default());

    save_session_to_path(&original, &path).expect("save succeeds");
    let restored = open_session_from_path(&path).expect("open succeeds");
    store.replace_current(restored.clone());

    assert_eq!(store.current(), restored);
    assert_eq!(restored, original);
}
