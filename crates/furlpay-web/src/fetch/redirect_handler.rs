//! Redirect validation and tracking.

use url::Url;
use crate::security::ssrf_protection::{SsrfGuard, SsrfError};

/// Tracks redirect chains and validates each hop.
pub struct RedirectHandler {
    ssrf_guard: SsrfGuard,
    max_redirects: u32,
}

impl RedirectHandler {
    pub fn new(max_redirects: u32) -> Self {
        Self {
            ssrf_guard: SsrfGuard::new(),
            max_redirects,
        }
    }

    /// Validate a redirect destination URL.
    pub fn validate_redirect(&self, from: &Url, to: &str) -> Result<Url, SsrfError> {
        // Resolve relative URLs against the source
        let target = from.join(to)
            .map_err(|e| SsrfError::InvalidUrl(format!("Invalid redirect URL '{}': {}", to, e)))?;

        // SSRF-validate the redirect target
        self.ssrf_guard.validate_url(target.as_str())?;

        Ok(target)
    }

    /// Check if more redirects are allowed.
    pub fn allows_more(&self, current_count: u32) -> bool {
        current_count < self.max_redirects
    }
}
