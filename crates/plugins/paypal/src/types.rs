//! PayPal-specific request/response types.

use serde::{Deserialize, Serialize};

/// OAuth2 token response from PayPal.
#[derive(Debug, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// A unit of currency amount for PayPal API requests.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Money {
    pub currency_code: String,
    pub value: String,
}

/// Order creation request for the PayPal Orders API.
#[derive(Debug, Serialize)]
pub struct CreateOrderRequest {
    pub intent: String,
    pub purchase_units: Vec<PurchaseUnit>,
}

/// A purchase unit inside an order request.
#[derive(Debug, Serialize)]
pub struct PurchaseUnit {
    pub amount: Money,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
}

/// PayPal order response (partial).
#[derive(Debug, Deserialize)]
pub struct OrderResponse {
    pub id: String,
    pub status: String,
}

/// PayPal capture response for an order.
#[derive(Debug, Deserialize)]
pub struct CaptureOrderResponse {
    pub id: String,
    pub status: String,
}

/// PayPal refund response.
#[derive(Debug, Deserialize)]
pub struct RefundResponse {
    pub id: String,
    pub status: String,
}

/// PayPal webhook event envelope.
#[derive(Debug, Deserialize)]
pub struct PayPalWebhookEvent {
    pub id: String,
    pub event_type: String,
    pub resource: serde_json::Value,
}

/// Request body for registering a webhook in PayPal.
#[derive(Debug, Serialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub event_types: Vec<PayPalEventType>,
}

/// An event type entry for PayPal webhook registration.
#[derive(Debug, Serialize)]
pub struct PayPalEventType {
    pub name: String,
}

/// A PayPal webhook as returned by the management API.
#[derive(Debug, Deserialize)]
pub struct PayPalWebhookInfo {
    pub id: String,
    pub url: String,
    #[serde(default)]
    pub event_types: Vec<PayPalWebhookEventType>,
}

/// Event type entry in a PayPal webhook info response.
#[derive(Debug, Deserialize)]
pub struct PayPalWebhookEventType {
    pub name: String,
}

/// Response from listing PayPal webhooks.
#[derive(Debug, Deserialize)]
pub struct ListWebhooksResponse {
    pub webhooks: Vec<PayPalWebhookInfo>,
}
