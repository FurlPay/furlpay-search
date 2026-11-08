//! Fallback handling for unrecognized, ambiguous, or general-text queries.

use super::candidate::{PatternCandidate, PatternCategory, ValidationStatus};
use std::collections::HashMap;

/// Fallback generator ensuring arbitrary queries are never rejected.
pub struct FallbackGenerator;

impl FallbackGenerator {
    /// Generate a general fallback candidate when no specialized recognizer claims the query.
    pub fn generate_fallback(raw_query: &str) -> PatternCandidate {
        let trimmed = raw_query.trim();
        let is_multilingual_or_question = trimmed.ends_with('?')
            || trimmed.split_whitespace().count() > 3
            || trimmed.chars().any(|c| (c as u32) > 127);

        let (display_name, route) = if is_multilingual_or_question {
            ("Natural Language / Global Web Query", "GlobalWebDiscovery")
        } else if trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            ("Unrecognized Identifier / Keyword", "GeneralSearch")
        } else {
            ("General Search Query", "GlobalWebDiscovery")
        };

        PatternCandidate {
            recognizer_id: "general.universal_fallback".into(),
            category: PatternCategory::GeneralText,
            display_name: display_name.into(),
            raw_value: trimmed.to_string(),
            normalized_value: trimmed.to_string(),
            validation_status: ValidationStatus::Unvalidated,
            confidence: 0.50,
            metadata: HashMap::new(),
            evidence: "No specialized domain syntax detected; routed to universal global search".into(),
            suggested_route: route.into(),
        }
    }
}
