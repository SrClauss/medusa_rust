//! Store payment-providers handler — public read-only list
use axum::{extract::State, Json};
use sqlx::Row;
use crate::{error::AppError, state::AppState};

pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, is_installed FROM payment_providers WHERE is_installed = TRUE ORDER BY id",
    )
    .fetch_all(&*state.db)
    .await?;
    let payment_providers: Vec<_> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.get::<String, _>("id"),
                "is_installed": r.get::<bool, _>("is_installed"),
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "payment_providers": payment_providers })))
}
