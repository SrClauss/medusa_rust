//! Admin reservations handlers — stub CRUD
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
    pub location_id: Option<Uuid>,
    pub inventory_item_id: Option<Uuid>,
}
fn d20() -> i64 { 20 }

pub async fn list(
    State(_): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "reservations": [],
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
        "reservation": {
            "id": id,
            "inventory_item_id": payload.get("inventory_item_id").cloned().unwrap_or(serde_json::Value::Null),
            "location_id": payload.get("location_id").cloned().unwrap_or(serde_json::Value::Null),
            "quantity": payload.get("quantity").cloned().unwrap_or(serde_json::json!(0)),
            "line_item_id": payload.get("line_item_id").cloned().unwrap_or(serde_json::Value::Null),
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
        "reservation": {
            "id": id,
            "inventory_item_id": null,
            "location_id": null,
            "quantity": 0,
            "line_item_id": null,
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
        "reservation": {
            "id": id,
            "quantity": payload.get("quantity").cloned().unwrap_or(serde_json::json!(0)),
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
    Ok(Json(serde_json::json!({"id": id, "object": "reservation", "deleted": true})))
}
