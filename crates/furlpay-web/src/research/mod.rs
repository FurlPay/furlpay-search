//! Multi-source research synthesis, fact extraction, citations, ranking, and authority analysis.

pub mod citations;
pub mod evidence;
pub mod source_analysis;
pub mod synthesis;
pub mod coverage;
pub mod stopping;
pub mod ranking;

pub use citations::Citation;
pub use evidence::{EvidenceExtractor, EvidenceItem};
pub use source_analysis::{SourceAnalyzer, SourceScore, SourceTier};
pub use synthesis::{ResearchSynthesizer, ResearchReport};
pub use coverage::{CoverageTracker, CoverageReport, SubtopicProgress};
pub use stopping::{AdaptiveStoppingController, StoppingReason};
pub use ranking::{ResearchRanker, CandidateDocument, RankingWeights};
