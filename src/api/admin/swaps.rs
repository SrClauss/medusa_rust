//! Admin swaps handlers — DB-backed CRUD
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

fn swap_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "fulfillment_status": r.get::<String, _>("fulfillment_status"),
        "payment_status": r.get::<String, _>("payment_status"),
        "order_id": r.get::<Uuid, _>("order_id"),
        "difference_due": r.get::<Option<i64>, _>("difference_due"),
        "cart_id": r.get::<Option<Uuid>, _>("cart_id"),
        "confirmed_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("confirmed_at"),
        "canceled_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("canceled_at"),
        "no_notification": r.get::<Option<bool>, _>("no_notification"),
        "allow_backorder": r.get::<bool, _>("allow_backorder"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "return_order": null,
        "additional_items": [],
        "fulfillments": [],
        "payment": null,
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, fulfillment_status, payment_status, order_id, difference_due, cart_id, \
         confirmed_at, canceled_at, no_notification, allow_backorder, created_at, updated_at \
         FROM swaps ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM swaps")
        .fetch_one(&*state.db)
        .await?;
    let swaps: Vec<_> = rows.iter().map(swap_json).collect();
    Ok(Json(serde_json::json!({"swaps": swaps, "count": count, "offset": p.offset, "limit": p.limit})))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, fulfillment_status, payment_status, order_id, difference_due, cart_id, \
         confirmed_at, canceled_at, no_notification, allow_backorder, created_at, updated_at \
         FROM swaps WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Swap not found".into()))?;
    Ok(Json(serde_json::json!({"swap": swap_json(&r)})))
}
