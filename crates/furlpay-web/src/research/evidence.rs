//! Evidence and fact candidate extraction from unstructured text.

use serde::{Deserialize, Serialize};

/// A discrete factual claim or observation extracted from a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub statement: String,
    pub source_url: String,
    pub confidence: f32,
    pub citation_id: usize,
    pub has_numerical_claim: bool,
}

pub struct EvidenceExtractor;

impl EvidenceExtractor {
    /// Extract high-information evidence sentences from text content.
    pub fn extract_evidence(text: &str, source_url: &str, citation_id: usize) -> Vec<EvidenceItem> {
        let mut items = Vec::new();

        for sentence in text.split(&['.', '\n', ';'][..]) {
            let s = sentence.trim();
            // Filter trivial or navigation fragments
            if s.len() < 30 || s.len() > 300 {
                continue;
            }

            let lower = s.to_lowercase();
            // Skip UI boilerplate
            if lower.contains("cookie") || lower.contains("all rights reserved") || lower.contains("sign up") || lower.contains("privacy policy") {
                continue;
            }

            let has_numerical = s.chars().any(|c| c.is_ascii_digit());
            let word_count = s.split_whitespace().count();

            if word_count >= 6 {
                let mut confidence: f32 = 0.70;
                if has_numerical {
                    confidence += 0.15;
                }
                if lower.contains("announced") || lower.contains("released") || lower.contains("according to") || lower.contains("architecture") || lower.contains("designed") {
                    confidence += 0.10;
                }

                items.push(EvidenceItem {
                    statement: s.to_string(),
                    source_url: source_url.to_string(),
                    confidence: confidence.min(0.98),
                    citation_id,
                    has_numerical_claim: has_numerical,
                });
            }

            if items.len() >= 5 {
                break;
            }
        }

        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_evidence_numerical() {
        let text = "DeepSeek-V3 is a 671B parameter Mixture-of-Experts language model that activates only 37B parameters per token. Click here to sign up. All rights reserved.";
        let items = EvidenceExtractor::extract_evidence(text, "https://deepseek.com", 1);

        assert_eq!(items.len(), 1);
        assert!(items[0].has_numerical_claim);
        assert!(items[0].statement.contains("671B parameter"));
    }
}
