//! SendGrid email notification provider.
//!
//! Enabled with Cargo feature `notify-sendgrid`.

#[cfg(feature = "notify-sendgrid")]
use crate::notifications::{Notification, NotificationChannel, NotificationProvider};
#[cfg(feature = "notify-sendgrid")]
use async_trait::async_trait;
#[cfg(feature = "notify-sendgrid")]
use serde_json::json;

/// SendGrid email provider using the v3 Mail Send API.
///
/// Set the `SENDGRID_API_KEY` and optionally `SENDGRID_FROM` environment
/// variables before using this provider.
#[cfg(feature = "notify-sendgrid")]
pub struct SendGridNotificationProvider {
    api_key: String,
    from: String,
    http: reqwest::Client,
}

#[cfg(feature = "notify-sendgrid")]
impl SendGridNotificationProvider {
    /// Create a new provider.
    ///
    /// `api_key` — SendGrid API key (starts with `SG.`).
    /// `from`    — default sender address.
    pub fn new(api_key: String, from: String) -> Self {
        Self {
            api_key,
            from,
            http: reqwest::Client::new(),
        }
    }

    /// Builds from environment variables (`SENDGRID_API_KEY`, `SENDGRID_FROM`).
    pub fn from_env() -> anyhow::Result<Self> {
        let api_key = std::env::var("SENDGRID_API_KEY")
            .map_err(|_| anyhow::anyhow!("SENDGRID_API_KEY not set"))?;
        let from = std::env::var("SENDGRID_FROM")
            .unwrap_or_else(|_| "noreply@example.com".into());
        Ok(Self::new(api_key, from))
    }
}

#[cfg(feature = "notify-sendgrid")]
#[async_trait]
impl NotificationProvider for SendGridNotificationProvider {
    fn name(&self) -> &str {
        "sendgrid"
    }

    fn channel(&self) -> NotificationChannel {
        NotificationChannel::Email
    }

    async fn send(&self, notification: Notification) -> anyhow::Result<Option<String>> {
        let from_address = notification
            .from
            .as_deref()
            .unwrap_or(&self.from);

        let body = json!({
            "personalizations": [{
                "to": [{ "email": notification.to }],
                "dynamic_template_data": notification.data,
            }],
            "from": { "email": from_address },
            "subject": notification.subject,
            "template_id": notification.template,
        });

        let resp = self
            .http
            .post("https://api.sendgrid.com/v3/mail/send")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        // SendGrid returns the message ID in the `X-Message-Id` header.
        let message_id = resp
            .headers()
            .get("X-Message-Id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Ok(message_id)
    }
}
