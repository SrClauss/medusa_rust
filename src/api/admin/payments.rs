//! Admin payments handlers — stub
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};
use crate::events::{Event, PaymentCapturedEvent};

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
        "payments": [],
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
        "payment": {
            "id": id,
            "amount": null,
            "currency_code": null,
            "provider_id": null,
            "status": "pending",
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    })))
}

pub async fn capture(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let amount = payload.get("amount").and_then(|v| v.as_i64());

    // Publish PaymentCaptured event
    if let Err(e) = state.event_bus.publish(Event::PaymentCaptured(PaymentCapturedEvent {
        payment_id: id,
        amount,
        currency_code: payload.get("currency_code").and_then(|v| v.as_str()).map(|s| s.to_string()),
        provider_id: payload.get("provider_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
    })).await {
        tracing::warn!(error = %e, "Failed to publish PaymentCaptured event");
    }

    Ok(Json(serde_json::json!({
        "payment": {
            "id": id,
            "status": "captured",
            "amount": amount,
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    })))
}

pub async fn refund(
    State(_): State<AppState>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "refund": {
            "id": Uuid::new_v4(),
            "payment_id": id,
            "amount": null,
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }
    })))
}
