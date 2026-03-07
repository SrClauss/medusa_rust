//! Admin fulfillment_sets handlers — stub CRUD
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

pub async fn get(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "fulfillment_set": {
            "id": id,
            "name": null,
            "type": null,
            "service_zones": [],
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    })))
}

pub async fn delete_one(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"id": id, "object": "fulfillment-set", "deleted": true})))
}

pub async fn create_service_zone(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let zone_id = Uuid::new_v4();
    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "fulfillment_set": {
            "id": id,
            "service_zones": [{
                "id": zone_id,
                "name": payload.get("name").cloned().unwrap_or(serde_json::Value::Null),
                "fulfillment_set_id": id,
                "geo_zones": [],
                "created_at": chrono::Utc::now(),
                "updated_at": chrono::Utc::now(),
            }],
        }
    }))))
}

pub async fn update_service_zone(
    State(_): State<AppState>,
    Path((id, zone_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "fulfillment_set": {
            "id": id,
            "service_zones": [{
                "id": zone_id,
                "name": payload.get("name").cloned().unwrap_or(serde_json::Value::Null),
                "fulfillment_set_id": id,
                "geo_zones": [],
                "created_at": chrono::Utc::now(),
                "updated_at": chrono::Utc::now(),
            }],
        }
    })))
}

pub async fn delete_service_zone(
    State(_): State<AppState>,
    Path((_id, zone_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"id": zone_id, "object": "service-zone", "deleted": true})))
}
