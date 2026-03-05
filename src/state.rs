//! Shared application state, passed to every Axum handler via `State<AppState>`.
//!
//! `AppState` is `Clone + Send + Sync` because all inner types are wrapped in
//! `Arc` or are `Copy`.

use moka::future::Cache;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;

/// Persistent configuration for the file-storage backend.
#[derive(Clone, Debug)]
pub struct StorageConfig {
    /// Root directory where uploaded assets are stored on disk.
    pub upload_dir: String,
    /// Public base URL used to build asset URLs returned to clients.
    pub public_base_url: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            upload_dir: "uploads".into(),
            public_base_url: "http://localhost:9000".into(),
        }
    }
}

/// Global application state shared across all request handlers.
///
/// All fields are `Arc`-wrapped so that `AppState: Clone` is cheap and
/// the state remains `Send + Sync`, which is required by Axum.
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool (SQLx).
    pub db: Arc<PgPool>,

    /// In-process cache backed by Moka (high-frequency data such as
    /// product prices, categories and session tokens).
    pub cache: Arc<Cache<String, Value>>,

    /// File-storage configuration (local disk or object store).
    pub storage_config: Arc<StorageConfig>,

    /// Secret key used to sign and verify JWT tokens.
    pub jwt_secret: String,
}

// Compile-time proof that AppState satisfies the Axum requirements.
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AppState>();
};
