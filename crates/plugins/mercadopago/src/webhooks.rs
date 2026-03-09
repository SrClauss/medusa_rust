//! Webhook management helpers for Mercado Pago.

use anyhow::Context;
use plugin_api::WebhookInfo;
use reqwest::Client;

use crate::types::{CreateWebhookRequest, ListWebhooksResponse, MpWebhookEvent};

/// Create a webhook endpoint in Mercado Pago.
pub async fn create_webhook(
    client: &Client,
    base_url: &str,
    access_token: &str,
    url: &str,
    events: Vec<String>,
) -> anyhow::Result<String> {
    let body = CreateWebhookRequest {
        url: url.to_string(),
        events: events.into_iter().map(|e| MpWebhookEvent { name: e }).collect(),
    };

    let resp = client
        .post(format!("{base_url}/v1/webhooks"))
        .bearer_auth(access_token)
        .json(&body)
        .send()
        .await
        .context("MercadoPago create_webhook request failed")?;

    let status = resp.status();
    let json: serde_json::Value = resp
        .json()
        .await
        .context("failed to parse create_webhook response")?;

    if !status.is_success() {
        anyhow::bail!("MercadoPago create_webhook error {status}: {json}");
    }

    let id = json["id"]
        .as_str()
        .or_else(|| json["id"].as_i64().map(|_| ""))
        .unwrap_or("")
        .to_string();

    // MP may return numeric ID
    let id = if id.is_empty() {
        json["id"].to_string()
    } else {
        id
    };

    tracing::info!(webhook_id = %id, "MercadoPago webhook created");
    Ok(id)
}

/// List webhook endpoints registered in Mercado Pago.
pub async fn list_webhooks(
    client: &Client,
    base_url: &str,
    access_token: &str,
) -> anyhow::Result<Vec<WebhookInfo>> {
    let resp = client
        .get(format!("{base_url}/v1/webhooks"))
        .bearer_auth(access_token)
        .send()
        .await
        .context("MercadoPago list_webhooks request failed")?;

    let status = resp.status();
    let list: ListWebhooksResponse = resp
        .json()
        .await
        .context("failed to parse list_webhooks response")?;

    if !status.is_success() {
        anyhow::bail!("MercadoPago list_webhooks error {status}");
    }

    Ok(list
        .results
        .into_iter()
        .map(|w| WebhookInfo {
            id: w.id,
            url: w.url,
            events: w.events.into_iter().map(|e| e.name).collect(),
            active: w.active,
        })
        .collect())
}

/// Delete a webhook endpoint from Mercado Pago.
pub async fn delete_webhook(
    client: &Client,
    base_url: &str,
    access_token: &str,
    webhook_id: &str,
) -> anyhow::Result<()> {
    let resp = client
        .delete(format!("{base_url}/v1/webhooks/{webhook_id}"))
        .bearer_auth(access_token)
        .send()
        .await
        .context("MercadoPago delete_webhook request failed")?;

    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("MercadoPago delete_webhook error {status}");
    }

    tracing::info!(webhook_id = %webhook_id, "MercadoPago webhook deleted");
    Ok(())
}
