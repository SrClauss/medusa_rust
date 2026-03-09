#![recursion_limit = "256"]
//! Library entrypoint for integration tests and other consumers.

pub mod api;
pub mod auth;
pub mod core;
pub mod error;
pub mod models;
pub mod routes_manifest;
pub mod state;
pub mod storage;
pub mod wizard;

// extensibility plugins
pub mod plugins;

// event bus
mod events;
pub use events::EventBus;

// re-export commonly used types
pub use state::{AppState, StorageConfig};
