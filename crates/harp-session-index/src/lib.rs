//! Harp Session analyzer — index Execution sessions from raw Rollout artifacts.

pub mod detect;
mod dialect;
pub mod error;
pub mod index;
pub mod spill;
pub mod store;
pub mod verify;

pub use detect::DialectId;
pub use error::SessionIndexError;

pub type SessionIndexResult<T> = Result<T, SessionIndexError>;
