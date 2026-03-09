# Plugin PayPal para Medusa Rust

Plugin de pagamento para integração com [PayPal](https://developer.paypal.com/).

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
// Opcional: usar sandbox
config.insert("base_url".to_string(), "https://api-m.sandbox.paypal.com".to_string());
plugin.initialize(config).await?;

// Criar order (o cliente deve aprovar no PayPal antes da captura)
let order = plugin.create_payment(50.0, "USD", HashMap::new()).await?;
println!("Order ID: {}", order["id"]);

// Após aprovação do cliente, capturar
let capture = plugin.capture_payment(order["id"].as_str().unwrap(), None).await?;
```

## Gerenciamento de Webhooks

```rust
let webhook_id = plugin.create_webhook(
    "https://meusite.com/hooks/payment/paypal",
    vec![
        "PAYMENT.CAPTURE.COMPLETED".to_string(),
        "PAYMENT.CAPTURE.DENIED".to_string(),
        "PAYMENT.CAPTURE.REFUNDED".to_string(),
    ],
).await?;

let webhooks = plugin.list_webhooks().await?;
plugin.delete_webhook(&webhook_id).await?;
```

## Configuração

| Variável de Ambiente | Descrição |
|---|---|
| `PAYPAL_CLIENT_ID` | Client ID da aplicação PayPal |
| `PAYPAL_CLIENT_SECRET` | Client Secret da aplicação PayPal |

### URLs

- **Sandbox**: `https://api-m.sandbox.paypal.com`
- **Produção**: `https://api-m.paypal.com`

## Eventos de Webhook Suportados

| Evento | Ação Resultante |
|---|---|
| `CHECKOUT.ORDER.APPROVED` | `Authorized` |
| `PAYMENT.CAPTURE.COMPLETED` | `Captured` |
| `PAYMENT.CAPTURE.DENIED` | `Failed` |
| `PAYMENT.CAPTURE.DECLINED` | `Failed` |
| `PAYMENT.CAPTURE.PENDING` | `Pending` |
| `PAYMENT.CAPTURE.REFUNDED` | `Refunded` |
| `PAYMENT.CAPTURE.REVERSED` | `Refunded` |
| `PAYMENT.ORDER.CANCELLED` | `Cancelled` |

## Fluxo OAuth2

O plugin obtém automaticamente um access token OAuth2 durante `initialize()`.
O token é armazenado em memória e reutilizado para todas as chamadas subsequentes.
Para renovar o token, chame `initialize()` novamente.

## Executar Testes

```bash
cargo test --package paypal_plugin
```
