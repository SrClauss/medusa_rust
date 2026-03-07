#![allow(unused_imports)]
#![allow(unused_variables)]
//! Admin auth handlers — POST/GET/DELETE /admin/auth
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;
use validator::Validate;
use crate::{auth::{argon::verify_password, jwt::encode_admin_token}, error::AppError, state::AppState};

#[derive(Debug, Deserialize, Validate)]
pub struct LoginPayload {
    #[validate(email)] pub email: String,
    #[validate(length(min = 1))] pub password: String,
}

fn row_to_user(row: &sqlx::postgres::PgRow) -> serde_json::Value {
    use sqlx::Row;
    serde_json::json!({
        "id":         row.get::<Uuid, _>("id"),
        "email":      row.get::<String, _>("email"),
        "first_name": row.get::<Option<String>, _>("first_name"),
        "last_name":  row.get::<Option<String>, _>("last_name"),
        "role":       row.get::<String, _>("role"),
        "api_token":  row.get::<Option<String>, _>("api_token"),
        "metadata":   row.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
        "deleted_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("deleted_at"),
    })
}

/// POST /admin/auth
pub async fn login(State(state): State<AppState>, Json(payload): Json<LoginPayload>) -> Result<Json<serde_json::Value>, AppError> {
    if let Err(e) = payload.validate() { return Err(AppError::Validation(e.to_string())); }
    let row = sqlx::query("SELECT id, email, first_name, last_name, role, api_token, password_hash, metadata, created_at, updated_at, deleted_at FROM users WHERE email = $1 AND deleted_at IS NULL")
        .bind(&payload.email).fetch_optional(&*state.db).await?.ok_or(AppError::Unauthorized)?;
    let hash: Option<String> = row.get("password_hash");
    if !verify_password(&payload.password, hash.as_deref().unwrap_or(""))? { return Err(AppError::Unauthorized); }
    let user_id: Uuid = row.get("id");
    let user_email: String = row.get("email");
    let token = encode_admin_token(&user_id, &user_email, &state.jwt_secret, 24)?;
    let user = row_to_user(&row);
    Ok(Json(serde_json::json!({ "user": user, "token": token })))
}

/// GET /admin/auth
pub async fn get_session(State(state): State<AppState>, axum::Extension(auth): axum::Extension<crate::auth::jwt::AuthAdmin>) -> Result<Json<serde_json::Value>, AppError> {
    let uid: Uuid = auth.0.sub.parse().unwrap_or_default();
    let row = sqlx::query("SELECT id, email, first_name, last_name, role, api_token, metadata, created_at, updated_at, deleted_at FROM users WHERE id = $1 AND deleted_at IS NULL")
        .bind(uid).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("User not found".into()))?;
    Ok(Json(serde_json::json!({ "user": row_to_user(&row) })))
}

/// DELETE /admin/auth
pub async fn logout() -> axum::http::StatusCode { axum::http::StatusCode::OK }
