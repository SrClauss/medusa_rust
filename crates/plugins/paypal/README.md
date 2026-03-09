# Plugin PayPal para Medusa Rust

Plugin de pagamento para integração com [PayPal](https://developer.paypal.com/), implementando o fluxo da [PayPal Orders API v2](https://developer.paypal.com/api/orders/v2/). A lógica espelha o plugin oficial do MedusaJS adaptado para Rust.

## Instalação

```toml
[dependencies]
paypal_plugin = { path = "crates/plugins/paypal" }
```

## Uso Básico

```rust
use paypal_plugin::PayPalPlugin;
use plugin_api::PaymentProvider;
use std::collections::HashMap;

let mut plugin = PayPalPlugin::new();
let mut config = HashMap::new();
config.insert("client_id".to_string(), std::env::var("PAYPAL_CLIENT_ID").unwrap());
config.insert("client_secret".to_string(), std::env::var("PAYPAL_CLIENT_SECRET").unwrap());
// Opcional: usar sandbox (padrão) ou produção
config.insert("base_url".to_string(), "https://api-m.sandbox.paypal.com".to_string());
plugin.initialize(config).await?;
// initialize() obtém automaticamente o access token OAuth2
```

## Fluxo de Pagamento

```rust
// 1. Criar order (intent=CAPTURE)
let mut meta = HashMap::new();
meta.insert("order_id".to_string(), "order_abc".to_string());  // referência interna
let order = plugin.create_payment(50.0, "USD", meta).await?;
println!("Order ID: {}", order["id"]);
// → Redirecionar o cliente para aprovar o pagamento no PayPal

// 2. Após aprovação do cliente, capturar:
let captured = plugin.capture_payment(order["id"].as_str().unwrap(), None).await?;
let capture_id = &captured["purchase_units"][0]["payments"]["captures"][0]["id"];

// 3. Reembolsar (usar o capture ID, não o order ID):
let refunded = plugin.refund_payment(capture_id.as_str().unwrap(), None, None).await?;
```

## Gerenciamento de Webhooks

```rust
let webhook_id = plugin.create_webhook(
    "https://meusite.com/hooks/payment/paypal",
    vec![
        "PAYMENT.CAPTURE.COMPLETED".to_string(),
        "PAYMENT.CAPTURE.DENIED".to_string(),
        "PAYMENT.CAPTURE.REFUNDED".to_string(),
        "CHECKOUT.ORDER.APPROVED".to_string(),
    ],
).await?;

let webhooks = plugin.list_webhooks().await?;
for wh in &webhooks {
    println!("{}: {}", wh.id, wh.url);
}

plugin.delete_webhook(&webhook_id).await?;
```

## Configuração

| Variável de Ambiente | Descrição |
|---|---|
| `PAYPAL_CLIENT_ID` | Client ID da aplicação PayPal |
| `PAYPAL_CLIENT_SECRET` | Client Secret da aplicação PayPal |
| `PAYPAL_BASE_URL` | URL base da API (omitir para sandbox padrão) |

### URLs da API

| Ambiente | URL |
|----------|-----|
| **Sandbox** | `https://api-m.sandbox.paypal.com` |
| **Produção** | `https://api-m.paypal.com` |

## Fluxo OAuth2

Na chamada a `initialize()`, o plugin:

1. Envia `POST /v1/oauth2/token` com `grant_type=client_credentials` usando HTTP Basic Auth
2. Armazena o `access_token` em memória
3. Usa o token como `Authorization: Bearer` em todas as chamadas subsequentes

Para renovar o token (após expiração), chame `initialize()` novamente.

## Eventos de Webhook Suportados

| Evento PayPal | Ação Normalizada |
|---|---|
| `CHECKOUT.ORDER.APPROVED` | `Authorized` |
| `PAYMENT.CAPTURE.COMPLETED` | `Captured` |
| `PAYMENT.CAPTURE.DENIED` | `Failed` |
| `PAYMENT.CAPTURE.DECLINED` | `Failed` |
| `PAYMENT.CAPTURE.PENDING` | `Pending` |
| `PAYMENT.CAPTURE.REFUNDED` | `Refunded` |
| `PAYMENT.CAPTURE.REVERSED` | `Refunded` |
| `PAYMENT.ORDER.CANCELLED` | `Cancelled` |

## Nota sobre Cancelamento

A Orders API do PayPal não expõe um endpoint REST direto para cancelar orders não aprovadas.
O método `cancel_payment()` retorna uma resposta sintética `{"id": "...", "status": "VOIDED"}`.
Orders não capturadas expiram automaticamente após alguns dias.

## Executar Testes

Os testes injetam um access token pré-gerado para evitar o fluxo OAuth nos testes:

```bash
cargo test --package paypal_plugin
```

### Cobertura dos testes

| Teste | Endpoint mockado |
|-------|-----------------|
| `test_create_payment_success` | `POST /v2/checkout/orders` |
| `test_create_payment_negative_amount_fails` | validação local |
| `test_capture_payment_success` | `POST /v2/checkout/orders/{id}/capture` |
| `test_refund_payment_success` | `POST /v2/payments/captures/{id}/refund` |
| `test_cancel_payment_returns_voided` | resposta sintética local |
| `test_webhook_capture_completed` | parsing local |
| `test_webhook_capture_denied` | parsing local |
| `test_webhook_capture_refunded` | parsing local |
| `test_webhook_order_approved` | parsing local |
| `test_create_webhook` | `POST /v1/notifications/webhooks` |
| `test_list_webhooks` | `GET /v1/notifications/webhooks` |
| `test_delete_webhook` | `DELETE /v1/notifications/webhooks/{id}` |

## Referências

- [PayPal REST API](https://developer.paypal.com/api/rest/)
- [Orders API v2](https://developer.paypal.com/api/orders/v2/)
- [Captures API](https://developer.paypal.com/api/payments/v2/)
- [Webhooks API](https://developer.paypal.com/api/rest/webhooks/)
