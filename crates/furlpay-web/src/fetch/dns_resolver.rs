//! DNS resolution with SSRF-validated IP checking.

use std::net::IpAddr;
use crate::security::ssrf_protection::{SsrfGuard, SsrfError};

/// SSRF-aware DNS resolver that validates every resolved IP address.
pub struct SafeDnsResolver {
    ssrf_guard: SsrfGuard,
}

impl SafeDnsResolver {
    pub fn new() -> Self {
        Self {
            ssrf_guard: SsrfGuard::new(),
        }
    }

    /// Resolve a hostname and validate all returned IPs against SSRF rules.
    pub async fn resolve_validated(&self, host: &str, port: u16) -> Result<Vec<std::net::SocketAddr>, SsrfError> {
        // Direct IP address
        if let Ok(ip) = host.parse::<IpAddr>() {
            self.ssrf_guard.validate_ip(ip)?;
            return Ok(vec![std::net::SocketAddr::new(ip, port)]);
        }

        let addrs: Vec<std::net::SocketAddr> = tokio::net::lookup_host(format!("{}:{}", host, port))
            .await
            .map_err(|e| SsrfError::DnsResolutionFailed(host.into(), e.to_string()))?
            .collect();

        if addrs.is_empty() {
            return Err(SsrfError::DnsResolutionFailed(host.into(), "No addresses returned".into()));
        }

        // Validate every returned IP
        for addr in &addrs {
            self.ssrf_guard.validate_ip(addr.ip())?;
        }

        Ok(addrs)
    }
}
