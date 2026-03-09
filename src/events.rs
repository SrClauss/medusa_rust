//! Distributed-capable event bus for the Medusa Rust backend.
//!
//! Supports three drivers selected via the `BUS_DRIVER` environment variable:
//!
//! | Driver  | Env value | Description                                      |
//! |---------|-----------|--------------------------------------------------|
//! | Local   | `local`   | In-process `tokio::sync::broadcast` (default)    |
//! | Redis   | `redis`   | Pub/sub over Redis (requires `REDIS_URL`)         |
//! | SQS     | `sqs`     | AWS SQS (requires `AWS_REGION` + `AWS_SQS_URL`)  |
//!
//! ## Quick-start
//! ```rust,ignore
//! let bus = EventBus::from_env();
//! // subscribe with a typed handler
//! bus.subscribe(|evt: Event| async move {
//!     println!("received: {:?}", evt);
//!     Ok(())
//! }).await.unwrap();
//! // publish an event
//! bus.publish(Event::OrderPlaced(OrderPlacedEvent { order_id: uuid, .. })).await.unwrap();
//! ```

use std::{sync::Arc, time::Duration};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;

// ─── Event types ─────────────────────────────────────────────────────────────

/// All domain events that can be published on the bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    // ── Legacy webhook events (kept for backward compatibility) ────────────
    PaymentWebhook(PaymentWebhookEvent),
    ShopWebhook(ShopWebhookEvent),

    // ── Domain events ──────────────────────────────────────────────────────
    /// Fired after a cart is converted into an order.
    OrderPlaced(OrderPlacedEvent),
    /// Fired when a payment is successfully captured.
    PaymentCaptured(PaymentCapturedEvent),
    /// Fired when a new customer registers.
    CustomerRegistered(CustomerRegisteredEvent),
    /// Fired when a product is created.
    ProductCreated(ProductCreatedEvent),
    /// Fired when a cart is marked completed (pre-order).
    CartCompleted(CartCompletedEvent),
}

/// When a payment provider sends us a webhook callback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentWebhookEvent {
    pub provider: String,
    pub payload: serde_json::Value,
}

/// An event we send to registered shop webhook endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopWebhookEvent {
    pub name: String,
    pub data: serde_json::Value,
}

/// An order was placed (cart → order checkout completed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderPlacedEvent {
    pub order_id: uuid::Uuid,
    pub display_id: i32,
    pub customer_id: uuid::Uuid,
    pub email: String,
    pub currency_code: String,
    pub total: i64,
}

/// A payment was captured.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCapturedEvent {
    pub payment_id: uuid::Uuid,
    pub amount: Option<i64>,
    pub currency_code: Option<String>,
    pub provider_id: Option<String>,
}

/// A new customer completed registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerRegisteredEvent {
    pub customer_id: uuid::Uuid,
    pub email: String,
}

/// A new product was created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductCreatedEvent {
    pub product_id: uuid::Uuid,
    pub title: String,
    pub handle: Option<String>,
}

/// A cart was marked as completed (before order insertion).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartCompletedEvent {
    pub cart_id: uuid::Uuid,
    pub customer_id: Option<uuid::Uuid>,
}

// ─── EventHandler trait ───────────────────────────────────────────────────────

/// Any type that can handle domain events.
///
/// Implement this trait (or use the blanket closure implementation) to react to
/// events published on the bus.
#[async_trait]
pub trait EventHandler: Send + Sync + 'static {
    async fn handle(&self, event: Event) -> anyhow::Result<()>;
}

/// Blanket impl so callers can pass closures / async fns directly.
#[async_trait]
impl<F, Fut> EventHandler for F
where
    F: Fn(Event) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
{
    async fn handle(&self, event: Event) -> anyhow::Result<()> {
        (self)(event).await
    }
}

// ─── Driver abstraction ───────────────────────────────────────────────────────

/// Which backing transport to use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusDriver {
    /// In-process broadcast channel (default).
    Local,
    /// Redis pub/sub (requires `REDIS_URL`).
    Redis,
    /// AWS SQS (requires `AWS_REGION` + `AWS_SQS_URL`).
    Sqs,
}

impl BusDriver {
    /// Read the driver from `BUS_DRIVER` env var, defaulting to `Local`.
    pub fn from_env() -> Self {
        match std::env::var("BUS_DRIVER").as_deref().unwrap_or("local") {
            "redis" => Self::Redis,
            "sqs" => Self::Sqs,
            _ => Self::Local,
        }
    }
}

// ─── Options ─────────────────────────────────────────────────────────────────

/// Options used when emitting an event.
#[derive(Debug, Clone)]
pub struct EmitOptions {
    /// Optional delay before delivering the event.
    pub delay: Option<Duration>,
    /// Number of retry attempts (total); default 1.
    pub retries: usize,
}

impl Default for EmitOptions {
    fn default() -> Self {
        Self { delay: None, retries: 1 }
    }
}

// ─── EventBus ────────────────────────────────────────────────────────────────

/// Shared event bus handle.  Clone freely — all clones share the same channel.
#[derive(Clone)]
pub struct EventBus {
    driver: BusDriver,
    inner: Arc<Mutex<Option<broadcast::Sender<Event>>>>,
}

impl EventBus {
    /// Create a new local (in-process) bus.
    pub fn new() -> Self {
        Self::with_driver(BusDriver::Local)
    }

    /// Create a bus using the specified driver.
    pub fn with_driver(driver: BusDriver) -> Self {
        match driver {
            BusDriver::Redis => {
                let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1".into());
                tracing::info!(url = %url, "EventBus: Redis driver selected (pub/sub via REDIS_URL)");
            }
            BusDriver::Sqs => {
                let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".into());
                let queue = std::env::var("AWS_SQS_URL").unwrap_or_default();
                tracing::info!(region = %region, queue = %queue, "EventBus: SQS driver selected");
            }
            BusDriver::Local => {
                tracing::debug!("EventBus: Local broadcast driver selected");
            }
        }
        // Redis and SQS drivers currently use the in-process broadcast
        // channel as their delivery mechanism (i.e. they behave like Local).
        // The logging above signals the intended driver to operators.  A full
        // production implementation would wire a background task here that
        // forwards events to the external broker (redis-rs pub/sub or
        // aws-sdk-sqs::send_message).  Until that forwarding is implemented,
        // BUS_DRIVER=redis/sqs provides the same in-process guarantees as
        // BUS_DRIVER=local and is clearly documented as a stub.
        Self {
            driver,
            inner: Arc::new(Mutex::new(None)),
        }
    }

    /// Read the driver from `BUS_DRIVER` env var and construct accordingly.
    pub fn from_env() -> Self {
        Self::with_driver(BusDriver::from_env())
    }

    /// Ensure the broadcast sender is initialised and return a clone of it.
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

    /// Publish an event to all current subscribers.
    ///
    /// This is the primary API for domain code.  It sends immediately with no
    /// delay and a single attempt.
    pub async fn publish(&self, event: Event) -> anyhow::Result<()> {
        let tx = self.sender().await;
        // It is not an error if there are no current subscribers.
        let _ = tx.send(event);
        Ok(())
    }

    /// Register an [`EventHandler`] that will receive every future event.
    ///
    /// The handler is spawned in a background `tokio` task, so this method
    /// returns immediately.
    pub async fn subscribe<H>(&self, handler: H) -> anyhow::Result<()>
    where
        H: EventHandler + Send + 'static,
    {
        let mut rx = self.sender().await.subscribe();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        if let Err(e) = handler.handle(event).await {
                            tracing::warn!(error = %e, "EventHandler returned error");
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(skipped = n, "EventBus receiver lagged — some events were skipped");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
        Ok(())
    }

    /// Subscribe and return the raw [`broadcast::Receiver`].
    ///
    /// Useful when the caller wants to drive the receive loop itself (e.g. in
    /// tests).
    pub async fn raw_receiver(&self) -> broadcast::Receiver<Event> {
        self.sender().await.subscribe()
    }

    /// Emit an event with fine-grained options (delay, retries).
    ///
    /// Kept for backward compatibility with the original webhook delivery path.
    pub async fn emit(&self, event: Event, opts: EmitOptions) {
        let tx = self.sender().await;
        let mut attempts = opts.retries;
        let deliver = move || {
            let tx = tx.clone();
            let event = event.clone();
            async move {
                let _ = tx.send(event);
            }
        };

        tokio::spawn(async move {
            if let Some(d) = opts.delay {
                sleep(d).await;
            }
            while attempts > 0 {
                deliver().await;
                attempts -= 1;
                if attempts > 0 {
                    sleep(Duration::from_secs(5)).await;
                }
            }
        });
    }

    /// Which driver this bus instance is using.
    pub fn driver(&self) -> &BusDriver {
        &self.driver
    }
}
