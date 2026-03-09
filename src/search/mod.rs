//! Search module — provider trait and concrete implementations.
//!
//! Mirrors the MedusaJS search plugin architecture (`medusa-plugin-meilisearch`,
//! `medusa-plugin-algolia`).  A single `SearchProvider` trait is implemented by
//! each backend; a `SearchService` registry routes calls to the active provider.
//!
//! # Enabled providers
//!
//! | Feature              | Provider                  |
//! |----------------------|---------------------------|
//! | `search-meilisearch` | `MeiliSearchProvider`     |
//! | `search-algolia`     | `AlgoliaSearchProvider`   |

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// ─── Sub-modules ──────────────────────────────────────────────────────────────

pub mod meilisearch;
pub mod algolia;

// ─── Core types ───────────────────────────────────────────────────────────────

/// A document to be indexed or removed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocument {
    /// Primary key used by the search engine.
    pub id: String,
    /// Arbitrary fields to index / store.
    pub data: serde_json::Value,
}

/// Parameters for a search query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Full-text search string.
    pub q: String,
    /// Maximum number of hits to return.
    pub limit: Option<usize>,
    /// Number of hits to skip (for pagination).
    pub offset: Option<usize>,
    /// Optional filter expression (provider-specific syntax).
    pub filter: Option<String>,
}

/// Search results returned by a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub hits: Vec<serde_json::Value>,
    pub total: Option<u64>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

/// Trait that every search backend must implement.
#[async_trait]
pub trait SearchProvider: Send + Sync {
    /// Short lower-case name, e.g. `"meilisearch"`, `"algolia"`.
    fn name(&self) -> &str;

    /// Index (or re-index) a batch of documents into `index_name`.
    async fn index_documents(
        &self,
        index_name: &str,
        documents: Vec<IndexDocument>,
    ) -> anyhow::Result<()>;

    /// Remove documents by their IDs from `index_name`.
    async fn delete_documents(
        &self,
        index_name: &str,
        ids: Vec<String>,
    ) -> anyhow::Result<()>;

    /// Execute a search query against `index_name`.
    async fn search(
        &self,
        index_name: &str,
        query: SearchQuery,
    ) -> anyhow::Result<SearchResults>;
}

// ─── Service ──────────────────────────────────────────────────────────────────

use std::sync::Arc;

/// Wrapper service over a configured `SearchProvider`.
///
/// The project only ever has one active search backend at a time (unlike
/// MedusaJS which can have multiple plugins, but only the last registered
/// wins per resource).
pub struct SearchService {
    provider: Option<Arc<dyn SearchProvider>>,
}

impl SearchService {
    pub fn new() -> Self {
        Self { provider: None }
    }

    /// Set the active provider.
    pub fn set_provider(&mut self, provider: Arc<dyn SearchProvider>) {
        tracing::info!(provider = %provider.name(), "Search provider registered");
        self.provider = Some(provider);
    }

    fn get_provider(&self) -> anyhow::Result<&Arc<dyn SearchProvider>> {
        self.provider
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No search provider configured"))
    }

    pub async fn index_documents(
        &self,
        index_name: &str,
        documents: Vec<IndexDocument>,
    ) -> anyhow::Result<()> {
        self.get_provider()?.index_documents(index_name, documents).await
    }

    pub async fn delete_documents(
        &self,
        index_name: &str,
        ids: Vec<String>,
    ) -> anyhow::Result<()> {
        self.get_provider()?.delete_documents(index_name, ids).await
    }

    pub async fn search(
        &self,
        index_name: &str,
        query: SearchQuery,
    ) -> anyhow::Result<SearchResults> {
        self.get_provider()?.search(index_name, query).await
    }
}

impl Default for SearchService {
    fn default() -> Self {
        Self::new()
    }
}
