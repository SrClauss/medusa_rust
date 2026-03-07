//! Store return-reasons handlers — public read-only
use axum::{extract::{Path, Query, State}, Json};
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

fn build_return_reason(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "value": r.get::<String, _>("value"),
        "label": r.get::<String, _>("label"),
        "description": r.get::<Option<String>, _>("description"),
        "parent_return_reason_id": r.get::<Option<Uuid>, _>("parent_return_reason_id"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, value, label, description, parent_return_reason_id, metadata, \
         created_at, updated_at \
         FROM return_reasons WHERE deleted_at IS NULL ORDER BY created_at ASC \
         LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM return_reasons WHERE deleted_at IS NULL")
            .fetch_one(&*state.db)
            .await?;
    let return_reasons: Vec<_> = rows.iter().map(build_return_reason).collect();
    Ok(Json(serde_json::json!({
        "return_reasons": return_reasons,
        "count": count,
        "offset": p.offset,
        "limit": p.limit,
    })))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, value, label, description, parent_return_reason_id, metadata, \
         created_at, updated_at \
         FROM return_reasons WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Return reason not found".into()))?;
    Ok(Json(serde_json::json!({ "return_reason": build_return_reason(&r) })))
}
