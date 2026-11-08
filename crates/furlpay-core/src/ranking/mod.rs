//! Multi-signal relevance scoring and Reciprocal Rank Fusion.

pub mod adaptive;
pub mod rrf;
pub mod scorer;

pub use adaptive::{intent_classifier, QueryIntent, RankingExplanation, RankingFeatures, RankingWeights};
