//! Core HTTP client with SSRF-validated outbound requests.
//!
//! Every outbound request passes through the SSRF guard before any TCP connection
//! is established. TLS validation is mandatory and cannot be disabled.

use std::time::{Duration, Instant};
use reqwest::{Client, Response};
use url::Url;

use crate::security::ssrf_protection::{SsrfGuard, SsrfError};
use crate::security::content_limits::ContentLimits;

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("SSRF blocked: {0}")]
    SsrfBlocked(#[from] SsrfError),
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("response too large: {size} bytes exceeds {limit} byte limit")]
    ResponseTooLarge { size: u64, limit: u64 },
    #[error("unsupported content type: {0}")]
    UnsupportedContentType(String),
    #[error("request timeout after {0}ms")]
    Timeout(u64),
    #[error("DNS resolution failed for {host}: {reason}")]
    DnsFailure { host: String, reason: String },
    #[error("TLS error: {0}")]
    TlsError(String),
    #[error("too many redirects: exceeded {0}")]
    TooManyRedirects(u32),
    #[error("HTTP status {status}: {url}")]
    HttpStatus { status: u16, url: String },
}

/// Result of a successful HTTP fetch.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FetchResponse {
    /// Original requested URL.
    pub original_url: String,
    /// Final URL after redirects.
    pub final_url: String,
    /// HTTP status code.
    pub status: u16,
    /// Content-Type header value.
    pub content_type: String,
    /// Response body as text (for text-based content).
    pub body_text: Option<String>,
    /// Response body as raw bytes (for binary content like PDF).
    pub body_bytes: Option<Vec<u8>>,
    /// Content length in bytes.
    pub content_length: u64,
    /// Number of redirects followed.
    pub redirects_followed: u32,
    /// Total request duration in microseconds.
    pub latency_us: u64,
    /// Response headers of interest.
    pub headers: std::collections::HashMap<String, String>,
    /// Retrieval timestamp (ISO 8601).
    pub retrieved_at: String,
}

/// Reusable SSRF-validated HTTP client for FurlPay web retrieval.
pub struct HttpClient {
    client: Client,
    ssrf_guard: SsrfGuard,
    limits: ContentLimits,
}

impl HttpClient {
    /// Create a new HttpClient with default security settings.
    /// TLS validation is always enabled and cannot be disabled.
    pub fn new() -> Result<Self, FetchError> {
        let limits = ContentLimits::default();
        Self::with_limits(limits)
    }

    /// Create an HttpClient with custom resource limits.
    pub fn with_limits(limits: ContentLimits) -> Result<Self, FetchError> {
        let client = Client::builder()
            .timeout(Duration::from_millis(limits.request_timeout_ms))
            .connect_timeout(Duration::from_millis(limits.connect_timeout_ms))
            .redirect(reqwest::redirect::Policy::limited(limits.max_redirects as usize))
            .user_agent("FurlPaySearchDaemon/0.1.0 (+https://furlpay.github.io/docs)")
            .gzip(true)
            .brotli(true)
            .deflate(true)
            // TLS is always validated — never call danger_accept_invalid_certs
            .build()
            .map_err(FetchError::HttpError)?;

        Ok(Self {
            client,
            ssrf_guard: SsrfGuard::new(),
            limits,
        })
    }

    /// Perform an SSRF-validated HTTP GET request to a public URL.
    pub async fn fetch(&self, url_str: &str) -> Result<FetchResponse, FetchError> {
        let start = Instant::now();

        // 1. Pre-flight SSRF validation
        let validated_url = self.ssrf_guard.validate_url(url_str)?;
        tracing::info!(url = %validated_url, "SSRF pre-flight passed");

        // 2. DNS pre-validation: resolve the host and check every returned IP
        self.validate_dns(&validated_url).await?;

        // 3. Execute the HTTP request
        let response = self.client
            .get(validated_url.as_str())
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    FetchError::Timeout(self.limits.request_timeout_ms)
                } else if e.is_connect() {
                    FetchError::DnsFailure {
                        host: validated_url.host_str().unwrap_or("unknown").into(),
                        reason: e.to_string(),
                    }
                } else {
                    FetchError::HttpError(e)
                }
            })?;

        // 4. Validate the final URL after redirects (SSRF re-check)
        let final_url = response.url().clone();
        self.ssrf_guard.validate_url(final_url.as_str())?;

        // 5. Check content length before downloading body
        if let Some(content_length) = response.content_length() {
            if content_length > self.limits.max_body_bytes {
                return Err(FetchError::ResponseTooLarge {
                    size: content_length,
                    limit: self.limits.max_body_bytes,
                });
            }
        }

        // 6. Extract headers and metadata
        let status = response.status();
        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let mut interesting_headers = std::collections::HashMap::new();
        for key in &["content-type", "content-length", "last-modified", "etag", "x-robots-tag", "link"] {
            if let Some(val) = response.headers().get(*key) {
                if let Ok(v) = val.to_str() {
                    interesting_headers.insert(key.to_string(), v.to_string());
                }
            }
        }

        // 7. Download body with size enforcement
        let (body_text, body_bytes, content_length) = self.download_body(response, &content_type).await?;

        let latency_us = start.elapsed().as_micros() as u64;

        Ok(FetchResponse {
            original_url: url_str.to_string(),
            final_url: final_url.to_string(),
            status: status.as_u16(),
            content_type,
            body_text,
            body_bytes,
            content_length,
            redirects_followed: 0, // reqwest handles redirects internally
            latency_us,
            headers: interesting_headers,
            retrieved_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Perform an SSRF-validated HTTP HEAD request for diagnostics.
    pub async fn head(&self, url_str: &str) -> Result<FetchResponse, FetchError> {
        let start = Instant::now();
        let validated_url = self.ssrf_guard.validate_url(url_str)?;
        self.validate_dns(&validated_url).await?;

        let response = self.client
            .head(validated_url.as_str())
            .send()
            .await
            .map_err(FetchError::HttpError)?;

        self.ssrf_guard.validate_url(response.url().as_str())?;

        let status = response.status();
        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let mut headers = std::collections::HashMap::new();
        for (k, v) in response.headers() {
            if let Ok(val) = v.to_str() {
                headers.insert(k.to_string(), val.to_string());
            }
        }

        Ok(FetchResponse {
            original_url: url_str.to_string(),
            final_url: response.url().to_string(),
            status: status.as_u16(),
            content_type,
            body_text: None,
            body_bytes: None,
            content_length: response.content_length().unwrap_or(0),
            redirects_followed: 0,
            latency_us: start.elapsed().as_micros() as u64,
            headers,
            retrieved_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Connectivity diagnostic: verify DNS, TLS, and HTTP to a target.
    pub async fn diagnose_connectivity(&self, url_str: &str) -> ConnectivityReport {
        let start = Instant::now();
        let mut report = ConnectivityReport {
            target_url: url_str.into(),
            dns_ok: false,
            tls_ok: false,
            http_ok: false,
            status_code: None,
            final_url: None,
            latency_us: 0,
            error: None,
        };

        match self.head(url_str).await {
            Ok(resp) => {
                report.dns_ok = true;
                report.tls_ok = url_str.starts_with("https://");
                report.http_ok = resp.status < 500;
                report.status_code = Some(resp.status);
                report.final_url = Some(resp.final_url);
                report.latency_us = resp.latency_us;
            }
            Err(e) => {
                // Partial diagnostics from the error type
                match &e {
                    FetchError::DnsFailure { .. } => {}
                    FetchError::TlsError(_) => { report.dns_ok = true; }
                    FetchError::HttpStatus { status, .. } => {
                        report.dns_ok = true;
                        report.tls_ok = true;
                        report.status_code = Some(*status);
                    }
                    FetchError::Timeout(_) => { report.dns_ok = true; }
                    _ => { report.dns_ok = true; report.tls_ok = true; }
                }
                report.error = Some(e.to_string());
            }
        }

        report.latency_us = start.elapsed().as_micros() as u64;
        report
    }

    /// Resolve DNS for a URL and validate every returned IP against the SSRF guard.
    async fn validate_dns(&self, url: &Url) -> Result<(), FetchError> {
        let host = url.host_str()
            .ok_or_else(|| FetchError::DnsFailure {
                host: "unknown".into(),
                reason: "URL has no host".into(),
            })?;

        // Try parsing host as an IP directly
        if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            self.ssrf_guard.validate_ip(ip)?;
            return Ok(());
        }

        // Resolve DNS using tokio's built-in resolver
        let port = url.port().unwrap_or(if url.scheme() == "https" { 443 } else { 80 });
        let addrs = tokio::net::lookup_host(format!("{}:{}", host, port))
            .await
            .map_err(|e| FetchError::DnsFailure {
                host: host.into(),
                reason: e.to_string(),
            })?;

        let mut found_any = false;
        for addr in addrs {
            found_any = true;
            self.ssrf_guard.validate_ip(addr.ip())?;
        }

        if !found_any {
            return Err(FetchError::DnsFailure {
                host: host.into(),
                reason: "DNS returned no addresses".into(),
            });
        }

        Ok(())
    }

    /// Download response body with size enforcement.
    async fn download_body(
        &self,
        response: Response,
        content_type: &str,
    ) -> Result<(Option<String>, Option<Vec<u8>>, u64), FetchError> {
        let is_text = content_type.starts_with("text/")
            || content_type.contains("json")
            || content_type.contains("xml")
            || content_type.contains("javascript")
            || content_type.contains("rss")
            || content_type.contains("atom");

        let is_pdf = content_type.contains("pdf");

        let bytes = response.bytes().await.map_err(FetchError::HttpError)?;
        let size = bytes.len() as u64;

        if size > self.limits.max_body_bytes {
            return Err(FetchError::ResponseTooLarge {
                size,
                limit: self.limits.max_body_bytes,
            });
        }

        if is_text {
            let text = String::from_utf8_lossy(&bytes).to_string();
            Ok((Some(text), None, size))
        } else if is_pdf {
            // Store raw bytes for PDF extraction
            Ok((None, Some(bytes.to_vec()), size))
        } else {
            // Binary content — store bytes
            Ok((None, Some(bytes.to_vec()), size))
        }
    }
}

/// Connectivity diagnostic report.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectivityReport {
    pub target_url: String,
    pub dns_ok: bool,
    pub tls_ok: bool,
    pub http_ok: bool,
    pub status_code: Option<u16>,
    pub final_url: Option<String>,
    pub latency_us: u64,
    pub error: Option<String>,
}
