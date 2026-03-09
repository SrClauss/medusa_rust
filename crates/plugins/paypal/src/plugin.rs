//! Core PayPal plugin implementation.
//!
//! PayPal uses OAuth2 client-credentials flow. The `initialize` method fetches
//! an access token that is cached in the plugin struct for subsequent requests.
//!
//! Mirrors the logic of the official MedusaJS PayPal plugin, adapted for Rust.

use anyhow::Context;
use async_trait::async_trait;
use plugin_api::{PaymentAction, PaymentProvider, WebhookActionResult, WebhookInfo, WebhookPayload};
use reqwest::Client;
use std::collections::HashMap;
use tracing::{error, info};

use crate::types::{CreateOrderRequest, Money, OAuthTokenResponse, PayPalWebhookEvent, PurchaseUnit};
use crate::webhooks;

/// Default PayPal sandbox API base URL.
const DEFAULT_BASE_URL: &str = "https://api-m.sandbox.paypal.com";
/// Token endpoint (relative to base_url).
const TOKEN_PATH: &str = "/v1/oauth2/token";

/// Payment provider plugin for PayPal.
pub struct PayPalPlugin {
    client_id: String,
    client_secret: String,
    /// Bearer access token obtained via OAuth2.
    access_token: String,
    base_url: String,
    client: Client,
}

impl PayPalPlugin {
    /// Create a new plugin with default (sandbox) configuration.
    pub fn new() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            access_token: String::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            client: Client::new(),
        }
    }

    /// Convenience constructor for tests — skips OAuth flow.
    pub fn with_token(access_token: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            access_token: access_token.into(),
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    /// Fetch a new OAuth2 access token from PayPal.
    async fn fetch_access_token(&mut self) -> anyhow::Result<()> {
        let resp = self
            .client
            .post(format!("{}{}", self.base_url, TOKEN_PATH))
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await
            .context("PayPal OAuth2 token request failed")?;

        let status = resp.status();
        let token_resp: OAuthTokenResponse = resp
            .json()
            .await
            .context("failed to parse PayPal OAuth2 token response")?;

        if !status.is_success() {
            anyhow::bail!("PayPal OAuth2 error {status}");
        }

        self.access_token = token_resp.access_token;
        Ok(())
    }

    /// Format a float amount as a string with 2 decimal places (PayPal requirement).
    fn format_amount(amount: f64) -> String {
        format!("{:.2}", amount)
    }
}

impl Default for PayPalPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PaymentProvider for PayPalPlugin {
    fn name(&self) -> &str {
        "paypal"
    }

    /// Initialize the plugin, fetching an OAuth2 access token.
    async fn initialize(&mut self, config: HashMap<String, String>) -> anyhow::Result<()> {
        if let Some(id) = config.get("client_id") {
            self.client_id = id.clone();
        } else {
            anyhow::bail!("PayPal: 'client_id' is required in config");
        }
        if let Some(secret) = config.get("client_secret") {
            self.client_secret = secret.clone();
        } else {
            anyhow::bail!("PayPal: 'client_secret' is required in config");
        }
        if let Some(url) = config.get("base_url") {
            self.base_url = url.clone();
        }

        // Allow tests to inject a pre-existing token to skip OAuth
        if let Some(token) = config.get("access_token") {
            self.access_token = token.clone();
        } else {
            self.fetch_access_token().await?;
        }

        info!(provider = "paypal", "PayPal plugin initialized");
        Ok(())
    }

    /// Create a PayPal order (equivalent to a payment intent).
    ///
    /// Returns the order JSON. The `CAPTURE` intent is used so that the buyer's
    /// payment method is charged at capture time, matching MedusaJS behavior.
    async fn create_payment(
        &self,
        amount: f64,
        currency: &str,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<serde_json::Value> {
        if amount <= 0.0 {
            anyhow::bail!("PayPal: amount must be positive");
        }

        let body = CreateOrderRequest {
            intent: "CAPTURE".to_string(),
            purchase_units: vec![PurchaseUnit {
                amount: Money {
                    currency_code: currency.to_uppercase(),
                    value: Self::format_amount(amount),
                },
                reference_id: metadata.get("order_id").cloned(),
            }],
        };

        let resp = self
            .client
            .post(format!("{}/v2/checkout/orders", self.base_url))
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .await
            .context("PayPal create_payment request failed")?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .context("failed to parse create_payment response")?;

        if !status.is_success() {
            error!(provider = "paypal", status = %status, "create_payment failed");
            anyhow::bail!("PayPal create_payment error {status}: {json}");
        }

        info!(
            provider = "paypal",
            order_id = json["id"].as_str().unwrap_or("?"),
            "PayPal order created"
        );
        Ok(json)
    }

    /// Capture a PayPal order.
    async fn capture_payment(
        &self,
        payment_id: &str,
        _amount: Option<f64>,
    ) -> anyhow::Result<serde_json::Value> {
        let resp = self
            .client
            .post(format!("{}/v2/checkout/orders/{payment_id}/capture", self.base_url))
            .bearer_auth(&self.access_token)
            .json(&serde_json::json!({}))
            .send()
            .await
            .context("PayPal capture_payment request failed")?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .context("failed to parse capture_payment response")?;

        if !status.is_success() {
            error!(provider = "paypal", payment_id, status = %status, "capture_payment failed");
            anyhow::bail!("PayPal capture_payment error {status}: {json}");
        }

        // Extract capture ID from the response for logging
        let capture_id = json["purchase_units"][0]["payments"]["captures"][0]["id"]
            .as_str()
            .unwrap_or("?");
        info!(provider = "paypal", payment_id, capture_id, "PayPal order captured");
        Ok(json)
    }

    /// Refund a captured PayPal payment.
    ///
    /// `payment_id` here must be the **capture ID** (not the order ID).
    async fn refund_payment(
        &self,
        payment_id: &str,
        amount: Option<f64>,
        _reason: Option<&str>,
    ) -> anyhow::Result<serde_json::Value> {
        let mut body = serde_json::json!({});
        if let Some(amt) = amount {
            body["amount"] = serde_json::json!({
                "value": Self::format_amount(amt),
                "currency_code": "USD"  // caller should pass currency via metadata
            });
        }

        let resp = self
            .client
            .post(format!("{}/v2/payments/captures/{payment_id}/refund", self.base_url))
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .await
            .context("PayPal refund_payment request failed")?;

        let status = resp.status();
        let json: serde_json::Value = resp
            .json()
            .await
            .context("failed to parse refund_payment response")?;

        if !status.is_success() {
            error!(provider = "paypal", payment_id, status = %status, "refund_payment failed");
            anyhow::bail!("PayPal refund_payment error {status}: {json}");
        }

        info!(provider = "paypal", payment_id, "PayPal payment refunded");
        Ok(json)
    }

    /// Cancel/void a PayPal order that has not been captured.
    ///
    /// Uncaptured orders can only be voided via the PayPal dashboard; the REST
    /// API does not provide a direct cancellation endpoint for orders.  We
    /// return a synthetic response to stay consistent with the trait interface.
    async fn cancel_payment(&self, payment_id: &str) -> anyhow::Result<serde_json::Value> {
        // PayPal does not expose a direct cancel endpoint for orders via REST API.
        // Authorized payments (PayPal BA agreements) can be voided; for standard
        // orders we return a best-effort synthetic response.
        info!(
            provider = "paypal",
            payment_id, "PayPal cancel requested (order expires automatically)"
        );
        Ok(serde_json::json!({
            "id": payment_id,
            "status": "VOIDED"
        }))
    }

    /// Parse a PayPal webhook event and return a normalized action.
    async fn get_webhook_action_and_data(
        &self,
        payload: WebhookPayload,
    ) -> anyhow::Result<WebhookActionResult> {
        let event: PayPalWebhookEvent = serde_json::from_value(payload.raw_data.clone())
            .context("failed to parse PayPal webhook event")?;

        let action = match event.event_type.as_str() {
            "CHECKOUT.ORDER.APPROVED" => PaymentAction::Authorized,
            "PAYMENT.CAPTURE.COMPLETED" => PaymentAction::Captured,
            "PAYMENT.CAPTURE.DENIED" | "PAYMENT.CAPTURE.DECLINED" => PaymentAction::Failed,
            "PAYMENT.CAPTURE.PENDING" => PaymentAction::Pending,
            "PAYMENT.CAPTURE.REFUNDED" | "PAYMENT.CAPTURE.REVERSED" => PaymentAction::Refunded,
            "PAYMENT.ORDER.CANCELLED" => PaymentAction::Cancelled,
            _ => PaymentAction::NotSupported,
        };

        let mut data = HashMap::new();
        data.insert(
            "event_type".to_string(),
            serde_json::Value::String(event.event_type),
        );
        data.insert("event_id".to_string(), serde_json::Value::String(event.id));
        data.insert("resource".to_string(), event.resource);

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
