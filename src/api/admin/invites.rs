//! Admin invites handlers (stub).
#![allow(unused_variables)]

use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
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

pub async fn list(State(_): State<AppState>, Query(p): Query<ListParams>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"invites":[],"count":0,"offset":p.offset,"limit":p.limit})))
}

pub async fn create(State(_): State<AppState>, Json(_): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    Ok((StatusCode::CREATED, Json(serde_json::json!({"invite":{}}))))
}

pub async fn delete_one(State(_): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"id":id,"object":"invite","deleted":true})))
}

pub async fn accept(State(_): State<AppState>, Path(id): Path<Uuid>, Json(_): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"invite":{"id":id}})))
}
