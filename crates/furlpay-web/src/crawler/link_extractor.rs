//! HTML link extraction, relative URL resolution, and anchor text analysis.

use url::Url;

/// Extracted hyperlink with its target URL and anchor text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredLink {
    pub url: String,
    pub anchor_text: String,
    pub is_internal: bool,
}

pub struct LinkExtractor;

impl LinkExtractor {
    /// Extract all valid HTTP/HTTPS links from HTML text, resolving relative links against `base_url`.
    pub fn extract_links(html: &str, base_url: &str) -> Vec<DiscoveredLink> {
        let base = match Url::parse(base_url) {
            Ok(u) => u,
            Err(_) => return Vec::new(),
        };

        let base_host = base.host_str().unwrap_or("").to_lowercase();
        let document = scraper::Html::parse_document(html);
        let link_selector = match scraper::Selector::parse("a[href]") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let mut results = Vec::new();
        let ignored_extensions = [
            ".jpg", ".jpeg", ".png", ".gif", ".svg", ".webp", ".ico",
            ".mp3", ".mp4", ".wav", ".avi", ".mov", ".zip", ".tar", ".gz",
            ".exe", ".dmg", ".pkg", ".iso", ".css", ".js",
        ];

        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                let trimmed_href = href.trim();
                if trimmed_href.is_empty() || trimmed_href.starts_with('#') || trimmed_href.starts_with("javascript:") || trimmed_href.starts_with("mailto:") || trimmed_href.starts_with("tel:") {
                    continue;
                }

                if let Ok(resolved) = base.join(trimmed_href) {
                    let scheme = resolved.scheme();
                    if scheme != "http" && scheme != "https" {
                        continue;
                    }

                    let url_str = resolved.to_string();
                    let lower_url = url_str.to_lowercase();

                    // Skip static media files
                    if ignored_extensions.iter().any(|ext| lower_url.ends_with(ext)) {
                        continue;
                    }

                    let resolved_host = resolved.host_str().unwrap_or("").to_lowercase();
                    let is_internal = resolved_host == base_host || resolved_host.ends_with(&format!(".{}", base_host));

                    let anchor_text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();

                    if !results.iter().any(|r: &DiscoveredLink| r.url == url_str) {
                        results.push(DiscoveredLink {
                            url: url_str,
                            anchor_text,
                            is_internal,
                        });
                    }
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_and_resolve_links() {
        let html = r##"
            <html><body>
                <a href="/docs/guide">Documentation Guide</a>
                <a href="https://external.com/api">External API</a>
                <a href="#section">Fragment</a>
                <a href="mailto:test@example.com">Email</a>
                <a href="/image.png">Image</a>
            </body></html>
        "##;
        let links = LinkExtractor::extract_links(html, "https://example.com/home");

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].url, "https://example.com/docs/guide");
        assert_eq!(links[0].anchor_text, "Documentation Guide");
        assert!(links[0].is_internal);

        assert_eq!(links[1].url, "https://external.com/api");
        assert_eq!(links[1].anchor_text, "External API");
        assert!(!links[1].is_internal);
    }
}
