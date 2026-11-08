//! Dynamic confidence calculation and ranking calibration.

use super::candidate::{PatternCandidate, ValidationStatus};

/// Calibrate and score candidate confidence based on evidence and validation depth.
pub struct ConfidenceScorer;

impl ConfidenceScorer {
    /// Calculate a normalized confidence score [0.0, 1.0] for a candidate.
    pub fn score(candidate: &PatternCandidate, context_boost: f32) -> f32 {
        let base_score = match candidate.validation_status {
            ValidationStatus::CryptographicallyVerified => 0.98,
            ValidationStatus::FormatValidated => 0.88,
            ValidationStatus::SynthesizedOrTestOnly => 0.70,
            ValidationStatus::ChecksumFailed => 0.35,
            ValidationStatus::Unvalidated => 0.50,
        };

        // Combine base score with context boost and candidate-specific confidence
        let combined = (base_score * 0.6) + (candidate.confidence * 0.3) + (context_boost * 0.1);
        combined.clamp(0.0, 1.0)
    }

    /// Sort candidates in descending order of calibrated confidence.
    pub fn rank_candidates(candidates: &mut [PatternCandidate]) {
        candidates.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recognition::candidate::PatternCategory;
    use std::collections::HashMap;

    #[test]
    fn test_confidence_calibration() {
        let candidate = PatternCandidate {
            recognizer_id: "test".into(),
            category: PatternCategory::Blockchain,
            display_name: "Test Candidate".into(),
            raw_value: "val".into(),
            normalized_value: "val".into(),
            validation_status: ValidationStatus::CryptographicallyVerified,
            confidence: 0.99,
            metadata: HashMap::new(),
            evidence: "valid".into(),
            suggested_route: "route".into(),
        };

        let scored = ConfidenceScorer::score(&candidate, 0.5);
        assert!(scored >= 0.90);
    }
}
