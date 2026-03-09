//! Payment provider abstractions used by the plugin system.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use axum::http::HeaderMap;

/// An action a provider instructs us to perform as a result of receiving a
/// webhook.  Mirrors MedusaJS `PaymentActions`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentAction {
    Authorized,
    Captured,
    Failed,
    Pending,
    RequiresMore,
    Canceled,
    NotSupported,
}

/// Additional data attached to a webhook action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookActionData {
    pub session_id: String,
    pub amount: i64,
}

/// Result returned by providers when processing a callback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookActionResult {
    pub action: PaymentAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<WebhookActionData>,
}

/// Payload passed to providers when a webhook arrives.  It contains the
/// parsed JSON, the raw body and any headers (for signature verification).
#[derive(Debug, Clone)]
pub struct WebhookPayload {
    pub data: Value,
    pub raw: String,
    pub headers: HeaderMap,
}

/// Trait that must be implemented by payment providers (plugins).
pub trait PaymentProvider: Send + Sync {
    fn id(&self) -> &str;
    fn init(&self) -> anyhow::Result<()> { Ok(()) }

    /// Process an incoming webhook payload and return the corresponding action.
    fn get_webhook_action_and_data(
        &self,
        payload: &WebhookPayload,
    ) -> anyhow::Result<WebhookActionResult>;
}
