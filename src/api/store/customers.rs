//! Store customer handlers — MedusaJS v1 compatible
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use uuid::Uuid;
use crate::{auth::{argon::hash_password, jwt::AuthCustomer}, error::AppError, state::AppState};

async fn fetch_customer(state: &AppState, id: Uuid) -> Result<serde_json::Value, AppError> {
    let r = sqlx::query!(
        "SELECT id, email, first_name, last_name, phone, has_account, billing_address_id, metadata, created_at, updated_at FROM customers WHERE id = $1 AND deleted_at IS NULL",
        id
    ).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Customer not found".into()))?;

    let addresses = sqlx::query!(
        "SELECT id, first_name, last_name, phone, company, address_1, address_2, city, country_code, province, postal_code, metadata, created_at, updated_at FROM addresses WHERE customer_id = $1",
        id
    ).fetch_all(&*state.db).await?
    .into_iter()
    .map(|a| serde_json::json!({"id":a.id,"first_name":a.first_name,"last_name":a.last_name,"phone":a.phone,"company":a.company,"address_1":a.address_1,"address_2":a.address_2,"city":a.city,"country_code":a.country_code,"province":a.province,"postal_code":a.postal_code,"metadata":a.metadata,"created_at":a.created_at,"updated_at":a.updated_at}))
    .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "id": r.id,
        "email": r.email,
        "first_name": r.first_name,
        "last_name": r.last_name,
        "phone": r.phone,
        "has_account": r.has_account,
        "billing_address_id": r.billing_address_id,
        "billing_address": null,
        "shipping_addresses": addresses,
        "metadata": r.metadata,
        "created_at": r.created_at,
        "updated_at": r.updated_at,
    }))
}

pub async fn get_me(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>) -> Result<Json<serde_json::Value>, AppError> {
    let id: Uuid = auth.0.sub.parse().unwrap_or_default();
    Ok(Json(serde_json::json!({ "customer": fetch_customer(&state, id).await? })))
}

pub async fn create_customer(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let email = payload.get("email").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("email required".into()))?;
    let password = payload.get("password").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("password required".into()))?;
    let hash = hash_password(password)?;
    let id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO customers (id, email, first_name, last_name, phone, password_hash, has_account, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,true,NOW(),NOW())",
        id, email,
        payload.get("first_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("last_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("phone").and_then(|v| v.as_str()).map(|s| s.to_string()),
        hash,
    ).execute(&*state.db).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "customer": fetch_customer(&state, id).await? }))))
}

pub async fn update_me(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let id: Uuid = auth.0.sub.parse().unwrap_or_default();
    sqlx::query!(
        "UPDATE customers SET first_name = COALESCE($2, first_name), last_name = COALESCE($3, last_name), phone = COALESCE($4, phone), updated_at = NOW() WHERE id = $1",
        id,
        payload.get("first_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("last_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("phone").and_then(|v| v.as_str()).map(|s| s.to_string()),
    ).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({ "customer": fetch_customer(&state, id).await? })))
}

pub async fn request_password_reset(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let _email = payload.get("email").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("email required".into()))?;
    // In production: generate reset token, store it, send email
    Ok(Json(serde_json::json!({})))
}

pub async fn reset_password(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let email = payload.get("email").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("email required".into()))?;
    let password = payload.get("password").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("password required".into()))?;
    let _token = payload.get("token").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("token required".into()))?;
    let hash = hash_password(password)?;
    sqlx::query!("UPDATE customers SET password_hash = $1, updated_at = NOW() WHERE email = $2", hash, email).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({})))
}

#[derive(Debug, Deserialize)]
pub struct OrdersParams { #[serde(default="d10")] pub limit: i64, #[serde(default)] pub offset: i64 }
fn d10() -> i64 { 10 }

pub async fn list_orders(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>, Query(p): Query<OrdersParams>) -> Result<Json<serde_json::Value>, AppError> {
    let id: Uuid = auth.0.sub.parse().unwrap_or_default();
    let orders = sqlx::query!(
        "SELECT id, status, display_id, currency_code, created_at, updated_at FROM orders WHERE customer_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        id, p.limit, p.offset
    ).fetch_all(&*state.db).await?
    .into_iter()
    .map(|r| serde_json::json!({"id":r.id,"status":r.status,"display_id":r.display_id,"currency_code":r.currency_code,"created_at":r.created_at,"updated_at":r.updated_at,"items":[],"total":0}))
    .collect::<Vec<_>>();
    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM orders WHERE customer_id = $1", id).fetch_one(&*state.db).await?.unwrap_or(0);
    Ok(Json(serde_json::json!({"orders":orders,"count":count,"offset":p.offset,"limit":p.limit})))
}

pub async fn list_addresses(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>) -> Result<Json<serde_json::Value>, AppError> {
    let id: Uuid = auth.0.sub.parse().unwrap_or_default();
    let addresses = sqlx::query!("SELECT id, first_name, last_name, phone, company, address_1, address_2, city, country_code, province, postal_code, metadata, created_at, updated_at FROM addresses WHERE customer_id = $1", id)
        .fetch_all(&*state.db).await?.into_iter()
        .map(|a| serde_json::json!({"id":a.id,"first_name":a.first_name,"last_name":a.last_name,"phone":a.phone,"company":a.company,"address_1":a.address_1,"address_2":a.address_2,"city":a.city,"country_code":a.country_code,"province":a.province,"postal_code":a.postal_code,"metadata":a.metadata,"created_at":a.created_at,"updated_at":a.updated_at}))
        .collect::<Vec<_>>();
    let count = addresses.len() as i64;
    Ok(Json(serde_json::json!({"addresses":addresses,"count":count})))
}

pub async fn add_address(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let cid: Uuid = auth.0.sub.parse().unwrap_or_default();
    let addr = payload.get("address").unwrap_or(&payload);
    let aid = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO addresses (id, customer_id, first_name, last_name, phone, company, address_1, address_2, city, country_code, province, postal_code, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,NOW(),NOW())",
        aid, cid,
        addr.get("first_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        addr.get("last_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        addr.get("phone").and_then(|v| v.as_str()).map(|s| s.to_string()),
        addr.get("company").and_then(|v| v.as_str()).map(|s| s.to_string()),
        addr.get("address_1").and_then(|v| v.as_str()).map(|s| s.to_string()),
        addr.get("address_2").and_then(|v| v.as_str()).map(|s| s.to_string()),
        addr.get("city").and_then(|v| v.as_str()).map(|s| s.to_string()),
        addr.get("country_code").and_then(|v| v.as_str()).map(|s| s.to_string()),
        addr.get("province").and_then(|v| v.as_str()).map(|s| s.to_string()),
        addr.get("postal_code").and_then(|v| v.as_str()).map(|s| s.to_string()),
    ).execute(&*state.db).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "customer": fetch_customer(&state, cid).await? }))))
}

pub async fn get_address(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>, Path(address_id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let cid: Uuid = auth.0.sub.parse().unwrap_or_default();
    let a = sqlx::query!("SELECT id, first_name, last_name, phone, company, address_1, address_2, city, country_code, province, postal_code, metadata, created_at, updated_at FROM addresses WHERE id = $1 AND customer_id = $2", address_id, cid)
        .fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Address not found".into()))?;
    Ok(Json(serde_json::json!({"address":{"id":a.id,"first_name":a.first_name,"last_name":a.last_name,"phone":a.phone,"company":a.company,"address_1":a.address_1,"address_2":a.address_2,"city":a.city,"country_code":a.country_code,"province":a.province,"postal_code":a.postal_code,"metadata":a.metadata,"created_at":a.created_at,"updated_at":a.updated_at}})))
}

pub async fn update_address(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>, Path(address_id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let cid: Uuid = auth.0.sub.parse().unwrap_or_default();
    sqlx::query!("UPDATE addresses SET first_name = COALESCE($3, first_name), last_name = COALESCE($4, last_name), phone = COALESCE($5, phone), company = COALESCE($6, company), address_1 = COALESCE($7, address_1), address_2 = COALESCE($8, address_2), city = COALESCE($9, city), country_code = COALESCE($10, country_code), province = COALESCE($11, province), postal_code = COALESCE($12, postal_code), updated_at = NOW() WHERE id = $1 AND customer_id = $2",
        address_id, cid,
        payload.get("first_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("last_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("phone").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("company").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("address_1").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("address_2").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("city").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("country_code").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("province").and_then(|v| v.as_str()).map(|s| s.to_string()),
        payload.get("postal_code").and_then(|v| v.as_str()).map(|s| s.to_string()),
    ).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({ "customer": fetch_customer(&state, cid).await? })))
}

pub async fn delete_address(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>, Path(address_id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let cid: Uuid = auth.0.sub.parse().unwrap_or_default();
    sqlx::query!("DELETE FROM addresses WHERE id = $1 AND customer_id = $2", address_id, cid).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({ "customer": fetch_customer(&state, cid).await? })))
}

pub async fn list_payment_methods(State(_): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"payment_methods":[]})))
}

pub async fn add_payment_method(State(_): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>, Json(_): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({"payment_method":{}}))))
}
