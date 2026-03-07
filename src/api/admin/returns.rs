//! Admin returns handlers — list and receive with SQL
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
    pub order_id: Option<Uuid>,
}
fn d20() -> i64 { 20 }

fn build_return(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "status": r.get::<String, _>("status"),
        "refund_amount": r.get::<Option<i64>, _>("refund_amount"),
        "order_id": r.get::<Option<Uuid>, _>("order_id"),
        "swap_id": r.get::<Option<Uuid>, _>("swap_id"),
        "received_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("received_at"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (rows, count) = if let Some(order_id) = p.order_id {
        let rows = sqlx::query(
            "SELECT id, status, refund_amount, order_id, swap_id, received_at, metadata, \
             created_at, updated_at \
             FROM returns WHERE order_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(order_id)
        .bind(p.limit)
        .bind(p.offset)
        .fetch_all(&*state.db)
        .await?;
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM returns WHERE order_id = $1")
                .bind(order_id)
                .fetch_one(&*state.db)
                .await?;
        (rows, count)
    } else {
        let rows = sqlx::query(
            "SELECT id, status, refund_amount, order_id, swap_id, received_at, metadata, \
             created_at, updated_at \
             FROM returns ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(p.limit)
        .bind(p.offset)
        .fetch_all(&*state.db)
        .await?;
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM returns")
            .fetch_one(&*state.db)
            .await?;
        (rows, count)
    };
    let returns: Vec<_> = rows.iter().map(build_return).collect();
    Ok(Json(serde_json::json!({
        "returns": returns,
        "count": count,
        "offset": p.offset,
        "limit": p.limit,
    })))
}

pub async fn receive(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "UPDATE returns SET status = 'received', received_at = NOW(), updated_at = NOW() \
         WHERE id = $1 \
         RETURNING id, status, refund_amount, order_id, swap_id, received_at, metadata, \
         created_at, updated_at",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Return not found".into()))?;
    Ok(Json(serde_json::json!({ "return": build_return(&r) })))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, status, refund_amount, order_id, swap_id, received_at, metadata, \
         created_at, updated_at FROM returns WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Return not found".into()))?;
    Ok(Json(serde_json::json!({ "return": build_return(&r) })))
}

pub async fn cancel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "UPDATE returns SET status = 'canceled', updated_at = NOW() \
         WHERE id = $1 \
         RETURNING id, status, refund_amount, order_id, swap_id, received_at, metadata, \
         created_at, updated_at",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Return not found".into()))?;
    Ok(Json(serde_json::json!({ "return": build_return(&r) })))
}

pub async fn receive_items(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "return": {
            "id": id,
            "status": "requires_action",
            "items": [],
        }
    })))
}

pub async fn confirm_receive(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "return": {
            "id": id,
            "status": "received",
        }
    })))
}

pub async fn request(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "return": {
            "id": id,
            "status": "requested",
        }
    })))
}

pub async fn add_shipping_method(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "return": {
            "id": id,
            "shipping_methods": [],
        }
    })))
}

pub async fn dismiss_items(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "return": {
            "id": id,
            "items": [],
        }
    })))
}
