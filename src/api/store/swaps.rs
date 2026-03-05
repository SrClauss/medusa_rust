//! Store swaps handlers
use axum::{{extract::{{Path, State}}, http::StatusCode, Json}};
use sqlx::Row;
use uuid::Uuid;
use crate::{{error::AppError, state::AppState}};
pub async fn create(State(_): State<AppState>, Json(_): Json<serde_json::Value>) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {{
    Ok((StatusCode::CREATED, Json(serde_json::json!({{"swap": {{"id": Uuid::new_v4()}}}}))))
}}
pub async fn get_by_cart(State(state): State<AppState>, Path(cart_id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {{
    let r = sqlx::query("SELECT id, fulfillment_status, payment_status, order_id, cart_id, created_at, updated_at FROM swaps WHERE cart_id = $1")
        .bind(cart_id).fetch_optional(&*state.db).await?.ok_or_else(|| AppError::NotFound("Swap not found".into()))?;
    Ok(Json(serde_json::json!({{"swap": {{
        "id": r.get::<Uuid,_>("id"),
        "fulfillment_status": r.get::<String,_>("fulfillment_status"),
        "payment_status": r.get::<String,_>("payment_status"),
        "order_id": r.get::<Uuid,_>("order_id"),
        "cart_id": r.get::<Option<Uuid>,_>("cart_id"),
        "created_at": r.get::<chrono::DateTime<chrono::Utc>,_>("created_at"),
        "updated_at": r.get::<chrono::DateTime<chrono::Utc>,_>("updated_at"),
    }}}})))
}}
