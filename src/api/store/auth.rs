#![allow(unused_imports, unused_variables)]
//! Store auth handlers
use axum::{extract::{Path, State}, Json};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;
use validator::Validate;
use crate::{auth::{argon::verify_password, jwt::encode_store_token}, error::AppError, state::AppState};

#[derive(Debug, Deserialize, Validate)]
pub struct LoginPayload {
    #[validate(email)] pub email: String,
    #[validate(length(min = 1))] pub password: String,
}

pub async fn login(State(state): State<AppState>, Json(payload): Json<LoginPayload>) -> Result<Json<serde_json::Value>, AppError> {
    if let Err(e) = payload.validate() { return Err(AppError::Validation(e.to_string())); }
    let row = sqlx::query("SELECT id, email, first_name, last_name, phone, has_account, password_hash, metadata, created_at, updated_at FROM customers WHERE email = $1 AND deleted_at IS NULL")
        .bind(&payload.email).fetch_optional(&*state.db).await?.ok_or(AppError::Unauthorized)?;
    let hash: Option<String> = row.get("password_hash");
    if !verify_password(&payload.password, hash.as_deref().unwrap_or(""))? { return Err(AppError::Unauthorized); }
    let uid: Uuid = row.get("id");
    let email: String = row.get("email");
    let token = encode_store_token(&uid, &email, &state.jwt_secret, 24)?;
    Ok(Json(serde_json::json!({ "customer": {
        "id": uid, "email": email,
        "first_name": row.get::<Option<String>, _>("first_name"),
        "last_name":  row.get::<Option<String>, _>("last_name"),
        "phone":      row.get::<Option<String>, _>("phone"),
        "has_account": row.get::<bool, _>("has_account"),
        "metadata":   row.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    }, "access_token": token })))
}

pub async fn get_session(State(state): State<AppState>, axum::Extension(auth): axum::Extension<crate::auth::jwt::AuthCustomer>) -> Result<Json<serde_json::Value>, AppError> {
    let id: Uuid = auth.0.sub.parse().unwrap_or_default();
    let row = sqlx::query("SELECT id, email, first_name, last_name, phone, has_account, metadata, created_at, updated_at FROM customers WHERE id = $1 AND deleted_at IS NULL")
        .bind(id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Customer not found".into()))?;
    Ok(Json(serde_json::json!({ "customer": {
        "id": row.get::<Uuid, _>("id"), "email": row.get::<String, _>("email"),
        "first_name": row.get::<Option<String>, _>("first_name"),
        "last_name":  row.get::<Option<String>, _>("last_name"),
        "phone":      row.get::<Option<String>, _>("phone"),
        "has_account": row.get::<bool, _>("has_account"),
        "metadata":   row.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    }})))
}

pub async fn logout() -> axum::http::StatusCode { axum::http::StatusCode::OK }
pub async fn oauth_callback(Path(provider): Path<String>) -> Result<Json<serde_json::Value>, AppError> {
    Err(AppError::BadRequest(format!("OAuth provider '{}' not yet configured", provider)))
}
