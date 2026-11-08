//! Subtopic coverage and research completeness assessment:
//!
//! Coverage = (Σ_i importance_i * completion_i) / (Σ_i importance_i)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtopicProgress {
    pub name: String,
    pub importance: f32,
    pub evidence_count: usize,
    pub completion: f32, // [0.0, 1.0]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub subtopics: Vec<SubtopicProgress>,
    pub overall_coverage: f32,
    pub is_adequate: bool,
}

pub struct CoverageTracker {
    subtopics: Vec<SubtopicProgress>,
    target_evidence_per_subtopic: usize,
    adequacy_threshold: f32,
}

impl CoverageTracker {
    pub fn new(subtopic_names: &[String]) -> Self {
        let subtopics = subtopic_names.iter().map(|name| SubtopicProgress {
            name: name.clone(),
            importance: 1.0,
            evidence_count: 0,
            completion: 0.0,
        }).collect();

        Self {
            subtopics,
            target_evidence_per_subtopic: 3,
            adequacy_threshold: 0.75,
        }
    }

    /// Record a discovered piece of evidence against matching subtopics.
    pub fn record_evidence(&mut self, text: &str) {
        let lower = text.to_lowercase();
        for subtopic in &mut self.subtopics {
            let terms: Vec<&str> = subtopic.name.split_whitespace().collect();
            if terms.iter().any(|&t| lower.contains(&t.to_lowercase())) {
                subtopic.evidence_count += 1;
                subtopic.completion = (subtopic.evidence_count as f32 / self.target_evidence_per_subtopic as f32).min(1.0);
            }
        }
    }

    /// Compute current overall coverage and adequacy.
    pub fn evaluate_coverage(&self) -> CoverageReport {
        if self.subtopics.is_empty() {
            return CoverageReport {
                subtopics: vec![],
                overall_coverage: 1.0,
                is_adequate: true,
            };
        }

        let total_weight: f32 = self.subtopics.iter().map(|s| s.importance).sum();
        let weighted_completion: f32 = self.subtopics.iter().map(|s| s.importance * s.completion).sum();

        let overall_coverage = if total_weight > 0.0 {
            weighted_completion / total_weight
        } else {
            0.0
        };

        CoverageReport {
            subtopics: self.subtopics.clone(),
            overall_coverage,
            is_adequate: overall_coverage >= self.adequacy_threshold,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_tracking() {
        let mut tracker = CoverageTracker::new(&["consensus mechanism".into(), "transaction fees".into()]);
        assert_eq!(tracker.evaluate_coverage().overall_coverage, 0.0);

        tracker.record_evidence("Proof of Stake consensus mechanism validated");
        tracker.record_evidence("Byzantine fault tolerance consensus mechanism");
        tracker.record_evidence("Nakamoto consensus mechanism finalized");

        let report = tracker.evaluate_coverage();
        assert!(report.overall_coverage >= 0.50);
    }
}
