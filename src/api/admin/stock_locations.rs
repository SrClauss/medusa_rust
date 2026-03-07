//! Admin stock_locations handlers — DB-backed CRUD
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

fn sl_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "name": r.get::<String, _>("name"),
        "address": r.get::<Option<serde_json::Value>, _>("address"),
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
        "SELECT id, name, address, metadata, created_at, updated_at \
         FROM stock_locations WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM stock_locations WHERE deleted_at IS NULL")
            .fetch_one(&*state.db)
            .await?;
    let locations: Vec<_> = rows.iter().map(sl_json).collect();
    Ok(Json(serde_json::json!({"stock_locations": locations, "count": count, "offset": p.offset, "limit": p.limit})))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, name, address, metadata, created_at, updated_at \
         FROM stock_locations WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Stock location not found".into()))?;
    Ok(Json(serde_json::json!({"stock_location": sl_json(&r)})))
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
    sqlx::query(
        "INSERT INTO stock_locations (id, name, address, metadata, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, NOW(), NOW())",
    )
    .bind(id)
    .bind(name)
    .bind(payload.get("address").cloned())
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
        "UPDATE stock_locations SET \
         name = COALESCE($2, name), \
         address = COALESCE($3, address), \
         metadata = COALESCE($4, metadata), \
         updated_at = NOW() \
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(payload.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("address").cloned())
    .bind(payload.get("metadata").cloned())
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE stock_locations SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({"id": id, "object": "stock-location", "deleted": true})))
}

pub async fn link_fulfillment_providers(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(providers) = payload.get("fulfillment_provider_ids").and_then(|v| v.as_array()) {
        for pid_val in providers {
            if let Some(pid) = pid_val.as_str() {
                let _ = sqlx::query(
                    "INSERT INTO stock_location_fulfillment_providers (stock_location_id, provider_id) \
                     VALUES ($1, $2) ON CONFLICT DO NOTHING",
                )
                .bind(id)
                .bind(pid)
                .execute(&*state.db)
                .await;
            }
        }
    }
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn link_fulfillment_sets(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(set_ids) = payload.get("fulfillment_set_ids").and_then(|v| v.as_array()) {
        for sid_val in set_ids {
            if let Some(sid_str) = sid_val.as_str() {
                if let Ok(sid) = uuid::Uuid::parse_str(sid_str) {
                    let _ = sqlx::query(
                        "INSERT INTO stock_location_fulfillment_sets (stock_location_id, fulfillment_set_id) \
                         VALUES ($1, $2) ON CONFLICT DO NOTHING",
                    )
                    .bind(id)
                    .bind(sid)
                    .execute(&*state.db)
                    .await;
                }
            }
        }
    }
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn link_sales_channels(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(channel_ids) = payload.get("sales_channel_ids").and_then(|v| v.as_array()) {
        for cid_val in channel_ids {
            if let Some(cid_str) = cid_val.as_str() {
                if let Ok(cid) = uuid::Uuid::parse_str(cid_str) {
                    let _ = sqlx::query(
                        "INSERT INTO stock_location_sales_channels (stock_location_id, sales_channel_id) \
                         VALUES ($1, $2) ON CONFLICT DO NOTHING",
                    )
                    .bind(id)
                    .bind(cid)
                    .execute(&*state.db)
                    .await;
                }
            }
        }
    }
    get(axum::extract::State(state), axum::extract::Path(id)).await
}
