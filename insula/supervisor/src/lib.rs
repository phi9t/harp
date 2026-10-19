#![forbid(unsafe_code)]
//! The supervisor owns service intent and observation. `ExecutionOwner` is the
//! only authority allowed to create, stop, enter, contain or reap a service.
pub mod error;
pub mod model;
pub mod owner;
pub mod owner_rpc;
pub mod probes;
pub mod store;
pub mod supervisor;
pub mod transport;
pub use error::{Error, Result};
pub use model::*;
pub use owner::ExecutionOwner;
pub use supervisor::Supervisor;
