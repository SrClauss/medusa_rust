//! Common traits and types for MedusaRust payment provider plugins.
//!
//! All payment provider crates must implement the [`PaymentProvider`] trait
//! defined here, ensuring a uniform interface for the core application.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Possible actions resulting from a payment event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentAction {
    /// Payment authorized but not yet captured.
    Authorized,
    /// Payment captured (funds debited).
    Captured,
    /// Payment fully refunded.
    Refunded,
    /// Partial refund processed.
    PartialRefund,
    /// Payment failed.
    Failed,
    /// Payment cancelled.
    Cancelled,
    /// Payment pending confirmation.
    Pending,
    /// Action not supported by this provider.
    NotSupported,
}

/// Normalized webhook payload passed to plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// Provider identifier (e.g. "asaas", "stripe").
    pub provider: String,
    /// Payment identifier in the provider's system.
    pub provider_id: String,
    /// Optional order identifier in the Medusa system.
    pub order_id: Option<String>,
    /// Payment amount.
    pub amount: Option<f64>,
    /// Currency code (e.g. "BRL", "USD").
    pub currency: Option<String>,
    /// Raw webhook body as parsed JSON.
    pub raw_data: serde_json::Value,
}

/// Result returned after processing a webhook.
#[derive(Debug)]
pub struct WebhookActionResult {
    /// The normalized action to perform.
    pub action: PaymentAction,
    /// Additional provider-specific data.
    pub data: HashMap<String, serde_json::Value>,
}

/// Information about a registered webhook endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookInfo {
    /// Provider-assigned webhook identifier.
    pub id: String,
    /// Target URL that receives events.
    pub url: String,
    /// Event types the webhook subscribes to.
    pub events: Vec<String>,
    /// Whether the webhook is currently active.
    pub active: bool,
}

/// Trait that every payment provider plugin must implement.
///
/// Default implementations for webhook management methods return an error
/// indicating the operation is unsupported, allowing plugins that do not
/// support a feature to compile without implementing it.
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    /// Unique identifier for this provider (e.g. `"asaas"`, `"stripe"`).
    fn name(&self) -> &str;

    /// Initialize the provider with runtime configuration values.
    async fn initialize(&mut self, config: HashMap<String, String>) -> anyhow::Result<()>;

    /// Create a payment session / payment intent.
    async fn create_payment(
        &self,
        amount: f64,
        currency: &str,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<serde_json::Value>;

    /// Capture a previously authorized payment.
    async fn capture_payment(
        &self,
        payment_id: &str,
        amount: Option<f64>,
    ) -> anyhow::Result<serde_json::Value>;

    /// Refund a captured payment, optionally for a partial amount.
    async fn refund_payment(
        &self,
        payment_id: &str,
        amount: Option<f64>,
        reason: Option<&str>,
    ) -> anyhow::Result<serde_json::Value>;

    /// Cancel / void a payment that has not yet been captured.
    async fn cancel_payment(&self, payment_id: &str) -> anyhow::Result<serde_json::Value>;

    /// Parse an incoming webhook and return the normalized action.
    async fn get_webhook_action_and_data(
        &self,
        payload: WebhookPayload,
    ) -> anyhow::Result<WebhookActionResult>;

    // ── Webhook management (optional) ────────────────────────────────────────

    /// Register a new webhook endpoint at the provider.
    ///
    /// Returns the provider-assigned webhook identifier.
    async fn create_webhook(
        &self,
        _url: &str,
        _events: Vec<String>,
    ) -> anyhow::Result<String> {
        Err(anyhow::anyhow!(
            "create_webhook not implemented for {}",
            self.name()
        ))
    }

    /// List all webhook endpoints registered at the provider.
    async fn list_webhooks(&self) -> anyhow::Result<Vec<WebhookInfo>> {
        Err(anyhow::anyhow!(
            "list_webhooks not implemented for {}",
            self.name()
        ))
    }

    /// Remove a registered webhook endpoint.
    async fn delete_webhook(&self, _webhook_id: &str) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(
            "delete_webhook not implemented for {}",
            self.name()
        ))
    }

    /// Update the event subscriptions of an existing webhook.
    async fn update_webhook(
        &self,
        _webhook_id: &str,
        _events: Vec<String>,
    ) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(
            "update_webhook not implemented for {}",
            self.name()
        ))
    }
}
