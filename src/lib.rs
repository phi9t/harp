pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod cli;
pub mod core;
pub mod eval;
pub mod insula;
pub mod provider;
pub mod supervisor;
pub mod tools;
pub mod tui;
