# Prompts Sequenciais para Completar o Medusa Rust

> Use estes prompts um de cada vez para completar o projeto de forma incremental.
> Cada prompt é curto e focado em uma tarefa específica.

---

## ✅ CONCLUÍDO — EventBus & Sagas (Fase 6)

Implementado via `PROMPT_CORRECAO_GAPS_EVENTBUS_SAGAS.md`:

- ✅ `Cargo.toml`: features `local-bus`, `redis-bus`, `local-workflows`, `redis-workflows`, `distributed` + dep `redis = "0.24"` opcional
- ✅ `src/events.rs`: `subscribe()` retorna `SubscriptionHandle`; backend Redis real com `#[cfg(feature = "redis-bus")]`; `AuditInterceptor`; 6 testes unitários
- ✅ `src/sagas.rs`: `WorkflowEngine::execute_persistent()` e `execute_persistent_with_events()`
- ✅ `migrations/20260309000001_workflow_persistence.sql`: `workflow_executions`, `workflow_steps`, `event_audit`

---

## FASE 1: Completar Rotas Admin Faltantes

### 1.1 Customers (6 rotas faltando)
```
Implemente as rotas faltantes de customers admin:
- DELETE /admin/customers/{id}
- GET /admin/customers/{id}/addresses
- POST /admin/customers/{id}/addresses
- POST /admin/customers/{id}/addresses/{address_id}
- DELETE /admin/customers/{id}/addresses/{address_id}
- POST /admin/customers/{id}/customer-groups

Siga o padrão existente em src/api/admin/customers.rs
```

### 1.2 Draft Orders (1 rota faltando)
```
Implemente POST /admin/draft-orders/{id}/convert-to-order que converte um draft order em order real. Siga o padrão de src/api/admin/draft_orders.rs
```

### 1.3 Invites (3 rotas faltando)
```
Implemente as rotas faltantes de invites:
- POST /admin/invites/accept (aceitar pelo token)
- GET /admin/invites/{id}
- POST /admin/invites/{id}/resend

Siga o padrão de src/api/admin/invites.rs
```

### 1.4 Locales Admin (2 rotas)
```
Implemente as rotas de locales para admin:
- GET /admin/locales
- GET /admin/locales/{code}

Crie src/api/admin/locales.rs seguindo o padrão das outras rotas admin
```

### 1.5 Orders - Rotas Faltantes (10+ rotas)
```
Implemente as rotas faltantes de orders:
- POST /admin/orders/export
- GET /admin/orders/{id}/changes
- GET /admin/orders/{id}/credit-lines
- GET /admin/orders/{id}/fulfillments
- POST /admin/orders/{id}/fulfillments/{fid}/mark-as-delivered
- POST /admin/orders/{id}/fulfillments/{fid}/shipments
- GET /admin/orders/{id}/line-items
- GET /admin/orders/{id}/preview
- GET /admin/orders/{id}/shipping-options
- POST /admin/orders/{id}/transfer
- POST /admin/orders/{id}/transfer/cancel

Adicione em src/api/admin/orders.rs
```

### 1.6 Inventory Items Batch (2 rotas)
```
Implemente as rotas batch de inventory items:
- POST /admin/inventory-items/{id}/location-levels/batch
- POST /admin/inventory-items/location-levels/batch

Adicione em src/api/admin/inventory_items.rs
```

### 1.7 Product Categories CRUD (5 rotas)
```
Implemente CRUD completo de product categories admin:
- POST /admin/product-categories (create)
- GET /admin/product-categories/{id} (get)
- POST /admin/product-categories/{id} (update)
- DELETE /admin/product-categories/{id} (delete)
- POST /admin/product-categories/{id}/products (link)

Atualize src/api/admin/categories.rs
```

### 1.8 Product Variants Standalone
```
Implemente GET /admin/product-variants para listar todas as variantes independente do produto. Adicione em src/api/admin/products.rs
```

### 1.9 Products Batch/Import/Export (7 rotas)
```
Implemente as rotas batch de products:
- POST /admin/products/batch
- POST /admin/products/export
- POST /admin/products/import
- POST /admin/products/import/{transaction_id}/confirm
- POST /admin/products/{id}/variants/batch
- POST /admin/products/{id}/variants/inventory-items/batch
- POST /admin/products/{id}/variants/{vid}/images/batch
- GET /admin/products/{id}/variants/{vid}/inventory-items

Adicione em src/api/admin/products.rs
```

### 1.10 Price Lists - Add Prices
```
Implemente POST /admin/price-lists/{id}/prices para adicionar preços individuais a uma price list. Siga o padrão de src/api/admin/price_lists.rs
```

### 1.11 Promotions Rule Options (2 rotas)
```
Implemente as rotas de rule options para promotions:
- GET /admin/promotions/rule-attribute-options/{rule_type}
- GET /admin/promotions/rule-value-options/{rule_type}/{rule_attribute_id}

Adicione em src/api/admin/promotions.rs
```

### 1.12 Returns Action Routes (6 rotas)
```
Implemente as rotas de action para returns:
- POST /admin/returns/{id}/dismiss-items/{action_id}
- POST /admin/returns/{id}/receive-items/{action_id}
- POST /admin/returns/{id}/request-items
- POST /admin/returns/{id}/request-items/{action_id}
- POST /admin/returns/{id}/shipping-method/{action_id}

Adicione em src/api/admin/returns.rs
```

### 1.13 Shipping Options CRUD (4 rotas)
```
Implemente CRUD completo de shipping options:
- GET /admin/shipping-options/{id}
- POST /admin/shipping-options/{id}
- DELETE /admin/shipping-options/{id}
- POST /admin/shipping-options/{id}/rules/batch

Atualize src/api/admin/shipping_options.rs
```

### 1.14 Store Credit Accounts (6 rotas)
```
Implemente CRUD completo de store credit accounts admin:
- GET /admin/store-credit-accounts
- POST /admin/store-credit-accounts
- GET /admin/store-credit-accounts/{id}
- POST /admin/store-credit-accounts/{id}
- DELETE /admin/store-credit-accounts/{id}
- POST /admin/store-credit-accounts/{id}/transactions

Crie src/api/admin/store_credit_accounts.rs
```

### 1.15 Tax Rates CRUD (6 rotas)
```
Implemente CRUD completo de tax rates:
- POST /admin/tax-rates
- GET /admin/tax-rates/{id}
- POST /admin/tax-rates/{id}
- DELETE /admin/tax-rates/{id}
- POST /admin/tax-rates/{id}/rules
- DELETE /admin/tax-rates/{id}/rules/{rule_id}

Atualize src/api/admin/tax_rates.rs
```

### 1.16 Translations (7 rotas)
```
Implemente as rotas de translations:
- GET /admin/translations
- POST /admin/translations
- POST /admin/translations/batch
- GET /admin/translations/entities
- GET /admin/translations/settings
- POST /admin/translations/settings/batch

Crie src/api/admin/translations.rs
```

### 1.17 Gift Cards Orders
```
Implemente GET /admin/gift-cards/{id}/orders para listar orders relacionadas a um gift card. Adicione em src/api/admin/gift_cards.rs
```

### 1.18 API Keys Sales Channels
```
Implemente POST /admin/api-keys/{id}/sales-channels para vincular canais de vendas a uma API key. Adicione em src/api/admin/api_keys.rs
```

### 1.19 Payments Providers
```
Implemente GET /admin/payments/payment-providers para listar provedores de pagamento. Adicione em src/api/admin/payments.rs
```

---

## FASE 2: Testes para Rotas Novas

### 2.1 Testes Customers
```
Crie testes para as novas rotas de customers em tests/admin_routes_tests.rs:
- Test delete customer
- Test list/create/update/delete addresses
- Test link customer groups
```

### 2.2 Testes Gerais
```
Adicione testes para todas as rotas implementadas na fase 1: draft orders convert, invites, locales, orders extended, inventory batch, categories, variants, products batch, price lists prices, promotions rules, returns actions, shipping options, store credits, tax rates, translations, gift cards orders, api keys sales channels, payments providers. Use o padrão de tests/admin_routes_tests.rs
```

---

## FASE 3: Migrações SQL Faltantes

### 3.1 Tabela Translations
```
Crie a migração para tabela translations em migrations/:
- id, entity_type, entity_id, locale, field, value, created_at, updated_at
```

### 3.2 Tabela Store Credit Accounts
```
Crie a migração para store_credit_accounts em migrations/:
- id, customer_id, balance, currency_code, created_at, updated_at
E store_credit_transactions:
- id, account_id, amount, type, reference, created_at
```

### 3.3 Tabela Locales
```
Crie a migração para tabela locales em migrations/:
- code (PK), name, native_name, is_default, created_at
```

---

## FASE 4: Lógica de Negócio Real (Core)

### 4.1 Cart Completion Real
```
Implemente lógica real de cart completion em src/core/cart.rs:
- Validar estoque disponível
- Criar order a partir do cart
- Processar pagamento
- Atualizar inventory
- Enviar notificações
```

### 4.2 Inventory Management Real
```
Implemente lógica real de inventory em src/core/inventory.rs:
- Reservar estoque
- Liberar estoque
- Atualizar níveis por location
- Validar disponibilidade
- Suporte a backorder
```

### 4.3 Pricing Engine Real
```
Implemente engine de pricing em src/core/pricing.rs:
- Aplicar price lists
- Calcular descontos
- Suporte a moedas múltiplas
- Price preferences por região
```

### 4.4 Tax Calculation Real
```
Implemente cálculo de impostos em src/core/tax.rs:
- Calcular por região
- Aplicar tax rates
- Suporte a tax inclusive/exclusive
- Tax overrides
```

### 4.5 Returns Processing
```
Implemente processamento de devoluções em src/core/:
- Validar itens retornáveis
- Calcular reembolso
- Atualizar estoque
- Criar crédito ou reembolso
```

### 4.6 Fulfillment Processing
```
Implemente processamento de fulfillment em src/core/:
- Criar shipments
- Tracking integration
- Status updates
- Notificações
```

---

## FASE 5: Integrações Externas

### 5.1 Payment Provider Interface
```
Crie interface para payment providers em src/core/payments/:
- Trait PaymentProvider
- Implementação Stripe (stub)
- Implementação PayPal (stub)
- Capture/Refund/Cancel
```

### 5.2 Shipping Provider Interface
```
Crie interface para shipping providers em src/core/shipping/:
- Trait ShippingProvider
- Cálculo de frete
- Tracking
- Label generation (stub)
```

### 5.3 Email/Notification Provider
```
Crie interface para notifications em src/core/notifications/:
- Trait NotificationProvider
- Templates de email
- SMS (stub)
- Push (stub)
```

### 5.4 Webhook System
```
Implemente sistema de webhooks:
- Registro de endpoints
- Envio de eventos
- Retry logic
- Signature validation
```

---

## FASE 6: Frontend Admin Panel

### 6.1 Setup Frontend Project
```
Crie projeto frontend em /admin-panel usando React + Vite + TailwindCSS + React Query. Configure proxy para API em localhost:3000
```

### 6.2 Auth e Layout Base
```
Implemente autenticação e layout base do admin panel:
- Login page
- Sidebar navigation
- Header com user info
- Protected routes
```

### 6.3 Dashboard
```
Implemente dashboard do admin panel:
- Cards de métricas (orders, revenue, customers)
- Gráfico de vendas
- Últimos pedidos
- Alertas de estoque baixo
```

### 6.4 Products Management
```
Implemente tela de produtos:
- Lista com filtros e busca
- Criar/Editar produto
- Gerenciar variantes
- Gerenciar opções
- Upload de imagens
```

### 6.5 Orders Management
```
Implemente tela de pedidos:
- Lista com filtros e status
- Detalhes do pedido
- Fulfillment actions
- Refund actions
- Timeline de eventos
```

### 6.6 Customers Management
```
Implemente tela de clientes:
- Lista com busca
- Detalhes do cliente
- Histórico de pedidos
- Customer groups
- Endereços
```

### 6.7 Inventory Management
```
Implemente tela de inventário:
- Lista de inventory items
- Níveis por location
- Ajustes de estoque
- Reservations
```

### 6.8 Settings Pages
```
Implemente páginas de configuração:
- Regions
- Currencies
- Tax settings
- Shipping options
- Sales channels
- API keys
- Users/Invites
```

### 6.9 Discounts & Promotions
```
Implemente tela de descontos:
- Lista de promoções
- Criar/Editar promoção
- Rules builder
- Campaigns
```

### 6.10 Gift Cards
```
Implemente tela de gift cards:
- Lista de gift cards
- Criar gift card
- Histórico de uso
- Balance updates
```

---

## FASE 7: Store Frontend

### 7.1 Setup Store Project
```
Crie projeto frontend da loja em /storefront usando Next.js + TailwindCSS + React Query. Configure API routes para o backend.
```

### 7.2 Homepage e Navegação
```
Implemente homepage da loja:
- Hero section
- Produtos em destaque
- Categorias
- Header com cart/search/account
- Footer
```

### 7.3 Product Listing
```
Implemente listagem de produtos:
- Grid de produtos
- Filtros (categoria, preço, etc)
- Ordenação
- Paginação
- Quick view
```

### 7.4 Product Detail
```
Implemente página de produto:
- Galeria de imagens
- Seleção de variantes
- Add to cart
- Descrição
- Reviews (stub)
```

### 7.5 Cart Page
```
Implemente página do carrinho:
- Lista de itens
- Atualizar quantidade
- Remover item
- Aplicar cupom
- Subtotal/Total
- Checkout button
```

### 7.6 Checkout Flow
```
Implemente fluxo de checkout:
- Informações de contato
- Endereço de entrega
- Método de envio
- Pagamento
- Confirmação
```

### 7.7 Customer Account
```
Implemente área do cliente:
- Login/Register
- Meus pedidos
- Meus endereços
- Meus dados
- Wishlist (stub)
```

---

## FASE 8: DevOps e Deploy

### 8.1 Docker Compose Full
```
Atualize docker-compose.yml para incluir:
- API Rust
- PostgreSQL
- Redis (cache)
- MinIO (S3)
- Admin frontend
- Store frontend
- Nginx reverse proxy
```

### 8.2 CI/CD Pipeline
```
Crie GitHub Actions para:
- Lint e format check
- Cargo test
- Build Docker images
- Deploy to staging
```

### 8.3 Production Config
```
Crie configurações para produção:
- Environment variables
- SSL/TLS
- Rate limiting
- CORS settings
- Logging estruturado
```

---

## Dicas de Uso

1. **Execute um prompt por vez** - Aguarde a conclusão antes do próximo
2. **Verifique o código gerado** - Teste com `cargo test` após cada fase
3. **Atualize IMPLEMENTATION_PLAN.md** - Marque rotas como ✅ quando implementadas
4. **Commite frequentemente** - Um commit por prompt completado
5. **Priorize por dependência** - Fase 1 → Fase 2 → Fase 3 → etc.

---

## Contagem de Tarefas

| Fase | Prompts | Descrição |
|------|---------|-----------|
| 1 | 19 | Rotas admin faltantes |
| 2 | 2 | Testes |
| 3 | 3 | Migrações SQL |
| 4 | 6 | Lógica de negócio |
| 5 | 4 | Integrações |
| 6 | 10 | Frontend admin |
| 7 | 7 | Frontend store |
| 8 | 3 | DevOps |
| **Total** | **54** | Prompts para completar o projeto |

> Estimativa: ~2-3 horas por fase com AI assistance
