# plugin_api

Crate central de abstrações para o sistema de plugins de pagamento do **MedusaRust**.

Define o trait `PaymentProvider` (assíncrono) e todos os tipos compartilhados — `WebhookPayload`, `WebhookActionResult`, `WebhookInfo` e `PaymentAction` — que todos os plugins de pagamento devem usar.

## Instalação

```toml
[dependencies]
plugin_api = { path = "crates/plugin_api" }
```

## Visão Geral dos Tipos

### `PaymentAction`

Ação normalizada retornada após processar um webhook:

```rust
pub enum PaymentAction {
    Authorized,    // Pagamento autorizado (não capturado)
    Captured,      // Pagamento capturado (dinheiro debitado)
    Refunded,      // Reembolso total
    PartialRefund, // Reembolso parcial
    Failed,        // Pagamento falhou
    Cancelled,     // Pagamento cancelado/estornado
    Pending,       // Aguardando confirmação
    NotSupported,  // Evento não mapeado
}
```

### `WebhookPayload`

Payload normalizado recebido pelo servidor e repassado ao plugin:

```rust
pub struct WebhookPayload {
    pub provider: String,          // "asaas" | "stripe" | "paypal" | "mercadopago"
    pub provider_id: String,       // ID do pagamento no provedor
    pub order_id: Option<String>,  // ID do pedido interno (se disponível)
    pub amount: Option<f64>,       // Valor do pagamento
    pub currency: Option<String>,  // Código da moeda: "BRL", "USD", etc.
    pub raw_data: serde_json::Value, // Corpo JSON bruto do webhook
}
```

### `WebhookActionResult`

Resultado retornado pelo método `get_webhook_action_and_data`:

```rust
pub struct WebhookActionResult {
    pub action: PaymentAction,
    pub data: HashMap<String, serde_json::Value>, // Dados extras do provedor
}
```

### `WebhookInfo`

Informações sobre um endpoint de webhook registrado:

```rust
pub struct WebhookInfo {
    pub id: String,           // ID atribuído pelo provedor
    pub url: String,          // URL que recebe os eventos
    pub events: Vec<String>,  // Lista de tipos de evento
    pub active: bool,         // Se o endpoint está ativo
}
```

---

## Trait `PaymentProvider`

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

    // Métodos opcionais — implementação padrão retorna Err
    async fn create_webhook(&self, url: &str, events: Vec<String>) -> anyhow::Result<String>;
    async fn list_webhooks(&self) -> anyhow::Result<Vec<WebhookInfo>>;
    async fn delete_webhook(&self, webhook_id: &str) -> anyhow::Result<()>;
    async fn update_webhook(&self, webhook_id: &str, events: Vec<String>) -> anyhow::Result<()>;
}
```

### Referência de Métodos

| Método | Obrigatório | Descrição |
|--------|:-----------:|-----------|
| `name()` | ✅ | Identificador único do provider, ex: `"stripe"` |
| `initialize(config)` | ✅ | Inicializa com credenciais vindas de variáveis de ambiente |
| `create_payment(amount, currency, metadata)` | ✅ | Cria sessão/intenção de pagamento |
| `capture_payment(id, amount?)` | ✅ | Captura um pagamento autorizado |
| `refund_payment(id, amount?, reason?)` | ✅ | Reembolsa total ou parcialmente |
| `cancel_payment(id)` | ✅ | Cancela/estorna um pagamento |
| `get_webhook_action_and_data(payload)` | ✅ | Parseia evento do provedor e retorna `PaymentAction` normalizado |
| `create_webhook(url, events)` | ⚪ | Registra endpoint de webhook no provedor |
| `list_webhooks()` | ⚪ | Lista endpoints registrados |
| `delete_webhook(id)` | ⚪ | Remove endpoint registrado |
| `update_webhook(id, events)` | ⚪ | Atualiza eventos de um endpoint |

> ⚪ **Opcionais**: os métodos de gerenciamento de webhooks possuem implementação padrão que retorna `Err`. Plugins que não suportam a funcionalidade não precisam implementá-los.

---

## Como Implementar um Novo Plugin

### 1. Adicionar `plugin_api` como dependência

```toml
# crates/plugins/meu_plugin/Cargo.toml
[dependencies]
plugin_api  = { path = "../../plugin_api" }
async-trait = "0.1"
reqwest     = { version = "0.12", features = ["json"] }
serde       = { version = "1.0", features = ["derive"] }
serde_json  = "1.0"
anyhow      = "1.0"
tokio       = { version = "1", features = ["full"] }
tracing     = "0.1"
```

### 2. Implementar o trait

```rust
use async_trait::async_trait;
use plugin_api::{PaymentAction, PaymentProvider, WebhookActionResult, WebhookPayload};
use std::collections::HashMap;

pub struct MeuPlugin {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

#[async_trait]
impl PaymentProvider for MeuPlugin {
    fn name(&self) -> &str {
        "meu_plugin"
    }

    async fn initialize(&mut self, config: HashMap<String, String>) -> anyhow::Result<()> {
        self.api_key = config
            .get("api_key")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("'api_key' obrigatório"))?;
        Ok(())
    }

    async fn create_payment(
        &self,
        amount: f64,
        currency: &str,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<serde_json::Value> {
        // Chamar a API do provedor...
        todo!()
    }

    async fn capture_payment(&self, payment_id: &str, _amount: Option<f64>) -> anyhow::Result<serde_json::Value> {
        todo!()
    }

    async fn refund_payment(&self, payment_id: &str, _amount: Option<f64>, _reason: Option<&str>) -> anyhow::Result<serde_json::Value> {
        todo!()
    }

    async fn cancel_payment(&self, payment_id: &str) -> anyhow::Result<serde_json::Value> {
        todo!()
    }

    async fn get_webhook_action_and_data(
        &self,
        payload: WebhookPayload,
    ) -> anyhow::Result<WebhookActionResult> {
        let event = payload.raw_data["event"].as_str().unwrap_or("");
        let action = match event {
            "payment.success"  => PaymentAction::Captured,
            "payment.failed"   => PaymentAction::Failed,
            "payment.refunded" => PaymentAction::Refunded,
            _                  => PaymentAction::NotSupported,
        };
        Ok(WebhookActionResult {
            action,
            data: HashMap::new(),
        })
    }
}
```

### 3. Escrever testes com `mockito`

```rust
// crates/plugins/meu_plugin/tests/integration_tests.rs
use meu_plugin::MeuPlugin;
use mockito::Server;
use plugin_api::{PaymentAction, PaymentProvider, WebhookPayload};
use std::collections::HashMap;

async fn plugin_with_server(server: &Server) -> MeuPlugin {
    let mut plugin = MeuPlugin::new();
    let mut config = HashMap::new();
    config.insert("api_key".into(), "test_key".into());
    config.insert("base_url".into(), server.url());
    plugin.initialize(config).await.unwrap();
    plugin
}

#[tokio::test]
async fn test_create_payment_success() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/payments")
        .with_status(200)
        .with_body(r#"{"id":"pay_1","status":"pending"}"#)
        .create_async()
        .await;

    let plugin = plugin_with_server(&server).await;
    let result = plugin.create_payment(100.0, "BRL", HashMap::new()).await.unwrap();
    assert_eq!(result["id"], "pay_1");
    mock.assert_async().await;
}
```

### 4. Registrar no workspace e no `main.rs`

Ver instruções completas em [`crates/plugins/README.md`](../plugins/README.md).

---

## Plugins Disponíveis

| Crate | Provedor | Região |
|-------|----------|--------|
| [`asaas_plugin`](../plugins/asaas/README.md) | Asaas | Brasil (PIX, Boleto, Cartão) |
| [`mercadopago_plugin`](../plugins/mercadopago/README.md) | Mercado Pago | América Latina |
| [`stripe_plugin`](../plugins/stripe/README.md) | Stripe | Internacional |
| [`paypal_plugin`](../plugins/paypal/README.md) | PayPal | Internacional |

---

## Dependências

| Crate | Uso |
|-------|-----|
| `async-trait` | Permite métodos `async` em traits de objetos |
| `serde` / `serde_json` | Serialização de `WebhookPayload`, `WebhookInfo`, `PaymentAction` |
| `anyhow` | Tipo de erro ergonômico para `Result` nos métodos do trait |
| `chrono` | Suporte a timestamps nos tipos auxiliares |
| `reqwest` | Re-exportado para conveniência dos plugins |
