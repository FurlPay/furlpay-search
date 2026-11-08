//! Browser permission boundaries and security policies.

use std::time::Duration;
use crate::security::ssrf_protection::SsrfGuard;

/// Security boundary configuration for browser instances.
#[derive(Debug, Clone)]
pub struct BrowserPermissions {
    /// Allow JavaScript execution in web pages.
    pub allow_javascript: bool,
    /// Allow browser to navigate to new URLs.
    pub allow_navigation: bool,
    /// Allow file downloads.
    pub allow_downloads: bool,
    /// Block local file access (file:// scheme).
    pub block_local_files: bool,
    /// Enforce SSRF protection on every navigation.
    pub block_private_networks: bool,
    /// Require explicit user confirmation for form submissions (POST/PUT).
    pub require_form_authorization: bool,
    /// Maximum time allowed for page load and rendering.
    pub navigation_timeout: Duration,
    /// Maximum response body size in bytes.
    pub max_response_bytes: usize,
    /// Maximum number of concurrent tabs.
    pub max_tabs: usize,
}

impl Default for BrowserPermissions {
    fn default() -> Self {
        Self {
            allow_javascript: true,
            allow_navigation: true,
            allow_downloads: false,
            block_local_files: true,
            block_private_networks: true,
            require_form_authorization: true,
            navigation_timeout: Duration::from_secs(15),
            max_response_bytes: 10 * 1024 * 1024, // 10MB
            max_tabs: 5,
        }
    }
}

impl BrowserPermissions {
    /// Validate whether a target URL is permitted for navigation under current permissions.
    pub fn validate_target_url(&self, raw_url: &str) -> Result<(), String> {
        let trimmed = raw_url.trim();

        // 1. Parse with structured URL parser
        let parsed = url::Url::parse(trimmed)
            .map_err(|e| format!("Invalid URL syntax: {}", e))?;

        // 2. Allow only explicitly supported schemes (http/https)
        let scheme = parsed.scheme();
        if scheme != "http" && scheme != "https" {
            return Err(format!("Scheme '{}' is not permitted. Only HTTP and HTTPS are allowed", scheme));
        }

        // 3. Reject embedded userinfo/credentials (e.g. https://user:pass@evil.com)
        if !parsed.username().is_empty() || parsed.password().is_some() {
            return Err("Embedded URL credentials are prohibited".into());
        }

        // 4. Enforce SSRF protection and destination IP checks
        if self.block_private_networks {
            let guard = SsrfGuard::new();
            guard.validate_url(trimmed)
                .map_err(|e| format!("Network isolation violation: {}", e))?;
        }

        Ok(())
    }

    /// Check if a form action is allowed without user prompt.
    pub fn check_form_submission(&self, method: &str) -> Result<(), String> {
        let m = method.to_uppercase();
        if self.require_form_authorization && (m == "POST" || m == "PUT" || m == "DELETE" || m == "PATCH") {
            return Err("Form submission requires explicit user authorization".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocks_file_scheme() {
        let perms = BrowserPermissions::default();
        assert!(perms.validate_target_url("file:///etc/passwd").is_err());
        assert!(perms.validate_target_url("file://C:/Windows/win.ini").is_err());
    }

    #[test]
    fn test_blocks_localhost_ssrf() {
        let perms = BrowserPermissions::default();
        assert!(perms.validate_target_url("http://127.0.0.1:8080/admin").is_err());
        assert!(perms.validate_target_url("http://localhost:3000").is_err());
    }

    #[test]
    fn test_allows_public_url() {
        let perms = BrowserPermissions::default();
        assert!(perms.validate_target_url("https://www.rust-lang.org").is_ok());
    }

    #[test]
    fn test_form_submission_check() {
        let perms = BrowserPermissions::default();
        assert!(perms.check_form_submission("GET").is_ok());
        assert!(perms.check_form_submission("POST").is_err());
    }
}
