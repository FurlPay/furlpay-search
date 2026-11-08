//! Retrieval quality integration test asserting MRR > 0.435.

use furlpay_eval::dataset::Dataset;
use furlpay_eval::recall_eval::evaluate_dataset;

#[test]
fn test_retrieval_quality_mrr_gate() {
    let dataset = Dataset::default_fixture();
    let metrics = evaluate_dataset(&dataset);

    println!("Quality Metrics: MRR={:.4}, P@1={:.2}%, R@10={:.2}%",
        metrics.mrr, metrics.precision_at_1 * 100.0, metrics.recall_at_10 * 100.0
    );

    assert!(
        metrics.mrr > 0.435,
        "Acceptance criteria failure: MRR {:.4} <= 0.435 target!",
        metrics.mrr
    );
    assert_eq!(metrics.precision_at_1, 1.0);
}
