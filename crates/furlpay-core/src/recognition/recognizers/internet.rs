//! Internet networking recognizers (URLs, Domains, IPv4, IPv6, Email addresses).

use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use super::PatternRecognizer;
use crate::recognition::candidate::{PatternCandidate, PatternCategory, ValidationStatus};

pub struct InternetRecognizer;

impl InternetRecognizer {
    pub fn new() -> Self {
        Self
    }

    /// Check if string looks like an email address.
    fn is_email(s: &str) -> bool {
        if s.len() < 5 || s.len() > 254 {
            return false;
        }
        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        let (local, domain) = (parts[0], parts[1]);
        if local.is_empty() || domain.is_empty() {
            return false;
        }
        // Local part characters
        if !local.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' || c == '+') {
            return false;
        }
        if local.starts_with('.') || local.ends_with('.') || local.contains("..") {
            return false;
        }
        // Domain part must contain at least one dot and valid labels
        if !domain.contains('.') {
            return false;
        }
        let domain_labels: Vec<&str> = domain.split('.').collect();
        if domain_labels.iter().any(|label| label.is_empty() || label.starts_with('-') || label.ends_with('-')) {
            return false;
        }
        let tld = domain_labels.last().unwrap();
        if tld.len() < 2 || !tld.chars().all(|c| c.is_ascii_alphabetic()) {
            return false;
        }
        true
    }

    /// Check if string looks like a standard fully qualified domain name.
    fn is_domain(s: &str) -> bool {
        if s.len() < 4 || s.len() > 253 || s.contains('/') || s.contains(':') || s.contains('@') {
            return false;
        }
        let labels: Vec<&str> = s.split('.').collect();
        if labels.len() < 2 {
            return false;
        }
        for label in &labels {
            if label.is_empty() || label.len() > 63 {
                return false;
            }
            if label.starts_with('-') || label.ends_with('-') {
                return false;
            }
            if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return false;
            }
        }
        let tld = labels.last().unwrap();
        if tld.len() < 2 || !tld.chars().all(|c| c.is_ascii_alphabetic()) {
            return false;
        }
        // Known common TLDs or 2-letter ccTLDs
        const COMMON_TLDS: &[&str] = &[
            "com", "org", "net", "edu", "gov", "mil", "int", "io", "ai", "co", "me", "dev",
            "app", "xyz", "info", "biz", "us", "uk", "de", "fr", "jp", "cn", "in", "ca", "eu",
            "tech", "online", "cloud", "finance", "crypto", "network", "rs"
        ];
        COMMON_TLDS.contains(&tld.to_lowercase().as_str()) || tld.len() == 2
    }
}

impl PatternRecognizer for InternetRecognizer {
    fn id(&self) -> &str {
        "internet.universal"
    }

    fn category(&self) -> PatternCategory {
        PatternCategory::Internet
    }

    fn recognize(&self, input: &str) -> Vec<PatternCandidate> {
        let trimmed = input.trim();
        let mut candidates = Vec::new();

        // 1. URL Recognition (with explicit scheme)
        if trimmed.starts_with("http://")
            || trimmed.starts_with("https://")
            || trimmed.starts_with("ftp://")
            || trimmed.starts_with("ws://")
            || trimmed.starts_with("wss://")
        {
            let scheme_end = trimmed.find("://").unwrap();
            let scheme = &trimmed[..scheme_end];
            let after_scheme = &trimmed[scheme_end + 3..];
            let host_part = after_scheme.split(&['/', '?', '#'][..]).next().unwrap_or("");

            let mut meta = HashMap::new();
            meta.insert("scheme".into(), scheme.to_lowercase());
            meta.insert("host".into(), host_part.to_string());
            meta.insert("is_secure".into(), (scheme == "https" || scheme == "wss").to_string());

            candidates.push(PatternCandidate {
                recognizer_id: "internet.url".into(),
                category: PatternCategory::Internet,
                display_name: format!("Uniform Resource Locator ({})", scheme.to_uppercase()),
                raw_value: trimmed.to_string(),
                normalized_value: trimmed.to_string(),
                validation_status: ValidationStatus::FormatValidated,
                confidence: 0.99,
                metadata: meta,
                evidence: format!("Valid URL with scheme '{}' and host '{}'", scheme, host_part),
                suggested_route: "WebRetrieval".into(),
            });
            return candidates;
        }

        // 2. IPv4 Address Recognition
        if let Ok(ipv4) = Ipv4Addr::from_str(trimmed) {
            let mut meta = HashMap::new();
            meta.insert("standard".into(), "IPv4".into());
            meta.insert("is_loopback".into(), ipv4.is_loopback().to_string());
            meta.insert("is_private".into(), ipv4.is_private().to_string());
            meta.insert("is_multicast".into(), ipv4.is_multicast().to_string());
            meta.insert("is_broadcast".into(), ipv4.is_broadcast().to_string());

            candidates.push(PatternCandidate {
                recognizer_id: "internet.ipv4".into(),
                category: PatternCategory::Internet,
                display_name: format!("IPv4 Address ({})", ipv4),
                raw_value: trimmed.to_string(),
                normalized_value: ipv4.to_string(),
                validation_status: ValidationStatus::FormatValidated,
                confidence: 0.99,
                metadata: meta,
                evidence: format!("Valid dotted-quad IPv4 address (Private: {}, Loopback: {})", ipv4.is_private(), ipv4.is_loopback()),
                suggested_route: "NetworkDiagnostics".into(),
            });
            return candidates;
        }

        // 3. IPv6 Address Recognition
        if trimmed.contains(':') && !trimmed.contains("://") {
            if let Ok(ipv6) = Ipv6Addr::from_str(trimmed) {
                let mut meta = HashMap::new();
                meta.insert("standard".into(), "IPv6".into());
                meta.insert("is_loopback".into(), ipv6.is_loopback().to_string());
                meta.insert("is_multicast".into(), ipv6.is_multicast().to_string());

                candidates.push(PatternCandidate {
                    recognizer_id: "internet.ipv6".into(),
                    category: PatternCategory::Internet,
                    display_name: format!("IPv6 Address ({})", ipv6),
                    raw_value: trimmed.to_string(),
                    normalized_value: ipv6.to_string(),
                    validation_status: ValidationStatus::FormatValidated,
                    confidence: 0.99,
                    metadata: meta,
                    evidence: format!("Valid IPv6 standard representation (Loopback: {})", ipv6.is_loopback()),
                    suggested_route: "NetworkDiagnostics".into(),
                });
                return candidates;
            }
        }

        // 4. Email Address Recognition
        if Self::is_email(trimmed) {
            let parts: Vec<&str> = trimmed.split('@').collect();
            let mut meta = HashMap::new();
            meta.insert("domain".into(), parts[1].to_lowercase());
            meta.insert("mailbox".into(), parts[0].to_string());

            candidates.push(PatternCandidate {
                recognizer_id: "internet.email".into(),
                category: PatternCategory::Internet,
                display_name: "Email Address".into(),
                raw_value: trimmed.to_string(),
                normalized_value: format!("{}@{}", parts[0], parts[1].to_lowercase()),
                validation_status: ValidationStatus::FormatValidated,
                confidence: 0.96,
                metadata: meta,
                evidence: format!("RFC 5322 email syntax with domain '{}'", parts[1]),
                suggested_route: "Communication".into(),
            });
            return candidates;
        }

        // 5. Domain Name Recognition
        if Self::is_domain(trimmed) {
            let mut meta = HashMap::new();
            meta.insert("host".into(), trimmed.to_lowercase());

            candidates.push(PatternCandidate {
                recognizer_id: "internet.domain".into(),
                category: PatternCategory::Internet,
                display_name: format!("Internet Domain ({})", trimmed.to_lowercase()),
                raw_value: trimmed.to_string(),
                normalized_value: trimmed.to_lowercase(),
                validation_status: ValidationStatus::FormatValidated,
                confidence: 0.92,
                metadata: meta,
                evidence: format!("Syntactically valid FQDN ending with acknowledged TLD"),
                suggested_route: "WebRetrieval".into(),
            });
            return candidates;
        }

        candidates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url() {
        let rec = InternetRecognizer::new();
        let res = rec.recognize("https://docs.rs/tokio/latest/tokio/");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].recognizer_id, "internet.url");
        assert_eq!(res[0].metadata.get("scheme").unwrap(), "https");
    }

    #[test]
    fn test_ipv4() {
        let rec = InternetRecognizer::new();
        let res = rec.recognize("192.168.1.100");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].recognizer_id, "internet.ipv4");
        assert_eq!(res[0].metadata.get("is_private").unwrap(), "true");
    }

    #[test]
    fn test_ipv6() {
        let rec = InternetRecognizer::new();
        let res = rec.recognize("2001:0db8:85a3:0000:0000:8a2e:0370:7334");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].recognizer_id, "internet.ipv6");
    }

    #[test]
    fn test_email() {
        let rec = InternetRecognizer::new();
        let res = rec.recognize("developer@furlpay.io");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].recognizer_id, "internet.email");
        assert_eq!(res[0].metadata.get("domain").unwrap(), "furlpay.io");
    }

    #[test]
    fn test_domain() {
        let rec = InternetRecognizer::new();
        let res = rec.recognize("crates.io");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].recognizer_id, "internet.domain");
    }
}
