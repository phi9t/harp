//! XDG session-index store paths.
use std::path::{Path, PathBuf};

/// Resolve `$XDG_DATA_HOME/harp/session-index` or `~/.local/share/harp/session-index`.
pub fn default_index_root() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("harp").join("session-index");
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("harp")
        .join("session-index")
}

pub fn validate_session_id(session_id: &str) -> crate::SessionIndexResult<&str> {
    let parsed = uuid::Uuid::parse_str(session_id).map_err(|_| {
        crate::SessionIndexError::InvalidSessionId {
            value: session_id.to_owned(),
        }
    })?;
    if parsed.to_string() != session_id {
        return Err(crate::SessionIndexError::InvalidSessionId {
            value: session_id.to_owned(),
        });
    }
    Ok(session_id)
}

pub fn session_dir(index_root: &Path, session_id: &str) -> crate::SessionIndexResult<PathBuf> {
    Ok(index_root.join(validate_session_id(session_id)?))
}
