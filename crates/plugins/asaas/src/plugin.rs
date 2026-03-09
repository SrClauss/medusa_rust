//! Core Asaas plugin implementation.

use anyhow::Context;
use async_trait::async_trait;
use plugin_api::{PaymentAction, PaymentProvider, WebhookActionResult, WebhookInfo, WebhookPayload};
use reqwest::Client;
use std::collections::HashMap;
use tracing::{error, info};

use crate::types::AsaasWebhookEvent;
use crate::webhooks;

/// Default Asaas sandbox base URL.
const DEFAULT_BASE_URL: &str = "https://sandbox.asaas.com/api/v3";

/// Payment provider plugin for Asaas (Brazilian payment gateway).
pub struct AsaasPlugin {
    api_key: String,
    base_url: String,
    client: Client,
}

impl AsaasPlugin {
    /// Create a new plugin instance with default (sandbox) configuration.
    /// Call [`PaymentProvider::initialize`] before using.
    pub fn new() -> Self {
        Self {
            api_key: String::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            client: Client::new(),
        }
    }

    /// Convenience constructor with explicit credentials (useful in tests).
    pub fn with_config(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.into(),
            client: Client::new(),
        }
    }
}

impl Default for AsaasPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PaymentProvider for AsaasPlugin {
    fn name(&self) -> &str {
        "asaas"
    }

    async fn initialize(&mut self, config: HashMap<String, String>) -> anyhow::Result<()> {
        if let Some(key) = config.get("api_key") {
            self.api_key = key.clone();
        } else {
            anyhow::bail!("Asaas: 'api_key' is required in config");
        }
        if let Some(url) = config.get("base_url") {
            self.base_url = url.clone();
        }
        info!(provider = "asaas", base_url = %self.base_url, "Asaas plugin initialized");
        Ok(())
    }

    /// Create a charge in Asaas.
    ///
    /// Required metadata keys:
    /// - `customer`: Asaas customer ID
    /// - `billing_type`: e.g. `"BOLETO"`, `"PIX"`, `"CREDIT_CARD"`
    /// - `due_date`: ISO date string `"YYYY-MM-DD"`
    async fn create_payment(
        &self,
        amount: f64,
        _currency: &str,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<serde_json::Value> {
        if amount <= 0.0 {
            anyhow::bail!("Asaas: amount must be positive");
        }

        let customer = metadata
            .get("customer")
            .context("Asaas: 'customer' metadata key required")?;
        let billing_type = metadata
            .get("billing_type")
            .cloned()
            .unwrap_or_else(|| "PIX".to_string());
        let due_date = metadata
            .get("due_date")
            .context("Asaas: 'due_date' metadata key required")?;

        let body = serde_json::json!({
            "customer": customer,
            "billingType": billing_type,
            "value": amount,
            "dueDate": due_date,
            "description": metadata.get("description"),
            "externalReference": metadata.get("order_id"),
        });

        let resp = self
            .client
            .post(format!("{}/payments", self.base_url))
            .header("access_token", &self.api_key)
            .json(&body)
            .send()
            .await
            .context("Asaas create_payment request failed")?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .context("failed to parse create_payment response")?;

        if !status.is_success() {
            error!(provider = "asaas", status = %status, "create_payment failed");
            anyhow::bail!("Asaas create_payment error {status}: {json}");
        }

        info!(provider = "asaas", payment_id = json["id"].as_str().unwrap_or("?"), "Payment created");
        Ok(json)
    }

    /// Confirm a payment as received in cash (captures it).
    async fn capture_payment(
        &self,
        payment_id: &str,
        _amount: Option<f64>,
    ) -> anyhow::Result<serde_json::Value> {
        let resp = self
            .client
            .post(format!("{}/payments/{payment_id}/receiveInCash", self.base_url))
            .header("access_token", &self.api_key)
            .json(&serde_json::json!({}))
            .send()
            .await
            .context("Asaas capture_payment request failed")?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .context("failed to parse capture_payment response")?;

        if !status.is_success() {
            error!(provider = "asaas", payment_id, status = %status, "capture_payment failed");
            anyhow::bail!("Asaas capture_payment error {status}: {json}");
        }

        info!(provider = "asaas", payment_id, "Payment captured");
        Ok(json)
    }

    async fn refund_payment(
        &self,
        payment_id: &str,
        _amount: Option<f64>,
        _reason: Option<&str>,
    ) -> anyhow::Result<serde_json::Value> {
        let resp = self
            .client
            .post(format!("{}/payments/{payment_id}/refund", self.base_url))
            .header("access_token", &self.api_key)
            .json(&serde_json::json!({}))
            .send()
            .await
            .context("Asaas refund_payment request failed")?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .context("failed to parse refund_payment response")?;

        if !status.is_success() {
            error!(provider = "asaas", payment_id, status = %status, "refund_payment failed");
            anyhow::bail!("Asaas refund_payment error {status}: {json}");
        }

        info!(provider = "asaas", payment_id, "Payment refunded");
        Ok(json)
    }

    async fn cancel_payment(&self, payment_id: &str) -> anyhow::Result<serde_json::Value> {
        let resp = self
            .client
            .delete(format!("{}/payments/{payment_id}", self.base_url))
            .header("access_token", &self.api_key)
            .send()
            .await
            .context("Asaas cancel_payment request failed")?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({"deleted": true}));

        if !status.is_success() {
            error!(provider = "asaas", payment_id, status = %status, "cancel_payment failed");
            anyhow::bail!("Asaas cancel_payment error {status}: {json}");
        }

        info!(provider = "asaas", payment_id, "Payment cancelled");
        Ok(json)
    }

    async fn get_webhook_action_and_data(
        &self,
        payload: WebhookPayload,
    ) -> anyhow::Result<WebhookActionResult> {
        let event: AsaasWebhookEvent = serde_json::from_value(payload.raw_data.clone())
            .context("failed to parse Asaas webhook event")?;

        let action = match event.event.as_str() {
            "PAYMENT_RECEIVED" | "PAYMENT_CONFIRMED" => PaymentAction::Captured,
            "PAYMENT_OVERDUE" | "PAYMENT_PENDING" => PaymentAction::Pending,
            "PAYMENT_REFUNDED" | "PAYMENT_PARTIALLY_REFUNDED" => PaymentAction::Refunded,
            "PAYMENT_DELETED" | "PAYMENT_ANTICIPATED" => PaymentAction::Cancelled,
            "PAYMENT_CREATED" => PaymentAction::Authorized,
            _ => PaymentAction::NotSupported,
        };

        let mut data = HashMap::new();
        data.insert("event".to_string(), serde_json::Value::String(event.event));
        if let Some(payment) = event.payment {
            data.insert("payment_id".to_string(), serde_json::Value::String(payment.id));
            data.insert(
                "amount".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(payment.value)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                ),
            );
        }

        Ok(WebhookActionResult { action, data })
    }

    async fn create_webhook(&self, url: &str, events: Vec<String>) -> anyhow::Result<String> {
        webhooks::create_webhook(&self.client, &self.base_url, &self.api_key, url, events).await
    }

    async fn list_webhooks(&self) -> anyhow::Result<Vec<WebhookInfo>> {
        webhooks::list_webhooks(&self.client, &self.base_url, &self.api_key).await
    }

    async fn delete_webhook(&self, webhook_id: &str) -> anyhow::Result<()> {
        webhooks::delete_webhook(&self.client, &self.base_url, &self.api_key, webhook_id).await
    }
}
