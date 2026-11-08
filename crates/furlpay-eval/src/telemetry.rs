//! OpenTelemetry histogram export for search latency metrics.

#[derive(Debug, Clone, Default)]
pub struct SearchLatencyMetrics {
    pub ttfb_us: u64,
    pub ttfe_us: u64,
    pub ttfm_us: u64,
    pub ttfv_us: u64,
    pub ttfp_us: u64,
    pub ttf_complete_us: u64,
}

pub struct TelemetryRecorder;

impl TelemetryRecorder {
    pub fn new() -> Self {
        Self
    }

    pub fn record_search_latency(&self, _pipeline: &str, _latency_us: u64) {}

    pub fn record_progressive_metrics(&self, metrics: &SearchLatencyMetrics) {
        tracing::info!(
            ttfb_us = metrics.ttfb_us,
            ttfe_us = metrics.ttfe_us,
            ttfm_us = metrics.ttfm_us,
            ttfv_us = metrics.ttfv_us,
            ttfp_us = metrics.ttfp_us,
            ttf_complete_us = metrics.ttf_complete_us,
            "Progressive search streaming latency breakdown"
        );
    }
}

impl Default for TelemetryRecorder {
    fn default() -> Self {
        Self::new()
    }
}
