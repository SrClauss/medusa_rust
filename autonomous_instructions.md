# Autonomous Agent Instructions — Medusa Rust Port

> **Target:** Complete port of Medusa JS v2 to Rust. Current coverage: ~70%.  
> **Stack:** Axum + SQLx (PostgreSQL) + Moka cache + MinIO/S3  
> **Repository:** `/home/runner/work/medusa_rust/medusa_rust` (or local clone)

---

## How to Use This Document

These instructions are designed for incremental, autonomous work.  
Each **Phase** is a self-contained unit you can complete and commit independently.  
Always:
1. Read the phase objectives
2. Check `IMPLEMENTATION_PLAN.md` for current status
3. Implement, test, and commit
4. Update `IMPLEMENTATION_PLAN.md` status (✅/🟡/❌)
5. Move to the next phase

---

## Environment Setup

```bash
# Build
cargo build

# Run tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt

# Run server (requires PostgreSQL + .env configured)
cargo run
```

---

## Conventions

- **Handler stubs:** One-liner functions returning a plausible JSON stub are acceptable for routing.
  The goal is 100% route coverage first, then full DB/business logic later.
- **Handler pattern:**  
  ```rust
  pub async fn list(State(_): State<AppState>, Query(p): Query<ListParams>)
      -> Result<Json<serde_json::Value>, AppError>
  { Ok(Json(serde_json::json!({"items":[],"count":0,"offset":p.offset,"limit":p.limit}))) }
  ```
- **Route constants:** Add to `src/routes_manifest.rs` under the appropriate section.
- **Module registration:** Register in `src/api/admin/mod.rs` or `src/api/store/mod.rs`.
- **Tests:** Mirror the pattern in `tests/returns_tests.rs`: test auth requirement (401 without token),
  route existence (not 404 with token), and method allowance (not 405).

---

## Phase 1 — Admin Core Commerce (Priority: High)

**Goal:** Implement or complete the following admin route groups.

### 1a. Customer Groups (DONE ✅)
Routes: `GET/POST /admin/customer-groups`, `GET/POST/DELETE /admin/customer-groups/{id}`,
`GET/POST/DELETE /admin/customer-groups/{id}/customers`

### 1b. Price Preferences
File: `src/api/admin/price_preferences.rs`  
Routes:
- `GET /admin/price-preferences` → list
- `POST /admin/price-preferences` → create
- `GET /admin/price-preferences/{id}` → get
- `POST /admin/price-preferences/{id}` → update
- `DELETE /admin/price-preferences/{id}` → delete

Constants to add in `routes_manifest.rs`:
```rust
pub const ADMIN_PRICE_PREFERENCES: &str = "/admin/price-preferences";
pub const ADMIN_PRICE_PREFERENCES_ID: &str = "/admin/price-preferences/{id}";
```

### 1c. Promotions & Campaigns
Files: `src/api/admin/promotions.rs`, `src/api/admin/campaigns.rs`  
Promotions routes:
- `GET/POST /admin/promotions`
- `GET/POST/DELETE /admin/promotions/{id}`
- `POST /admin/promotions/{id}/rules/batch`
- `POST /admin/promotions/{id}/buy-rules/batch`
- `POST /admin/promotions/{id}/target-rules/batch`  

Campaigns routes:
- `GET/POST /admin/campaigns`
- `GET/POST/DELETE /admin/campaigns/{id}`
- `POST /admin/campaigns/{id}/promotions`

### 1d. Fulfillment Sets & Service Zones
File: `src/api/admin/fulfillment_sets.rs`  
Routes:
- `GET /admin/fulfillment-sets/{id}` → get
- `DELETE /admin/fulfillment-sets/{id}` → delete
- `POST /admin/fulfillment-sets/{id}/service-zones` → create service zone
- `POST /admin/fulfillment-sets/{id}/service-zones/{zone_id}` → update zone
- `DELETE /admin/fulfillment-sets/{id}/service-zones/{zone_id}` → delete zone

---

## Phase 2 — Admin Operations (Priority: High)

### 2a. Order Edits
File: `src/api/admin/order_edits.rs`  
Routes:
- `GET /admin/order-edits` → list
- `POST /admin/order-edits` → create
- `GET /admin/order-edits/{id}` → get
- `POST /admin/order-edits/{id}` → update
- `DELETE /admin/order-edits/{id}` → delete
- `POST /admin/order-edits/{id}/confirm` → confirm
- `POST /admin/order-edits/{id}/request` → request
- `POST /admin/order-edits/{id}/cancel` → cancel

### 2b. Claims
File: `src/api/admin/claims.rs`  
Routes:
- `GET /admin/claims` → list
- `POST /admin/orders/{id}/claims` → already exists (orders.rs)
- `GET /admin/claims/{id}` → get
- `POST /admin/claims/{id}` → update
- `DELETE /admin/claims/{id}` → delete
- `POST /admin/claims/{id}/outbound/items` → add outbound item
- `POST /admin/claims/{id}/confirm` → confirm

### 2c. Exchanges
File: `src/api/admin/exchanges.rs`  
Routes:
- `GET /admin/exchanges` → list
- `GET /admin/exchanges/{id}` → get
- `POST /admin/exchanges/{id}/confirm` → confirm
- `DELETE /admin/exchanges/{id}` → cancel

### 2d. Full Returns Admin
Extend: `src/api/admin/returns.rs`  
Missing routes:
- `GET /admin/returns/{id}` → get
- `POST /admin/returns/{id}/cancel` → cancel
- `POST /admin/returns/{id}/receive/items` → receive items
- `POST /admin/returns/{id}/receive/confirm` → confirm receive
- `POST /admin/returns/{id}/request` → request return

---

## Phase 3 — Admin Extensibility (Priority: Medium)

### 3a. Workflows Executions
File: `src/api/admin/workflow_executions.rs`  
Routes:
- `GET /admin/workflows-executions` → list
- `GET /admin/workflows-executions/{id}` → get
- `POST /admin/workflows-executions/{workflow_id}/run` → run

### 3b. Notifications
File: `src/api/admin/notifications.rs`  
Routes:
- `GET /admin/notifications` → list
- `POST /admin/notifications/{id}/resend` → resend

### 3c. Index
File: `src/api/admin/index.rs`  
Routes:
- `GET /admin/index/details` → list details
- `POST /admin/index/sync` → sync index

### 3d. Views / Configurations
File: `src/api/admin/views.rs`  
Routes:
- `GET /admin/views/{entity}/columns` → list columns
- `GET /admin/views/{entity}/configurations` → list configs
- `POST /admin/views/{entity}/configurations` → create config
- `DELETE /admin/views/{entity}/configurations/{id}` → delete config

---

## Phase 4 — Admin Advanced (Priority: Medium)

### 4a. Fulfillment Providers (Admin)
File: `src/api/admin/fulfillment_providers.rs`  
Routes:
- `GET /admin/fulfillment-providers` → list
- `GET /admin/fulfillment-providers/{id}` → get

### 4b. Payment Collections (Admin)
File: `src/api/admin/payment_collections.rs`  
Routes:
- `GET /admin/payment-collections` → list
- `POST /admin/payment-collections` → create
- `GET /admin/payment-collections/{id}` → get
- `POST /admin/payment-collections/{id}` → update
- `DELETE /admin/payment-collections/{id}` → delete

### 4c. Refund Reasons
File: `src/api/admin/refund_reasons.rs`  
Routes:
- `GET /admin/refund-reasons` → list
- `POST /admin/refund-reasons` → create
- `DELETE /admin/refund-reasons/{id}` → delete

### 4d. Reservations
File: `src/api/admin/reservations.rs`  
Routes:
- `GET /admin/reservations` → list
- `POST /admin/reservations` → create
- `GET /admin/reservations/{id}` → get
- `POST /admin/reservations/{id}` → update
- `DELETE /admin/reservations/{id}` → delete

---

## Phase 5 — Full Business Logic (Priority: Low → Long-term)

For routes already wired as stubs, implement real database queries following the
patterns in `src/api/admin/discounts.rs`, `src/api/admin/regions.rs`:
1. Parse payload/path params
2. Run `sqlx::query!` or `sqlx::query_as!`
3. Call `build_*` helper to serialize related entities
4. Return `Json(serde_json::json!({...}))`

Prioritize:
- Inventory Items (already stubbed, add real queries)
- Price Lists (already stubbed, add real queries)
- Gift Cards (admin, already stubbed, add real queries)
- Draft Orders (stubbed, add real queries)

---

## Testing Strategy

Each new module should have a matching test file in `tests/`.  
Name pattern: `tests/{resource}_tests.rs`

Minimal test set per route group:
```rust
// 1. Unauthenticated request → 401
// 2. Authenticated GET list → not 401, not 404
// 3. Authenticated POST create → not 401, not 404, not 405
// 4. Authenticated GET/{id} → not 401, not 404 (use random UUID)
// 5. Authenticated DELETE/{id} → not 401, not 404
```

Use `tests/returns_tests.rs` as the canonical template.

---

## Coverage Tracking

After each phase, update `IMPLEMENTATION_PLAN.md`:
- Change `❌` to `✅` for implemented routes
- Update the summary table percentages
- Commit with message: `feat: implement {resource} routes (Phase N)`

Current target: **≥ 80% total coverage** (implemented: ~70%).

---

## Notes

- Do **not** implement payment processing logic (Stripe, PayPal) until routing is 100% complete.
- Do **not** add new external dependencies without checking for existing alternatives in `Cargo.toml`.
- Always run `cargo test` after each phase to ensure no regressions.
- If a migration is needed for a new table, create it under `migrations/` with a timestamp prefix:
  `YYYYMMDDHHMMSS_description.sql`
