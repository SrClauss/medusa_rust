//! Moka in-process cache helpers.

use moka::future::Cache;
use serde_json::Value;
use std::time::Duration;

/// Constructs the global Moka cache with sensible defaults.
///
/// The cache is generic over `String` keys and `serde_json::Value` values so
/// that heterogeneous data (prices, categories, sessions) can share a single
/// cache instance.
pub fn build_cache(max_capacity: u64, ttl_seconds: u64) -> Cache<String, Value> {
    Cache::builder()
        .max_capacity(max_capacity)
        .time_to_live(Duration::from_secs(ttl_seconds))
        .time_to_idle(Duration::from_secs(ttl_seconds / 2))
        .build()
}

/// Cache namespace prefixes — prevents key collisions between domains.
pub mod ns {
    pub const PRODUCTS: &str = "products";
    pub const CATEGORIES: &str = "categories";
    pub const REGIONS: &str = "regions";
    pub const PRICE_LISTS: &str = "price_lists";
    pub const SESSION: &str = "session";
    pub const SHIPPING: &str = "shipping_options";

    /// Formats a namespaced cache key.
    pub fn key(namespace: &str, id: &str) -> String {
        format!("{namespace}:{id}")
    }
}
