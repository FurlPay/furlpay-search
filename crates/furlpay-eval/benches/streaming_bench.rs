use criterion::{black_box, criterion_group, criterion_main, Criterion};
use furlpay_core::index::trigram::TrigramDirectory;
use furlpay_core::index::schema::DocId;
use furlpay_core::routing::classifier::QueryRouter;

fn bench_streaming_stages(c: &mut Criterion) {
    let mut group = c.benchmark_group("furlpay_eval_streaming_stages");

    let mut index = TrigramDirectory::new();
    let code = b"export async function authenticationMiddleware(req: Request, res: Response) { return next(); }";
    index.index_document(DocId(0), code);

    // Stage 1: Query classification latency
    group.bench_function("stage1_query_classification", |b| {
        b.iter(|| {
            QueryRouter::classify(black_box("authenticationMiddleware"))
        });
    });

    // Stage 2: Index lookup latency (TTFM)
    group.bench_function("stage2_index_lookup_ttfm", |b| {
        b.iter(|| {
            index.search_exact(black_box(b"authenticationMiddleware"))
        });
    });

    // Stage 3: Verification (TTFV - first verified result)
    group.bench_function("stage3_first_verified_result_ttfv", |b| {
        b.iter(|| {
            let matches = index.search_exact(black_box(b"authenticationMiddleware"));
            let verified = matches.iter().find(|&&id| id == DocId(0));
            black_box(verified)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_streaming_stages);
criterion_main!(benches);
