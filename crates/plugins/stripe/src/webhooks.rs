//! Webhook management helpers for the Stripe API.

use anyhow::Context;
use plugin_api::WebhookInfo;
use reqwest::Client;

use crate::types::{ListWebhookEndpointsResponse, StripeWebhookEndpoint};

/// Create a webhook endpoint in Stripe.
///
/// Stripe uses form-encoded bodies for its REST API.
pub async fn create_webhook(
    client: &Client,
    base_url: &str,
    secret_key: &str,
    url: &str,
    events: Vec<String>,
) -> anyhow::Result<String> {
    let mut form_params: Vec<(&str, String)> = vec![("url", url.to_string())];
    for event in &events {
        form_params.push(("enabled_events[]", event.clone()));
    }

    let resp = client
        .post(format!("{base_url}/v1/webhook_endpoints"))
        .basic_auth(secret_key, Some(""))
        .form(&form_params)
        .send()
        .await
        .context("Stripe create_webhook request failed")?;

    let status = resp.status();
    let json: serde_json::Value = resp
        .json()
        .await
        .context("failed to parse create_webhook response")?;

    if !status.is_success() {
        anyhow::bail!("Stripe create_webhook error {status}: {json}");
    }

    let id = json["id"]
        .as_str()
        .context("create_webhook response missing 'id'")?
        .to_string();

    tracing::info!(webhook_id = %id, "Stripe webhook created");
    Ok(id)
}

/// List all webhook endpoints registered in Stripe.
pub async fn list_webhooks(
    client: &Client,
    base_url: &str,
    secret_key: &str,
) -> anyhow::Result<Vec<WebhookInfo>> {
    let resp = client
        .get(format!("{base_url}/v1/webhook_endpoints"))
        .basic_auth(secret_key, Some(""))
        .send()
        .await
        .context("Stripe list_webhooks request failed")?;

    let status = resp.status();
    let list: ListWebhookEndpointsResponse = resp
        .json()
        .await
        .context("failed to parse list_webhooks response")?;

    if !status.is_success() {
        anyhow::bail!("Stripe list_webhooks error {status}");
    }

    Ok(list
        .data
        .into_iter()
        .map(|w: StripeWebhookEndpoint| WebhookInfo {
            id: w.id,
            url: w.url,
            events: w.enabled_events,
            active: w.status == "enabled",
        })
        .collect())
}

/// Delete a webhook endpoint from Stripe.
pub async fn delete_webhook(
    client: &Client,
    base_url: &str,
    secret_key: &str,
    webhook_id: &str,
) -> anyhow::Result<()> {
    let resp = client
        .delete(format!("{base_url}/v1/webhook_endpoints/{webhook_id}"))
        .basic_auth(secret_key, Some(""))
        .send()
        .await
        .context("Stripe delete_webhook request failed")?;

    let status = resp.status();
    if !status.is_success() {
        let json: serde_json::Value = resp
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({}));
        anyhow::bail!("Stripe delete_webhook error {status}: {json}");
    }

    tracing::info!(webhook_id = %webhook_id, "Stripe webhook deleted");
    Ok(())
}

/// Verify a Stripe webhook signature.
///
/// Stripe sends a `Stripe-Signature` header containing a timestamp and HMAC-SHA256
/// signature.  This function validates the signature using the webhook secret.
///
/// Returns `Ok(())` if the signature is valid, `Err` otherwise.
pub fn verify_webhook_signature(
    payload: &[u8],
    sig_header: &str,
    webhook_secret: &str,
) -> anyhow::Result<()> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    // Parse the Stripe-Signature header: "t=timestamp,v1=signature,..."
    let mut timestamp = "";
    let mut expected_sig = "";
    for part in sig_header.split(',') {
        if let Some(ts) = part.strip_prefix("t=") {
            timestamp = ts;
        } else if let Some(sig) = part.strip_prefix("v1=") {
            expected_sig = sig;
        }
    }

    if timestamp.is_empty() {
        anyhow::bail!("Stripe-Signature header missing timestamp");
    }
    if expected_sig.is_empty() {
        anyhow::bail!("Stripe-Signature header missing v1 signature");
    }

    // Build signed payload: "{timestamp}.{body}"
    let signed_payload = format!(
        "{}.{}",
        timestamp,
        std::str::from_utf8(payload).context("webhook payload is not valid UTF-8")?
    );

    let mut mac = Hmac::<Sha256>::new_from_slice(webhook_secret.as_bytes())
        .context("invalid webhook secret length")?;
    mac.update(signed_payload.as_bytes());
    let result = hex::encode(mac.finalize().into_bytes());

    if result != expected_sig {
        anyhow::bail!("Stripe webhook signature mismatch");
    }

    Ok(())
}
