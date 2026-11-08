//! Crawl frontier — priority queue of URLs awaiting exploration.

use std::collections::{HashSet, VecDeque};
use crate::discovery::deduplication::Deduplicator;

/// A URL scheduled for crawling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrawlItem {
    pub url: String,
    pub depth: u32,
    pub parent_url: Option<String>,
    pub priority: u32,
    pub anchor_text: Option<String>,
}

/// Frontier queue maintaining unvisited URLs.
#[derive(Debug, Default)]
pub struct CrawlFrontier {
    queue: VecDeque<CrawlItem>,
    enqueued_urls: HashSet<String>,
    visited_urls: HashSet<String>,
}

impl CrawlFrontier {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a seed URL to the frontier.
    pub fn push_seed(&mut self, url: &str) {
        let norm = Deduplicator::normalize_url(url);
        if self.enqueued_urls.insert(norm.clone()) {
            self.queue.push_back(CrawlItem {
                url: norm,
                depth: 0,
                parent_url: None,
                priority: 100,
                anchor_text: None,
            });
        }
    }

    /// Push a newly discovered link to the frontier.
    pub fn push_discovered(
        &mut self,
        url: &str,
        depth: u32,
        parent: &str,
        anchor_text: &str,
    ) -> bool {
        let norm = Deduplicator::normalize_url(url);
        if self.visited_urls.contains(&norm) || !self.enqueued_urls.insert(norm.clone()) {
            return false;
        }

        // Calculate heuristic priority
        let mut priority: u32 = 50;
        let lower = norm.to_lowercase();

        // Boost documentation, technical content, articles
        if lower.contains("/doc") || lower.contains("/guide") || lower.contains("/api") || lower.contains("/wiki") || lower.contains("/research") {
            priority += 30;
        }

        // Demote login, signup, privacy, terms
        if lower.contains("login") || lower.contains("signup") || lower.contains("privacy") || lower.contains("terms") || lower.contains("cookie") {
            priority = priority.saturating_sub(40);
        }

        let item = CrawlItem {
            url: norm,
            depth,
            parent_url: Some(parent.to_string()),
            priority,
            anchor_text: if anchor_text.is_empty() { None } else { Some(anchor_text.to_string()) },
        };

        // Insert sorted by priority
        let insert_idx = self.queue.iter().position(|q| q.priority < priority).unwrap_or(self.queue.len());
        self.queue.insert(insert_idx, item);
        true
    }

    /// Pop the next highest priority item from the frontier.
    pub fn pop_next(&mut self) -> Option<CrawlItem> {
        let item = self.queue.pop_front()?;
        self.visited_urls.insert(item.url.clone());
        Some(item)
    }

    /// Number of URLs remaining in the queue.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Whether the frontier is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Number of visited URLs.
    pub fn visited_count(&self) -> usize {
        self.visited_urls.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frontier_prioritization() {
        let mut frontier = CrawlFrontier::new();
        frontier.push_seed("https://example.com");

        frontier.push_discovered("https://example.com/terms", 1, "https://example.com", "Terms");
        frontier.push_discovered("https://example.com/docs/api", 1, "https://example.com", "API Docs");

        // Seed is popped first (priority 100)
        let seed = frontier.pop_next().unwrap();
        assert_eq!(seed.url, Deduplicator::normalize_url("https://example.com"));

        // Docs should pop before terms
        let first = frontier.pop_next().unwrap();
        assert_eq!(first.url, "https://example.com/docs/api");

        let second = frontier.pop_next().unwrap();
        assert_eq!(second.url, "https://example.com/terms");
    }
}
