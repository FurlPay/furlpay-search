//! GIEA Link context scoring and candidate prioritization:
//!
//! L(u) = λ R(p) + μ A(u) + ν C(u) + ξ X(u)
//!
//! - R(p): parent page relevance to the research topic
//! - A(u): anchor text relevance to query and subtopics
//! - C(u): surrounding link-context relevance
//! - X(u): link destination and novelty features

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkScoreWeights {
    pub lambda_parent: f32, // 0.35
    pub mu_anchor: f32,     // 0.25
    pub nu_context: f32,    // 0.20
    pub xi_novelty: f32,    // 0.20
}

impl Default for LinkScoreWeights {
    fn default() -> Self {
        Self {
            lambda_parent: 0.35,
            mu_anchor: 0.25,
            nu_context: 0.20,
            xi_novelty: 0.20,
        }
    }
}

pub struct LinkScorer {
    weights: LinkScoreWeights,
}

impl Default for LinkScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkScorer {
    pub fn new() -> Self {
        Self {
            weights: LinkScoreWeights::default(),
        }
    }

    pub fn with_weights(weights: LinkScoreWeights) -> Self {
        Self { weights }
    }

    /// Calculate candidate link score L(u) = λ R(p) + μ A(u) + ν C(u) + ξ X(u).
    pub fn score_link(
        &self,
        parent_relevance: f32,
        anchor_text: &str,
        link_context: &str,
        destination_url: &str,
        query_terms: &[&str],
        known_domains: &[String],
    ) -> f32 {
        // 1. Anchor text relevance A(u)
        let anchor_lower = anchor_text.to_lowercase();
        let anchor_matches = query_terms.iter().filter(|&&t| anchor_lower.contains(t)).count();
        let a_u = if query_terms.is_empty() {
            0.5
        } else {
            (anchor_matches as f32 / query_terms.len() as f32).min(1.0)
        };

        // 2. Surrounding context relevance C(u)
        let context_lower = link_context.to_lowercase();
        let context_matches = query_terms.iter().filter(|&&t| context_lower.contains(t)).count();
        let c_u = if query_terms.is_empty() {
            0.5
        } else {
            (context_matches as f32 / query_terms.len() as f32).min(1.0)
        };

        // 3. Link destination and novelty features X(u)
        let dest_lower = destination_url.to_lowercase();
        let mut x_u: f32 = 0.5;

        // Path quality bonus
        if dest_lower.contains("/doc") || dest_lower.contains("/api") || dest_lower.contains("/research") || dest_lower.contains("/paper") || dest_lower.contains("/whitepaper") {
            x_u += 0.30;
        }

        // Domain novelty bonus
        if let Ok(u) = url::Url::parse(destination_url) {
            if let Some(host) = u.host_str() {
                if !known_domains.iter().any(|d| d == host) {
                    x_u += 0.20; // Discovered link to an independent domain
                }
            }
        }

        // Demote low-value administrative endpoints
        if dest_lower.contains("login") || dest_lower.contains("signup") || dest_lower.contains("privacy") || dest_lower.contains("terms") || dest_lower.contains("cookie") {
            x_u -= 0.40;
        }
        let x_u = x_u.clamp(0.0, 1.0);

        let final_score = (self.weights.lambda_parent * parent_relevance.clamp(0.0, 1.0))
            + (self.weights.mu_anchor * a_u)
            + (self.weights.nu_context * c_u)
            + (self.weights.xi_novelty * x_u);

        final_score.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_scoring_technical_doc() {
        let scorer = LinkScorer::default();
        let score = scorer.score_link(
            0.85,
            "Rust Async Runtime Architecture",
            "Read the detailed Rust async runtime architecture documentation here.",
            "https://tokio.rs/docs/architecture",
            &["rust", "async", "architecture"],
            &["example.com".into()],
        );
        assert!(score >= 0.75);
    }

    #[test]
    fn test_link_scoring_demotes_terms() {
        let scorer = LinkScorer::default();
        let score = scorer.score_link(
            0.85,
            "Terms of Service",
            "Please read our terms of service and privacy policy.",
            "https://example.com/terms",
            &["rust", "async", "architecture"],
            &["example.com".into()],
        );
        assert!(score < 0.45);
    }
}
