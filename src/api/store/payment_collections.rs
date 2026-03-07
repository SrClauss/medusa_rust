//! Store payment_collections handlers (stub).
#![allow(unused_variables)]

use axum::{extract::{Path, State}, http::StatusCode, Json};
use uuid::Uuid;
use crate::{error::AppError, state::AppState};

pub async fn create(State(_): State<AppState>, Json(_): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({"payment_collection":{}}))))
}

pub async fn add_payment_session(State(_): State<AppState>, Path(id): Path<Uuid>, Json(_): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({"payment_session":{"payment_collection_id":id}}))))
}
