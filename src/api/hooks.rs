//! Handlers for incoming webhook callbacks (payment providers, etc.).

use axum::{extract::{Path, State}, body::Bytes, response::IntoResponse};
use axum::http::{HeaderMap, StatusCode};
use serde_json::Value;
use crate::{error::AppError, state::AppState};
use crate::core::payment::WebhookPayload;
use crate::events::{Event, EmitOptions};

/// Generic payment provider webhook endpoint.
/// POST /hooks/payment/:provider
pub async fn payment_provider_webhook(
    Path(provider): Path<String>,
    State(state): State<AppState>,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let raw_str = String::from_utf8(body.to_vec()).unwrap_or_default();
    let data: Value = serde_json::from_str(&raw_str).unwrap_or(Value::Null);

    let payload = WebhookPayload { data, raw: raw_str, headers: HeaderMap::new() };

    // persist webhook attempt
    sqlx::query(
        "INSERT INTO webhooks (provider, payload, status) VALUES ($1, $2, 'pending')",
    )
    .bind(&provider)
    .bind(&serde_json::to_value(&payload.data).map_err(|e| AppError::Internal(e.to_string()))?)
    .execute(&*state.db)
    .await
    .ok();

    // lookup provider and process action
    if let Some(p) = state.plugin_mgr.lock().await.payment_provider(&provider) {
        match p.get_webhook_action_and_data(&payload) {
            Ok(result) => {
                // emit internal event using the shared bus from AppState
                state.event_bus.emit(
                    Event::PaymentWebhook(crate::events::PaymentWebhookEvent { provider, payload: serde_json::to_value(&result).map_err(|e| AppError::Internal(e.to_string()))? }),
                    EmitOptions { delay: None, retries: 1 },
                ).await;
                Ok(StatusCode::OK)
            }
            Err(e) => {
                tracing::error!(error = %e, "webhook provider processing error");
                Err(AppError::Internal("provider error".into()))
            }
        }
    } else {
        Err(AppError::NotFound("payment provider not registered".into()))
    }
}
