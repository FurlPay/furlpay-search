//! Extensible search provider registry with health monitoring and telemetry.

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use crate::providers::adapters::{BraveSearchProvider, DuckDuckGoProvider, SearxngProvider, SerpApiProvider};
use crate::providers::search_provider::{SearchHit, SearchOptions, SearchProvider};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderHealth {
    pub name: String,
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub last_latency_ms: u64,
    pub is_enabled: bool,
}

pub struct ProviderRegistry {
    providers: Vec<Box<dyn SearchProvider>>,
    health_metrics: Arc<RwLock<Vec<ProviderHealth>>>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderRegistry {
    /// Initialize registry with default auto-detected providers.
    pub fn new() -> Self {
        let mut providers: Vec<Box<dyn SearchProvider>> = Vec::new();
        let mut health = Vec::new();

        // 1. DuckDuckGo (always available, zero-key)
        providers.push(Box::new(DuckDuckGoProvider::new()));
        health.push(ProviderHealth {
            name: "DuckDuckGo".into(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            last_latency_ms: 0,
            is_enabled: true,
        });

        // 2. Brave Search (if API key present)
        if let Ok(key) = std::env::var("BRAVE_SEARCH_API_KEY").or_else(|_| std::env::var("FURLPAY_BRAVE_KEY")) {
            if !key.trim().is_empty() {
                providers.push(Box::new(BraveSearchProvider::new(key)));
                health.push(ProviderHealth {
                    name: "BraveSearch".into(),
                    total_requests: 0,
                    successful_requests: 0,
                    failed_requests: 0,
                    last_latency_ms: 0,
                    is_enabled: true,
                });
            }
        }

        // 3. SearXNG (if instance URL present)
        if let Ok(url) = std::env::var("SEARXNG_URL").or_else(|_| std::env::var("FURLPAY_SEARXNG_URL")) {
            if !url.trim().is_empty() {
                providers.push(Box::new(SearxngProvider::new(url)));
                health.push(ProviderHealth {
                    name: "SearXNG".into(),
                    total_requests: 0,
                    successful_requests: 0,
                    failed_requests: 0,
                    last_latency_ms: 0,
                    is_enabled: true,
                });
            }
        }

        // 4. SerpAPI (if API key present)
        if let Ok(key) = std::env::var("SERPAPI_API_KEY").or_else(|_| std::env::var("FURLPAY_SERP_KEY")) {
            if !key.trim().is_empty() {
                providers.push(Box::new(SerpApiProvider::new(key)));
                health.push(ProviderHealth {
                    name: "SerpAPI".into(),
                    total_requests: 0,
                    successful_requests: 0,
                    failed_requests: 0,
                    last_latency_ms: 0,
                    is_enabled: true,
                });
            }
        }

        Self {
            providers,
            health_metrics: Arc::new(RwLock::new(health)),
        }
    }

    /// Register a custom search provider.
    pub fn register_provider(&mut self, provider: Box<dyn SearchProvider>) {
        let name = provider.name().to_string();
        self.providers.push(provider);
        let mut health = futures::executor::block_on(self.health_metrics.write());
        health.push(ProviderHealth {
            name,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            last_latency_ms: 0,
            is_enabled: true,
        });
    }

    /// Query all active providers in parallel and aggregate hits.
    pub async fn dispatch_search(
        &self,
        query: &str,
        options: &SearchOptions,
    ) -> (Vec<SearchHit>, Vec<ProviderHealth>) {
        let mut all_hits = Vec::new();

        for (idx, provider) in self.providers.iter().enumerate() {
            let start = Instant::now();
            let res = provider.search(query, options).await;
            let elapsed_ms = start.elapsed().as_millis() as u64;

            let mut health = self.health_metrics.write().await;
            if let Some(h) = health.get_mut(idx) {
                h.total_requests += 1;
                h.last_latency_ms = elapsed_ms;

                match res {
                    Ok(hits) => {
                        h.successful_requests += 1;
                        all_hits.extend(hits);
                    }
                    Err(e) => {
                        h.failed_requests += 1;
                        tracing::warn!(provider = provider.name(), error = %e, "Provider search failed");
                    }
                }
            }
        }

        let current_health = self.health_metrics.read().await.clone();
        (all_hits, current_health)
    }

    /// Get current health status of all registered providers.
    pub async fn get_health(&self) -> Vec<ProviderHealth> {
        self.health_metrics.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_registry_init() {
        let registry = ProviderRegistry::new();
        let health = registry.get_health().await;
        assert!(!health.is_empty());
        assert_eq!(health[0].name, "DuckDuckGo");
    }
}
