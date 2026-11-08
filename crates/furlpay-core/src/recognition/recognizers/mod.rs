//! Recognizers catalogue.

pub mod blockchain;
pub mod financial;
pub mod internet;
pub mod crypto;
pub mod software;
pub mod standards;

use crate::recognition::candidate::{PatternCandidate, PatternCategory};

/// Pluggable interface implemented by every pattern recognizer.
pub trait PatternRecognizer: Send + Sync {
    /// Unique recognizer identifier (e.g. "blockchain.tron.base58check").
    fn id(&self) -> &str;

    /// Domain category.
    fn category(&self) -> PatternCategory;

    /// Inspect the query string and return any recognized candidates.
    fn recognize(&self, input: &str) -> Vec<PatternCandidate>;
}
