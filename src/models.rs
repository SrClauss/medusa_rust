//! All entity models shared by store and admin handlers.
//!
//! Every struct that comes back from the database uses `sqlx::FromRow`.
//! Every struct sent to the API client uses `serde::Serialize`.
//! Structs used as request bodies use `serde::Deserialize + validator::Validate`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

// ─── Pagination ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub expand: Option<String>,
    pub fields: Option<String>,
    pub order: Option<String>,
    pub q: Option<String>,
}
fn default_limit() -> i64 { 20 }

// ─── Address ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Address {
    pub id: Uuid,
    pub customer_id: Option<Uuid>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub address_1: Option<String>,
    pub address_2: Option<String>,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub province: Option<String>,
    pub postal_code: Option<String>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddressInput {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub address_1: Option<String>,
    pub address_2: Option<String>,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub province: Option<String>,
    pub postal_code: Option<String>,
    pub metadata: Option<Value>,
}

// ─── Country ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Country {
    pub id: Uuid,
    pub iso_2: String,
    pub iso_3: String,
    pub num_code: Option<i32>,
    pub name: String,
    pub display_name: String,
    pub region_id: Option<Uuid>,
}

// ─── Currency ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Currency {
    pub code: String,
    pub symbol: String,
    pub symbol_native: String,
    pub name: String,
    pub includes_tax: bool,
}

// ─── Region ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Region {
    pub id: Uuid,
    pub name: String,
    pub currency_code: String,
    pub tax_rate: f64,
    pub tax_code: Option<String>,
    pub gift_cards_taxable: bool,
    pub automatic_taxes: bool,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegionWithRelations {
    #[serde(flatten)]
    pub region: Region,
    pub countries: Vec<Country>,
    pub currency: Option<Currency>,
    pub payment_providers: Vec<PaymentProvider>,
    pub fulfillment_providers: Vec<FulfillmentProvider>,
    pub tax_rates: Vec<TaxRate>,
}

// ─── Payment / Fulfillment Providers ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentProvider {
    pub id: String,
    pub is_installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FulfillmentProvider {
    pub id: String,
    pub is_installed: bool,
}

// ─── Product Type ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductType {
    pub id: Uuid,
    pub value: String,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ─── Product Tag ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductTag {
    pub id: Uuid,
    pub value: String,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ─── Product Image ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductImage {
    pub id: Uuid,
    pub url: String,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ─── Product Option ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductOption {
    pub id: Uuid,
    pub title: String,
    pub product_id: Uuid,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductOptionValue {
    pub id: Uuid,
    pub value: String,
    pub option_id: Uuid,
    pub variant_id: Uuid,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ─── Money Amount ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MoneyAmount {
    pub id: Uuid,
    pub currency_code: String,
    pub amount: i64,
    pub variant_id: Uuid,
    pub region_id: Option<Uuid>,
    pub price_list_id: Option<Uuid>,
    pub min_quantity: Option<i32>,
    pub max_quantity: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ─── Product Variant ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductVariant {
    pub id: Uuid,
    pub title: String,
    pub product_id: Uuid,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub ean: Option<String>,
    pub upc: Option<String>,
    pub inventory_quantity: i32,
    pub allow_backorder: bool,
    pub manage_inventory: bool,
    pub hs_code: Option<String>,
    pub origin_country: Option<String>,
    pub mid_code: Option<String>,
    pub material: Option<String>,
    pub weight: Option<f64>,
    pub length: Option<f64>,
    pub height: Option<f64>,
    pub width: Option<f64>,
    pub variant_rank: Option<i32>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductVariantWithRelations {
    #[serde(flatten)]
    pub variant: ProductVariant,
    pub prices: Vec<MoneyAmount>,
    pub options: Vec<ProductOptionValue>,
}

// ─── Product Category ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductCategory {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub handle: String,
    pub is_active: bool,
    pub is_internal: bool,
    pub parent_category_id: Option<Uuid>,
    pub rank: i32,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Product Collection ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductCollection {
    pub id: Uuid,
    pub title: String,
    pub handle: String,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ─── Product ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub handle: String,
    pub is_giftcard: bool,
    pub status: String,
    pub thumbnail: Option<String>,
    pub weight: Option<f64>,
    pub length: Option<f64>,
    pub height: Option<f64>,
    pub width: Option<f64>,
    pub hs_code: Option<String>,
    pub origin_country: Option<String>,
    pub mid_code: Option<String>,
    pub material: Option<String>,
    pub collection_id: Option<Uuid>,
    pub type_id: Option<Uuid>,
    pub discountable: bool,
    pub external_id: Option<String>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductWithRelations {
    #[serde(flatten)]
    pub product: Product,
    pub variants: Vec<ProductVariantWithRelations>,
    pub options: Vec<ProductOption>,
    pub images: Vec<ProductImage>,
    pub tags: Vec<ProductTag>,
    pub categories: Vec<ProductCategory>,
    pub collection: Option<ProductCollection>,
    #[serde(rename = "type")]
    pub product_type: Option<ProductType>,
}

// ─── Shipping Profile ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShippingProfile {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub profile_type: String,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ─── Shipping Option ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShippingOption {
    pub id: Uuid,
    pub name: String,
    pub region_id: Uuid,
    pub profile_id: Uuid,
    pub provider_id: String,
    pub price_type: String,
    pub amount: Option<i64>,
    pub is_return: bool,
    pub admin_only: bool,
    pub data: Option<Value>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShippingOptionWithRelations {
    #[serde(flatten)]
    pub shipping_option: ShippingOption,
    pub region: Option<Region>,
    pub requirements: Vec<ShippingOptionRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShippingOptionRequirement {
    pub id: Uuid,
    pub shipping_option_id: Uuid,
    #[serde(rename = "type")]
    pub requirement_type: String,
    pub amount: i64,
}

// ─── Shipping Method ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShippingMethod {
    pub id: Uuid,
    pub shipping_option_id: Uuid,
    pub cart_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    pub claim_order_id: Option<Uuid>,
    pub swap_id: Option<Uuid>,
    pub return_id: Option<Uuid>,
    pub price: i64,
    pub data: Option<Value>,
}

// ─── Tax Rate ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaxRate {
    pub id: Uuid,
    pub rate: Option<f64>,
    pub code: Option<String>,
    pub name: String,
    pub region_id: Uuid,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Discount ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Discount {
    pub id: Uuid,
    pub code: String,
    pub is_dynamic: bool,
    pub rule_id: Option<Uuid>,
    pub is_disabled: bool,
    pub parent_discount_id: Option<Uuid>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub valid_duration: Option<String>,
    pub usage_limit: Option<i32>,
    pub usage_count: i32,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DiscountRule {
    pub id: Uuid,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub rule_type: String,
    pub value: i64,
    pub allocation: Option<String>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ─── Gift Card ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GiftCard {
    pub id: Uuid,
    pub code: String,
    pub value: i64,
    pub balance: i64,
    pub region_id: Uuid,
    pub order_id: Option<Uuid>,
    pub is_disabled: bool,
    pub ends_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ─── Customer ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Customer {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub billing_address_id: Option<Uuid>,
    pub phone: Option<String>,
    pub has_account: bool,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CustomerWithRelations {
    #[serde(flatten)]
    pub customer: Customer,
    pub shipping_addresses: Vec<Address>,
    pub billing_address: Option<Address>,
}

// ─── Payment Session ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentSession {
    pub id: Uuid,
    pub cart_id: Option<Uuid>,
    pub provider_id: String,
    pub is_selected: Option<bool>,
    pub is_initiated: bool,
    pub status: String,
    pub data: Option<Value>,
    pub idempotency_key: Option<String>,
    pub amount: Option<i64>,
    pub payment_authorized_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Payment ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub swap_id: Option<Uuid>,
    pub cart_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    pub amount: i64,
    pub currency_code: String,
    pub amount_refunded: i64,
    pub provider_id: String,
    pub data: Option<Value>,
    pub captured_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Line Item ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LineItem {
    pub id: Uuid,
    pub cart_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    pub swap_id: Option<Uuid>,
    pub claim_order_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub is_return: bool,
    pub is_giftcard: bool,
    pub should_merge: bool,
    pub allow_discounts: bool,
    pub has_shipping: Option<bool>,
    pub unit_price: i64,
    pub variant_id: Option<Uuid>,
    pub quantity: i32,
    pub fulfilled_quantity: Option<i32>,
    pub returned_quantity: Option<i32>,
    pub shipped_quantity: Option<i32>,
    pub metadata: Option<Value>,
    pub original_item_id: Option<Uuid>,
    pub order_edit_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LineItemWithRelations {
    #[serde(flatten)]
    pub line_item: LineItem,
    pub variant: Option<ProductVariantWithRelations>,
    pub adjustments: Vec<LineItemAdjustment>,
    pub tax_lines: Vec<LineItemTaxLine>,
    pub subtotal: Option<i64>,
    pub discount_total: Option<i64>,
    pub total: Option<i64>,
    pub original_total: Option<i64>,
    pub original_tax_total: Option<i64>,
    pub tax_total: Option<i64>,
    pub unit_price_excl_tax: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LineItemAdjustment {
    pub id: Uuid,
    pub item_id: Uuid,
    pub description: String,
    pub discount_id: Option<Uuid>,
    pub amount: i64,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LineItemTaxLine {
    pub id: Uuid,
    pub rate: f64,
    pub name: String,
    pub code: Option<String>,
    pub item_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Cart ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Cart {
    pub id: Uuid,
    pub email: Option<String>,
    pub billing_address_id: Option<Uuid>,
    pub shipping_address_id: Option<Uuid>,
    pub region_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub payment_id: Option<Uuid>,
    pub cart_type: String,  // mapped from "type" column
    pub completed_at: Option<DateTime<Utc>>,
    pub payment_authorized_at: Option<DateTime<Utc>>,
    pub idempotency_key: Option<String>,
    pub context: Option<Value>,
    pub sales_channel_id: Option<Uuid>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CartWithRelations {
    pub id: Uuid,
    pub email: Option<String>,
    pub billing_address_id: Option<Uuid>,
    pub billing_address: Option<Address>,
    pub shipping_address_id: Option<Uuid>,
    pub shipping_address: Option<Address>,
    pub items: Vec<LineItemWithRelations>,
    pub region_id: Uuid,
    pub region: Option<RegionWithRelations>,
    pub discounts: Vec<Discount>,
    pub gift_cards: Vec<GiftCard>,
    pub customer_id: Option<Uuid>,
    pub customer: Option<Customer>,
    pub payment_session: Option<PaymentSession>,
    pub payment_sessions: Vec<PaymentSession>,
    pub payment: Option<Payment>,
    pub shipping_methods: Vec<ShippingMethod>,
    #[serde(rename = "type")]
    pub cart_type: String,
    pub completed_at: Option<DateTime<Utc>>,
    pub payment_authorized_at: Option<DateTime<Utc>>,
    pub idempotency_key: Option<String>,
    pub context: Option<Value>,
    pub sales_channel_id: Option<Uuid>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    // computed totals
    pub subtotal: i64,
    pub discount_total: i64,
    pub shipping_total: i64,
    pub tax_total: i64,
    pub gift_card_total: i64,
    pub total: i64,
}

// ─── Order ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub status: String,
    pub fulfillment_status: String,
    pub payment_status: String,
    pub display_id: i32,
    pub cart_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub email: String,
    pub billing_address_id: Option<Uuid>,
    pub shipping_address_id: Option<Uuid>,
    pub region_id: Uuid,
    pub currency_code: String,
    pub tax_rate: Option<f64>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
    pub no_notification: Option<bool>,
    pub idempotency_key: Option<String>,
    pub external_id: Option<String>,
    pub sales_channel_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderWithRelations {
    #[serde(flatten)]
    pub order: Order,
    pub billing_address: Option<Address>,
    pub shipping_address: Option<Address>,
    pub items: Vec<LineItemWithRelations>,
    pub region: Option<Region>,
    pub discounts: Vec<Discount>,
    pub gift_cards: Vec<GiftCard>,
    pub shipping_methods: Vec<ShippingMethod>,
    pub payments: Vec<Payment>,
    pub fulfillments: Vec<Fulfillment>,
    pub returns: Vec<Return>,
    pub claims: Vec<ClaimOrder>,
    pub swaps: Vec<Swap>,
    pub customer: Option<Customer>,
    // totals
    pub subtotal: i64,
    pub discount_total: i64,
    pub shipping_total: i64,
    pub tax_total: i64,
    pub refunded_total: i64,
    pub total: i64,
    pub paid_total: i64,
    pub refundable_amount: i64,
    pub gift_card_total: i64,
}

// ─── Fulfillment ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Fulfillment {
    pub id: Uuid,
    pub claim_order_id: Option<Uuid>,
    pub swap_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    pub provider_id: String,
    pub location_id: Option<Uuid>,
    pub no_notification: Option<bool>,
    pub tracking_numbers: Option<Value>,
    pub data: Option<Value>,
    pub shipped_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Return ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Return {
    pub id: Uuid,
    pub status: String,
    pub order_id: Option<Uuid>,
    pub swap_id: Option<Uuid>,
    pub claim_order_id: Option<Uuid>,
    pub shipping_method_id: Option<Uuid>,
    pub refund_amount: Option<i64>,
    pub no_notification: Option<bool>,
    pub idempotency_key: Option<String>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Swap ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Swap {
    pub id: Uuid,
    pub fulfillment_status: String,
    pub payment_status: String,
    pub order_id: Uuid,
    pub difference_due: Option<i64>,
    pub cart_id: Option<Uuid>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub no_notification: Option<bool>,
    pub allow_backorder: bool,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Claim Order ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClaimOrder {
    pub id: Uuid,
    pub payment_status: String,
    pub fulfillment_status: String,
    pub claim_type: String,
    pub order_id: Uuid,
    pub shipping_address_id: Option<Uuid>,
    pub refund_amount: Option<i64>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Admin User ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminUser {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: String,
    pub api_token: Option<String>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminUserWithPassword {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: String,
    pub password_hash: Option<String>,
    pub api_token: Option<String>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ─── Price List ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PriceList {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub price_list_type: String,
    pub status: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ─── Inventory Item ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryItem {
    pub id: Uuid,
    pub sku: Option<String>,
    pub origin_country: Option<String>,
    pub hs_code: Option<String>,
    pub mid_code: Option<String>,
    pub material: Option<String>,
    pub weight: Option<f64>,
    pub length: Option<f64>,
    pub height: Option<f64>,
    pub width: Option<f64>,
    pub requires_shipping: bool,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub title: Option<String>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InventoryLevel {
    pub id: Uuid,
    pub inventory_item_id: Uuid,
    pub location_id: Uuid,
    pub stocked_quantity: i32,
    pub reserved_quantity: i32,
    pub incoming_quantity: i32,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Draft Order ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DraftOrder {
    pub id: Uuid,
    pub status: String,
    pub display_id: i32,
    pub cart_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub no_notification_order: Option<bool>,
    pub idempotency_key: Option<String>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Batch Job ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BatchJob {
    pub id: Uuid,
    pub batch_type: String,
    pub created_by: Option<Uuid>,
    pub context: Option<Value>,
    pub result: Option<Value>,
    pub dry_run: bool,
    pub status: String,
    pub pre_processed_at: Option<DateTime<Utc>>,
    pub processing_at: Option<DateTime<Utc>>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Discount Condition ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DiscountCondition {
    pub id: Uuid,
    pub discount_rule_id: Uuid,
    pub condition_type: String,
    pub operator: String,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// ─── Refund ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Refund {
    pub id: Uuid,
    pub order_id: Uuid,
    pub amount: i64,
    pub note: Option<String>,
    pub reason: String,
    pub payment_id: Option<Uuid>,
    pub metadata: Option<Value>,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
