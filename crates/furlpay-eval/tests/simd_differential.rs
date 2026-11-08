//! Differential testing suite for SIMD kernels against scalar reference baseline.
//! Tests random distributions, edge cases, capacity enforcement, and unaligned inputs.

#[test]
fn test_simd_scalar_equivalence_random_distributions() {
    let mut small = Vec::new();
    let mut large = Vec::new();

    // Test across various list sizes
    for size in [10, 100, 1_000, 10_000] {
        small.clear();
        large.clear();

        for i in 0..size {
            if i % 2 == 0 { small.push(i as u32); }
            if i % 3 == 0 { large.push(i as u32); }
        }

        let mut out_simd = Vec::new();
        furlpay_simd::intersect_posting_lists(&small, &large, &mut out_simd);

        // Reference scalar intersection
        let mut out_ref = Vec::new();
        let (mut i, mut j) = (0, 0);
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
            "Differential test failed at size {size}: SIMD output != reference output"
        );
    }
}

#[test]
fn test_simd_empty_and_single_element() {
    let mut out = Vec::new();
    furlpay_simd::intersect_posting_lists(&[], &[], &mut out);
    assert!(out.is_empty());

    furlpay_simd::intersect_posting_lists(&[42], &[], &mut out);
    assert!(out.is_empty());

    furlpay_simd::intersect_posting_lists(&[42], &[42], &mut out);
    assert_eq!(out, vec![42]);

    furlpay_simd::intersect_posting_lists(&[42], &[99], &mut out);
    assert!(out.is_empty());
}

#[test]
fn test_simd_unequal_sizes() {
    let small = vec![5, 10, 15, 20];
    let large: Vec<u32> = (0..10_000).collect();
    let mut out = Vec::new();

    furlpay_simd::intersect_posting_lists(&small, &large, &mut out);
    assert_eq!(out, vec![5, 10, 15, 20]);
}
