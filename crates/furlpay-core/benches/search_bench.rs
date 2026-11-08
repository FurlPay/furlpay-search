use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use furlpay_core::index::trigram::TrigramDirectory;
use furlpay_core::index::schema::DocId;
use furlpay_core::routing::classifier::QueryRouter;
use furlpay_core::ranking::rrf::{reciprocal_rank_fusion, RankedResult, DEFAULT_K};
use furlpay_core::ranking::scorer::{RelevanceScorer, ScoringSignals};
use furlpay_core::ast::symbol_graph::SymbolGraph;
use furlpay_core::ast::treesitter::SymbolKind;

fn create_test_corpus(size: usize) -> Vec<String> {
    (0..size)
        .map(|i| {
            format!(
                "export async function authMiddleware_{i}(req: Request, res: Response) {{ \
                 const token = req.headers['authorization']; \
                 if (!verifyToken_{i}(token)) return res.status(401); \
                 return next(); }}"
            )
        })
        .collect()
}

fn build_trigram_index(corpus: &[String]) -> TrigramDirectory {
    let mut index = TrigramDirectory::new();
    for (i, doc) in corpus.iter().enumerate() {
        index.index_document(DocId(i as u32), doc.as_bytes());
    }
    index
}

fn benchmark_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("furlpay_core_microbenchmarks");

    // 1. Trigram search scaling across corpus sizes
    for corpus_size in [100, 1_000, 10_000] {
        let corpus = create_test_corpus(corpus_size);
        let index = build_trigram_index(&corpus);

        group.bench_with_input(
            BenchmarkId::new("trigram_search", corpus_size),
            &index,
            |b, idx| {
                b.iter(|| {
                    idx.search_exact(black_box(b"authMiddleware"))
                });
            },
        );
    }

    // 2. Query classification benchmark
    let queries = [
        "authMiddleware",
        "fn verify_jwt_token",
        "auth.*middleware",
        "who calls processPayment",
        "how to handle token expiration in auth middleware",
    ];
    group.bench_function("query_classification", |b| {
        b.iter(|| {
            for q in queries.iter() {
                QueryRouter::classify(black_box(q));
            }
        });
    });

    // 3. RRF Rank Fusion benchmark
    let list1: Vec<RankedResult> = (0..50)
        .map(|i| RankedResult {
            doc_id: DocId(i),
            original_score: 1.0 - (i as f32 * 0.01),
            rank: i + 1,
        })
        .collect();
    let list2: Vec<RankedResult> = (25..75)
        .map(|i| RankedResult {
            doc_id: DocId(i),
            original_score: 1.0 - ((i - 25) as f32 * 0.01),
            rank: i - 24,
        })
        .collect();

    group.bench_function("reciprocal_rank_fusion", |b| {
        b.iter(|| {
            reciprocal_rank_fusion(black_box(&[list1.clone(), list2.clone()]), DEFAULT_K)
        });
    });

    // 4. Relevance scorer benchmark
    let scorer = RelevanceScorer::new();
    let signals: Vec<(DocId, ScoringSignals)> = (0..100)
        .map(|i| {
            (
                DocId(i),
                ScoringSignals {
                    trigram_coverage: 0.8,
                    bm25_score: 12.5,
                    ast_score: 0.9,
                    semantic_score: 0.7,
                    recency_boost: 0.95,
                    path_relevance: 1.0,
                },
            )
        })
        .collect();

    group.bench_function("relevance_scorer_batch_100", |b| {
        b.iter(|| {
            scorer.score_batch(black_box(signals.clone()))
        });
    });

    // 5. AST Symbol Graph traversal
    let mut graph = SymbolGraph::new();
    let root = graph.add_symbol("main".into(), SymbolKind::Function, DocId(0), (0, 100), (0, 10));
    for i in 1..100 {
        let child = graph.add_symbol(
            format!("helper_{i}"),
            SymbolKind::Function,
            DocId(i as u32),
            (0, 50),
            (0, 5),
        );
        graph.add_call_edge(root, child);
    }

    group.bench_function("symbol_graph_callees_lookup", |b| {
        b.iter(|| {
            graph.callees_of(black_box(root))
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_search);
criterion_main!(benches);
