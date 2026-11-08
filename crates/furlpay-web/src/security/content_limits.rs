//! Content size and resource limits for web retrieval.

/// Resource limits for a single web retrieval operation.
#[derive(Debug, Clone)]
pub struct ContentLimits {
    /// Maximum response body size in bytes (default: 50 MB).
    pub max_body_bytes: u64,
    /// Maximum time to wait for DNS resolution (ms).
    pub dns_timeout_ms: u64,
    /// Maximum time to wait for TCP connection (ms).
    pub connect_timeout_ms: u64,
    /// Maximum time for the entire request (ms).
    pub request_timeout_ms: u64,
    /// Maximum number of redirects to follow.
    pub max_redirects: u32,
    /// Maximum number of concurrent fetches.
    pub max_concurrent_fetches: u32,
    /// Maximum extracted text length (characters).
    pub max_extracted_text_chars: usize,
    /// Maximum number of links to extract from a page.
    pub max_extracted_links: usize,
}

impl Default for ContentLimits {
    fn default() -> Self {
        Self {
            max_body_bytes: 50 * 1024 * 1024,       // 50 MB
            dns_timeout_ms: 5_000,                    // 5 seconds
            connect_timeout_ms: 10_000,               // 10 seconds
            request_timeout_ms: 30_000,               // 30 seconds
            max_redirects: 10,
            max_concurrent_fetches: 8,
            max_extracted_text_chars: 500_000,         // ~500K chars
            max_extracted_links: 200,
        }
    }
}

impl ContentLimits {
    pub fn new() -> Self {
        Self::default()
    }

    /// Stricter limits for untrusted or unknown sources.
    pub fn strict() -> Self {
        Self {
            max_body_bytes: 10 * 1024 * 1024,        // 10 MB
            dns_timeout_ms: 3_000,
            connect_timeout_ms: 5_000,
            request_timeout_ms: 15_000,
            max_redirects: 5,
            max_concurrent_fetches: 4,
            max_extracted_text_chars: 100_000,
            max_extracted_links: 50,
        }
    }
}
