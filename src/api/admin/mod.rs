//! Admin API routes.

pub mod api_keys;
pub mod auth;
pub mod batch_jobs;
pub mod campaigns;
pub mod categories;
pub mod claims;
pub mod collections;
pub mod currencies;
pub mod customer_groups;
pub mod customers;
pub mod discounts;
pub mod draft_orders;
pub mod exchanges;
pub mod feature_flags;
pub mod fulfillment_providers;
pub mod fulfillment_sets;
pub mod gift_cards;
pub mod inventory;
pub mod invites;
pub mod notifications;
pub mod order_edits;
pub mod orders;
pub mod payment_collections;
pub mod payment_providers;
pub mod payments;
pub mod plugins;
pub mod price_lists;
pub mod price_preferences;
pub mod product_tags;
pub mod product_types;
pub mod products;
pub mod promotions;
pub mod refund_reasons;
pub mod regions;
pub mod reservations;
pub mod return_reasons;
pub mod returns;
pub mod sales_channels;
pub mod shipping_option_types;
pub mod shipping_options;
pub mod shipping_profiles;
pub mod stock_locations;
pub mod stores;
pub mod swaps;
pub mod tax_providers;
pub mod tax_rates;
pub mod tax_regions;
pub mod uploads;
pub mod users;
pub mod workflow_executions;

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
        .route(ADMIN_PRODUCT_VARIANTS, get(products::list_all_variants))
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
            get(products::get_variant).put(products::update_variant).delete(products::delete_variant),
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
        .route(ADMIN_USERS_ME, get(users::get_me))
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
        .route(ADMIN_RETURNS_ID, get(returns::get))
        .route(ADMIN_RETURNS_ID_RECEIVE, post(returns::receive))
        .route(ADMIN_RETURNS_ID_CANCEL, post(returns::cancel))
        .route(ADMIN_RETURNS_ID_RECEIVE_ITEMS, post(returns::receive_items))
        .route(ADMIN_RETURNS_ID_RECEIVE_CONFIRM, post(returns::confirm_receive))
        .route(ADMIN_RETURNS_ID_REQUEST, post(returns::request))
        .route(ADMIN_RETURNS_ID_SHIPPING_METHOD, post(returns::add_shipping_method))
        .route(ADMIN_RETURNS_ID_DISMISS_ITEMS, post(returns::dismiss_items))
        // Return Reasons
        .route(ADMIN_RETURN_REASONS, get(return_reasons::list).post(return_reasons::create))
        .route(
            ADMIN_RETURN_REASONS_ID,
            get(return_reasons::get)
                .put(return_reasons::update)
                .delete(return_reasons::delete_one),
        )
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
        // Sales Channels
        .route(ADMIN_SALES_CHANNELS, get(sales_channels::list).post(sales_channels::create))
        .route(
            ADMIN_SALES_CHANNELS_ID,
            get(sales_channels::get).put(sales_channels::update).delete(sales_channels::delete_one),
        )
        .route(
            ADMIN_SALES_CHANNELS_ID_PRODUCTS,
            post(sales_channels::add_products).delete(sales_channels::remove_products),
        )
        // Stock Locations
        .route(ADMIN_STOCK_LOCATIONS, get(stock_locations::list).post(stock_locations::create))
        .route(
            ADMIN_STOCK_LOCATIONS_ID,
            get(stock_locations::get).post(stock_locations::update).delete(stock_locations::delete_one),
        )
        .route(ADMIN_STOCK_LOCATIONS_ID_FULFILLMENT_PROVIDERS, post(stock_locations::link_fulfillment_providers))
        .route(ADMIN_STOCK_LOCATIONS_ID_FULFILLMENT_SETS, post(stock_locations::link_fulfillment_sets))
        .route(ADMIN_STOCK_LOCATIONS_ID_SALES_CHANNELS, post(stock_locations::link_sales_channels))
        // Stores
        .route(ADMIN_STORES, get(stores::list))
        .route(ADMIN_STORES_ID, get(stores::get).post(stores::update))
        // Customer Groups
        .route(ADMIN_CUSTOMER_GROUPS, get(customer_groups::list).post(customer_groups::create))
        .route(
            ADMIN_CUSTOMER_GROUPS_ID,
            get(customer_groups::get).post(customer_groups::update).delete(customer_groups::delete_one),
        )
        .route(
            ADMIN_CUSTOMER_GROUPS_ID_CUSTOMERS,
            get(customer_groups::list_customers)
                .post(customer_groups::add_customers)
                .delete(customer_groups::remove_customers),
        )
        // API Keys
        .route(ADMIN_API_KEYS, get(api_keys::list).post(api_keys::create))
        .route(
            ADMIN_API_KEYS_ID,
            get(api_keys::get).post(api_keys::update).delete(api_keys::delete_one),
        )
        .route(ADMIN_API_KEYS_ID_REVOKE, post(api_keys::revoke))
        // Invites
        .route(ADMIN_INVITES, get(invites::list).post(invites::create))
        .route(ADMIN_INVITES_ID, delete(invites::delete_one))
        .route(ADMIN_INVITES_ID_ACCEPT, post(invites::accept))
        // Tax Providers
        .route(ADMIN_TAX_PROVIDERS, get(tax_providers::list))
        // Tax Regions
        .route(ADMIN_TAX_REGIONS, get(tax_regions::list).post(tax_regions::create))
        .route(ADMIN_TAX_REGIONS_ID, get(tax_regions::get).delete(tax_regions::delete_one))
        // Shipping Profiles
        .route(ADMIN_SHIPPING_PROFILES, get(shipping_profiles::list).post(shipping_profiles::create))
        .route(
            ADMIN_SHIPPING_PROFILES_ID,
            get(shipping_profiles::get).post(shipping_profiles::update).delete(shipping_profiles::delete_one),
        )
        // Payment Providers (admin)
        .route(ADMIN_PAYMENT_PROVIDERS, get(payment_providers::list))
        // Order Edits
        .route(ADMIN_ORDER_EDITS, get(order_edits::list).post(order_edits::create))
        .route(
            ADMIN_ORDER_EDITS_ID,
            get(order_edits::get).post(order_edits::update).delete(order_edits::delete_one),
        )
        .route(ADMIN_ORDER_EDITS_ID_REQUEST, post(order_edits::request))
        .route(ADMIN_ORDER_EDITS_ID_CONFIRM, post(order_edits::confirm))
        .route(ADMIN_ORDER_EDITS_ID_DECLINE, post(order_edits::decline))
        .route(ADMIN_ORDER_EDITS_ID_CANCEL, post(order_edits::cancel))
        .route(ADMIN_ORDER_EDITS_ID_ITEMS, post(order_edits::add_line_item))
        .route(ADMIN_ORDER_EDITS_ID_ITEMS_ID, post(order_edits::update_line_item))
        .route(ADMIN_ORDER_EDITS_ID_CHANGES_ID, delete(order_edits::delete_item_change))
        // Promotions
        .route(ADMIN_PROMOTIONS, get(promotions::list).post(promotions::create))
        .route(
            ADMIN_PROMOTIONS_ID,
            get(promotions::get).post(promotions::update).delete(promotions::delete_one),
        )
        .route(
            ADMIN_PROMOTIONS_ID_RULES,
            post(promotions::add_rules).delete(promotions::remove_rules),
        )
        .route(
            ADMIN_PROMOTIONS_ID_BUY_RULES,
            get(promotions::list_buy_rules),
        )
        .route(
            ADMIN_PROMOTIONS_ID_TARGET_RULES,
            get(promotions::list_target_rules),
        )
        .route(ADMIN_PROMOTIONS_ID_BUY_RULES_BATCH, post(promotions::batch_buy_rules))
        .route(ADMIN_PROMOTIONS_ID_TARGET_RULES_BATCH, post(promotions::batch_target_rules))
        .route(ADMIN_PROMOTIONS_RULE_ATTRIBUTE_OPTIONS, get(promotions::list_rule_attribute_options))
        .route(ADMIN_PROMOTIONS_RULE_VALUE_OPTIONS, get(promotions::list_rule_value_options))
        // Price Preferences
        .route(ADMIN_PRICE_PREFERENCES, get(price_preferences::list).post(price_preferences::create))
        .route(
            ADMIN_PRICE_PREFERENCES_ID,
            get(price_preferences::get).post(price_preferences::update).delete(price_preferences::delete_one),
        )
        // Campaigns
        .route(ADMIN_CAMPAIGNS, get(campaigns::list).post(campaigns::create))
        .route(
            ADMIN_CAMPAIGNS_ID,
            get(campaigns::get).post(campaigns::update).delete(campaigns::delete_one),
        )
        .route(ADMIN_CAMPAIGNS_ID_PROMOTIONS, post(campaigns::add_promotions))
        // Fulfillment Sets
        .route(ADMIN_FULFILLMENT_SETS_ID, get(fulfillment_sets::get).delete(fulfillment_sets::delete_one))
        .route(ADMIN_FULFILLMENT_SETS_ID_SERVICE_ZONES, post(fulfillment_sets::create_service_zone))
        .route(
            ADMIN_FULFILLMENT_SETS_ID_SERVICE_ZONES_ID,
            post(fulfillment_sets::update_service_zone).delete(fulfillment_sets::delete_service_zone),
        )
        // Claims
        .route(ADMIN_CLAIMS, get(claims::list))
        .route(
            ADMIN_CLAIMS_ID,
            get(claims::get).post(claims::update).delete(claims::delete_one),
        )
        .route(ADMIN_CLAIMS_ID_CANCEL, post(claims::cancel))
        .route(ADMIN_CLAIMS_ID_CONFIRM, post(claims::confirm))
        .route(ADMIN_CLAIMS_ID_REQUEST, post(claims::request))
        .route(ADMIN_CLAIMS_ID_OUTBOUND_ITEMS, post(claims::add_outbound_items))
        .route(ADMIN_CLAIMS_ID_INBOUND_ITEMS, post(claims::add_inbound_items))
        .route(ADMIN_CLAIMS_ID_SHIPPING_METHOD, post(claims::add_shipping_method))
        // Exchanges
        .route(ADMIN_EXCHANGES, get(exchanges::list).post(exchanges::create))
        .route(ADMIN_EXCHANGES_ID, get(exchanges::get))
        .route(ADMIN_EXCHANGES_ID_CANCEL, post(exchanges::cancel))
        .route(ADMIN_EXCHANGES_ID_CONFIRM, post(exchanges::confirm))
        .route(ADMIN_EXCHANGES_ID_REQUEST, post(exchanges::request))
        .route(ADMIN_EXCHANGES_ID_OUTBOUND_ITEMS, post(exchanges::add_outbound_items))
        .route(ADMIN_EXCHANGES_ID_INBOUND_ITEMS, post(exchanges::add_inbound_items))
        .route(ADMIN_EXCHANGES_ID_SHIPPING_METHOD, post(exchanges::add_shipping_method))
        // Workflow Executions
        .route(ADMIN_WORKFLOWS_EXECUTIONS, get(workflow_executions::list))
        .route(ADMIN_WORKFLOWS_EXECUTIONS_ID, get(workflow_executions::get))
        .route(ADMIN_WORKFLOWS_EXECUTIONS_ID_RUN, post(workflow_executions::run))
        // Fulfillment Providers
        .route(ADMIN_FULFILLMENT_PROVIDERS, get(fulfillment_providers::list))
        .route(ADMIN_FULFILLMENT_PROVIDERS_ID, get(fulfillment_providers::get))
        .route(ADMIN_FULFILLMENT_PROVIDERS_ID_OPTIONS, get(fulfillment_providers::list_options))
        // Payment Collections (admin)
        .route(ADMIN_PAYMENT_COLLECTIONS, get(payment_collections::list).post(payment_collections::create))
        .route(
            ADMIN_PAYMENT_COLLECTIONS_ID,
            get(payment_collections::get).post(payment_collections::update).delete(payment_collections::delete_one),
        )
        .route(ADMIN_PAYMENT_COLLECTIONS_ID_MARK_AS_PAID, post(payment_collections::mark_as_paid))
        // Refund Reasons
        .route(ADMIN_REFUND_REASONS, get(refund_reasons::list).post(refund_reasons::create))
        .route(
            ADMIN_REFUND_REASONS_ID,
            get(refund_reasons::get).post(refund_reasons::update).delete(refund_reasons::delete_one),
        )
        // Reservations
        .route(ADMIN_RESERVATIONS, get(reservations::list).post(reservations::create))
        .route(
            ADMIN_RESERVATIONS_ID,
            get(reservations::get).post(reservations::update).delete(reservations::delete_one),
        )
        // Product Tags (admin)
        .route(ADMIN_PRODUCT_TAGS, get(product_tags::list).post(product_tags::create))
        .route(
            ADMIN_PRODUCT_TAGS_ID,
            get(product_tags::get).post(product_tags::update).delete(product_tags::delete_one),
        )
        // Product Types (admin)
        .route(ADMIN_PRODUCT_TYPES, get(product_types::list).post(product_types::create))
        .route(
            ADMIN_PRODUCT_TYPES_ID,
            get(product_types::get).post(product_types::update).delete(product_types::delete_one),
        )
        // Payments (admin)
        .route(ADMIN_PAYMENTS, get(payments::list))
        .route(ADMIN_PAYMENTS_ID, get(payments::get))
        .route(ADMIN_PAYMENTS_ID_CAPTURE, post(payments::capture))
        .route(ADMIN_PAYMENTS_ID_REFUND, post(payments::refund))
        // Plugins
        .route(ADMIN_PLUGINS, get(plugins::list))
        // Shipping Option Types
        .route(ADMIN_SHIPPING_OPTION_TYPES, get(shipping_option_types::list).post(shipping_option_types::create))
        .route(
            ADMIN_SHIPPING_OPTION_TYPES_ID,
            get(shipping_option_types::get).post(shipping_option_types::update).delete(shipping_option_types::delete_one),
        )
        // Feature Flags
        .route(ADMIN_FEATURE_FLAGS, get(feature_flags::list))
        // Order Changes
        .route(ADMIN_ORDER_CHANGES_ID, delete(order_edits::delete_item_change))
        // Notifications
        .route(ADMIN_NOTIFICATIONS, get(notifications::list))
        .route(ADMIN_NOTIFICATIONS_ID, get(notifications::get))
        .route(ADMIN_NOTIFICATIONS_ID_RESEND, post(notifications::resend))
        // Apply auth middleware to all protected routes.
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            admin_auth_middleware,
        ));

    Router::new().merge(public_routes).merge(protected_routes)
}
