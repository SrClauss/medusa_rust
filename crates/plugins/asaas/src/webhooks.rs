//! Webhook management helpers for the Asaas API.

use anyhow::Context;
use plugin_api::WebhookInfo;
use reqwest::Client;

use crate::types::{AsaasWebhookInfo, CreateWebhookRequest, ListWebhooksResponse};

/// Create a webhook endpoint in Asaas.
///
/// # Arguments
/// * `client` – HTTP client.
/// * `base_url` – Base URL of the Asaas API (e.g. `https://api.asaas.com/v3`).
/// * `api_key` – Asaas API key (`access_token` header).
/// * `url` – Target URL that will receive events.
/// * `events` – List of event names to subscribe to.
///
/// Returns the provider-assigned webhook ID.
pub async fn create_webhook(
    client: &Client,
    base_url: &str,
    api_key: &str,
    url: &str,
    events: Vec<String>,
) -> anyhow::Result<String> {
    let body = CreateWebhookRequest {
        url: url.to_string(),
        email: String::new(),
        events,
        enabled: true,
        interrupted: false,
        auth_token: None,
    };

    let resp = client
        .post(format!("{base_url}/webhooks"))
        .header("access_token", api_key)
        .json(&body)
        .send()
        .await
        .context("Asaas create_webhook request failed")?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await.context("failed to parse create_webhook response")?;

    if !status.is_success() {
        anyhow::bail!("Asaas create_webhook error {status}: {json}");
    }

    let id = json["id"]
        .as_str()
        .context("create_webhook response missing 'id'")?
        .to_string();

    tracing::info!(webhook_id = %id, "Asaas webhook created");
    Ok(id)
}

/// List all webhook endpoints registered in Asaas.
pub async fn list_webhooks(
    client: &Client,
    base_url: &str,
    api_key: &str,
) -> anyhow::Result<Vec<WebhookInfo>> {
    let resp = client
        .get(format!("{base_url}/webhooks"))
        .header("access_token", api_key)
        .send()
        .await
        .context("Asaas list_webhooks request failed")?;

    let status = resp.status();
    let list: ListWebhooksResponse = resp
        .json()
        .await
        .context("failed to parse list_webhooks response")?;

    if !status.is_success() {
        anyhow::bail!("Asaas list_webhooks error {status}");
    }

    Ok(list
        .data
        .into_iter()
        .map(|w: AsaasWebhookInfo| WebhookInfo {
            id: w.id,
            url: w.url,
            events: w.events,
            active: w.enabled,
        })
        .collect())
}

/// Delete a webhook endpoint from Asaas.
pub async fn delete_webhook(
    client: &Client,
    base_url: &str,
    api_key: &str,
    webhook_id: &str,
) -> anyhow::Result<()> {
    let resp = client
        .delete(format!("{base_url}/webhooks/{webhook_id}"))
        .header("access_token", api_key)
        .send()
        .await
        .context("Asaas delete_webhook request failed")?;

    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("Asaas delete_webhook error {status}");
    }

    tracing::info!(webhook_id = %webhook_id, "Asaas webhook deleted");
    Ok(())
}
