use mockito::Server;
use paypal_plugin::PayPalPlugin;
use plugin_api::{PaymentAction, PaymentProvider, WebhookPayload};
use std::collections::HashMap;

/// Build a PayPalPlugin configured against the mock server URL.
/// We inject a pre-minted access token so that OAuth is skipped.
async fn plugin_with_server(server: &Server) -> PayPalPlugin {
    let mut config = HashMap::new();
    config.insert("client_id".to_string(), "test_client_id".to_string());
    config.insert("client_secret".to_string(), "test_secret".to_string());
    config.insert("access_token".to_string(), "test_token".to_string());
    config.insert("base_url".to_string(), server.url());

    let mut plugin = PayPalPlugin::new();
    plugin.initialize(config).await.unwrap();
    plugin
}

// ── create_payment ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_payment_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v2/checkout/orders")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"ORDER-123","status":"CREATED"}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let result = plugin
        .create_payment(50.0, "USD", HashMap::new())
        .await
        .unwrap();
    assert_eq!(result["id"], "ORDER-123");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_payment_negative_amount_fails() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;
    let result = plugin.create_payment(-10.0, "USD", HashMap::new()).await;
    assert!(result.is_err());
}

// ── capture_payment ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_capture_payment_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v2/checkout/orders/ORDER-123/capture")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"ORDER-123","status":"COMPLETED","purchase_units":[{"payments":{"captures":[{"id":"CAP-1","status":"COMPLETED"}]}}]}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let result = plugin.capture_payment("ORDER-123", None).await.unwrap();
    assert_eq!(result["status"], "COMPLETED");
    mock.assert_async().await;
}

// ── refund_payment ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_refund_payment_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v2/payments/captures/CAP-1/refund")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"REF-1","status":"COMPLETED"}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let result = plugin.refund_payment("CAP-1", None, None).await.unwrap();
    assert_eq!(result["status"], "COMPLETED");
    mock.assert_async().await;
}

// ── cancel_payment ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_cancel_payment_returns_voided() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;
    let result = plugin.cancel_payment("ORDER-123").await.unwrap();
    assert_eq!(result["status"], "VOIDED");
}

// ── webhook parsing ───────────────────────────────────────────────────────────

fn make_payload(raw: serde_json::Value) -> WebhookPayload {
    WebhookPayload {
        provider: "paypal".to_string(),
        provider_id: "ORDER-123".to_string(),
        order_id: None,
        amount: None,
        currency: None,
        raw_data: raw,
    }
}

#[tokio::test]
async fn test_webhook_capture_completed() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;

    let payload = make_payload(serde_json::json!({
        "id": "WH-123",
        "event_type": "PAYMENT.CAPTURE.COMPLETED",
        "resource": {"id": "CAP-1", "amount": {"value": "50.00", "currency_code": "USD"}}
    }));

    let result = plugin.get_webhook_action_and_data(payload).await.unwrap();
    assert_eq!(result.action, PaymentAction::Captured);
}

#[tokio::test]
async fn test_webhook_capture_denied() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;

    let payload = make_payload(serde_json::json!({
        "id": "WH-124",
        "event_type": "PAYMENT.CAPTURE.DENIED",
        "resource": {"id": "CAP-1"}
    }));

    let result = plugin.get_webhook_action_and_data(payload).await.unwrap();
    assert_eq!(result.action, PaymentAction::Failed);
}

#[tokio::test]
async fn test_webhook_capture_refunded() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;

    let payload = make_payload(serde_json::json!({
        "id": "WH-125",
        "event_type": "PAYMENT.CAPTURE.REFUNDED",
        "resource": {"id": "CAP-1"}
    }));

    let result = plugin.get_webhook_action_and_data(payload).await.unwrap();
    assert_eq!(result.action, PaymentAction::Refunded);
}

#[tokio::test]
async fn test_webhook_order_approved() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;

    let payload = make_payload(serde_json::json!({
        "id": "WH-126",
        "event_type": "CHECKOUT.ORDER.APPROVED",
        "resource": {"id": "ORDER-123"}
    }));

    let result = plugin.get_webhook_action_and_data(payload).await.unwrap();
    assert_eq!(result.action, PaymentAction::Authorized);
}

// ── webhook management ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_webhook() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/notifications/webhooks")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"WH-ENDPOINT-1","url":"https://example.com/hook"}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let id = plugin
        .create_webhook(
            "https://example.com/hook",
            vec!["PAYMENT.CAPTURE.COMPLETED".to_string()],
        )
        .await
        .unwrap();
    assert_eq!(id, "WH-ENDPOINT-1");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_webhooks() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/notifications/webhooks")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"webhooks":[{"id":"WH-ENDPOINT-1","url":"https://example.com/hook","event_types":[{"name":"PAYMENT.CAPTURE.COMPLETED"}]}]}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let list = plugin.list_webhooks().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, "WH-ENDPOINT-1");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_delete_webhook() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("DELETE", "/v1/notifications/webhooks/WH-ENDPOINT-1")
        .with_status(204)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    plugin.delete_webhook("WH-ENDPOINT-1").await.unwrap();
    mock.assert_async().await;
}
