//! Core Stripe plugin implementation.
//!
//! Mirrors the logic of the official MedusaJS Stripe plugin, adapted for Rust
//! and the `plugin_api` trait interface.

use anyhow::Context;
use async_trait::async_trait;
use plugin_api::{PaymentAction, PaymentProvider, WebhookActionResult, WebhookInfo, WebhookPayload};
use reqwest::Client;
use std::collections::HashMap;
use tracing::{error, info};

use crate::types::StripeWebhookEvent;
use crate::webhooks;

/// Default Stripe API base URL.
const DEFAULT_BASE_URL: &str = "https://api.stripe.com";

/// Payment provider plugin for Stripe.
pub struct StripePlugin {
    secret_key: String,
    webhook_secret: String,
    base_url: String,
    client: Client,
}

impl StripePlugin {
    /// Create a new plugin with default configuration.
    pub fn new() -> Self {
        Self {
            secret_key: String::new(),
            webhook_secret: String::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            client: Client::new(),
        }
    }

    /// Convenience constructor for tests.
    pub fn with_config(
        secret_key: impl Into<String>,
        webhook_secret: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            secret_key: secret_key.into(),
            webhook_secret: webhook_secret.into(),
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    /// Verify an incoming Stripe webhook signature.
    pub fn verify_signature(&self, payload: &[u8], sig_header: &str) -> anyhow::Result<()> {
        webhooks::verify_webhook_signature(payload, sig_header, &self.webhook_secret)
    }

    /// Convert amount in major currency units to Stripe's integer cents format.
    ///
    /// Stripe expects amounts in the smallest currency unit (cents for USD/BRL, etc.).
    fn to_stripe_amount(amount: f64) -> i64 {
        (amount * 100.0).round() as i64
    }
}

impl Default for StripePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PaymentProvider for StripePlugin {
    fn name(&self) -> &str {
        "stripe"
    }

    async fn initialize(&mut self, config: HashMap<String, String>) -> anyhow::Result<()> {
        if let Some(key) = config.get("secret_key") {
            self.secret_key = key.clone();
        } else {
            anyhow::bail!("Stripe: 'secret_key' is required in config");
        }
        if let Some(secret) = config.get("webhook_secret") {
            self.webhook_secret = secret.clone();
        }
        if let Some(url) = config.get("base_url") {
            self.base_url = url.clone();
        }
        info!(provider = "stripe", "Stripe plugin initialized");
        Ok(())
    }

    /// Create a PaymentIntent in Stripe.
    ///
    /// The PaymentIntent is created with `capture_method = "manual"` so that
    /// authorization and capture are separate steps, matching MedusaJS behavior.
    async fn create_payment(
        &self,
        amount: f64,
        currency: &str,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<serde_json::Value> {
        if amount <= 0.0 {
            anyhow::bail!("Stripe: amount must be positive");
        }

        let stripe_amount = Self::to_stripe_amount(amount);

        // Build form params; metadata is passed as metadata[key_name]=value
        let mut form_params: Vec<(String, String)> = vec![
            ("amount".to_string(), stripe_amount.to_string()),
            ("currency".to_string(), currency.to_lowercase()),
            ("capture_method".to_string(), "manual".to_string()),
        ];
        for (k, v) in &metadata {
            form_params.push((format!("metadata[{}]", k), v.clone()));
        }

        let resp = self
            .client
            .post(format!("{}/v1/payment_intents", self.base_url))
            .basic_auth(&self.secret_key, Some(""))
            .form(&form_params)
            .send()
            .await
            .context("Stripe create_payment request failed")?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .context("failed to parse create_payment response")?;

        if !status.is_success() {
            error!(provider = "stripe", status = %status, "create_payment failed");
            anyhow::bail!("Stripe create_payment error {status}: {json}");
        }

        info!(
            provider = "stripe",
            payment_intent_id = json["id"].as_str().unwrap_or("?"),
            "PaymentIntent created"
        );
        Ok(json)
    }

    /// Capture a previously authorized PaymentIntent.
    async fn capture_payment(
        &self,
        payment_id: &str,
        amount: Option<f64>,
    ) -> anyhow::Result<serde_json::Value> {
        let mut form_params: Vec<(&str, String)> = vec![];
        if let Some(amt) = amount {
            let stripe_amount = Self::to_stripe_amount(amt);
            form_params.push(("amount_to_capture", stripe_amount.to_string()));
        }

        let resp = self
            .client
            .post(format!("{}/v1/payment_intents/{payment_id}/capture", self.base_url))
            .basic_auth(&self.secret_key, Some(""))
            .form(&form_params)
            .send()
            .await
            .context("Stripe capture_payment request failed")?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .context("failed to parse capture_payment response")?;

        if !status.is_success() {
            error!(provider = "stripe", payment_id, status = %status, "capture_payment failed");
            anyhow::bail!("Stripe capture_payment error {status}: {json}");
        }

        info!(provider = "stripe", payment_id, "PaymentIntent captured");
        Ok(json)
    }

    /// Refund a payment via the Stripe Refunds API.
    async fn refund_payment(
        &self,
        payment_id: &str,
        amount: Option<f64>,
        _reason: Option<&str>,
    ) -> anyhow::Result<serde_json::Value> {
        let mut form_params: Vec<(&str, String)> = vec![
            ("payment_intent", payment_id.to_string()),
        ];
        if let Some(amt) = amount {
            let stripe_amount = Self::to_stripe_amount(amt);
            form_params.push(("amount", stripe_amount.to_string()));
        }

        let resp = self
            .client
            .post(format!("{}/v1/refunds", self.base_url))
            .basic_auth(&self.secret_key, Some(""))
            .form(&form_params)
            .send()
            .await
            .context("Stripe refund_payment request failed")?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .context("failed to parse refund_payment response")?;

        if !status.is_success() {
            error!(provider = "stripe", payment_id, status = %status, "refund_payment failed");
            anyhow::bail!("Stripe refund_payment error {status}: {json}");
        }

        info!(provider = "stripe", payment_id, "Payment refunded");
        Ok(json)
    }

    /// Cancel a PaymentIntent that has not yet been captured.
    async fn cancel_payment(&self, payment_id: &str) -> anyhow::Result<serde_json::Value> {
        let resp = self
            .client
            .post(format!("{}/v1/payment_intents/{payment_id}/cancel", self.base_url))
            .basic_auth(&self.secret_key, Some(""))
            .form::<[(&str, &str); 0]>(&[])
            .send()
            .await
            .context("Stripe cancel_payment request failed")?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .context("failed to parse cancel_payment response")?;

        if !status.is_success() {
            error!(provider = "stripe", payment_id, status = %status, "cancel_payment failed");
            anyhow::bail!("Stripe cancel_payment error {status}: {json}");
        }

        info!(provider = "stripe", payment_id, "PaymentIntent cancelled");
        Ok(json)
    }

    /// Parse a Stripe webhook event and return a normalized action.
    ///
    /// Maps Stripe event types to [`PaymentAction`] variants, matching the
    /// logic of the official MedusaJS Stripe plugin.
    async fn get_webhook_action_and_data(
        &self,
        payload: WebhookPayload,
    ) -> anyhow::Result<WebhookActionResult> {
        let event: StripeWebhookEvent = serde_json::from_value(payload.raw_data.clone())
            .context("failed to parse Stripe webhook event")?;

        let action = match event.event_type.as_str() {
            "payment_intent.amount_capturable_updated" => PaymentAction::Authorized,
            "payment_intent.succeeded" => PaymentAction::Captured,
            "payment_intent.payment_failed" => PaymentAction::Failed,
            "payment_intent.canceled" => PaymentAction::Cancelled,
            "payment_intent.processing" => PaymentAction::Pending,
            "charge.refunded" => PaymentAction::Refunded,
            "charge.refund.updated" => PaymentAction::Refunded,
            _ => PaymentAction::NotSupported,
        };

        let mut data = HashMap::new();
        data.insert(
            "event_type".to_string(),
            serde_json::Value::String(event.event_type),
        );
        data.insert("event_id".to_string(), serde_json::Value::String(event.id));
        data.insert("object".to_string(), event.data.object);

        Ok(WebhookActionResult { action, data })
    }

    async fn create_webhook(&self, url: &str, events: Vec<String>) -> anyhow::Result<String> {
        webhooks::create_webhook(&self.client, &self.base_url, &self.secret_key, url, events).await
    }

    async fn list_webhooks(&self) -> anyhow::Result<Vec<WebhookInfo>> {
        webhooks::list_webhooks(&self.client, &self.base_url, &self.secret_key).await
    }

    async fn delete_webhook(&self, webhook_id: &str) -> anyhow::Result<()> {
        webhooks::delete_webhook(&self.client, &self.base_url, &self.secret_key, webhook_id).await
    }
}
