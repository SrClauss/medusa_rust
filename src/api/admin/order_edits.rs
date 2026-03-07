//! Admin order_edits handlers — DB-backed CRUD
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
    pub order_id: Option<Uuid>,
}
fn d20() -> i64 { 20 }

fn oe_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "order_id": r.get::<Uuid, _>("order_id"),
        "status": r.get::<String, _>("status"),
        "internal_note": r.get::<Option<String>, _>("internal_note"),
        "declined_reason": r.get::<Option<String>, _>("declined_reason"),
        "requested_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("requested_at"),
        "confirmed_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("confirmed_at"),
        "declined_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("declined_at"),
        "canceled_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("canceled_at"),
        "difference_due": r.get::<Option<i64>, _>("difference_due"),
        "shipping_total": r.get::<i64, _>("shipping_total"),
        "discount_total": r.get::<i64, _>("discount_total"),
        "tax_total": r.get::<i64, _>("tax_total"),
        "subtotal": r.get::<i64, _>("subtotal"),
        "total": r.get::<i64, _>("total"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "changes": [],
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = if let Some(order_id) = p.order_id {
        sqlx::query(
            "SELECT id, order_id, status, internal_note, declined_reason, requested_at, \
             confirmed_at, declined_at, canceled_at, difference_due, shipping_total, discount_total, \
             tax_total, subtotal, total, metadata, created_at, updated_at \
             FROM order_edits WHERE order_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(order_id)
        .bind(p.limit)
        .bind(p.offset)
        .fetch_all(&*state.db)
        .await?
    } else {
        sqlx::query(
            "SELECT id, order_id, status, internal_note, declined_reason, requested_at, \
             confirmed_at, declined_at, canceled_at, difference_due, shipping_total, discount_total, \
             tax_total, subtotal, total, metadata, created_at, updated_at \
             FROM order_edits ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(p.limit)
        .bind(p.offset)
        .fetch_all(&*state.db)
        .await?
    };
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM order_edits")
        .fetch_one(&*state.db)
        .await?;
    let order_edits: Vec<_> = rows.iter().map(oe_json).collect();
    Ok(Json(serde_json::json!({"order_edits": order_edits, "count": count, "offset": p.offset, "limit": p.limit})))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, order_id, status, internal_note, declined_reason, requested_at, \
         confirmed_at, declined_at, canceled_at, difference_due, shipping_total, discount_total, \
         tax_total, subtotal, total, metadata, created_at, updated_at \
         FROM order_edits WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Order edit not found".into()))?;
    Ok(Json(serde_json::json!({"order_edit": oe_json(&r)})))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let order_id = payload
        .get("order_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::BadRequest("order_id required".into()))?;
    sqlx::query(
        "INSERT INTO order_edits (id, order_id, status, internal_note, metadata, created_at, updated_at) \
         VALUES ($1, $2, 'created', $3, $4, NOW(), NOW())",
    )
    .bind(id)
    .bind(order_id)
    .bind(payload.get("internal_note").and_then(|v| v.as_str()).map(|s| s.to_string()))
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
        "UPDATE order_edits SET \
         internal_note = COALESCE($2, internal_note), \
         metadata = COALESCE($3, metadata), \
         updated_at = NOW() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(payload.get("internal_note").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .bind(payload.get("metadata").cloned())
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("DELETE FROM order_edits WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({"id": id, "object": "order-edit", "deleted": true})))
}

pub async fn request(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "UPDATE order_edits SET status = 'requested', requested_at = NOW(), updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn confirm(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "UPDATE order_edits SET status = 'confirmed', confirmed_at = NOW(), updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn decline(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "UPDATE order_edits SET status = 'declined', declined_at = NOW(), \
         declined_reason = COALESCE($2, declined_reason), updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .bind(payload.get("declined_reason").and_then(|v| v.as_str()).map(|s| s.to_string()))
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "UPDATE order_edits SET status = 'canceled', canceled_at = NOW(), updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn add_line_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let change_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO order_item_changes (id, type, order_edit_id, metadata, created_at, updated_at) \
         VALUES ($1, 'item_add', $2, $3, NOW(), NOW())",
    )
    .bind(change_id)
    .bind(id)
    .bind(payload.get("metadata").cloned())
    .execute(&*state.db)
    .await?;
    get(axum::extract::State(state), axum::extract::Path(id))
        .await
        .map(|r| (StatusCode::CREATED, r))
}

pub async fn delete_item_change(
    State(state): State<AppState>,
    Path((_oe_id, change_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("DELETE FROM order_item_changes WHERE id = $1")
        .bind(change_id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({"id": change_id, "object": "item-change", "deleted": true})))
}

pub async fn update_line_item(
    State(state): State<AppState>,
    Path((oe_id, _item_id)): Path<(Uuid, Uuid)>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    get(axum::extract::State(state), axum::extract::Path(oe_id)).await
}
