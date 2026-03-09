# Plugin Stripe para Medusa Rust

Plugin de pagamento para integração com [Stripe](https://stripe.com/), o principal gateway de pagamentos internacional. Implementa a lógica do plugin oficial do MedusaJS adaptada para Rust.

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

// Criar PaymentIntent com capture_method=manual (igual ao plugin do MedusaJS)
let payment = plugin.create_payment(100.0, "usd", HashMap::new()).await?;
println!("PaymentIntent ID: {}", payment["id"]);
println!("Client Secret: {}", payment["client_secret"]);
```

## Fluxo Completo

```rust
let pi_id = payment["id"].as_str().unwrap();

// Depois da autorização do cartão pelo cliente, capturar:
let captured = plugin.capture_payment(pi_id, None).await?;
assert_eq!(captured["status"], "succeeded");

// Reembolsar via Refunds API:
let refunded = plugin.refund_payment(pi_id, Some(50.0), None).await?;

// Cancelar um PaymentIntent não capturado:
let cancelled = plugin.cancel_payment(pi_id).await?;
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
for wh in &webhooks {
    println!("{}: {} ({})", wh.id, wh.url, if wh.active { "ativo" } else { "inativo" });
}

plugin.delete_webhook(&webhook_id).await?;
```

## Verificação de Assinatura

O Stripe envia um header `Stripe-Signature` em cada webhook. Antes de processar,
valide a assinatura com HMAC-SHA256:

```rust
// No seu handler de webhook:
plugin.verify_signature(
    raw_body.as_bytes(),
    request.headers()["stripe-signature"].to_str().unwrap(),
)?;
// Se retornar Ok(()), a assinatura é válida.
```

## Configuração

| Variável de Ambiente | Descrição |
|---|---|
| `STRIPE_SECRET_KEY` | Chave secreta (`sk_live_...` ou `sk_test_...`) |
| `STRIPE_WEBHOOK_SECRET` | Segredo de webhook (`whsec_...`) — necessário para validação de assinatura |

## Conversão de Valores

O Stripe trabalha com **centavos** (menor unidade da moeda). O plugin converte automaticamente:

- `create_payment(100.0, "usd", ...)` → Stripe recebe `amount=10000`
- `capture_payment(id, Some(50.0))` → Stripe recebe `amount_to_capture=5000`

## Eventos de Webhook Suportados

| Evento Stripe | Ação Normalizada |
|---|---|
| `payment_intent.amount_capturable_updated` | `Authorized` |
| `payment_intent.succeeded` | `Captured` |
| `payment_intent.payment_failed` | `Failed` |
| `payment_intent.canceled` | `Cancelled` |
| `payment_intent.processing` | `Pending` |
| `charge.refunded` | `Refunded` |
| `charge.refund.updated` | `Refunded` |

## Executar Testes

```bash
cargo test --package stripe_plugin
```

### Cobertura dos testes

| Teste | Endpoint mockado |
|-------|-----------------|
| `test_create_payment_success` | `POST /v1/payment_intents` |
| `test_create_payment_negative_amount_fails` | validação local |
| `test_capture_payment_success` | `POST /v1/payment_intents/{id}/capture` |
| `test_refund_payment_success` | `POST /v1/refunds` |
| `test_cancel_payment_success` | `POST /v1/payment_intents/{id}/cancel` |
| `test_webhook_payment_intent_succeeded` | parsing local |
| `test_webhook_payment_intent_failed` | parsing local |
| `test_webhook_charge_refunded` | parsing local |
| `test_webhook_payment_intent_authorized` | parsing local |
| `test_create_webhook` | `POST /v1/webhook_endpoints` |
| `test_list_webhooks` | `GET /v1/webhook_endpoints` |
| `test_delete_webhook` | `DELETE /v1/webhook_endpoints/{id}` |

## Referências

- [Stripe API Reference](https://stripe.com/docs/api)
- [PaymentIntents](https://stripe.com/docs/api/payment_intents)
- [Webhooks](https://stripe.com/docs/webhooks)
- [Webhook Endpoints API](https://stripe.com/docs/api/webhook_endpoints)
