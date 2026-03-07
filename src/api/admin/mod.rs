//! Admin API routes.

pub mod auth;
pub mod batch_jobs;
pub mod categories;
pub mod collections;
pub mod currencies;
pub mod customers;
pub mod discounts;
pub mod draft_orders;
pub mod gift_cards;
pub mod inventory;
pub mod orders;
pub mod price_lists;
pub mod products;
pub mod regions;
pub mod returns;
pub mod shipping_options;
pub mod swaps;
pub mod tax_rates;
pub mod uploads;
pub mod users;

use axum::{middleware, routing::{delete, get, post, put}, Router};

use crate::{auth::jwt::admin_auth_middleware, routes_manifest::*, state::AppState};

/// Assembles the full `/admin` router, with auth middleware applied to all
/// routes except `POST /admin/auth` (login).
pub fn admin_router(state: AppState) -> Router<AppState> {
    let public_routes = Router::new()
        .route(ADMIN_AUTH, post(auth::login));

    let protected_routes = Router::new()
        // Auth
        .route(ADMIN_AUTH, get(auth::get_session).delete(auth::logout))
        // Products
        .route(ADMIN_PRODUCTS, get(products::list).post(products::create))
        .route(
            ADMIN_PRODUCTS_ID,
            get(products::get)
                .put(products::update)
                .delete(products::delete_one),
        )
        .route(
            ADMIN_PRODUCTS_ID_VARIANTS,
            get(products::list_variants).post(products::create_variant),
        )
        .route(
            ADMIN_PRODUCTS_ID_VARIANTS_ID,
            put(products::update_variant).delete(products::delete_variant),
        )
        .route(
            ADMIN_PRODUCTS_ID_OPTIONS,
            get(products::list_options).post(products::create_option),
        )
        .route(
            ADMIN_PRODUCTS_ID_OPTIONS_ID,
            put(products::update_option).delete(products::delete_option),
        )
        // Collections
        .route(ADMIN_COLLECTIONS, get(collections::list).post(collections::create))
        .route(
            ADMIN_COLLECTIONS_ID,
            get(collections::get).put(collections::update).delete(collections::delete_one),
        )
        .route(
            ADMIN_COLLECTIONS_ID_PRODUCTS_BATCH,
            post(collections::add_products).delete(collections::remove_products),
        )
        // Orders
        .route(ADMIN_ORDERS, get(orders::list).post(orders::create))
        .route(ADMIN_ORDERS_ID, get(orders::get).put(orders::update))
        .route(ADMIN_ORDERS_ID_CANCEL, post(orders::cancel))
        .route(ADMIN_ORDERS_ID_COMPLETE, post(orders::complete))
        .route(ADMIN_ORDERS_ID_ARCHIVE, post(orders::archive))
        .route(ADMIN_ORDERS_ID_FULFILLMENTS, post(orders::create_fulfillment))
        .route(ADMIN_ORDERS_ID_FULFILLMENTS_ID_CANCEL, post(orders::cancel_fulfillment))
        .route(ADMIN_ORDERS_ID_SHIPMENT, post(orders::create_shipment))
        .route(ADMIN_ORDERS_ID_REFUNDS, post(orders::create_refund))
        .route(ADMIN_ORDERS_ID_RETURNS, post(orders::request_return))
        .route(ADMIN_ORDERS_ID_SWAPS, post(orders::create_swap))
        .route(ADMIN_ORDERS_ID_CLAIMS, post(orders::create_claim))
        // Customers
        .route(ADMIN_CUSTOMERS, get(customers::list).post(customers::create))
        .route(ADMIN_CUSTOMERS_ID, get(customers::get).put(customers::update))
        // Categories
        .route(ADMIN_PRODUCT_CATEGORIES, get(categories::list).post(categories::create))
        .route(
            ADMIN_PRODUCT_CATEGORIES_ID,
            get(categories::get)
                .put(categories::update)
                .delete(categories::delete_one),
        )
        .route(
            ADMIN_PRODUCT_CATEGORIES_ID_PRODUCTS_BATCH,
            post(categories::add_products).delete(categories::remove_products),
        )
        // Regions
        .route(ADMIN_REGIONS, get(regions::list).post(regions::create))
        .route(
            ADMIN_REGIONS_ID,
            get(regions::get).put(regions::update).delete(regions::delete_one),
        )
        .route(ADMIN_REGIONS_ID_COUNTRIES, post(regions::add_country))
        .route(ADMIN_REGIONS_ID_COUNTRIES_CODE, delete(regions::remove_country))
        .route(ADMIN_REGIONS_ID_FULFILLMENT_PROVIDERS, post(regions::add_fulfillment_provider))
        .route(
            ADMIN_REGIONS_ID_FULFILLMENT_PROVIDERS_ID,
            delete(regions::remove_fulfillment_provider),
        )
        .route(ADMIN_REGIONS_ID_PAYMENT_PROVIDERS, post(regions::add_payment_provider))
        .route(
            ADMIN_REGIONS_ID_PAYMENT_PROVIDERS_ID,
            delete(regions::remove_payment_provider),
        )
        // Discounts
        .route(ADMIN_DISCOUNTS, get(discounts::list).post(discounts::create))
        .route(
            ADMIN_DISCOUNTS_ID,
            get(discounts::get).put(discounts::update).delete(discounts::delete_one),
        )
        .route(ADMIN_DISCOUNTS_CODE, get(discounts::get_by_code))
        .route(
            ADMIN_DISCOUNTS_ID_REGIONS_ID,
            post(discounts::add_region).delete(discounts::remove_region),
        )
        .route(ADMIN_DISCOUNTS_ID_CONDITIONS, get(discounts::list_conditions).post(discounts::create_condition))
        .route(
            ADMIN_DISCOUNTS_ID_CONDITIONS_ID,
            get(discounts::get_condition)
                .put(discounts::update_condition)
                .delete(discounts::delete_condition),
        )
        // Shipping Options
        .route(ADMIN_SHIPPING_OPTIONS, get(shipping_options::list).post(shipping_options::create))
        .route(
            ADMIN_SHIPPING_OPTIONS_ID,
            get(shipping_options::get)
                .put(shipping_options::update)
                .delete(shipping_options::delete_one),
        )
        // Users
        .route(ADMIN_USERS, get(users::list).post(users::create))
        .route(
            ADMIN_USERS_ID,
            get(users::get).put(users::update).delete(users::delete_one),
        )
        .route(ADMIN_USERS_PASSWORD_TOKEN, post(users::request_password_reset))
        .route(ADMIN_USERS_RESET_PASSWORD, post(users::reset_password))
        // Price Lists
        .route(ADMIN_PRICE_LISTS, get(price_lists::list).post(price_lists::create))
        .route(
            ADMIN_PRICE_LISTS_ID,
            get(price_lists::get).put(price_lists::update).delete(price_lists::delete_one),
        )
        .route(
            ADMIN_PRICE_LISTS_ID_PRICES_BATCH,
            post(price_lists::add_prices).delete(price_lists::delete_prices),
        )
        .route(ADMIN_PRICE_LISTS_ID_PRODUCTS, get(price_lists::list_products))
        // Inventory
        .route(ADMIN_INVENTORY_ITEMS, get(inventory::list).post(inventory::create))
        .route(
            ADMIN_INVENTORY_ITEMS_ID,
            get(inventory::get).put(inventory::update).delete(inventory::delete_one),
        )
        .route(
            ADMIN_INVENTORY_ITEMS_ID_LOCATION_LEVELS,
            get(inventory::list_levels).post(inventory::create_level),
        )
        .route(
            ADMIN_INVENTORY_ITEMS_ID_LOCATION_LEVELS_ID,
            put(inventory::update_level).delete(inventory::delete_level),
        )
        // Tax Rates
        .route(ADMIN_TAX_RATES, get(tax_rates::list).post(tax_rates::create))
        .route(
            ADMIN_TAX_RATES_ID,
            get(tax_rates::get).put(tax_rates::update).delete(tax_rates::delete_one),
        )
        // Uploads
        .route(ADMIN_UPLOADS, post(uploads::upload).delete(uploads::delete_files))
        // Returns
        .route(ADMIN_RETURNS, get(returns::list))
        .route(ADMIN_RETURNS_ID_RECEIVE, post(returns::receive))
        // Swaps
        .route(ADMIN_SWAPS, get(swaps::list))
        .route(ADMIN_SWAPS_ID, get(swaps::get))
        // Draft Orders
        .route(ADMIN_DRAFT_ORDERS, get(draft_orders::list).post(draft_orders::create))
        .route(
            ADMIN_DRAFT_ORDERS_ID,
            get(draft_orders::get)
                .put(draft_orders::update)
                .delete(draft_orders::delete_one),
        )
        .route(
            ADMIN_DRAFT_ORDERS_ID_LINE_ITEMS,
            post(draft_orders::add_line_item),
        )
        .route(
            ADMIN_DRAFT_ORDERS_ID_LINE_ITEMS_ID,
            put(draft_orders::update_line_item).delete(draft_orders::delete_line_item),
        )
        .route(ADMIN_DRAFT_ORDERS_ID_PAY, post(draft_orders::register_payment))
        // Gift Cards
        .route(ADMIN_GIFT_CARDS, get(gift_cards::list).post(gift_cards::create))
        .route(
            ADMIN_GIFT_CARDS_ID,
            get(gift_cards::get).put(gift_cards::update).delete(gift_cards::delete_one),
        )
        // Currencies
        .route(ADMIN_CURRENCIES, get(currencies::list))
        .route(ADMIN_CURRENCIES_CODE, get(currencies::get).put(currencies::update))
        // Batch Jobs
        .route(ADMIN_BATCH_JOBS, get(batch_jobs::list).post(batch_jobs::create))
        .route(ADMIN_BATCH_JOBS_ID, get(batch_jobs::get))
        .route(ADMIN_BATCH_JOBS_ID_CONFIRM, post(batch_jobs::confirm))
        .route(ADMIN_BATCH_JOBS_ID_CANCEL, post(batch_jobs::cancel))
        // Apply auth middleware to all protected routes.
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            admin_auth_middleware,
        ));

    Router::new().merge(public_routes).merge(protected_routes)
}
