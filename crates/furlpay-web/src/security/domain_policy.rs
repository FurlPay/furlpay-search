//! Domain-level access policy enforcement.

use std::collections::HashSet;

/// Domain access policy for web retrieval.
#[derive(Debug, Clone)]
pub struct DomainPolicy {
    /// Explicitly blocked domains (never fetch).
    blocked_domains: HashSet<String>,
    /// Domains known to require authentication (warn but allow).
    auth_required_domains: HashSet<String>,
    /// Maximum requests per domain per minute.
    pub rate_limit_per_minute: u32,
}

impl Default for DomainPolicy {
    fn default() -> Self {
        let mut blocked = HashSet::new();
        // Cloud metadata endpoints
        blocked.insert("metadata.google.internal".into());
        blocked.insert("169.254.169.254".into());
        // Internal Kubernetes service discovery
        blocked.insert("kubernetes.default.svc".into());

        let mut auth_required = HashSet::new();
        auth_required.insert("github.com".into());
        auth_required.insert("gitlab.com".into());

        Self {
            blocked_domains: blocked,
            auth_required_domains: auth_required,
            rate_limit_per_minute: 30,
        }
    }
}

impl DomainPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a domain is explicitly blocked.
    pub fn is_blocked(&self, domain: &str) -> bool {
        let lower = domain.to_lowercase();
        self.blocked_domains.contains(&lower)
    }

    /// Check if a domain is known to require authentication.
    pub fn requires_auth(&self, domain: &str) -> bool {
        let lower = domain.to_lowercase();
        self.auth_required_domains.iter().any(|d| lower.ends_with(d))
    }

    /// Add a domain to the block list.
    pub fn block_domain(&mut self, domain: String) {
        self.blocked_domains.insert(domain.to_lowercase());
    }
}
