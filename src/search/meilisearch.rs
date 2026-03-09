//! MeiliSearch search provider.
//!
//! Enabled with Cargo feature `search-meilisearch`.

#[cfg(feature = "search-meilisearch")]
use crate::search::{IndexDocument, SearchProvider, SearchQuery, SearchResults};
#[cfg(feature = "search-meilisearch")]
use async_trait::async_trait;
#[cfg(feature = "search-meilisearch")]
use serde_json::json;

/// MeiliSearch provider using the REST API directly.
///
/// Configure via environment variables:
/// - `MEILISEARCH_URL`     — MeiliSearch host URL (default `http://localhost:7700`)
/// - `MEILISEARCH_API_KEY` — Master or search API key
#[cfg(feature = "search-meilisearch")]
pub struct MeiliSearchProvider {
    base_url: String,
    api_key: Option<String>,
    http: reqwest::Client,
}

#[cfg(feature = "search-meilisearch")]
impl MeiliSearchProvider {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            http: reqwest::Client::new(),
        }
    }

    /// Builds from environment variables.
    pub fn from_env() -> Self {
        let base_url = std::env::var("MEILISEARCH_URL")
            .unwrap_or_else(|_| "http://localhost:7700".into());
        let api_key = std::env::var("MEILISEARCH_API_KEY").ok();
        Self::new(base_url, api_key)
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.request(method, url);
        if let Some(ref key) = self.api_key {
            req = req.bearer_auth(key);
        }
        req
    }
}

#[cfg(feature = "search-meilisearch")]
#[async_trait]
impl SearchProvider for MeiliSearchProvider {
    fn name(&self) -> &str {
        "meilisearch"
    }

    async fn index_documents(
        &self,
        index_name: &str,
        documents: Vec<IndexDocument>,
    ) -> anyhow::Result<()> {
        // Flatten each document: merge `id` into the `data` object.
        let docs: Vec<serde_json::Value> = documents
            .into_iter()
            .map(|d| {
                let mut v = d.data;
                if let Some(obj) = v.as_object_mut() {
                    obj.insert("id".into(), serde_json::Value::String(d.id));
                }
                v
            })
            .collect();

        self.request(
            reqwest::Method::POST,
            &format!("/indexes/{}/documents", index_name),
        )
        .json(&docs)
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
        self.request(
            reqwest::Method::POST,
            &format!("/indexes/{}/documents/delete-batch", index_name),
        )
        .json(&ids)
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
        let mut body = json!({
            "q": query.q,
            "limit": query.limit.unwrap_or(20),
            "offset": query.offset.unwrap_or(0),
        });

        if let Some(filter) = query.filter {
            body["filter"] = serde_json::Value::String(filter);
        }

        let resp: serde_json::Value = self
            .request(
                reqwest::Method::POST,
                &format!("/indexes/{}/search", index_name),
            )
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
            .get("estimatedTotalHits")
            .or_else(|| resp.get("totalHits"))
            .and_then(|t| t.as_u64());

        Ok(SearchResults {
            hits,
            total,
            offset: query.offset,
            limit: query.limit,
        })
    }
}
