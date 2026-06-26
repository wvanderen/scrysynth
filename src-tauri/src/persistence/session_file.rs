use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use crate::domain::session::{SessionDocument, CURRENT_SCHEMA_VERSION};

#[derive(Debug, Error)]
pub enum SessionFileError {
    #[error("failed to serialize session JSON: {0}")]
    Serialize(#[source] serde_json::Error),
    #[error("failed to deserialize session JSON: {0}")]
    Deserialize(#[source] serde_json::Error),
    #[error("failed to read session file {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write session file {path}: {source}")]
    Write {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("This is a v1 session — unsupported in Scrysynth v2. Open a v2 session or start a new one.")]
    LegacyV1Session,
    #[error("unsupported schemaVersion {found}; expected {expected}")]
    UnsupportedSchemaVersion { expected: u32, found: u32 },
}

/// Phase-1 probe: read only `schemaVersion` so a model-drift serde failure on a
/// v1 file can NEVER preempt the friendly version message (Pitfall #1). v1 files
/// are rejected here before the full `SessionDocument` deserialize is attempted.
#[derive(Deserialize)]
struct SchemaVersionProbe {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
}

pub fn save_session_to_path(
    session: &SessionDocument,
    path: &Path,
) -> Result<(), SessionFileError> {
    let json = serde_json::to_string_pretty(session).map_err(SessionFileError::Serialize)?;
    let temp_path = temporary_path(path);

    fs::write(&temp_path, json).map_err(|source| SessionFileError::Write {
        path: temp_path.display().to_string(),
        source,
    })?;

    fs::rename(&temp_path, path).map_err(|source| SessionFileError::Write {
        path: path.display().to_string(),
        source,
    })?;

    Ok(())
}

pub fn open_session_from_path(path: &Path) -> Result<SessionDocument, SessionFileError> {
    let contents = fs::read_to_string(path).map_err(|source| SessionFileError::Read {
        path: path.display().to_string(),
        source,
    })?;

    // Phase 1 — version probe (Pitfall #1). Parse only the version field so a v1
    // file (whose removed `audioPrimitive` shape would otherwise make the full
    // serde deserialize fail cryptically) is rejected with the friendly message.
    let probe: SchemaVersionProbe =
        serde_json::from_str(&contents).map_err(SessionFileError::Deserialize)?;

    if probe.schema_version == 1 {
        return Err(SessionFileError::LegacyV1Session);
    }

    if probe.schema_version != CURRENT_SCHEMA_VERSION {
        return Err(SessionFileError::UnsupportedSchemaVersion {
            expected: CURRENT_SCHEMA_VERSION,
            found: probe.schema_version,
        });
    }

    // Phase 2 — full deserialize (version already verified).
    let session: SessionDocument =
        serde_json::from_str(&contents).map_err(SessionFileError::Deserialize)?;

    Ok(session)
}

fn temporary_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!(".{name}.tmp"))
        .unwrap_or_else(|| ".session.tmp".to_string());

    path.with_file_name(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Write `contents` to a temp file; the returned `TempDir` guard cleans up on drop.
    fn write_temp(contents: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("session.json");
        std::fs::write(&path, contents).expect("write temp session");
        (dir, path)
    }

    #[test]
    fn v1_session_is_rejected_with_friendly_message() {
        // D-10: a v1 file must surface the specific v1 message, never a cryptic
        // serde error from the removed `audioPrimitive` field (Pitfall #1).
        let v1_contents = r#"{
            "schemaVersion": 1,
            "sessionId": "s",
            "title": "old",
            "createdAt": "2026-01-01T00:00:00Z",
            "updatedAt": "2026-01-01T00:00:00Z",
            "transport": { "tempoBpm": 120, "isPlaying": false, "positionBeats": 0 },
            "nodes": [],
            "routes": [],
            "buses": [],
            "macros": [],
            "scenes": [],
            "variations": [],
            "ownershipRules": [],
            "runtimeStatus": []
        }"#;
        let (_guard, path) = write_temp(v1_contents);

        let error = open_session_from_path(&path).expect_err("v1 is rejected");
        assert!(
            matches!(error, SessionFileError::LegacyV1Session),
            "expected LegacyV1Session, got {error:?}"
        );
        assert!(error.to_string().contains("v1 session"));
    }

    #[test]
    fn unknown_future_schema_version_is_rejected() {
        let contents = r#"{"schemaVersion": 99}"#;
        let (_guard, path) = write_temp(contents);

        let error = open_session_from_path(&path).expect_err("unknown version rejected");
        assert!(matches!(
            error,
            SessionFileError::UnsupportedSchemaVersion { expected: 2, found: 99 }
        ));
    }

    #[test]
    fn current_schema_round_trips() {
        let session = SessionDocument::default();
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("ok.json");
        save_session_to_path(&session, &path).expect("save");
        let restored = open_session_from_path(&path).expect("open");
        assert_eq!(restored.schema_version, CURRENT_SCHEMA_VERSION);
    }
}
