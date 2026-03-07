//! Admin sales_channels handlers — DB-backed CRUD
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

fn sc_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "name": r.get::<String, _>("name"),
        "description": r.get::<Option<String>, _>("description"),
        "is_disabled": r.get::<bool, _>("is_disabled"),
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
        "SELECT id, name, description, is_disabled, metadata, created_at, updated_at \
         FROM sales_channels WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sales_channels WHERE deleted_at IS NULL")
            .fetch_one(&*state.db)
            .await?;
    let sales_channels: Vec<_> = rows.iter().map(sc_json).collect();
    Ok(Json(serde_json::json!({"sales_channels": sales_channels, "count": count, "offset": p.offset, "limit": p.limit})))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, name, description, is_disabled, metadata, created_at, updated_at \
         FROM sales_channels WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Sales channel not found".into()))?;
    Ok(Json(serde_json::json!({"sales_channel": sc_json(&r)})))
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
        "INSERT INTO sales_channels (id, name, description, is_disabled, metadata, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
    )
    .bind(id)
    .bind(name)
    .bind(payload.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("is_disabled").and_then(|v| v.as_bool()).unwrap_or(false))
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
        "UPDATE sales_channels SET \
         name = COALESCE($2, name), \
         description = COALESCE($3, description), \
         is_disabled = COALESCE($4, is_disabled), \
         metadata = COALESCE($5, metadata), \
         updated_at = NOW() \
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(payload.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("is_disabled").and_then(|v| v.as_bool()))
    .bind(payload.get("metadata").cloned())
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE sales_channels SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({"id": id, "object": "sales-channel", "deleted": true})))
}

pub async fn add_products(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(product_ids) = payload.get("product_ids").and_then(|v| v.as_array()) {
        for pid_val in product_ids {
            if let Some(pid_str) = pid_val.as_str() {
                if let Ok(pid) = uuid::Uuid::parse_str(pid_str) {
                    let _ = sqlx::query(
                        "INSERT INTO sales_channel_products (sales_channel_id, product_id) \
                         VALUES ($1, $2) ON CONFLICT DO NOTHING",
                    )
                    .bind(id)
                    .bind(pid)
                    .execute(&*state.db)
                    .await;
                }
            }
        }
    }
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn remove_products(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(product_ids) = payload.get("product_ids").and_then(|v| v.as_array()) {
        for pid_val in product_ids {
            if let Some(pid_str) = pid_val.as_str() {
                if let Ok(pid) = uuid::Uuid::parse_str(pid_str) {
                    let _ = sqlx::query(
                        "DELETE FROM sales_channel_products WHERE sales_channel_id = $1 AND product_id = $2",
                    )
                    .bind(id)
                    .bind(pid)
                    .execute(&*state.db)
                    .await;
                }
            }
        }
    }
    get(axum::extract::State(state), axum::extract::Path(id)).await
}
