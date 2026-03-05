//! Admin order handlers
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListParams { #[serde(default="d20")] pub limit: i64, #[serde(default)] pub offset: i64, pub q: Option<String>, pub status: Option<String> }
fn d20() -> i64 { 20 }

pub async fn list(State(state): State<AppState>, Query(p): Query<ListParams>) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query!("SELECT id, status, fulfillment_status, payment_status, display_id, cart_id, customer_id, email, billing_address_id, shipping_address_id, region_id, currency_code, tax_rate, canceled_at, metadata, created_at, updated_at FROM orders WHERE 1=1 ORDER BY created_at DESC LIMIT $1 OFFSET $2", p.limit, p.offset)
        .fetch_all(&*state.db).await?;
    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM orders").fetch_one(&*state.db).await?.unwrap_or(0);
    let orders: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({"id":r.id,"status":r.status,"fulfillment_status":r.fulfillment_status,"payment_status":r.payment_status,"display_id":r.display_id,"cart_id":r.cart_id,"customer_id":r.customer_id,"email":r.email,"region_id":r.region_id,"currency_code":r.currency_code,"created_at":r.created_at,"updated_at":r.updated_at,"items":[],"shipping_methods":[],"payments":[],"fulfillments":[],"subtotal":0,"tax_total":0,"shipping_total":0,"discount_total":0,"total":0})).collect();
    Ok(Json(serde_json::json!({"orders":orders,"count":count,"offset":p.offset,"limit":p.limit})))
}

pub async fn get(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query!("SELECT id, status, fulfillment_status, payment_status, display_id, cart_id, customer_id, email, billing_address_id, shipping_address_id, region_id, currency_code, tax_rate, canceled_at, metadata, created_at, updated_at FROM orders WHERE id = $1", id)
        .fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Order not found".into()))?;
    Ok(Json(serde_json::json!({"order":{"id":r.id,"status":r.status,"fulfillment_status":r.fulfillment_status,"payment_status":r.payment_status,"display_id":r.display_id,"cart_id":r.cart_id,"customer_id":r.customer_id,"email":r.email,"region_id":r.region_id,"currency_code":r.currency_code,"created_at":r.created_at,"updated_at":r.updated_at,"items":[],"shipping_methods":[],"payments":[],"fulfillments":[],"subtotal":0,"tax_total":0,"shipping_total":0,"discount_total":0,"total":0}})))
}

pub async fn create(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({"order":{}}))))
}
pub async fn update(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query!("UPDATE orders SET metadata = COALESCE($2, metadata), updated_at = NOW() WHERE id = $1", id, payload.get("metadata").cloned()).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}
pub async fn cancel(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query!("UPDATE orders SET status = 'canceled', canceled_at = NOW(), updated_at = NOW() WHERE id = $1", id).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}
pub async fn complete(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query!("UPDATE orders SET status = 'completed', updated_at = NOW() WHERE id = $1", id).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}
pub async fn archive(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query!("UPDATE orders SET status = 'archived', updated_at = NOW() WHERE id = $1", id).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}
pub async fn create_fulfillment(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let fid = Uuid::new_v4();
    let provider = payload.get("provider_id").and_then(|v| v.as_str()).unwrap_or("manual");
    sqlx::query!("INSERT INTO fulfillments (id, order_id, provider_id, created_at, updated_at) VALUES ($1,$2,$3,NOW(),NOW())", fid, id, provider).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await.map(|r| (StatusCode::OK, r))
}
pub async fn cancel_fulfillment(State(state): State<AppState>, Path((order_id, fulfillment_id)): Path<(Uuid,Uuid)>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query!("UPDATE fulfillments SET canceled_at = NOW(), updated_at = NOW() WHERE id = $1 AND order_id = $2", fulfillment_id, order_id).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(order_id)).await
}
pub async fn create_shipment(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query!("UPDATE orders SET fulfillment_status = 'shipped', updated_at = NOW() WHERE id = $1", id).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}
pub async fn create_refund(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"refund":{"id":Uuid::new_v4(),"order_id":id}})))
}
pub async fn request_return(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"return":{"id":Uuid::new_v4(),"order_id":id,"status":"requested"}})))
}
pub async fn create_swap(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({"swap":{"id":Uuid::new_v4(),"order_id":id}}))))
}
pub async fn create_claim(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({"claim_order":{"id":Uuid::new_v4(),"order_id":id}}))))
}
