# Plugin Asaas para Medusa Rust

Plugin de pagamento para integração com [Asaas](https://www.asaas.com/), gateway brasileiro de pagamentos.

## Instalação

```toml
[dependencies]
asaas_plugin = { path = "crates/plugins/asaas" }
```

## Uso Básico

```rust
use asaas_plugin::AsaasPlugin;
use plugin_api::PaymentProvider;
use std::collections::HashMap;

let mut plugin = AsaasPlugin::new();
let mut config = HashMap::new();
config.insert("api_key".to_string(), std::env::var("ASAAS_API_KEY").unwrap());
config.insert("base_url".to_string(), std::env::var("ASAAS_BASE_URL").unwrap());
plugin.initialize(config).await?;

// Criar pagamento
let mut meta = HashMap::new();
meta.insert("customer".to_string(), "cus_abc123".to_string());
meta.insert("billing_type".to_string(), "PIX".to_string());
meta.insert("due_date".to_string(), "2025-12-31".to_string());

let payment = plugin.create_payment(100.0, "BRL", meta).await?;
println!("Payment ID: {}", payment["id"]);
```

## Gerenciamento de Webhooks

```rust
// Criar webhook
let webhook_id = plugin.create_webhook(
    "https://meusite.com/hooks/payment/asaas",
    vec!["PAYMENT_RECEIVED".to_string(), "PAYMENT_CONFIRMED".to_string()],
).await?;

// Listar webhooks
let webhooks = plugin.list_webhooks().await?;
for wh in webhooks {
    println!("{}: {}", wh.id, wh.url);
}

// Deletar webhook
plugin.delete_webhook(&webhook_id).await?;
```

## Configuração

| Variável de Ambiente | Descrição | Exemplo |
|---|---|---|
| `ASAAS_API_KEY` | API key do Asaas | `$aact_xxx...` |
| `ASAAS_BASE_URL` | URL base da API | `https://sandbox.asaas.com/api/v3` |

### URLs

- **Sandbox**: `https://sandbox.asaas.com/api/v3`
- **Produção**: `https://api.asaas.com/v3`

## Eventos de Webhook Suportados

| Evento | Ação Resultante |
|---|---|
| `PAYMENT_RECEIVED` | `Captured` |
| `PAYMENT_CONFIRMED` | `Captured` |
| `PAYMENT_OVERDUE` | `Pending` |
| `PAYMENT_PENDING` | `Pending` |
| `PAYMENT_REFUNDED` | `Refunded` |
| `PAYMENT_PARTIALLY_REFUNDED` | `Refunded` |
| `PAYMENT_DELETED` | `Cancelled` |
| `PAYMENT_CREATED` | `Authorized` |

## Executar Testes

```bash
cargo test --package asaas_plugin
```
