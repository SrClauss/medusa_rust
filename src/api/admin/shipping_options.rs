//! Admin shipping_options handlers — full CRUD
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
    pub region_id: Option<Uuid>,
    pub is_return: Option<bool>,
}
fn d20() -> i64 { 20 }

fn so_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "name": r.get::<String, _>("name"),
        "region_id": r.get::<Uuid, _>("region_id"),
        "profile_id": r.get::<Uuid, _>("profile_id"),
        "provider_id": r.get::<String, _>("provider_id"),
        "price_type": r.get::<String, _>("price_type"),
        "amount": r.get::<Option<i64>, _>("amount"),
        "is_return": r.get::<bool, _>("is_return"),
        "admin_only": r.get::<bool, _>("admin_only"),
        "data": r.get::<Option<serde_json::Value>, _>("data"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "requirements": [],
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = if let Some(region_id) = p.region_id {
        sqlx::query(
            "SELECT id, name, region_id, profile_id, provider_id, price_type, amount, is_return, \
             admin_only, data, metadata, created_at, updated_at \
             FROM shipping_options WHERE deleted_at IS NULL AND region_id = $1 \
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(region_id)
        .bind(p.limit)
        .bind(p.offset)
        .fetch_all(&*state.db)
        .await?
    } else {
        sqlx::query(
            "SELECT id, name, region_id, profile_id, provider_id, price_type, amount, is_return, \
             admin_only, data, metadata, created_at, updated_at \
             FROM shipping_options WHERE deleted_at IS NULL \
             ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(p.limit)
        .bind(p.offset)
        .fetch_all(&*state.db)
        .await?
    };
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM shipping_options WHERE deleted_at IS NULL")
        .fetch_one(&*state.db)
        .await?;
    let shipping_options: Vec<_> = rows.iter().map(|r| so_json(r)).collect();
    Ok(Json(serde_json::json!({ "shipping_options": shipping_options, "count": count, "offset": p.offset, "limit": p.limit })))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, name, region_id, profile_id, provider_id, price_type, amount, is_return, \
         admin_only, data, metadata, created_at, updated_at \
         FROM shipping_options WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Shipping option not found".into()))?;
    Ok(Json(serde_json::json!({ "shipping_option": so_json(&r) })))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let name = payload.get("name").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("name required".into()))?;
    let region_id: Uuid = payload.get("region_id").and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::BadRequest("region_id required".into()))?;
    let provider_id = payload.get("provider_id").and_then(|v| v.as_str()).unwrap_or("manual");
    let price_type = payload.get("price_type").and_then(|v| v.as_str()).unwrap_or("flat_rate");
    let amount: Option<i64> = payload.get("amount").and_then(|v| v.as_i64());
    // Get default shipping profile
    let profile_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM shipping_profiles WHERE type = 'default' LIMIT 1",
    )
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest("No default shipping profile found. Create one first.".into()))?;
    let r = sqlx::query(
        "INSERT INTO shipping_options (id, name, region_id, profile_id, provider_id, price_type, \
         amount, is_return, admin_only, data, metadata, created_at, updated_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,NOW(),NOW()) \
         RETURNING id, name, region_id, profile_id, provider_id, price_type, amount, is_return, \
         admin_only, data, metadata, created_at, updated_at",
    )
    .bind(id)
    .bind(name)
    .bind(region_id)
    .bind(profile_id)
    .bind(provider_id)
    .bind(price_type)
    .bind(amount)
    .bind(payload.get("is_return").and_then(|v| v.as_bool()).unwrap_or(false))
    .bind(payload.get("admin_only").and_then(|v| v.as_bool()).unwrap_or(false))
    .bind(payload.get("data").cloned().unwrap_or_else(|| serde_json::json!({})))
    .bind(payload.get("metadata").cloned())
    .fetch_one(&*state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "shipping_option": so_json(&r) }))))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let amount: Option<i64> = payload.get("amount").and_then(|v| v.as_i64());
    let r = sqlx::query(
        "UPDATE shipping_options SET \
         name = COALESCE($2, name), \
         amount = COALESCE($3, amount), \
         price_type = COALESCE($4, price_type), \
         admin_only = COALESCE($5, admin_only), \
         metadata = COALESCE($6, metadata), \
         updated_at = NOW() \
         WHERE id = $1 AND deleted_at IS NULL \
         RETURNING id, name, region_id, profile_id, provider_id, price_type, amount, is_return, \
         admin_only, data, metadata, created_at, updated_at",
    )
    .bind(id)
    .bind(payload.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(amount)
    .bind(payload.get("price_type").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("admin_only").and_then(|v| v.as_bool()))
    .bind(payload.get("metadata").cloned())
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Shipping option not found".into()))?;
    Ok(Json(serde_json::json!({ "shipping_option": so_json(&r) })))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE shipping_options SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({ "id": id, "object": "shipping-option", "deleted": true })))
}
