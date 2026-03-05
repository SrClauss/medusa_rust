//! Admin customer handlers
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};
#[derive(Debug, Deserialize)]
pub struct ListParams { #[serde(default="d20")] pub limit: i64, #[serde(default)] pub offset: i64 }
fn d20() -> i64 { 20 }

fn cust_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({"id":r.get::<Uuid,_>("id"),"email":r.get::<String,_>("email"),"first_name":r.get::<Option<String>,_>("first_name"),"last_name":r.get::<Option<String>,_>("last_name"),"phone":r.get::<Option<String>,_>("phone"),"has_account":r.get::<bool,_>("has_account"),"metadata":r.get::<Option<serde_json::Value>,_>("metadata"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),"billing_address":null,"shipping_addresses":[],"orders":[]})
}

pub async fn list(State(state): State<AppState>, Query(p): Query<ListParams>) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query("SELECT id, email, first_name, last_name, phone, has_account, metadata, created_at, updated_at FROM customers WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2")
        .bind(p.limit).bind(p.offset).fetch_all(&*state.db).await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM customers WHERE deleted_at IS NULL").fetch_one(&*state.db).await?;
    let customers: Vec<_> = rows.iter().map(|r| cust_json(r)).collect();
    Ok(Json(serde_json::json!({"customers":customers,"count":count,"offset":p.offset,"limit":p.limit})))
}

pub async fn get(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query("SELECT id, email, first_name, last_name, phone, has_account, metadata, created_at, updated_at FROM customers WHERE id = $1 AND deleted_at IS NULL")
        .bind(id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Customer not found".into()))?;
    Ok(Json(serde_json::json!({"customer":cust_json(&r)})))
}

pub async fn create(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let email = payload.get("email").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("email required".into()))?;
    let hash = payload.get("password").and_then(|v| v.as_str()).map(|p| crate::auth::argon::hash_password(p)).transpose()?;
    sqlx::query("INSERT INTO customers (id, email, first_name, last_name, phone, password_hash, has_account, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,true,NOW(),NOW())")
        .bind(id).bind(email).bind(payload.get("first_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("last_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("phone").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(hash).execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await.map(|r| (StatusCode::CREATED, r))
}

pub async fn update(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE customers SET first_name = COALESCE($2, first_name), last_name = COALESCE($3, last_name), phone = COALESCE($4, phone), updated_at = NOW() WHERE id = $1")
        .bind(id).bind(payload.get("first_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("last_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .bind(payload.get("phone").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .execute(&*state.db).await?;
    get(axum::extract::State(state), axum::extract::Path(id)).await
}
