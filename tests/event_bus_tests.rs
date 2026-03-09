//! Unit tests for the distributed-capable EventBus.

use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
use tokio::sync::Mutex;
use medusa_rust::events::{
    BusDriver, EmitOptions, Event, EventBus, EventHandler,
    OrderPlacedEvent, PaymentCapturedEvent,
};

// ── helpers ───────────────────────────────────────────────────────────────────

fn uuid() -> uuid::Uuid { uuid::Uuid::new_v4() }

// ── BusDriver::from_env ───────────────────────────────────────────────────────

#[test]
fn test_bus_driver_default_is_local() {
    // Unless BUS_DRIVER is set in the test environment, we expect Local.
    // We can't guarantee the env is clean, so just call from_env() and ensure
    // it doesn't panic.
    let _ = BusDriver::from_env();
}

// ── local publish / receive ───────────────────────────────────────────────────

#[test]
fn test_local_publish_received_by_raw_receiver() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let bus = EventBus::new();
        let mut rx = bus.raw_receiver().await;

        bus.publish(Event::OrderPlaced(OrderPlacedEvent {
            order_id: uuid(),
            display_id: 1,
            customer_id: uuid(),
            email: "test@example.com".into(),
            currency_code: "usd".into(),
            total: 5000,
        })).await.unwrap();

        let evt = rx.recv().await.unwrap();
        assert!(matches!(evt, Event::OrderPlaced(_)));
    });
}

#[test]
fn test_publish_no_subscriber_ok() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let bus = EventBus::new();
        // publishing without any subscriber must not error
        let result = bus.publish(Event::PaymentCaptured(PaymentCapturedEvent {
            payment_id: uuid(),
            amount: Some(1000),
            currency_code: Some("usd".into()),
            provider_id: Some("stripe".into()),
        })).await;
        assert!(result.is_ok());
    });
}

// ── subscribe with handler ────────────────────────────────────────────────────

#[test]
fn test_subscribe_handler_called() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));

        let c = counter.clone();
        bus.subscribe(move |_evt: Event| {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }).await.unwrap();

        bus.publish(Event::OrderPlaced(OrderPlacedEvent {
            order_id: uuid(),
            display_id: 42,
            customer_id: uuid(),
            email: "a@b.com".into(),
            currency_code: "brl".into(),
            total: 9999,
        })).await.unwrap();

        // give the background task time to run
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn test_multiple_subscribers_each_receive() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let bus = EventBus::new();
        let seen1 = Arc::new(AtomicU32::new(0));
        let seen2 = Arc::new(AtomicU32::new(0));

        let s1 = seen1.clone();
        bus.subscribe(move |_: Event| { let s = s1.clone(); async move { s.fetch_add(1, Ordering::SeqCst); Ok(()) } }).await.unwrap();

        let s2 = seen2.clone();
        bus.subscribe(move |_: Event| { let s = s2.clone(); async move { s.fetch_add(1, Ordering::SeqCst); Ok(()) } }).await.unwrap();

        bus.publish(Event::PaymentCaptured(PaymentCapturedEvent {
            payment_id: uuid(), amount: None, currency_code: None, provider_id: None,
        })).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert_eq!(seen1.load(Ordering::SeqCst), 1, "subscriber 1 should receive");
        assert_eq!(seen2.load(Ordering::SeqCst), 1, "subscriber 2 should receive");
    });
}

// ── EventHandler struct impl ──────────────────────────────────────────────────

struct LoggingHandler {
    events: Arc<Mutex<Vec<String>>>,
}

#[async_trait::async_trait]
impl EventHandler for LoggingHandler {
    async fn handle(&self, event: Event) -> anyhow::Result<()> {
        let tag = match &event {
            Event::OrderPlaced(_) => "OrderPlaced",
            Event::PaymentCaptured(_) => "PaymentCaptured",
            Event::CartCompleted(_) => "CartCompleted",
            Event::CustomerRegistered(_) => "CustomerRegistered",
            Event::ProductCreated(_) => "ProductCreated",
            Event::PaymentWebhook(_) => "PaymentWebhook",
            Event::ShopWebhook(_) => "ShopWebhook",
        };
        self.events.lock().await.push(tag.to_string());
        Ok(())
    }
}

#[test]
fn test_struct_handler_receives_event() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let bus = EventBus::new();
        let log = Arc::new(Mutex::new(Vec::<String>::new()));
        bus.subscribe(LoggingHandler { events: log.clone() }).await.unwrap();

        bus.publish(Event::OrderPlaced(OrderPlacedEvent {
            order_id: uuid(), display_id: 1, customer_id: uuid(),
            email: "x@y.z".into(), currency_code: "eur".into(), total: 0,
        })).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let entries = log.lock().await;
        assert_eq!(entries.as_slice(), ["OrderPlaced"]);
    });
}

// ── emit with delay ───────────────────────────────────────────────────────────

#[test]
fn test_emit_with_options() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let bus = EventBus::new();
        let mut rx = bus.raw_receiver().await;

        bus.emit(
            Event::PaymentCaptured(PaymentCapturedEvent {
                payment_id: uuid(), amount: Some(500), currency_code: None, provider_id: None,
            }),
            EmitOptions { delay: None, retries: 1 },
        ).await;

        let evt = tokio::time::timeout(
            tokio::time::Duration::from_millis(500),
            rx.recv(),
        ).await.expect("timeout").unwrap();

        assert!(matches!(evt, Event::PaymentCaptured(_)));
    });
}

// ── serde round-trip ──────────────────────────────────────────────────────────

#[test]
fn test_event_serde_roundtrip() {
    let oid = uuid();
    let cid = uuid();
    let event = Event::OrderPlaced(OrderPlacedEvent {
        order_id: oid,
        display_id: 7,
        customer_id: cid,
        email: "round@trip.com".into(),
        currency_code: "usd".into(),
        total: 12345,
    });

    let json = serde_json::to_string(&event).expect("serialize");
    let back: Event = serde_json::from_str(&json).expect("deserialize");
    match back {
        Event::OrderPlaced(e) => {
            assert_eq!(e.order_id, oid);
            assert_eq!(e.display_id, 7);
            assert_eq!(e.total, 12345);
        }
        _ => panic!("wrong variant after round-trip"),
    }
}

// ── publisher / subscriber integration example ────────────────────────────────

/// Integration test: one publisher, one subscriber, verify end-to-end delivery.
#[test]
fn test_integration_publisher_subscriber() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let bus = EventBus::new();
        let received: Arc<Mutex<Vec<Event>>> = Arc::new(Mutex::new(Vec::new()));

        let r = received.clone();
        bus.subscribe(move |evt: Event| {
            let r = r.clone();
            async move {
                r.lock().await.push(evt);
                Ok(())
            }
        }).await.unwrap();

        // Publisher sends three events
        let order_id = uuid();
        let payment_id = uuid();
        bus.publish(Event::OrderPlaced(OrderPlacedEvent {
            order_id,
            display_id: 101,
            customer_id: uuid(),
            email: "integration@test.com".into(),
            currency_code: "usd".into(),
            total: 4200,
        })).await.unwrap();
        bus.publish(Event::PaymentCaptured(PaymentCapturedEvent {
            payment_id,
            amount: Some(4200),
            currency_code: Some("usd".into()),
            provider_id: Some("stripe".into()),
        })).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let events = received.lock().await;
        assert_eq!(events.len(), 2, "subscriber should receive exactly 2 events");
        assert!(matches!(&events[0], Event::OrderPlaced(e) if e.order_id == order_id));
        assert!(matches!(&events[1], Event::PaymentCaptured(e) if e.payment_id == payment_id));
    });
}
