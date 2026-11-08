//! Crawl policy, scope boundaries, and depth control.

use std::time::Duration;

/// Rules governing crawl depth, page volume, domain boundaries, and pacing.
#[derive(Debug, Clone)]
pub struct CrawlPolicy {
    /// Maximum link hops from seed URLs (default: 2).
    pub max_depth: u32,
    /// Maximum total pages to fetch across the entire crawl (default: 25).
    pub max_pages: usize,
    /// Maximum unique domains to visit (default: 5).
    pub max_domains: usize,
    /// Whether to restrict crawling strictly to the seed domains.
    pub same_domain_only: bool,
    /// Delay between consecutive requests to the same domain.
    pub per_domain_delay: Duration,
    /// Total time budget for the crawl execution.
    pub total_timeout: Duration,
    /// Maximum concurrent HTTP requests.
    pub max_concurrency: usize,
    /// User agent string for crawl identification.
    pub user_agent: String,
    /// Respect robots.txt disallow directives.
    pub respect_robots: bool,
}

impl Default for CrawlPolicy {
    fn default() -> Self {
        Self {
            max_depth: 2,
            max_pages: 20,
            max_domains: 4,
            same_domain_only: false,
            per_domain_delay: Duration::from_millis(250),
            total_timeout: Duration::from_secs(30),
            max_concurrency: 4,
            user_agent: "FurlPayBot/0.1.0 (+https://furlpay.org/bot)".into(),
            respect_robots: true,
        }
    }
}

impl CrawlPolicy {
    /// Strict policy for single-domain exploration.
    pub fn single_domain(_domain: &str) -> Self {
        Self {
            max_depth: 2,
            max_pages: 15,
            max_domains: 1,
            same_domain_only: true,
            ..Default::default()
        }
    }

    /// Comprehensive policy for multi-source deep research.
    pub fn deep_research() -> Self {
        Self {
            max_depth: 3,
            max_pages: 40,
            max_domains: 8,
            same_domain_only: false,
            total_timeout: Duration::from_secs(60),
            ..Default::default()
        }
    }

    /// Check if a hop depth is within policy.
    pub fn is_depth_allowed(&self, depth: u32) -> bool {
        depth <= self.max_depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crawl_policy_defaults() {
        let policy = CrawlPolicy::default();
        assert_eq!(policy.max_depth, 2);
        assert!(policy.is_depth_allowed(2));
        assert!(!policy.is_depth_allowed(3));
    }
}
