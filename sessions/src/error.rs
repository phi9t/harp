use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionIndexError {
    #[error("invalid session id {value:?}; expected canonical lowercase hyphenated UUID")]
    InvalidSessionId { value: String },

    #[error("unsupported trace dialect '{dialect}': {detail}")]
    UnsupportedDialect { dialect: String, detail: String },

    #[error("failed to detect dialect for {path}: {detail}")]
    DetectFailed { path: PathBuf, detail: String },

    #[error("io error while {context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid JSONL at {path}:{line}: {detail}")]
    InvalidJsonl {
        path: PathBuf,
        line: u64,
        detail: String,
    },

    #[error("rollout {path} is missing session_meta.payload.id")]
    MissingSessionId { path: PathBuf },

    #[error("invalid stored session summary at {path}: {source}")]
    InvalidStoredSummary {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("rollout session id {observed} does not match verify target {expected}")]
    SessionIdMismatch { expected: String, observed: String },

    #[error("failed to serialize {artifact}: {source}")]
    Serialization {
        artifact: &'static str,
        #[source]
        source: serde_json::Error,
    },
}
