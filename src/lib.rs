#![recursion_limit = "256"]
//! Library entrypoint for integration tests and other consumers.

pub mod api;
pub mod auth;
pub mod core;
pub mod error;
pub mod models;
pub mod notifications;
pub mod routes_manifest;
pub mod search;
pub mod state;
pub mod storage;
pub mod wizard;

// extensibility plugins
pub mod plugins;

// event bus
pub mod events;
pub use events::{BusDriver, Event, EventBus, EventHandler};

// sagas / workflow orchestration
pub mod sagas;

// re-export commonly used types
pub use state::{AppState, StorageConfig};
