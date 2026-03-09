use mercadopago_plugin::MercadoPagoPlugin;
use mockito::Server;
use plugin_api::{PaymentAction, PaymentProvider, WebhookPayload};
use std::collections::HashMap;

async fn plugin_with_server(server: &Server) -> MercadoPagoPlugin {
    let mut plugin = MercadoPagoPlugin::new();
    let mut config = HashMap::new();
    config.insert("access_token".to_string(), "test_token".to_string());
    config.insert("base_url".to_string(), server.url());
    plugin.initialize(config).await.unwrap();
    plugin
}

#[tokio::test]
async fn test_create_payment_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/payments")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":123456,"status":"pending","status_detail":"pending_waiting_payment","transaction_amount":50.0,"currency_id":"BRL"}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let mut meta = HashMap::new();
    meta.insert("payer_email".to_string(), "test@example.com".to_string());
    meta.insert("payment_method_id".to_string(), "pix".to_string());

    let result = plugin.create_payment(50.0, "BRL", meta).await.unwrap();
    assert_eq!(result["id"], 123456);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_payment_negative_amount_fails() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;
    let mut meta = HashMap::new();
    meta.insert("payer_email".to_string(), "test@example.com".to_string());

    let result = plugin.create_payment(-10.0, "BRL", meta).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_capture_payment_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/payments/123456")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":123456,"status":"approved","capture":true}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let result = plugin.capture_payment("123456", None).await.unwrap();
    assert_eq!(result["status"], "approved");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_refund_payment_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/payments/123456/refunds")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":789,"status":"approved","payment_id":123456}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let result = plugin.refund_payment("123456", None, None).await.unwrap();
    assert_eq!(result["status"], "approved");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_cancel_payment_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("PUT", "/v1/payments/123456")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":123456,"status":"cancelled"}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let result = plugin.cancel_payment("123456").await.unwrap();
    assert_eq!(result["status"], "cancelled");
    mock.assert_async().await;
}

fn make_payload(raw: serde_json::Value) -> WebhookPayload {
    WebhookPayload {
        provider: "mercadopago".to_string(),
        provider_id: "123456".to_string(),
        order_id: None,
        amount: None,
        currency: None,
        raw_data: raw,
    }
}

#[tokio::test]
async fn test_webhook_payment_approved() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;

    let payload = make_payload(serde_json::json!({
        "action": "payment.updated",
        "data": {"id": "123456", "status": "approved"}
    }));

    let result = plugin.get_webhook_action_and_data(payload).await.unwrap();
    assert_eq!(result.action, PaymentAction::Captured);
}

#[tokio::test]
async fn test_webhook_payment_rejected() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;

    let payload = make_payload(serde_json::json!({
        "action": "payment.updated",
        "data": {"id": "123456", "status": "rejected"}
    }));

    let result = plugin.get_webhook_action_and_data(payload).await.unwrap();
    assert_eq!(result.action, PaymentAction::Failed);
}

#[tokio::test]
async fn test_webhook_payment_created() {
    let server = Server::new_async().await;
    let plugin = plugin_with_server(&server).await;

    let payload = make_payload(serde_json::json!({
        "action": "payment.created",
        "data": {"id": "123456"}
    }));

    let result = plugin.get_webhook_action_and_data(payload).await.unwrap();
    assert_eq!(result.action, PaymentAction::Authorized);
}

#[tokio::test]
async fn test_create_webhook() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/webhooks")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"wh_mp_1","url":"https://example.com/hook"}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let id = plugin
        .create_webhook(
            "https://example.com/hook",
            vec!["payment.updated".to_string()],
        )
        .await
        .unwrap();
    assert_eq!(id, "wh_mp_1");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_webhooks() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/webhooks")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"results":[{"id":"wh_mp_1","url":"https://example.com/hook","events":[{"name":"payment.updated"}],"active":true}]}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let list = plugin.list_webhooks().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, "wh_mp_1");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_delete_webhook() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("DELETE", "/v1/webhooks/wh_mp_1")
        .with_status(204)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    plugin.delete_webhook("wh_mp_1").await.unwrap();
    mock.assert_async().await;
}
