# Plugin Mercado Pago para Medusa Rust

Plugin de pagamento para integração com [Mercado Pago](https://www.mercadopago.com/).

## Instalação

```toml
[dependencies]
mercadopago_plugin = { path = "crates/plugins/mercadopago" }
```

## Uso Básico

```rust
use mercadopago_plugin::MercadoPagoPlugin;
use plugin_api::PaymentProvider;
use std::collections::HashMap;

let mut plugin = MercadoPagoPlugin::new();
let mut config = HashMap::new();
config.insert("access_token".to_string(), std::env::var("MP_ACCESS_TOKEN").unwrap());
plugin.initialize(config).await?;

let mut meta = HashMap::new();
meta.insert("payer_email".to_string(), "user@example.com".to_string());
meta.insert("payment_method_id".to_string(), "pix".to_string());

let payment = plugin.create_payment(50.0, "BRL", meta).await?;
println!("Payment ID: {}", payment["id"]);
```

## Gerenciamento de Webhooks

```rust
let webhook_id = plugin.create_webhook(
    "https://meusite.com/hooks/payment/mercadopago",
    vec!["payment.created".to_string(), "payment.updated".to_string()],
).await?;

let webhooks = plugin.list_webhooks().await?;
plugin.delete_webhook(&webhook_id).await?;
```

## Configuração

| Variável de Ambiente | Descrição |
|---|---|
| `MP_ACCESS_TOKEN` | Token de acesso do Mercado Pago |

## Eventos de Webhook Suportados

| Evento | Ação Resultante |
|---|---|
| `payment.created` | `Authorized` |
| `payment.updated` (approved) | `Captured` |
| `payment.updated` (cancelled) | `Cancelled` |
| `payment.updated` (refunded) | `Refunded` |
| `payment.updated` (pending) | `Pending` |
| `payment.updated` (rejected) | `Failed` |

## Executar Testes

```bash
cargo test --package mercadopago_plugin
```
