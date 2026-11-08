//! Pattern recognizers registry.

use std::collections::HashSet;
use super::candidate::{PatternCandidate, PatternCategory};
use super::recognizers::PatternRecognizer;
use super::recognizers::blockchain::BlockchainRecognizer;
use super::recognizers::crypto::CryptoRecognizer;
use super::recognizers::financial::FinancialRecognizer;
use super::recognizers::internet::InternetRecognizer;
use super::recognizers::software::SoftwareRecognizer;
use super::recognizers::standards::StandardsRecognizer;

/// Registry holding active pattern recognizers.
pub struct PatternRegistry {
    recognizers: Vec<Box<dyn PatternRecognizer>>,
    disabled_ids: HashSet<String>,
}

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternRegistry {
    /// Create a registry initialized with all universal standard recognizers.
    pub fn new() -> Self {
        let mut registry = Self {
            recognizers: Vec::new(),
            disabled_ids: HashSet::new(),
        };

        registry.register(Box::new(BlockchainRecognizer::new()));
        registry.register(Box::new(FinancialRecognizer::new()));
        registry.register(Box::new(InternetRecognizer::new()));
        registry.register(Box::new(CryptoRecognizer::new()));
        registry.register(Box::new(SoftwareRecognizer::new()));
        registry.register(Box::new(StandardsRecognizer::new()));

        registry
    }

    /// Register a custom recognizer.
    pub fn register(&mut self, recognizer: Box<dyn PatternRecognizer>) {
        self.recognizers.push(recognizer);
    }

    /// Disable a specific recognizer by ID.
    pub fn disable(&mut self, id: &str) {
        self.disabled_ids.insert(id.to_string());
    }

    /// Re-enable a disabled recognizer.
    pub fn enable(&mut self, id: &str) {
        self.disabled_ids.remove(id);
    }

    /// Execute all enabled recognizers on the given input string.
    pub fn recognize(&self, input: &str) -> Vec<PatternCandidate> {
        let mut all_candidates = Vec::new();

        for recognizer in &self.recognizers {
            if !self.disabled_ids.contains(recognizer.id()) {
                let candidates = recognizer.recognize(input);
                all_candidates.extend(candidates);
            }
        }

        all_candidates
    }

    /// Execute recognizers filtered by a specific domain category.
    pub fn recognize_category(&self, input: &str, category: PatternCategory) -> Vec<PatternCandidate> {
        let mut candidates = Vec::new();

        for recognizer in &self.recognizers {
            if recognizer.category() == category && !self.disabled_ids.contains(recognizer.id()) {
                candidates.extend(recognizer.recognize(input));
            }
        }

        candidates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_all_domains() {
        let registry = PatternRegistry::new();

        // Blockchain
        let b = registry.recognize("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t");
        assert!(!b.is_empty());
        assert_eq!(b[0].category, PatternCategory::Blockchain);

        // Financial
        let f = registry.recognize("DE89370400440532013000");
        assert!(!f.is_empty());
        assert_eq!(f[0].category, PatternCategory::Financial);

        // Internet
        let i = registry.recognize("https://furlpay.io");
        assert!(!i.is_empty());
        assert_eq!(i[0].category, PatternCategory::Internet);

        // Crypto
        let c = registry.recognize("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert!(!c.is_empty());
        assert_eq!(c[0].category, PatternCategory::Cryptography);

        // Software
        let s = registry.recognize("GET /api/v1/search");
        assert!(!s.is_empty());
        assert_eq!(s[0].category, PatternCategory::Software);

        // Standards
        let std = registry.recognize("978-0-306-40615-7");
        assert!(!std.is_empty());
        assert_eq!(std[0].category, PatternCategory::Documents);
    }
}
