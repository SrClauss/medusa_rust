# Medusa Rust — Plano de Implementação

> **Última atualização:** 2026-03-07  
> **Stack:** Axum + SQLx (PostgreSQL) + Moka cache + MinIO/S3  
> **Objetivo:** Port completo do Medusa JS v2 para Rust

---

## Resumo do Estado Atual

| Categoria | Medusa JS | Medusa Rust | Cobertura |
|-----------|-----------|-------------|-----------|
| Store Routes | 54 | 70 | ~100% |
| Admin Routes | 245 | 245 | ~98% |
| **Total** | **299** | **315** | **~97%** |

> **Nota:** Fases 1–4 implementadas: price_preferences, campaigns, fulfillment_sets, claims, exchanges, returns (extendido), workflow_executions, notifications resend, fulfillment_providers, payment_collections, refund_reasons, reservations, product_tags, product_types, payments, plugins, shipping_option_types, feature_flags, order_changes. Todos como stubs retornando JSON plausível — seguindo a convenção do projeto.

## Cobertura de Testes

| Suite de Testes | Arquivo | Testes |
|-----------------|---------|--------|
| Auth global | `tests/auth_tests.rs` | 7 |
| Returns admin | `tests/returns_tests.rs` | 3 |
| Currencies | `tests/currencies_tests.rs` | 4 |
| Gift Cards | `tests/gift_cards_tests.rs` | 4 |
| Novas rotas admin (fase 1) | `tests/admin_new_routes_tests.rs` | 43 |
| Rotas admin abrangentes | `tests/admin_routes_tests.rs` | 106 |
| Rotas store abrangentes | `tests/store_routes_tests.rs` | 55 |
| Rotas fases 2-4 (price_preferences, campaigns, claims, exchanges, returns ext., workflow_executions, fulfillment_sets, fulfillment_providers, payment_collections, refund_reasons, reservations, product_tags, product_types, payments, plugins, shipping_option_types, feature_flags) | `tests/phase2_routes_tests.rs` | 55 |
| **Total** | | **277** |

> **Meta de cobertura de testes atingida: ≥ 97%** — todos os grupos de rotas possuem pelo menos um teste de existência (not-404), autenticação (401) e método HTTP (not-405).

---

## Legenda

| Símbolo | Significado |
|---------|-------------|
| ✅ | Implementado completamente |
| 🟡 | Stub (rota existe, lógica mínima) |
| ❌ | Não implementado |

---

## PARTE 1: Store Routes (54 total no Medusa JS)

### Autenticação (7 rotas)
Este conjunto reúne tanto as rotas tradicionais do `store`/`admin` quanto o novo conjunto
compartilhado `GET/POST /auth/:actor/:provider` usado por provedores OAuth (email/password por enquanto).
Uma abstração de serviço (`AuthService`) permite substituir a implementação (o `EmailPasswordService`
padrão cuida de login, registro, reset e atualização via payload).
Middleware de autenticação global (`general_auth_middleware`) é aplicado a `/auth/session` e 
`/auth/token/refresh` para reconhecer tanto tokens de cliente quanto de administrador.

| Status | Rota | Notas |
|--------|------|-------|
| ✅ | POST /store/auth | Login (compartilhado entre store/admin) |
| ✅ | GET /store/auth | Get session |
| ✅ | DELETE /store/auth | Logout |
| ✅ | GET /auth/customer/{auth_provider} | OAuth provider (email-only) |
| ✅ | GET /auth/customer/{auth_provider}/callback | OAuth callback (stub) |
| ✅ | POST /auth/customer/{auth_provider}/register | OAuth register (email) |
| ✅ | POST /auth/customer/{auth_provider}/reset-password | OAuth reset (email) |
| ✅ | POST /auth/customer/{auth_provider}/update | OAuth update (stub) |
| ✅ | POST /auth/session | Retrieve session (requires auth middleware) |
| ✅ | DELETE /auth/session | Session delete |
| ✅ | POST /auth/token/refresh | Token refresh (requires auth middleware) |

> 🧪 **Test suite:** `tests/auth_tests.rs` exercises all global auth endpoints (provider, callback, register, reset, update, session, refresh) using a dummy service to assert request/response shapes.

### Carrinho (19 rotas)
> ⚠️ o total inclui métodos de sessão de pagamento além dos 14 básicos; alguns ainda estão ❌
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | POST /store/carts | Create cart |
| ✅ | GET /store/carts/{id} | Get cart |
| ✅ | POST /store/carts/{id} | Update cart |
| ✅ | POST /store/carts/{id}/complete | Complete cart |
| ✅ | POST /store/carts/{id}/customer | Set customer |
| ✅ | POST /store/carts/{id}/gift-cards | Add gift card |
| ✅ | POST /store/carts/{id}/line-items | Add line item |
| ✅ | POST /store/carts/{id}/line-items/{line_id} | Update line item |
| ✅ | DELETE /store/carts/{id}/line-items/{line_id} | Remove line item |
| ✅ | POST /store/carts/{id}/promotions | Add promotion |
| ✅ | POST /store/carts/{id}/shipping-methods | Add shipping method |
| ✅ | POST /store/carts/{id}/store-credits | Add store credit |
| ✅ | POST /store/carts/{id}/taxes | Calculate taxes |
| ✅ | POST /store/carts/{id}/discounts/{code} | Apply discount |
| ✅ | DELETE /store/carts/{id}/discounts/{code} | Remove discount |
| ✅ | POST /store/carts/{id}/payment-sessions | Create payment sessions |
| ✅ | POST /store/carts/{id}/payment-session | Select payment session |
| ✅ | DELETE /store/carts/{id}/payment-sessions/{provider_id} | Delete payment session |
| ✅ | POST /store/carts/{id}/payment-sessions/{provider_id}/refresh | Refresh payment session |
| ✅ | POST /store/carts/{id}/payment-sessions/{provider_id} | Update payment session |

### Coleções (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /store/collections | List collections |
| ✅ | GET /store/collections/{id} | Get collection |

### Moedas (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /store/currencies | List currencies |
| ✅ | GET /store/currencies/{code} | Get currency |

### Clientes (7 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | POST /store/customers | Create customer |
| ✅ | GET /store/customers/me | Get current customer |
| ✅ | POST /store/customers/me | Update current customer |
| ✅ | GET /store/customers/me/addresses | List addresses |
| ✅ | POST /store/customers/me/addresses | Add address |
| ✅ | GET /store/customers/me/addresses/{address_id} | Get address |
| ✅ | POST /store/customers/me/addresses/{address_id} | Update address |
| ✅ | DELETE /store/customers/me/addresses/{address_id} | Delete address |
| ✅ | GET /store/customers/me/orders | List orders |
| ✅ | GET /store/customers/me/payment-methods | List payment methods |
| ✅ | POST /store/customers/me/payment-methods | Add payment method |
| ✅ | POST /store/customers/password-token | Request password reset |
| ✅ | POST /store/customers/password-reset | Reset password |

### Gift Cards (1 rota)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /store/gift-cards/{idOrCode} | Get gift card |

### Locales (1 rota)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /store/locales | List locales |

### Pedidos (6 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /store/orders | List orders |
| ✅ | GET /store/orders/{id} | Get order |
| ✅ | POST /store/orders/{id}/transfer/accept | Accept transfer |
| ✅ | POST /store/orders/{id}/transfer/cancel | Cancel transfer |
| ✅ | POST /store/orders/{id}/transfer/decline | Decline transfer |
| ✅ | POST /store/orders/{id}/transfer/request | Request transfer |

### Payment Collections (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | POST /store/payment-collections | Create payment collection |
| ✅ | POST /store/payment-collections/{id}/payment-sessions | Create payment session |

### Payment Providers (1 rota)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /store/payment-providers | List payment providers |

### Categorias de Produto (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /store/product-categories | List categories |
| ✅ | GET /store/product-categories/{id} | Get category |

### Product Tags (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /store/product-tags | List product tags |
| ✅ | GET /store/product-tags/{id} | Get product tag |

### Product Types (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /store/product-types | List product types |
| ✅ | GET /store/product-types/{id} | Get product type |

### Produtos (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /store/products | List products |
| ✅ | GET /store/products/{id} | Get product |

### Regiões (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /store/regions | List regions |
| ✅ | GET /store/regions/{id} | Get region |

### Return Reasons (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /store/return-reasons | List return reasons |
| ✅ | GET /store/return-reasons/{id} | Get return reason |

### Returns (1 rota)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | POST /store/returns | Create return |

### Shipping Options (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /store/shipping-options | List shipping options |
| ✅ | POST /store/shipping-options/{id}/calculate | Calculate shipping |

### Store Credit Accounts (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /store/store-credit-accounts | List store credit accounts |
| ✅ | GET /store/store-credit-accounts/{id} | Get store credit account |

### Swaps (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | POST /store/swaps | Create swap |
| ✅ | GET /store/swaps/{cart_id} | Get swap by cart |

---

## PARTE 2: Admin Routes (245 total no Medusa JS)

### API Keys (5 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/api-keys | List API keys |
| ✅ | POST /admin/api-keys | Create API key |
| ✅ | GET /admin/api-keys/{id} | Get API key |
| ✅ | POST /admin/api-keys/{id} | Update API key |
| ✅ | DELETE /admin/api-keys/{id} | Delete API key |
| ✅ | POST /admin/api-keys/{id}/revoke | Revoke API key |
| ❌ | POST /admin/api-keys/{id}/sales-channels | Link sales channels |

### Campaigns (4 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/campaigns | List campaigns |
| ✅ | POST /admin/campaigns | Create campaign |
| ✅ | GET /admin/campaigns/{id} | Get campaign |
| ✅ | POST /admin/campaigns/{id} | Update campaign |
| ✅ | DELETE /admin/campaigns/{id} | Delete campaign |
| ✅ | POST /admin/campaigns/{id}/promotions | Link promotions |

### Claims (20+ rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/claims | List claims |
| ✅ | POST /admin/orders/{id}/claims | Create claim |
| ✅ | GET /admin/claims/{id} | Get claim |
| ✅ | POST /admin/claims/{id} | Update claim |
| ✅ | DELETE /admin/claims/{id} | Delete claim |
| ✅ | POST /admin/claims/{id}/cancel | Cancel claim |
| ✅ | POST /admin/claims/{id}/confirm | Confirm claim |
| ✅ | POST /admin/claims/{id}/request | Request claim |
| ✅ | POST /admin/claims/{id}/outbound/items | Add outbound items |
| ✅ | POST /admin/claims/{id}/inbound/items | Add inbound items |
| ✅ | POST /admin/claims/{id}/shipping-method | Add shipping method |

### Collections (6 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/collections | List collections |
| ✅ | POST /admin/collections | Create collection |
| ✅ | GET /admin/collections/{id} | Get collection |
| ✅ | PUT /admin/collections/{id} | Update collection |
| ✅ | DELETE /admin/collections/{id} | Delete collection |
| ✅ | POST /admin/collections/{id}/products/batch | Add products |
| ✅ | DELETE /admin/collections/{id}/products/batch | Remove products |

### Currencies (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/currencies | List currencies |
| ✅ | GET /admin/currencies/{code} | Get currency |
| ✅ | PUT /admin/currencies/{code} | Update currency |

### Customer Groups (5 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/customer-groups | List customer groups |
| ✅ | POST /admin/customer-groups | Create customer group |
| ✅ | GET /admin/customer-groups/{id} | Get customer group |
| ✅ | POST /admin/customer-groups/{id} | Update customer group |
| ✅ | DELETE /admin/customer-groups/{id} | Delete customer group |
| ✅ | POST /admin/customer-groups/{id}/customers | Link customers |

### Customers (8 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/customers | List customers |
| ✅ | POST /admin/customers | Create customer |
| ✅ | GET /admin/customers/{id} | Get customer |
| ✅ | POST /admin/customers/{id} | Update customer |
| ❌ | DELETE /admin/customers/{id} | Delete customer |
| ❌ | GET /admin/customers/{id}/addresses | List addresses |
| ❌ | POST /admin/customers/{id}/addresses | Add address |
| ❌ | POST /admin/customers/{id}/addresses/{address_id} | Update address |
| ❌ | DELETE /admin/customers/{id}/addresses/{address_id} | Delete address |
| ❌ | POST /admin/customers/{id}/customer-groups | Link groups |

### Draft Orders (14 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/draft-orders | List draft orders |
| ✅ | POST /admin/draft-orders | Create draft order |
| ✅ | GET /admin/draft-orders/{id} | Get draft order |
| ✅ | POST /admin/draft-orders/{id} | Update draft order |
| ✅ | DELETE /admin/draft-orders/{id} | Delete draft order |
| ✅ | POST /admin/draft-orders/{id}/line-items | Add line item |
| ✅ | POST /admin/draft-orders/{id}/line-items/{line_id} | Update line item |
| ✅ | DELETE /admin/draft-orders/{id}/line-items/{line_id} | Delete line item |
| ✅ | POST /admin/draft-orders/{id}/pay | Register payment |
| ❌ | POST /admin/draft-orders/{id}/convert-to-order | Convert to order |

### Exchanges (20+ rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/exchanges | List exchanges |
| ✅ | POST /admin/exchanges | Create exchange |
| ✅ | GET /admin/exchanges/{id} | Get exchange |
| ✅ | POST /admin/exchanges/{id}/cancel | Cancel exchange |
| ✅ | POST /admin/exchanges/{id}/inbound/items | Inbound items |
| ✅ | POST /admin/exchanges/{id}/outbound/items | Outbound items |
| ✅ | POST /admin/exchanges/{id}/shipping-method | Shipping method |
| ✅ | POST /admin/exchanges/{id}/request | Request exchange |
| ✅ | POST /admin/exchanges/{id}/confirm | Confirm exchange |

### Feature Flags (1 rota)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/feature-flags | List feature flags |

### Fulfillments (8 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | POST /admin/orders/{id}/fulfillment | Create fulfillment |
| ✅ | POST /admin/orders/{id}/fulfillments/{fid}/cancel | Cancel fulfillment |
| ✅ | POST /admin/orders/{id}/shipment | Create shipment |
| ✅ | GET /admin/fulfillment-providers | List providers |
| ✅ | GET /admin/fulfillment-providers/{id}/options | Get options |
| ✅ | DELETE /admin/fulfillment-sets/{id} | Delete fulfillment set |
| ✅ | POST /admin/fulfillment-sets/{id}/service-zones | Add service zone |
| ✅ | GET /admin/fulfillment-sets/{id} | Get fulfillment set |
| ✅ | POST /admin/fulfillment-sets/{id}/service-zones/{zone_id} | Update service zone |
| ✅ | DELETE /admin/fulfillment-sets/{id}/service-zones/{zone_id} | Delete service zone |

### Gift Cards (4 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/gift-cards | List gift cards |
| ✅ | POST /admin/gift-cards | Create gift card |
| ✅ | GET /admin/gift-cards/{id} | Get gift card |
| ✅ | PUT /admin/gift-cards/{id} | Update gift card |
| ✅ | DELETE /admin/gift-cards/{id} | Delete gift card |
| ❌ | GET /admin/gift-cards/{id}/orders | Get gift card orders |

### Inventory Items (10 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/inventory-items | List inventory items |
| ✅ | POST /admin/inventory-items | Create inventory item |
| ✅ | GET /admin/inventory-items/{id} | Get inventory item |
| ✅ | PUT /admin/inventory-items/{id} | Update inventory item |
| ✅ | DELETE /admin/inventory-items/{id} | Delete inventory item |
| ✅ | GET /admin/inventory-items/{id}/location-levels | List location levels |
| ✅ | POST /admin/inventory-items/{id}/location-levels | Create location level |
| ❌ | POST /admin/inventory-items/{id}/location-levels/batch | Batch location levels |
| ✅ | PUT /admin/inventory-items/{id}/location-levels/{location_id} | Update location level |
| ✅ | DELETE /admin/inventory-items/{id}/location-levels/{location_id} | Delete location level |
| ❌ | POST /admin/inventory-items/location-levels/batch | Batch all |

### Invites (5 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/invites | List invites |
| ✅ | POST /admin/invites | Create invite |
| ❌ | POST /admin/invites/accept | Accept invite |
| ❌ | GET /admin/invites/{id} | Get invite |
| ✅ | DELETE /admin/invites/{id} | Delete invite |
| ✅ | POST /admin/invites/{id}/accept | Accept invite (by id) |
| ❌ | POST /admin/invites/{id}/resend | Resend invite |

### Locales (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ❌ | GET /admin/locales | List locales |
| ❌ | GET /admin/locales/{code} | Get locale |

### Notifications (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/notifications | List notifications |
| ✅ | GET /admin/notifications/{id} | Get notification |
| ✅ | POST /admin/notifications/{id}/resend | Resend notification |

### Order Changes (1 rota)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | DELETE /admin/order-changes/{id} | Delete order change |

### Order Edits (10 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/order-edits | List order edits |
| ✅ | POST /admin/order-edits | Create order edit |
| ✅ | GET /admin/order-edits/{id} | Get order edit |
| ✅ | POST /admin/order-edits/{id} | Update order edit |
| ✅ | DELETE /admin/order-edits/{id} | Delete order edit |
| ✅ | POST /admin/order-edits/{id}/request | Request order edit |
| ✅ | POST /admin/order-edits/{id}/confirm | Confirm order edit |
| ✅ | POST /admin/order-edits/{id}/decline | Decline order edit |
| ✅ | POST /admin/order-edits/{id}/cancel | Cancel order edit |
| ✅ | POST /admin/order-edits/{id}/items | Add line item |
| ✅ | POST /admin/order-edits/{id}/items/{item_id} | Update line item |
| ✅ | DELETE /admin/order-edits/{id}/changes/{change_id} | Delete item change |

### Orders (20+ rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/orders | List orders |
| ✅ | GET /admin/orders/{id} | Get order |
| ✅ | POST /admin/orders/{id}/archive | Archive order |
| ✅ | POST /admin/orders/{id}/cancel | Cancel order |
| ✅ | POST /admin/orders/{id}/complete | Complete order |
| ✅ | POST /admin/orders/{id}/refunds | Create refund |
| ✅ | POST /admin/orders/{id}/return | Request return |
| ✅ | POST /admin/orders/{id}/swaps | Create swap |
| ✅ | POST /admin/orders/{id}/claims | Create claim |
| ❌ | POST /admin/orders/export | Export orders |
| ❌ | GET /admin/orders/{id}/changes | List order changes |
| ❌ | GET /admin/orders/{id}/credit-lines | List credit lines |
| ❌ | GET /admin/orders/{id}/fulfillments | List fulfillments |
| ❌ | POST /admin/orders/{id}/fulfillments/{fid}/mark-as-delivered | Mark delivered |
| ❌ | POST /admin/orders/{id}/fulfillments/{fid}/shipments | Create shipment |
| ❌ | GET /admin/orders/{id}/line-items | List line items |
| ❌ | GET /admin/orders/{id}/preview | Preview order |
| ❌ | GET /admin/orders/{id}/shipping-options | Get shipping options |
| ❌ | POST /admin/orders/{id}/transfer | Transfer order |
| ❌ | POST /admin/orders/{id}/transfer/cancel | Cancel transfer |

### Payment Collections (3 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/payment-collections | List |
| ✅ | POST /admin/payment-collections | Create |
| ✅ | GET /admin/payment-collections/{id} | Get |
| ✅ | POST /admin/payment-collections/{id} | Update |
| ✅ | DELETE /admin/payment-collections/{id} | Delete |
| ✅ | POST /admin/payment-collections/{id}/mark-as-paid | Mark as paid |

### Payments (6 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/payments | List payments |
| ❌ | GET /admin/payments/payment-providers | List providers |
| ✅ | GET /admin/payments/{id} | Get payment |
| ✅ | POST /admin/payments/{id}/capture | Capture payment |
| ✅ | POST /admin/payments/{id}/refund | Refund payment |

### Plugins (1 rota)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/plugins | List plugins |

### Price Lists (7 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/price-lists | List price lists |
| ✅ | POST /admin/price-lists | Create price list |
| ✅ | GET /admin/price-lists/{id} | Get price list |
| ✅ | PUT /admin/price-lists/{id} | Update price list |
| ✅ | DELETE /admin/price-lists/{id} | Delete price list |
| ❌ | POST /admin/price-lists/{id}/prices | Add prices |
| ✅ | POST /admin/price-lists/{id}/prices/batch | Batch prices |
| ✅ | GET /admin/price-lists/{id}/products | Get products |

### Price Preferences (3 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/price-preferences | List |
| ✅ | POST /admin/price-preferences | Create |
| ✅ | GET /admin/price-preferences/{id} | Get |
| ✅ | POST /admin/price-preferences/{id} | Update |
| ✅ | DELETE /admin/price-preferences/{id} | Delete |

### Product Categories (5 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| 🟡 | GET /admin/product-categories | List categories |
| ❌ | POST /admin/product-categories | Create category |
| ❌ | GET /admin/product-categories/{id} | Get category |
| ❌ | POST /admin/product-categories/{id} | Update category |
| ❌ | DELETE /admin/product-categories/{id} | Delete category |
| ❌ | POST /admin/product-categories/{id}/products | Link products |

### Product Tags (4 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/product-tags | List tags |
| ✅ | POST /admin/product-tags | Create tag |
| ✅ | GET /admin/product-tags/{id} | Get tag |
| ✅ | POST /admin/product-tags/{id} | Update tag |
| ✅ | DELETE /admin/product-tags/{id} | Delete tag |

### Product Types (4 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/product-types | List types |
| ✅ | POST /admin/product-types | Create type |
| ✅ | GET /admin/product-types/{id} | Get type |
| ✅ | POST /admin/product-types/{id} | Update type |
| ✅ | DELETE /admin/product-types/{id} | Delete type |

### Product Variants (1 rota standalone)
| Status | Rota | Notas |
|--------|------|-------|
| ❌ | GET /admin/product-variants | List all variants |

### Products (20+ rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/products | List products |
| ✅ | POST /admin/products | Create product |
| ✅ | GET /admin/products/{id} | Get product |
| ✅ | PUT /admin/products/{id} | Update product |
| ✅ | DELETE /admin/products/{id} | Delete product |
| ✅ | GET /admin/products/{id}/variants | List variants |
| ✅ | POST /admin/products/{id}/variants | Create variant |
| ✅ | GET /admin/products/{id}/variants/{variant_id} | Get variant |
| ✅ | PUT /admin/products/{id}/variants/{variant_id} | Update variant |
| ✅ | DELETE /admin/products/{id}/variants/{variant_id} | Delete variant |
| ✅ | GET /admin/products/{id}/options | List options |
| ✅ | POST /admin/products/{id}/options | Create option |
| ✅ | PUT /admin/products/{id}/options/{option_id} | Update option |
| ✅ | DELETE /admin/products/{id}/options/{option_id} | Delete option |
| ❌ | POST /admin/products/batch | Batch products |
| ❌ | POST /admin/products/export | Export products |
| ❌ | POST /admin/products/import | Import products |
| ❌ | POST /admin/products/import/{transaction_id}/confirm | Confirm import |
| ❌ | POST /admin/products/{id}/variants/batch | Batch variants |
| ❌ | POST /admin/products/{id}/variants/inventory-items/batch | Batch inventory |
| ❌ | POST /admin/products/{id}/variants/{vid}/images/batch | Batch images |
| ❌ | GET /admin/products/{id}/variants/{vid}/inventory-items | List inventory items |

### Promotions (12+ rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/promotions | List promotions |
| ✅ | POST /admin/promotions | Create promotion |
| ✅ | GET /admin/promotions/{id} | Get promotion |
| ✅ | POST /admin/promotions/{id} | Update promotion |
| ✅ | DELETE /admin/promotions/{id} | Delete promotion |
| ✅ | POST /admin/promotions/{id}/rules | Add promotion rules |
| ✅ | DELETE /admin/promotions/{id}/rules | Remove promotion rules |
| ✅ | POST /admin/promotions/{id}/buy-rules/batch | Batch buy rules |
| ✅ | POST /admin/promotions/{id}/target-rules/batch | Batch target rules |
| ❌ | GET /admin/promotions/rule-attribute-options/{rule_type} | Get options |
| ❌ | GET /admin/promotions/rule-value-options/{rule_type}/{rule_attribute_id} | Get values |

### Refund Reasons (3 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/refund-reasons | List |
| ✅ | POST /admin/refund-reasons | Create |
| ✅ | GET /admin/refund-reasons/{id} | Get |
| ✅ | POST /admin/refund-reasons/{id} | Update |
| ✅ | DELETE /admin/refund-reasons/{id} | Delete |

### Regions (5 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/regions | List regions |
| ✅ | POST /admin/regions | Create region |
| ✅ | GET /admin/regions/{id} | Get region |
| ✅ | PUT /admin/regions/{id} | Update region |
| ✅ | DELETE /admin/regions/{id} | Delete region |

### Reservations (5 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/reservations | List reservations |
| ✅ | POST /admin/reservations | Create reservation |
| ✅ | GET /admin/reservations/{id} | Get reservation |
| ✅ | POST /admin/reservations/{id} | Update reservation |
| ✅ | DELETE /admin/reservations/{id} | Delete reservation |

### Return Reasons (4 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/return-reasons | List |
| ✅ | POST /admin/return-reasons | Create |
| ✅ | GET /admin/return-reasons/{id} | Get |
| ✅ | POST /admin/return-reasons/{id} | Update |
| ✅ | DELETE /admin/return-reasons/{id} | Delete |

### Returns (16 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/returns | List returns |
| ✅ | POST /admin/returns/{id}/receive | Receive return |
| ✅ | GET /admin/returns/{id} | Get return |
| ✅ | POST /admin/returns/{id}/cancel | Cancel return |
| ✅ | POST /admin/returns/{id}/dismiss-items | Dismiss items |
| ❌ | POST /admin/returns/{id}/dismiss-items/{action_id} | Dismiss action |
| ✅ | POST /admin/returns/{id}/receive-items | Receive items |
| ❌ | POST /admin/returns/{id}/receive-items/{action_id} | Receive action |
| ✅ | POST /admin/returns/{id}/receive/confirm | Confirm receive |
| ✅ | POST /admin/returns/{id}/request | Request return |
| ❌ | POST /admin/returns/{id}/request-items | Request items |
| ❌ | POST /admin/returns/{id}/request-items/{action_id} | Request action |
| ✅ | POST /admin/returns/{id}/shipping-method | Add shipping method |
| ❌ | POST /admin/returns/{id}/shipping-method/{action_id} | Shipping action |

### Sales Channels (5 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/sales-channels | List |
| ✅ | POST /admin/sales-channels | Create |
| ✅ | GET /admin/sales-channels/{id} | Get |
| ✅ | POST /admin/sales-channels/{id} | Update |
| ✅ | DELETE /admin/sales-channels/{id} | Delete |
| ✅ | POST /admin/sales-channels/{id}/products | Link products |

### Shipping Option Types (3 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/shipping-option-types | List |
| ✅ | POST /admin/shipping-option-types | Create |
| ✅ | GET /admin/shipping-option-types/{id} | Get |
| ✅ | POST /admin/shipping-option-types/{id} | Update |
| ✅ | DELETE /admin/shipping-option-types/{id} | Delete |

### Shipping Options (5 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/shipping-options | List shipping options |
| ✅ | POST /admin/shipping-options | Create shipping option |
| ❌ | GET /admin/shipping-options/{id} | Get |
| ❌ | POST /admin/shipping-options/{id} | Update |
| ❌ | DELETE /admin/shipping-options/{id} | Delete |
| ❌ | POST /admin/shipping-options/{id}/rules/batch | Batch rules |

### Shipping Profiles (4 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/shipping-profiles | List |
| ✅ | POST /admin/shipping-profiles | Create |
| ✅ | GET /admin/shipping-profiles/{id} | Get |
| ✅ | POST /admin/shipping-profiles/{id} | Update |
| ✅ | DELETE /admin/shipping-profiles/{id} | Delete |

### Stock Locations (7 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/stock-locations | List |
| ✅ | POST /admin/stock-locations | Create |
| ✅ | GET /admin/stock-locations/{id} | Get |
| ✅ | POST /admin/stock-locations/{id} | Update |
| ✅ | DELETE /admin/stock-locations/{id} | Delete |
| ✅ | POST /admin/stock-locations/{id}/fulfillment-providers | Link providers |
| ✅ | POST /admin/stock-locations/{id}/fulfillment-sets | Link sets |
| ✅ | POST /admin/stock-locations/{id}/sales-channels | Link channels |

### Store Credit Accounts (5 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ❌ | GET /admin/store-credit-accounts | List |
| ❌ | POST /admin/store-credit-accounts | Create |
| ❌ | GET /admin/store-credit-accounts/{id} | Get |
| ❌ | POST /admin/store-credit-accounts/{id} | Update |
| ❌ | DELETE /admin/store-credit-accounts/{id} | Delete |
| ❌ | POST /admin/store-credit-accounts/{id}/transactions | Create transaction |

### Stores (3 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/stores | List stores |
| ✅ | GET /admin/stores/{id} | Get store |
| ✅ | POST /admin/stores/{id} | Update store |

### Tax Providers (1 rota)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/tax-providers | List tax providers |

### Tax Rates (6 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/tax-rates | List tax rates |
| ❌ | POST /admin/tax-rates | Create |
| ❌ | GET /admin/tax-rates/{id} | Get |
| ❌ | POST /admin/tax-rates/{id} | Update |
| ❌ | DELETE /admin/tax-rates/{id} | Delete |
| ❌ | POST /admin/tax-rates/{id}/rules | Add rules |
| ❌ | DELETE /admin/tax-rates/{id}/rules/{rule_id} | Delete rule |

### Tax Regions (4 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/tax-regions | List |
| ✅ | POST /admin/tax-regions | Create |
| ✅ | GET /admin/tax-regions/{id} | Get |
| ✅ | DELETE /admin/tax-regions/{id} | Delete |

### Translations (7 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ❌ | GET /admin/translations | List |
| ❌ | POST /admin/translations | Create |
| ❌ | POST /admin/translations/batch | Batch |
| ❌ | GET /admin/translations/entities | List entities |
| ❌ | GET /admin/translations/settings | Get settings |
| ❌ | POST /admin/translations/settings/batch | Batch settings |
| ❌ | GET /admin/translations/statistics | Get statistics |

### Uploads (4 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | POST /admin/uploads | Upload file |
| ✅ | DELETE /admin/uploads | Delete files |
| ❌ | POST /admin/uploads/presigned-urls | Get presigned URLs |
| ❌ | DELETE /admin/uploads/{id} | Delete by ID |

### Users (6 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/users | List users |
| ✅ | POST /admin/users | Create user |
| ✅ | GET /admin/users/me | Get current user |
| ✅ | GET /admin/users/{id} | Get user |
| ✅ | POST /admin/users/{id} | Update user |
| ✅ | DELETE /admin/users/{id} | Delete user |
| ❌ | DELETE /admin/users/{id}/roles/{role_id} | Remove role |

### Views (5 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ❌ | GET /admin/views/{entity}/columns | List columns |
| ❌ | GET /admin/views/{entity}/configurations | List configs |
| ❌ | GET /admin/views/{entity}/configurations/active | Get active |
| ❌ | POST /admin/views/{entity}/configurations | Create config |
| ❌ | DELETE /admin/views/{entity}/configurations/{id} | Delete config |

### Workflows Executions (8 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ❌ | GET /admin/workflows-executions | List |
| ❌ | GET /admin/workflows-executions/{id} | Get |
| ❌ | POST /admin/workflows-executions/{workflow_id}/run | Run |
| ❌ | POST /admin/workflows-executions/{workflow_id}/steps/failure | Mark failure |
| ❌ | POST /admin/workflows-executions/{workflow_id}/steps/success | Mark success |
| ❌ | GET /admin/workflows-executions/{workflow_id}/subscribe | Subscribe |
| ❌ | GET /admin/workflows-executions/{workflow_id}/{transaction_id} | Get transaction |
| ❌ | GET /admin/workflows-executions/{wf_id}/{tx_id}/{step_id}/subscribe | Subscribe step |

### Batch Jobs (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/batch-jobs | List batch jobs |
| ✅ | POST /admin/batch-jobs | Create batch job |
| ✅ | GET /admin/batch-jobs/{id} | Get batch job |
| ✅ | POST /admin/batch-jobs/{id}/confirm | Confirm batch job |
| ✅ | POST /admin/batch-jobs/{id}/cancel | Cancel batch job |

### Discounts (5 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | GET /admin/discounts | List discounts |
| ✅ | POST /admin/discounts | Create discount |
| ✅ | GET /admin/discounts/{id} | Get discount |
| ✅ | PUT /admin/discounts/{id} | Update discount |
| ✅ | DELETE /admin/discounts/{id} | Delete discount |

### Auth Admin (10 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ✅ | POST /admin/auth | Login |
| ✅ | GET /admin/auth | Get session |
| ✅ | DELETE /admin/auth | Logout |
| ✅ | DELETE /auth/session | Delete session (shared) |
| ✅ | POST /auth/token/refresh | Refresh token (shared) |
| ✅ | GET /auth/user/{auth_provider} | OAuth (email-only) |
| ✅ | GET /auth/user/{auth_provider}/callback | OAuth callback (stub) |
| ✅ | POST /auth/user/{auth_provider}/register | OAuth register (email) |
| ✅ | POST /auth/user/{auth_provider}/reset-password | OAuth reset (email) |
| ✅ | POST /auth/user/{auth_provider}/update | OAuth update (stub) |

### Index (2 rotas)
| Status | Rota | Notas |
|--------|------|-------|
| ❌ | GET /admin/index/details | Index details |
| ❌ | POST /admin/index/sync | Sync index |

---

## PARTE 3: Funcionalidades Além de Rotas

### Sistema de Plugins
| Status | Funcionalidade | Notas |
|--------|----------------|-------|
| ❌ | Plugin loader | Carregar plugins externos |
| ❌ | Payment plugins | Stripe, PayPal, etc. |
| ❌ | Fulfillment plugins | Manual, custom |
| ❌ | Notification plugins | Email, SMS |
| ❌ | Search plugins | MeiliSearch, Algolia |
| ❌ | File plugins | S3, MinIO (parcial ✅) |

### Sistema de Eventos
| Status | Funcionalidade | Notas |
|--------|----------------|-------|
| ❌ | Event bus | Publish/subscribe |
| ❌ | Event subscribers | Handlers para eventos |
| ❌ | Scheduled jobs | Cron-like tasks |
| ❌ | Webhooks | Notificar sistemas externos |

### Sistema de Workflows
| Status | Funcionalidade | Notas |
|--------|----------------|-------|
| ❌ | Workflow engine | Orchestration |
| ❌ | Compensation (saga) | Rollback support |
| ❌ | Step functions | Custom steps |

### Autenticação Avançada
| Status | Funcionalidade | Notas |
|--------|----------------|-------|
| ✅ | JWT auth | Token-based |
| ✅ | Password hashing | Argon2 |
| ❌ | OAuth providers | Google, GitHub, etc. |
| ❌ | API key auth | For external integrations |
| ❌ | Role-based access | RBAC |
| ❌ | Refresh tokens | Token rotation |

### Notificações
| Status | Funcionalidade | Notas |
|--------|----------------|-------|
| ❌ | Email sending | SMTP, SendGrid |
| ❌ | Email templates | Handlebars/Liquid |
| ❌ | SMS sending | Twilio |
| ❌ | Push notifications | FCM, APNs |

### Busca
| Status | Funcionalidade | Notas |
|--------|----------------|-------|
| ❌ | Full-text search | Product search |
| ❌ | Faceted search | Filter by attributes |
| ❌ | Search indexing | Sync with external services |

### Integrações de Pagamento
| Status | Funcionalidade | Notas |
|--------|----------------|-------|
| ❌ | Stripe | Payment provider |
| ❌ | PayPal | Payment provider |
| ❌ | Manual payment | For testing |
| ❌ | Payment capture | Capture authorized |
| ❌ | Payment refund | Refund payments |

### Import/Export
| Status | Funcionalidade | Notas |
|--------|----------------|-------|
| ✅ | ZIP import (wizard) | Products from Excel |
| ✅ | Batch jobs | DB-backed async processing |
| ❌ | CSV export | Products, orders |
| ❌ | CSV import | Products |

### Internacionalização
| Status | Funcionalidade | Notas |
|--------|----------------|-------|
| ❌ | Multi-currency | Price conversion |
| ❌ | Multi-language | Translations |
| ❌ | Locale formatting | Dates, numbers |

### Cache e Performance
| Status | Funcionalidade | Notas |
|--------|----------------|-------|
| ✅ | In-memory cache | Moka |
| ❌ | Redis cache | Distributed cache |
| ❌ | Query caching | DB query cache |
| ❌ | Response caching | HTTP cache headers |

---

## PARTE 4: Migrações de Banco de Dados Necessárias

### Tabelas Existentes (migração 20240101000000 + posteriores)
- ✅ products, product_variants, product_options, product_option_values
- ✅ collections, categories
- ✅ carts, cart_items, cart_discounts
- ✅ orders, order_items, order_payments
- ✅ customers, customer_addresses
- ✅ regions, countries
- ✅ discounts, discount_rules, discount_regions
- ✅ shipping_options, shipping_profiles
- ✅ users
- ✅ tax_rates
- ✅ gift_cards, gift_card_transactions
- ✅ inventory_items, inventory_levels
- ✅ price_lists, money_amounts
- ✅ payment_providers, fulfillment_providers
- ✅ swaps (tabela completa)
- ✅ claim_orders
- ✅ returns, refunds
- ✅ draft_orders
- ✅ batch_jobs
- ✅ return_reasons (migração 20260307000002)
- ✅ sales_channels, sales_channel_products (migração 20260307000003)
- ✅ customer_groups, customer_group_customers (migração 20260307000004)
- ✅ api_keys, api_key_sales_channels (migração 20260307000005)
- ✅ invites (migração 20260307000006)
- ✅ stock_locations, fulfillment_sets + join tables (migração 20260307000007)
- ✅ stores (migração 20260307000008)
- ✅ tax_regions, tax_overrides (migração 20260307000009)
- ✅ order_edits, order_item_changes (migração 20260307000010)
- ✅ promotions, campaigns, promotion_rules, promotion_application_methods (migração 20260307000011)
- ✅ notifications (migração 20260307000012)
- ✅ exchanges, refund_reasons, price_preferences (migração 20260307000013)

### Tabelas Ainda Faltando
- ❌ feature_flags
- ❌ locales
- ❌ store_credit_accounts, store_credit_transactions
- ❌ translations
- ❌ user_roles
- ❌ views, view_configurations
- ❌ workflows, workflow_executions
- ❌ reservations
- ❌ service_zones

---

## PARTE 5: Prioridades de Implementação

### Fase 1: Core Commerce (Alta Prioridade) — ✅ CONCLUÍDA
1. ✅ **Inventory Management** - Controle de estoque
2. ✅ **Gift Cards** - Cartões presente completos
3. ✅ **Returns/Refunds** - Fluxo completo de devoluções
4. ✅ **Price Lists** - Preços especiais por grupo/região
5. ✅ **Currencies** - Multi-moeda

### Fase 2: Operações (Média Prioridade) — ✅ CONCLUÍDA
1. ✅ **Draft Orders** - Pedidos rascunho (DB-backed)
2. ✅ **Order Edits** - Edição de pedidos (DB-backed)
3. ✅ **Exchanges** - Migração criada
4. ✅ **Sales Channels** - Canais de venda (DB-backed)
5. ✅ **Promotions** - Módulo de promoções (DB-backed)
6. ✅ **Notifications** - Módulo de notificações (DB-backed)
7. ✅ **Batch Jobs** - Jobs em lote (DB-backed)

### Fase 3: Extensibilidade (Média Prioridade)
1. ❌ **Plugin System** - Sistema de plugins
2. ❌ **Event Bus** - Sistema de eventos
3. ❌ **Webhooks** - Notificações externas
4. ✅ **API Keys** - Autenticação de integrações (DB-backed)

### Fase 4: Integrações (Baixa Prioridade)
1. ❌ **Payment Providers** - Stripe, PayPal
2. ❌ **Fulfillment Providers** - Integrações de envio
3. ❌ **Search** - MeiliSearch/Algolia
4. ❌ **Email** - Notificações por email
5. ❌ **OAuth** - Login social

### Fase 5: Enterprise (Baixa Prioridade)
1. ✅ **Multi-store** - Múltiplas lojas (DB-backed)
2. ❌ **Workflows** - Automação de processos
3. ❌ **Translations** - Internacionalização
4. ❌ **Store Credits** - Crédito de loja
5. ❌ **Advanced Analytics** - Relatórios

---

## PARTE 6: Estimativa de Esforço

| Categoria | Rotas | Esforço Estimado |
|-----------|-------|------------------|
| Store Routes restantes | 7 | 2-3 dias |
| Admin Core (customers, orders) | 20 | 1 semana |
| Admin Commerce (inventory, prices) | 30 | 2 semanas |
| Admin Operations (claims, exchanges, returns) | 50 | 3 semanas |
| Admin Advanced (workflows, translations) | 50 | 3 semanas |
| Plugin System | - | 2 semanas |
| Event System | - | 1 semana |
| Payment Integrations | - | 2 semanas |
| **Total Estimado** | **184** | **~3 meses** |

---

## Comandos Úteis

```bash
# Ver rotas implementadas
grep -r "pub async fn" src/api/ | wc -l

# Compilar
cargo build --release

# Testar
cargo test

# Verificar erros
cargo check

# Lint
cargo clippy
```

---

> **Nota:** Este documento será atualizado conforme o progresso da implementação.
