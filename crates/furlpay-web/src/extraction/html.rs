//! HTML content extraction — title, main text, headings, links, metadata.
//!
//! Uses the `scraper` crate for CSS-selector-based DOM traversal.
//! Strips scripts, styles, nav, footer, and boilerplate elements to extract
//! the main readable content.

use scraper::{Html, Selector};
use std::collections::HashSet;

/// Extracted content from an HTML page.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractedHtml {
    /// Page title from <title> or og:title.
    pub title: String,
    /// Meta description.
    pub description: String,
    /// Main readable text content.
    pub main_text: String,
    /// Headings extracted from h1-h6.
    pub headings: Vec<Heading>,
    /// Links found on the page.
    pub links: Vec<ExtractedLink>,
    /// Open Graph and meta tags.
    pub metadata: HtmlMetadata,
    /// Canonical URL if specified.
    pub canonical_url: Option<String>,
    /// Language from html[lang] or meta.
    pub language: Option<String>,
    /// Approximate word count of main text.
    pub word_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Heading {
    pub level: u8,
    pub text: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractedLink {
    pub url: String,
    pub text: String,
    pub rel: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HtmlMetadata {
    pub og_title: Option<String>,
    pub og_description: Option<String>,
    pub og_image: Option<String>,
    pub og_url: Option<String>,
    pub og_type: Option<String>,
    pub author: Option<String>,
    pub published_date: Option<String>,
    pub site_name: Option<String>,
}

/// HTML content extractor.
pub struct HtmlExtractor {
    max_text_chars: usize,
    max_links: usize,
}

impl HtmlExtractor {
    pub fn new() -> Self {
        Self {
            max_text_chars: 500_000,
            max_links: 200,
        }
    }

    pub fn with_limits(max_text_chars: usize, max_links: usize) -> Self {
        Self { max_text_chars, max_links }
    }

    /// Extract structured content from raw HTML.
    pub fn extract(&self, html_str: &str, base_url: Option<&str>) -> ExtractedHtml {
        let document = Html::parse_document(html_str);

        let title = self.extract_title(&document);
        let description = self.extract_meta_description(&document);
        let metadata = self.extract_metadata(&document);
        let canonical_url = self.extract_canonical(&document);
        let language = self.extract_language(&document);
        let headings = self.extract_headings(&document);
        let links = self.extract_links(&document, base_url);
        let main_text = self.extract_main_text(&document);
        let word_count = main_text.split_whitespace().count();

        ExtractedHtml {
            title,
            description,
            main_text,
            headings,
            links,
            metadata,
            canonical_url,
            language,
            word_count,
        }
    }

    fn extract_title(&self, doc: &Html) -> String {
        // Try <title> first
        if let Ok(sel) = Selector::parse("title") {
            if let Some(el) = doc.select(&sel).next() {
                let title = el.text().collect::<String>().trim().to_string();
                if !title.is_empty() {
                    return title;
                }
            }
        }
        // Fallback to og:title
        if let Ok(sel) = Selector::parse(r#"meta[property="og:title"]"#) {
            if let Some(el) = doc.select(&sel).next() {
                if let Some(content) = el.value().attr("content") {
                    return content.trim().to_string();
                }
            }
        }
        // Fallback to h1
        if let Ok(sel) = Selector::parse("h1") {
            if let Some(el) = doc.select(&sel).next() {
                return el.text().collect::<String>().trim().to_string();
            }
        }
        String::new()
    }

    fn extract_meta_description(&self, doc: &Html) -> String {
        for selector_str in &[
            r#"meta[name="description"]"#,
            r#"meta[property="og:description"]"#,
        ] {
            if let Ok(sel) = Selector::parse(selector_str) {
                if let Some(el) = doc.select(&sel).next() {
                    if let Some(content) = el.value().attr("content") {
                        let desc = content.trim().to_string();
                        if !desc.is_empty() {
                            return desc;
                        }
                    }
                }
            }
        }
        String::new()
    }

    fn extract_metadata(&self, doc: &Html) -> HtmlMetadata {
        let mut meta = HtmlMetadata::default();

        if let Ok(meta_sel) = Selector::parse("meta") {
            for el in doc.select(&meta_sel) {
                let content = el.value().attr("content").map(|c| c.trim().to_string());
                if let Some(content_str) = content {
                    if let Some(prop) = el.value().attr("property") {
                        match prop {
                            "og:title" => meta.og_title = Some(content_str.clone()),
                            "og:description" => meta.og_description = Some(content_str.clone()),
                            "og:image" => meta.og_image = Some(content_str.clone()),
                            "og:url" => meta.og_url = Some(content_str.clone()),
                            "og:type" => meta.og_type = Some(content_str.clone()),
                            "og:site_name" => meta.site_name = Some(content_str.clone()),
                            "article:published_time" => meta.published_date = Some(content_str.clone()),
                            _ => {}
                        }
                    }
                    if let Some(name) = el.value().attr("name") {
                        match name {
                            "author" => meta.author = Some(content_str.clone()),
                            "date" | "datePublished" => meta.published_date = Some(content_str.clone()),
                            _ => {}
                        }
                    }
                }
            }
        }

        meta
    }

    fn extract_canonical(&self, doc: &Html) -> Option<String> {
        if let Ok(sel) = Selector::parse(r#"link[rel="canonical"]"#) {
            if let Some(el) = doc.select(&sel).next() {
                return el.value().attr("href").map(|s| s.trim().to_string());
            }
        }
        None
    }

    fn extract_language(&self, doc: &Html) -> Option<String> {
        if let Ok(sel) = Selector::parse("html") {
            if let Some(el) = doc.select(&sel).next() {
                if let Some(lang) = el.value().attr("lang") {
                    return Some(lang.trim().to_string());
                }
            }
        }
        None
    }

    fn extract_headings(&self, doc: &Html) -> Vec<Heading> {
        let mut headings = Vec::new();
        let heading_tags: [(u8, &str); 6] = [
            (1, "h1"), (2, "h2"), (3, "h3"), (4, "h4"), (5, "h5"), (6, "h6")
        ];
        for (level, tag) in heading_tags {
            if let Ok(sel) = Selector::parse(tag) {
                for el in doc.select(&sel) {
                    let text = el.text().collect::<String>().trim().to_string();
                    if !text.is_empty() {
                        headings.push(Heading { level, text });
                    }
                }
            }
        }
        headings
    }

    fn extract_links(&self, doc: &Html, base_url: Option<&str>) -> Vec<ExtractedLink> {
        let mut links = Vec::new();
        let mut seen = HashSet::new();

        if let Ok(sel) = Selector::parse("a[href]") {
            for el in doc.select(&sel) {
                if links.len() >= self.max_links {
                    break;
                }
                if let Some(href) = el.value().attr("href") {
                    let href = href.trim();
                    if href.is_empty() || href.starts_with('#') || href.starts_with("javascript:") {
                        continue;
                    }

                    // Resolve relative URLs
                    let resolved = if let Some(base) = base_url {
                        if let Ok(base_url) = url::Url::parse(base) {
                            base_url.join(href).map(|u| u.to_string()).unwrap_or_else(|_| href.to_string())
                        } else {
                            href.to_string()
                        }
                    } else {
                        href.to_string()
                    };

                    if seen.contains(&resolved) {
                        continue;
                    }
                    seen.insert(resolved.clone());

                    let text = el.text().collect::<String>().trim().to_string();
                    let rel = el.value().attr("rel").map(String::from);

                    links.push(ExtractedLink { url: resolved, text, rel });
                }
            }
        }
        links
    }

    /// Extract the main readable text, stripping boilerplate elements.
    fn extract_main_text(&self, doc: &Html) -> String {
        // Elements to skip entirely
        let skip_tags: HashSet<&str> = [
            "script", "style", "noscript", "nav", "footer", "header",
            "aside", "iframe", "svg", "form", "button",
        ].into_iter().collect();

        let mut text_parts: Vec<String> = Vec::new();
        let mut total_chars = 0;

        // Try <main> or <article> first for focused content
        for container_selector in &["main", "article", r#"div[role="main"]"#, ".content", "#content"] {
            if let Ok(sel) = Selector::parse(container_selector) {
                if let Some(container) = doc.select(&sel).next() {
                    self.collect_text_recursive(container, &skip_tags, &mut text_parts, &mut total_chars);
                    if total_chars > 100 {
                        return self.clean_text(&text_parts.join("\n"));
                    }
                    // Reset if container was too sparse
                    text_parts.clear();
                    total_chars = 0;
                }
            }
        }

        // Fallback: extract from <body>
        if let Ok(sel) = Selector::parse("body") {
            if let Some(body) = doc.select(&sel).next() {
                self.collect_text_recursive(body, &skip_tags, &mut text_parts, &mut total_chars);
            }
        }

        self.clean_text(&text_parts.join("\n"))
    }

    fn collect_text_recursive(
        &self,
        element: scraper::ElementRef<'_>,
        skip_tags: &HashSet<&str>,
        parts: &mut Vec<String>,
        total_chars: &mut usize,
    ) {
        if *total_chars >= self.max_text_chars {
            return;
        }

        let tag_name = element.value().name();
        if skip_tags.contains(tag_name) {
            return;
        }

        // Block-level elements get line breaks
        let is_block = matches!(tag_name,
            "p" | "div" | "section" | "article" | "main" | "li" | "tr" |
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "blockquote" | "pre" | "br"
        );

        for child in element.children() {
            match child.value() {
                scraper::node::Node::Text(text) => {
                    let t = text.trim();
                    if !t.is_empty() && *total_chars < self.max_text_chars {
                        parts.push(t.to_string());
                        *total_chars += t.len();
                    }
                }
                scraper::node::Node::Element(_) => {
                    if let Some(child_el) = scraper::ElementRef::wrap(child) {
                        self.collect_text_recursive(child_el, skip_tags, parts, total_chars);
                    }
                }
                _ => {}
            }
        }

        if is_block && !parts.is_empty() {
            parts.push(String::new()); // blank line separator
        }
    }

    /// Clean extracted text: normalize whitespace, remove excess blank lines.
    fn clean_text(&self, text: &str) -> String {
        let mut result = String::new();
        let mut blank_count = 0;

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                blank_count += 1;
                if blank_count <= 1 {
                    result.push('\n');
                }
            } else {
                blank_count = 0;
                if !result.is_empty() && !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push_str(trimmed);
            }
        }

        let result = result.trim().to_string();
        if result.len() > self.max_text_chars {
            result[..self.max_text_chars].to_string()
        } else {
            result
        }
    }
}

impl Default for HtmlExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title() {
        let html = r#"<html><head><title>Test Page Title</title></head><body><p>Hello</p></body></html>"#;
        let extractor = HtmlExtractor::new();
        let result = extractor.extract(html, None);
        assert_eq!(result.title, "Test Page Title");
    }

    #[test]
    fn test_extract_main_text() {
        let html = r#"
            <html><body>
                <nav>Navigation</nav>
                <main><p>This is the main content.</p><p>Second paragraph.</p></main>
                <footer>Footer text</footer>
            </body></html>
        "#;
        let extractor = HtmlExtractor::new();
        let result = extractor.extract(html, None);
        assert!(result.main_text.contains("main content"));
        assert!(!result.main_text.contains("Navigation"));
        assert!(!result.main_text.contains("Footer text"));
    }

    #[test]
    fn test_extract_links() {
        let html = r#"
            <html><body>
                <a href="https://example.com">Example</a>
                <a href="/relative">Relative Link</a>
                <a href="javascript:void(0)">JS Link</a>
            </body></html>
        "#;
        let extractor = HtmlExtractor::new();
        let result = extractor.extract(html, Some("https://base.com"));
        assert_eq!(result.links.len(), 2); // JS link excluded
        assert_eq!(result.links[0].url, "https://example.com/");
        assert_eq!(result.links[1].url, "https://base.com/relative");
    }

    #[test]
    fn test_extract_metadata() {
        let html = r#"
            <html><head>
                <meta property="og:title" content="OG Title">
                <meta property="og:description" content="OG Desc">
                <meta name="author" content="Test Author">
                <link rel="canonical" href="https://example.com/canonical">
            </head><body></body></html>
        "#;
        let extractor = HtmlExtractor::new();
        let result = extractor.extract(html, None);
        assert_eq!(result.metadata.og_title.as_deref(), Some("OG Title"));
        assert_eq!(result.metadata.author.as_deref(), Some("Test Author"));
        assert_eq!(result.canonical_url.as_deref(), Some("https://example.com/canonical"));
    }
}
