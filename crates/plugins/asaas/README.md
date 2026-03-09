# Plugin Asaas para Medusa Rust

Plugin de pagamento para integração com [Asaas](https://www.asaas.com/), gateway brasileiro de pagamentos que suporta PIX, Boleto e Cartão de Crédito.

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
config.insert("base_url".to_string(), std::env::var("ASAAS_BASE_URL").unwrap_or_else(|_| "https://sandbox.asaas.com/api/v3".into()));
plugin.initialize(config).await?;

// Criar cobrança PIX
let mut meta = HashMap::new();
meta.insert("customer".to_string(), "cus_abc123".to_string());   // ID do cliente no Asaas
meta.insert("billing_type".to_string(), "PIX".to_string());      // PIX | BOLETO | CREDIT_CARD
meta.insert("due_date".to_string(), "2025-12-31".to_string());   // YYYY-MM-DD

let payment = plugin.create_payment(100.0, "BRL", meta).await?;
println!("ID da cobrança: {}", payment["id"]);
println!("URL do PIX: {}", payment["pixQrCode"]);
```

## Captura, Reembolso e Cancelamento

```rust
// Confirmar pagamento como recebido em dinheiro (receiveInCash)
let captured = plugin.capture_payment(&payment["id"].as_str().unwrap(), None).await?;

// Reembolsar
let refunded = plugin.refund_payment(&payment["id"].as_str().unwrap(), None, None).await?;

// Cancelar (DELETE /payments/{id})
let cancelled = plugin.cancel_payment(&payment["id"].as_str().unwrap()).await?;
```

## Gerenciamento de Webhooks

```rust
// Criar webhook
let webhook_id = plugin.create_webhook(
    "https://meusite.com/hooks/payment/asaas",
    vec!["PAYMENT_RECEIVED".to_string(), "PAYMENT_CONFIRMED".to_string()],
).await?;

// Listar webhooks cadastrados
let webhooks = plugin.list_webhooks().await?;
for wh in &webhooks {
    println!("{}: {} (ativo: {})", wh.id, wh.url, wh.active);
}

// Deletar webhook
plugin.delete_webhook(&webhook_id).await?;
```

## Receber Webhooks no Servidor

O servidor MedusaRust expõe a rota `POST /hooks/payment/asaas`. Ao receber um evento,
o payload é normalizado e o `PluginManager` delega ao `AsaasPlugin`:

```rust
// O plugin parseia automaticamente o evento recebido:
let result = plugin.get_webhook_action_and_data(payload).await?;
match result.action {
    PaymentAction::Captured => { /* atualizar pedido como pago */ }
    PaymentAction::Refunded => { /* registrar reembolso */ }
    PaymentAction::Pending  => { /* manter como pendente */ }
    _ => {}
}
```

## Configuração

| Variável de Ambiente | Descrição | Exemplo |
|---|---|---|
| `ASAAS_API_KEY` | API key do Asaas | `$aact_YTU5YTE0M...` |
| `ASAAS_BASE_URL` | URL base da API | `https://sandbox.asaas.com/api/v3` |

### URLs da API

| Ambiente | URL |
|----------|-----|
| **Sandbox** | `https://sandbox.asaas.com/api/v3` |
| **Produção** | `https://api.asaas.com/v3` |

## Eventos de Webhook Suportados

| Evento Asaas | Ação Normalizada |
|---|---|
| `PAYMENT_RECEIVED` | `Captured` |
| `PAYMENT_CONFIRMED` | `Captured` |
| `PAYMENT_OVERDUE` | `Pending` |
| `PAYMENT_PENDING` | `Pending` |
| `PAYMENT_REFUNDED` | `Refunded` |
| `PAYMENT_PARTIALLY_REFUNDED` | `Refunded` |
| `PAYMENT_DELETED` | `Cancelled` |
| `PAYMENT_ANTICIPATED` | `Cancelled` |
| `PAYMENT_CREATED` | `Authorized` |

## Executar Testes

Os testes usam `mockito` para simular a API do Asaas — nenhuma chave real é necessária:

```bash
cargo test --package asaas_plugin
```

### Cobertura dos testes

| Teste | Endpoint mockado |
|-------|-----------------|
| `test_create_payment_success` | `POST /payments` |
| `test_create_payment_negative_amount_fails` | validação local |
| `test_capture_payment_success` | `POST /payments/{id}/receiveInCash` |
| `test_refund_payment_success` | `POST /payments/{id}/refund` |
| `test_cancel_payment_success` | `DELETE /payments/{id}` |
| `test_webhook_payment_received` | parsing local |
| `test_webhook_payment_failed` | parsing local |
| `test_webhook_payment_refunded` | parsing local |
| `test_create_webhook` | `POST /webhooks` |
| `test_list_webhooks` | `GET /webhooks` |
| `test_delete_webhook` | `DELETE /webhooks/{id}` |

## Referências

- [Documentação da API Asaas](https://docs.asaas.com/reference)
- [Webhooks Asaas](https://docs.asaas.com/reference/webhooks)
- [Criar Webhook](https://docs.asaas.com/reference/criar-webhook)
