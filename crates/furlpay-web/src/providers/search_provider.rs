//! Provider-independent search interface and data contracts.

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub published_date: Option<String>,
    pub provider: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub country: Option<String>,
    pub language: Option<String>,
    pub freshness: Option<String>,
    pub limit: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            country: None,
            language: None,
            freshness: None,
            limit: 10,
        }
    }
}

pub trait SearchProvider: Send + Sync {
    /// Human-readable provider name (e.g. "DuckDuckGo", "BraveSearch", "SearXNG", "SerpAPI").
    fn name(&self) -> &str;

    /// Execute a search request across this provider.
    fn search<'a>(&'a self, query: &'a str, options: &'a SearchOptions) -> BoxFuture<'a, Result<Vec<SearchHit>, String>>;
}
