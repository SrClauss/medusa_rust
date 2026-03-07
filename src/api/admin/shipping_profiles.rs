//! Admin shipping_profiles handlers — DB-backed CRUD
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

fn sp_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "name": r.get::<String, _>("name"),
        "type": r.get::<String, _>("type"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "shipping_options": [],
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, name, type, metadata, created_at, updated_at \
         FROM shipping_profiles WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM shipping_profiles WHERE deleted_at IS NULL")
            .fetch_one(&*state.db)
            .await?;
    let shipping_profiles: Vec<_> = rows.iter().map(sp_json).collect();
    Ok(Json(serde_json::json!({"shipping_profiles": shipping_profiles, "count": count, "offset": p.offset, "limit": p.limit})))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, name, type, metadata, created_at, updated_at \
         FROM shipping_profiles WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Shipping profile not found".into()))?;
    Ok(Json(serde_json::json!({"shipping_profile": sp_json(&r)})))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("name required".into()))?;
    let profile_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("default");
    sqlx::query(
        "INSERT INTO shipping_profiles (id, name, type, metadata, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, NOW(), NOW())",
    )
    .bind(id)
    .bind(name)
    .bind(profile_type)
    .bind(payload.get("metadata").cloned())
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id))
        .await
        .map(|r| (StatusCode::CREATED, r))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "UPDATE shipping_profiles SET \
         name = COALESCE($2, name), \
         type = COALESCE($3, type), \
         metadata = COALESCE($4, metadata), \
         updated_at = NOW() \
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(payload.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("type").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("metadata").cloned())
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE shipping_profiles SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({"id": id, "object": "shipping-profile", "deleted": true})))
}
