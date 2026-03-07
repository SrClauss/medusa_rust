//! Store customer handlers
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use hex;
use crate::{auth::{argon::hash_password, jwt::AuthCustomer}, error::AppError, state::AppState};

async fn fetch_customer(state: &AppState, id: Uuid) -> Result<serde_json::Value, AppError> {
    let r = sqlx::query("SELECT id, email, first_name, last_name, phone, has_account, billing_address_id, metadata, created_at, updated_at FROM customers WHERE id = $1 AND deleted_at IS NULL")
        .bind(id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Customer not found".into()))?;
    let addresses = sqlx::query("SELECT id, first_name, last_name, phone, company, address_1, address_2, city, country_code, province, postal_code, metadata, created_at, updated_at FROM addresses WHERE customer_id = $1")
        .bind(id).fetch_all(&*state.db).await?
        .into_iter().map(|a| serde_json::json!({"id":a.get::<Uuid,_>("id"),"first_name":a.get::<Option<String>,_>("first_name"),"last_name":a.get::<Option<String>,_>("last_name"),"phone":a.get::<Option<String>,_>("phone"),"company":a.get::<Option<String>,_>("company"),"address_1":a.get::<Option<String>,_>("address_1"),"address_2":a.get::<Option<String>,_>("address_2"),"city":a.get::<Option<String>,_>("city"),"country_code":a.get::<Option<String>,_>("country_code"),"province":a.get::<Option<String>,_>("province"),"postal_code":a.get::<Option<String>,_>("postal_code"),"metadata":a.get::<Option<serde_json::Value>,_>("metadata")})).collect::<Vec<_>>();
    Ok(serde_json::json!({"id":r.get::<Uuid,_>("id"),"email":r.get::<String,_>("email"),"first_name":r.get::<Option<String>,_>("first_name"),"last_name":r.get::<Option<String>,_>("last_name"),"phone":r.get::<Option<String>,_>("phone"),"has_account":r.get::<bool,_>("has_account"),"billing_address_id":r.get::<Option<Uuid>,_>("billing_address_id"),"billing_address":null,"shipping_addresses":addresses,"metadata":r.get::<Option<serde_json::Value>,_>("metadata"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at")}))
}

pub async fn get_me(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>) -> Result<Json<serde_json::Value>, AppError> {
    let id: Uuid = auth.0.sub.parse().unwrap_or_default();
    Ok(Json(serde_json::json!({"customer":fetch_customer(&state, id).await?})))
}

pub async fn create_customer(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let email = payload.get("email").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("email required".into()))?;
    let password = payload.get("password").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("password required".into()))?;
    let hash = hash_password(password)?;
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO customers (id, email, first_name, last_name, phone, password_hash, has_account, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,true,NOW(),NOW())")
        .bind(id).bind(email).bind(payload.get("first_name").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("last_name").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("phone").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(hash).execute(&*state.db).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"customer":fetch_customer(&state, id).await?}))))
}

pub async fn update_me(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let id: Uuid = auth.0.sub.parse().unwrap_or_default();
    sqlx::query("UPDATE customers SET first_name = COALESCE($2, first_name), last_name = COALESCE($3, last_name), phone = COALESCE($4, phone), updated_at = NOW() WHERE id = $1")
        .bind(id).bind(payload.get("first_name").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("last_name").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("phone").and_then(|v| v.as_str()).map(|s| s.to_string())).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"customer":fetch_customer(&state, id).await?})))
}

pub async fn request_password_reset(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let email = payload.get("email").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("email required".into()))?;
    // ensure customer exists
    let exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM customers WHERE email = $1 AND deleted_at IS NULL")
        .bind(email).fetch_optional(&*state.db).await?;
    if exists.is_none() {
        return Err(AppError::NotFound("Customer not found".into()));
    }
    // generate simple token (not persisted)
    let token: String = hex::encode(rand::random::<[u8;16]>());
    // log the token so it can be picked up by tests or a dev
    tracing::info!("password reset token for {}: {}", email, token);
    // in a full implementation we'd store and send email; for now return token for testing
    Ok(Json(serde_json::json!({"email":email,"token":token})))
}

pub async fn reset_password(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let email = payload.get("email").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("email required".into()))?;
    let password = payload.get("password").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("password required".into()))?;
    let hash = hash_password(password)?;
    sqlx::query("UPDATE customers SET password_hash = $1, updated_at = NOW() WHERE email = $2").bind(hash).bind(email).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({})))
}

#[derive(Debug, Deserialize)]
pub struct OrdersParams { #[serde(default="d10")] pub limit: i64, #[serde(default)] pub offset: i64 }
fn d10() -> i64 { 10 }

pub async fn list_orders(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>, Query(p): Query<OrdersParams>) -> Result<Json<serde_json::Value>, AppError> {
    let id: Uuid = auth.0.sub.parse().unwrap_or_default();
    let orders = sqlx::query("SELECT id, status, fulfillment_status, payment_status, display_id, cart_id, customer_id, email, region_id, currency_code, tax_rate, shipping_address_id, billing_address_id, metadata, created_at, updated_at, canceled_at FROM orders WHERE customer_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
        .bind(id).bind(p.limit).bind(p.offset).fetch_all(&*state.db).await?
        .into_iter().map(|r| serde_json::json!({"id":r.get::<Uuid,_>("id"),"object":"order","status":r.get::<String,_>("status"),"fulfillment_status":r.get::<String,_>("fulfillment_status"),"payment_status":r.get::<String,_>("payment_status"),"display_id":r.get::<i32,_>("display_id"),"cart_id":r.get::<Option<Uuid>,_>("cart_id"),"customer_id":r.get::<Uuid,_>("customer_id"),"email":r.get::<String,_>("email"),"region_id":r.get::<Uuid,_>("region_id"),"currency_code":r.get::<String,_>("currency_code"),"canceled_at":r.get::<Option<chrono::DateTime<chrono::Utc>>,_>("canceled_at"),"created_at":r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),"updated_at":r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),"items":[],"shipping_methods":[],"payments":[],"fulfillments":[],"returns":[],"discounts":[],"gift_cards":[],"subtotal":0,"tax_total":0,"shipping_total":0,"discount_total":0,"total":0})).collect::<Vec<_>>();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM orders WHERE customer_id = $1").bind(id).fetch_one(&*state.db).await?;
    Ok(Json(serde_json::json!({"orders":orders,"count":count,"offset":p.offset,"limit":p.limit})))
}

pub async fn list_addresses(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>) -> Result<Json<serde_json::Value>, AppError> {
    let id: Uuid = auth.0.sub.parse().unwrap_or_default();
    let addresses = sqlx::query("SELECT id, first_name, last_name, phone, company, address_1, address_2, city, country_code, province, postal_code, metadata, created_at, updated_at FROM addresses WHERE customer_id = $1").bind(id).fetch_all(&*state.db).await?
        .into_iter().map(|a| serde_json::json!({"id":a.get::<Uuid,_>("id"),"first_name":a.get::<Option<String>,_>("first_name"),"last_name":a.get::<Option<String>,_>("last_name"),"phone":a.get::<Option<String>,_>("phone"),"company":a.get::<Option<String>,_>("company"),"address_1":a.get::<Option<String>,_>("address_1"),"address_2":a.get::<Option<String>,_>("address_2"),"city":a.get::<Option<String>,_>("city"),"country_code":a.get::<Option<String>,_>("country_code"),"province":a.get::<Option<String>,_>("province"),"postal_code":a.get::<Option<String>,_>("postal_code")})).collect::<Vec<_>>();
    let count = addresses.len() as i64;
    Ok(Json(serde_json::json!({"addresses":addresses,"count":count})))
}

pub async fn add_address(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let cid: Uuid = auth.0.sub.parse().unwrap_or_default();
    let addr = payload.get("address").unwrap_or(&payload);
    let aid = Uuid::new_v4();
    sqlx::query("INSERT INTO addresses (id, customer_id, first_name, last_name, phone, company, address_1, address_2, city, country_code, province, postal_code, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,NOW(),NOW())")
        .bind(aid).bind(cid).bind(addr.get("first_name").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("last_name").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("phone").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("company").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("address_1").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("address_2").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("city").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("country_code").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("province").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(addr.get("postal_code").and_then(|v| v.as_str()).map(|s| s.to_string())).execute(&*state.db).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"customer":fetch_customer(&state, cid).await?}))))
}

pub async fn get_address(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>, Path(address_id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let cid: Uuid = auth.0.sub.parse().unwrap_or_default();
    let a = sqlx::query("SELECT id, first_name, last_name, phone, company, address_1, address_2, city, country_code, province, postal_code, metadata, created_at, updated_at FROM addresses WHERE id = $1 AND customer_id = $2")
        .bind(address_id).bind(cid).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Address not found".into()))?;
    Ok(Json(serde_json::json!({"address":{"id":a.get::<Uuid,_>("id"),"first_name":a.get::<Option<String>,_>("first_name"),"last_name":a.get::<Option<String>,_>("last_name"),"phone":a.get::<Option<String>,_>("phone"),"company":a.get::<Option<String>,_>("company"),"address_1":a.get::<Option<String>,_>("address_1"),"address_2":a.get::<Option<String>,_>("address_2"),"city":a.get::<Option<String>,_>("city"),"country_code":a.get::<Option<String>,_>("country_code"),"province":a.get::<Option<String>,_>("province"),"postal_code":a.get::<Option<String>,_>("postal_code")}})))
}

pub async fn update_address(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>, Path(address_id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    let cid: Uuid = auth.0.sub.parse().unwrap_or_default();
    sqlx::query("UPDATE addresses SET first_name=COALESCE($3,first_name), last_name=COALESCE($4,last_name), phone=COALESCE($5,phone), company=COALESCE($6,company), address_1=COALESCE($7,address_1), address_2=COALESCE($8,address_2), city=COALESCE($9,city), country_code=COALESCE($10,country_code), province=COALESCE($11,province), postal_code=COALESCE($12,postal_code), updated_at=NOW() WHERE id=$1 AND customer_id=$2")
        .bind(address_id).bind(cid).bind(payload.get("first_name").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("last_name").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("phone").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("company").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("address_1").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("address_2").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("city").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("country_code").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("province").and_then(|v| v.as_str()).map(|s| s.to_string())).bind(payload.get("postal_code").and_then(|v| v.as_str()).map(|s| s.to_string())).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"customer":fetch_customer(&state, cid).await?})))
}

pub async fn delete_address(State(state): State<AppState>, axum::Extension(auth): axum::Extension<AuthCustomer>, Path(address_id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    let cid: Uuid = auth.0.sub.parse().unwrap_or_default();
    sqlx::query("DELETE FROM addresses WHERE id = $1 AND customer_id = $2").bind(address_id).bind(cid).execute(&*state.db).await?;
    Ok(Json(serde_json::json!({"customer":fetch_customer(&state, cid).await?})))
}

pub async fn list_payment_methods(State(state): State<AppState>, _: axum::Extension<AuthCustomer>) -> Result<Json<serde_json::Value>, AppError> {
    // Return whatever has been stored in memory so far.  We deliberately
    // clone the vector so the lock holds for a minimal amount of time.
    let methods = {
        let guard = state.payment_methods.lock().await;
        guard.clone()
    };
    Ok(Json(serde_json::json!({"payment_methods": methods})))
}

pub async fn add_payment_method(State(state): State<AppState>, _: axum::Extension<AuthCustomer>, Json(mut payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    // The frontend is free to send whatever keys it wants – we simply attach
    // an id and store the object verbatim.  This mirrors Medusa's behaviour,
    // where the payment provider adds metadata such as `id` and `created_at`.
    let id = format!("pm_{}", Uuid::new_v4());
    // don't overwrite if caller already provided one
    if payload.get("id").is_none() {
        payload["id"] = serde_json::Value::String(id.clone());
    }

    // keep the method in our in‑memory list
    {
        let mut guard = state.payment_methods.lock().await;
        guard.push(payload.clone());
    }

    Ok((StatusCode::CREATED, Json(serde_json::json!({"payment_method": payload}))))
}
