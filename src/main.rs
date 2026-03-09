//! Application entry point — composes AppState and starts the Axum server.
#![recursion_limit = "256"]
mod api;
mod auth;
mod core;
mod error;
mod models;
mod routes_manifest;
mod state;
mod events;
mod plugins;
mod sagas;
mod storage;
mod wizard;

use crate::plugins::PluginManager;

use std::sync::Arc;
use moka::future::Cache;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use state::{AppState, StorageConfig};
use crate::api::auth::EmailPasswordService;
use storage::{cache::build_cache, db::create_pool, s3::S3Storage};
use events::EventBus;

// Payment plugins
use plugin_api::PaymentProvider as _;
use asaas_plugin::AsaasPlugin;
use mercadopago_plugin::MercadoPagoPlugin;
use stripe_plugin::StripePlugin;
use paypal_plugin::PayPalPlugin;

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
    let auth_service = crate::api::auth::EmailPasswordService::new(pool.clone());

    let state = AppState {
        db:             Arc::new(pool.clone()),
        cache:          Arc::new(cache),
        storage:        Arc::new(s3),
        storage_config: Arc::new(storage_config),
        jwt_secret,
        auth_service:   Arc::new(auth_service),
        payment_methods: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        plugin_mgr:      Arc::new(tokio::sync::Mutex::new(crate::plugins::PluginManager::new())),
        event_bus:       Arc::new(EventBus::from_env()),
    };

    // feature-gated built-in plugin registration
    #[cfg(feature = "manual_plugin")]
    {
        let mut pm = state.plugin_mgr.lock().await;
        crate::plugins::register_builtin_plugins(&mut pm, &state)
            .expect("failed to register built-in plugins");
    }

    // ── Async payment plugins ─────────────────────────────────────────────────
    {
        let mut pm = state.plugin_mgr.lock().await;

        // Asaas
        let mut asaas = AsaasPlugin::new();
        let mut asaas_cfg = std::collections::HashMap::new();
        if let Ok(key) = std::env::var("ASAAS_API_KEY") {
            asaas_cfg.insert("api_key".to_string(), key);
        }
        if let Ok(url) = std::env::var("ASAAS_BASE_URL") {
            asaas_cfg.insert("base_url".to_string(), url);
        }
        if !asaas_cfg.is_empty() {
            if let Err(e) = asaas.initialize(asaas_cfg).await {
                tracing::warn!("Asaas plugin skipped: {e}");
            } else {
                pm.register_async_payment_provider(asaas);
                tracing::info!("Asaas payment plugin registered");
            }
        }

        // Mercado Pago
        let mut mp = MercadoPagoPlugin::new();
        let mut mp_cfg = std::collections::HashMap::new();
        if let Ok(token) = std::env::var("MP_ACCESS_TOKEN") {
            mp_cfg.insert("access_token".to_string(), token);
        }
        if !mp_cfg.is_empty() {
            if let Err(e) = mp.initialize(mp_cfg).await {
                tracing::warn!("MercadoPago plugin skipped: {e}");
            } else {
                pm.register_async_payment_provider(mp);
                tracing::info!("MercadoPago payment plugin registered");
            }
        }

        // Stripe
        let mut stripe = StripePlugin::new();
        let mut stripe_cfg = std::collections::HashMap::new();
        if let Ok(key) = std::env::var("STRIPE_SECRET_KEY") {
            stripe_cfg.insert("secret_key".to_string(), key);
        }
        if let Ok(secret) = std::env::var("STRIPE_WEBHOOK_SECRET") {
            stripe_cfg.insert("webhook_secret".to_string(), secret);
        }
        if !stripe_cfg.is_empty() {
            if let Err(e) = stripe.initialize(stripe_cfg).await {
                tracing::warn!("Stripe plugin skipped: {e}");
            } else {
                pm.register_async_payment_provider(stripe);
                tracing::info!("Stripe payment plugin registered");
            }
        }

        // PayPal
        let mut paypal = PayPalPlugin::new();
        let mut paypal_cfg = std::collections::HashMap::new();
        if let Ok(id) = std::env::var("PAYPAL_CLIENT_ID") {
            paypal_cfg.insert("client_id".to_string(), id);
        }
        if let Ok(secret) = std::env::var("PAYPAL_CLIENT_SECRET") {
            paypal_cfg.insert("client_secret".to_string(), secret);
        }
        if let Ok(url) = std::env::var("PAYPAL_BASE_URL") {
            paypal_cfg.insert("base_url".to_string(), url);
        }
        if !paypal_cfg.is_empty() {
            if let Err(e) = paypal.initialize(paypal_cfg).await {
                tracing::warn!("PayPal plugin skipped: {e}");
            } else {
                pm.register_async_payment_provider(paypal);
                tracing::info!("PayPal payment plugin registered");
            }
        }
    }

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
