//! Store collections handlers
use axum::extract::{Path, State};
use axum::Json;
use sqlx::Row;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

pub async fn list(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT id, title, handle, metadata, created_at, updated_at FROM product_collections WHERE deleted_at IS NULL ORDER BY title")
        .fetch_all(&*state.db).await?;
    let collections: Vec<_> = rows.iter().map(|r| serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "title": r.get::<String, _>("title"),
        "handle": r.get::<String, _>("handle"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    })).collect();
    let count = collections.len();
    Ok(Json(serde_json::json!({ "collections": collections, "count": count })))
}

pub async fn get(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("SELECT id, title, handle, metadata, created_at, updated_at FROM product_collections WHERE id = $1 AND deleted_at IS NULL")
        .bind(id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Collection not found".into()))?;
    Ok(Json(serde_json::json!({ "collection": {
        "id": r.get::<Uuid, _>("id"),
        "title": r.get::<String, _>("title"),
        "handle": r.get::<String, _>("handle"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    }})))
}
