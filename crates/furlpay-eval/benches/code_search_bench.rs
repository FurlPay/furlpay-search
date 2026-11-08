//! Criterion benchmark for end-to-end code search latency (p50/p90/p95/p99).

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, black_box};
use furlpay_core::index::trigram::TrigramDirectory;
use furlpay_core::index::schema::DocId;

fn bench_trigram_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("furlpay_core_search");

    // Create and populate a test index with realistic code lines
    let mut index = TrigramDirectory::new();
    let sample_files: Vec<String> = (0..500)
        .map(|i| {
            format!(
                "pub async fn handle_payment_transaction_{i}(req: &PaymentRequest) -> Result<TransferReceipt, PaymentError> {{ \
                 let auth = verify_jwt_token(&req.token)?; \
                 let ledger = acquire_ledger_lock().await; \
                 ledger.settle_transfer(req.from, req.to, req.amount).await?; \
                 Ok(TransferReceipt::new(req.id, 0x{:x})) }}",
                i
            )
        })
        .collect();

    for (i, content) in sample_files.iter().enumerate() {
        index.index_document(DocId(i as u32), content.as_bytes());
    }

    group.bench_function("exact_search_short_identifier", |b| {
        b.iter(|| index.search_exact(black_box(b"handle_payment")))
    });

    group.bench_function("exact_search_symbol_type", |b| {
        b.iter(|| index.search_exact(black_box(b"TransferReceipt")))
    });

    group.bench_function("exact_search_multi_token", |b| {
        b.iter(|| index.search_exact(black_box(b"verify_jwt_token")))
    });

    group.bench_function("lookup_single_trigram_hash", |b| {
        let hash = xxhash_rust::xxh3::xxh3_64(b"han");
        b.iter(|| index.lookup(black_box(hash)))
    });

    for size in [50, 200, 500].iter() {
        let mut idx = TrigramDirectory::new();
        for i in 0..*size {
            let content = format!("fn authenticate_user_{i}(token: &str) -> bool {{ true }}");
            idx.index_document(DocId(i as u32), content.as_bytes());
        }
        group.bench_with_input(
            BenchmarkId::new("corpus_scaling", size),
            size,
            |b, _| b.iter(|| idx.search_exact(black_box(b"authenticate_user"))),
        );
    }

    group.finish();
}

criterion_group!(benches, bench_trigram_index);
criterion_main!(benches);
