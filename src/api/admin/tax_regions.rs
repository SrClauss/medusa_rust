//! Admin tax_regions handlers — DB-backed CRUD
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

fn tr_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "provider_id": r.get::<Option<String>, _>("provider_id"),
        "country_code": r.get::<String, _>("country_code"),
        "province_code": r.get::<Option<String>, _>("province_code"),
        "parent_id": r.get::<Option<Uuid>, _>("parent_id"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "tax_rates": [],
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, provider_id, country_code, province_code, parent_id, metadata, created_at, updated_at \
         FROM tax_regions WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM tax_regions WHERE deleted_at IS NULL")
            .fetch_one(&*state.db)
            .await?;
    let tax_regions: Vec<_> = rows.iter().map(tr_json).collect();
    Ok(Json(serde_json::json!({"tax_regions": tax_regions, "count": count, "offset": p.offset, "limit": p.limit})))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, provider_id, country_code, province_code, parent_id, metadata, created_at, updated_at \
         FROM tax_regions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Tax region not found".into()))?;
    Ok(Json(serde_json::json!({"tax_region": tr_json(&r)})))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let country_code = payload
        .get("country_code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("country_code required".into()))?;
    let parent_id = payload
        .get("parent_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok());
    sqlx::query(
        "INSERT INTO tax_regions (id, provider_id, country_code, province_code, parent_id, metadata, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())",
    )
    .bind(id)
    .bind(payload.get("provider_id").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(country_code)
    .bind(payload.get("province_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(parent_id)
    .bind(payload.get("metadata").cloned())
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id))
        .await
        .map(|r| (StatusCode::CREATED, r))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE tax_regions SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({"id": id, "object": "tax-region", "deleted": true})))
}
