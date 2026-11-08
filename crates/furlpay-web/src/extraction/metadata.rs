//! Unified metadata extraction from fetched content.

/// Unified metadata for any retrieved document.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentMetadata {
    pub original_url: String,
    pub final_url: String,
    pub title: String,
    pub description: String,
    pub content_type: String,
    pub document_format: DocumentFormat,
    pub language: Option<String>,
    pub author: Option<String>,
    pub published_date: Option<String>,
    pub canonical_url: Option<String>,
    pub word_count: usize,
    pub byte_size: u64,
    pub retrieved_at: String,
    pub latency_us: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DocumentFormat {
    Html,
    Pdf,
    PlainText,
    Json,
    Xml,
    Rss,
    Atom,
    Markdown,
    Unknown(String),
}

/// Detect document format from content-type header and URL.
pub fn detect_format(content_type: &str, url: &str) -> DocumentFormat {
    let ct = content_type.to_lowercase();

    if ct.contains("text/html") || ct.contains("application/xhtml") {
        DocumentFormat::Html
    } else if ct.contains("application/pdf") {
        DocumentFormat::Pdf
    } else if ct.contains("text/plain") {
        // Check URL extension for markdown
        if url.ends_with(".md") || url.ends_with(".markdown") {
            DocumentFormat::Markdown
        } else {
            DocumentFormat::PlainText
        }
    } else if ct.contains("application/json") || ct.contains("text/json") {
        DocumentFormat::Json
    } else if ct.contains("application/rss+xml") {
        DocumentFormat::Rss
    } else if ct.contains("application/atom+xml") {
        DocumentFormat::Atom
    } else if ct.contains("text/xml") || ct.contains("application/xml") {
        DocumentFormat::Xml
    } else if ct.contains("text/markdown") {
        DocumentFormat::Markdown
    } else {
        DocumentFormat::Unknown(content_type.to_string())
    }
}
