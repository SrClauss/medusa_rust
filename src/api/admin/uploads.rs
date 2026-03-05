//! Admin file-upload handlers — backed by S3/MinIO.
//!
//! POST /admin/uploads        — multipart file upload (one or many files)
//! DELETE /admin/uploads      — delete files by key
//! GET  /admin/uploads/presigned — generate presigned URL (future)

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

#[derive(Debug, Serialize)]
pub struct UploadedFile {
    pub key: String,
    pub url: String,
    pub name: String,
    pub size: usize,
    pub content_type: String,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub uploads: Vec<UploadedFile>,
}

/// POST /admin/uploads
///
/// Accepts a `multipart/form-data` body with one or more `files` fields.
/// Each file is stored in the configured bucket under a UUID-prefixed key.
pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), AppError> {
    let mut uploads: Vec<UploadedFile> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {e}")))?
    {
        let field_name = field.name().unwrap_or("file").to_string();
        let file_name = field
            .file_name()
            .unwrap_or("upload")
            .to_string();

        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::Internal(format!("Read error: {e}")))?
            .to_vec();

        let size = data.len();

        // Build a unique key: <uuid>/<original_filename>
        let ext = std::path::Path::new(&file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let key = if ext.is_empty() {
            format!("uploads/{}/{}", Uuid::new_v4(), file_name)
        } else {
            format!("uploads/{}/{}", Uuid::new_v4(), file_name)
        };

        let url = state.storage.upload_file(&key, data, &content_type).await?;

        tracing::info!(
            key = %key,
            url = %url,
            size = size,
            content_type = %content_type,
            "File uploaded."
        );

        uploads.push(UploadedFile {
            key,
            url,
            name: file_name,
            size,
            content_type,
        });
    }

    if uploads.is_empty() {
        return Err(AppError::BadRequest(
            "No files provided. Send a multipart/form-data body with a 'files' field.".into(),
        ));
    }

    Ok((StatusCode::CREATED, Json(UploadResponse { uploads })))
}

// ─── Delete files ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DeleteFilesPayload {
    pub file_keys: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DeleteFilesResponse {
    pub success: bool,
}

/// DELETE /admin/uploads
pub async fn delete_files(
    State(state): State<AppState>,
    Json(payload): Json<DeleteFilesPayload>,
) -> Result<Json<DeleteFilesResponse>, AppError> {
    for key in &payload.file_keys {
        if let Err(e) = state.storage.delete_file(key).await {
            tracing::warn!(key = %key, error = %e, "Could not delete file (may already be gone).");
        }
    }
    Ok(Json(DeleteFilesResponse { success: true }))
}

/// GET /admin/uploads/presigned  — placeholder, returns 501
pub async fn list(_: State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "uploads": [],
        "note": "Use POST /admin/uploads to upload files."
    })))
}

// Keep stub methods required by the admin mod.rs router
pub async fn get(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    Err(AppError::NotFound("Upload not found".into()))
}
pub async fn create(_: State<AppState>, _: Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    Err(AppError::BadRequest("Use the multipart upload endpoint".into()))
}
pub async fn update(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    Err(AppError::NotFound("Upload not found".into()))
}
pub async fn delete_one(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    Err(AppError::NotFound("Upload not found".into()))
}
pub async fn add_products(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn remove_products(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn add_country(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn remove_country(_: State<AppState>, _: axum::extract::Path<(uuid::Uuid, String)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn add_fulfillment_provider(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn remove_fulfillment_provider(_: State<AppState>, _: axum::extract::Path<(uuid::Uuid, String)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn add_payment_provider(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn remove_payment_provider(_: State<AppState>, _: axum::extract::Path<(uuid::Uuid, String)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn get_by_code(_: State<AppState>, _: axum::extract::Path<String>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn add_region(_: State<AppState>, _: axum::extract::Path<(uuid::Uuid, uuid::Uuid)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn remove_region(_: State<AppState>, _: axum::extract::Path<(uuid::Uuid, uuid::Uuid)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn list_conditions(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({"discount_conditions":[]}))) }
pub async fn create_condition(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>, _: Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> { Ok((StatusCode::CREATED, Json(serde_json::json!({})))) }
pub async fn get_condition(_: State<AppState>, _: axum::extract::Path<(uuid::Uuid, uuid::Uuid)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn update_condition(_: State<AppState>, _: axum::extract::Path<(uuid::Uuid, uuid::Uuid)>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn delete_condition(_: State<AppState>, _: axum::extract::Path<(uuid::Uuid, uuid::Uuid)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn request_password_reset(_: State<AppState>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn reset_password(_: State<AppState>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn add_prices(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn delete_prices(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn list_products(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({"products":[]}))) }
pub async fn list_levels(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({"inventory_levels":[]}))) }
pub async fn create_level(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>, _: Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> { Ok((StatusCode::CREATED, Json(serde_json::json!({})))) }
pub async fn update_level(_: State<AppState>, _: axum::extract::Path<(uuid::Uuid, uuid::Uuid)>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn delete_level(_: State<AppState>, _: axum::extract::Path<(uuid::Uuid, uuid::Uuid)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn receive(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn add_line_item(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>, _: Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> { Ok((StatusCode::CREATED, Json(serde_json::json!({})))) }
pub async fn update_line_item(_: State<AppState>, _: axum::extract::Path<(uuid::Uuid, uuid::Uuid)>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn delete_line_item(_: State<AppState>, _: axum::extract::Path<(uuid::Uuid, uuid::Uuid)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn register_payment(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn confirm(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn cancel(_: State<AppState>, _: axum::extract::Path<uuid::Uuid>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
