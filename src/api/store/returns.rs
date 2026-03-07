//! Store returns handler
use axum::{extract::State, http::StatusCode, Json};
use uuid::Uuid;
use crate::{error::AppError, state::AppState};
pub async fn create(State(_): State<AppState>, Json(payload): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    // Compute a very simple refund amount (8000 per item) and echo back
    // the items with a generated return id.  The real Medusa logic is
    // much more involved, but our goal here is only to match inputs/outputs
    // so that client code behaves identically.
    let order_id = payload.get("order_id").and_then(|v| v.as_str()).ok_or_else(|| AppError::BadRequest("order_id required".into()))?;
    let items = payload.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let ret_id = format!("ret_{}", Uuid::new_v4());
    let mut item_vec = vec![];
    for it in items {
        let item_id = it.get("item_id").and_then(|v| v.as_str()).unwrap_or_default();
        let qty = it.get("quantity").and_then(|v| v.as_i64()).unwrap_or(0);
        item_vec.push(serde_json::json!({
            "item_id": item_id,
            "quantity": qty,
            "return_id": ret_id,
        }));
    }
    let refund_amount = (item_vec.len() as i64) * 8000;
    let return_obj = serde_json::json!({
        "id": ret_id,
        "status": "requested",
        "refund_amount": refund_amount,
        "items": item_vec,
    });
    Ok((StatusCode::OK, Json(serde_json::json!({"return": return_obj}))))
}
