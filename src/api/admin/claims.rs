//! Admin claims handlers — stub CRUD
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
        "claims": [],
        "count": 0,
        "offset": p.offset,
        "limit": p.limit,
    })))
}

pub async fn get(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "claim": {
            "id": id,
            "type": null,
            "order_id": null,
            "return_id": null,
            "status": "open",
            "additional_items": [],
            "claim_items": [],
            "fulfillments": [],
            "shipping_methods": [],
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    })))
}

pub async fn update(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "claim": {
            "id": id,
            "status": "open",
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    })))
}

pub async fn delete_one(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"id": id, "object": "claim", "deleted": true})))
}

pub async fn cancel(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "claim": {
            "id": id,
            "status": "canceled",
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    })))
}

pub async fn confirm(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "claim": {
            "id": id,
            "status": "confirmed",
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    })))
}

pub async fn request(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "claim": {
            "id": id,
            "status": "requested",
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    })))
}

pub async fn add_outbound_items(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "claim": {
            "id": id,
            "additional_items": [],
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    }))))
}

pub async fn add_inbound_items(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "claim": {
            "id": id,
            "claim_items": [],
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    }))))
}

pub async fn add_shipping_method(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "claim": {
            "id": id,
            "shipping_methods": [],
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    }))))
}
