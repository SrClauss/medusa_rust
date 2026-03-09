//! Algolia search provider.
//!
//! Enabled with Cargo feature `search-algolia`.

#[cfg(feature = "search-algolia")]
use crate::search::{IndexDocument, SearchProvider, SearchQuery, SearchResults};
#[cfg(feature = "search-algolia")]
use async_trait::async_trait;
#[cfg(feature = "search-algolia")]
use serde_json::json;

/// Algolia provider using the Search REST API.
///
/// Configure via environment variables:
/// - `ALGOLIA_APP_ID`    — Algolia Application ID
/// - `ALGOLIA_ADMIN_KEY` — Admin API key (for write operations)
/// - `ALGOLIA_SEARCH_KEY`— Search-only API key (for read operations)
#[cfg(feature = "search-algolia")]
pub struct AlgoliaSearchProvider {
    app_id: String,
    admin_key: String,
    search_key: String,
    http: reqwest::Client,
}

#[cfg(feature = "search-algolia")]
impl AlgoliaSearchProvider {
    pub fn new(app_id: String, admin_key: String, search_key: String) -> Self {
        Self {
            app_id,
            admin_key,
            search_key,
            http: reqwest::Client::new(),
        }
    }

    /// Builds from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        let app_id = std::env::var("ALGOLIA_APP_ID")
            .map_err(|_| anyhow::anyhow!("ALGOLIA_APP_ID not set"))?;
        let admin_key = std::env::var("ALGOLIA_ADMIN_KEY")
            .map_err(|_| anyhow::anyhow!("ALGOLIA_ADMIN_KEY not set"))?;
        let search_key = std::env::var("ALGOLIA_SEARCH_KEY")
            .unwrap_or_else(|_| admin_key.clone());
        Ok(Self::new(app_id, admin_key, search_key))
    }

    fn write_url(&self, path: &str) -> String {
        format!("https://{}.algolia.net{}", self.app_id, path)
    }

    fn read_url(&self, path: &str) -> String {
        format!("https://{}-dsn.algolia.net{}", self.app_id, path)
    }
}

#[cfg(feature = "search-algolia")]
#[async_trait]
impl SearchProvider for AlgoliaSearchProvider {
    fn name(&self) -> &str {
        "algolia"
    }

    async fn index_documents(
        &self,
        index_name: &str,
        documents: Vec<IndexDocument>,
    ) -> anyhow::Result<()> {
        // Algolia batch endpoint.
        let requests: Vec<serde_json::Value> = documents
            .into_iter()
            .map(|d| {
                let mut body = d.data;
                if let Some(obj) = body.as_object_mut() {
                    obj.insert("objectID".into(), serde_json::Value::String(d.id));
                }
                json!({ "action": "updateObject", "body": body })
            })
            .collect();

        self.http
            .post(self.write_url(&format!("/1/indexes/{}/batch", index_name)))
            .header("X-Algolia-Application-Id", &self.app_id)
            .header("X-Algolia-API-Key", &self.admin_key)
            .json(&json!({ "requests": requests }))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn delete_documents(
        &self,
        index_name: &str,
        ids: Vec<String>,
    ) -> anyhow::Result<()> {
        let requests: Vec<serde_json::Value> = ids
            .into_iter()
            .map(|id| json!({ "action": "deleteObject", "body": { "objectID": id } }))
            .collect();

        self.http
            .post(self.write_url(&format!("/1/indexes/{}/batch", index_name)))
            .header("X-Algolia-Application-Id", &self.app_id)
            .header("X-Algolia-API-Key", &self.admin_key)
            .json(&json!({ "requests": requests }))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn search(
        &self,
        index_name: &str,
        query: SearchQuery,
    ) -> anyhow::Result<SearchResults> {
        let body = json!({
            "query": query.q,
            "hitsPerPage": query.limit.unwrap_or(20),
            // Algolia uses page-based pagination; derive page from offset + limit.
            // Note: offset must be a multiple of limit for accurate pagination.
            "page": query.offset.unwrap_or(0) / query.limit.unwrap_or(20).max(1),
            "filters": query.filter,
        });

        let resp: serde_json::Value = self
            .http
            .post(self.read_url(&format!("/1/indexes/{}/query", index_name)))
            .header("X-Algolia-Application-Id", &self.app_id)
            .header("X-Algolia-API-Key", &self.search_key)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let hits = resp
            .get("hits")
            .and_then(|h| h.as_array())
            .cloned()
            .unwrap_or_default();

        let total = resp
            .get("nbHits")
            .and_then(|t| t.as_u64());

        Ok(SearchResults {
            hits,
            total,
            offset: query.offset,
            limit: query.limit,
        })
    }
}
