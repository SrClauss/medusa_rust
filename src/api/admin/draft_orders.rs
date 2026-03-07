//! Admin draft_orders handlers — DB-backed CRUD
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

fn draft_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "status": r.get::<String, _>("status"),
        "display_id": r.get::<i32, _>("display_id"),
        "cart_id": r.get::<Option<Uuid>, _>("cart_id"),
        "order_id": r.get::<Option<Uuid>, _>("order_id"),
        "canceled_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("canceled_at"),
        "completed_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("completed_at"),
        "no_notification_order": r.get::<Option<bool>, _>("no_notification_order"),
        "idempotency_key": r.get::<Option<String>, _>("idempotency_key"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "cart": null,
        "order": null,
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, status, display_id, cart_id, order_id, canceled_at, completed_at, \
         no_notification_order, idempotency_key, metadata, created_at, updated_at \
         FROM draft_orders ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM draft_orders")
        .fetch_one(&*state.db)
        .await?;
    let draft_orders: Vec<_> = rows.iter().map(draft_json).collect();
    Ok(Json(serde_json::json!({"draft_orders": draft_orders, "count": count, "offset": p.offset, "limit": p.limit})))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, status, display_id, cart_id, order_id, canceled_at, completed_at, \
         no_notification_order, idempotency_key, metadata, created_at, updated_at \
         FROM draft_orders WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Draft order not found".into()))?;
    Ok(Json(serde_json::json!({"draft_order": draft_json(&r)})))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let display_id: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(display_id), 0) + 1 FROM draft_orders")
        .fetch_one(&*state.db)
        .await
        .unwrap_or(1);
    sqlx::query(
        "INSERT INTO draft_orders (id, status, display_id, no_notification_order, metadata, created_at, updated_at) \
         VALUES ($1, 'open', $2, $3, $4, NOW(), NOW())",
    )
    .bind(id)
    .bind(display_id)
    .bind(payload.get("no_notification_order").and_then(|v| v.as_bool()))
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
        "UPDATE draft_orders SET \
         no_notification_order = COALESCE($2, no_notification_order), \
         metadata = COALESCE($3, metadata), \
         updated_at = NOW() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(payload.get("no_notification_order").and_then(|v| v.as_bool()))
    .bind(payload.get("metadata").cloned())
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("DELETE FROM draft_orders WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({"id": id, "object": "draft-order", "deleted": true})))
}

pub async fn add_line_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    // Draft order must have a cart; create one if missing
    let cart_id: Option<Uuid> = sqlx::query_scalar("SELECT cart_id FROM draft_orders WHERE id = $1")
        .bind(id)
        .fetch_optional(&*state.db)
        .await?
        .flatten();
    let cart_id = if let Some(cid) = cart_id {
        cid
    } else {
        let cid = Uuid::new_v4();
        sqlx::query("INSERT INTO carts (id, created_at, updated_at) VALUES ($1, NOW(), NOW())")
            .bind(cid)
            .execute(&*state.db)
            .await?;
        sqlx::query("UPDATE draft_orders SET cart_id = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(cid)
            .execute(&*state.db)
            .await?;
        cid
    };
    let line_id = Uuid::new_v4();
    let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("Item");
    let unit_price: i64 = payload.get("unit_price").and_then(|v| v.as_i64()).unwrap_or(0);
    let quantity: i32 = payload.get("quantity").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
    sqlx::query(
        "INSERT INTO line_items (id, cart_id, title, unit_price, quantity, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
    )
    .bind(line_id)
    .bind(cart_id)
    .bind(title)
    .bind(unit_price)
    .bind(quantity)
    .execute(&*state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "draft_order": {"id": id, "cart_id": cart_id},
        "line_item": {"id": line_id, "title": title, "unit_price": unit_price, "quantity": quantity}
    }))))
}

pub async fn update_line_item(
    State(state): State<AppState>,
    Path((_id, line_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "UPDATE line_items SET quantity = COALESCE($2, quantity), unit_price = COALESCE($3, unit_price), updated_at = NOW() WHERE id = $1",
    )
    .bind(line_id)
    .bind(payload.get("quantity").and_then(|v| v.as_i64()).map(|q| q as i32))
    .bind(payload.get("unit_price").and_then(|v| v.as_i64()))
    .execute(&*state.db)
    .await?;
    Ok(Json(serde_json::json!({"line_item": {"id": line_id}})))
}

pub async fn delete_line_item(
    State(state): State<AppState>,
    Path((_id, line_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("DELETE FROM line_items WHERE id = $1")
        .bind(line_id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({"id": line_id, "object": "line-item", "deleted": true})))
}

pub async fn register_payment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE draft_orders SET status = 'completed', completed_at = NOW(), updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}
