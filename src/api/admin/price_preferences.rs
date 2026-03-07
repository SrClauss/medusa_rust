//! Admin price_preferences handlers — stub CRUD
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
        "price_preferences": [],
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
        "price_preference": {
            "id": id,
            "attribute": payload.get("attribute").cloned().unwrap_or(serde_json::Value::Null),
            "value": payload.get("value").cloned().unwrap_or(serde_json::Value::Null),
            "is_tax_inclusive": payload.get("is_tax_inclusive").and_then(|v| v.as_bool()).unwrap_or(false),
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
        "price_preference": {
            "id": id,
            "attribute": null,
            "value": null,
            "is_tax_inclusive": false,
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
        "price_preference": {
            "id": id,
            "attribute": payload.get("attribute").cloned().unwrap_or(serde_json::Value::Null),
            "value": payload.get("value").cloned().unwrap_or(serde_json::Value::Null),
            "is_tax_inclusive": payload.get("is_tax_inclusive").and_then(|v| v.as_bool()).unwrap_or(false),
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    })))
}

pub async fn delete_one(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"id": id, "object": "price-preference", "deleted": true})))
}
