use criterion::{black_box, criterion_group, criterion_main, Criterion};
use furlpay_core::ranking::rrf::{reciprocal_rank_fusion, RankedResult, DEFAULT_K};
use furlpay_core::ranking::scorer::{RelevanceScorer, ScoringSignals};
use furlpay_core::index::schema::DocId;

fn bench_ranking(c: &mut Criterion) {
    let mut group = c.benchmark_group("furlpay_eval_ranking");

    let list1: Vec<RankedResult> = (0..32)
        .map(|i| RankedResult {
            doc_id: DocId(i),
            original_score: 1.0 - (i as f32 * 0.02),
            rank: i + 1,
        })
        .collect();
    let list2: Vec<RankedResult> = (16..48)
        .map(|i| RankedResult {
            doc_id: DocId(i),
            original_score: 1.0 - ((i - 16) as f32 * 0.02),
            rank: i - 15,
        })
        .collect();

    group.bench_function("rrf_k60_top32_fusion", |b| {
        b.iter(|| {
            reciprocal_rank_fusion(black_box(&[list1.clone(), list2.clone()]), DEFAULT_K)
        });
    });

    let scorer = RelevanceScorer::new();
    let signals: Vec<(DocId, ScoringSignals)> = (0..32)
        .map(|i| {
            (
                DocId(i),
                ScoringSignals {
                    trigram_coverage: 0.95,
                    bm25_score: 18.2,
                    ast_score: 1.0,
                    semantic_score: 0.85,
                    recency_boost: 1.0,
                    path_relevance: 1.0,
                },
            )
        })
        .collect();

    group.bench_function("multi_signal_scorer_top32", |b| {
        b.iter(|| {
            scorer.score_batch(black_box(signals.clone()))
        });
    });

    group.finish();
}

criterion_group!(benches, bench_ranking);
criterion_main!(benches);
