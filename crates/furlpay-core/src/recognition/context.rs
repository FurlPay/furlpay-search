//! Ambient query context and intent clue analyzer.

use super::candidate::PatternCategory;

/// Context analyzer that provides soft confidence adjustments based on query phrasing.
pub struct ContextAnalyzer;

impl ContextAnalyzer {
    /// Calculate soft confidence adjustment [0.0, 1.0] for a given category based on query context.
    pub fn context_boost(query: &str, category: PatternCategory) -> f32 {
        let lower = query.to_lowercase();
        let terms: Vec<&str> = lower.split_whitespace().collect();

        match category {
            PatternCategory::Blockchain => {
                if terms.iter().any(|&t| t == "wallet" || t == "address" || t == "crypto" || t == "chain" || t == "tx") {
                    0.20
                } else {
                    0.0
                }
            }
            PatternCategory::Financial => {
                if terms.iter().any(|&t| t == "iban" || t == "swift" || t == "bic" || t == "bank" || t == "account" || t == "payment") {
                    0.20
                } else {
                    0.0
                }
            }
            PatternCategory::Internet => {
                if terms.iter().any(|&t| t == "url" || t == "link" || t == "site" || t == "web" || t == "ip" || t == "host") {
                    0.20
                } else {
                    0.0
                }
            }
            PatternCategory::Cryptography => {
                if terms.iter().any(|&t| t == "hash" || t == "digest" || t == "sha" || t == "uuid" || t == "checksum") {
                    0.20
                } else {
                    0.0
                }
            }
            PatternCategory::Software => {
                if terms.iter().any(|&t| t == "version" || t == "commit" || t == "git" || t == "api" || t == "route" || t == "endpoint") {
                    0.20
                } else {
                    0.0
                }
            }
            PatternCategory::Documents => {
                if terms.iter().any(|&t| t == "isbn" || t == "book" || t == "doi" || t == "paper" || t == "article") {
                    0.20
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }
}
