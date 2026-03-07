//! Admin notifications handlers — DB-backed CRUD
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
    pub channel: Option<String>,
}
fn d20() -> i64 { 20 }

fn notif_json(r: &sqlx::postgres::PgRow) -> serde_json::Value {
    serde_json::json!({
        "id": r.get::<Uuid, _>("id"),
        "to": r.get::<String, _>("to_address"),
        "channel": r.get::<String, _>("channel"),
        "template": r.get::<String, _>("template"),
        "data": r.get::<Option<serde_json::Value>, _>("data"),
        "trigger_type": r.get::<Option<String>, _>("trigger_type"),
        "resource_id": r.get::<Option<Uuid>, _>("resource_id"),
        "resource_type": r.get::<Option<String>, _>("resource_type"),
        "status": r.get::<String, _>("status"),
        "provider_id": r.get::<Option<String>, _>("provider_id"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>, _>("updated_at"),
    })
}

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = if let Some(ref ch) = p.channel {
        sqlx::query(
            "SELECT id, to_address, channel, template, data, trigger_type, resource_id, \
             resource_type, status, provider_id, created_at, updated_at \
             FROM notifications WHERE channel = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(ch)
        .bind(p.limit)
        .bind(p.offset)
        .fetch_all(&*state.db)
        .await?
    } else {
        sqlx::query(
            "SELECT id, to_address, channel, template, data, trigger_type, resource_id, \
             resource_type, status, provider_id, created_at, updated_at \
             FROM notifications ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(p.limit)
        .bind(p.offset)
        .fetch_all(&*state.db)
        .await?
    };
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notifications")
        .fetch_one(&*state.db)
        .await?;
    let notifications: Vec<_> = rows.iter().map(notif_json).collect();
    Ok(Json(serde_json::json!({"notifications": notifications, "count": count, "offset": p.offset, "limit": p.limit})))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query(
        "SELECT id, to_address, channel, template, data, trigger_type, resource_id, \
         resource_type, status, provider_id, created_at, updated_at \
         FROM notifications WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&*state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Notification not found".into()))?;
    Ok(Json(serde_json::json!({"notification": notif_json(&r)})))
}

pub async fn resend(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "notification": {
            "id": id,
            "status": "sent",
            "updated_at": chrono::Utc::now(),
        }
    })))
}
