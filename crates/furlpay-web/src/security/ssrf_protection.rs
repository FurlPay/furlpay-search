//! Server-Side Request Forgery (SSRF) protection.
//!
//! Validates all outbound request targets before any TCP connection is established.
//! Blocks access to:
//! - Localhost and loopback addresses (127.0.0.0/8, ::1)
//! - Private network ranges (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16)
//! - Link-local addresses (169.254.0.0/16, fe80::/10)
//! - Multicast ranges (224.0.0.0/4, ff00::/8)
//! - Cloud metadata endpoints (169.254.169.254)
//! - Reserved and unspecified addresses
//! - Non-HTTP/HTTPS schemes

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use url::Url;

#[derive(Debug, Clone, thiserror::Error)]
pub enum SsrfError {
    #[error("blocked scheme: only http and https are permitted, got '{0}'")]
    BlockedScheme(String),
    #[error("blocked host: '{0}' resolves to a private/reserved IP address")]
    BlockedPrivateIp(String),
    #[error("blocked host: localhost and loopback addresses are not permitted")]
    BlockedLoopback,
    #[error("blocked host: link-local address detected")]
    BlockedLinkLocal,
    #[error("blocked host: multicast address detected")]
    BlockedMulticast,
    #[error("blocked host: cloud metadata endpoint detected ({0})")]
    BlockedMetadata(String),
    #[error("blocked host: unspecified/reserved address")]
    BlockedReserved,
    #[error("blocked port: port {0} is not permitted")]
    BlockedPort(u16),
    #[error("URL parse error: {0}")]
    InvalidUrl(String),
    #[error("DNS resolution failed for host '{0}': {1}")]
    DnsResolutionFailed(String, String),
    #[error("redirect to blocked destination: {0}")]
    BlockedRedirect(String),
    #[error("download size exceeds limit: {0} bytes > {1} bytes")]
    ContentTooLarge(u64, u64),
}

/// SSRF guard that validates URLs and resolved IPs before network connections.
#[derive(Debug, Clone)]
pub struct SsrfGuard {
    /// Permitted schemes
    pub allowed_schemes: Vec<String>,
    /// Permitted ports (empty = allow standard ports only)
    pub allowed_ports: Vec<u16>,
    /// Maximum download size in bytes (default: 50 MB)
    pub max_download_bytes: u64,
    /// Maximum number of redirects to follow
    pub max_redirects: u32,
    /// Additional blocked domains
    pub blocked_domains: Vec<String>,
}

impl Default for SsrfGuard {
    fn default() -> Self {
        Self {
            allowed_schemes: vec!["http".into(), "https".into()],
            allowed_ports: vec![80, 443, 8080, 8443],
            max_download_bytes: 50 * 1024 * 1024, // 50 MB
            max_redirects: 10,
            blocked_domains: vec![
                "metadata.google.internal".into(),
                "metadata.google.com".into(),
            ],
        }
    }
}

impl SsrfGuard {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate a URL before making any network request.
    pub fn validate_url(&self, url_str: &str) -> Result<Url, SsrfError> {
        let url = Url::parse(url_str)
            .map_err(|e| SsrfError::InvalidUrl(format!("{}: {}", url_str, e)))?;

        // 1. Scheme validation
        let scheme = url.scheme().to_lowercase();
        if !self.allowed_schemes.contains(&scheme) {
            return Err(SsrfError::BlockedScheme(scheme));
        }

        // 2. Host validation
        let host = url.host_str()
            .ok_or_else(|| SsrfError::InvalidUrl("URL has no host".into()))?;

        let host_lower = host.to_lowercase();

        // Block localhost and loopback hostnames
        if host_lower == "localhost" || host_lower.ends_with(".localhost") || host_lower.ends_with(".local") {
            return Err(SsrfError::BlockedLoopback);
        }

        // Block known cloud metadata endpoints
        for blocked in &self.blocked_domains {
            if host_lower == blocked.to_lowercase() {
                return Err(SsrfError::BlockedMetadata(host.into()));
            }
        }

        // If host is an IP address literal, validate it immediately against private/reserved ranges
        if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            self.validate_ip(ip)?;
        }

        // 3. Port validation
        let port = url.port().unwrap_or(if scheme == "https" { 443 } else { 80 });
        if !self.allowed_ports.is_empty() && !self.allowed_ports.contains(&port) {
            return Err(SsrfError::BlockedPort(port));
        }

        Ok(url)
    }

    /// Validate a resolved IP address before establishing a TCP connection.
    pub fn validate_ip(&self, ip: IpAddr) -> Result<(), SsrfError> {
        match ip {
            IpAddr::V4(ipv4) => self.validate_ipv4(ipv4),
            IpAddr::V6(ipv6) => self.validate_ipv6(ipv6),
        }
    }

    fn validate_ipv4(&self, ip: Ipv4Addr) -> Result<(), SsrfError> {
        // Loopback: 127.0.0.0/8
        if ip.is_loopback() {
            return Err(SsrfError::BlockedLoopback);
        }

        // Unspecified: 0.0.0.0
        if ip.is_unspecified() {
            return Err(SsrfError::BlockedReserved);
        }

        // Private ranges
        if ip.is_private() {
            return Err(SsrfError::BlockedPrivateIp(ip.to_string()));
        }

        // Link-local: 169.254.0.0/16
        if ip.is_link_local() {
            return Err(SsrfError::BlockedLinkLocal);
        }

        // Cloud metadata: 169.254.169.254
        if ip == Ipv4Addr::new(169, 254, 169, 254) {
            return Err(SsrfError::BlockedMetadata(ip.to_string()));
        }

        // Multicast: 224.0.0.0/4
        if ip.is_multicast() {
            return Err(SsrfError::BlockedMulticast);
        }

        // Broadcast: 255.255.255.255
        if ip.is_broadcast() {
            return Err(SsrfError::BlockedReserved);
        }

        // Documentation ranges: 192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24
        let octets = ip.octets();
        if (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
            || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
            || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
        {
            return Err(SsrfError::BlockedReserved);
        }

        Ok(())
    }

    fn validate_ipv6(&self, ip: Ipv6Addr) -> Result<(), SsrfError> {
        // Loopback: ::1
        if ip.is_loopback() {
            return Err(SsrfError::BlockedLoopback);
        }

        // Unspecified: ::
        if ip.is_unspecified() {
            return Err(SsrfError::BlockedReserved);
        }

        // Multicast: ff00::/8
        if ip.is_multicast() {
            return Err(SsrfError::BlockedMulticast);
        }

        // Check if this is an IPv4-mapped IPv6 address (::ffff:x.x.x.x)
        if let Some(ipv4) = ip.to_ipv4_mapped() {
            return self.validate_ipv4(ipv4);
        }

        // Link-local: fe80::/10
        let segments = ip.segments();
        if segments[0] & 0xffc0 == 0xfe80 {
            return Err(SsrfError::BlockedLinkLocal);
        }

        // Unique local: fc00::/7
        if segments[0] & 0xfe00 == 0xfc00 {
            return Err(SsrfError::BlockedPrivateIp(ip.to_string()));
        }

        Ok(())
    }

    /// Validate a redirect target URL.
    pub fn validate_redirect(&self, redirect_url: &str) -> Result<Url, SsrfError> {
        self.validate_url(redirect_url)
            .map_err(|e| SsrfError::BlockedRedirect(format!("{}: {}", redirect_url, e)))
    }

    /// Check download size against the configured limit.
    pub fn validate_content_length(&self, length: u64) -> Result<(), SsrfError> {
        if length > self.max_download_bytes {
            return Err(SsrfError::ContentTooLarge(length, self.max_download_bytes));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allows_public_https() {
        let guard = SsrfGuard::new();
        assert!(guard.validate_url("https://www.rust-lang.org/").is_ok());
        assert!(guard.validate_url("https://docs.rs/reqwest").is_ok());
    }

    #[test]
    fn test_blocks_localhost() {
        let guard = SsrfGuard::new();
        assert!(guard.validate_ip(IpAddr::V4(Ipv4Addr::LOCALHOST)).is_err());
        assert!(guard.validate_ip(IpAddr::V6(Ipv6Addr::LOCALHOST)).is_err());
    }

    #[test]
    fn test_blocks_private_ranges() {
        let guard = SsrfGuard::new();
        assert!(guard.validate_ip("10.0.0.1".parse().unwrap()).is_err());
        assert!(guard.validate_ip("172.16.0.1".parse().unwrap()).is_err());
        assert!(guard.validate_ip("192.168.1.1".parse().unwrap()).is_err());
    }

    #[test]
    fn test_blocks_metadata() {
        let guard = SsrfGuard::new();
        assert!(guard.validate_ip("169.254.169.254".parse().unwrap()).is_err());
        assert!(guard.validate_url("http://metadata.google.internal/").is_err());
    }

    #[test]
    fn test_blocks_non_http_schemes() {
        let guard = SsrfGuard::new();
        assert!(guard.validate_url("ftp://example.com/file").is_err());
        assert!(guard.validate_url("file:///etc/passwd").is_err());
        assert!(guard.validate_url("gopher://example.com").is_err());
    }

    #[test]
    fn test_blocks_link_local() {
        let guard = SsrfGuard::new();
        assert!(guard.validate_ip("169.254.1.1".parse().unwrap()).is_err());
    }

    #[test]
    fn test_allows_public_ip() {
        let guard = SsrfGuard::new();
        assert!(guard.validate_ip("8.8.8.8".parse().unwrap()).is_ok());
        assert!(guard.validate_ip("1.1.1.1".parse().unwrap()).is_ok());
    }
}
