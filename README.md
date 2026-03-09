# MedusaRust 🦀

> **Port do [MedusaJS v2](https://medusajs.com/) para Rust** — API de e-commerce de alta performance construída com [Axum](https://github.com/tokio-rs/axum), [SQLx](https://github.com/launchbadge/sqlx) (PostgreSQL), [Moka](https://github.com/moka-rs/moka) (cache in-process) e MinIO/S3.

[![Rust](https://img.shields.io/badge/rust-1.82+-orange?logo=rust)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/axum-0.8-blue)](https://github.com/tokio-rs/axum)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## Índice

- [O que é MedusaRust?](#o-que-é-medusarust)
- [Comparativo com MedusaJS](#comparativo-com-medusajs)
- [Pré-requisitos](#pré-requisitos)
- [Início Rápido (Docker Compose)](#início-rápido-docker-compose)
- [Instalação Manual](#instalação-manual)
- [Variáveis de Ambiente](#variáveis-de-ambiente)
- [Autenticação](#autenticação)
- [Referência da API](#referência-da-api)
  - [Store API](#store-api)
  - [Admin API](#admin-api)
- [Funcionalidades Além de Rotas](#funcionalidades-além-de-rotas)
- [O que Falta em Comparação ao MedusaJS](#o-que-falta-em-comparação-ao-medusajs)
- [Testes](#testes)
- [Estrutura do Projeto](#estrutura-do-projeto)
- [Contribuindo](#contribuindo)

---

## O que é MedusaRust?

MedusaRust é uma reimplementação em Rust do back-end do [MedusaJS v2](https://medusajs.com/), um framework de comércio eletrônico headless open-source. O objetivo é oferecer a mesma API REST compatível com o Medusa JS, porém com as vantagens do ecossistema Rust:

- **Performance**: binário compilado, sem overhead de JIT, latência de milissegundos.
- **Segurança de memória**: garantias em tempo de compilação, sem race conditions.
- **Imagem Docker pequena**: ~50 MB (vs centenas de MB do Node.js).
- **Baixo consumo de recursos**: ideal para ambientes com restrições de memória/CPU.

### Cobertura de rotas vs MedusaJS v2

| Categoria        | MedusaJS | MedusaRust | Cobertura |
|-----------------|----------|------------|-----------|
| Store Routes    | 54       | ~70        | ~100%     |
| Admin Routes    | 245      | ~245       | ~98%      |
| **Total**       | **299**  | **315**    | **~97%**  |

> Rotas marcadas com `🟡` retornam JSON plausível (stub); rotas `✅` possuem lógica completa de banco de dados.

---

## Comparativo com MedusaJS

| Aspecto                     | MedusaJS v2            | MedusaRust                        |
|-----------------------------|------------------------|-----------------------------------|
| Linguagem                   | TypeScript / Node.js   | Rust                              |
| Framework web               | Fastify                | Axum                              |
| Banco de dados              | PostgreSQL (MikroORM)  | PostgreSQL (SQLx)                 |
| Autenticação                | JWT + bcrypt           | JWT + Argon2                      |
| Cache                       | Redis (opcional)       | Moka in-process (padrão)          |
| Object storage              | S3/MinIO               | S3/MinIO ✅                       |
| Plugin system               | ✅                     | ✅ Trait `PaymentProvider` async + 4 plugins (Asaas, Mercado Pago, Stripe, PayPal) |
| Payment Providers           | ✅ Stripe, PayPal, etc.| ✅ Asaas, Mercado Pago, Stripe, PayPal |
| Event bus                   | ✅ (Redis/SQS)         | ✅ Local broadcast + Redis real (`redis-bus` feature) + `SubscriptionHandle` com `cancel()` + `AuditInterceptor` |
| Workflows / Sagas           | ✅                     | ✅ Saga state machine com steps + compensações; `execute_persistent()` + `execute_persistent_with_events()`; migração SQL |
| Webhooks (receber)          | ✅                     | ✅ Rota `/hooks/payment/:provider` |
| Webhooks (criar/gerenciar)  | ✅                     | ✅ API de criação/listagem/exclusão por plugin |
| OAuth social login          | ✅                     | ❌ (stub)                         |
| Email / SMS                 | ✅                     | ❌ Não implementado               |
| Search (MeiliSearch/Algolia)| ✅                     | ❌ Não implementado               |
| RBAC / Roles                | ✅                     | ❌ Não implementado               |
| Multi-currency              | ✅                     | 🟡 Parcial                        |
| Multi-language              | ✅                     | ❌ Não implementado               |
| Admin Dashboard             | ✅ (Medusa Admin)      | ❌ Use o dashboard do MedusaJS    |
| Import/Export (CSV)         | ✅                     | 🟡 Excel/ZIP wizard               |
| Imagem Docker               | ~400 MB                | ~50 MB                            |
| Uso de memória (idle)       | ~150 MB                | ~10 MB                            |

---

## Pré-requisitos

- **Docker** e **Docker Compose** (para o início rápido)
- **OU** para instalação manual:
  - [Rust](https://rustup.rs/) 1.82+
  - PostgreSQL 15+
  - MinIO ou acesso a S3 (para upload de arquivos)

---

## Início Rápido (Docker Compose)

A forma mais simples de rodar o MedusaRust localmente com PostgreSQL e MinIO:

```bash
# 1. Clone o repositório
git clone https://github.com/SrClauss/medusa_rust.git
cd medusa_rust

# 2. Suba os serviços
docker compose up -d

# 3. Aguarde a inicialização (~30s na primeira vez enquanto o Rust compila)
docker compose logs -f app
```

Após a inicialização:

| Serviço          | URL                              | Credenciais             |
|-----------------|----------------------------------|-------------------------|
| **API REST**    | http://localhost:9000            | —                       |
| **MinIO Console** | http://localhost:9001          | admin / minioadmin      |
| **MinIO S3 API** | http://localhost:9002           | minioadmin / minioadmin |
| **PostgreSQL**  | localhost:5432                   | medusa / medusa         |

### Verificar se está rodando

```bash
curl http://localhost:9000/health
# {"status":"ok"}
```

### Parar os serviços

```bash
docker compose down          # mantém dados
docker compose down -v       # remove volumes (apaga dados)
```

---

## Instalação Manual

### 1. Clonar e configurar variáveis de ambiente

```bash
git clone https://github.com/SrClauss/medusa_rust.git
cd medusa_rust

cp .env.example .env
# Edite .env com suas configurações
```

### 2. Criar banco de dados PostgreSQL

```bash
psql -U postgres -c "CREATE USER medusa WITH PASSWORD 'medusa';"
psql -U postgres -c "CREATE DATABASE medusa_rust OWNER medusa;"
```

> As migrações são executadas **automaticamente** ao iniciar o servidor.

### 3. Configurar MinIO (ou S3)

Para desenvolvimento local, use Docker:

```bash
docker run -d \
  --name minio \
  -p 9002:9000 \
  -p 9001:9001 \
  -e MINIO_ROOT_USER=minioadmin \
  -e MINIO_ROOT_PASSWORD=minioadmin \
  minio/minio server /data --console-address ":9001"
```

Acesse http://localhost:9001 e crie o bucket `medusa-uploads` (ou deixe o servidor criar automaticamente).

### 4. Compilar e executar

```bash
# Modo desenvolvimento
cargo run

# Modo produção (binário otimizado)
cargo build --release
./target/release/medusa_rust
```

O servidor iniciará em `http://0.0.0.0:9000` por padrão.

---

## Variáveis de Ambiente

Copie `.env.example` para `.env` e ajuste conforme necessário:

| Variável              | Padrão                                          | Descrição                                                  |
|-----------------------|-------------------------------------------------|------------------------------------------------------------|
| `DATABASE_URL`        | `postgres://medusa:medusa@localhost:5432/medusa_rust` | Connection string do PostgreSQL                      |
| `JWT_SECRET`          | *(obrigatório)*                                 | Chave secreta para assinar tokens JWT. Use `openssl rand -hex 64` |
| `HOST`                | `0.0.0.0`                                       | Interface de rede para o servidor HTTP                     |
| `PORT`                | `9000`                                          | Porta HTTP do servidor                                     |
| `S3_ENDPOINT`         | `http://localhost:9002`                         | URL do endpoint S3 (vazio para AWS S3 nativo)              |
| `S3_BUCKET`           | `medusa-uploads`                                | Nome do bucket S3/MinIO                                    |
| `S3_REGION`           | `us-east-1`                                     | Região do S3                                               |
| `S3_ACCESS_KEY`       | `minioadmin`                                    | Access key do S3/MinIO                                     |
| `S3_SECRET_KEY`       | `minioadmin`                                    | Secret key do S3/MinIO                                     |
| `S3_FORCE_PATH_STYLE` | `true`                                          | `true` para MinIO; `false` para AWS S3                     |
| `S3_PUBLIC_URL`       | `http://localhost:9002/medusa-uploads`          | Prefixo público das URLs de assets retornadas pela API     |
| `MOKA_MAX_CAPACITY`   | `10000`                                         | Número máximo de entradas no cache in-process              |
| `MOKA_TTL_SECS`       | `300`                                           | Tempo de vida das entradas de cache (segundos)             |
| `UPLOAD_DIR`          | `uploads`                                       | Diretório local de upload (fallback quando S3 não configurado) |
| **Plugin Asaas**      |                                                 |                                                            |
| `ASAAS_API_KEY`       | *(opcional)*                                    | API key do Asaas. Se ausente, o plugin é ignorado na inicialização. |
| `ASAAS_BASE_URL`      | `https://sandbox.asaas.com/api/v3`             | URL base da API Asaas (sandbox ou produção)                |
| **Plugin Mercado Pago** |                                               |                                                            |
| `MP_ACCESS_TOKEN`     | *(opcional)*                                    | Access token do Mercado Pago. Se ausente, o plugin é ignorado. |
| **Plugin Stripe**     |                                                 |                                                            |
| `STRIPE_SECRET_KEY`   | *(opcional)*                                    | Chave secreta da API Stripe (`sk_live_...` ou `sk_test_...`). |
| `STRIPE_WEBHOOK_SECRET` | *(opcional)*                                  | Segredo para validação de webhooks Stripe (`whsec_...`).   |
| **Plugin PayPal**     |                                                 |                                                            |
| `PAYPAL_CLIENT_ID`    | *(opcional)*                                    | Client ID da aplicação PayPal.                             |
| `PAYPAL_CLIENT_SECRET`| *(opcional)*                                    | Client Secret da aplicação PayPal.                         |
| `PAYPAL_BASE_URL`     | `https://api-m.sandbox.paypal.com`             | URL base da API PayPal (sandbox ou produção).              |
| **Event Bus**         |                                                 |                                                            |
| `BUS_DRIVER`          | `local`                                         | Driver do event bus: `local` (in-process), `redis` ou `sqs`. |
| `REDIS_URL`           | `redis://127.0.0.1`                             | URL do Redis (somente quando `BUS_DRIVER=redis`).          |
| `AWS_REGION`          | `us-east-1`                                     | Região AWS (somente quando `BUS_DRIVER=sqs`).              |
| `AWS_SQS_URL`         | *(obrigatório para sqs)*                        | URL da fila SQS (somente quando `BUS_DRIVER=sqs`).         |

### Gerar JWT_SECRET seguro

```bash
openssl rand -hex 64
```

---

## Plugins de Pagamento

O MedusaRust inclui um sistema completo de plugins de pagamento com 4 provedores prontos para uso. Cada plugin é um crate Rust independente dentro do workspace.

### Provedores disponíveis

| Provedor | Crate | Moedas | Recursos |
|----------|-------|--------|----------|
| **Asaas** | `asaas_plugin` | BRL | PIX, Boleto, Cartão — gateway brasileiro |
| **Mercado Pago** | `mercadopago_plugin` | BRL, USD, ARS... | PIX, Cartão — América Latina |
| **Stripe** | `stripe_plugin` | USD, EUR, BRL... | PaymentIntents, validação de assinatura |
| **PayPal** | `paypal_plugin` | USD, EUR... | Orders API, OAuth2 automático |

### Configurar um plugin

Cada plugin é ativado automaticamente na inicialização do servidor se suas variáveis de ambiente estiverem presentes. Por exemplo, para ativar o Stripe:

```bash
# .env
STRIPE_SECRET_KEY=sk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...
```

### Criar/gerenciar webhooks via código

```rust
use stripe_plugin::StripePlugin;
use plugin_api::PaymentProvider;

let id = plugin.create_webhook(
    "https://meusite.com/hooks/payment/stripe",
    vec!["payment_intent.succeeded".to_string()],
).await?;

let all = plugin.list_webhooks().await?;
plugin.delete_webhook(&id).await?;
```

### Receber webhooks

O servidor expõe a rota `POST /hooks/payment/:provider` para receber eventos de qualquer provedor registrado.

Para mais detalhes, consulte [`crates/plugins/README.md`](crates/plugins/README.md) e o README individual de cada plugin.

---

A API usa **JWT (Bearer Token)**. Há dois contextos de autenticação:

- **Admin** (`/admin/*`) — usuários administradores
- **Store** (`/store/*`) — clientes da loja

### Login como Admin

```bash
curl -X POST http://localhost:9000/admin/auth \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@example.com", "password": "sua-senha"}'
```

Resposta:

```json
{
  "user": { "id": "...", "email": "admin@example.com", ... },
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
}
```

### Usar o token nas requisições

```bash
export TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."

curl http://localhost:9000/admin/products \
  -H "Authorization: Bearer $TOKEN"
```

### Login como Cliente (Store)

```bash
curl -X POST http://localhost:9000/store/auth \
  -H "Content-Type: application/json" \
  -d '{"email": "cliente@example.com", "password": "senha"}'
```

### Registro de Cliente

```bash
curl -X POST http://localhost:9000/store/customers \
  -H "Content-Type: application/json" \
  -d '{"email": "novo@example.com", "password": "senha", "first_name": "João", "last_name": "Silva"}'
```

### Reset de Senha

```bash
# 1. Solicitar token de reset
curl -X POST http://localhost:9000/store/customers/password-token \
  -H "Content-Type: application/json" \
  -d '{"email": "cliente@example.com"}'

# 2. Definir nova senha (use o token recebido por e-mail / API)
curl -X POST http://localhost:9000/store/customers/password-reset \
  -H "Content-Type: application/json" \
  -d '{"token": "<reset-token>", "password": "nova-senha"}'
```

---

## Referência da API

### Store API

Rotas públicas e autenticadas da loja. Base: `http://localhost:9000`

#### Autenticação Store

| Método | Rota | Descrição |
|--------|------|-----------|
| `POST` | `/store/auth` | Login de cliente |
| `GET`  | `/store/auth` | Obter sessão atual |
| `DELETE` | `/store/auth` | Logout |

> **Nota:** As rotas abaixo usam o prefixo `/auth/` (sem `/store/`) porque são **rotas compartilhadas** entre store e admin — o mesmo endpoint atende clientes (`/auth/customer/...`) e administradores (`/auth/user/...`). Isso espelha o comportamento do MedusaJS v2.

| Método | Rota | Descrição |
|--------|------|-----------|
| `POST` | `/auth/customer/{provider}` | Login OAuth de cliente (provider: `emailpass`) |
| `GET`  | `/auth/customer/{provider}/callback` | Callback OAuth |
| `POST` | `/auth/customer/{provider}/register` | Registro via provider |
| `POST` | `/auth/customer/{provider}/reset-password` | Reset de senha via provider |
| `POST` | `/auth/customer/{provider}/update` | Atualizar credenciais |
| `POST` | `/auth/session` | Recuperar sessão (requer autenticação) |
| `DELETE` | `/auth/session` | Excluir sessão |
| `POST` | `/auth/token/refresh` | Renovar token JWT |

#### Carrinho

| Método | Rota | Descrição |
|--------|------|-----------|
| `POST` | `/store/carts` | Criar carrinho |
| `GET`  | `/store/carts/{id}` | Obter carrinho |
| `POST` | `/store/carts/{id}` | Atualizar carrinho |
| `POST` | `/store/carts/{id}/complete` | Finalizar carrinho |
| `POST` | `/store/carts/{id}/line-items` | Adicionar item |
| `POST` | `/store/carts/{id}/line-items/{line_id}` | Atualizar item |
| `DELETE` | `/store/carts/{id}/line-items/{line_id}` | Remover item |
| `POST` | `/store/carts/{id}/promotions` | Aplicar promoção |
| `POST` | `/store/carts/{id}/discounts/{code}` | Aplicar desconto |
| `DELETE` | `/store/carts/{id}/discounts/{code}` | Remover desconto |
| `POST` | `/store/carts/{id}/shipping-methods` | Adicionar frete |
| `POST` | `/store/carts/{id}/taxes` | Calcular impostos |
| `POST` | `/store/carts/{id}/payment-sessions` | Criar sessões de pagamento |
| `POST` | `/store/carts/{id}/payment-session` | Selecionar sessão |

#### Clientes

| Método | Rota | Descrição |
|--------|------|-----------|
| `POST` | `/store/customers` | Registrar cliente |
| `GET`  | `/store/customers/me` | Dados do cliente logado |
| `POST` | `/store/customers/me` | Atualizar dados |
| `GET`  | `/store/customers/me/addresses` | Listar endereços |
| `POST` | `/store/customers/me/addresses` | Adicionar endereço |
| `GET`  | `/store/customers/me/addresses/{id}` | Obter endereço |
| `POST` | `/store/customers/me/addresses/{id}` | Atualizar endereço |
| `DELETE` | `/store/customers/me/addresses/{id}` | Excluir endereço |
| `GET`  | `/store/customers/me/orders` | Pedidos do cliente |
| `POST` | `/store/customers/password-token` | Solicitar reset de senha |
| `POST` | `/store/customers/password-reset` | Resetar senha |

#### Produtos

| Método | Rota | Descrição |
|--------|------|-----------|
| `GET`  | `/store/products` | Listar produtos |
| `GET`  | `/store/products/{id}` | Obter produto |
| `GET`  | `/store/product-categories` | Listar categorias |
| `GET`  | `/store/product-categories/{id}` | Obter categoria |
| `GET`  | `/store/product-tags` | Listar tags |
| `GET`  | `/store/product-types` | Listar tipos |
| `GET`  | `/store/collections` | Listar coleções |
| `GET`  | `/store/collections/{id}` | Obter coleção |

#### Pedidos & Devoluções

| Método | Rota | Descrição |
|--------|------|-----------|
| `GET`  | `/store/orders` | Listar pedidos |
| `GET`  | `/store/orders/{id}` | Obter pedido |
| `POST` | `/store/orders/{id}/transfer/request` | Solicitar transferência |
| `POST` | `/store/orders/{id}/transfer/accept` | Aceitar transferência |
| `POST` | `/store/orders/{id}/transfer/decline` | Recusar transferência |
| `POST` | `/store/returns` | Criar devolução |

#### Outros (Store)

| Método | Rota | Descrição |
|--------|------|-----------|
| `GET`  | `/store/currencies` | Listar moedas |
| `GET`  | `/store/currencies/{code}` | Obter moeda |
| `GET`  | `/store/regions` | Listar regiões |
| `GET`  | `/store/regions/{id}` | Obter região |
| `GET`  | `/store/shipping-options` | Opções de frete |
| `POST` | `/store/shipping-options/{id}/calculate` | Calcular frete |
| `GET`  | `/store/gift-cards/{idOrCode}` | Obter gift card |
| `GET`  | `/store/return-reasons` | Motivos de devolução |
| `GET`  | `/store/payment-providers` | Provedores de pagamento |
| `GET`  | `/store/locales` | Localidades |

---

### Admin API

Todas as rotas admin exigem autenticação via `Authorization: Bearer <token>`.

#### Exemplo completo — criar produto

```bash
# 1. Fazer login
TOKEN=$(curl -s -X POST http://localhost:9000/admin/auth \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"senha"}' \
  | jq -r '.token')

# 2. Criar produto
curl -X POST http://localhost:9000/admin/products \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Camiseta Rust",
    "description": "Camiseta estampada com a mascote do Rust",
    "status": "published"
  }'
```

#### Produtos

| Método | Rota | Descrição |
|--------|------|-----------|
| `GET`  | `/admin/products` | Listar produtos |
| `POST` | `/admin/products` | Criar produto |
| `GET`  | `/admin/products/{id}` | Obter produto |
| `PUT`  | `/admin/products/{id}` | Atualizar produto |
| `DELETE` | `/admin/products/{id}` | Excluir produto |
| `GET`  | `/admin/products/{id}/variants` | Listar variantes |
| `POST` | `/admin/products/{id}/variants` | Criar variante |
| `PUT`  | `/admin/products/{id}/variants/{vid}` | Atualizar variante |
| `DELETE` | `/admin/products/{id}/variants/{vid}` | Excluir variante |
| `GET`  | `/admin/products/{id}/options` | Listar opções |
| `POST` | `/admin/products/{id}/options` | Criar opção |
| `PUT`  | `/admin/products/{id}/options/{oid}` | Atualizar opção |
| `DELETE` | `/admin/products/{id}/options/{oid}` | Excluir opção |

#### Pedidos

| Método | Rota | Descrição |
|--------|------|-----------|
| `GET`  | `/admin/orders` | Listar pedidos |
| `GET`  | `/admin/orders/{id}` | Obter pedido |
| `POST` | `/admin/orders/{id}/archive` | Arquivar pedido |
| `POST` | `/admin/orders/{id}/cancel` | Cancelar pedido |
| `POST` | `/admin/orders/{id}/complete` | Completar pedido |
| `POST` | `/admin/orders/{id}/refunds` | Criar reembolso |
| `POST` | `/admin/orders/{id}/return` | Solicitar devolução |
| `POST` | `/admin/orders/{id}/swaps` | Criar troca |
| `POST` | `/admin/orders/{id}/claims` | Criar reclamação |
| `POST` | `/admin/orders/{id}/fulfillment` | Criar fulfillment |
| `POST` | `/admin/orders/{id}/fulfillments/{fid}/cancel` | Cancelar fulfillment |
| `POST` | `/admin/orders/{id}/shipment` | Registrar envio |

#### Clientes (Admin)

| Método | Rota | Descrição |
|--------|------|-----------|
| `GET`  | `/admin/customers` | Listar clientes |
| `POST` | `/admin/customers` | Criar cliente |
| `GET`  | `/admin/customers/{id}` | Obter cliente |
| `POST` | `/admin/customers/{id}` | Atualizar cliente |
| `GET`  | `/admin/customer-groups` | Listar grupos |
| `POST` | `/admin/customer-groups` | Criar grupo |
| `POST` | `/admin/customer-groups/{id}/customers` | Vincular clientes |

#### Regiões & Impostos

| Método | Rota | Descrição |
|--------|------|-----------|
| `GET`  | `/admin/regions` | Listar regiões |
| `POST` | `/admin/regions` | Criar região |
| `PUT`  | `/admin/regions/{id}` | Atualizar região |
| `DELETE` | `/admin/regions/{id}` | Excluir região |
| `GET`  | `/admin/tax-regions` | Listar regiões fiscais |
| `POST` | `/admin/tax-regions` | Criar região fiscal |
| `GET`  | `/admin/tax-rates` | Listar alíquotas |
| `GET`  | `/admin/tax-providers` | Provedores fiscais |

#### Descontos & Promoções

| Método | Rota | Descrição |
|--------|------|-----------|
| `GET`  | `/admin/discounts` | Listar descontos |
| `POST` | `/admin/discounts` | Criar desconto |
| `PUT`  | `/admin/discounts/{id}` | Atualizar desconto |
| `DELETE` | `/admin/discounts/{id}` | Excluir desconto |
| `GET`  | `/admin/promotions` | Listar promoções |
| `POST` | `/admin/promotions` | Criar promoção |
| `DELETE` | `/admin/promotions/{id}` | Excluir promoção |
| `GET`  | `/admin/campaigns` | Listar campanhas |
| `POST` | `/admin/campaigns` | Criar campanha |
| `GET`  | `/admin/price-lists` | Listar listas de preço |
| `POST` | `/admin/price-lists` | Criar lista de preço |

#### Inventário & Logística

| Método | Rota | Descrição |
|--------|------|-----------|
| `GET`  | `/admin/inventory-items` | Listar itens de inventário |
| `POST` | `/admin/inventory-items` | Criar item |
| `PUT`  | `/admin/inventory-items/{id}` | Atualizar item |
| `GET`  | `/admin/reservations` | Listar reservas |
| `POST` | `/admin/reservations` | Criar reserva |
| `GET`  | `/admin/stock-locations` | Listar localizações |
| `POST` | `/admin/stock-locations` | Criar localização |
| `GET`  | `/admin/shipping-options` | Opções de frete |
| `POST` | `/admin/shipping-options` | Criar opção de frete |
| `GET`  | `/admin/shipping-profiles` | Perfis de frete |
| `GET`  | `/admin/fulfillment-providers` | Provedores de fulfillment |
| `GET`  | `/admin/fulfillment-sets/{id}` | Obter fulfillment set |

#### Usuários & Autenticação Admin

| Método | Rota | Descrição |
|--------|------|-----------|
| `GET`  | `/admin/users` | Listar usuários |
| `POST` | `/admin/users` | Criar usuário |
| `GET`  | `/admin/users/me` | Usuário atual |
| `GET`  | `/admin/users/{id}` | Obter usuário |
| `POST` | `/admin/users/{id}` | Atualizar usuário |
| `DELETE` | `/admin/users/{id}` | Excluir usuário |
| `GET`  | `/admin/invites` | Listar convites |
| `POST` | `/admin/invites` | Criar convite |
| `DELETE` | `/admin/invites/{id}` | Excluir convite |
| `POST` | `/admin/invites/{id}/accept` | Aceitar convite |
| `GET`  | `/admin/api-keys` | Listar API keys |
| `POST` | `/admin/api-keys` | Criar API key |
| `POST` | `/admin/api-keys/{id}/revoke` | Revogar API key |

#### Outras Rotas Admin

| Método | Rota | Descrição |
|--------|------|-----------|
| `GET`  | `/admin/gift-cards` | Listar gift cards |
| `POST` | `/admin/gift-cards` | Criar gift card |
| `GET`  | `/admin/currencies` | Listar moedas |
| `PUT`  | `/admin/currencies/{code}` | Atualizar moeda |
| `GET`  | `/admin/stores` | Listar lojas |
| `POST` | `/admin/stores/{id}` | Atualizar loja |
| `GET`  | `/admin/sales-channels` | Canais de venda |
| `POST` | `/admin/sales-channels` | Criar canal |
| `GET`  | `/admin/notifications` | Listar notificações |
| `POST` | `/admin/notifications/{id}/resend` | Reenviar notificação |
| `GET`  | `/admin/batch-jobs` | Listar batch jobs |
| `POST` | `/admin/batch-jobs` | Criar batch job |
| `POST` | `/admin/uploads` | Upload de arquivo |
| `DELETE` | `/admin/uploads` | Excluir arquivos |
| `GET`  | `/admin/plugins` | Listar plugins |
| `POST` | `/hooks/payment/:provider` | Receber webhook de provedores de pagamento |
| `GET`  | `/admin/feature-flags` | Feature flags |
| `GET`  | `/admin/returns` | Listar devoluções |
| `GET`  | `/admin/claims` | Listar reclamações |
| `GET`  | `/admin/exchanges` | Listar trocas |
| `GET`  | `/admin/draft-orders` | Rascunhos de pedido |
| `GET`  | `/admin/order-edits` | Edições de pedido |
| `GET`  | `/admin/payment-collections` | Coleções de pagamento |
| `GET`  | `/admin/payments` | Listar pagamentos |
| `POST` | `/admin/payments/{id}/capture` | Capturar pagamento |
| `POST` | `/admin/payments/{id}/refund` | Reembolsar pagamento |
| `GET`  | `/admin/refund-reasons` | Motivos de reembolso |
| `GET`  | `/admin/return-reasons` | Motivos de devolução |

---

## Funcionalidades Além de Rotas

### ✅ Implementado

| Funcionalidade | Detalhes |
|----------------|----------|
| **JWT Authentication** | Tokens assinados com HS256, middleware de autenticação para admin e store |
| **Password Hashing** | Argon2id (state-of-the-art) |
| **PostgreSQL com migrações** | SQLx + migrações automáticas na inicialização |
| **Cache in-process** | Moka com TTL e capacidade configuráveis |
| **Object Storage** | MinIO / AWS S3 compatível (upload, delete, URLs públicas) |
| **CORS configurável** | Headers CORS liberados para desenvolvimento |
| **Batch Jobs (async)** | Processamento assíncrono de jobs com persistência em banco |
| **Import de produtos (Excel/ZIP)** | Wizard de importação via arquivo Excel dentro de ZIP |
| **Logs estruturados** | tracing + tracing-subscriber com filtro por nível (`RUST_LOG`) |
| **Docker multi-stage** | Imagem de produção ~50 MB |
| **Plugin System** | Trait `PaymentProvider` assíncrono (`plugin_api`), `PluginManager` com registro dinâmico |
| **Payment Providers** | Asaas, Mercado Pago, Stripe e PayPal — criar/capturar/reembolsar/cancelar pagamentos |
| **Gerenciamento de Webhooks** | Criar, listar e excluir webhooks via API REST em cada provedor de pagamento |
| **Webhook Ingestion** | Rota `/hooks/payment/:provider` — parseia e normaliza eventos de todos os provedores |
| **Stripe Webhook Signature** | Validação de assinatura `Stripe-Signature` com HMAC-SHA256 |
| **PayPal OAuth2** | Obtenção automática de access token na inicialização do plugin |
| **Event Bus** | Publish/subscribe de eventos de domínio; drivers `local` (in-process) e `redis` (real, via feature `redis-bus`); `subscribe()` retorna `SubscriptionHandle` com `cancel()`; `AuditInterceptor` para auditoria em banco |
| **Workflows / Sagas** | Orquestração com steps, compensações (rollback) e persistência; `execute_persistent()` com idempotência via `transaction_id`; `execute_persistent_with_events()` publica eventos no EventBus após conclusão/falha |

### 🟡 Parcialmente Implementado

| Funcionalidade | Estado |
|----------------|--------|
| **Multi-moeda** | Tabela e rotas de listagem; sem conversão automática |
| **OAuth Providers** | Estrutura de rota presente; apenas email/password funciona |
| **Fulfillment Providers** | Listagem; sem integração com transportadoras |

---

## O que Falta em Comparação ao MedusaJS

As seguintes funcionalidades existem no MedusaJS mas **ainda não estão implementadas** no MedusaRust:

### Rotas Admin não implementadas

| Grupo | Rotas faltando |
|-------|----------------|
| Customers | DELETE, addresses CRUD, link groups |
| Products | batch, export, import, variant images batch |
| Orders | export, order changes, credit lines, fulfillments list, mark-delivered, transfer |
| Product Categories | POST, GET/{id}, PUT, DELETE |
| Tax Rates | POST, GET/{id}, PUT, DELETE, rules |
| Shipping Options | GET/{id}, PUT, DELETE, rules batch |
| Gift Cards | GET orders |
| Store Credits | CRUD completo |
| Locales (admin) | GET list, GET by code |
| Translations | CRUD completo + batch + settings |
| Views | CRUD de configurações |
| Workflow Executions | CRUD completo + step management |
| Index | details, sync |
| Uploads | presigned URLs, DELETE by ID |
| Promotions | rule-attribute-options, rule-value-options |
| Returns | action-level endpoints (dismiss, receive, request, shipping actions) |
| Draft Orders | convert-to-order |
| Invites | accept (sem ID), GET/{id}, resend |
| Payments | payment-providers |

### Funcionalidades de sistema faltando

| Funcionalidade | Impacto |
|----------------|---------|
| **Scheduled Jobs** | Sem tarefas agendadas (expirar descontos, etc.) |
| **Email / SMS** | Sem envio de emails transacionais ou SMS |
| **Full-text Search** | Sem integração com MeiliSearch ou Algolia |
| **Redis Cache** | Apenas cache in-process (não distribuído) |
| **RBAC** | Sem controle de acesso baseado em roles |
| **OAuth social** | Google, GitHub, etc. não implementados |
| **Multi-language** | Traduções e internacionalização ausentes |
| **Admin Dashboard** | Sem UI admin (use o dashboard do MedusaJS apontando para esta API) |
| **Store Credits (admin)** | CRUD de créditos de loja ausente |

---

## Testes

O projeto inclui testes de integração abrangentes:

```bash
# Executar todos os testes (incluindo plugins de pagamento)
cargo test --workspace

# Executar apenas os testes de um plugin específico
cargo test --package asaas_plugin
cargo test --package mercadopago_plugin
cargo test --package stripe_plugin
cargo test --package paypal_plugin

# Executar suites de integração do servidor principal
cargo test --test auth_tests
cargo test --test admin_routes_tests
cargo test --test store_routes_tests

# Com output detalhado
cargo test -- --nocapture
```

### Suites de teste disponíveis

| Suite | Arquivo / Pacote | Testes |
|-------|-----------------|--------|
| Auth global | `tests/auth_tests.rs` | 7 |
| Returns admin | `tests/returns_tests.rs` | 3 |
| Currencies | `tests/currencies_tests.rs` | 4 |
| Gift Cards | `tests/gift_cards_tests.rs` | 4 |
| Novas rotas admin | `tests/admin_new_routes_tests.rs` | 43 |
| Rotas admin | `tests/admin_routes_tests.rs` | 106 |
| Rotas store | `tests/store_routes_tests.rs` | 55 |
| Fases 2-4 | `tests/phase2_routes_tests.rs` | 55 |
| **Plugin Asaas** | `asaas_plugin` (integration) | **11** |
| **Plugin Mercado Pago** | `mercadopago_plugin` (integration) | **11** |
| **Plugin Stripe** | `stripe_plugin` (integration) | **12** |
| **Plugin PayPal** | `paypal_plugin` (integration) | **12** |
| **Total** | | **~323** |

> ⚠️ Os testes de integração do servidor exigem uma instância PostgreSQL rodando. Use `docker compose up -d postgres` antes de rodá-los.  
> ✅ Os testes dos plugins de pagamento não precisam de banco de dados — usam `mockito` para simular as APIs externas.

---

## Estrutura do Projeto

```
medusa_rust/
├── Cargo.toml               # Workspace root + dependências do binário principal
├── src/
│   ├── main.rs              # Ponto de entrada: compõe AppState, registra plugins e inicia o servidor
│   ├── lib.rs               # Re-exportações para testes de integração
│   ├── routes.rs            # Registro de todas as rotas
│   ├── routes_manifest.rs   # Constantes de todas as rotas (para testes)
│   ├── models.rs            # Structs de domínio compartilhados
│   ├── state.rs             # AppState (pool, cache, S3, JWT, plugin_mgr)
│   ├── error.rs             # Tipos de erro centralizados
│   ├── api/
│   │   ├── mod.rs           # build_router()
│   │   ├── admin/           # Handlers do Admin API (um arquivo por recurso)
│   │   ├── store/           # Handlers do Store API (um arquivo por recurso)
│   │   └── auth/            # Serviço de autenticação (EmailPasswordService)
│   ├── auth/
│   │   ├── argon.rs         # Hashing Argon2
│   │   ├── jwt.rs           # Geração/validação de JWT
│   │   └── mod.rs           # Middleware de autenticação
│   ├── core/
│   │   ├── cart.rs          # Lógica de carrinho
│   │   ├── inventory.rs     # Lógica de inventário
│   │   ├── payment.rs       # Trait PaymentProvider (síncrono, legado)
│   │   ├── pricing.rs       # Cálculo de preços
│   │   └── tax.rs           # Cálculo de impostos
│   ├── plugins/
│   │   └── mod.rs           # PluginManager (suporta providers síncronos e assíncronos)
│   ├── storage/
│   │   ├── db.rs            # Pool SQLx + migrações
│   │   ├── s3.rs            # Cliente S3/MinIO
│   │   └── cache.rs         # Builder do cache Moka
│   └── wizard/
│       ├── excel.rs         # Parser Excel (calamine)
│       ├── zip_import.rs    # Import via ZIP
│       └── slugify.rs       # Geração de slugs
├── crates/
│   ├── plugin_api/          # Trait PaymentProvider assíncrono e tipos comuns
│   │   └── src/lib.rs
│   └── plugins/
│       ├── README.md        # Guia de arquitetura e como adicionar novos plugins
│       ├── asaas/           # Plugin Asaas (gateway brasileiro)
│       │   ├── src/{lib,plugin,types,webhooks}.rs
│       │   └── tests/integration_tests.rs
│       ├── mercadopago/     # Plugin Mercado Pago
│       │   ├── src/{lib,plugin,types,webhooks}.rs
│       │   └── tests/integration_tests.rs
│       ├── stripe/          # Plugin Stripe (com validação de assinatura)
│       │   ├── src/{lib,plugin,types,webhooks}.rs
│       │   └── tests/integration_tests.rs
│       └── paypal/          # Plugin PayPal (com OAuth2)
│           ├── src/{lib,plugin,types,webhooks}.rs
│           └── tests/integration_tests.rs
├── migrations/              # Migrações SQL (executadas automaticamente)
├── tests/                   # Testes de integração do servidor principal
├── Dockerfile               # Multi-stage: builder Rust + runtime Debian slim
├── docker-compose.yml       # Stack completa: app + postgres + minio
├── .env.example             # Template de variáveis de ambiente
└── IMPLEMENTATION_PLAN.md   # Plano detalhado de implementação e cobertura de rotas
```

---

## Contribuindo

Contribuições são bem-vindas! Siga as convenções do projeto:

### Convenções de código

1. **Novos endpoints**: Adicione a constante de rota em `src/routes_manifest.rs`.
2. **Handler pattern**: `query row → call build_* helper → return Json(...)`.
3. **Testes**: Cada novo grupo de rotas deve ter testes de: autenticação (401), existência (não-404) e método HTTP (não-405). Veja `tests/returns_tests.rs` como modelo.
4. **Stubs são aceitáveis**: Retornar JSON plausível é preferível a não ter a rota.

### Comandos úteis

```bash
# Compilar workspace completo (incluindo plugins)
cargo build --workspace

# Compilar apenas o binário principal
cargo build

# Verificar erros de compilação (rápido)
cargo check --workspace

# Lint
cargo clippy --workspace

# Formatar código
cargo fmt --all

# Testes — servidor principal (requer PostgreSQL)
cargo test

# Testes — todos os plugins de pagamento (sem banco de dados)
cargo test --package asaas_plugin --package mercadopago_plugin --package stripe_plugin --package paypal_plugin

# Testes — workspace completo
cargo test --workspace

# Build de produção
cargo build --release

# Contar handlers implementados
grep -r "pub async fn" src/api/ | wc -l
```

### Fluxo de contribuição

```bash
# 1. Fork e clone
git clone https://github.com/seu-usuario/medusa_rust.git
cd medusa_rust

# 2. Criar branch
git checkout -b feat/nome-da-feature

# 3. Fazer alterações e testar
cargo test

# 4. Commit e push
git commit -m "feat: adicionar rota X"
git push origin feat/nome-da-feature

# 5. Abrir Pull Request
```

---

## Licença

MIT — veja [LICENSE](LICENSE) para detalhes.

---

<div align="center">
  <sub>Construído com ❤️ e 🦀 — Port do <a href="https://medusajs.com/">MedusaJS v2</a> para Rust</sub>
</div>
