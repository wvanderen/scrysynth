use std::fs;

use scrysynth_lib::application::session_store::SessionStore;
use scrysynth_lib::persistence::session_file::{open_session_from_path, save_session_to_path};
use tempfile::tempdir;

#[test]
fn session_persistence_round_trip_preserves_canonical_collections() {
    let temp_dir = tempdir().expect("tempdir");
    let path = temp_dir.path().join("session.json");
    let store = SessionStore::new_default();
    let original = store.current();

    save_session_to_path(&original, &path).expect("save succeeds");
    let restored = open_session_from_path(&path).expect("open succeeds");

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

    assert!(result.is_err(), "corrupt JSON should fail cleanly");
}

#[test]
fn session_persistence_rejects_unsupported_schema_version() {
    let temp_dir = tempdir().expect("tempdir");
    let path = temp_dir.path().join("unsupported-schema-version.json");
    let store = SessionStore::new_default();
    let session = store.current();

    let mut value = serde_json::to_value(session).expect("session serializes");
    value["schema_version"] = serde_json::json!(99);
    fs::write(
        &path,
        serde_json::to_string_pretty(&value).expect("fixture serializes"),
    )
    .expect("fixture writes");

    let result = open_session_from_path(&path);

    assert!(
        result.is_err(),
        "unsupported schema version should fail cleanly"
    );
}
