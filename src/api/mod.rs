//! Top-level router — merges admin, store and wizard routes.

pub mod auth;
pub mod admin;
pub mod store;
pub mod hooks;

use axum::{routing::{get, post}, Router};
use crate::{routes_manifest::*, state::AppState};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(auth::auth_router(state.clone()))
        .merge(admin::admin_router(state.clone()))
        .merge(store::store_router(state.clone()))
        // webhook receiver for payment providers
        .route("/hooks/payment/:provider", post(hooks::payment_provider_webhook))
        // Wizard / Importer endpoints
        .route(WIZARD_IMPORT, post(wizard_import))
        .route(WIZARD_IMPORT_STATUS_ID, get(wizard_import_status))
        // Health check
        .route("/health", get(health))
        .with_state(state)
}

async fn health() -> &'static str { "ok" }

/// POST /wizard/import
/// Accepts a multipart upload with a single `file` field containing a .zip.
async fn wizard_import(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut multipart: axum::extract::Multipart,
) -> Result<axum::Json<serde_json::Value>, crate::error::AppError> {
    use crate::wizard::zip_import::{base64_encode, ImportJob};

    let mut zip_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await
        .map_err(|e| crate::error::AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let bytes = field.bytes().await
                .map_err(|e| crate::error::AppError::Internal(e.to_string()))?;
            zip_bytes = Some(bytes.to_vec());
        }
    }

    let zip_bytes = zip_bytes.ok_or_else(|| crate::error::AppError::BadRequest("Missing 'file' field".into()))?;
    let job_id = uuid::Uuid::new_v4();

    let job = ImportJob {
        job_id,
        zip_base64: base64_encode(&zip_bytes),
        triggered_by: uuid::Uuid::nil(),
    };

    // Spawn background task — replace with Apalis dispatch for persistent queue.
    let db = state.db.clone();
    let storage = state.storage.clone();
    tokio::spawn(async move {
        match crate::wizard::zip_import::run_import_job(&job, &db, Some(&storage)).await {
            Ok(summary) => {
                tracing::info!(
                    job_id = %job.job_id,
                    created_products = summary.created_products,
                    created_variants = summary.created_variants,
                    uploaded_assets = summary.uploaded_assets,
                    "Import completed successfully."
                );
            }
            Err(e) => {
                tracing::error!(job_id = %job.job_id, "Import failed: {}", e);
            }
        }
    });

    Ok(axum::Json(serde_json::json!({
        "job_id": job_id,
        "status": "queued",
        "message": "Import queued. Poll /wizard/import/{job_id}/status for progress.",
    })))
}

/// GET /wizard/import/:job_id/status
async fn wizard_import_status(
    axum::extract::Path(job_id): axum::extract::Path<uuid::Uuid>,
) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "job_id": job_id,
        "status": "processing",
        "message": "Persistent job status requires Apalis PostgreSQL storage.",
    }))
}
