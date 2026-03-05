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
use storage::{cache::build_cache, db::create_pool};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "super-secret-change-me".into());
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "9000".into());

    let pool = create_pool(&database_url).await?;

    // Run migrations at startup using the runtime migrator (no DATABASE_URL at compile time needed).
    storage::db::run_migrations(&pool).await?;

    let cache: Cache<String, serde_json::Value> = build_cache(10_000, 300);
    let storage_config = StorageConfig {
        upload_dir: std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".into()),
        public_base_url: std::env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| format!("http://{}:{}", host, port)),
    };

    let state = AppState {
        db: Arc::new(pool),
        cache: Arc::new(cache),
        storage_config: Arc::new(storage_config),
        jwt_secret,
    };

    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::PUT, axum::http::Method::DELETE, axum::http::Method::OPTIONS])
        .allow_headers(tower_http::cors::Any);

    let app = api::build_router(state)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors);

    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("🚀 MedusaRust listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
