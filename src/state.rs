//! Shared application state — passed to every Axum handler via `State<AppState>`.
//!
//! `AppState` is `Clone + Send + Sync` because all inner types are wrapped in
//! `Arc` or are `Copy`/`Clone`.

use moka::future::Cache;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;

use crate::storage::s3::StorageBackend;

// ─── Storage Config ───────────────────────────────────────────────────────────

/// Configuration for the object-storage backend (MinIO or AWS S3).
///
/// Set `S3_ENDPOINT` to a custom URL (e.g. `http://localhost:9000`) to use
/// MinIO or any other S3-compatible service.  Leave it unset to use AWS S3.
#[derive(Clone, Debug)]
pub struct StorageConfig {
    // ── S3 / MinIO ──────────────────────────────────────────────────────────
    /// Custom endpoint URL — set to `http://localhost:9000` for MinIO.
    /// Leave empty to use the standard AWS S3 endpoint.
    pub s3_endpoint: Option<String>,
    /// S3 bucket name.
    pub s3_bucket: String,
    /// AWS / MinIO region.
    pub s3_region: String,
    /// Access-key ID.
    pub s3_access_key: String,
    /// Secret access key.
    pub s3_secret_key: String,
    /// Use path-style URLs — **required** for MinIO; optional for AWS S3.
    pub s3_force_path_style: bool,
    /// Override public base URL (e.g. CDN / nginx proxy in front of MinIO).
    /// Falls back to `<s3_endpoint>/<s3_bucket>` when unset.
    pub s3_public_url: Option<String>,

    // ── Legacy / fallback ───────────────────────────────────────────────────
    /// Local upload directory (used only if S3 is not configured).
    pub upload_dir: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            s3_endpoint: None,
            s3_bucket: "medusa-uploads".into(),
            s3_region: "us-east-1".into(),
            s3_access_key: "minioadmin".into(),
            s3_secret_key: "minioadmin".into(),
            s3_force_path_style: true,
            s3_public_url: None,
            upload_dir: "uploads".into(),
        }
    }
}

impl StorageConfig {
    /// Builds a `StorageConfig` from environment variables.
    pub fn from_env() -> Self {
        Self {
            s3_endpoint: std::env::var("S3_ENDPOINT").ok(),
            s3_bucket: std::env::var("S3_BUCKET").unwrap_or_else(|_| "medusa-uploads".into()),
            s3_region: std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".into()),
            s3_access_key: std::env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".into()),
            s3_secret_key: std::env::var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin".into()),
            s3_force_path_style: std::env::var("S3_FORCE_PATH_STYLE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            s3_public_url: std::env::var("S3_PUBLIC_URL").ok(),
            upload_dir: std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".into()),
        }
    }

    /// Returns the public base URL for assets stored in the bucket.
    pub fn public_base_url(&self) -> String {
        if let Some(ref url) = self.s3_public_url {
            return url.clone();
        }
        if let Some(ref endpoint) = self.s3_endpoint {
            return format!("{}/{}", endpoint.trim_end_matches('/'), self.s3_bucket);
        }
        format!(
            "https://{}.s3.{}.amazonaws.com",
            self.s3_bucket, self.s3_region
        )
    }
}

// ─── AppState ─────────────────────────────────────────────────────────────────

/// Global application state shared across all request handlers.
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool (SQLx).
    pub db: Arc<PgPool>,

    /// In-process Moka cache (prices, categories, session tokens).
    pub cache: Arc<Cache<String, Value>>,

    /// File-storage backend — MinIO or AWS S3 (or any S3-compatible service).
    pub storage: Arc<dyn StorageBackend>,

    /// Storage configuration (bucket, region, endpoint, …).
    pub storage_config: Arc<StorageConfig>,

    /// Secret used to sign and verify JWT tokens.
    pub jwt_secret: String,

    /// Authentication service used by `/auth` endpoints.
    pub auth_service: Arc<dyn crate::api::auth::AuthService>,

    /// In-memory store for payment methods created via the storefront API.
    ///
    /// The real Medusa backend persists these to the database and also
    /// interfaces with third-party providers (Stripe, etc.). Our minimal
    /// implementation simply keeps them in a mutex-protected `Vec` so that
    /// `GET` can return whatever has been added during the lifetime of the
    /// process.  This is only used by the stubbed customer payment methods
    /// routes and is sufficient for tests that are only concerned with the
    /// request/response shape.
    pub payment_methods: Arc<tokio::sync::Mutex<Vec<serde_json::Value>>>,
}

// Compile-time proof that AppState satisfies Axum's requirements.
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AppState>();
};
