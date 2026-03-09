//! Webhook management helpers for the PayPal REST API.

use anyhow::Context;
use plugin_api::WebhookInfo;
use reqwest::Client;

use crate::types::{CreateWebhookRequest, ListWebhooksResponse, PayPalEventType, PayPalWebhookInfo};

/// Create a webhook endpoint in PayPal.
pub async fn create_webhook(
    client: &Client,
    base_url: &str,
    access_token: &str,
    url: &str,
    events: Vec<String>,
) -> anyhow::Result<String> {
    let body = CreateWebhookRequest {
        url: url.to_string(),
        event_types: events
            .into_iter()
            .map(|e| PayPalEventType { name: e })
            .collect(),
    };

    let resp = client
        .post(format!("{base_url}/v1/notifications/webhooks"))
        .bearer_auth(access_token)
        .json(&body)
        .send()
        .await
        .context("PayPal create_webhook request failed")?;

    let status = resp.status();
    let json: serde_json::Value = resp
        .json()
        .await
        .context("failed to parse create_webhook response")?;

    if !status.is_success() {
        anyhow::bail!("PayPal create_webhook error {status}: {json}");
    }

    let id = json["id"]
        .as_str()
        .context("create_webhook response missing 'id'")?
        .to_string();

    tracing::info!(webhook_id = %id, "PayPal webhook created");
    Ok(id)
}

/// List webhook endpoints registered in PayPal.
pub async fn list_webhooks(
    client: &Client,
    base_url: &str,
    access_token: &str,
) -> anyhow::Result<Vec<WebhookInfo>> {
    let resp = client
        .get(format!("{base_url}/v1/notifications/webhooks"))
        .bearer_auth(access_token)
        .send()
        .await
        .context("PayPal list_webhooks request failed")?;

    let status = resp.status();
    let list: ListWebhooksResponse = resp
        .json()
        .await
        .context("failed to parse list_webhooks response")?;

    if !status.is_success() {
        anyhow::bail!("PayPal list_webhooks error {status}");
    }

    Ok(list
        .webhooks
        .into_iter()
        .map(|w: PayPalWebhookInfo| WebhookInfo {
            id: w.id,
            url: w.url,
            events: w.event_types.into_iter().map(|e| e.name).collect(),
            active: true,
        })
        .collect())
}

/// Delete a webhook endpoint from PayPal.
pub async fn delete_webhook(
    client: &Client,
    base_url: &str,
    access_token: &str,
    webhook_id: &str,
) -> anyhow::Result<()> {
    let resp = client
        .delete(format!("{base_url}/v1/notifications/webhooks/{webhook_id}"))
        .bearer_auth(access_token)
        .send()
        .await
        .context("PayPal delete_webhook request failed")?;

    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("PayPal delete_webhook error {status}");
    }

    tracing::info!(webhook_id = %webhook_id, "PayPal webhook deleted");
    Ok(())
}
