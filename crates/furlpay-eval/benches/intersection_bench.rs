//! SIMD vs Scalar intersection benchmark.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, black_box};

fn bench_intersection(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_vs_scalar_intersection");

    let arch = furlpay_simd::detect_cpu();
    println!("Detected Hardware Acceleration: {:?} ({})", arch, arch.name());

    for size in [100, 1_000, 10_000, 50_000].iter() {
        let small: Vec<u32> = (0..*size as u32).step_by(3).collect();
        let large: Vec<u32> = (0..*size as u32).step_by(2).collect();

        // 1. Verify semantic equivalence between scalar and SIMD before measuring speed
        let mut out_simd = Vec::new();
        furlpay_simd::intersect_posting_lists(&small, &large, &mut out_simd);

        // Reference intersection:
        let mut out_ref = Vec::new();
        let mut i = 0;
        let mut j = 0;
        while i < small.len() && j < large.len() {
            if small[i] == large[j] {
                out_ref.push(small[i]);
                i += 1;
                j += 1;
            } else if small[i] < large[j] {
                i += 1;
            } else {
                j += 1;
            }
        }

        assert_eq!(
            out_simd, out_ref,
            "CRITICAL: SIMD output must match reference scalar output bit-for-bit!"
        );

        // 2. Measure throughput
        group.bench_with_input(
            BenchmarkId::new("simd_intersect", size),
            size,
            |b, _| {
                let mut output = Vec::with_capacity(small.len());
                b.iter(|| {
                    furlpay_simd::intersect_posting_lists(
                        black_box(&small),
                        black_box(&large),
                        black_box(&mut output),
                    );
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_intersection);
criterion_main!(benches);
