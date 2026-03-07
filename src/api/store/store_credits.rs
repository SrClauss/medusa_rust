//! Store store_credits handlers (stub).
#![allow(unused_variables)]

use axum::{extract::{Path, Query, State}, Json};
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
    Ok(Json(serde_json::json!({"store_credit_accounts":[],"count":0,"offset":p.offset,"limit":p.limit})))
}

pub async fn get(State(_): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({"store_credit_account":{"id":id}})))
}
