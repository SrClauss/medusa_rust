//! Admin stores handlers — DB-backed CRUD
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

fn store_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "name": r.get::<String, _>("name"),
        "default_currency_code": r.get::<Option<String>, _>("default_currency_code"),
        "swap_link_template": r.get::<Option<String>, _>("swap_link_template"),
        "payment_link_template": r.get::<Option<String>, _>("payment_link_template"),
        "invite_link_template": r.get::<Option<String>, _>("invite_link_template"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "currencies": [],
        "default_currency": null,
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, name, default_currency_code, swap_link_template, payment_link_template, \
         invite_link_template, metadata, created_at, updated_at \
         FROM stores ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM stores")
        .fetch_one(&*state.db)
        .await?;
    let stores: Vec<_> = rows.iter().map(store_json).collect();
    Ok(Json(serde_json::json!({"stores": stores, "count": count, "offset": p.offset, "limit": p.limit})))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, name, default_currency_code, swap_link_template, payment_link_template, \
         invite_link_template, metadata, created_at, updated_at \
         FROM stores WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Store not found".into()))?;
    Ok(Json(serde_json::json!({"store": store_json(&r)})))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "UPDATE stores SET \
         name = COALESCE($2, name), \
         default_currency_code = COALESCE($3, default_currency_code), \
         swap_link_template = COALESCE($4, swap_link_template), \
         payment_link_template = COALESCE($5, payment_link_template), \
         invite_link_template = COALESCE($6, invite_link_template), \
         metadata = COALESCE($7, metadata), \
         updated_at = NOW() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(payload.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("default_currency_code").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("swap_link_template").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("payment_link_template").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("invite_link_template").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("metadata").cloned())
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}
