//! Domain authority, source reliability, and content freshness assessment.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceScore {
    pub domain: String,
    pub authority_score: f32,
    pub is_official_doc: bool,
    pub tier: SourceTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceTier {
    Tier1Primary,    // .gov, .edu, official docs, peer-reviewed (arXiv, RFCs)
    Tier2Reputable,  // Major tech media, GitHub verified orgs, Wikipedia
    Tier3Standard,   // General blogs, forums, independent sites
    Tier4Untrusted,  // Low-reputation, spammy, unverified
}

pub struct SourceAnalyzer;

impl SourceAnalyzer {
    /// Evaluate the source quality and authority of a URL.
    pub fn analyze_source(url_str: &str) -> SourceScore {
        let domain = url::Url::parse(url_str)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
            .unwrap_or_default();

        let mut score: f32 = 0.50;
        let mut is_official = false;
        let mut tier = SourceTier::Tier3Standard;

        // Primary institutional TLDs
        if domain.ends_with(".gov") || domain.ends_with(".mil") {
            score += 0.40;
            tier = SourceTier::Tier1Primary;
        } else if domain.ends_with(".edu") || domain.ends_with(".ac.uk") {
            score += 0.35;
            tier = SourceTier::Tier1Primary;
        } else if domain.contains("arxiv.org") || domain.contains("doi.org") || domain.contains("nature.com") || domain.contains("science.org") {
            score += 0.40;
            tier = SourceTier::Tier1Primary;
        } else if domain.contains("docs.rs") || domain.contains("github.com") || domain.contains("developer.mozilla.org") || domain.contains("rust-lang.org") || domain.contains("python.org") {
            score += 0.35;
            is_official = true;
            tier = SourceTier::Tier1Primary;
        } else if domain.contains("wikipedia.org") || domain.contains("reuters.com") || domain.contains("bloomberg.com") || domain.contains("apnews.com") {
            score += 0.25;
            tier = SourceTier::Tier2Reputable;
        }

        // HTTPS bonus
        if url_str.starts_with("https://") {
            score += 0.05;
        }

        SourceScore {
            domain,
            authority_score: score.min(0.99),
            is_official_doc: is_official,
            tier,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier1_institutional() {
        let score = SourceAnalyzer::analyze_source("https://arxiv.org/abs/2412.19437");
        assert_eq!(score.tier, SourceTier::Tier1Primary);
        assert!(score.authority_score >= 0.90);
    }

    #[test]
    fn test_official_docs() {
        let score = SourceAnalyzer::analyze_source("https://docs.rs/tokio/latest/tokio/");
        assert!(score.is_official_doc);
        assert_eq!(score.tier, SourceTier::Tier1Primary);
    }
}
