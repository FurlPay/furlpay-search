//! Cloudflare Workers KV cache adapter (Milestone 2 stub).

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub url: String,
    pub markdown_content: String,
    pub cached_at: u64,
    pub ttl_seconds: u64,
}

pub struct EdgeVault;

impl EdgeVault {
    pub async fn lookup(_url: &str) -> Option<CacheEntry> {
        None
    }
}
