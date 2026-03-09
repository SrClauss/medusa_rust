//! Mercado Pago-specific request/response types.

use serde::{Deserialize, Serialize};

/// Request body for creating a Mercado Pago payment.
#[derive(Debug, Serialize)]
pub struct CreatePaymentRequest {
    pub transaction_amount: f64,
    pub currency_id: String,
    pub description: String,
    pub payment_method_id: String,
    pub payer: PayerInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_reference: Option<String>,
}

/// Payer information for a Mercado Pago payment.
#[derive(Debug, Serialize, Deserialize)]
pub struct PayerInfo {
    pub email: String,
}

/// Mercado Pago payment response.
#[derive(Debug, Deserialize)]
pub struct PaymentResponse {
    pub id: i64,
    pub status: String,
    pub status_detail: String,
    pub transaction_amount: f64,
    pub currency_id: String,
}

/// Mercado Pago webhook notification.
#[derive(Debug, Deserialize)]
pub struct MpWebhookNotification {
    pub action: Option<String>,
    #[serde(rename = "type")]
    pub notification_type: Option<String>,
    pub data: Option<MpWebhookData>,
}

/// Data object embedded in a Mercado Pago webhook notification.
#[derive(Debug, Deserialize)]
pub struct MpWebhookData {
    pub id: Option<serde_json::Value>,
}

/// Request body for creating a Mercado Pago webhook.
#[derive(Debug, Serialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub events: Vec<MpWebhookEvent>,
}

/// A single event object in the MP webhook request.
#[derive(Debug, Serialize)]
pub struct MpWebhookEvent {
    pub name: String,
}

/// Mercado Pago webhook info from the API.
#[derive(Debug, Deserialize)]
pub struct MpWebhookInfo {
    pub id: String,
    pub url: String,
    #[serde(default)]
    pub events: Vec<MpWebhookEventInfo>,
    #[serde(default)]
    pub active: bool,
}

/// Event info returned from the MP webhooks list.
#[derive(Debug, Deserialize)]
pub struct MpWebhookEventInfo {
    pub name: String,
}

/// Response from listing MP webhooks.
#[derive(Debug, Deserialize)]
pub struct ListWebhooksResponse {
    pub results: Vec<MpWebhookInfo>,
}
