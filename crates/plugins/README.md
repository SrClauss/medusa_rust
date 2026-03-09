# Payment Provider Plugins

Este diretório contém os plugins de pagamento do MedusaRust, cada um implementando
o trait `PaymentProvider` definido no crate `plugin_api`.

## Arquitetura

```
crates/
├── plugin_api/          # Trait PaymentProvider e tipos comuns
│   └── src/lib.rs
└── plugins/
    ├── asaas/           # Gateway brasileiro Asaas (PIX, Boleto, Cartão)
    ├── mercadopago/     # Mercado Pago (América Latina)
    ├── stripe/          # Stripe (internacional)
    └── paypal/          # PayPal (internacional, OAuth2)
```

### Trait `PaymentProvider`

Definido em `crates/plugin_api/src/lib.rs` e implementado de forma assíncrona por cada plugin:

```rust
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn initialize(&mut self, config: HashMap<String, String>) -> anyhow::Result<()>;
    async fn create_payment(&self, amount: f64, currency: &str, metadata: HashMap<String, String>) -> anyhow::Result<serde_json::Value>;
    async fn capture_payment(&self, payment_id: &str, amount: Option<f64>) -> anyhow::Result<serde_json::Value>;
    async fn refund_payment(&self, payment_id: &str, amount: Option<f64>, reason: Option<&str>) -> anyhow::Result<serde_json::Value>;
    async fn cancel_payment(&self, payment_id: &str) -> anyhow::Result<serde_json::Value>;
    async fn get_webhook_action_and_data(&self, payload: WebhookPayload) -> anyhow::Result<WebhookActionResult>;

    // Gerenciamento de endpoints de webhook (implementação padrão retorna erro)
    async fn create_webhook(&self, url: &str, events: Vec<String>) -> anyhow::Result<String>;
    async fn list_webhooks(&self) -> anyhow::Result<Vec<WebhookInfo>>;
    async fn delete_webhook(&self, webhook_id: &str) -> anyhow::Result<()>;
    async fn update_webhook(&self, webhook_id: &str, events: Vec<String>) -> anyhow::Result<()>;
}
```

### `PluginManager`

O `PluginManager` (em `src/plugins/mod.rs`) armazena instâncias de providers e os
disponibiliza para os handlers HTTP. Suporta dois tipos de providers:

- **Legado (síncrono)**: `register_payment_provider()` — para o trait `crate::core::payment::PaymentProvider`
- **Novo (assíncrono)**: `register_async_payment_provider()` — para o trait `plugin_api::PaymentProvider`

Os plugins deste diretório usam o registro assíncrono.

## Status dos Plugins

| Funcionalidade | Asaas | Mercado Pago | Stripe | PayPal |
|----------------|-------|-------------|--------|--------|
| Criar pagamento | ✅ | ✅ | ✅ PaymentIntent | ✅ Order |
| Capturar | ✅ receiveInCash | ✅ | ✅ capture | ✅ capture |
| Reembolsar | ✅ | ✅ | ✅ Refunds API | ✅ |
| Cancelar | ✅ DELETE | ✅ PUT | ✅ cancel | ✅ (sintético) |
| Parsear webhooks | ✅ | ✅ | ✅ | ✅ |
| Criar webhook | ✅ | ✅ | ✅ | ✅ |
| Listar webhooks | ✅ | ✅ | ✅ | ✅ |
| Excluir webhook | ✅ | ✅ | ✅ | ✅ |
| Validação de assinatura | — | — | ✅ HMAC-SHA256 | — |
| OAuth2 automático | — | — | — | ✅ |
| Testes (mockito) | ✅ 11 | ✅ 11 | ✅ 12 | ✅ 12 |

## Ativar Plugins em Produção

Os plugins são carregados automaticamente na inicialização do servidor se as variáveis
de ambiente estiverem configuradas:

```bash
# .env

# Asaas
ASAAS_API_KEY=$aact_...
ASAAS_BASE_URL=https://api.asaas.com/v3   # omitir para sandbox

# Mercado Pago
MP_ACCESS_TOKEN=APP_USR-...

# Stripe
STRIPE_SECRET_KEY=sk_live_...
STRIPE_WEBHOOK_SECRET=whsec_...

# PayPal
PAYPAL_CLIENT_ID=AYmJ...
PAYPAL_CLIENT_SECRET=ELuC...
PAYPAL_BASE_URL=https://api-m.paypal.com   # omitir para sandbox
```

Plugins sem variáveis configuradas são ignorados com um `WARN` no log.

## Como Adicionar um Novo Provedor

1. **Criar o crate**:
   ```bash
   mkdir -p crates/plugins/meu_plugin/src crates/plugins/meu_plugin/tests
   ```

2. **Implementar a estrutura** com a mesma organização dos existentes:
   ```
   meu_plugin/
   ├── Cargo.toml       # dep: plugin_api, reqwest, serde, anyhow, tokio, tracing
   ├── README.md
   ├── src/
   │   ├── lib.rs       # re-exporta o plugin principal
   │   ├── plugin.rs    # MeuPlugin: impl PaymentProvider
   │   ├── types.rs     # structs de req/resp específicos do provedor
   │   └── webhooks.rs  # funções create/list/delete_webhook
   └── tests/
       └── integration_tests.rs  # testes com mockito
   ```

3. **Adicionar ao workspace** em `Cargo.toml` raiz:
   ```toml
   [workspace]
   members = [
       # ... existentes
       "crates/plugins/meu_plugin",
   ]
   ```

4. **Adicionar dependência** no `Cargo.toml` principal:
   ```toml
   [dependencies]
   meu_plugin = { path = "crates/plugins/meu_plugin" }
   ```

5. **Registrar** em `src/main.rs` no bloco de inicialização de plugins.

## Como Testar Individualmente

```bash
# Plugin específico (sem PostgreSQL necessário)
cargo test --package asaas_plugin
cargo test --package mercadopago_plugin
cargo test --package stripe_plugin
cargo test --package paypal_plugin

# Todos os plugins
cargo test --package asaas_plugin --package mercadopago_plugin --package stripe_plugin --package paypal_plugin

# Workspace completo (servidor + todos os plugins)
cargo test --workspace
```

## Uso via Código

```rust
use asaas_plugin::AsaasPlugin;
use plugin_api::PaymentProvider;
use std::collections::HashMap;

let mut plugin = AsaasPlugin::new();
let mut config = HashMap::new();
config.insert("api_key".into(), std::env::var("ASAAS_API_KEY").unwrap());
plugin.initialize(config).await?;

// Criar pagamento
let mut meta = HashMap::new();
meta.insert("customer".into(), "cus_abc".into());
meta.insert("billing_type".into(), "PIX".into());
meta.insert("due_date".into(), "2025-12-31".into());
let payment = plugin.create_payment(100.0, "BRL", meta).await?;

// Gerenciar webhooks
let wh_id = plugin.create_webhook(
    "https://meu-servidor.com/hooks/payment/asaas",
    vec!["PAYMENT_RECEIVED".into()],
).await?;
let lista = plugin.list_webhooks().await?;
plugin.delete_webhook(&wh_id).await?;
```

## Troubleshooting

**Build falha com "not found in workspace"**  
Verifique que o novo crate está listado em `[workspace] members` no `Cargo.toml` raiz.

**Testes falham com "connection refused"**  
Os testes usam `mockito` — não é necessária conectividade com a internet. Verifique que `mockito` está em `[dev-dependencies]`.

**Plugin não é carregado na inicialização**  
Verifique o log do servidor: um `WARN` indica qual variável de ambiente está faltando.

**Erro 401 em produção**  
Verifique que as credenciais nas variáveis de ambiente são de produção (não sandbox).
