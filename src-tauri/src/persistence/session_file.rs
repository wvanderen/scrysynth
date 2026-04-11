use std::fs;
use std::path::{Path, PathBuf};

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
    #[error("unsupported schemaVersion {found}; expected {expected}")]
    UnsupportedSchemaVersion { expected: u32, found: u32 },
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

    let session: SessionDocument =
        serde_json::from_str(&contents).map_err(SessionFileError::Deserialize)?;

    if session.schema_version != CURRENT_SCHEMA_VERSION {
        return Err(SessionFileError::UnsupportedSchemaVersion {
            expected: CURRENT_SCHEMA_VERSION,
            found: session.schema_version,
        });
    }

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
