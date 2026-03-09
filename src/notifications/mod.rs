//! Notification system — provider trait, service orchestration, and concrete
//! provider implementations.
//!
//! Mirrors MedusaJS's `NotificationService` pattern where pluggable providers
//! handle delivery and the service layer manages routing and persistence.
//!
//! # Enabled providers
//!
//! | Feature          | Provider                    |
//! |------------------|-----------------------------|
//! | `notify-sendgrid`| `SendGridNotificationProvider` |
//! | `notify-smtp`    | `SmtpNotificationProvider`  |
//! | `notify-twilio`  | `TwilioNotificationProvider`|

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// ─── Sub-modules ──────────────────────────────────────────────────────────────

pub mod sendgrid;
pub mod smtp;
pub mod twilio;

// ─── Core types ───────────────────────────────────────────────────────────────

/// A notification to be sent to a recipient.
///
/// Mirrors `NotificationService.sendNotification()` data shape from MedusaJS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Destination address — email address or phone number.
    pub to: String,
    /// Optional subject line (email only).
    pub subject: Option<String>,
    /// Template identifier, e.g. `"order.placed"`.
    pub template: String,
    /// Template variables.
    pub data: serde_json::Value,
    /// Optional sender address / phone number.  Uses provider default when `None`.
    pub from: Option<String>,
}

/// Delivery channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Sms,
}

/// Trait that every notification backend must implement.
#[async_trait]
pub trait NotificationProvider: Send + Sync {
    /// Short name, e.g. `"sendgrid"`, `"smtp"`, `"twilio"`.
    fn name(&self) -> &str;

    /// Channel this provider supports.
    fn channel(&self) -> NotificationChannel;

    /// Send the notification.  Returns an opaque external message ID on success.
    async fn send(&self, notification: Notification) -> anyhow::Result<Option<String>>;
}

// ─── Service ──────────────────────────────────────────────────────────────────

use std::collections::HashMap;
use std::sync::Arc;

/// Central service that routes notifications to the registered provider.
///
/// In MedusaJS each provider is registered by name in the container; here we
/// store them in a plain `HashMap`.
pub struct NotificationService {
    email_provider: Option<Arc<dyn NotificationProvider>>,
    sms_provider: Option<Arc<dyn NotificationProvider>>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            email_provider: None,
            sms_provider: None,
        }
    }

    /// Register a provider.  The last one registered per channel wins.
    pub fn register(&mut self, provider: Arc<dyn NotificationProvider>) {
        tracing::info!(
            provider = %provider.name(),
            channel = ?provider.channel(),
            "Notification provider registered"
        );
        match provider.channel() {
            NotificationChannel::Email => self.email_provider = Some(provider),
            NotificationChannel::Sms   => self.sms_provider   = Some(provider),
        }
    }

    /// Send a notification via the appropriate registered provider.
    pub async fn send(
        &self,
        channel: NotificationChannel,
        notification: Notification,
    ) -> anyhow::Result<Option<String>> {
        let provider = match channel {
            NotificationChannel::Email => self.email_provider.as_ref(),
            NotificationChannel::Sms   => self.sms_provider.as_ref(),
        };

        let provider = provider.ok_or_else(|| {
            anyhow::anyhow!("No {:?} notification provider registered", channel)
        })?;

        provider.send(notification).await
    }

    /// Convenience helper — send an email notification.
    pub async fn send_email(&self, notification: Notification) -> anyhow::Result<Option<String>> {
        self.send(NotificationChannel::Email, notification).await
    }

    /// Convenience helper — send an SMS notification.
    pub async fn send_sms(&self, notification: Notification) -> anyhow::Result<Option<String>> {
        self.send(NotificationChannel::Sms, notification).await
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}
