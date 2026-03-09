//! Twilio SMS notification provider.
//!
//! Enabled with Cargo feature `notify-twilio`.

#[cfg(feature = "notify-twilio")]
use crate::notifications::{Notification, NotificationChannel, NotificationProvider};
#[cfg(feature = "notify-twilio")]
use async_trait::async_trait;

/// Twilio SMS provider using the Messages API.
///
/// Configure via environment variables:
/// - `TWILIO_ACCOUNT_SID` — your Twilio account SID
/// - `TWILIO_AUTH_TOKEN`  — your Twilio auth token
/// - `TWILIO_FROM`        — sender phone number or messaging service SID
#[cfg(feature = "notify-twilio")]
pub struct TwilioNotificationProvider {
    account_sid: String,
    auth_token: String,
    from: String,
    http: reqwest::Client,
}

#[cfg(feature = "notify-twilio")]
impl TwilioNotificationProvider {
    pub fn new(account_sid: String, auth_token: String, from: String) -> Self {
        Self {
            account_sid,
            auth_token,
            from,
            http: reqwest::Client::new(),
        }
    }

    /// Builds from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        let account_sid = std::env::var("TWILIO_ACCOUNT_SID")
            .map_err(|_| anyhow::anyhow!("TWILIO_ACCOUNT_SID not set"))?;
        let auth_token = std::env::var("TWILIO_AUTH_TOKEN")
            .map_err(|_| anyhow::anyhow!("TWILIO_AUTH_TOKEN not set"))?;
        let from = std::env::var("TWILIO_FROM")
            .map_err(|_| anyhow::anyhow!("TWILIO_FROM not set"))?;
        Ok(Self::new(account_sid, auth_token, from))
    }
}

#[cfg(feature = "notify-twilio")]
#[async_trait]
impl NotificationProvider for TwilioNotificationProvider {
    fn name(&self) -> &str {
        "twilio"
    }

    fn channel(&self) -> NotificationChannel {
        NotificationChannel::Sms
    }

    async fn send(&self, notification: Notification) -> anyhow::Result<Option<String>> {
        let from_number = notification
            .from
            .as_deref()
            .unwrap_or(&self.from);

        // Format SMS body: "{template}: {key}={value}, ..." for readability.
        // In production, replace this with a proper template rendering step.
        let body = if let Some(obj) = notification.data.as_object() {
            let pairs: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}={}", k, v.as_str().unwrap_or(&v.to_string())))
                .collect();
            format!("{}: {}", notification.template, pairs.join(", "))
        } else {
            format!("{}: {}", notification.template, notification.data)
        };

        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.account_sid
        );

        let resp: serde_json::Value = self
            .http
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&[("To", notification.to.as_str()), ("From", from_number), ("Body", body.as_str())])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let sid = resp
            .get("sid")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        Ok(sid)
    }
}
