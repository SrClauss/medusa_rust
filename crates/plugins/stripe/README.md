# Plugin Stripe para Medusa Rust

Plugin de pagamento para integração com [Stripe](https://stripe.com/).

## Instalação

```toml
[dependencies]
stripe_plugin = { path = "crates/plugins/stripe" }
```

## Uso Básico

```rust
use stripe_plugin::StripePlugin;
use plugin_api::PaymentProvider;
use std::collections::HashMap;

let mut plugin = StripePlugin::new();
let mut config = HashMap::new();
config.insert("secret_key".to_string(), std::env::var("STRIPE_SECRET_KEY").unwrap());
config.insert("webhook_secret".to_string(), std::env::var("STRIPE_WEBHOOK_SECRET").unwrap());
plugin.initialize(config).await?;

// Criar PaymentIntent (captura manual, como no MedusaJS)
let payment = plugin.create_payment(100.0, "usd", HashMap::new()).await?;
println!("PaymentIntent ID: {}", payment["id"]);

// Capturar
let captured = plugin.capture_payment(payment["id"].as_str().unwrap(), None).await?;
```

## Gerenciamento de Webhooks

```rust
let webhook_id = plugin.create_webhook(
    "https://meusite.com/hooks/payment/stripe",
    vec![
        "payment_intent.succeeded".to_string(),
        "payment_intent.payment_failed".to_string(),
        "charge.refunded".to_string(),
    ],
).await?;

let webhooks = plugin.list_webhooks().await?;
plugin.delete_webhook(&webhook_id).await?;
```

## Configuração

| Variável de Ambiente | Descrição |
|---|---|
| `STRIPE_SECRET_KEY` | Chave secreta da API (sk_live_... ou sk_test_...) |
| `STRIPE_WEBHOOK_SECRET` | Segredo para validação de webhooks (whsec_...) |

## Eventos de Webhook Suportados

| Evento | Ação Resultante |
|---|---|
| `payment_intent.amount_capturable_updated` | `Authorized` |
| `payment_intent.succeeded` | `Captured` |
| `payment_intent.payment_failed` | `Failed` |
| `payment_intent.canceled` | `Cancelled` |
| `payment_intent.processing` | `Pending` |
| `charge.refunded` | `Refunded` |
| `charge.refund.updated` | `Refunded` |

## Verificação de Assinatura

```rust
// Verificar assinatura do webhook antes de processar
plugin.verify_signature(
    raw_body.as_bytes(),
    request.headers()["stripe-signature"].to_str().unwrap(),
)?;
```

## Executar Testes

```bash
cargo test --package stripe_plugin
```
