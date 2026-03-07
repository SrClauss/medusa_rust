//! Admin feature_flags handlers — stub
use axum::{extract::State, Json};
use crate::{error::AppError, state::AppState};

pub async fn list(
    State(_): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "flags": []
    })))
}
