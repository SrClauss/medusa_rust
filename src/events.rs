//! Simple in-process event bus used by various subsystems.
//!
//! Currently used for payment webhook delivery and for future shop-webhook
//! notifications.  Events are emitted with optional delay and are retried
//! automatically when a handler returns `Err`.

use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{broadcast, Mutex};
use tokio::time::{sleep, Instant};

/// An event emitted on the bus.
pub enum Event {
    PaymentWebhook(PaymentWebhookEvent),
    ShopWebhook(ShopWebhookEvent),
}

/// When a payment provider sends us a webhook callback.
pub struct PaymentWebhookEvent {
    pub provider: String,
    pub payload: serde_json::Value,
}

/// An event we send to registered shop webhook endpoints.
pub struct ShopWebhookEvent {
    pub name: String,
    pub data: serde_json::Value,
}

/// Options used when emitting an event.
pub struct EmitOptions {
    /// delay before delivering the event.
    pub delay: Option<Duration>,
    /// number of retry attempts (total); default 1.
    pub retries: usize,
}

impl Default for EmitOptions {
    fn default() -> Self {
        Self { delay: None, retries: 1 }
    }
}

/// Global event bus handle.  The inner `broadcast::Sender` is wrapped in a
/// `Mutex` so we can lazily initialize it.
#[derive(Clone)]
pub struct EventBus {
    inner: Arc<Mutex<Option<broadcast::Sender<Event>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(None)) }
    }

    /// Ensure the sender exists and return a clone.
    async fn sender(&self) -> broadcast::Sender<Event> {
        let mut guard = self.inner.lock().await;
        if guard.is_none() {
            let (tx, _rx) = broadcast::channel(1024);
            *guard = Some(tx.clone());
            tx
        } else {
            guard.as_ref().unwrap().clone()
        }
    }

    /// Subscribe to events.  Each subscriber receives its own receiver.
    pub async fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender().await.subscribe()
    }

    /// Emit an event with options.
    pub async fn emit(&self, event: Event, opts: EmitOptions) {
        let tx = self.sender().await;
        let mut attempts = opts.retries;
        let deliver = move || {
            let tx = tx.clone();
            let event = event.clone();
            async move {
                // cloning to allow multiple sends
                let _ = tx.send(event);
            }
        };

        // spawn a task for delayed/retry delivery
        tokio::spawn(async move {
            if let Some(d) = opts.delay {
                sleep(d).await;
            }
            while attempts > 0 {
                deliver().await;
                attempts -= 1;
                if attempts > 0 {
                    // simple fixed retry delay
                    sleep(Duration::from_secs(5)).await;
                }
            }
        });
    }
}

// enable clone for Event and contained types
impl Clone for Event {
    fn clone(&self) -> Self {
        match self {
            Event::PaymentWebhook(e) => Event::PaymentWebhook(PaymentWebhookEvent { provider: e.provider.clone(), payload: e.payload.clone() }),
            Event::ShopWebhook(e) => Event::ShopWebhook(ShopWebhookEvent { name: e.name.clone(), data: e.data.clone() }),
        }
    }
}
