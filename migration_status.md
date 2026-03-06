# MedusaRust — Route Implementation Status

> Generated: 2026-03-05  
> Backend: Axum + SQLx (PostgreSQL) + Moka cache  
> Object storage: MinIO / AWS S3 via `aws-sdk-s3`

---

## Legend
| Symbol | Meaning |
|--------|---------|
| ✅ | Fully implemented — request/response matches MedusaJS format |
| 🟡 | Stubbed — route wired, business logic minimal (e.g. returns empty array) |
| ❌ | Not yet implemented |

---

## Store Routes (`/store/…`)

| # | Method | Path | Handler | Status |
|---|--------|------|---------|--------|
| 1 | POST | /store/auth | `store::auth::login` | ✅ |
| 2 | GET | /store/auth | `store::auth::get_session` | ✅ |
| 3 | DELETE | /store/auth | `store::auth::logout` | ✅ |
| 4 | GET | /store/products | `store::products::list_products` | ✅ |
| 5 | GET | /store/products/:id | `store::products::get_product` | ✅ |
| 6 | GET | /store/collections | `store::collections::list` | ✅ |
| 7 | GET | /store/collections/:id | `store::collections::get` | ✅ |
| 8 | GET | /store/product-categories | `store::categories::list` | ✅ |
| 9 | GET | /store/product-categories/:id | `store::categories::get` | ✅ |
| 10 | POST | /store/carts | `store::carts::create` | ✅ |
| 11 | GET | /store/carts/:id | `store::carts::get` | ✅ |
| 12 | POST | /store/carts/:id | `store::carts::update` | ✅ |
| 13 | POST | /store/carts/:id/line-items | `store::carts::add_line_item` | ✅ |
| 14 | POST | /store/carts/:id/line-items/:line_id | `store::carts::update_line_item` | ✅ |
| 15 | DELETE | /store/carts/:id/line-items/:line_id | `store::carts::remove_line_item` | ✅ |
| 16 | POST | /store/carts/:id/payment-sessions | `store::carts::create_payment_sessions` | ✅ |
| 17 | POST | /store/carts/:id/payment-session | `store::carts::select_payment_session` | ✅ |
| 18 | DELETE | /store/carts/:id/payment-sessions/:provider_id | `store::carts::delete_payment_session` | ✅ |
| 19 | POST | /store/carts/:id/payment-sessions/:provider_id/refresh | `store::carts::refresh_payment_session` | ✅ |
| 20 | POST | /store/carts/:id/payment-sessions/:provider_id | `store::carts::update_payment_session` | ✅ |
| 21 | POST | /store/carts/:id/shipping-methods | `store::carts::add_shipping_method` | ✅ |
| 22 | POST | /store/carts/:id/complete | `store::carts::complete` | ✅ |
| 23 | POST | /store/carts/:id/taxes | `store::carts::calculate_taxes` | ✅ |
| 24 | POST | /store/carts/:id/discounts/:code | `store::carts::apply_discount` | 🟡 |
| 25 | DELETE | /store/carts/:id/discounts/:code | `store::carts::remove_discount` | 🟡 |
| 26 | GET | /store/customers/me | `store::customers::get_me` | ✅ |
| 27 | POST | /store/customers | `store::customers::create_customer` | ✅ |
| 28 | POST | /store/customers/me | `store::customers::update_me` | ✅ |
| 29 | POST | /store/customers/password-token | `store::customers::request_password_reset` | 🟡 |
| 30 | POST | /store/customers/password-reset | `store::customers::reset_password` | ✅ |
| 31 | GET | /store/customers/me/orders | `store::customers::list_orders` | ✅ |
| 32 | GET | /store/customers/me/addresses | `store::customers::list_addresses` | ✅ |
| 33 | POST | /store/customers/me/addresses | `store::customers::add_address` | ✅ |
| 34 | GET | /store/customers/me/addresses/:address_id | `store::customers::get_address` | ✅ |
| 35 | POST | /store/customers/me/addresses/:address_id | `store::customers::update_address` | ✅ |
| 36 | DELETE | /store/customers/me/addresses/:address_id | `store::customers::delete_address` | ✅ |
| 37 | GET | /store/customers/me/payment-methods | `store::customers::list_payment_methods` | 🟡 |
| 38 | POST | /store/customers/me/payment-methods | `store::customers::add_payment_method` | 🟡 |
| 39 | GET | /store/orders/:id | `store::orders::get_order` | ✅ |
| 40 | GET | /store/orders | `store::orders::get_order_by_params` | ✅ |
| 41 | GET | /store/regions | `store::regions::list` | ✅ |
| 42 | GET | /store/regions/:id | `store::regions::get` | ✅ |
| 43 | GET | /store/shipping-options | `store::shipping_options::list` | ✅ |
| 44 | GET | /store/shipping-options/:cart_id | `store::shipping_options::get_for_cart` | ✅ |
| 45 | POST | /store/swaps | `store::swaps::create` | 🟡 |
| 46 | GET | /store/swaps/:cart_id | `store::swaps::get_by_cart` | ✅ |
| 47 | POST | /store/returns | (via admin returns) | 🟡 |

---

## Admin Routes (`/admin/…`)

| # | Method | Path | Handler | Status |
|---|--------|------|---------|--------|
| 1 | POST | /admin/auth | `admin::auth::login` | ✅ |
| 2 | GET | /admin/auth | `admin::auth::get_session` | ✅ |
| 3 | DELETE | /admin/auth | `admin::auth::logout` | ✅ |
| 4 | GET | /admin/products | `admin::products::list` | ✅ |
| 5 | POST | /admin/products | `admin::products::create` | ✅ |
| 6 | GET | /admin/products/:id | `admin::products::get` | ✅ |
| 7 | PUT | /admin/products/:id | `admin::products::update` | ✅ |
| 8 | DELETE | /admin/products/:id | `admin::products::delete_one` | ✅ |
| 9 | GET | /admin/products/:id/variants | `admin::products::list_variants` | ✅ |
| 10 | POST | /admin/products/:id/variants | `admin::products::create_variant` | ✅ |
| 11 | GET | /admin/products/:id/variants/:variant_id | `admin::products::get_variant` | ✅ |
| 12 | PUT | /admin/products/:id/variants/:variant_id | `admin::products::update_variant` | ✅ |
| 13 | DELETE | /admin/products/:id/variants/:variant_id | `admin::products::delete_variant` | ✅ |
| 14 | GET | /admin/products/:id/options | `admin::products::list_options` | ✅ |
| 15 | POST | /admin/products/:id/options | `admin::products::create_option` | ✅ |
| 16 | PUT | /admin/products/:id/options/:option_id | `admin::products::update_option` | ✅ |
| 17 | DELETE | /admin/products/:id/options/:option_id | `admin::products::delete_option` | ✅ |
| 18 | GET | /admin/collections | `admin::collections::list` | ✅ |
| 19 | POST | /admin/collections | `admin::collections::create` | ✅ |
| 20 | GET | /admin/collections/:id | `admin::collections::get` | ✅ |
| 21 | PUT | /admin/collections/:id | `admin::collections::update` | ✅ |
| 22 | DELETE | /admin/collections/:id | `admin::collections::delete_one` | ✅ |
| 23 | POST | /admin/collections/:id/products/batch | `admin::collections::add_products` | ✅ |
| 24 | DELETE | /admin/collections/:id/products/batch | `admin::collections::remove_products` | ✅ |
| 25 | GET | /admin/orders | `admin::orders::list` | ✅ |
| 26 | GET | /admin/orders/:id | `admin::orders::get` | ✅ (with line items + totals) |
| 27 | POST | /admin/orders/:id/complete | `admin::orders::complete` | ✅ |
| 28 | POST | /admin/orders/:id/cancel | `admin::orders::cancel` | ✅ |
| 29 | POST | /admin/orders/:id/archive | `admin::orders::archive` | ✅ |
| 30 | POST | /admin/orders/:id/fulfillment | `admin::orders::create_fulfillment` | ✅ |
| 31 | POST | /admin/orders/:id/fulfillments/:fulfillment_id/cancel | `admin::orders::cancel_fulfillment` | ✅ |
| 32 | POST | /admin/orders/:id/shipment | `admin::orders::create_shipment` | ✅ |
| 33 | POST | /admin/orders/:id/refunds | `admin::orders::create_refund` | 🟡 |
| 34 | POST | /admin/orders/:id/return | `admin::orders::request_return` | 🟡 |
| 35 | POST | /admin/orders/:id/swaps | `admin::orders::create_swap` | 🟡 |
| 36 | POST | /admin/orders/:id/claims | `admin::orders::create_claim` | 🟡 |
| 37 | GET | /admin/customers | `admin::customers::list` | ✅ |
| 38 | POST | /admin/customers | `admin::customers::create` | ✅ |
| 39 | GET | /admin/customers/:id | `admin::customers::get` | ✅ |
| 40 | POST | /admin/customers/:id | `admin::customers::update` | ✅ |
| 41 | GET | /admin/discounts | `admin::discounts::list` | ✅ |
| 42 | POST | /admin/discounts | `admin::discounts::create` | ✅ |
| 43 | GET | /admin/discounts/:id | `admin::discounts::get` | ✅ |
| 44 | PUT | /admin/discounts/:id | `admin::discounts::update` | ✅ |
| 45 | DELETE | /admin/discounts/:id | `admin::discounts::delete` | ✅ |
| 46 | GET | /admin/regions | `admin::regions::list` | ✅ |
| 47 | POST | /admin/regions | `admin::regions::create` | ✅ |
| 48 | GET | /admin/regions/:id | `admin::regions::get` | ✅ |
| 49 | PUT | /admin/regions/:id | `admin::regions::update` | ✅ |
| 50 | DELETE | /admin/regions/:id | `admin::regions::delete` | ✅ |
| 51 | GET | /admin/shipping-options | `admin::shipping_options::list` | ✅ |
| 52 | POST | /admin/shipping-options | `admin::shipping_options::create` | ✅ |
| 53 | GET | /admin/users | `admin::users::list` | ✅ |
| 54 | POST | /admin/users | `admin::users::create` | ✅ |
| 55 | POST | /admin/uploads | `admin::uploads::upload` | ✅ (S3/MinIO) |
| 56 | DELETE | /admin/uploads | `admin::uploads::delete_files` | ✅ (S3/MinIO) |
| 57 | GET | /admin/gift-cards | `admin::gift_cards::list` | 🟡 |
| 58 | POST | /admin/gift-cards | `admin::gift_cards::create` | 🟡 |
| 59 | GET | /admin/returns | `admin::returns::list` | 🟡 |
| 60 | POST | /admin/returns/:id/receive | `admin::returns::receive` | 🟡 |
| 61 | GET | /admin/draft-orders | `admin::draft_orders::list` | 🟡 |
| 62 | POST | /admin/draft-orders | `admin::draft_orders::create` | 🟡 |
| 63 | GET | /admin/batch-jobs | `admin::batch_jobs::list` | 🟡 |
| 64 | POST | /admin/batch-jobs | `admin::batch_jobs::create` | 🟡 |
| 65 | GET | /admin/inventory-items | `admin::inventory::list` | 🟡 |
| 66 | GET | /admin/price-lists | `admin::price_lists::list` | 🟡 |
| 67 | GET | /admin/tax-rates | `admin::tax_rates::list` | ✅ |
| 68 | GET | /admin/product-categories | `admin::categories::list` | 🟡 |

---

## Wizard Routes (`/wizard/…`)

| # | Method | Path | Description | Status |
|---|--------|------|-------------|--------|
| 1 | POST | /wizard/import | Upload zip (import.xlsx + assets/) | ✅ |
| 2 | GET | /wizard/import/:job_id | Poll import job status | 🟡 |

---

## Object Storage (MinIO / S3)

| Feature | Status | Notes |
|---------|--------|-------|
| MinIO container | ✅ | `docker-compose.yml` — port 9002 (API), 9001 (console) |
| AWS S3 support | ✅ | Set `S3_ENDPOINT=` (empty) + `S3_FORCE_PATH_STYLE=false` |
| Bucket auto-create | ✅ | `S3Storage::ensure_bucket_exists()` on startup |
| File upload (admin) | ✅ | `POST /admin/uploads` — multipart, returns public URLs |
| File delete (admin) | ✅ | `DELETE /admin/uploads` — by key |
| Wizard asset upload | ✅ | Assets from zip stored in bucket after DB commit |
| Presigned URLs | 🟡 | Planned — `GET /admin/uploads/presigned` |

---

## Remaining TODO (Next Prompt)

- [ ] Full discount calculation in cart (apply rule, compute discount_total)
- [ ] Real payment provider integration (Stripe, PayPal)
- [ ] Swap / claim / return full business logic
- [ ] Presigned URL endpoint for direct browser upload to MinIO
- [ ] Password-reset email sending (token generation done, email not sent)
- [ ] Fulfillment provider plugin system
- [ ] Admin region shipping-option full CRUD (GET/:id, PUT/:id, DELETE/:id)
- [ ] Admin users GET/:id, PUT/:id, DELETE/:id (routes wired but CRUD implemented)
- [ ] Inventory reservation on cart completion
- [ ] Admin returns/swaps/draft-orders/gift-cards/batch-jobs/price-lists/inventory full business logic
