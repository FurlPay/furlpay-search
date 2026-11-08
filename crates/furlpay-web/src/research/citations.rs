//! Structured citation tracking for verified claims.

use serde::{Deserialize, Serialize};

/// A formal academic/journalistic citation linked to an online source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Citation {
    /// Citation index (e.g. 1, 2, 3).
    pub id: usize,
    /// Target web URL.
    pub url: String,
    /// Page or document title.
    pub title: String,
    /// Publisher or domain name.
    pub publisher: String,
    /// Relevant quote or snippet supporting the finding.
    pub supporting_snippet: String,
    /// Timestamp when source was accessed.
    pub accessed_at: String,
    /// Published date if detectable.
    pub published_date: Option<String>,
}

impl Citation {
    pub fn new(
        id: usize,
        url: impl Into<String>,
        title: impl Into<String>,
        publisher: impl Into<String>,
        snippet: impl Into<String>,
    ) -> Self {
        Self {
            id,
            url: url.into(),
            title: title.into(),
            publisher: publisher.into(),
            supporting_snippet: snippet.into(),
            accessed_at: chrono::Utc::now().to_rfc3339(),
            published_date: None,
        }
    }

    /// Format as Markdown citation link: `[^1]: [Title](url) — Publisher`
    pub fn to_markdown_footnote(&self) -> String {
        format!(
            "[^{}]: [{}]({}) — *{}*",
            self.id, self.title, self.url, self.publisher
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citation_formatting() {
        let cit = Citation::new(
            1,
            "https://docs.rs/tokio",
            "Tokio Documentation",
            "docs.rs",
            "Asynchronous runtime for the Rust programming language",
        );
        let md = cit.to_markdown_footnote();
        assert!(md.contains("[^1]: [Tokio Documentation](https://docs.rs/tokio)"));
    }
}
