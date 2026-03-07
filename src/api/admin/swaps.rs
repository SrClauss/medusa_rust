#![allow(unused_variables)]
//! Admin swaps handlers — placeholder forwarding to full implementation
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use uuid::Uuid;
use crate::{error::AppError, state::AppState};
#[derive(Debug, Deserialize)]
pub struct ListParams { #[serde(default="d20")] pub limit: i64, #[serde(default)] pub offset: i64 }
fn d20() -> i64 { 20 }
pub async fn list(State(_): State<AppState>, Query(p): Query<ListParams>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({"swaps":[],"count":0,"offset":p.offset,"limit":p.limit}))) }
pub async fn get(State(_): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn create(State(_): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> { Ok((StatusCode::CREATED, Json(serde_json::json!({})))) }
pub async fn update(State(_): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn delete_one(State(_): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({"id":id,"object":"swaps","deleted":true}))) }
pub async fn add_products(State(_): State<AppState>, Path(id): Path<Uuid>, Json(_): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn remove_products(State(_): State<AppState>, Path(id): Path<Uuid>, Json(_): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn add_country(State(_): State<AppState>, Path(id): Path<Uuid>, Json(_): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn remove_country(State(_): State<AppState>, Path((region_id, code)): Path<(Uuid,String)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn add_fulfillment_provider(State(_): State<AppState>, Path(id): Path<Uuid>, Json(_): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn remove_fulfillment_provider(State(_): State<AppState>, Path((id, pid)): Path<(Uuid,String)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn add_payment_provider(State(_): State<AppState>, Path(id): Path<Uuid>, Json(_): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn remove_payment_provider(State(_): State<AppState>, Path((id, pid)): Path<(Uuid,String)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn get_by_code(State(_): State<AppState>, Path(code): Path<String>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn add_region(State(_): State<AppState>, Path((id, rid)): Path<(Uuid,Uuid)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn remove_region(State(_): State<AppState>, Path((id, rid)): Path<(Uuid,Uuid)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn list_conditions(State(_): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({"discount_conditions":[]}))) }
pub async fn create_condition(State(_): State<AppState>, Path(id): Path<Uuid>, Json(_): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> { Ok((StatusCode::CREATED, Json(serde_json::json!({})))) }
pub async fn get_condition(State(_): State<AppState>, Path((id,cid)): Path<(Uuid,Uuid)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn update_condition(State(_): State<AppState>, Path((id,cid)): Path<(Uuid,Uuid)>, Json(_): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn delete_condition(State(_): State<AppState>, Path((id,cid)): Path<(Uuid,Uuid)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn request_password_reset(State(_): State<AppState>, Json(_): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn reset_password(State(_): State<AppState>, Json(_): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn add_prices(State(_): State<AppState>, Path(id): Path<Uuid>, Json(_): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn delete_prices(State(_): State<AppState>, Path(id): Path<Uuid>, Json(_): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn list_products(State(_): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({"products":[]}))) }
pub async fn list_levels(State(_): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({"inventory_levels":[]}))) }
pub async fn create_level(State(_): State<AppState>, Path(id): Path<Uuid>, Json(_): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> { Ok((StatusCode::CREATED, Json(serde_json::json!({})))) }
pub async fn update_level(State(_): State<AppState>, Path((id,lid)): Path<(Uuid,Uuid)>, Json(_): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn delete_level(State(_): State<AppState>, Path((id,lid)): Path<(Uuid,Uuid)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn upload(State(_): State<AppState>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({"uploads":[]}))) }
pub async fn delete_files(State(_): State<AppState>, Json(_): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn receive(State(_): State<AppState>, Path(id): Path<Uuid>, Json(_): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({"return":{"id":id}}))) }
pub async fn add_line_item(State(_): State<AppState>, Path(id): Path<Uuid>, Json(_): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> { Ok((StatusCode::CREATED, Json(serde_json::json!({})))) }
pub async fn update_line_item(State(_): State<AppState>, Path((id,lid)): Path<(Uuid,Uuid)>, Json(_): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn delete_line_item(State(_): State<AppState>, Path((id,lid)): Path<(Uuid,Uuid)>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn register_payment(State(_): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn confirm(State(_): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
pub async fn cancel(State(_): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> { Ok(Json(serde_json::json!({}))) }
