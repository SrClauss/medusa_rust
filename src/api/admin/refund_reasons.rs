//! Admin refund_reasons handlers — stub CRUD
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
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

pub async fn list(
    State(_): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "refund_reasons": [],
        "count": 0,
        "offset": p.offset,
        "limit": p.limit,
    })))
}

pub async fn create(
    State(_): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4();
    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "refund_reason": {
            "id": id,
            "label": payload.get("label").cloned().unwrap_or(serde_json::Value::Null),
            "description": payload.get("description").cloned().unwrap_or(serde_json::Value::Null),
            "metadata": payload.get("metadata").cloned().unwrap_or(serde_json::Value::Null),
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    }))))
}

pub async fn get(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "refund_reason": {
            "id": id,
            "label": null,
            "description": null,
            "metadata": null,
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    })))
}

pub async fn update(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "refund_reason": {
            "id": id,
            "label": payload.get("label").cloned().unwrap_or(serde_json::Value::Null),
            "description": payload.get("description").cloned().unwrap_or(serde_json::Value::Null),
            "metadata": payload.get("metadata").cloned().unwrap_or(serde_json::Value::Null),
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    })))
}

pub async fn delete_one(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"id": id, "object": "refund-reason", "deleted": true})))
}
