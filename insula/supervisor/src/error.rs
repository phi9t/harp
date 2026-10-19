use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error, PartialEq, Eq)]
#[serde(tag = "code", content = "message", rename_all = "snake_case")]
pub enum Error {
    #[error("invalid request: {0}")]
    Invalid(String),
    #[error("request conflicts with durable state: {0}")]
    Conflict(String),
    #[error("controller ownership is unavailable or uncertain")]
    Ownership,
    #[error("stale generation")]
    StaleGeneration,
    #[error("outcome unknown; reconcile before another mutation")]
    Unknown,
    #[error("service not found")]
    NotFound,
    #[error("controller is shutting down")]
    ShuttingDown,
    #[error("operation is still pending")]
    Pending,
    #[error("capability unavailable: {0}")]
    Unsupported(String),
    #[error("owner transport or mechanism unavailable")]
    Unavailable,
    #[error("durable store failure")]
    Storage,
    #[error("local IO failure")]
    Io,
}
pub type Result<T> = std::result::Result<T, Error>;
impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::Io
    }
}
impl From<rusqlite::Error> for Error {
    fn from(_: rusqlite::Error) -> Self {
        Self::Storage
    }
}
impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Self::Invalid("malformed JSON".into())
    }
}
