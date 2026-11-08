//! Benchmark and evaluation report generator.

use serde::{Deserialize, Serialize};
use crate::recall_eval::QualityMetrics;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub os: String,
    pub arch: String,
    pub rustc_version: String,
    pub cpu_simd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub timestamp_utc: String,
    pub commit_hash: String,
    pub environment: EnvironmentInfo,
    pub quality: QualityReport,
    pub performance_targets: TargetComparison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub total_queries: usize,
    pub mrr: f64,
    pub target_mrr: f64,
    pub precision_at_1: f64,
    pub recall_at_10: f64,
    pub mrr_passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetComparison {
    pub p50_target_us: u64,
    pub p95_target_us: u64,
    pub p99_target_us: u64,
    pub mrr_target: f64,
}

pub fn generate_report(metrics: &QualityMetrics) -> BenchmarkReport {
    let cpu = furlpay_simd::detect_cpu();

    BenchmarkReport {
        timestamp_utc: chrono_stub(),
        commit_hash: "head".into(),
        environment: EnvironmentInfo {
            os: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
            rustc_version: "rustc 1.98.1".into(),
            cpu_simd: cpu.name().into(),
        },
        quality: QualityReport {
            total_queries: metrics.total_queries,
            mrr: metrics.mrr,
            target_mrr: 0.435,
            precision_at_1: metrics.precision_at_1,
            recall_at_10: metrics.recall_at_10,
            mrr_passed: metrics.mrr >= 0.435,
        },
        performance_targets: TargetComparison {
            p50_target_us: 268,
            p95_target_us: 395,
            p99_target_us: 489,
            mrr_target: 0.435,
        },
    }
}

fn chrono_stub() -> String {
    "2026-11-08T16:15:00Z".into()
}
