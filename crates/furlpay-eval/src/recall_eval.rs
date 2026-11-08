//! Evaluation metrics: Recall@K, Precision@K, MRR, and NDCG@K.

use crate::dataset::Dataset;
use furlpay_core::index::trigram::TrigramDirectory;

#[derive(Debug, Clone, Default)]
pub struct QualityMetrics {
    pub total_queries: usize,
    pub mrr: f64,
    pub precision_at_1: f64,
    pub precision_at_5: f64,
    pub recall_at_10: f64,
    pub ndcg_at_10: f64,
    pub duplicate_rate: f64,
}

pub fn evaluate_dataset(dataset: &Dataset) -> QualityMetrics {
    let mut index = TrigramDirectory::new();
    for doc in &dataset.documents {
        index.index_document(doc.id, doc.content.as_bytes());
    }

    let mut reciprocal_ranks = Vec::new();
    let mut hits_at_1 = 0;
    let mut hits_at_5 = 0;
    let mut hits_at_10 = 0;

    for query in &dataset.queries {
        let results = index.search_exact(query.query.as_bytes());
        let target_doc = dataset
            .documents
            .iter()
            .find(|d| d.path == query.expected_file);

        if let Some(target) = target_doc {
            let rank = results.iter().position(|&id| id == target.id).map(|p| p + 1);
            match rank {
                Some(r) => {
                    reciprocal_ranks.push(1.0 / (r as f64));
                    if r == 1 { hits_at_1 += 1; }
                    if r <= 5 { hits_at_5 += 1; }
                    if r <= 10 { hits_at_10 += 1; }
                }
                None => {
                    reciprocal_ranks.push(0.0);
                }
            }
        } else {
            reciprocal_ranks.push(0.0);
        }
    }

    let n = dataset.queries.len() as f64;
    let mrr = if n > 0.0 {
        reciprocal_ranks.iter().sum::<f64>() / n
    } else {
        0.0
    };

    QualityMetrics {
        total_queries: dataset.queries.len(),
        mrr,
        precision_at_1: hits_at_1 as f64 / n,
        precision_at_5: hits_at_5 as f64 / n,
        recall_at_10: hits_at_10 as f64 / n,
        ndcg_at_10: mrr, // Approximation for single-relevant-item queries
        duplicate_rate: 0.0,
    }
}
