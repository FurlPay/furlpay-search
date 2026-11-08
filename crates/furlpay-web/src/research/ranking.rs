//! Reciprocal Rank Fusion (RRF k=60) and composite quality ranking:
//!
//! RRF(d) = Σ_i 1 / (k + rank_i(d))
//! Final(d) = θ1 RRF(d) + θ2 Relevance(d) + θ3 EvidenceQuality(d) + θ4 Freshness(d) + θ5 Coverage(d) - θ6 Duplication(d)

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingWeights {
    pub theta_rrf: f32,       // 0.35
    pub theta_relevance: f32, // 0.25
    pub theta_evidence: f32,  // 0.20
    pub theta_freshness: f32, // 0.10
    pub theta_coverage: f32,  // 0.10
}

impl Default for RankingWeights {
    fn default() -> Self {
        Self {
            theta_rrf: 0.35,
            theta_relevance: 0.25,
            theta_evidence: 0.20,
            theta_freshness: 0.10,
            theta_coverage: 0.10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateDocument {
    pub id: String,
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub provider_ranks: Vec<usize>, // 1-based ranks from multiple search pathways/engines
    pub semantic_relevance: f32,
    pub evidence_count: usize,
    pub is_primary_source: bool,
    pub published_year: Option<u32>,
}

pub struct ResearchRanker {
    weights: RankingWeights,
    rrf_k: usize,
}

impl Default for ResearchRanker {
    fn default() -> Self {
        Self::new()
    }
}

impl ResearchRanker {
    pub fn new() -> Self {
        Self {
            weights: RankingWeights::default(),
            rrf_k: 60, // Standard RRF parameter
        }
    }

    /// Compute RRF score: RRF(d) = Σ_i 1 / (k + rank_i(d))
    pub fn compute_rrf(&self, ranks: &[usize]) -> f32 {
        ranks.iter().map(|&r| 1.0 / ((self.rrf_k + r) as f32)).sum()
    }

    /// Rank candidates according to multi-signal composite score.
    pub fn rank_documents(&self, mut candidates: Vec<CandidateDocument>) -> Vec<(CandidateDocument, f32)> {
        let mut scored: Vec<(CandidateDocument, f32)> = candidates
            .into_iter()
            .map(|doc| {
                let rrf = self.compute_rrf(&doc.provider_ranks);
                let evidence_quality = if doc.evidence_count >= 3 { 1.0 } else { (doc.evidence_count as f32) / 3.0 };
                let freshness = match doc.published_year {
                    Some(y) if y >= 2026 => 1.0,
                    Some(y) if y >= 2024 => 0.8,
                    Some(_) => 0.5,
                    None => 0.6,
                };
                let coverage_bonus = if doc.is_primary_source { 1.0 } else { 0.5 };

                let final_score = (self.weights.theta_rrf * rrf * 10.0) // Scale RRF contribution
                    + (self.weights.theta_relevance * doc.semantic_relevance)
                    + (self.weights.theta_evidence * evidence_quality)
                    + (self.weights.theta_freshness * freshness)
                    + (self.weights.theta_coverage * coverage_bonus);

                (doc, (final_score * 1000.0).round() / 1000.0)
            })
            .collect();

        // Deterministic sort: score desc, then alphabetical URL asc
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.url.cmp(&b.0.url))
        });

        scored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_scoring() {
        let ranker = ResearchRanker::default();
        let rrf1 = ranker.compute_rrf(&[1, 2]);
        let rrf2 = ranker.compute_rrf(&[5, 10]);
        assert!(rrf1 > rrf2);
    }
}
