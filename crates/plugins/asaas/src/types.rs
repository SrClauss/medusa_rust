//! Asaas-specific request/response types.

use serde::{Deserialize, Serialize};

/// Request body for creating an Asaas charge.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChargeRequest {
    pub customer: String,
    pub billing_type: String,
    pub value: f64,
    pub due_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_reference: Option<String>,
}

/// Response body returned by Asaas when creating a charge.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChargeResponse {
    pub id: String,
    pub status: String,
    pub value: f64,
    pub billing_type: String,
    #[serde(default)]
    pub due_date: String,
    #[serde(default)]
    pub invoice_url: Option<String>,
    #[serde(default)]
    pub bank_slip_url: Option<String>,
}

/// Asaas webhook event envelope.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsaasWebhookEvent {
    pub event: String,
    pub payment: Option<AsaasWebhookPayment>,
}

/// Payment object embedded in an Asaas webhook event.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsaasWebhookPayment {
    pub id: String,
    pub status: String,
    pub value: f64,
    #[serde(default)]
    pub external_reference: Option<String>,
}

/// Request body for registering a webhook in Asaas.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWebhookRequest {
    pub url: String,
    pub email: String,
    pub events: Vec<String>,
    pub enabled: bool,
    pub interrupted: bool,
    pub auth_token: Option<String>,
}

/// A webhook endpoint as returned by the Asaas API.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsaasWebhookInfo {
    pub id: String,
    pub url: String,
    pub email: String,
    pub enabled: bool,
    pub events: Vec<String>,
}

/// Response from `GET /webhooks`.
#[derive(Debug, Deserialize)]
pub struct ListWebhooksResponse {
    pub data: Vec<AsaasWebhookInfo>,
}
