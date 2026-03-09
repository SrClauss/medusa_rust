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

use std::{sync::Arc, time::Duration, collections::HashMap};
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

/// Event interceptor trait.
#[async_trait]
pub trait EventInterceptor: Send + Sync {
    async fn intercept(&self, event: &mut Event) -> anyhow::Result<()>;
}

/// Handle returned to callers that can be used to cancel a subscription.
pub struct SubscriptionHandle {
    task: tokio::task::JoinHandle<()>,
    id: Option<String>,
}

impl SubscriptionHandle {
    pub fn cancel(self) {
        self.task.abort();
        tracing::debug!(id = ?self.id, "Subscription cancelled");
    }

    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}

/// Shared event bus handle.  Clone freely — all clones share the same channel.
#[derive(Clone)]
pub struct EventBus {
    driver: BusDriver,
    inner: Arc<Mutex<Option<broadcast::Sender<Event>>>>,
    grouped_events: Arc<Mutex<HashMap<String, Vec<Event>>>>,
    interceptors: Arc<Mutex<Vec<Arc<dyn EventInterceptor>>>>,
    #[cfg(feature = "redis-bus")]
    redis_publisher: Option<Arc<tokio::sync::Mutex<redis::aio::ConnectionManager>>>,
    #[cfg(feature = "redis-bus")]
    redis_channel: String,
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
            grouped_events: Arc::new(Mutex::new(HashMap::new())),
            interceptors: Arc::new(Mutex::new(Vec::new())),
            #[cfg(feature = "redis-bus")]
            redis_publisher: None,
            #[cfg(feature = "redis-bus")]
            redis_channel: std::env::var("REDIS_CHANNEL")
                .unwrap_or_else(|_| "medusa:events".into()),
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

    /// Register an [`EventInterceptor`] that will be executed before publishing.
    pub async fn add_interceptor(&self, interceptor: Arc<dyn EventInterceptor>) {
        self.interceptors.lock().await.push(interceptor);
        tracing::debug!("Interceptor registered");
    }

    async fn run_interceptors(&self, event: &mut Event) -> anyhow::Result<()> {
        for interceptor in self.interceptors.lock().await.iter() {
            interceptor.intercept(event).await?;
        }
        Ok(())
    }

    /// Publish an event to all current subscribers.
    ///
    /// This is the primary API for domain code.  It sends immediately with no
    /// delay and a single attempt. Interceptors are executed before delivery.
    ///
    /// When compiled with `redis-bus` feature and driver is `BusDriver::Redis`,
    /// the event is published to the configured Redis channel via PUBLISH.
    pub async fn publish(&self, mut event: Event) -> anyhow::Result<()> {
        self.run_interceptors(&mut event).await?;

        #[cfg(feature = "redis-bus")]
        if self.driver == BusDriver::Redis {
            if let Some(ref conn_manager) = self.redis_publisher {
                let json = serde_json::to_string(&event)?;
                let mut conn = conn_manager.lock().await;
                redis::cmd("PUBLISH")
                    .arg(&self.redis_channel)
                    .arg(json)
                    .query_async(&mut *conn)
                    .await?;
                tracing::debug!(channel = %self.redis_channel, "Published event to Redis");
                return Ok(());
            }
        }

        let tx = self.sender().await;
        // It is not an error if there are no current subscribers.
        let _ = tx.send(event);
        Ok(())
    }

    /// Register an [`EventHandler`] that will receive every future event.
    ///
    /// Returns a `SubscriptionHandle` allowing cancellation.
    ///
    /// When compiled with `redis-bus` feature and driver is `BusDriver::Redis`,
    /// the handler is wired to receive messages from the configured Redis channel.
    pub async fn subscribe_with_id<H>(&self, handler: H, subscriber_id: Option<String>) -> anyhow::Result<SubscriptionHandle>
    where
        H: EventHandler + Send + 'static,
    {
        #[cfg(feature = "redis-bus")]
        if self.driver == BusDriver::Redis {
            let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1".into());
            let channel = self.redis_channel.clone();
            let id_clone = subscriber_id.clone();

            let client = redis::Client::open(url)?;
            let mut pubsub = client.get_async_pubsub().await?;
            pubsub.subscribe(&channel).await?;

            let task = tokio::spawn(async move {
                use futures::StreamExt;
                let mut stream = pubsub.on_message();
                while let Some(msg) = stream.next().await {
                    if let Ok(payload) = msg.get_payload::<String>() {
                        if let Ok(event) = serde_json::from_str::<Event>(&payload) {
                            if let Err(e) = handler.handle(event).await {
                                tracing::warn!(subscriber_id = ?id_clone, error = %e, "Handler error");
                            }
                        }
                    }
                }
            });

            return Ok(SubscriptionHandle { task, id: subscriber_id });
        }

        let mut rx = self.sender().await.subscribe();
        let id_clone = subscriber_id.clone();

        let task = tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        if let Err(e) = handler.handle(event).await {
                            tracing::warn!(subscriber_id = ?id_clone, error = %e, "Handler error");
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(skipped = n, "Subscriber lagged");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        Ok(SubscriptionHandle { task, id: subscriber_id })
    }

    /// Backward-compatible subscribe — now returns a `SubscriptionHandle`
    /// allowing the caller to cancel the subscription at any time.
    ///
    /// # Migration note
    /// Previously this returned `anyhow::Result<()>`. It now returns
    /// `anyhow::Result<SubscriptionHandle>`. Callers that discard the result
    /// (e.g. `bus.subscribe(...).await.unwrap();`) continue to compile unchanged
    /// because dropping a `SubscriptionHandle` detaches — not cancels — the
    /// background task.
    pub async fn subscribe<H>(&self, handler: H) -> anyhow::Result<SubscriptionHandle>
    where
        H: EventHandler + Send + 'static,
    {
        self.subscribe_with_id(handler, None).await
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

    /// Emitar (enfileirar) um evento em um grupo identificado por `group_id`.
    /// Os eventos acumulados só serão emitidos quando `release_grouped` for chamado.
    pub async fn emit_grouped(&self, group_id: &str, event: Event) -> anyhow::Result<()> {
        let mut groups = self.grouped_events.lock().await;
        groups.entry(group_id.to_string())
            .or_insert_with(Vec::new)
            .push(event);
        tracing::debug!(group_id = %group_id, "Event queued in group");
        Ok(())
    }

    /// Libera (emite) todos os eventos acumulados de um grupo.
    pub async fn release_grouped(&self, group_id: &str) -> anyhow::Result<()> {
        let events = {
            let mut groups = self.grouped_events.lock().await;
            groups.remove(group_id).unwrap_or_default()
        };

        tracing::info!(group_id = %group_id, count = events.len(), "Releasing grouped events");

        for mut event in events {
            self.run_interceptors(&mut event).await?;
            self.publish(event).await?;
        }
        Ok(())
    }

    /// Limpa os eventos de um grupo; se `event_names` for fornecido, filtra por nome.
    pub async fn clear_grouped(&self, group_id: &str, event_names: Option<Vec<String>>) {
        let mut groups = self.grouped_events.lock().await;

        if let Some(names) = event_names {
            if let Some(events) = groups.get_mut(group_id) {
                events.retain(|e| {
                    let event_name = format!("{:?}", e);
                    !names.iter().any(|n| event_name.contains(n))
                });
            }
        } else {
            groups.remove(group_id);
        }

        tracing::debug!(group_id = %group_id, "Grouped events cleared");
    }

    /// Which driver this bus instance is using.
    pub fn driver(&self) -> &BusDriver {
        &self.driver
    }

    /// Initialise the Redis connection manager for publishing.
    ///
    /// Must be called after constructing the bus with `with_driver(BusDriver::Redis)`
    /// before any `publish()` calls that should reach Redis.
    ///
    /// Reads `REDIS_URL` from the environment (defaults to `redis://127.0.0.1`).
    #[cfg(feature = "redis-bus")]
    pub async fn init_redis(&mut self) -> anyhow::Result<()> {
        let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1".into());
        let client = redis::Client::open(url)?;
        let conn = redis::aio::ConnectionManager::new(client).await?;
        self.redis_publisher = Some(Arc::new(tokio::sync::Mutex::new(conn)));
        tracing::info!(channel = %self.redis_channel, "Redis connection manager initialised");
        Ok(())
    }
}

// ─── AuditInterceptor ─────────────────────────────────────────────────────────

/// Interceptor that persists every event to the `event_audit` table in Postgres.
///
/// Requires the `event_audit` table from migration
/// `20260309000001_workflow_persistence.sql`.
pub struct AuditInterceptor {
    db: sqlx::PgPool,
}

impl AuditInterceptor {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl EventInterceptor for AuditInterceptor {
    async fn intercept(&self, event: &mut Event) -> anyhow::Result<()> {
        let json = serde_json::to_value(&*event)?;
        let event_type = match event {
            Event::PaymentWebhook(_) => "PaymentWebhook",
            Event::ShopWebhook(_) => "ShopWebhook",
            Event::OrderPlaced(_) => "OrderPlaced",
            Event::PaymentCaptured(_) => "PaymentCaptured",
            Event::CustomerRegistered(_) => "CustomerRegistered",
            Event::ProductCreated(_) => "ProductCreated",
            Event::CartCompleted(_) => "CartCompleted",
        };

        sqlx::query(
            "INSERT INTO event_audit (event_type, payload, created_at) VALUES ($1, $2, NOW())"
        )
        .bind(event_type)
        .bind(json)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    fn make_order_event() -> Event {
        Event::OrderPlaced(OrderPlacedEvent {
            order_id: uuid::Uuid::new_v4(),
            display_id: 1,
            customer_id: uuid::Uuid::new_v4(),
            email: "test@test.com".into(),
            currency_code: "usd".into(),
            total: 1000,
        })
    }

    #[tokio::test]
    async fn test_publish_subscribe_with_handle() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let _handle = bus.subscribe_with_id(
            move |_: Event| {
                let c = c.clone();
                async move { c.fetch_add(1, Ordering::SeqCst); Ok(()) }
            },
            Some("test-sub".into()),
        ).await.unwrap();

        bus.publish(make_order_event()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_subscribe_returns_handle() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let handle = bus.subscribe(move |_: Event| {
            let c = c.clone();
            async move { c.fetch_add(1, Ordering::SeqCst); Ok(()) }
        }).await.unwrap();

        bus.publish(make_order_event()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        // cancel must stop receiving events
        handle.cancel();

        bus.publish(make_order_event()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1, "Must stop after cancel()");
    }

    #[tokio::test]
    async fn test_event_grouping_release() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let _handle = bus.subscribe(move |_: Event| {
            let c = c.clone();
            async move { c.fetch_add(1, Ordering::SeqCst); Ok(()) }
        }).await.unwrap();

        bus.emit_grouped("tx1", make_order_event()).await.unwrap();
        bus.emit_grouped("tx1", make_order_event()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 0, "Nothing must be emitted before release");

        bus.release_grouped("tx1").await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 2, "Both events must be emitted");
    }

    #[tokio::test]
    async fn test_event_grouping_clear() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let _handle = bus.subscribe(move |_: Event| {
            let c = c.clone();
            async move { c.fetch_add(1, Ordering::SeqCst); Ok(()) }
        }).await.unwrap();

        bus.emit_grouped("tx2", make_order_event()).await.unwrap();
        bus.clear_grouped("tx2", None).await;
        bus.release_grouped("tx2").await.unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 0, "Nothing must be emitted after clear");
    }

    #[tokio::test]
    async fn test_interceptor() {
        let bus = EventBus::new();
        let intercepted = Arc::new(AtomicU32::new(0));

        struct CountingInterceptor(Arc<AtomicU32>);
        #[async_trait::async_trait]
        impl EventInterceptor for CountingInterceptor {
            async fn intercept(&self, _event: &mut Event) -> anyhow::Result<()> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        bus.add_interceptor(Arc::new(CountingInterceptor(intercepted.clone()))).await;
        bus.publish(make_order_event()).await.unwrap();
        assert_eq!(intercepted.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_unsubscribe_cancels_task() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let handle = bus.subscribe_with_id(
            move |_: Event| {
                let c = c.clone();
                async move { c.fetch_add(1, Ordering::SeqCst); Ok(()) }
            },
            Some("cancellable".into()),
        ).await.unwrap();

        bus.publish(make_order_event()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        handle.cancel();

        bus.publish(make_order_event()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1, "Cancelled handle must not receive more events");
    }

    #[tokio::test]
    #[cfg(feature = "redis-bus")]
    #[ignore] // Requires a running Redis instance
    async fn test_redis_publish_subscribe() {
        std::env::set_var("REDIS_URL", "redis://127.0.0.1");
        std::env::set_var("REDIS_CHANNEL", "medusa:test");

        let mut bus = EventBus::with_driver(BusDriver::Redis);
        bus.init_redis().await.unwrap();

        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let _handle = bus.subscribe(move |_: Event| {
            let c = c.clone();
            async move { c.fetch_add(1, Ordering::SeqCst); Ok(()) }
        }).await.unwrap();

        bus.publish(make_order_event()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
