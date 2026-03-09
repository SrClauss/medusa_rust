//! Core Mercado Pago plugin implementation.

use anyhow::Context;
use async_trait::async_trait;
use plugin_api::{PaymentAction, PaymentProvider, WebhookActionResult, WebhookInfo, WebhookPayload};
use reqwest::Client;
use std::collections::HashMap;
use tracing::{error, info};

use crate::types::MpWebhookNotification;
use crate::webhooks;

/// Default Mercado Pago API base URL.
const DEFAULT_BASE_URL: &str = "https://api.mercadopago.com";

/// Payment provider plugin for Mercado Pago.
pub struct MercadoPagoPlugin {
    access_token: String,
    base_url: String,
    client: Client,
}

impl MercadoPagoPlugin {
    /// Create a new plugin with default configuration.
    pub fn new() -> Self {
        Self {
            access_token: String::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            client: Client::new(),
        }
    }

    /// Convenience constructor for tests.
    pub fn with_config(access_token: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            base_url: base_url.into(),
            client: Client::new(),
        }
    }
}

impl Default for MercadoPagoPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PaymentProvider for MercadoPagoPlugin {
    fn name(&self) -> &str {
        "mercadopago"
    }

    async fn initialize(&mut self, config: HashMap<String, String>) -> anyhow::Result<()> {
        if let Some(token) = config.get("access_token") {
            self.access_token = token.clone();
        } else {
            anyhow::bail!("MercadoPago: 'access_token' is required in config");
        }
        if let Some(url) = config.get("base_url") {
            self.base_url = url.clone();
        }
        info!(provider = "mercadopago", "MercadoPago plugin initialized");
        Ok(())
    }

    /// Create a payment intent via the Mercado Pago Payments API.
    ///
    /// Required metadata keys:
    /// - `payment_method_id`: e.g. `"pix"`, `"credit_card"`
    /// - `payer_email`: e-mail of the payer
    async fn create_payment(
        &self,
        amount: f64,
        currency: &str,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<serde_json::Value> {
        if amount <= 0.0 {
            anyhow::bail!("MercadoPago: amount must be positive");
        }

        let payment_method_id = metadata
            .get("payment_method_id")
            .cloned()
            .unwrap_or_else(|| "pix".to_string());
        let payer_email = metadata
            .get("payer_email")
            .context("MercadoPago: 'payer_email' metadata key required")?;

        let body = serde_json::json!({
            "transaction_amount": amount,
            "currency_id": currency,
            "description": metadata.get("description").unwrap_or(&String::new()),
            "payment_method_id": payment_method_id,
            "payer": {"email": payer_email},
            "external_reference": metadata.get("order_id"),
        });

        let resp = self
            .client
            .post(format!("{}/v1/payments", self.base_url))
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .await
            .context("MercadoPago create_payment request failed")?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .context("failed to parse create_payment response")?;

        if !status.is_success() {
            error!(provider = "mercadopago", status = %status, "create_payment failed");
            anyhow::bail!("MercadoPago create_payment error {status}: {json}");
        }

        info!(
            provider = "mercadopago",
            payment_id = json["id"].to_string(),
            "Payment created"
        );
        Ok(json)
    }

    async fn capture_payment(
        &self,
        payment_id: &str,
        _amount: Option<f64>,
    ) -> anyhow::Result<serde_json::Value> {
        let resp = self
            .client
            .post(format!("{}/v1/payments/{payment_id}", self.base_url))
            .bearer_auth(&self.access_token)
            .json(&serde_json::json!({"capture": true}))
            .send()
            .await
            .context("MercadoPago capture_payment request failed")?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .context("failed to parse capture_payment response")?;

        if !status.is_success() {
            error!(provider = "mercadopago", payment_id, status = %status, "capture_payment failed");
            anyhow::bail!("MercadoPago capture_payment error {status}: {json}");
        }

        info!(provider = "mercadopago", payment_id, "Payment captured");
        Ok(json)
    }

    async fn refund_payment(
        &self,
        payment_id: &str,
        amount: Option<f64>,
        _reason: Option<&str>,
    ) -> anyhow::Result<serde_json::Value> {
        let mut body = serde_json::json!({});
        if let Some(amt) = amount {
            body["amount"] = serde_json::json!(amt);
        }

        let resp = self
            .client
            .post(format!("{}/v1/payments/{payment_id}/refunds", self.base_url))
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .await
            .context("MercadoPago refund_payment request failed")?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .context("failed to parse refund_payment response")?;

        if !status.is_success() {
            error!(provider = "mercadopago", payment_id, status = %status, "refund_payment failed");
            anyhow::bail!("MercadoPago refund_payment error {status}: {json}");
        }

        info!(provider = "mercadopago", payment_id, "Payment refunded");
        Ok(json)
    }

    async fn cancel_payment(&self, payment_id: &str) -> anyhow::Result<serde_json::Value> {
        let resp = self
            .client
            .put(format!("{}/v1/payments/{payment_id}", self.base_url))
            .bearer_auth(&self.access_token)
            .json(&serde_json::json!({"status": "cancelled"}))
            .send()
            .await
            .context("MercadoPago cancel_payment request failed")?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .context("failed to parse cancel_payment response")?;

        if !status.is_success() {
            error!(provider = "mercadopago", payment_id, status = %status, "cancel_payment failed");
            anyhow::bail!("MercadoPago cancel_payment error {status}: {json}");
        }

        info!(provider = "mercadopago", payment_id, "Payment cancelled");
        Ok(json)
    }

    async fn get_webhook_action_and_data(
        &self,
        payload: WebhookPayload,
    ) -> anyhow::Result<WebhookActionResult> {
        let notification: MpWebhookNotification =
            serde_json::from_value(payload.raw_data.clone())
                .context("failed to parse MercadoPago webhook notification")?;

        let action = match notification.action.as_deref() {
            Some("payment.updated") => {
                // Determine from status embedded in raw_data
                match payload.raw_data["data"]["status"].as_str() {
                    Some("approved") => PaymentAction::Captured,
                    Some("cancelled") => PaymentAction::Cancelled,
                    Some("refunded") => PaymentAction::Refunded,
                    Some("pending") | Some("in_process") => PaymentAction::Pending,
                    Some("rejected") => PaymentAction::Failed,
                    _ => PaymentAction::NotSupported,
                }
            }
            Some("payment.created") => PaymentAction::Authorized,
            _ => PaymentAction::NotSupported,
        };

        let mut data = HashMap::new();
        if let Some(action_str) = &notification.action {
            data.insert(
                "action".to_string(),
                serde_json::Value::String(action_str.clone()),
            );
        }
        if let Some(d) = &notification.data {
            if let Some(id) = &d.id {
                data.insert("payment_id".to_string(), id.clone());
            }
        }

        Ok(WebhookActionResult { action, data })
    }

    async fn create_webhook(&self, url: &str, events: Vec<String>) -> anyhow::Result<String> {
        webhooks::create_webhook(&self.client, &self.base_url, &self.access_token, url, events)
            .await
    }

    async fn list_webhooks(&self) -> anyhow::Result<Vec<WebhookInfo>> {
        webhooks::list_webhooks(&self.client, &self.base_url, &self.access_token).await
    }

    async fn delete_webhook(&self, webhook_id: &str) -> anyhow::Result<()> {
        webhooks::delete_webhook(&self.client, &self.base_url, &self.access_token, webhook_id)
            .await
    }
}
