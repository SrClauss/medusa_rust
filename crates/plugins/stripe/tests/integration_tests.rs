use mockito::Server;
use plugin_api::{PaymentAction, PaymentProvider, WebhookPayload};
use std::collections::HashMap;
use stripe_plugin::StripePlugin;

async fn plugin_with_server(server: &Server) -> StripePlugin {
    let mut plugin = StripePlugin::new();
    let mut config = HashMap::new();
    config.insert("secret_key".to_string(), "sk_test_key".to_string());
    config.insert("webhook_secret".to_string(), "whsec_test".to_string());
    config.insert("base_url".to_string(), server.url());
    plugin.initialize(config).await.unwrap();
    plugin
}

// ── create_payment ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_payment_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/payment_intents")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"pi_123","status":"requires_payment_method","amount":10000,"currency":"usd"}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let result = plugin
        .create_payment(100.0, "usd", HashMap::new())
        .await
        .unwrap();
    assert_eq!(result["id"], "pi_123");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_payment_negative_amount_fails() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;
    let result = plugin.create_payment(-5.0, "usd", HashMap::new()).await;
    assert!(result.is_err());
}

// ── capture_payment ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_capture_payment_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/payment_intents/pi_123/capture")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"pi_123","status":"succeeded","amount":10000}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let result = plugin.capture_payment("pi_123", None).await.unwrap();
    assert_eq!(result["status"], "succeeded");
    mock.assert_async().await;
}

// ── refund_payment ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_refund_payment_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/refunds")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"re_123","status":"succeeded","amount":10000}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let result = plugin.refund_payment("pi_123", None, None).await.unwrap();
    assert_eq!(result["status"], "succeeded");
    mock.assert_async().await;
}

// ── cancel_payment ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_cancel_payment_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/payment_intents/pi_123/cancel")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"pi_123","status":"canceled"}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let result = plugin.cancel_payment("pi_123").await.unwrap();
    assert_eq!(result["status"], "canceled");
    mock.assert_async().await;
}

// ── webhook parsing ───────────────────────────────────────────────────────────

fn make_payload(event_json: serde_json::Value) -> WebhookPayload {
    WebhookPayload {
        provider: "stripe".to_string(),
        provider_id: "pi_123".to_string(),
        order_id: None,
        amount: None,
        currency: None,
        raw_data: event_json,
    }
}

#[tokio::test]
async fn test_webhook_payment_intent_succeeded() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;

    let payload = make_payload(serde_json::json!({
        "id": "evt_123",
        "type": "payment_intent.succeeded",
        "data": {"object": {"id": "pi_123", "amount": 10000}}
    }));

    let result = plugin.get_webhook_action_and_data(payload).await.unwrap();
    assert_eq!(result.action, PaymentAction::Captured);
}

#[tokio::test]
async fn test_webhook_payment_intent_failed() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;

    let payload = make_payload(serde_json::json!({
        "id": "evt_124",
        "type": "payment_intent.payment_failed",
        "data": {"object": {"id": "pi_123"}}
    }));

    let result = plugin.get_webhook_action_and_data(payload).await.unwrap();
    assert_eq!(result.action, PaymentAction::Failed);
}

#[tokio::test]
async fn test_webhook_charge_refunded() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;

    let payload = make_payload(serde_json::json!({
        "id": "evt_125",
        "type": "charge.refunded",
        "data": {"object": {"id": "ch_123"}}
    }));

    let result = plugin.get_webhook_action_and_data(payload).await.unwrap();
    assert_eq!(result.action, PaymentAction::Refunded);
}

#[tokio::test]
async fn test_webhook_payment_intent_authorized() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;

    let payload = make_payload(serde_json::json!({
        "id": "evt_126",
        "type": "payment_intent.amount_capturable_updated",
        "data": {"object": {"id": "pi_123"}}
    }));

    let result = plugin.get_webhook_action_and_data(payload).await.unwrap();
    assert_eq!(result.action, PaymentAction::Authorized);
}

// ── webhook management ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_webhook() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/webhook_endpoints")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"we_123","url":"https://example.com/hook","status":"enabled","enabled_events":["payment_intent.succeeded"]}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let id = plugin
        .create_webhook(
            "https://example.com/hook",
            vec!["payment_intent.succeeded".to_string()],
        )
        .await
        .unwrap();
    assert_eq!(id, "we_123");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_webhooks() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/webhook_endpoints")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"data":[{"id":"we_123","url":"https://example.com/hook","status":"enabled","enabled_events":["payment_intent.succeeded"]}]}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let list = plugin.list_webhooks().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, "we_123");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_delete_webhook() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("DELETE", "/v1/webhook_endpoints/we_123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"we_123","deleted":true}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    plugin.delete_webhook("we_123").await.unwrap();
    mock.assert_async().await;
}
