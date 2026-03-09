//! SMTP email notification provider using `lettre`.
//!
//! Enabled with Cargo feature `notify-smtp`.

#[cfg(feature = "notify-smtp")]
use crate::notifications::{Notification, NotificationChannel, NotificationProvider};
#[cfg(feature = "notify-smtp")]
use async_trait::async_trait;
#[cfg(feature = "notify-smtp")]
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

/// SMTP email provider backed by `lettre`.
///
/// Configure via environment variables:
/// - `SMTP_HOST`     — SMTP server hostname
/// - `SMTP_PORT`     — SMTP server port (default `587`)
/// - `SMTP_USERNAME` — SMTP login username
/// - `SMTP_PASSWORD` — SMTP login password
/// - `SMTP_FROM`     — default sender address
#[cfg(feature = "notify-smtp")]
pub struct SmtpNotificationProvider {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from: String,
}

#[cfg(feature = "notify-smtp")]
impl SmtpNotificationProvider {
    pub fn new(
        host: &str,
        port: u16,
        username: String,
        password: String,
        from: String,
    ) -> anyhow::Result<Self> {
        let creds = Credentials::new(username, password);
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(host)?
            .port(port)
            .credentials(creds)
            .build();
        Ok(Self { mailer, from })
    }

    /// Builds from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        let host = std::env::var("SMTP_HOST")
            .map_err(|_| anyhow::anyhow!("SMTP_HOST not set"))?;
        let port = std::env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(587u16);
        let username = std::env::var("SMTP_USERNAME")
            .map_err(|_| anyhow::anyhow!("SMTP_USERNAME not set"))?;
        let password = std::env::var("SMTP_PASSWORD")
            .map_err(|_| anyhow::anyhow!("SMTP_PASSWORD not set"))?;
        let from = std::env::var("SMTP_FROM")
            .unwrap_or_else(|_| "noreply@example.com".into());
        Self::new(&host, port, username, password, from)
    }
}

#[cfg(feature = "notify-smtp")]
#[async_trait]
impl NotificationProvider for SmtpNotificationProvider {
    fn name(&self) -> &str {
        "smtp"
    }

    fn channel(&self) -> NotificationChannel {
        NotificationChannel::Email
    }

    async fn send(&self, notification: Notification) -> anyhow::Result<Option<String>> {
        let from_address = notification
            .from
            .as_deref()
            .unwrap_or(&self.from);

        let subject = notification
            .subject
            .unwrap_or_else(|| notification.template.clone());

        // Render body: use the `data` as a simple JSON string for now.
        // In a real implementation you would render an HTML template here.
        let body = serde_json::to_string_pretty(&notification.data)
            .unwrap_or_else(|_| notification.data.to_string());

        let email = Message::builder()
            .from(from_address.parse()?)
            .to(notification.to.parse()?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)?;

        self.mailer.send(email).await?;
        Ok(None)
    }
}
