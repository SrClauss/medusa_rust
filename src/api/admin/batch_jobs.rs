//! Admin batch_jobs handlers — DB-backed CRUD
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(default = "d20")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}
fn d20() -> i64 { 20 }

fn batch_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "type": r.get::<String, _>("type"),
        "status": r.get::<String, _>("status"),
        "created_by": r.get::<Option<Uuid>, _>("created_by"),
        "context": r.get::<Option<serde_json::Value>, _>("context"),
        "result": r.get::<Option<serde_json::Value>, _>("result"),
        "dry_run": r.get::<bool, _>("dry_run"),
        "pre_processed_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("pre_processed_at"),
        "processing_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("processing_at"),
        "confirmed_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("confirmed_at"),
        "completed_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at"),
        "failed_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("failed_at"),
        "canceled_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("canceled_at"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, type, status, created_by, context, result, dry_run, \
         pre_processed_at, processing_at, confirmed_at, completed_at, failed_at, canceled_at, \
         created_at, updated_at FROM batch_jobs ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM batch_jobs")
        .fetch_one(&*state.db)
        .await?;
    let batch_jobs: Vec<_> = rows.iter().map(batch_json).collect();
    Ok(Json(serde_json::json!({"batch_jobs": batch_jobs, "count": count, "offset": p.offset, "limit": p.limit})))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, type, status, created_by, context, result, dry_run, \
         pre_processed_at, processing_at, confirmed_at, completed_at, failed_at, canceled_at, \
         created_at, updated_at FROM batch_jobs WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Batch job not found".into()))?;
    Ok(Json(serde_json::json!({"batch_job": batch_json(&r)})))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let job_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("product-import");
    let dry_run = payload.get("dry_run").and_then(|v| v.as_bool()).unwrap_or(false);
    sqlx::query(
        "INSERT INTO batch_jobs (id, type, status, context, dry_run, created_at, updated_at) \
         VALUES ($1, $2, 'created', $3, $4, NOW(), NOW())",
    )
    .bind(id)
    .bind(job_type)
    .bind(payload.get("context").cloned())
    .bind(dry_run)
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id))
        .await
        .map(|r| (StatusCode::CREATED, r))
}

pub async fn confirm(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "UPDATE batch_jobs SET status = 'confirmed', confirmed_at = NOW(), updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "UPDATE batch_jobs SET status = 'canceled', canceled_at = NOW(), updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}
