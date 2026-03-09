# Plugin Mercado Pago para Medusa Rust

Plugin de pagamento para integração com [Mercado Pago](https://www.mercadopago.com/), a maior plataforma de pagamentos da América Latina.

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

// Criar pagamento PIX
let mut meta = HashMap::new();
meta.insert("payer_email".to_string(), "user@example.com".to_string());
meta.insert("payment_method_id".to_string(), "pix".to_string()); // pix | credit_card | boleto
meta.insert("order_id".to_string(), "order_123".to_string());    // referência interna

let payment = plugin.create_payment(50.0, "BRL", meta).await?;
println!("Payment ID: {}", payment["id"]);
println!("Status: {}", payment["status"]);
```

## Captura, Reembolso e Cancelamento

```rust
let id = payment["id"].to_string();

// Capturar pagamento autorizado (PUT com capture=true)
let captured = plugin.capture_payment(&id, None).await?;

// Reembolsar (parcial ou total)
let refunded = plugin.refund_payment(&id, Some(25.0), None).await?;

// Cancelar (PUT com status=cancelled)
let cancelled = plugin.cancel_payment(&id).await?;
```

## Gerenciamento de Webhooks

```rust
// Criar webhook
let webhook_id = plugin.create_webhook(
    "https://meusite.com/hooks/payment/mercadopago",
    vec!["payment.created".to_string(), "payment.updated".to_string()],
).await?;

// Listar
let webhooks = plugin.list_webhooks().await?;
for wh in &webhooks {
    println!("{}: {} (ativo: {})", wh.id, wh.url, wh.active);
}

// Deletar
plugin.delete_webhook(&webhook_id).await?;
```

## Configuração

| Variável de Ambiente | Descrição |
|---|---|
| `MP_ACCESS_TOKEN` | Access token do Mercado Pago (`APP_USR-...`) |

> Para obter seu `access_token`, acesse o [painel de desenvolvedores](https://www.mercadopago.com/developers/pt/docs/your-integrations/credentials).

## Eventos de Webhook Suportados

| Evento MP | Ação Normalizada |
|---|---|
| `payment.created` | `Authorized` |
| `payment.updated` (approved) | `Captured` |
| `payment.updated` (cancelled) | `Cancelled` |
| `payment.updated` (refunded) | `Refunded` |
| `payment.updated` (pending / in_process) | `Pending` |
| `payment.updated` (rejected) | `Failed` |

## Executar Testes

```bash
cargo test --package mercadopago_plugin
```

### Cobertura dos testes

| Teste | Endpoint mockado |
|-------|-----------------|
| `test_create_payment_success` | `POST /v1/payments` |
| `test_create_payment_negative_amount_fails` | validação local |
| `test_capture_payment_success` | `POST /v1/payments/{id}` |
| `test_refund_payment_success` | `POST /v1/payments/{id}/refunds` |
| `test_cancel_payment_success` | `PUT /v1/payments/{id}` |
| `test_webhook_payment_approved` | parsing local |
| `test_webhook_payment_rejected` | parsing local |
| `test_webhook_payment_created` | parsing local |
| `test_create_webhook` | `POST /v1/webhooks` |
| `test_list_webhooks` | `GET /v1/webhooks` |
| `test_delete_webhook` | `DELETE /v1/webhooks/{id}` |

## Referências

- [Checkout API](https://www.mercadopago.com/developers/pt/docs/checkout-api/introduction)
- [Notificações Webhook](https://www.mercadopago.com/developers/pt/docs/your-integrations/notifications/webhooks)
