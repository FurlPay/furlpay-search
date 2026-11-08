//! Multi-signal relevance scorer combining trigram, BM25, AST, and other signals.

use crate::index::schema::DocId;

/// Individual scoring signals for a document.
#[derive(Debug, Clone, Default)]
pub struct ScoringSignals {
    pub trigram_coverage: f32,
    pub bm25_score: f32,
    pub ast_score: f32,
    pub semantic_score: f32,
    pub recency_boost: f32,
    pub path_relevance: f32,
}

/// Weights for combining scoring signals.
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    pub trigram: f32,
    pub bm25: f32,
    pub ast: f32,
    pub semantic: f32,
    pub recency: f32,
    pub path: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            trigram: 0.30,
            bm25: 0.25,
            ast: 0.20,
            semantic: 0.10,
            recency: 0.05,
            path: 0.10,
        }
    }
}

/// Scored document with all component signals preserved.
#[derive(Debug, Clone)]
pub struct ScoredDocument {
    pub doc_id: DocId,
    pub signals: ScoringSignals,
    pub combined_score: f32,
}

/// Multi-signal relevance scorer.
pub struct RelevanceScorer {
    weights: ScoringWeights,
}

impl RelevanceScorer {
    pub fn new() -> Self {
        Self {
            weights: ScoringWeights::default(),
        }
    }

    pub fn with_weights(weights: ScoringWeights) -> Self {
        Self { weights }
    }

    /// Compute the combined relevance score for a document.
    pub fn score(&self, doc_id: DocId, signals: ScoringSignals) -> ScoredDocument {
        let combined = signals.trigram_coverage * self.weights.trigram
            + signals.bm25_score * self.weights.bm25
            + signals.ast_score * self.weights.ast
            + signals.semantic_score * self.weights.semantic
            + signals.recency_boost * self.weights.recency
            + signals.path_relevance * self.weights.path;

        ScoredDocument {
            doc_id,
            signals,
            combined_score: combined,
        }
    }

    /// Score and rank a batch of documents.
    pub fn score_batch(&self, documents: Vec<(DocId, ScoringSignals)>) -> Vec<ScoredDocument> {
        let mut scored: Vec<ScoredDocument> = documents
            .into_iter()
            .map(|(doc_id, signals)| self.score(doc_id, signals))
            .collect();
        scored.sort_by(|a, b| {
            b.combined_score
                .partial_cmp(&a.combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored
    }
}

impl Default for RelevanceScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoring() {
        let scorer = RelevanceScorer::new();
        let signals = ScoringSignals {
            trigram_coverage: 1.0,
            bm25_score: 0.8,
            ast_score: 0.5,
            ..Default::default()
        };
        let result = scorer.score(DocId(1), signals);
        assert!(result.combined_score > 0.0);
    }

    #[test]
    fn test_batch_scoring_order() {
        let scorer = RelevanceScorer::new();
        let docs = vec![
            (DocId(1), ScoringSignals { trigram_coverage: 0.5, bm25_score: 0.3, ..Default::default() }),
            (DocId(2), ScoringSignals { trigram_coverage: 1.0, bm25_score: 0.9, ..Default::default() }),
        ];
        let ranked = scorer.score_batch(docs);
        assert_eq!(ranked[0].doc_id, DocId(2));
    }
}
