//! Multi-source evidence synthesis and cited report generation.

use serde::{Deserialize, Serialize};
use super::citations::Citation;
use super::evidence::EvidenceItem;
use super::source_analysis::{SourceAnalyzer, SourceTier};

/// A synthesized deep research dossier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReport {
    pub topic: String,
    pub executive_summary: String,
    pub key_findings: Vec<String>,
    pub evidence: Vec<EvidenceItem>,
    pub citations: Vec<Citation>,
    pub primary_sources_count: usize,
    pub generated_at: String,
}

pub struct ResearchSynthesizer;

impl ResearchSynthesizer {
    /// Synthesize evidence and citations into a structured research report.
    pub fn synthesize(
        topic: &str,
        evidence: Vec<EvidenceItem>,
        citations: Vec<Citation>,
    ) -> ResearchReport {
        let mut key_findings = Vec::new();
        for item in &evidence {
            if item.confidence >= 0.80 {
                let finding_text = format!("{} [^{}]", item.statement, item.citation_id);
                if !key_findings.contains(&finding_text) {
                    key_findings.push(finding_text);
                }
            }
            if key_findings.len() >= 6 {
                break;
            }
        }

        let primary_sources_count = citations.iter().filter(|c| {
            let score = SourceAnalyzer::analyze_source(&c.url);
            score.tier == SourceTier::Tier1Primary || score.tier == SourceTier::Tier2Reputable
        }).count();

        let summary = format!(
            "Comprehensive research into '{}' across {} verified sources ({} primary/authoritative). Key findings and quantitative evidence synthesized below.",
            topic,
            citations.len(),
            primary_sources_count
        );

        ResearchReport {
            topic: topic.to_string(),
            executive_summary: summary,
            key_findings,
            evidence,
            citations,
            primary_sources_count,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesizer() {
        let citation = Citation::new(
            1,
            "https://arxiv.org/abs/2412.19437",
            "DeepSeek-V3 Technical Report",
            "arXiv",
            "DeepSeek-V3 operates with 671B parameters and activates 37B per token",
        );
        let evidence = EvidenceItem {
            statement: "DeepSeek-V3 operates with 671B parameters and activates 37B per token".into(),
            source_url: "https://arxiv.org/abs/2412.19437".into(),
            confidence: 0.95,
            citation_id: 1,
            has_numerical_claim: true,
        };

        let report = ResearchSynthesizer::synthesize("DeepSeek-V3", vec![evidence], vec![citation]);
        assert_eq!(report.citations.len(), 1);
        assert_eq!(report.primary_sources_count, 1);
        assert!(!report.key_findings.is_empty());
    }
}
