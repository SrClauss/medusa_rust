# Payment Provider Plugins

Este diretório contém os plugins de pagamento do MedusaRust, cada um implementando
o trait `PaymentProvider` definido no crate `plugin_api`.

## Arquitetura

```
crates/
├── plugin_api/          # Trait PaymentProvider e tipos comuns
└── plugins/
    ├── asaas/           # Gateway brasileiro Asaas
    ├── mercadopago/     # Mercado Pago
    ├── stripe/          # Stripe (international)
    └── paypal/          # PayPal (international)
```

### Trait `PaymentProvider`

Todos os plugins implementam o trait assíncrono `PaymentProvider` definido em
`crates/plugin_api/src/lib.rs`. O trait inclui:

- `create_payment` — cria uma sessão/intenção de pagamento
- `capture_payment` — captura um pagamento autorizado
- `refund_payment` — reembolsa (total ou parcialmente)
- `cancel_payment` — cancela/estorna
- `get_webhook_action_and_data` — parseia webhooks do provedor
- `create_webhook` / `list_webhooks` / `delete_webhook` — gerencia endpoints

### `PluginManager`

O `PluginManager` (em `src/plugins/mod.rs`) armazena instâncias de providers
e os disponibiliza para os handlers HTTP durante o processamento de webhooks.

## Como Adicionar um Novo Provedor

1. Crie `crates/plugins/<nome>/` com a mesma estrutura dos existentes:
   - `Cargo.toml` com dependência em `plugin_api`
   - `src/lib.rs`, `src/plugin.rs`, `src/types.rs`, `src/webhooks.rs`
   - `tests/integration_tests.rs` usando `mockito`
   - `README.md`

2. Implemente o trait `plugin_api::PaymentProvider` para sua struct.

3. Adicione o novo crate ao workspace em `Cargo.toml` raiz:
   ```toml
   [workspace]
   members = [
       # ... existentes
       "crates/plugins/<nome>",
   ]
   ```

4. Adicione a dependência no `Cargo.toml` principal:
   ```toml
   [dependencies]
   nome_plugin = { path = "crates/plugins/<nome>" }
   ```

5. Registre o plugin em `src/main.rs`.

## Como Testar Individualmente

```bash
# Testar apenas o plugin Asaas
cargo test --package asaas_plugin

# Testar apenas o plugin Stripe
cargo test --package stripe_plugin

# Testar todos os plugins
cargo test --workspace
```

## Configuração de Webhooks Automáticos

Cada plugin suporta criação de webhooks via API. Exemplo com Stripe:

```rust
use stripe_plugin::StripePlugin;
use plugin_api::PaymentProvider;

let webhook_id = stripe_plugin.create_webhook(
    "https://meusite.com/hooks/payment/stripe",
    vec![
        "payment_intent.succeeded".to_string(),
        "payment_intent.payment_failed".to_string(),
    ],
).await?;
```

## Troubleshooting

### Build falha com "not found in workspace"
Verifique que o novo crate está listado em `[workspace] members` no `Cargo.toml` raiz.

### Testes falham com timeout
Os testes usam `mockito` para mockar APIs externas. Não é necessária conectividade
com a internet.

### Erro 401 em produção
Verifique que as variáveis de ambiente com as credenciais estão corretamente
configuradas (veja o README de cada plugin).
