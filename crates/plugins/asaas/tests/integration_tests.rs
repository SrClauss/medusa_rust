use asaas_plugin::AsaasPlugin;
use mockito::Server;
use plugin_api::{PaymentAction, PaymentProvider, WebhookPayload};
use std::collections::HashMap;

// ── helpers ──────────────────────────────────────────────────────────────────

async fn plugin_with_server(server: &Server) -> AsaasPlugin {
    let mut plugin = AsaasPlugin::new();
    let mut config = HashMap::new();
    config.insert("api_key".to_string(), "test_key".to_string());
    config.insert("base_url".to_string(), server.url());
    plugin.initialize(config).await.unwrap();
    plugin
}

// ── create_payment ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_payment_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/payments")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"pay_123","status":"PENDING","value":100.0,"billingType":"PIX","dueDate":"2025-12-31"}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let mut meta = HashMap::new();
    meta.insert("customer".to_string(), "cus_abc".to_string());
    meta.insert("billing_type".to_string(), "PIX".to_string());
    meta.insert("due_date".to_string(), "2025-12-31".to_string());

    let result = plugin.create_payment(100.0, "BRL", meta).await.unwrap();
    assert_eq!(result["id"], "pay_123");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_payment_negative_amount_fails() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;
    let mut meta = HashMap::new();
    meta.insert("customer".to_string(), "cus_abc".to_string());
    meta.insert("due_date".to_string(), "2025-12-31".to_string());

    let result = plugin.create_payment(-50.0, "BRL", meta).await;
    assert!(result.is_err());
}

// ── capture_payment ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_capture_payment_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/payments/pay_123/receiveInCash")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"pay_123","status":"RECEIVED","value":100.0}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let result = plugin.capture_payment("pay_123", None).await.unwrap();
    assert_eq!(result["status"], "RECEIVED");
    mock.assert_async().await;
}

// ── refund_payment ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_refund_payment_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/payments/pay_123/refund")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"pay_123","status":"REFUNDED","value":100.0}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let result = plugin.refund_payment("pay_123", None, None).await.unwrap();
    assert_eq!(result["status"], "REFUNDED");
    mock.assert_async().await;
}

// ── cancel_payment ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_cancel_payment_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("DELETE", "/payments/pay_123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"deleted":true}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let result = plugin.cancel_payment("pay_123").await.unwrap();
    assert_eq!(result["deleted"], true);
    mock.assert_async().await;
}

// ── webhook parsing ───────────────────────────────────────────────────────────

fn make_payload(event_json: serde_json::Value) -> WebhookPayload {
    WebhookPayload {
        provider: "asaas".to_string(),
        provider_id: "pay_123".to_string(),
        order_id: None,
        amount: None,
        currency: None,
        raw_data: event_json,
    }
}

#[tokio::test]
async fn test_webhook_payment_received() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;

    let payload = make_payload(serde_json::json!({
        "event": "PAYMENT_RECEIVED",
        "payment": {"id": "pay_123", "status": "RECEIVED", "value": 100.0}
    }));

    let result = plugin.get_webhook_action_and_data(payload).await.unwrap();
    assert_eq!(result.action, PaymentAction::Captured);
}

#[tokio::test]
async fn test_webhook_payment_failed() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;

    let payload = make_payload(serde_json::json!({
        "event": "PAYMENT_OVERDUE",
        "payment": {"id": "pay_123", "status": "OVERDUE", "value": 100.0}
    }));

    let result = plugin.get_webhook_action_and_data(payload).await.unwrap();
    assert_eq!(result.action, PaymentAction::Pending);
}

#[tokio::test]
async fn test_webhook_payment_refunded() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;

    let payload = make_payload(serde_json::json!({
        "event": "PAYMENT_REFUNDED",
        "payment": {"id": "pay_123", "status": "REFUNDED", "value": 100.0}
    }));

    let result = plugin.get_webhook_action_and_data(payload).await.unwrap();
    assert_eq!(result.action, PaymentAction::Refunded);
}

// ── webhook management ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_webhook() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/webhooks")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"wh_abc","url":"https://example.com/hook","enabled":true}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let id = plugin
        .create_webhook(
            "https://example.com/hook",
            vec!["PAYMENT_RECEIVED".to_string()],
        )
        .await
        .unwrap();
    assert_eq!(id, "wh_abc");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_webhooks() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/webhooks")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"data":[{"id":"wh_abc","url":"https://example.com/hook","email":"","enabled":true,"events":["PAYMENT_RECEIVED"]}]}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let list = plugin.list_webhooks().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, "wh_abc");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_delete_webhook() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("DELETE", "/webhooks/wh_abc")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"deleted":true}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    plugin.delete_webhook("wh_abc").await.unwrap();
    mock.assert_async().await;
}
