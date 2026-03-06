//! Admin tax_rates handlers — full CRUD
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
}
fn d20() -> i64 { 20 }

fn tr_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "rate": r.get::<Option<f64>, _>("rate"),
        "code": r.get::<Option<String>, _>("code"),
        "name": r.get::<String, _>("name"),
        "region_id": r.get::<Uuid, _>("region_id"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "products": [],
        "product_types": [],
        "shipping_options": [],
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = if let Some(region_id) = p.region_id {
        sqlx::query(
            "SELECT id, rate, code, name, region_id, metadata, created_at, updated_at \
             FROM tax_rates WHERE region_id = $1 ORDER BY name LIMIT $2 OFFSET $3",
        )
        .bind(region_id)
        .bind(p.limit)
        .bind(p.offset)
        .fetch_all(&*state.db)
        .await?
    } else {
        sqlx::query(
            "SELECT id, rate, code, name, region_id, metadata, created_at, updated_at \
             FROM tax_rates ORDER BY name LIMIT $1 OFFSET $2",
        )
        .bind(p.limit)
        .bind(p.offset)
        .fetch_all(&*state.db)
        .await?
    };
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tax_rates")
        .fetch_one(&*state.db)
        .await?;
    let tax_rates: Vec<_> = rows.iter().map(|r| tr_json(r)).collect();
    Ok(Json(serde_json::json!({ "tax_rates": tax_rates, "count": count, "offset": p.offset, "limit": p.limit })))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, rate, code, name, region_id, metadata, created_at, updated_at \
         FROM tax_rates WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Tax rate not found".into()))?;
    Ok(Json(serde_json::json!({ "tax_rate": tr_json(&r) })))
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
    let rate: Option<f64> = payload.get("rate").and_then(|v| v.as_f64());
    let r = sqlx::query(
        "INSERT INTO tax_rates (id, rate, code, name, region_id, metadata, created_at, updated_at) \
         VALUES ($1,$2,$3,$4,$5,$6,NOW(),NOW()) \
         RETURNING id, rate, code, name, region_id, metadata, created_at, updated_at",
    )
    .bind(id)
    .bind(rate)
    .bind(payload.get("code").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(name)
    .bind(region_id)
    .bind(payload.get("metadata").cloned())
    .fetch_one(&*state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "tax_rate": tr_json(&r) }))))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rate: Option<f64> = payload.get("rate").and_then(|v| v.as_f64());
    let r = sqlx::query(
        "UPDATE tax_rates SET \
         name = COALESCE($2, name), \
         rate = COALESCE($3, rate), \
         code = COALESCE($4, code), \
         metadata = COALESCE($5, metadata), \
         updated_at = NOW() \
         WHERE id = $1 \
         RETURNING id, rate, code, name, region_id, metadata, created_at, updated_at",
    )
    .bind(id)
    .bind(payload.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(rate)
    .bind(payload.get("code").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("metadata").cloned())
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Tax rate not found".into()))?;
    Ok(Json(serde_json::json!({ "tax_rate": tr_json(&r) })))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let deleted = sqlx::query("DELETE FROM tax_rates WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound("Tax rate not found".into()));
    }
    Ok(Json(serde_json::json!({ "id": id, "object": "tax-rate", "deleted": true })))
}
