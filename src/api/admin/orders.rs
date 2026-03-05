//! Admin order handlers
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};
#[derive(Debug, Deserialize)]
pub struct ListParams { #[serde(default="d20")] pub limit: i64, #[serde(default)] pub offset: i64 }
fn d20() -> i64 { 20 }

fn order_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({"id":r.get::<Uuid,_>("id"),"status":r.get::<String,_>("status"),"fulfillment_status":r.get::<String,_>("fulfillment_status"),"payment_status":r.get::<String,_>("payment_status"),"display_id":r.get::<i32,_>("display_id"),"cart_id":r.get::<Option<Uuid>,_>("cart_id"),"customer_id":r.get::<Uuid,_>("customer_id"),"email":r.get::<String,_>("email"),"region_id":r.get::<Uuid,_>("region_id"),"currency_code":r.get::<String,_>("currency_code"),"metadata":r.get::<Option<serde_json::Value>,_>("metadata"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),"items":[],"shipping_methods":[],"payments":[],"fulfillments":[],"subtotal":0,"tax_total":0,"shipping_total":0,"discount_total":0,"total":0})
}

pub async fn list(State(state): State<AppState>, Query(p): Query<ListParams>) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT id, status, fulfillment_status, payment_status, display_id, cart_id, customer_id, email, region_id, currency_code, metadata, created_at, updated_at FROM orders ORDER BY created_at DESC LIMIT $1 OFFSET $2")
        .bind(p.limit).bind(p.offset).fetch_all(&*state.db).await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM orders").fetch_one(&*state.db).await?;
    let orders: Vec<_> = rows.iter().map(|r| order_json(r)).collect();
    Ok(Json(serde_json::json!({"orders":orders,"count":count,"offset":p.offset,"limit":p.limit})))
}

pub async fn get(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("SELECT id, status, fulfillment_status, payment_status, display_id, cart_id, customer_id, email, region_id, currency_code, metadata, created_at, updated_at FROM orders WHERE id = $1")
        .bind(id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Order not found".into()))?;
    Ok(Json(serde_json::json!({"order":order_json(&r)})))
}

pub async fn create(_: State<AppState>, _: Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> { Ok((StatusCode::CREATED, Json(serde_json::json!({"order":{}})))) }

pub async fn update(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE orders SET metadata = COALESCE($2, metadata), updated_at = NOW() WHERE id = $1").bind(id).bind(payload.get("metadata").cloned()).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn cancel(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE orders SET status = 'canceled', canceled_at = NOW(), updated_at = NOW() WHERE id = $1").bind(id).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn complete(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE orders SET status = 'completed', updated_at = NOW() WHERE id = $1").bind(id).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn archive(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE orders SET status = 'archived', updated_at = NOW() WHERE id = $1").bind(id).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn create_fulfillment(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let fid = Uuid::new_v4();
    let provider = payload.get("provider_id").and_then(|v| v.as_str()).unwrap_or("manual");
    sqlx::query("INSERT INTO fulfillments (id, order_id, provider_id, created_at, updated_at) VALUES ($1,$2,$3,NOW(),NOW())").bind(fid).bind(id).bind(provider).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await.map(|r| (StatusCode::OK, r))
}

pub async fn cancel_fulfillment(State(state): State<AppState>, Path((order_id, fulfillment_id)): Path<(Uuid,Uuid)>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE fulfillments SET canceled_at = NOW(), updated_at = NOW() WHERE id = $1 AND order_id = $2").bind(fulfillment_id).bind(order_id).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(order_id)).await
}

pub async fn create_shipment(State(state): State<AppState>, Path(id): Path<Uuid>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE orders SET fulfillment_status = 'shipped', updated_at = NOW() WHERE id = $1").bind(id).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}

pub async fn create_refund(_: State<AppState>, Path(id): Path<Uuid>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({"refund":{"id":Uuid::new_v4(),"order_id":id}}))) }
pub async fn request_return(_: State<AppState>, Path(id): Path<Uuid>, _: Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({"return":{"id":Uuid::new_v4(),"order_id":id,"status":"requested"}}))) }
pub async fn create_swap(_: State<AppState>, Path(id): Path<Uuid>, _: Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> { Ok((StatusCode::CREATED, Json(serde_json::json!({"swap":{"id":Uuid::new_v4(),"order_id":id}})))) }
pub async fn create_claim(_: State<AppState>, Path(id): Path<Uuid>, _: Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> { Ok((StatusCode::CREATED, Json(serde_json::json!({"claim_order":{"id":Uuid::new_v4(),"order_id":id}})))) }
