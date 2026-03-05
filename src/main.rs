//! Application entry point — composes AppState and starts the Axum server.
mod api;
mod auth;
mod core;
mod error;
mod models;
mod routes_manifest;
mod state;
mod storage;
mod wizard;

use std::sync::Arc;
use moka::future::Cache;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use state::{AppState, StorageConfig};
use storage::{cache::build_cache, db::create_pool, s3::S3Storage};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret   = std::env::var("JWT_SECRET").unwrap_or_else(|_| "super-secret-change-me".into());
    let host         = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port         = std::env::var("PORT").unwrap_or_else(|_| "9000".into());

    // ── Database ──────────────────────────────────────────────────────────────
    let pool = create_pool(&database_url).await?;
    storage::db::run_migrations(&pool).await?;

    // ── Object Storage (MinIO / S3) ───────────────────────────────────────────
    let storage_config = StorageConfig::from_env();
    let s3 = S3Storage::new(&storage_config);
    s3.ensure_bucket_exists().await?;
    tracing::info!(
        bucket = %storage_config.s3_bucket,
        endpoint = ?storage_config.s3_endpoint,
        "Object storage ready."
    );

    // ── In-process cache ──────────────────────────────────────────────────────
    // 10 000 entries, 5-minute TTL.  Override with MOKA_MAX_CAPACITY / MOKA_TTL_SECS.
    let max_capacity: u64 = std::env::var("MOKA_MAX_CAPACITY")
        .ok().and_then(|v| v.parse().ok()).unwrap_or(10_000);
    let ttl_secs: u64 = std::env::var("MOKA_TTL_SECS")
        .ok().and_then(|v| v.parse().ok()).unwrap_or(300);
    let cache: Cache<String, serde_json::Value> = build_cache(max_capacity, ttl_secs);

    // ── AppState ──────────────────────────────────────────────────────────────
    let state = AppState {
        db:             Arc::new(pool),
        cache:          Arc::new(cache),
        storage:        Arc::new(s3),
        storage_config: Arc::new(storage_config),
        jwt_secret,
    };

    // ── CORS ──────────────────────────────────────────────────────────────────
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(tower_http::cors::Any);

    // ── Router ────────────────────────────────────────────────────────────────
    let app = api::build_router(state)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors);

    // ── Server ────────────────────────────────────────────────────────────────
    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("🚀 MedusaRust listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
