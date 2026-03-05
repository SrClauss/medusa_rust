//! Store returns handler
use axum::{extract::State, http::StatusCode, Json};
use uuid::Uuid;
use crate::{error::AppError, state::AppState};
pub async fn create(State(_): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({"return":{"id":Uuid::new_v4(),"status":"requested"}}))))
}
