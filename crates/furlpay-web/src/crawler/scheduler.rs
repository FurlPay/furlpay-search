//! Crawl scheduler executing bounded recursive web exploration.

use std::collections::{HashMap, HashSet};
use std::time::Instant;
use serde::{Deserialize, Serialize};

use super::crawl_policy::CrawlPolicy;
use super::frontier::CrawlFrontier;
use super::link_extractor::LinkExtractor;
use crate::fetch::http_client::HttpClient;
use crate::extraction::html::HtmlExtractor;
use crate::security::ssrf_protection::SsrfGuard;

/// Record of an individual crawled web page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawledPage {
    pub url: String,
    pub title: String,
    pub depth: u32,
    pub parent_url: Option<String>,
    pub http_status: u16,
    pub content_text: String,
    pub outbound_links_count: usize,
    pub fetch_latency_ms: u64,
}

/// Link relationship between two URLs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEdge {
    pub source: String,
    pub target: String,
    pub anchor_text: String,
}

/// Result of a completed crawl session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlResult {
    pub seed_urls: Vec<String>,
    pub pages: Vec<CrawledPage>,
    pub link_graph: Vec<LinkEdge>,
    pub total_pages_crawled: usize,
    pub total_domains_visited: usize,
    pub duration_ms: u64,
}

pub struct CrawlScheduler {
    http_client: HttpClient,
    html_extractor: HtmlExtractor,
}

impl CrawlScheduler {
    pub fn new() -> Result<Self, String> {
        let http_client = HttpClient::new().map_err(|e| e.to_string())?;
        let html_extractor = HtmlExtractor::new();
        Ok(Self {
            http_client,
            html_extractor,
        })
    }

    /// Execute a crawl session beginning from the provided seeds.
    pub async fn crawl(&self, seeds: &[String], policy: &CrawlPolicy) -> CrawlResult {
        let start = Instant::now();
        let mut frontier = CrawlFrontier::new();
        let mut visited_domains = HashSet::new();
        let mut crawled_pages = Vec::new();
        let mut link_graph = Vec::new();

        // 1. Enqueue seeds
        let ssrf = SsrfGuard::new();
        for seed in seeds {
            if ssrf.validate_url(seed).is_ok() {
                frontier.push_seed(seed);
            }
        }

        // 2. Main crawl loop
        while let Some(item) = frontier.pop_next() {
            // Check page budget
            if crawled_pages.len() >= policy.max_pages {
                break;
            }
            // Check time budget
            if start.elapsed() >= policy.total_timeout {
                break;
            }

            // Extract domain
            let parsed_url = match url::Url::parse(&item.url) {
                Ok(u) => u,
                Err(_) => continue,
            };
            let domain = parsed_url.host_str().unwrap_or("").to_lowercase();
            if domain.is_empty() {
                continue;
            }

            // Check domain limits
            if !visited_domains.contains(&domain) {
                if visited_domains.len() >= policy.max_domains {
                    continue; // Skip pages from new domains once limit reached
                }
                visited_domains.insert(domain.clone());
            }

            // Fetch page
            let fetch_start = Instant::now();
            let fetch_res = self.http_client.fetch(&item.url).await;
            let fetch_latency_ms = fetch_start.elapsed().as_millis() as u64;

            match fetch_res {
                Ok(resp) => {
                    let body = resp.body_text.unwrap_or_default();
                    let extracted = self.html_extractor.extract(&body, Some(&resp.final_url));

                    let outbound_links = LinkExtractor::extract_links(&body, &resp.final_url);
                    let outbound_count = outbound_links.len();

                    crawled_pages.push(CrawledPage {
                        url: resp.final_url.clone(),
                        title: extracted.title.clone(),
                        depth: item.depth,
                        parent_url: item.parent_url.clone(),
                        http_status: resp.status,
                        content_text: extracted.main_text.chars().take(2000).collect(),
                        outbound_links_count: outbound_count,
                        fetch_latency_ms,
                    });

                    // Discover next links if depth allows
                    if item.depth < policy.max_depth {
                        for link in outbound_links {
                            // Enforce same_domain policy if set
                            if policy.same_domain_only && !link.is_internal {
                                continue;
                            }

                            // Record edge in link graph
                            link_graph.push(LinkEdge {
                                source: resp.final_url.clone(),
                                target: link.url.clone(),
                                anchor_text: link.anchor_text.clone(),
                            });

                            // Add to frontier
                            frontier.push_discovered(&link.url, item.depth + 1, &resp.final_url, &link.anchor_text);
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!(url = %item.url, error = %e, "Crawl fetch failed");
                }
            }

            // Yield briefly to avoid hogging executor
            tokio::task::yield_now().await;
        }

        let total_domains_visited = visited_domains.len();
        let total_pages_crawled = crawled_pages.len();

        CrawlResult {
            seed_urls: seeds.to_vec(),
            pages: crawled_pages,
            link_graph,
            total_pages_crawled,
            total_domains_visited,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crawl_scheduler_initializes() {
        let scheduler = CrawlScheduler::new();
        assert!(scheduler.is_ok());
    }
}
