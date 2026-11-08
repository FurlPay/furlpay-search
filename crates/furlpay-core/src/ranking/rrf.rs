//! Reciprocal Rank Fusion (RRF) implementation with configurable k parameter.
//!
//! RRF combines multiple ranked lists into a single ranking using:
//! score(d) = Σ 1 / (k + rank_i(d))

use std::collections::HashMap;
use crate::index::schema::DocId;

/// Default RRF k parameter as specified in the master plan.
pub const DEFAULT_K: f32 = 60.0;

/// A ranked result from a single retrieval pipeline.
#[derive(Debug, Clone)]
pub struct RankedResult {
    pub doc_id: DocId,
    pub original_score: f32,
    pub rank: u32,
}

/// Result of RRF fusion with combined score.
#[derive(Debug, Clone)]
pub struct FusedResult {
    pub doc_id: DocId,
    pub rrf_score: f32,
    pub source_count: u32,
}

/// Perform Reciprocal Rank Fusion across multiple ranked lists.
pub fn reciprocal_rank_fusion(ranked_lists: &[Vec<RankedResult>], k: f32) -> Vec<FusedResult> {
    let mut scores: HashMap<DocId, (f32, u32)> = HashMap::new();

    for list in ranked_lists {
        for result in list {
            let rrf_contribution = 1.0 / (k + result.rank as f32);
            let entry = scores.entry(result.doc_id).or_insert((0.0, 0));
            entry.0 += rrf_contribution;
            entry.1 += 1;
        }
    }

    let mut fused: Vec<FusedResult> = scores
        .into_iter()
        .map(|(doc_id, (score, count))| FusedResult {
            doc_id,
            rrf_score: score,
            source_count: count,
        })
        .collect();

    fused.sort_by(|a, b| {
        b.rrf_score
            .partial_cmp(&a.rrf_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.doc_id.cmp(&b.doc_id))
    });

    fused
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_basic() {
        let list1 = vec![
            RankedResult { doc_id: DocId(1), original_score: 0.9, rank: 1 },
            RankedResult { doc_id: DocId(2), original_score: 0.8, rank: 2 },
        ];
        let list2 = vec![
            RankedResult { doc_id: DocId(2), original_score: 0.95, rank: 1 },
            RankedResult { doc_id: DocId(1), original_score: 0.85, rank: 2 },
        ];

        let fused = reciprocal_rank_fusion(&[list1, list2], DEFAULT_K);
        assert!(!fused.is_empty());
        // Both docs appear in both lists so should be ranked
        assert_eq!(fused.len(), 2);
    }

    #[test]
    fn test_rrf_empty() {
        let fused = reciprocal_rank_fusion(&[], DEFAULT_K);
        assert!(fused.is_empty());
    }
}
