//! Store API routes.

pub mod auth;
pub mod carts;
pub mod categories;
pub mod collections;
pub mod customers;
pub mod orders;
pub mod products;
pub mod regions;
pub mod returns;
pub mod shipping_options;
pub mod swaps;

use axum::{middleware, routing::{delete, get, post}, Router};
use crate::{auth::jwt::store_auth_middleware, routes_manifest::*, state::AppState};

pub fn store_router(state: AppState) -> Router<AppState> {
    // Routes accessible without authentication
    let public_routes = Router::new()
        // Auth
        .route(STORE_AUTH, get(auth::get_session).post(auth::login).delete(auth::logout))
        .route(STORE_AUTH_PROVIDER, post(auth::oauth_callback))
        // Products
        .route(STORE_PRODUCTS, get(products::list_products))
        .route(STORE_PRODUCTS_ID, get(products::get_product))
        // Collections
        .route(STORE_COLLECTIONS, get(collections::list))
        .route(STORE_COLLECTIONS_ID, get(collections::get))
        // Categories
        .route(STORE_PRODUCT_CATEGORIES, get(categories::list))
        .route(STORE_PRODUCT_CATEGORIES_ID, get(categories::get))
        // Carts (public — guests create carts without being logged in)
        .route(STORE_CARTS, post(carts::create_cart))
        .route(STORE_CARTS_ID, get(carts::get_cart).post(carts::update_cart))
        .route(STORE_CARTS_ID_LINE_ITEMS, post(carts::add_line_item))
        .route(STORE_CARTS_ID_LINE_ITEMS_ID, post(carts::update_line_item).delete(carts::delete_line_item))
        .route(STORE_CARTS_ID_PAYMENT_SESSIONS, post(carts::create_payment_sessions))
        .route(STORE_CARTS_ID_PAYMENT_SESSION, post(carts::select_payment_session))
        .route(STORE_CARTS_ID_PAYMENT_SESSIONS_ID, delete(carts::delete_payment_session))
        .route(STORE_CARTS_ID_SHIPPING_METHODS, post(carts::add_shipping_method))
        .route(STORE_CARTS_ID_COMPLETE, post(carts::complete_cart))
        .route(STORE_CARTS_ID_TAXES, post(carts::calculate_taxes))
        // Customer registration & password reset (public)
        .route(STORE_CUSTOMERS, post(customers::create_customer))
        .route(STORE_CUSTOMERS_PASSWORD_TOKEN, post(customers::request_password_reset))
        .route(STORE_CUSTOMERS_PASSWORD_RESET, post(customers::reset_password))
        // Orders (lookup without auth via email+display_id or cart_id)
        .route(STORE_ORDERS, get(orders::get_order_by_params))
        .route(STORE_ORDERS_ID, get(orders::get_order))
        .route(STORE_ORDERS_BATCH, get(orders::get_orders_batch))
        // Regions
        .route(STORE_REGIONS, get(regions::list))
        .route(STORE_REGIONS_ID, get(regions::get))
        // Shipping options for cart
        .route(STORE_SHIPPING_OPTIONS, get(shipping_options::list))
        .route(STORE_SHIPPING_OPTIONS_CART_ID, get(shipping_options::get_for_cart))
        // Swaps & Returns (public — customer provides email/return_id)
        .route(STORE_SWAPS, post(swaps::create))
        .route(STORE_SWAPS_CART_ID, get(swaps::get_by_cart))
        .route(STORE_RETURNS, post(returns::create));

    // Routes that require a customer JWT
    let protected_routes = Router::new()
        .route(STORE_CUSTOMERS_ME, get(customers::get_me).post(customers::update_me))
        .route(STORE_CUSTOMERS_ME_ORDERS, get(customers::list_orders))
        .route(STORE_CUSTOMERS_ME_ADDRESSES, get(customers::list_addresses).post(customers::add_address))
        .route(STORE_CUSTOMERS_ME_ADDRESSES_ID, get(customers::get_address).post(customers::update_address).delete(customers::delete_address))
        .route(STORE_CUSTOMERS_ME_PAYMENT_METHODS, get(customers::list_payment_methods).post(customers::add_payment_method))
        .route_layer(middleware::from_fn_with_state(state.clone(), store_auth_middleware));

    Router::new().merge(public_routes).merge(protected_routes)
}
