# Prompt: Completar Event Bus & Workflows com Feature-Parity do MedusaJS

## FILOSOFIA DO PROJETO

Este é um **PORT do MedusaJS para Rust**. Toda implementação deve:

1. ✅ **Consultar PRIMEIRO** o código original em `vendor/medusa_js/`
2. ✅ **Analisar** a arquitetura TypeScript/JavaScript
3. ✅ **Adaptar** para Rust mantendo a mesma lógica e comportamento
4. ✅ **Seguir** os padrões já estabelecidos no projeto (veja `crates/plugins/` como exemplo)

**⚠️ IMPORTANTE**: NÃO invente padrões novos sem verificar como o MedusaJS implementa primeiro!

---

## ARQUITETURA MODULAR COM REDIS OPCIONAL

Este prompt implementa Event Bus e Workflows com **Redis como dependência opcional** usando Cargo features.

### Três Modos de Operação

#### 🟢 Modo Local (Padrão - Sem Redis)
- Event bus: `tokio::sync::broadcast` (in-memory)
- Workflows: Estado em memória + persistência PostgreSQL
- **Quando usar**: Desenvolvimento, aplicação monolítica, single-instance
- **Limitações**: Eventos não atravessam processos, sem coordenação distribuída

#### 🟡 Modo Híbrido (Redis para Event Bus)
- Event bus: Redis Pub/Sub (distribuído)
- Workflows: Persistência PostgreSQL apenas
- **Quando usar**: Múltiplas instâncias, mas workflows gerenciados centralmente
- **Vantagens**: Eventos distribuídos, workflows ainda locais

#### 🔴 Modo Distribuído Completo (Feature `distributed`)
- Event bus: Redis Pub/Sub
- Workflows: Redis Streams + locking distribuído + PostgreSQL
- **Quando usar**: Produção multi-instância, alta disponibilidade
- **Vantagens**: Coordenação completa, retry automático, locks distribuídos

### Estrutura de Features no Cargo.toml

```toml
[features]
default = ["local-bus", "local-workflows"]

# Event Bus modes
local-bus = []
redis-bus = ["redis", "redis-streams"]

# Workflow modes
local-workflows = []
redis-workflows = ["redis", "redis-streams"]

# Full distributed mode
distributed = ["redis-bus", "redis-workflows"]

[dependencies]
# Sempre presentes
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
anyhow = "1.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio", "uuid"] }

# Opcional - só com features redis
redis = { version = "0.24", features = ["aio", "tokio-comp", "connection-manager", "streams"], optional = true }
```

### Comandos de Compilação

```bash
# Modo local (padrão)
cargo build

# Com Redis para Event Bus
cargo build --features redis-bus

# Modo distribuído completo
cargo build --features distributed

# Testes seletivos
cargo test --features redis-bus
cargo test --features distributed
```

---

## Contexto Atual

O PR #13 já implementou uma **base funcional** de Event Bus e Sagas, mas diversos recursos **CRÍTICOS** do MedusaJS original ainda estão faltando. Este prompt visa completar a implementação com feature-parity.

### ✅ O que JÁ está implementado:
- Event bus in-memory (`tokio::sync::broadcast`)
- Enum `Event` com eventos de domínio
- API `publish()` / `subscribe()`
- Sagas com steps + compensations
- `WorkflowEngine` para registrar sagas
- 9 testes básicos

### ❌ O que está FALTANDO (gaps críticos):

#### Event Bus:
1. 🔴 **Redis/SQS drivers são stubs** (linhas 203-209 de `src/events.rs` confirmam)
2. 🔴 **Event grouping** (para transações distribuídas)
3. 🔴 **Unsubscribe** (causa memory leaks)
4. 🟡 **Interceptors** (auditoria/modificação de eventos)

#### Workflows/Sagas:
1. 🔴 **Zero persistência** (workflows perdidos em restart)
2. 🔴 **Não integrado com EventBus** (workflows não reagem a eventos)
3. 🔴 **Sem distributed transaction IDs** (sem idempotency)
4. 🟡 **Sem scheduler** (retry automático)

**Referência completa dos gaps**: Ver `ANALISE_DIFERENCAS_IMPLEMENTACAO.md` e `STATUS_POS_MERGE_PR13.md`

---

## TAREFA PRINCIPAL

Implementar **TODOS** os gaps identificados seguindo **rigorosamente** o MedusaJS original, com adaptações Rust idiomáticas quando necessário.

---

## PARTE 1: Event Bus — Corrigir Gaps Críticos

### 📚 Código de Referência Obrigatório

**Leia e analise COMPLETAMENTE antes de codificar**:

```
vendor/medusa_js/packages/modules/event-bus-local/src/services/event-bus-local.ts
vendor/medusa_js/packages/modules/event-bus-redis/src/services/event-bus-redis.ts
```

**Foque em**:
- Método `emit()` e opções (delay, grouping)
- Métodos `groupOrEmitEvent()`, `releaseGroupedEvents()`, `clearGroupedEvents()`
- Método `subscribe()` e `unsubscribe()` com subscriberId
- Sistema de interceptors (`callInterceptors()`)
- Tratamento de erros e retry

---

### Task 1.1: Implementar Redis Driver com Feature Flag

**Problema atual**: Linhas 203-209 de `src/events.rs` confirmam que Redis é stub.

**Solução com Features**:

1. **Adicionar dependências opcionais** no `Cargo.toml` (veja seção "Arquitetura Modular" acima)

2. **Criar abstração de backend** em `src/events/backend.rs` (novo arquivo):

```rust
use async_trait::async_trait;
use crate::events::Event;

#[async_trait]
pub trait EventBackend: Send + Sync {
    async fn publish(&self, event: &Event) -> anyhow::Result<()>;
    async fn subscribe<H>(&self, handler: H) -> anyhow::Result<SubscriptionHandle>
    where
        H: EventHandler + 'static;
}

pub struct SubscriptionHandle {
    task: tokio::task::JoinHandle<()>,
    id: Option<String>,
}

impl SubscriptionHandle {
    pub fn cancel(self) {
        self.task.abort();
        tracing::debug!(id = ?self.id, "Subscription cancelled");
    }
}
```

3. **Implementar LocalBackend** em `src/events/local_backend.rs`:

```rust
use super::backend::{EventBackend, SubscriptionHandle};
use crate::events::{Event, EventHandler};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

pub struct LocalBackend {
    tx: Arc<Mutex<broadcast::Sender<Event>>>,
}

impl LocalBackend {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            tx: Arc::new(Mutex::new(tx)),
        }
    }
}

#[async_trait]
impl EventBackend for LocalBackend {
    async fn publish(&self, event: &Event) -> anyhow::Result<()> {
        let _ = self.tx.lock().await.send(event.clone());
        tracing::debug!("Event published (local)");
        Ok(())
    }

    async fn subscribe<H>(&self, handler: H) -> anyhow::Result<SubscriptionHandle>
    where
        H: EventHandler + 'static,
    {
        let mut rx = self.tx.lock().await.subscribe();
        
        let task = tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        if let Err(e) = handler.handle(event).await {
                            tracing::warn!(error = %e, "Handler failed");
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(skipped = n, "Subscriber lagged");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
        
        Ok(SubscriptionHandle { task, id: None })
    }
}
```

4. **Implementar RedisBackend** (APENAS com feature flag) em `src/events/redis_backend.rs`:

```rust
#[cfg(feature = "redis-bus")]
use super::backend::{EventBackend, SubscriptionHandle};
#[cfg(feature = "redis-bus")]
use crate::events::{Event, EventHandler};
#[cfg(feature = "redis-bus")]
use async_trait::async_trait;
#[cfg(feature = "redis-bus")]
use redis::aio::ConnectionManager;

#[cfg(feature = "redis-bus")]
pub struct RedisBackend {
    conn_manager: ConnectionManager,
    channel: String,
}

#[cfg(feature = "redis-bus")]
impl RedisBackend {
    pub async fn new(redis_url: &str, channel: String) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn_manager = client.get_connection_manager().await?;
        Ok(Self { conn_manager, channel })
    }
}

#[cfg(feature = "redis-bus")]
#[async_trait]
impl EventBackend for RedisBackend {
    async fn publish(&self, event: &Event) -> anyhow::Result<()> {
        let json = serde_json::to_string(event)?;
        let mut conn = self.conn_manager.clone();
        
        redis::cmd("PUBLISH")
            .arg(&self.channel)
            .arg(json)
            .query_async(&mut conn)
            .await?;
            
        tracing::debug!(channel = %self.channel, "Published event to Redis");
        Ok(())
    }

    async fn subscribe<H>(&self, handler: H) -> anyhow::Result<SubscriptionHandle>
    where
        H: EventHandler + 'static,
    {
        use redis::AsyncCommands;
        
        let client = redis::Client::open(
            std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1".into())
        )?;
        let mut pubsub = client.get_async_pubsub().await?;
        pubsub.subscribe(&self.channel).await?;
        
        let task = tokio::spawn(async move {
            let mut stream = pubsub.on_message();
            while let Some(msg) = stream.next().await {
                if let Ok(payload) = msg.get_payload::<String>() {
                    if let Ok(event) = serde_json::from_str::<Event>(&payload) {
                        if let Err(e) = handler.handle(event).await {
                            tracing::warn!(error = %e, "Handler failed");
                        }
                    }
                }
            }
        });
        
        Ok(SubscriptionHandle { task, id: None })
    }
}
```

5. **Refatorar EventBus** em `src/events.rs`:

```rust
pub struct EventBus {
    backend: Arc<dyn EventBackend>,
    grouped_events: Arc<Mutex<HashMap<String, Vec<Event>>>>,
    interceptors: Arc<Mutex<Vec<Arc<dyn EventInterceptor>>>>,
}

impl EventBus {
    /// Cria event bus baseado em variável de ambiente BUS_DRIVER
    pub fn from_env() -> Self {
        let driver = std::env::var("BUS_DRIVER").unwrap_or_else(|_| "local".into());
        
        let backend: Arc<dyn EventBackend> = match driver.as_str() {
            #[cfg(feature = "redis-bus")]
            "redis" => {
                let url = std::env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://127.0.0.1".into());
                let channel = std::env::var("REDIS_CHANNEL")
                    .unwrap_or_else(|_| "medusa:events".into());
                
                match RedisBackend::new(&url, channel).await {
                    Ok(backend) => {
                        tracing::info!("EventBus initialized with Redis backend");
                        Arc::new(backend)
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "Failed to initialize Redis backend, falling back to Local"
                        );
                        Arc::new(LocalBackend::new())
                    }
                }
            }
            #[cfg(not(feature = "redis-bus"))]
            "redis" => {
                tracing::warn!(
                    "Redis driver requested but not compiled! Compile with --features redis-bus. Falling back to Local."
                );
                Arc::new(LocalBackend::new())
            }
            _ => {
                tracing::info!("EventBus initialized with Local backend");
                Arc::new(LocalBackend::new())
            }
        };
        
        Self {
            backend,
            grouped_events: Arc::new(Mutex::new(HashMap::new())),
            interceptors: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub async fn publish(&self, mut event: Event) -> anyhow::Result<()> {
        self.run_interceptors(&mut event).await?;
        self.backend.publish(&event).await
    }
    
    pub async fn subscribe<H>(&self, handler: H) -> anyhow::Result<SubscriptionHandle>
    where
        H: EventHandler + 'static,
    {
        self.backend.subscribe(handler).await
    }
}
```

**Testes com feature flags**:

```rust
#[tokio::test]
async fn test_local_publish_subscribe() {
    std::env::set_var("BUS_DRIVER", "local");
    // ... teste existente
}

#[tokio::test]
#[cfg(feature = "redis-bus")]
#[ignore] // Requer Redis rodando
async fn test_redis_publish_subscribe() {
    std::env::set_var("BUS_DRIVER", "redis");
    std::env::set_var("REDIS_URL", "redis://127.0.0.1");
    // ... teste
}
```

**✅ Vantagens desta abordagem**:
- Compila sem Redis por padrão
- Código limpo com trait abstrato
- Feature flags previnem linking de deps não usadas
- Fallback gracioso se Redis falhar

---

### Task 1.2: Implementar Event Grouping

**Referência MedusaJS**: 
- Método `groupOrEmitEvent()` (linha ~80 de `event-bus-local.ts`)
- Método `releaseGroupedEvents()` (linha ~130)
- Método `clearGroupedEvents()` (linha ~160)

**Solução**:

```rust
// Já existe: grouped_events: Arc<Mutex<HashMap<String, Vec<Event>>>>

impl EventBus {
    /// Acumula evento em um grupo (não emite ainda).
    /// Usado em transações distribuídas.
    pub async fn emit_grouped(&self, group_id: &str, event: Event) -> anyhow::Result<()> {
        let mut groups = self.grouped_events.lock().await;
        groups.entry(group_id.to_string())
            .or_insert_with(Vec::new)
            .push(event);
        tracing::debug!(group_id = %group_id, "Event queued in group");
        Ok(())
    }

    /// Libera todos os eventos de um grupo.
    /// Seguindo MedusaJS: chama callInterceptors antes de emitir.
    pub async fn release_grouped(&self, group_id: &str) -> anyhow::Result<()> {
        let events = {
            let mut groups = self.grouped_events.lock().await;
            groups.remove(group_id).unwrap_or_default()
        };
        
        tracing::info!(group_id = %group_id, count = events.len(), "Releasing grouped events");
        
        for mut event in events {
            // Chama interceptors antes de emitir (ver MedusaJS linha 145)
            self.run_interceptors(&mut event).await?;
            self.publish(event).await?;
        }
        Ok(())
    }

    /// Descarta todos os eventos de um grupo.
    pub async fn clear_grouped(&self, group_id: &str, event_names: Option<Vec<String>>) {
        let mut groups = self.grouped_events.lock().await;
        
        if let Some(names) = event_names {
            // Filtro seletivo (ver MedusaJS linha 166)
            if let Some(events) = groups.get_mut(group_id) {
                events.retain(|e| {
                    let event_name = format!("{:?}", e);
                    !names.iter().any(|n| event_name.contains(n))
                });
            }
        } else {
            groups.remove(group_id);
        }
        
        tracing::debug!(group_id = %group_id, "Grouped events cleared");
    }
}
```

**Uso típico** (adicionar exemplo em rotas):
```rust
// src/api/store/carts.rs - no handler de complete_cart
async fn complete_cart_with_transaction(
    State(state): State<AppState>,
    Path(cart_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let tx_id = Uuid::new_v4().to_string();
    let bus = state.event_bus();
    
    // Acumula eventos ao invés de emitir
    bus.emit_grouped(&tx_id, Event::CartCompleted(/* ... */)).await?;
    bus.emit_grouped(&tx_id, Event::OrderPlaced(/* ... */)).await?;
    
    // Tenta gravar no banco
    let result = sqlx::query!("INSERT INTO orders ...").execute(&state.db).await;
    
    if result.is_ok() {
        bus.release_grouped(&tx_id).await?;  // ✅ Sucesso: emite tudo
    } else {
        bus.clear_grouped(&tx_id, None).await;  // ❌ Falha: descarta
    }
    
    // ...
}
```

**Testes**:
```rust
#[tokio::test]
async fn test_event_grouping_release() {
    let bus = EventBus::new();
    let counter = Arc::new(AtomicU32::new(0));
    let c = counter.clone();
    
    bus.subscribe(move |_: Event| {
        let c = c.clone();
        async move { c.fetch_add(1, Ordering::SeqCst); Ok(()) }
    }).await.unwrap();
    
    bus.emit_grouped("tx1", Event::OrderPlaced(/* ... */)).await.unwrap();
    bus.emit_grouped("tx1", Event::PaymentCaptured(/* ... */)).await.unwrap();
    
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 0);  // Nada emitido
    
    bus.release_grouped("tx1").await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 2);  // Agora sim!
}

#[tokio::test]
async fn test_event_grouping_clear() {
    let bus = EventBus::new();
    let counter = Arc::new(AtomicU32::new(0));
    let c = counter.clone();
    
    bus.subscribe(move |_: Event| {
        let c = c.clone();
        async move { c.fetch_add(1, Ordering::SeqCst); Ok(()) }
    }).await.unwrap();
    
    bus.emit_grouped("tx2", Event::OrderPlaced(/* ... */)).await.unwrap();
    bus.clear_grouped("tx2", None).await;  // Descarta
    bus.release_grouped("tx2").await.unwrap();  // Não tem nada para liberar
    
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 0);  // Nada foi emitido
}
```

---

### Task 1.3: Implementar Unsubscribe

**Referência MedusaJS**: Método `unsubscribe()` (linha ~210 de `event-bus-local.ts`)

**Solução**:

```rust
pub struct SubscriptionHandle {
    task: tokio::task::JoinHandle<()>,
    id: Option<String>,
}

impl SubscriptionHandle {
    pub fn cancel(self) {
        self.task.abort();
        tracing::debug!(id = ?self.id, "Subscription cancelled");
    }
    
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}

impl EventBus {
    /// Subscribe com optional subscriber ID (para unsubscribe seletivo).
    pub async fn subscribe_with_id<H>(
        &self,
        handler: H,
        subscriber_id: Option<String>,
    ) -> anyhow::Result<SubscriptionHandle>
    where
        H: EventHandler + Send + 'static,
    {
        match &*self.backend {
            BusBackend::Local(tx) => {
                let mut rx = tx.lock().await.subscribe();
                let id_clone = subscriber_id.clone();
                
                let task = tokio::spawn(async move {
                    loop {
                        match rx.recv().await {
                            Ok(event) => {
                                if let Err(e) = handler.handle(event).await {
                                    tracing::warn!(
                                        subscriber_id = ?id_clone,
                                        error = %e,
                                        "Handler error"
                                    );
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(n)) => {
                                tracing::warn!(skipped = n, "Subscriber lagged");
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                        }
                    }
                });
                
                Ok(SubscriptionHandle { task, id: subscriber_id })
            }
            BusBackend::Redis(driver) => {
                driver.subscribe(handler).await
            }
            _ => Err(anyhow::anyhow!("Unimplemented backend")),
        }
    }
    
    // Backward compatibility
    pub async fn subscribe<H>(&self, handler: H) -> anyhow::Result<SubscriptionHandle>
    where
        H: EventHandler + Send + 'static,
    {
        self.subscribe_with_id(handler, None).await
    }
}
```

**Testes**:
```rust
#[tokio::test]
async fn test_unsubscribe() {
    let bus = EventBus::new();
    let counter = Arc::new(AtomicU32::new(0));
    let c = counter.clone();
    
    let handle = bus.subscribe_with_id(
        move |_: Event| {
            let c = c.clone();
            async move { c.fetch_add(1, Ordering::SeqCst); Ok(()) }
        },
        Some("test_subscriber".into()),
    ).await.unwrap();
    
    bus.publish(Event::OrderPlaced(/* ... */)).await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    
    handle.cancel();  // Unsubscribe
    
    bus.publish(Event::OrderPlaced(/* ... */)).await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1);  // Não incrementou!
}
```

---

### Task 1.4: Implementar Interceptors

**Referência MedusaJS**: Método `callInterceptors()` usado antes de emitir

**Solução**:

```rust
#[async_trait]
pub trait EventInterceptor: Send + Sync {
    async fn intercept(&self, event: &mut Event) -> anyhow::Result<()>;
}

pub struct EventBus {
    // ... campos existentes ...
    interceptors: Arc<Mutex<Vec<Arc<dyn EventInterceptor>>>>,
}

impl EventBus {
    pub async fn add_interceptor(&self, interceptor: Arc<dyn EventInterceptor>) {
        self.interceptors.lock().await.push(interceptor);
        tracing::debug!("Interceptor registered");
    }

    async fn run_interceptors(&self, event: &mut Event) -> anyhow::Result<()> {
        for interceptor in self.interceptors.lock().await.iter() {
            interceptor.intercept(event).await?;
        }
        Ok(())
    }

    pub async fn publish(&self, mut event: Event) -> anyhow::Result<()> {
        // Chama interceptors ANTES de publicar
        self.run_interceptors(&mut event).await?;
        
        match &*self.backend {
            BusBackend::Local(tx) => {
                let _ = tx.lock().await.send(event);
                Ok(())
            }
            BusBackend::Redis(driver) => {
                driver.publish(&event).await
            }
            _ => Err(anyhow::anyhow!("Unimplemented")),
        }
    }
}
```

**Exemplo de interceptor** (adicionar em `src/events.rs` ou novo arquivo):
```rust
/// Audit interceptor que loga todos os eventos no banco.
pub struct AuditInterceptor {
    db: sqlx::PgPool,
}

#[async_trait]
impl EventInterceptor for AuditInterceptor {
    async fn intercept(&self, event: &mut Event) -> anyhow::Result<()> {
        let json = serde_json::to_value(event)?;
        let event_type = format!("{:?}", event);
        
        sqlx::query!(
            "INSERT INTO event_audit (event_type, payload, created_at) VALUES ($1, $2, NOW())",
            event_type,
            json
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
}
```

---

## PARTE 2: Workflows/Sagas — Adicionar Persistência

### 📚 Código de Referência Obrigatório

**Leia e analise**:

```
vendor/medusa_js/packages/modules/workflow-engine-inmemory/src/services/workflow-orchestrator.ts
vendor/medusa_js/packages/modules/workflow-engine-inmemory/src/utils/in-memory-storage.ts
```

**Foque em**:
- Interface `DistributedTransactionStorage`
- Estado de workflows (`TransactionState`)
- Persistência de steps individuais
- Sistema de retry e recovery

---

### Task 2.1: Criar Migração de Persistência

**Arquivo**: `migrations/20260309000001_workflow_persistence.sql`

```sql
-- Estados possíveis baseados no MedusaJS TransactionState
CREATE TYPE workflow_state AS ENUM (
    'not_started',
    'invoking',
    'waiting_to_compensate',
    'compensating',
    'done',
    'reverted',
    'failed'
);

CREATE TYPE step_state AS ENUM (
    'idle',
    'invoking',
    'waiting_to_compensate',
    'compensating',
    'done',
    'reverted',
    'failed',
    'dormant',
    'skipped',
    'timeout'
);

-- Execuções de workflows
CREATE TABLE workflow_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id TEXT NOT NULL UNIQUE,  -- idempotency key
    workflow_name TEXT NOT NULL,
    state workflow_state NOT NULL DEFAULT 'not_started',
    context JSONB NOT NULL DEFAULT '{}',
    current_step TEXT,
    error TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workflow_executions_name ON workflow_executions(workflow_name);
CREATE INDEX idx_workflow_executions_state ON workflow_executions(state);
CREATE INDEX idx_workflow_executions_transaction_id ON workflow_executions(transaction_id);

-- Passos individuais
CREATE TABLE workflow_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    step_name TEXT NOT NULL,
    step_depth INTEGER NOT NULL DEFAULT 0,
    state step_state NOT NULL DEFAULT 'idle',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    invoke_result JSONB,
    compensate_result JSONB,
    error TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    UNIQUE(execution_id, step_name)
);

CREATE INDEX idx_workflow_steps_execution ON workflow_steps(execution_id);
CREATE INDEX idx_workflow_steps_state ON workflow_steps(state);

-- Tabela de auditoria (opcional, mas recomendado)
CREATE TABLE event_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_event_audit_type ON event_audit(event_type);
CREATE INDEX idx_event_audit_created ON event_audit(created_at);
```

---

### Task 2.2: Implementar Trait SagaStorage

**Arquivo**: `src/sagas.rs` (adicionar ao final)

```rust
#[async_trait]
pub trait SagaStorage: Send + Sync {
    /// Cria nova execução, retorna UUID.
    async fn create_execution(
        &self,
        transaction_id: &str,
        name: &str,
        ctx: &SagaContext,
    ) -> anyhow::Result<uuid::Uuid>;
    
    /// Atualiza estado da execução.
    async fn update_state(
        &self,
        id: uuid::Uuid,
        state: &str,
        current_step: Option<&str>,
        error: Option<&str>,
    ) -> anyhow::Result<()>;
    
    /// Salva estado de um step.
    async fn save_step(
        &self,
        exec_id: uuid::Uuid,
        step_name: &str,
        state: &str,
        result: Option<Value>,
        error: Option<&str>,
    ) -> anyhow::Result<()>;
    
    /// Carrega execução por transaction_id (para idempotency).
    async fn load_by_transaction_id(
        &self,
        transaction_id: &str,
    ) -> anyhow::Result<Option<(uuid::Uuid, String, SagaContext, String)>>;
    
    /// Incrementa retry count.
    async fn increment_retry(&self, id: uuid::Uuid) -> anyhow::Result<i32>;
}

/// Implementação PostgreSQL do storage.
pub struct PostgresSagaStorage {
    pool: sqlx::PgPool,
}

impl PostgresSagaStorage {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SagaStorage for PostgresSagaStorage {
    async fn create_execution(
        &self,
        transaction_id: &str,
        name: &str,
        ctx: &SagaContext,
    ) -> anyhow::Result<uuid::Uuid> {
        let ctx_json = serde_json::to_value(&ctx.data)?;
        
        let rec = sqlx::query!(
            r#"
            INSERT INTO workflow_executions (transaction_id, workflow_name, state, context)
            VALUES ($1, $2, 'not_started', $3)
            RETURNING id
            "#,
            transaction_id,
            name,
            ctx_json
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(rec.id)
    }
    
    async fn update_state(
        &self,
        id: uuid::Uuid,
        state: &str,
        current_step: Option<&str>,
        error: Option<&str>,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE workflow_executions
            SET state = $1::workflow_state,
                current_step = $2,
                error = $3,
                updated_at = NOW()
            WHERE id = $4
            "#,
            state,
            current_step,
            error,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    
    async fn save_step(
        &self,
        exec_id: uuid::Uuid,
        step_name: &str,
        state: &str,
        result: Option<Value>,
        error: Option<&str>,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO workflow_steps (execution_id, step_name, state, invoke_result, error, started_at)
            VALUES ($1, $2, $3::step_state, $4, $5, NOW())
            ON CONFLICT (execution_id, step_name)
            DO UPDATE SET
                state = EXCLUDED.state,
                invoke_result = EXCLUDED.invoke_result,
                error = EXCLUDED.error,
                completed_at = CASE WHEN EXCLUDED.state IN ('done', 'failed', 'reverted') THEN NOW() ELSE workflow_steps.completed_at END,
                attempts = workflow_steps.attempts + 1
            "#,
            exec_id,
            step_name,
            state,
            result,
            error
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    
    async fn load_by_transaction_id(
        &self,
        transaction_id: &str,
    ) -> anyhow::Result<Option<(uuid::Uuid, String, SagaContext, String)>> {
        let rec = sqlx::query!(
            r#"
            SELECT id, workflow_name, context, state::TEXT as "state!"
            FROM workflow_executions
            WHERE transaction_id = $1
            "#,
            transaction_id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(r) = rec {
            let data: HashMap<String, Value> = serde_json::from_value(r.context)?;
            let ctx = SagaContext::with_data(data);
            Ok(Some((r.id, r.workflow_name, ctx, r.state)))
        } else {
            Ok(None)
        }
    }
    
    async fn increment_retry(&self, id: uuid::Uuid) -> anyhow::Result<i32> {
        let rec = sqlx::query!(
            r#"
            UPDATE workflow_executions
            SET retry_count = retry_count + 1,
                updated_at = NOW()
            WHERE id = $1
            RETURNING retry_count
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(rec.retry_count)
    }
}
```

---

### Task 2.3: Adicionar `run_persistent()` à Saga

**Arquivo**: `src/sagas.rs` (adicionar método)

```rust
impl Saga {
    /// Executa saga com persistência e idempotency.
    /// 
    /// **Persistência**: Sempre usa PostgreSQL (workflow_executions table)
    /// **Coordenação distribuída**: Requer feature `redis-workflows` para:
    ///   - Locks distribuídos (evitar execução duplicada)
    ///   - Fila de retry com Redis Streams
    ///   - Scheduler distribuído
    /// 
    /// Modo local é suficiente para single-instance!
    pub async fn run_persistent(
        &self,
        transaction_id: String,
        initial_ctx: SagaContext,
        storage: &dyn SagaStorage,
    ) -> anyhow::Result<SagaOutcome> {
        // Verificar se já existe (idempotency)
        if let Some((exec_id, _, saved_ctx, state)) = storage.load_by_transaction_id(&transaction_id).await? {
            tracing::info!(
                transaction_id = %transaction_id,
                execution_id = %exec_id,
                state = %state,
                "Workflow already exists (idempotent)"
            );
            
            if state == "done" {
                return Ok(SagaOutcome::Completed(saved_ctx));
            } else if state == "reverted" || state == "failed" {
                return Ok(SagaOutcome::Compensated {
                    failed_step: "unknown".into(),
                    error: anyhow::anyhow!("Previously failed"),
                    context: saved_ctx,
                });
            }
            // Se state == "invoking", continuar...
        }
        
        // Criar nova execução
        let exec_id = storage.create_execution(&transaction_id, &self.name, &initial_ctx).await?;
        storage.update_state(exec_id, "invoking", None, None).await?;
        
        let mut ctx = initial_ctx;
        let mut completed: Vec<&StepDef> = Vec::new();
        
        for step in &self.steps {
            storage.update_state(exec_id, "invoking", Some(&step.name), None).await?;
            storage.save_step(exec_id, &step.name, "invoking", None, None).await?;
            
            tracing::debug!(
                transaction_id = %transaction_id,
                saga = %self.name,
                step = %step.name,
                "Executing step"
            );
            
            match (step.action)(ctx.clone()).await {
                Ok((new_ctx, result)) => {
                    ctx = new_ctx;
                    storage.save_step(
                        exec_id,
                        &step.name,
                        "done",
                        result.output.clone(),
                        None
                    ).await?;
                    
                    if let Some(output) = result.output {
                        ctx.set(step.name.clone(), output);
                    }
                    completed.push(step);
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    tracing::warn!(
                        transaction_id = %transaction_id,
                        saga = %self.name,
                        step = %step.name,
                        error = %e,
                        "Step failed — running compensations"
                    );
                    
                    storage.save_step(exec_id, &step.name, "failed", None, Some(&err_msg)).await?;
                    storage.update_state(exec_id, "compensating", Some(&step.name), Some(&err_msg)).await?;
                    
                    // Compensações em ordem reversa
                    for prev in completed.iter().rev() {
                        if let Some(comp) = &prev.compensation {
                            storage.save_step(exec_id, &prev.name, "compensating", None, None).await?;
                            
                            match (comp)(ctx.clone()).await {
                                Ok(new_ctx) => {
                                    ctx = new_ctx;
                                    storage.save_step(exec_id, &prev.name, "reverted", None, None).await?;
                                }
                                Err(ce) => {
                                    let ce_msg = ce.to_string();
                                    storage.save_step(exec_id, &prev.name, "failed", None, Some(&ce_msg)).await?;
                                    tracing::error!(
                                        saga = %self.name,
                                        step = %prev.name,
                                        error = %ce,
                                        "Compensation failed"
                                    );
                                }
                            }
                        }
                    }
                    
                    storage.update_state(exec_id, "reverted", None, Some(&err_msg)).await?;
                    return Ok(SagaOutcome::Compensated {
                        failed_step: step.name.clone(),
                        error: e,
                        context: ctx,
                    });
                }
            }
        }
        
        storage.update_state(exec_id, "done", None, None).await?;
        tracing::info!(
            transaction_id = %transaction_id,
            saga = %self.name,
            "Saga completed successfully"
        );
        
        Ok(SagaOutcome::Completed(ctx))
    }
}
```

---

## PARTE 3: Integração EventBus ↔ Workflows

### Task 3.1: WorkflowEventListener

**Arquivo**: `src/workflow_listener.rs` (novo)

```rust
use crate::events::{Event, EventBus};
use crate::sagas::{SagaContext, SagaStorage, WorkflowEngine};
use std::sync::Arc;
use serde_json::json;

pub struct WorkflowEventListener {
    engine: Arc<WorkflowEngine>,
    storage: Arc<dyn SagaStorage>,
}

impl WorkflowEventListener {
    pub fn new(engine: Arc<WorkflowEngine>, storage: Arc<dyn SagaStorage>) -> Self {
        Self { engine, storage }
    }

    pub async fn start(self, event_bus: &EventBus) -> anyhow::Result<()> {
        event_bus.subscribe(move |event: Event| {
            let engine = self.engine.clone();
            let storage = self.storage.clone();
            
            async move {
                match event {
                    Event::OrderPlaced(data) => {
                        let mut ctx = SagaContext::new();
                        ctx.set("order_id", json!(data.order_id.to_string()));
                        ctx.set("customer_id", json!(data.customer_id.to_string()));
                        ctx.set("total", json!(data.total));
                        ctx.set("email", json!(data.email));
                        
                        let tx_id = format!("order-fulfillment-{}", data.order_id);
                        
                        tracing::info!(
                            order_id = %data.order_id,
                            transaction_id = %tx_id,
                            "Triggering order_fulfillment workflow"
                        );
                        
                        // Executar com persistência
                        if let Some(saga) = engine.get("order_fulfillment") {
                            match saga.run_persistent(tx_id, ctx, storage.as_ref()).await {
                                Ok(outcome) => {
                                    if outcome.is_completed() {
                                        tracing::info!(order_id = %data.order_id, "Order fulfillment completed");
                                    } else {
                                        tracing::warn!(order_id = %data.order_id, "Order fulfillment compensated");
                                    }
                                }
                                Err(e) => {
                                    tracing::error!(order_id = %data.order_id, error = %e, "Order fulfillment failed");
                                }
                            }
                        }
                    }
                    
                    Event::PaymentCaptured(data) => {
                        let mut ctx = SagaContext::new();
                        ctx.set("payment_id", json!(data.payment_id.to_string()));
                        if let Some(amt) = data.amount {
                            ctx.set("amount", json!(amt));
                        }
                        
                        let tx_id = format!("payment-confirm-{}", data.payment_id);
                        
                        tracing::info!(
                            payment_id = %data.payment_id,
                            "Triggering payment_confirmation workflow"
                        );
                        
                        if let Some(saga) = engine.get("payment_confirmation") {
                            let _ = saga.run_persistent(tx_id, ctx, storage.as_ref()).await;
                        }
                    }
                    
                    _ => {}
                }
                Ok(())
            }
        }).await?;
        
        Ok(())
    }
}
```

**Adicionar método `get()` ao WorkflowEngine**:
```rust
// src/sagas.rs
impl WorkflowEngine {
    pub fn get(&self, name: &str) -> Option<&Saga> {
        self.sagas.get(name)
    }
}
```

**Registrar no main.rs**:
```rust
// src/main.rs
mod workflow_listener;
use workflow_listener::WorkflowEventListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ... setup existente ...
    
    let workflow_engine = Arc::new(WorkflowEngine::new());
    let saga_storage = Arc::new(PostgresSagaStorage::new(pool.clone()));
    
    // Registrar sagas
    workflow_engine.register(create_order_fulfillment_saga());
    workflow_engine.register(create_payment_confirmation_saga());
    
    // Iniciar listener
    let listener = WorkflowEventListener::new(workflow_engine.clone(), saga_storage.clone());
    listener.start(&event_bus).await?;
    
    // ... iniciar servidor ...
}

// Exemplo de saga order_fulfillment
fn create_order_fulfillment_saga() -> Saga {
    Saga::builder("order_fulfillment")
        .step(
            "reserve_inventory",
            |mut ctx| async move {
                let order_id = ctx.get("order_id").and_then(|v| v.as_str()).unwrap();
                tracing::info!(order_id = %order_id, "Reserving inventory");
                
                // TODO: chamar serviço de inventory
                
                ctx.set("reservation_id", json!("res-123"));
                Ok((ctx, StepResult::done()))
            },
            Some(|mut ctx| async move {
                let res_id = ctx.get("reservation_id").and_then(|v| v.as_str()).unwrap();
                tracing::info!(reservation_id = %res_id, "Releasing inventory");
                
                // TODO: chamar serviço para liberar
                
                Ok(ctx)
            }),
        )
        .step(
            "charge_payment",
            |mut ctx| async move {
                let order_id = ctx.get("order_id").and_then(|v| v.as_str()).unwrap();
                tracing::info!(order_id = %order_id, "Charging payment");
                
                // TODO: chamar plugin de pagamento
                
                ctx.set("charge_id", json!("chg-456"));
                Ok((ctx, StepResult::done()))
            },
            Some(|mut ctx| async move {
                let charge_id = ctx.get("charge_id").and_then(|v| v.as_str()).unwrap();
                tracing::info!(charge_id = %charge_id, "Refunding payment");
                
                // TODO: estornar
                
                Ok(ctx)
            }),
        )
        .step(
            "confirm_order",
            |ctx| async move {
                let order_id = ctx.get("order_id").and_then(|v| v.as_str()).unwrap();
                tracing::info!(order_id = %order_id, "Order confirmed");
                
                // TODO: atualizar status no banco
                
                Ok((ctx, StepResult::done()))
            },
            None,
        )
        .build()
}
```

---

## PARTE 4: Scheduler de Workflows (Prioridade P1)

### Task 4.1: Workflow Scheduler

**Arquivo**: `src/workflow_scheduler.rs` (novo)

```rust
use crate::sagas::{SagaStorage, WorkflowEngine};
use std::sync::Arc;
use std::time::Duration;

pub struct WorkflowScheduler {
    engine: Arc<WorkflowEngine>,
    storage: Arc<dyn SagaStorage>,
    interval_secs: u64,
}

impl WorkflowScheduler {
    pub fn new(
        engine: Arc<WorkflowEngine>,
        storage: Arc<dyn SagaStorage>,
        interval_secs: u64,
    ) -> Self {
        Self { engine, storage, interval_secs }
    }
    
    pub async fn start(self) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(self.interval_secs));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = self.process_stuck_workflows().await {
                    tracing::error!(error = %e, "Scheduler: process_stuck_workflows failed");
                }
                
                if let Err(e) = self.retry_failed_workflows().await {
                    tracing::error!(error = %e, "Scheduler: retry_failed_workflows failed");
                }
            }
        });
    }
    
    async fn process_stuck_workflows(&self) -> anyhow::Result<()> {
        // SELECT workflows com state='invoking' e updated_at > 10 min atrás
        // Para cada um, tentar retomar ou marcar como failed
        
        tracing::debug!("Checking for stuck workflows...");
        // TODO: implementar query + lógica de retry
        Ok(())
    }
    
    async fn retry_failed_workflows(&self) -> anyhow::Result<()> {
        // SELECT workflows com state='failed' e retry_count < max_retries
        // Para cada um, tentar executar novamente
        
        tracing::debug!("Checking for failed workflows to retry...");
        // TODO: implementar query + lógica de retry
        Ok(())
    }
}
```

---

## CRITÉRIOS DE ACEITAÇÃO FINAIS

### Compilação e Testes
- [ ] `cargo build --workspace` compila sem erros
- [ ] `cargo test --workspace` passa com **100% dos testes**
- [ ] Testes específicos:
  - [ ] `cargo test event_bus_tests::test_redis*` (com Redis rodando)
  - [ ] `cargo test event_bus_tests::test_event_grouping*`
  - [ ] `cargo test event_bus_tests::test_unsubscribe`
  - [ ] Testes de saga com persistência
  - [ ] Testes de integração EventBus ↔ Workflows

### Feature-Parity
- [ ] Event grouping funciona conforme MedusaJS
- [ ] Redis driver publica e consome eventos de verdade
- [ ] Unsubscribe cancela subscribers corretamente
- [ ] Interceptors são chamados antes de emitir eventos
- [ ] Workflows persistem estado no PostgreSQL
- [ ] Idempotency funciona (mesmo transaction_id não duplica)
- [ ] Workflows reagem automaticamente a eventos
- [ ] Compensations são executadas e persistidas

### Documentação
- [ ] `README.md` atualizado com:
  - [ ] Variáveis de ambiente (BUS_DRIVER, REDIS_URL, etc.)
  - [ ] Exemplos de uso de event grouping
  - [ ] Exemplos de criação de workflows
- [ ] `IMPLEMENTATION_PLAN.md` atualizado com status de cada feature
- [ ] Comentários no código explicando lógica complexa

### Adaptações Rust Idiomáticas
- [ ] Usar `Arc<Mutex<>>` ao invés de locks globais
- [ ] Preferir `tokio::sync::broadcast` para local pubsub
- [ ] Usar `async_trait` para traits assíncronos
- [ ] Error handling com `anyhow::Result`
- [ ] Logs com `tracing::*!` macros
- [ ] Testes com `#[tokio::test]`

---

## NOTAS IMPORTANTES

### Sobre Moka e Cache vs Event Bus

**IMPORTANTE**: Moka e Redis têm propósitos diferentes:

- **Moka**: Cache in-memory local. Já está implementado no projeto para cache de dados (consultas, sessões, etc.). Continua sendo usado independentemente do modo de Event Bus.

- **Redis (Event Bus)**: Broker de mensagens distribuído. Usado APENAS quando você precisa que eventos atravessem múltiplos processos/containers.

**Convivência**: Ambos podem e devem coexistir no mesmo sistema:

```rust
// AppState contém ambos
pub struct AppState {
    pub cache: Arc<Cache<String, Value>>,  // Moka - sempre presente
    pub event_bus: EventBus,                // Driver configurável
    pub db: PgPool,
    // ...
}
```

**Quando usar cada um**:
- Use **Moka** para: resultados de queries, sessões, configurações, rate limiting local
- Use **Event Bus Local** para: desenvolvimento, aplicação monolítica
- Use **Event Bus Redis** para: múltiplas instâncias, microserviços, sistemas distribuídos

Eles não competem, são complementares!

### Sobre Prioridades
- **P0 (Obrigatório)**: Tasks 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 3.1
- **P1 (Importante)**: Task 1.4, Task 4.1
- **P2 (Nice-to-have)**: Sub-workflows, SQS driver

---

## CHECKLIST DE PROGRESSO

### FASE 1: Abstrações (Obrigatória)
- [ ] Trait `EventBackend` com `publish()` e `subscribe()`
- [ ] Trait `SagaStorage` com persistência Postgres
- [ ] Event grouping, unsubscribe, interceptors
- [ ] Migração SQL de workflow_executions

### FASE 2: Drivers Redis (Opcional)
- [ ] `RedisBackend` com feature flag `redis-bus`
- [ ] Testes com `#[cfg(feature = "redis-bus")]`
- [ ] Fallback gracioso para Local

### FASE 3: Integração EventBus ↔ Workflows
- [ ] `WorkflowEventListener` 
- [ ] Sagas de exemplo (order_fulfillment, etc)
- [ ] Testes de integração

### FASE 4: Coordenação Distribuída Avançada (Opcional)
- [ ] Locks distribuídos com Redis (feature `redis-workflows`)
- [ ] Scheduler com Redis Streams
- [ ] Retry automático distribuído

### Documentação
- [ ] README atualizado com feature flags
- [ ] Exemplos de cada modo de operação
- [ ] Tabela comparativa Local vs Híbrido vs Distribuído

---

## MATRIZ DE DECISÃO: Qual Modo Usar?

| Critério | Local | Híbrido (redis-bus) | Distribuído (distributed) |
|----------|-------|---------------------|---------------------------|
| **Instâncias** | 1 | 2+ | 2+ |
| **Eventos entre processos** | ❌ | ✅ | ✅ |
| **Workflows distribuídos** | ❌ | ❌ | ✅ |
| **Locks distribuídos** | ❌ | ❌ | ✅ |
| **Retry automático** | ⚠️ Manual | ⚠️ Manual | ✅ |
| **Complexidade** | Baixa | Média | Alta |
| **Deps externas** | Postgres | Postgres + Redis | Postgres + Redis |
| **Recomendado para** | Dev, MVPs | Produção pequena | Produção escalável |

---

**Estimativa por fase**:
- Fase 1 (abstrações): 2-3 dias
- Fase 2 (Redis opcional): 1-2 dias
- Fase 3 (integração): 1 dia
- Fase 4 (distribuído avançado): 2-3 dias

**Total**: 6-9 dias de desenvolvimento full-time

---

**Boa implementação! 🚀**
