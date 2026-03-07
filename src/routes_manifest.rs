//! Route-path manifest for MedusaRust.
//!
//! Every path constant declared here **must** be used in the Axum `Router`
//! composition in `src/api/`.  If a constant is left unused the Rust compiler
//! will emit a `dead_code` warning, giving an early signal that a route has
//! been declared but not yet wired up.
//!
//! Naming convention: `<SCOPE>_<RESOURCE>[_<SUB_RESOURCE>][_ID]`
//!   e.g. `ADMIN_PRODUCTS`, `ADMIN_PRODUCTS_ID`, `STORE_CARTS_ID_LINE_ITEMS`

#![allow(dead_code)] // Remove to restore compiler warnings for unlinked routes.

// ─── ADMIN ────────────────────────────────────────────────────────────────────

// Auth
pub const ADMIN_AUTH: &str = "/admin/auth";

// Global auth/provider routes (used by both store + admin)
pub const AUTH_ACTOR_PROVIDER: &str = "/auth/{actor_type}/{auth_provider}";
pub const AUTH_ACTOR_PROVIDER_CALLBACK: &str = "/auth/{actor_type}/{auth_provider}/callback";
pub const AUTH_ACTOR_PROVIDER_REGISTER: &str = "/auth/{actor_type}/{auth_provider}/register";
pub const AUTH_ACTOR_PROVIDER_RESET_PASSWORD: &str = "/auth/{actor_type}/{auth_provider}/reset-password";
pub const AUTH_ACTOR_PROVIDER_UPDATE: &str = "/auth/{actor_type}/{auth_provider}/update";
pub const AUTH_SESSION: &str = "/auth/session";
pub const AUTH_TOKEN_REFRESH: &str = "/auth/token/refresh";

// Products
pub const ADMIN_PRODUCTS: &str = "/admin/products";
pub const ADMIN_PRODUCTS_ID: &str = "/admin/products/{id}";
pub const ADMIN_PRODUCTS_ID_VARIANTS: &str = "/admin/products/{id}/variants";
pub const ADMIN_PRODUCTS_ID_VARIANTS_ID: &str = "/admin/products/{id}/variants/{variant_id}";
pub const ADMIN_PRODUCTS_ID_OPTIONS: &str = "/admin/products/{id}/options";
pub const ADMIN_PRODUCTS_ID_OPTIONS_ID: &str = "/admin/products/{id}/options/{option_id}";

// Collections
pub const ADMIN_COLLECTIONS: &str = "/admin/collections";
pub const ADMIN_COLLECTIONS_ID: &str = "/admin/collections/{id}";
pub const ADMIN_COLLECTIONS_ID_PRODUCTS_BATCH: &str = "/admin/collections/{id}/products/batch";

// Orders
pub const ADMIN_ORDERS: &str = "/admin/orders";
pub const ADMIN_ORDERS_ID: &str = "/admin/orders/{id}";
pub const ADMIN_ORDERS_ID_CANCEL: &str = "/admin/orders/{id}/cancel";
pub const ADMIN_ORDERS_ID_COMPLETE: &str = "/admin/orders/{id}/complete";
pub const ADMIN_ORDERS_ID_ARCHIVE: &str = "/admin/orders/{id}/archive";
pub const ADMIN_ORDERS_ID_FULFILLMENTS: &str = "/admin/orders/{id}/fulfillments";
pub const ADMIN_ORDERS_ID_FULFILLMENTS_ID_CANCEL: &str =
    "/admin/orders/{id}/fulfillments/{fulfillment_id}/cancel";
pub const ADMIN_ORDERS_ID_SHIPMENT: &str = "/admin/orders/{id}/shipment";
pub const ADMIN_ORDERS_ID_REFUNDS: &str = "/admin/orders/{id}/refunds";
pub const ADMIN_ORDERS_ID_RETURNS: &str = "/admin/orders/{id}/returns";
pub const ADMIN_ORDERS_ID_SWAPS: &str = "/admin/orders/{id}/swaps";
pub const ADMIN_ORDERS_ID_CLAIMS: &str = "/admin/orders/{id}/claims";

// Customers
pub const ADMIN_CUSTOMERS: &str = "/admin/customers";
pub const ADMIN_CUSTOMERS_ID: &str = "/admin/customers/{id}";

// Product Categories
pub const ADMIN_PRODUCT_CATEGORIES: &str = "/admin/product-categories";
pub const ADMIN_PRODUCT_CATEGORIES_ID: &str = "/admin/product-categories/{id}";
pub const ADMIN_PRODUCT_CATEGORIES_ID_PRODUCTS_BATCH: &str =
    "/admin/product-categories/{id}/products/batch";

// Regions
pub const ADMIN_REGIONS: &str = "/admin/regions";
pub const ADMIN_REGIONS_ID: &str = "/admin/regions/{id}";
pub const ADMIN_REGIONS_ID_COUNTRIES: &str = "/admin/regions/{id}/countries";
pub const ADMIN_REGIONS_ID_COUNTRIES_CODE: &str = "/admin/regions/{id}/countries/{country_code}";
pub const ADMIN_REGIONS_ID_FULFILLMENT_PROVIDERS: &str =
    "/admin/regions/{id}/fulfillment-providers";
pub const ADMIN_REGIONS_ID_FULFILLMENT_PROVIDERS_ID: &str =
    "/admin/regions/{id}/fulfillment-providers/{provider_id}";
pub const ADMIN_REGIONS_ID_PAYMENT_PROVIDERS: &str = "/admin/regions/{id}/payment-providers";
pub const ADMIN_REGIONS_ID_PAYMENT_PROVIDERS_ID: &str =
    "/admin/regions/{id}/payment-providers/{provider_id}";

// Discounts
pub const ADMIN_DISCOUNTS: &str = "/admin/discounts";
pub const ADMIN_DISCOUNTS_ID: &str = "/admin/discounts/{id}";
pub const ADMIN_DISCOUNTS_CODE: &str = "/admin/discounts/code/{code}";
pub const ADMIN_DISCOUNTS_ID_REGIONS_ID: &str = "/admin/discounts/{id}/regions/{region_id}";
pub const ADMIN_DISCOUNTS_ID_CONDITIONS: &str = "/admin/discounts/{id}/conditions";
pub const ADMIN_DISCOUNTS_ID_CONDITIONS_ID: &str =
    "/admin/discounts/{id}/conditions/{condition_id}";

// Shipping Options
pub const ADMIN_SHIPPING_OPTIONS: &str = "/admin/shipping-options";
pub const ADMIN_SHIPPING_OPTIONS_ID: &str = "/admin/shipping-options/{id}";

// Users
pub const ADMIN_USERS: &str = "/admin/users";
pub const ADMIN_USERS_ME: &str = "/admin/users/me";
pub const ADMIN_USERS_ID: &str = "/admin/users/{id}";
pub const ADMIN_USERS_PASSWORD_TOKEN: &str = "/admin/users/password-token";
pub const ADMIN_USERS_RESET_PASSWORD: &str = "/admin/users/reset-password";

// Price Lists
pub const ADMIN_PRICE_LISTS: &str = "/admin/price-lists";
pub const ADMIN_PRICE_LISTS_ID: &str = "/admin/price-lists/{id}";
pub const ADMIN_PRICE_LISTS_ID_PRICES_BATCH: &str = "/admin/price-lists/{id}/prices/batch";
pub const ADMIN_PRICE_LISTS_ID_PRODUCTS: &str = "/admin/price-lists/{id}/products";

// Inventory Items
pub const ADMIN_INVENTORY_ITEMS: &str = "/admin/inventory-items";
pub const ADMIN_INVENTORY_ITEMS_ID: &str = "/admin/inventory-items/{id}";
pub const ADMIN_INVENTORY_ITEMS_ID_LOCATION_LEVELS: &str =
    "/admin/inventory-items/{id}/location-levels";
pub const ADMIN_INVENTORY_ITEMS_ID_LOCATION_LEVELS_ID: &str =
    "/admin/inventory-items/{id}/location-levels/{location_id}";

// Tax Rates
pub const ADMIN_TAX_RATES: &str = "/admin/tax-rates";
pub const ADMIN_TAX_RATES_ID: &str = "/admin/tax-rates/{id}";

// Uploads
pub const ADMIN_UPLOADS: &str = "/admin/uploads";

// Returns
pub const ADMIN_RETURNS: &str = "/admin/returns";
pub const ADMIN_RETURNS_ID_RECEIVE: &str = "/admin/returns/{id}/receive";

// Return Reasons
pub const ADMIN_RETURN_REASONS: &str = "/admin/return-reasons";
pub const ADMIN_RETURN_REASONS_ID: &str = "/admin/return-reasons/{id}";

// Swaps
pub const ADMIN_SWAPS: &str = "/admin/swaps";
pub const ADMIN_SWAPS_ID: &str = "/admin/swaps/{id}";

// Draft Orders
pub const ADMIN_DRAFT_ORDERS: &str = "/admin/draft-orders";
pub const ADMIN_DRAFT_ORDERS_ID: &str = "/admin/draft-orders/{id}";
pub const ADMIN_DRAFT_ORDERS_ID_LINE_ITEMS: &str = "/admin/draft-orders/{id}/line-items";
pub const ADMIN_DRAFT_ORDERS_ID_LINE_ITEMS_ID: &str =
    "/admin/draft-orders/{id}/line-items/{line_id}";
pub const ADMIN_DRAFT_ORDERS_ID_PAY: &str = "/admin/draft-orders/{id}/pay";

// Gift Cards
pub const ADMIN_GIFT_CARDS: &str = "/admin/gift-cards";
pub const ADMIN_GIFT_CARDS_ID: &str = "/admin/gift-cards/{id}";

// Currencies
pub const ADMIN_CURRENCIES: &str = "/admin/currencies";
pub const ADMIN_CURRENCIES_CODE: &str = "/admin/currencies/{code}";

// Batch Jobs
pub const ADMIN_BATCH_JOBS: &str = "/admin/batch-jobs";
pub const ADMIN_BATCH_JOBS_ID: &str = "/admin/batch-jobs/{id}";
pub const ADMIN_BATCH_JOBS_ID_CONFIRM: &str = "/admin/batch-jobs/{id}/confirm";
pub const ADMIN_BATCH_JOBS_ID_CANCEL: &str = "/admin/batch-jobs/{id}/cancel";

// ─── STORE ────────────────────────────────────────────────────────────────────

// Auth
pub const STORE_AUTH: &str = "/store/auth";
pub const STORE_AUTH_PROVIDER: &str = "/store/auth/{provider}";

// Products
pub const STORE_PRODUCTS: &str = "/store/products";
pub const STORE_PRODUCTS_ID: &str = "/store/products/{id}";

// Collections
pub const STORE_COLLECTIONS: &str = "/store/collections";
pub const STORE_COLLECTIONS_ID: &str = "/store/collections/{id}";

// Carts
pub const STORE_CARTS: &str = "/store/carts";
pub const STORE_CARTS_ID: &str = "/store/carts/{id}";
pub const STORE_CARTS_ID_LINE_ITEMS: &str = "/store/carts/{id}/line-items";
pub const STORE_CARTS_ID_LINE_ITEMS_ID: &str = "/store/carts/{id}/line-items/{line_id}";
pub const STORE_CARTS_ID_PAYMENT_SESSIONS: &str = "/store/carts/{id}/payment-sessions";
pub const STORE_CARTS_ID_PAYMENT_SESSION: &str = "/store/carts/{id}/payment-session";
pub const STORE_CARTS_ID_PAYMENT_SESSIONS_ID: &str =
    "/store/carts/{id}/payment-sessions/{provider_id}";
pub const STORE_CARTS_ID_SHIPPING_METHODS: &str = "/store/carts/{id}/shipping-methods";
pub const STORE_CARTS_ID_COMPLETE: &str = "/store/carts/{id}/complete";
pub const STORE_CARTS_ID_TAXES: &str = "/store/carts/{id}/taxes";
pub const STORE_CARTS_ID_CUSTOMER: &str = "/store/carts/{id}/customer";

// Customers
pub const STORE_CUSTOMERS: &str = "/store/customers";
pub const STORE_CUSTOMERS_ME: &str = "/store/customers/me";
pub const STORE_CUSTOMERS_PASSWORD_TOKEN: &str = "/store/customers/password-token";
pub const STORE_CUSTOMERS_PASSWORD_RESET: &str = "/store/customers/password-reset";
pub const STORE_CUSTOMERS_ME_ORDERS: &str = "/store/customers/me/orders";
pub const STORE_CUSTOMERS_ME_ADDRESSES: &str = "/store/customers/me/addresses";
pub const STORE_CUSTOMERS_ME_ADDRESSES_ID: &str = "/store/customers/me/addresses/{address_id}";
pub const STORE_CUSTOMERS_ME_PAYMENT_METHODS: &str = "/store/customers/me/payment-methods";

// Orders
pub const STORE_ORDERS: &str = "/store/orders";
pub const STORE_ORDERS_ID: &str = "/store/orders/{id}";
pub const STORE_ORDERS_BATCH: &str = "/store/orders/batch";

// Regions
pub const STORE_REGIONS: &str = "/store/regions";
pub const STORE_REGIONS_ID: &str = "/store/regions/{id}";

// Shipping Options
pub const STORE_SHIPPING_OPTIONS: &str = "/store/shipping-options";
pub const STORE_SHIPPING_OPTIONS_CART_ID: &str = "/store/shipping-options/{cart_id}";

// Product Categories
pub const STORE_PRODUCT_CATEGORIES: &str = "/store/product-categories";
pub const STORE_PRODUCT_CATEGORIES_ID: &str = "/store/product-categories/{id}";

// Swaps
pub const STORE_SWAPS: &str = "/store/swaps";
pub const STORE_SWAPS_CART_ID: &str = "/store/swaps/{cart_id}";

// Returns
pub const STORE_RETURNS: &str = "/store/returns";

// Return Reasons
pub const STORE_RETURN_REASONS: &str = "/store/return-reasons";
pub const STORE_RETURN_REASONS_ID: &str = "/store/return-reasons/{id}";

// Gift Cards
pub const STORE_GIFT_CARDS_ID: &str = "/store/gift-cards/{id_or_code}";

// Currencies
pub const STORE_CURRENCIES: &str = "/store/currencies";
pub const STORE_CURRENCIES_CODE: &str = "/store/currencies/{code}";

// Product Tags
pub const STORE_PRODUCT_TAGS: &str = "/store/product-tags";
pub const STORE_PRODUCT_TAGS_ID: &str = "/store/product-tags/{id}";

// Product Types
pub const STORE_PRODUCT_TYPES: &str = "/store/product-types";
pub const STORE_PRODUCT_TYPES_ID: &str = "/store/product-types/{id}";

// Payment Providers
pub const STORE_PAYMENT_PROVIDERS: &str = "/store/payment-providers";

// Sales Channels
pub const ADMIN_SALES_CHANNELS: &str = "/admin/sales-channels";
pub const ADMIN_SALES_CHANNELS_ID: &str = "/admin/sales-channels/{id}";
pub const ADMIN_SALES_CHANNELS_ID_PRODUCTS: &str = "/admin/sales-channels/{id}/products";

// Stock Locations
pub const ADMIN_STOCK_LOCATIONS: &str = "/admin/stock-locations";
pub const ADMIN_STOCK_LOCATIONS_ID: &str = "/admin/stock-locations/{id}";
pub const ADMIN_STOCK_LOCATIONS_ID_FULFILLMENT_PROVIDERS: &str =
    "/admin/stock-locations/{id}/fulfillment-providers";
pub const ADMIN_STOCK_LOCATIONS_ID_FULFILLMENT_SETS: &str =
    "/admin/stock-locations/{id}/fulfillment-sets";
pub const ADMIN_STOCK_LOCATIONS_ID_SALES_CHANNELS: &str =
    "/admin/stock-locations/{id}/sales-channels";

// Stores
pub const ADMIN_STORES: &str = "/admin/stores";
pub const ADMIN_STORES_ID: &str = "/admin/stores/{id}";

// Customer Groups
pub const ADMIN_CUSTOMER_GROUPS: &str = "/admin/customer-groups";
pub const ADMIN_CUSTOMER_GROUPS_ID: &str = "/admin/customer-groups/{id}";
pub const ADMIN_CUSTOMER_GROUPS_ID_CUSTOMERS: &str = "/admin/customer-groups/{id}/customers";

// API Keys
pub const ADMIN_API_KEYS: &str = "/admin/api-keys";
pub const ADMIN_API_KEYS_ID: &str = "/admin/api-keys/{id}";
pub const ADMIN_API_KEYS_ID_REVOKE: &str = "/admin/api-keys/{id}/revoke";

// Invites
pub const ADMIN_INVITES: &str = "/admin/invites";
pub const ADMIN_INVITES_ID: &str = "/admin/invites/{id}";
pub const ADMIN_INVITES_ID_ACCEPT: &str = "/admin/invites/{id}/accept";

// Tax Providers
pub const ADMIN_TAX_PROVIDERS: &str = "/admin/tax-providers";

// Tax Regions
pub const ADMIN_TAX_REGIONS: &str = "/admin/tax-regions";
pub const ADMIN_TAX_REGIONS_ID: &str = "/admin/tax-regions/{id}";

// Shipping Profiles
pub const ADMIN_SHIPPING_PROFILES: &str = "/admin/shipping-profiles";
pub const ADMIN_SHIPPING_PROFILES_ID: &str = "/admin/shipping-profiles/{id}";

// Payment Providers (admin)
pub const ADMIN_PAYMENT_PROVIDERS: &str = "/admin/payment-providers";

// Order Edits
pub const ADMIN_ORDER_EDITS: &str = "/admin/order-edits";
pub const ADMIN_ORDER_EDITS_ID: &str = "/admin/order-edits/{id}";
pub const ADMIN_ORDER_EDITS_ID_REQUEST: &str = "/admin/order-edits/{id}/request";
pub const ADMIN_ORDER_EDITS_ID_CONFIRM: &str = "/admin/order-edits/{id}/confirm";
pub const ADMIN_ORDER_EDITS_ID_DECLINE: &str = "/admin/order-edits/{id}/decline";
pub const ADMIN_ORDER_EDITS_ID_CANCEL: &str = "/admin/order-edits/{id}/cancel";
pub const ADMIN_ORDER_EDITS_ID_ITEMS: &str = "/admin/order-edits/{id}/items";
pub const ADMIN_ORDER_EDITS_ID_ITEMS_ID: &str = "/admin/order-edits/{id}/items/{item_id}";
pub const ADMIN_ORDER_EDITS_ID_CHANGES_ID: &str = "/admin/order-edits/{id}/changes/{change_id}";

// Promotions
pub const ADMIN_PROMOTIONS: &str = "/admin/promotions";
pub const ADMIN_PROMOTIONS_ID: &str = "/admin/promotions/{id}";
pub const ADMIN_PROMOTIONS_ID_RULES: &str = "/admin/promotions/{id}/rules";
pub const ADMIN_PROMOTIONS_ID_BUY_RULES: &str = "/admin/promotions/{id}/buy-rules";
pub const ADMIN_PROMOTIONS_ID_TARGET_RULES: &str = "/admin/promotions/{id}/target-rules";

// Notifications
pub const ADMIN_NOTIFICATIONS: &str = "/admin/notifications";
pub const ADMIN_NOTIFICATIONS_ID: &str = "/admin/notifications/{id}";

// ─── STORE (additional) ───────────────────────────────────────────────────────

// Locales
pub const STORE_LOCALES: &str = "/store/locales";

// Payment Collections
pub const STORE_PAYMENT_COLLECTIONS: &str = "/store/payment-collections";
pub const STORE_PAYMENT_COLLECTIONS_ID_SESSIONS: &str =
    "/store/payment-collections/{id}/payment-sessions";

// Store Credit Accounts
pub const STORE_STORE_CREDIT_ACCOUNTS: &str = "/store/store-credit-accounts";
pub const STORE_STORE_CREDIT_ACCOUNTS_ID: &str = "/store/store-credit-accounts/{id}";

// Cart sub-routes (additional)
pub const STORE_CARTS_ID_DISCOUNTS_CODE: &str = "/store/carts/{id}/discounts/{code}";
pub const STORE_CARTS_ID_GIFT_CARDS: &str = "/store/carts/{id}/gift-cards";
pub const STORE_CARTS_ID_PROMOTIONS: &str = "/store/carts/{id}/promotions";
pub const STORE_CARTS_ID_STORE_CREDITS: &str = "/store/carts/{id}/store-credits";
pub const STORE_CARTS_ID_PAYMENT_SESSIONS_ID_REFRESH: &str =
    "/store/carts/{id}/payment-sessions/{provider_id}/refresh";

// Order Transfers
pub const STORE_ORDERS_ID_TRANSFER_ACCEPT: &str = "/store/orders/{id}/transfer/accept";
pub const STORE_ORDERS_ID_TRANSFER_CANCEL: &str = "/store/orders/{id}/transfer/cancel";
pub const STORE_ORDERS_ID_TRANSFER_DECLINE: &str = "/store/orders/{id}/transfer/decline";
pub const STORE_ORDERS_ID_TRANSFER_REQUEST: &str = "/store/orders/{id}/transfer/request";

// Shipping Option calculate
pub const STORE_SHIPPING_OPTIONS_ID_CALCULATE: &str = "/store/shipping-options/{id}/calculate";

// ─── Wizard / Importer ────────────────────────────────────────────────────────
pub const WIZARD_IMPORT: &str = "/wizard/import";
pub const WIZARD_IMPORT_STATUS_ID: &str = "/wizard/import/{job_id}/status";

// ─── Phase 1b: Price Preferences ─────────────────────────────────────────────
pub const ADMIN_PRICE_PREFERENCES: &str = "/admin/price-preferences";
pub const ADMIN_PRICE_PREFERENCES_ID: &str = "/admin/price-preferences/{id}";

// ─── Phase 1c: Campaigns ─────────────────────────────────────────────────────
pub const ADMIN_CAMPAIGNS: &str = "/admin/campaigns";
pub const ADMIN_CAMPAIGNS_ID: &str = "/admin/campaigns/{id}";
pub const ADMIN_CAMPAIGNS_ID_PROMOTIONS: &str = "/admin/campaigns/{id}/promotions";

// Promotions extra rules
pub const ADMIN_PROMOTIONS_ID_BUY_RULES_BATCH: &str = "/admin/promotions/{id}/buy-rules/batch";
pub const ADMIN_PROMOTIONS_ID_TARGET_RULES_BATCH: &str = "/admin/promotions/{id}/target-rules/batch";
pub const ADMIN_PROMOTIONS_RULE_ATTRIBUTE_OPTIONS: &str = "/admin/promotions/rule-attribute-options/{rule_type}";
pub const ADMIN_PROMOTIONS_RULE_VALUE_OPTIONS: &str = "/admin/promotions/rule-value-options/{rule_type}/{rule_attribute_id}";

// ─── Phase 1d: Fulfillment Sets ───────────────────────────────────────────────
pub const ADMIN_FULFILLMENT_SETS_ID: &str = "/admin/fulfillment-sets/{id}";
pub const ADMIN_FULFILLMENT_SETS_ID_SERVICE_ZONES: &str = "/admin/fulfillment-sets/{id}/service-zones";
pub const ADMIN_FULFILLMENT_SETS_ID_SERVICE_ZONES_ID: &str = "/admin/fulfillment-sets/{id}/service-zones/{zone_id}";

// ─── Phase 2b: Claims ─────────────────────────────────────────────────────────
pub const ADMIN_CLAIMS: &str = "/admin/claims";
pub const ADMIN_CLAIMS_ID: &str = "/admin/claims/{id}";
pub const ADMIN_CLAIMS_ID_CANCEL: &str = "/admin/claims/{id}/cancel";
pub const ADMIN_CLAIMS_ID_CONFIRM: &str = "/admin/claims/{id}/confirm";
pub const ADMIN_CLAIMS_ID_REQUEST: &str = "/admin/claims/{id}/request";
pub const ADMIN_CLAIMS_ID_OUTBOUND_ITEMS: &str = "/admin/claims/{id}/outbound/items";
pub const ADMIN_CLAIMS_ID_INBOUND_ITEMS: &str = "/admin/claims/{id}/inbound/items";
pub const ADMIN_CLAIMS_ID_SHIPPING_METHOD: &str = "/admin/claims/{id}/shipping-method";

// ─── Phase 2c: Exchanges ──────────────────────────────────────────────────────
pub const ADMIN_EXCHANGES: &str = "/admin/exchanges";
pub const ADMIN_EXCHANGES_ID: &str = "/admin/exchanges/{id}";
pub const ADMIN_EXCHANGES_ID_CANCEL: &str = "/admin/exchanges/{id}/cancel";
pub const ADMIN_EXCHANGES_ID_CONFIRM: &str = "/admin/exchanges/{id}/confirm";
pub const ADMIN_EXCHANGES_ID_REQUEST: &str = "/admin/exchanges/{id}/request";
pub const ADMIN_EXCHANGES_ID_OUTBOUND_ITEMS: &str = "/admin/exchanges/{id}/outbound/items";
pub const ADMIN_EXCHANGES_ID_INBOUND_ITEMS: &str = "/admin/exchanges/{id}/inbound/items";
pub const ADMIN_EXCHANGES_ID_SHIPPING_METHOD: &str = "/admin/exchanges/{id}/shipping-method";

// ─── Phase 2d: Returns (extended) ─────────────────────────────────────────────
pub const ADMIN_RETURNS_ID: &str = "/admin/returns/{id}";
pub const ADMIN_RETURNS_ID_CANCEL: &str = "/admin/returns/{id}/cancel";
pub const ADMIN_RETURNS_ID_RECEIVE_ITEMS: &str = "/admin/returns/{id}/receive-items";
pub const ADMIN_RETURNS_ID_RECEIVE_CONFIRM: &str = "/admin/returns/{id}/receive/confirm";
pub const ADMIN_RETURNS_ID_REQUEST: &str = "/admin/returns/{id}/request";
pub const ADMIN_RETURNS_ID_SHIPPING_METHOD: &str = "/admin/returns/{id}/shipping-method";
pub const ADMIN_RETURNS_ID_DISMISS_ITEMS: &str = "/admin/returns/{id}/dismiss-items";

// ─── Phase 3a: Workflow Executions ────────────────────────────────────────────
pub const ADMIN_WORKFLOWS_EXECUTIONS: &str = "/admin/workflows-executions";
pub const ADMIN_WORKFLOWS_EXECUTIONS_ID: &str = "/admin/workflows-executions/{id}";
pub const ADMIN_WORKFLOWS_EXECUTIONS_ID_RUN: &str = "/admin/workflows-executions/{workflow_id}/run";

// ─── Phase 3b: Notifications (extended) ───────────────────────────────────────
pub const ADMIN_NOTIFICATIONS_ID_RESEND: &str = "/admin/notifications/{id}/resend";

// ─── Phase 4a: Fulfillment Providers ─────────────────────────────────────────
pub const ADMIN_FULFILLMENT_PROVIDERS: &str = "/admin/fulfillment-providers";
pub const ADMIN_FULFILLMENT_PROVIDERS_ID: &str = "/admin/fulfillment-providers/{id}";
pub const ADMIN_FULFILLMENT_PROVIDERS_ID_OPTIONS: &str = "/admin/fulfillment-providers/{id}/options";

// ─── Phase 4b: Payment Collections (admin) ───────────────────────────────────
pub const ADMIN_PAYMENT_COLLECTIONS: &str = "/admin/payment-collections";
pub const ADMIN_PAYMENT_COLLECTIONS_ID: &str = "/admin/payment-collections/{id}";
pub const ADMIN_PAYMENT_COLLECTIONS_ID_MARK_AS_PAID: &str = "/admin/payment-collections/{id}/mark-as-paid";

// ─── Phase 4c: Refund Reasons ─────────────────────────────────────────────────
pub const ADMIN_REFUND_REASONS: &str = "/admin/refund-reasons";
pub const ADMIN_REFUND_REASONS_ID: &str = "/admin/refund-reasons/{id}";

// ─── Phase 4d: Reservations ───────────────────────────────────────────────────
pub const ADMIN_RESERVATIONS: &str = "/admin/reservations";
pub const ADMIN_RESERVATIONS_ID: &str = "/admin/reservations/{id}";

// ─── Additional missing routes ────────────────────────────────────────────────

// Product Tags (admin)
pub const ADMIN_PRODUCT_TAGS: &str = "/admin/product-tags";
pub const ADMIN_PRODUCT_TAGS_ID: &str = "/admin/product-tags/{id}";

// Product Types (admin)
pub const ADMIN_PRODUCT_TYPES: &str = "/admin/product-types";
pub const ADMIN_PRODUCT_TYPES_ID: &str = "/admin/product-types/{id}";

// Product Variants standalone
pub const ADMIN_PRODUCT_VARIANTS: &str = "/admin/product-variants";

// Payments (admin)
pub const ADMIN_PAYMENTS: &str = "/admin/payments";
pub const ADMIN_PAYMENTS_ID: &str = "/admin/payments/{id}";
pub const ADMIN_PAYMENTS_ID_CAPTURE: &str = "/admin/payments/{id}/capture";
pub const ADMIN_PAYMENTS_ID_REFUND: &str = "/admin/payments/{id}/refund";

// Plugins
pub const ADMIN_PLUGINS: &str = "/admin/plugins";

// Shipping Option Types
pub const ADMIN_SHIPPING_OPTION_TYPES: &str = "/admin/shipping-option-types";
pub const ADMIN_SHIPPING_OPTION_TYPES_ID: &str = "/admin/shipping-option-types/{id}";

// Feature Flags
pub const ADMIN_FEATURE_FLAGS: &str = "/admin/feature-flags";

// Order Changes
pub const ADMIN_ORDER_CHANGES_ID: &str = "/admin/order-changes/{id}";
