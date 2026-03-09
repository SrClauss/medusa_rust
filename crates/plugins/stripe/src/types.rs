//! Stripe-specific request/response types.

use serde::{Deserialize, Serialize};

/// A Stripe PaymentIntent object (partial).
#[derive(Debug, Deserialize)]
pub struct PaymentIntent {
    pub id: String,
    pub status: String,
    pub amount: i64,
    pub currency: String,
    #[serde(default)]
    pub client_secret: Option<String>,
}

/// A Stripe refund object (partial).
#[derive(Debug, Deserialize)]
pub struct Refund {
    pub id: String,
    pub status: String,
    pub amount: i64,
}

/// A Stripe webhook endpoint object (partial).
#[derive(Debug, Deserialize)]
pub struct StripeWebhookEndpoint {
    pub id: String,
    pub url: String,
    pub enabled_events: Vec<String>,
    pub status: String,
}

/// Response from `GET /v1/webhook_endpoints`.
#[derive(Debug, Deserialize)]
pub struct ListWebhookEndpointsResponse {
    pub data: Vec<StripeWebhookEndpoint>,
}

/// Stripe webhook event envelope.
#[derive(Debug, Deserialize)]
pub struct StripeWebhookEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: StripeWebhookEventData,
}

/// Data object inside a Stripe webhook event.
#[derive(Debug, Deserialize)]
pub struct StripeWebhookEventData {
    pub object: serde_json::Value,
}

/// Form-encoded body for creating a PaymentIntent via Stripe API.
#[derive(Debug, Serialize)]
pub struct CreatePaymentIntentForm {
    pub amount: String,
    pub currency: String,
    pub capture_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}
