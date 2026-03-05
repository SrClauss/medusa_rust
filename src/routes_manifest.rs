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

// Products
pub const ADMIN_PRODUCTS: &str = "/admin/products";
pub const ADMIN_PRODUCTS_ID: &str = "/admin/products/:id";
pub const ADMIN_PRODUCTS_ID_VARIANTS: &str = "/admin/products/:id/variants";
pub const ADMIN_PRODUCTS_ID_VARIANTS_ID: &str = "/admin/products/:id/variants/:variant_id";
pub const ADMIN_PRODUCTS_ID_OPTIONS: &str = "/admin/products/:id/options";
pub const ADMIN_PRODUCTS_ID_OPTIONS_ID: &str = "/admin/products/:id/options/:option_id";

// Collections
pub const ADMIN_COLLECTIONS: &str = "/admin/collections";
pub const ADMIN_COLLECTIONS_ID: &str = "/admin/collections/:id";
pub const ADMIN_COLLECTIONS_ID_PRODUCTS_BATCH: &str = "/admin/collections/:id/products/batch";

// Orders
pub const ADMIN_ORDERS: &str = "/admin/orders";
pub const ADMIN_ORDERS_ID: &str = "/admin/orders/:id";
pub const ADMIN_ORDERS_ID_CANCEL: &str = "/admin/orders/:id/cancel";
pub const ADMIN_ORDERS_ID_COMPLETE: &str = "/admin/orders/:id/complete";
pub const ADMIN_ORDERS_ID_ARCHIVE: &str = "/admin/orders/:id/archive";
pub const ADMIN_ORDERS_ID_FULFILLMENTS: &str = "/admin/orders/:id/fulfillments";
pub const ADMIN_ORDERS_ID_FULFILLMENTS_ID_CANCEL: &str =
    "/admin/orders/:id/fulfillments/:fulfillment_id/cancel";
pub const ADMIN_ORDERS_ID_SHIPMENT: &str = "/admin/orders/:id/shipment";
pub const ADMIN_ORDERS_ID_REFUNDS: &str = "/admin/orders/:id/refunds";
pub const ADMIN_ORDERS_ID_RETURNS: &str = "/admin/orders/:id/returns";
pub const ADMIN_ORDERS_ID_SWAPS: &str = "/admin/orders/:id/swaps";
pub const ADMIN_ORDERS_ID_CLAIMS: &str = "/admin/orders/:id/claims";

// Customers
pub const ADMIN_CUSTOMERS: &str = "/admin/customers";
pub const ADMIN_CUSTOMERS_ID: &str = "/admin/customers/:id";

// Product Categories
pub const ADMIN_PRODUCT_CATEGORIES: &str = "/admin/product-categories";
pub const ADMIN_PRODUCT_CATEGORIES_ID: &str = "/admin/product-categories/:id";
pub const ADMIN_PRODUCT_CATEGORIES_ID_PRODUCTS_BATCH: &str =
    "/admin/product-categories/:id/products/batch";

// Regions
pub const ADMIN_REGIONS: &str = "/admin/regions";
pub const ADMIN_REGIONS_ID: &str = "/admin/regions/:id";
pub const ADMIN_REGIONS_ID_COUNTRIES: &str = "/admin/regions/:id/countries";
pub const ADMIN_REGIONS_ID_COUNTRIES_CODE: &str = "/admin/regions/:id/countries/:country_code";
pub const ADMIN_REGIONS_ID_FULFILLMENT_PROVIDERS: &str =
    "/admin/regions/:id/fulfillment-providers";
pub const ADMIN_REGIONS_ID_FULFILLMENT_PROVIDERS_ID: &str =
    "/admin/regions/:id/fulfillment-providers/:provider_id";
pub const ADMIN_REGIONS_ID_PAYMENT_PROVIDERS: &str = "/admin/regions/:id/payment-providers";
pub const ADMIN_REGIONS_ID_PAYMENT_PROVIDERS_ID: &str =
    "/admin/regions/:id/payment-providers/:provider_id";

// Discounts
pub const ADMIN_DISCOUNTS: &str = "/admin/discounts";
pub const ADMIN_DISCOUNTS_ID: &str = "/admin/discounts/:id";
pub const ADMIN_DISCOUNTS_CODE: &str = "/admin/discounts/code/:code";
pub const ADMIN_DISCOUNTS_ID_REGIONS_ID: &str = "/admin/discounts/:id/regions/:region_id";
pub const ADMIN_DISCOUNTS_ID_CONDITIONS: &str = "/admin/discounts/:id/conditions";
pub const ADMIN_DISCOUNTS_ID_CONDITIONS_ID: &str =
    "/admin/discounts/:id/conditions/:condition_id";

// Shipping Options
pub const ADMIN_SHIPPING_OPTIONS: &str = "/admin/shipping-options";
pub const ADMIN_SHIPPING_OPTIONS_ID: &str = "/admin/shipping-options/:id";

// Users
pub const ADMIN_USERS: &str = "/admin/users";
pub const ADMIN_USERS_ID: &str = "/admin/users/:id";
pub const ADMIN_USERS_PASSWORD_TOKEN: &str = "/admin/users/password-token";
pub const ADMIN_USERS_RESET_PASSWORD: &str = "/admin/users/reset-password";

// Price Lists
pub const ADMIN_PRICE_LISTS: &str = "/admin/price-lists";
pub const ADMIN_PRICE_LISTS_ID: &str = "/admin/price-lists/:id";
pub const ADMIN_PRICE_LISTS_ID_PRICES_BATCH: &str = "/admin/price-lists/:id/prices/batch";
pub const ADMIN_PRICE_LISTS_ID_PRODUCTS: &str = "/admin/price-lists/:id/products";

// Inventory Items
pub const ADMIN_INVENTORY_ITEMS: &str = "/admin/inventory-items";
pub const ADMIN_INVENTORY_ITEMS_ID: &str = "/admin/inventory-items/:id";
pub const ADMIN_INVENTORY_ITEMS_ID_LOCATION_LEVELS: &str =
    "/admin/inventory-items/:id/location-levels";
pub const ADMIN_INVENTORY_ITEMS_ID_LOCATION_LEVELS_ID: &str =
    "/admin/inventory-items/:id/location-levels/:location_id";

// Tax Rates
pub const ADMIN_TAX_RATES: &str = "/admin/tax-rates";
pub const ADMIN_TAX_RATES_ID: &str = "/admin/tax-rates/:id";

// Uploads
pub const ADMIN_UPLOADS: &str = "/admin/uploads";

// Returns
pub const ADMIN_RETURNS: &str = "/admin/returns";
pub const ADMIN_RETURNS_ID_RECEIVE: &str = "/admin/returns/:id/receive";

// Swaps
pub const ADMIN_SWAPS: &str = "/admin/swaps";
pub const ADMIN_SWAPS_ID: &str = "/admin/swaps/:id";

// Draft Orders
pub const ADMIN_DRAFT_ORDERS: &str = "/admin/draft-orders";
pub const ADMIN_DRAFT_ORDERS_ID: &str = "/admin/draft-orders/:id";
pub const ADMIN_DRAFT_ORDERS_ID_LINE_ITEMS: &str = "/admin/draft-orders/:id/line-items";
pub const ADMIN_DRAFT_ORDERS_ID_LINE_ITEMS_ID: &str =
    "/admin/draft-orders/:id/line-items/:line_id";
pub const ADMIN_DRAFT_ORDERS_ID_PAY: &str = "/admin/draft-orders/:id/pay";

// Gift Cards
pub const ADMIN_GIFT_CARDS: &str = "/admin/gift-cards";
pub const ADMIN_GIFT_CARDS_ID: &str = "/admin/gift-cards/:id";

// Batch Jobs
pub const ADMIN_BATCH_JOBS: &str = "/admin/batch-jobs";
pub const ADMIN_BATCH_JOBS_ID: &str = "/admin/batch-jobs/:id";
pub const ADMIN_BATCH_JOBS_ID_CONFIRM: &str = "/admin/batch-jobs/:id/confirm";
pub const ADMIN_BATCH_JOBS_ID_CANCEL: &str = "/admin/batch-jobs/:id/cancel";

// ─── STORE ────────────────────────────────────────────────────────────────────

// Auth
pub const STORE_AUTH: &str = "/store/auth";
pub const STORE_AUTH_PROVIDER: &str = "/store/auth/:provider";

// Products
pub const STORE_PRODUCTS: &str = "/store/products";
pub const STORE_PRODUCTS_ID: &str = "/store/products/:id";

// Collections
pub const STORE_COLLECTIONS: &str = "/store/collections";
pub const STORE_COLLECTIONS_ID: &str = "/store/collections/:id";

// Carts
pub const STORE_CARTS: &str = "/store/carts";
pub const STORE_CARTS_ID: &str = "/store/carts/:id";
pub const STORE_CARTS_ID_LINE_ITEMS: &str = "/store/carts/:id/line-items";
pub const STORE_CARTS_ID_LINE_ITEMS_ID: &str = "/store/carts/:id/line-items/:line_id";
pub const STORE_CARTS_ID_PAYMENT_SESSIONS: &str = "/store/carts/:id/payment-sessions";
pub const STORE_CARTS_ID_PAYMENT_SESSION: &str = "/store/carts/:id/payment-session";
pub const STORE_CARTS_ID_PAYMENT_SESSIONS_ID: &str =
    "/store/carts/:id/payment-sessions/:provider_id";
pub const STORE_CARTS_ID_SHIPPING_METHODS: &str = "/store/carts/:id/shipping-methods";
pub const STORE_CARTS_ID_COMPLETE: &str = "/store/carts/:id/complete";
pub const STORE_CARTS_ID_TAXES: &str = "/store/carts/:id/taxes";

// Customers
pub const STORE_CUSTOMERS: &str = "/store/customers";
pub const STORE_CUSTOMERS_ME: &str = "/store/customers/me";
pub const STORE_CUSTOMERS_PASSWORD_TOKEN: &str = "/store/customers/password-token";
pub const STORE_CUSTOMERS_PASSWORD_RESET: &str = "/store/customers/password-reset";
pub const STORE_CUSTOMERS_ME_ORDERS: &str = "/store/customers/me/orders";
pub const STORE_CUSTOMERS_ME_ADDRESSES: &str = "/store/customers/me/addresses";
pub const STORE_CUSTOMERS_ME_ADDRESSES_ID: &str = "/store/customers/me/addresses/:address_id";
pub const STORE_CUSTOMERS_ME_PAYMENT_METHODS: &str = "/store/customers/me/payment-methods";

// Orders
pub const STORE_ORDERS: &str = "/store/orders";
pub const STORE_ORDERS_ID: &str = "/store/orders/:id";
pub const STORE_ORDERS_BATCH: &str = "/store/orders/batch";

// Regions
pub const STORE_REGIONS: &str = "/store/regions";
pub const STORE_REGIONS_ID: &str = "/store/regions/:id";

// Shipping Options
pub const STORE_SHIPPING_OPTIONS: &str = "/store/shipping-options";
pub const STORE_SHIPPING_OPTIONS_CART_ID: &str = "/store/shipping-options/:cart_id";

// Product Categories
pub const STORE_PRODUCT_CATEGORIES: &str = "/store/product-categories";
pub const STORE_PRODUCT_CATEGORIES_ID: &str = "/store/product-categories/:id";

// Swaps
pub const STORE_SWAPS: &str = "/store/swaps";
pub const STORE_SWAPS_CART_ID: &str = "/store/swaps/:cart_id";

// Returns
pub const STORE_RETURNS: &str = "/store/returns";

// ─── Wizard / Importer ────────────────────────────────────────────────────────
pub const WIZARD_IMPORT: &str = "/wizard/import";
pub const WIZARD_IMPORT_STATUS_ID: &str = "/wizard/import/:job_id/status";
