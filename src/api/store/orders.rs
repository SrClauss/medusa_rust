//! Store order handlers
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct OrderParams {
    pub cart_id: Option<Uuid>,
    pub display_id: Option<i32>,
    pub email: Option<String>,
}

fn order_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "status": r.get::<String, _>("status"),
        "fulfillment_status": r.get::<String, _>("fulfillment_status"),
        "payment_status": r.get::<String, _>("payment_status"),
        "display_id": r.get::<i32, _>("display_id"),
        "cart_id": r.get::<Option<Uuid>, _>("cart_id"),
        "customer_id": r.get::<Uuid, _>("customer_id"),
        "email": r.get::<String, _>("email"),
        "region_id": r.get::<Uuid, _>("region_id"),
        "currency_code": r.get::<String, _>("currency_code"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "items": [],
        "shipping_methods": [],
        "payments": [],
        "fulfillments": [],
        "subtotal": 0,
        "tax_total": 0,
        "shipping_total": 0,
        "discount_total": 0,
        "total": 0,
    })
}

pub async fn get_order(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("SELECT id, status, fulfillment_status, payment_status, display_id, cart_id, customer_id, email, region_id, currency_code, metadata, created_at, updated_at FROM orders WHERE id = $1")
        .bind(id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Order not found".into()))?;
    Ok(Json(serde_json::json!({ "order": order_json(&r) })))
}

pub async fn get_order_by_params(State(state): State<AppState>, Query(p): Query<OrderParams>) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(cart_id) = p.cart_id {
        let r = sqlx::query("SELECT id, status, fulfillment_status, payment_status, display_id, cart_id, customer_id, email, region_id, currency_code, metadata, created_at, updated_at FROM orders WHERE cart_id = $1")
            .bind(cart_id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Order not found".into()))?;
        return Ok(Json(serde_json::json!({ "order": order_json(&r) })));
    }
    if let (Some(did), Some(email)) = (p.display_id, p.email.as_ref()) {
        let r = sqlx::query("SELECT id, status, fulfillment_status, payment_status, display_id, cart_id, customer_id, email, region_id, currency_code, metadata, created_at, updated_at FROM orders WHERE display_id = $1 AND email = $2")
            .bind(did).bind(email).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Order not found".into()))?;
        return Ok(Json(serde_json::json!({ "order": order_json(&r) })));
    }
    Err(AppError::BadRequest("Provide cart_id, or display_id + email".into()))
}

pub async fn get_orders_batch(State(state): State<AppState>, Query(p): Query<OrderParams>) -> Result<Json<serde_json::Value>, AppError> {
    get_order_by_params(axum::extract::State(state), axum::extract::Query(p)).await
}
