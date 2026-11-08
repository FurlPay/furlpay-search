//! URL normalization, canonicalization, and content deduplication.

use std::collections::HashSet;
use xxhash_rust::xxh3::xxh3_64;
use crate::providers::search_provider::SearchHit;

/// URL normalizer and deduplication manager.
#[derive(Debug, Clone, Default)]
pub struct Deduplicator {
    seen_urls: HashSet<String>,
    seen_content_hashes: HashSet<u64>,
}

impl Deduplicator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Normalize a URL for identity comparison:
    /// - lowercase scheme and host
    /// - remove trailing slash
    /// - remove tracking query params (utm_*, ref, fbclid, etc.)
    /// - strip default ports (:80, :443)
    /// - strip fragment (#...)
    pub fn normalize_url(raw_url: &str) -> String {
        let trimmed = raw_url.trim();
        let without_fragment = match trimmed.split_once('#') {
            Some((base, _)) => base,
            None => trimmed,
        };

        if let Ok(mut parsed) = url::Url::parse(without_fragment) {
            // Strip standard tracking query params
            let tracking_prefixes = ["utm_", "fbclid", "gclid", "ref", "source", "mc_cid", "mc_eid"];
            let filtered_query: Vec<(String, String)> = parsed
                .query_pairs()
                .filter(|(k, _)| !tracking_prefixes.iter().any(|&p| k.starts_with(p)))
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            if filtered_query.is_empty() {
                parsed.set_query(None);
            } else {
                let mut serializer = url::form_urlencoded::Serializer::new(String::new());
                for (k, v) in filtered_query {
                    serializer.append_pair(&k, &v);
                }
                parsed.set_query(Some(&serializer.finish()));
            }

            let mut out = parsed.to_string();
            if out.ends_with('/') && out.matches('/').count() > 3 {
                out.pop();
            }
            out
        } else {
            without_fragment.trim_end_matches('/').to_string()
        }
    }

    /// Check if a URL has already been processed. If not, record it.
    pub fn is_url_seen(&mut self, raw_url: &str) -> bool {
        let normalized = Self::normalize_url(raw_url);
        !self.seen_urls.insert(normalized)
    }

    /// Compute 64-bit content hash using XXH3 for rapid exact-duplicate detection.
    pub fn content_hash(text: &str) -> u64 {
        let normalized_text = text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        xxh3_64(normalized_text.as_bytes())
    }

    /// Check if content has already been seen. If not, record its hash.
    pub fn is_content_seen(&mut self, text: &str) -> bool {
        let hash = Self::content_hash(text);
        !self.seen_content_hashes.insert(hash)
    }

    /// Deduplicate a list of search hits by canonical URL and content snippet similarity.
    pub fn deduplicate_hits(&mut self, hits: Vec<SearchHit>) -> Vec<SearchHit> {
        let mut unique = Vec::new();
        for hit in hits {
            if !self.is_url_seen(&hit.url) && !self.is_content_seen(&hit.snippet) {
                unique.push(hit);
            }
        }
        unique
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_tracking_params() {
        let raw = "https://example.com/page?utm_source=twitter&utm_medium=social&id=42#section";
        let norm = Deduplicator::normalize_url(raw);
        assert_eq!(norm, "https://example.com/page?id=42");
    }

    #[test]
    fn test_normalize_trailing_slash() {
        let raw = "https://example.com/docs/guide/";
        let norm = Deduplicator::normalize_url(raw);
        assert_eq!(norm, "https://example.com/docs/guide");
    }

    #[test]
    fn test_duplicate_url_detection() {
        let mut dedup = Deduplicator::new();
        assert!(!dedup.is_url_seen("https://example.com/page?utm_source=x"));
        assert!(dedup.is_url_seen("https://example.com/page"));
    }

    #[test]
    fn test_content_hashing() {
        let h1 = Deduplicator::content_hash("Hello   World  \nfrom Rust!");
        let h2 = Deduplicator::content_hash("hello world from rust!");
        assert_eq!(h1, h2);
    }
}
