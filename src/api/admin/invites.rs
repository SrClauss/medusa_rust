//! Admin invites handlers — DB-backed CRUD
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use chrono::Utc;
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
}
fn d20() -> i64 { 20 }

fn invite_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "user_email": r.get::<String, _>("user_email"),
        "role": r.get::<String, _>("role"),
        "accepted": r.get::<bool, _>("accepted"),
        "token": r.get::<String, _>("token"),
        "expires_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("expires_at"),
        "metadata": r.get::<Option<serde_json::Value>, _>("metadata"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    })
}

fn generate_invite_token() -> String {
    use rand::Rng;
    let rng = rand::thread_rng();
    rng.sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, user_email, role, accepted, token, expires_at, metadata, created_at, updated_at \
         FROM invites WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(p.limit)
    .bind(p.offset)
    .fetch_all(&*state.db)
    .await?;
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM invites WHERE deleted_at IS NULL")
            .fetch_one(&*state.db)
            .await?;
    let invites: Vec<_> = rows.iter().map(invite_json).collect();
    Ok(Json(serde_json::json!({"invites": invites, "count": count, "offset": p.offset, "limit": p.limit})))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, user_email, role, accepted, token, expires_at, metadata, created_at, updated_at \
         FROM invites WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Invite not found".into()))?;
    Ok(Json(serde_json::json!({"invite": invite_json(&r)})))
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    let user_email = payload
        .get("user")
        .and_then(|v| v.as_str())
        .or_else(|| payload.get("email").and_then(|v| v.as_str()))
        .ok_or_else(|| AppError::BadRequest("user email required".into()))?;
    let role = payload.get("role").and_then(|v| v.as_str()).unwrap_or("member");
    let token = generate_invite_token();
    let expires_at = Utc::now() + chrono::Duration::days(7);
    sqlx::query(
        "INSERT INTO invites (id, user_email, role, token, expires_at, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, NOW(), NOW()) \
         ON CONFLICT (user_email) DO UPDATE SET \
         role = EXCLUDED.role, token = EXCLUDED.token, expires_at = EXCLUDED.expires_at, \
         accepted = FALSE, updated_at = NOW(), deleted_at = NULL",
    )
    .bind(id)
    .bind(user_email)
    .bind(role)
    .bind(&token)
    .bind(expires_at)
    .execute(&*state.db)
    .await?;
    // Fetch by token since id might differ on conflict
    let r = sqlx::query(
        "SELECT id, user_email, role, accepted, token, expires_at, metadata, created_at, updated_at \
         FROM invites WHERE token = $1",
    )
    .bind(&token)
    .fetch_one(&*state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"invite": invite_json(&r)}))))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE invites SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&*state.db)
        .await?;
    Ok(Json(serde_json::json!({"id": id, "object": "invite", "deleted": true})))
}

pub async fn accept(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verify token matches
    let token = payload.get("token").and_then(|v| v.as_str()).unwrap_or("");
    let r = sqlx::query(
        "UPDATE invites SET accepted = TRUE, updated_at = NOW() \
         WHERE id = $1 AND token = $2 AND (expires_at IS NULL OR expires_at > NOW()) \
         RETURNING id, user_email, role, accepted, token, expires_at, metadata, created_at, updated_at",
    )
    .bind(id)
    .bind(token)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest("Invalid or expired invite token".into()))?;
    Ok(Json(serde_json::json!({"invite": invite_json(&r)})))
}
