//! Admin gift-card handlers — full CRUD with SQL
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

fn generate_gift_card_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
    let mut gen = || -> String {
        (0..4).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
    };
    format!("{}-{}-{}-{}", gen(), gen(), gen(), gen())
}

fn build_gift_card(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "code": r.get::<String, _>("code"),
        "value": r.get::<i64, _>("value"),
        "balance": r.get::<i64, _>("balance"),
        "region_id": r.get::<Uuid, _>("region_id"),
        "order_id": r.get::<Option<Uuid>, _>("order_id"),
        "is_disabled": r.get::<bool, _>("is_disabled"),
        "ends_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("ends_at"),
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
        "SELECT id, code, value, balance, region_id, order_id, is_disabled, ends_at, \
         metadata, created_at, updated_at \
         FROM gift_cards WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM gift_cards WHERE deleted_at IS NULL")
        .fetch_one(&*state.db)
        .await?;
    let gift_cards: Vec<_> = rows.iter().map(build_gift_card).collect();
    Ok(Json(serde_json::json!({
        "gift_cards": gift_cards,
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
        "SELECT id, code, value, balance, region_id, order_id, is_disabled, ends_at, \
         metadata, created_at, updated_at \
         FROM gift_cards WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Gift card not found".into()))?;
    Ok(Json(serde_json::json!({ "gift_card": build_gift_card(&r) })))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let value: i64 = payload.get("value").and_then(|v| v.as_i64())
        .ok_or_else(|| AppError::BadRequest("value required".into()))?;
    let region_id: Uuid = payload
        .get("region_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::BadRequest("region_id required".into()))?;
    let id = Uuid::new_v4();
    let code = generate_gift_card_code();
    let r = sqlx::query(
        "INSERT INTO gift_cards \
         (id, code, value, balance, region_id, is_disabled, ends_at, metadata, created_at, updated_at) \
         VALUES ($1,$2,$3,$3,$4,$5,$6,$7,NOW(),NOW()) \
         RETURNING id, code, value, balance, region_id, order_id, is_disabled, ends_at, \
         metadata, created_at, updated_at",
    )
    .bind(id)
    .bind(&code)
    .bind(value)
    .bind(region_id)
    .bind(false)
    .bind(
        payload
            .get("ends_at")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok()),
    )
    .bind(payload.get("metadata").cloned())
    .fetch_one(&*state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "gift_card": build_gift_card(&r) }))))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "UPDATE gift_cards SET \
         is_disabled = COALESCE($2, is_disabled), \
         ends_at = COALESCE($3, ends_at), \
         metadata = COALESCE($4, metadata), \
         updated_at = NOW() \
         WHERE id = $1 AND deleted_at IS NULL \
         RETURNING id, code, value, balance, region_id, order_id, is_disabled, ends_at, \
         metadata, created_at, updated_at",
    )
    .bind(id)
    .bind(payload.get("is_disabled").and_then(|v| v.as_bool()))
    .bind(
        payload
            .get("ends_at")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok()),
    )
    .bind(payload.get("metadata").cloned())
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Gift card not found".into()))?;
    Ok(Json(serde_json::json!({ "gift_card": build_gift_card(&r) })))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE gift_cards SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({ "id": id, "object": "gift-card", "deleted": true })))
}
