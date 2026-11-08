//! Universal Pattern Intelligence Orchestrator.

use super::candidate::PatternCandidate;
use super::confidence::ConfidenceScorer;
use super::context::ContextAnalyzer;
use super::fallback::FallbackGenerator;
use super::normalizer::QueryNormalizer;
use super::registry::PatternRegistry;
use super::tokenizer::QueryTokenizer;

/// The primary Universal Pattern Recognition Engine.
pub struct PatternEngine {
    registry: PatternRegistry,
}

impl Default for PatternEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternEngine {
    /// Create a new pattern engine with default registered recognizers.
    pub fn new() -> Self {
        Self {
            registry: PatternRegistry::new(),
        }
    }

    /// Access the underlying registry to register, enable, or disable recognizers.
    pub fn registry_mut(&mut self) -> &mut PatternRegistry {
        &mut self.registry
    }

    /// Analyze a search query and return all recognized and ranked pattern candidates.
    pub fn analyze(&self, query: &str) -> Vec<PatternCandidate> {
        let normalized = QueryNormalizer::normalize(query);
        let candidate_tokens = QueryTokenizer::extract_candidate_tokens(&normalized);

        let mut recognized = Vec::new();

        // 1. Try full normalized string first
        let full_matches = self.registry.recognize(&normalized);
        recognized.extend(full_matches);

        // 2. If full string did not match or if query contains sub-tokens, check extracted tokens
        for token in candidate_tokens {
            if token != normalized {
                let token_matches = self.registry.recognize(&token);
                for mut tm in token_matches {
                    // Check for duplicate candidate
                    if !recognized.iter().any(|c| c.recognizer_id == tm.recognizer_id && c.raw_value == tm.raw_value) {
                        tm.confidence *= 0.90; // slight sub-token penalty
                        recognized.push(tm);
                    }
                }
            }
        }

        // 3. Calibrate confidence and rank
        for candidate in &mut recognized {
            let boost = ContextAnalyzer::context_boost(query, candidate.category);
            candidate.confidence = ConfidenceScorer::score(candidate, boost);
        }
        ConfidenceScorer::rank_candidates(&mut recognized);

        // 4. If no specialized candidates were recognized, append universal fallback
        if recognized.is_empty() {
            recognized.push(FallbackGenerator::generate_fallback(query));
        }

        recognized
    }

    /// Return the single highest-confidence candidate for the query.
    pub fn best_match(&self, query: &str) -> PatternCandidate {
        let results = self.analyze(query);
        results.into_iter().next().unwrap_or_else(|| FallbackGenerator::generate_fallback(query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recognition::ValidationStatus;

    #[test]
    fn test_engine_tron_real() {
        let engine = PatternEngine::new();
        let matches = engine.analyze("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].recognizer_id, "blockchain.tron.base58check");
        assert_eq!(matches[0].validation_status, ValidationStatus::CryptographicallyVerified);
    }

    #[test]
    fn test_engine_iban_real() {
        let engine = PatternEngine::new();
        let matches = engine.analyze("DE89 3704 0044 0532 0130 00");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].recognizer_id, "financial.iban");
        assert_eq!(matches[0].validation_status, ValidationStatus::CryptographicallyVerified);
    }

    #[test]
    fn test_engine_arbitrary_unfamiliar_query() {
        let engine = PatternEngine::new();
        let query = "what is the quantum entanglement entropy of a black hole horizon?";
        let matches = engine.analyze(query);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].recognizer_id, "general.universal_fallback");
        assert_eq!(matches[0].suggested_route, "GlobalWebDiscovery");
    }
}
