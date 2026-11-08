//! GIEA Adaptive stopping controller:
//!
//! Gain_t = NewRelevantEvidence_t / (RetrievalCost_t + ε)

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use super::coverage::CoverageReport;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoppingReason {
    MaxPagesReached,
    MaxDurationExceeded,
    MaxDomainsReached,
    CoverageTargetMet,
    DiminishingMarginalGain,
    FrontierExhausted,
    ClientCancelled,
}

pub struct AdaptiveStoppingController {
    max_pages: usize,
    max_domains: usize,
    max_duration: Duration,
    min_marginal_gain: f32,
    start_time: Instant,
    last_evidence_count: usize,
}

impl AdaptiveStoppingController {
    pub fn new(max_pages: usize, max_domains: usize, max_duration: Duration) -> Self {
        Self {
            max_pages,
            max_domains,
            max_duration,
            min_marginal_gain: 0.10,
            start_time: Instant::now(),
            last_evidence_count: 0,
        }
    }

    /// Check if research should stop and return reason.
    pub fn should_stop(
        &mut self,
        current_pages: usize,
        current_domains: usize,
        total_evidence: usize,
        coverage: &CoverageReport,
        frontier_len: usize,
        is_cancelled: bool,
    ) -> Option<StoppingReason> {
        if is_cancelled {
            return Some(StoppingReason::ClientCancelled);
        }

        if current_pages >= self.max_pages {
            return Some(StoppingReason::MaxPagesReached);
        }

        if current_domains >= self.max_domains {
            return Some(StoppingReason::MaxDomainsReached);
        }

        if self.start_time.elapsed() >= self.max_duration {
            return Some(StoppingReason::MaxDurationExceeded);
        }

        if frontier_len == 0 {
            return Some(StoppingReason::FrontierExhausted);
        }

        // Calculate marginal evidence gain
        let new_evidence = total_evidence.saturating_sub(self.last_evidence_count);
        self.last_evidence_count = total_evidence;

        let marginal_gain = (new_evidence as f32) / (1.0 + 0.05); // ε = 0.05

        // If coverage is adequate and marginal gain drops below threshold -> stop gracefully
        if coverage.is_adequate && current_pages >= 5 && marginal_gain < self.min_marginal_gain {
            return Some(StoppingReason::DiminishingMarginalGain);
        }

        if coverage.overall_coverage >= 0.95 && current_pages >= 4 {
            return Some(StoppingReason::CoverageTargetMet);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hard_page_limit() {
        let mut controller = AdaptiveStoppingController::new(10, 5, Duration::from_secs(30));
        let coverage = CoverageReport { subtopics: vec![], overall_coverage: 0.5, is_adequate: false };
        let reason = controller.should_stop(10, 2, 5, &coverage, 10, false);
        assert_eq!(reason, Some(StoppingReason::MaxPagesReached));
    }

    #[test]
    fn test_coverage_target_met() {
        let mut controller = AdaptiveStoppingController::new(50, 10, Duration::from_secs(30));
        let coverage = CoverageReport { subtopics: vec![], overall_coverage: 0.98, is_adequate: true };
        let reason = controller.should_stop(6, 3, 12, &coverage, 20, false);
        assert_eq!(reason, Some(StoppingReason::CoverageTargetMet));
    }
}
