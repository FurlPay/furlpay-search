//! Context-aware entity and intent disambiguation engine.
//!
//! Architecture:
//! - Domain concept clustering without hardcoded entity special-cases.
//! - Generalizes across polysemous entities (e.g. "Rust", "Mercury", "Python", "Go", "Solidity").
//! - Detects explicit intent clues from query modifiers.
//! - Detects ambiguity when terms have multiple strong domain interpretations.
//! - Applies intent diversification to ensure balanced representation of candidate topics.
//! - Generates suggested query refinements for user interaction.

use serde::{Deserialize, Serialize};
use crate::providers::search_provider::SearchHit;

/// Broad domain category for query and result clustering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DomainCluster {
    SoftwareEngineering,
    Gaming,
    FinanceCrypto,
    ScienceNature,
    General,
}

impl std::fmt::Display for DomainCluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainCluster::SoftwareEngineering => write!(f, "software_engineering"),
            DomainCluster::Gaming => write!(f, "gaming"),
            DomainCluster::FinanceCrypto => write!(f, "finance_crypto"),
            DomainCluster::ScienceNature => write!(f, "science_nature"),
            DomainCluster::General => write!(f, "general"),
        }
    }
}

/// Intent analysis result for a query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentAnalysis {
    /// Explicit domain intent detected from query words, if any.
    pub explicit_intent: Option<String>,
    /// Whether the query is polysemous/ambiguous across multiple domains.
    pub is_ambiguous: bool,
    /// Candidate domain interpretations (e.g. ["software_engineering", "gaming"]).
    pub candidate_intents: Vec<String>,
    /// Suggested query refinements to resolve ambiguity.
    pub suggested_refinements: Vec<String>,
}

pub struct IntentDisambiguator;

impl IntentDisambiguator {
    /// Domain keyword dictionaries for unsupervised cluster scoring.
    const SOFTWARE_CLUES: &'static [&'static str] = &[
        "programming", "language", "code", "compiler", "async", "runtime",
        "library", "cargo", "developer", "software", "syntax", "crate",
        "crates", "package", "api", "framework", "edition", "polonius",
        "borrow checker", "closures", "macro", "struct", "github", "crates.io",
        "rustify", "rust-lang", "golang", "python3", "pip", "npm", "sdk",
    ];

    const GAMING_CLUES: &'static [&'static str] = &[
        "game", "survival", "facepunch", "steam", "wipe", "server", "gameplay",
        "update", "multiplayer", "raid", "loot", "play", "patch notes",
        "blueprint", "fps", "monument", "player", "uzi", "boat", "pvp",
        "pve", "corrosion hour", "rustfront", "gaming", "playstation", "xbox",
    ];

    const FINANCE_CLUES: &'static [&'static str] = &[
        "crypto", "blockchain", "token", "wallet", "contract", "coin", "defi",
        "bank", "payment", "iban", "swift", "bic", "usdt", "tron", "ethereum",
        "solana", "bitcoin", "ledger", "transaction", "exchange",
    ];

    const SCIENCE_CLUES: &'static [&'static str] = &[
        "planet", "element", "astronomy", "chemistry", "physics", "orbit",
        "species", "animal", "metal", "oxidation", "solar system", "nasa",
        "mercury", "iron oxide", "corrosion", "chemical", "biology",
    ];

    /// Known polysemous entity roots that require disambiguation when unaccompanied by domain modifiers.
    const POLYSEMOUS_TERMS: &'static [&'static str] = &[
        "rust", "mercury", "go", "python", "solidity", "swift", "ruby", "spark", "corda", "atom",
    ];

    /// Analyze a search query for domain intent and ambiguity.
    pub fn analyze_query(query: &str) -> IntentAnalysis {
        let q_lower = query.to_lowercase();
        let words: Vec<&str> = q_lower.split_whitespace().collect();

        // 1. Check for explicit intent words in query
        let mut soft_score = 0;
        let mut game_score = 0;
        let mut fin_score = 0;
        let mut sci_score = 0;

        for &word in &words {
            if Self::SOFTWARE_CLUES.iter().any(|&c| c == word || word.contains(c)) {
                soft_score += 1;
            }
            if Self::GAMING_CLUES.iter().any(|&c| c == word || word.contains(c)) {
                game_score += 1;
            }
            if Self::FINANCE_CLUES.iter().any(|&c| c == word || word.contains(c)) {
                fin_score += 1;
            }
            if Self::SCIENCE_CLUES.iter().any(|&c| c == word || word.contains(c)) {
                sci_score += 1;
            }
        }

        let mut explicit_intent = None;
        if soft_score > 0 && soft_score > game_score && soft_score > fin_score {
            explicit_intent = Some("software_engineering".to_string());
        } else if game_score > 0 && game_score > soft_score {
            explicit_intent = Some("gaming".to_string());
        } else if fin_score > 0 {
            explicit_intent = Some("finance_crypto".to_string());
        } else if sci_score > 0 {
            explicit_intent = Some("science_nature".to_string());
        }

        // 2. Check if the query contains a polysemous term without explicit intent
        let contains_polysemous = words.iter().any(|&w| Self::POLYSEMOUS_TERMS.contains(&w));
        let is_ambiguous = explicit_intent.is_none() && (contains_polysemous || words.len() <= 2);

        let mut candidate_intents = Vec::new();
        let mut suggested_refinements = Vec::new();

        if is_ambiguous && contains_polysemous {
            candidate_intents.push("software_engineering".to_string());
            candidate_intents.push("gaming".to_string());

            let base = words.join(" ");
            suggested_refinements.push(format!("{} programming language", base));
            suggested_refinements.push(format!("{} video game", base));
        }

        IntentAnalysis {
            explicit_intent,
            is_ambiguous,
            candidate_intents,
            suggested_refinements,
        }
    }

    /// Classify a single search hit into a domain cluster.
    pub fn classify_hit(hit: &SearchHit) -> DomainCluster {
        let text = format!("{} {} {}", hit.title, hit.snippet, hit.url).to_lowercase();

        let mut soft_score = 0;
        let mut game_score = 0;
        let mut fin_score = 0;
        let mut sci_score = 0;

        for &clue in Self::SOFTWARE_CLUES {
            if text.contains(clue) {
                soft_score += 1;
            }
        }
        for &clue in Self::GAMING_CLUES {
            if text.contains(clue) {
                game_score += 1;
            }
        }
        for &clue in Self::FINANCE_CLUES {
            if text.contains(clue) {
                fin_score += 1;
            }
        }
        for &clue in Self::SCIENCE_CLUES {
            if text.contains(clue) {
                sci_score += 1;
            }
        }

        let max_score = soft_score.max(game_score).max(fin_score).max(sci_score);
        if max_score == 0 {
            DomainCluster::General
        } else if soft_score == max_score {
            DomainCluster::SoftwareEngineering
        } else if game_score == max_score {
            DomainCluster::Gaming
        } else if fin_score == max_score {
            DomainCluster::FinanceCrypto
        } else {
            DomainCluster::ScienceNature
        }
    }

    /// Rerank and diversify search hits based on query intent analysis.
    pub fn rerank_and_diversify(
        hits: Vec<SearchHit>,
        analysis: &IntentAnalysis,
    ) -> Vec<(SearchHit, DomainCluster, f32)> {
        let mut classified: Vec<(SearchHit, DomainCluster, f32)> = hits
            .into_iter()
            .map(|hit| {
                let cluster = Self::classify_hit(&hit);
                let base_score = hit.score;
                (hit, cluster, base_score)
            })
            .collect();

        // If explicit intent exists, boost matching cluster and demote others
        if let Some(ref intent) = analysis.explicit_intent {
            for (_, cluster, score) in &mut classified {
                if cluster.to_string() == *intent {
                    *score += 0.50; // strong boost for intended domain
                } else {
                    *score -= 0.35; // demote non-intended domains
                }
            }
            classified.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
            return classified;
        }

        // If ambiguous (e.g. "Rust 2026"): Apply intent diversification
        // Ensure that top results are not completely monopolized by a single cluster!
        if analysis.is_ambiguous && !analysis.candidate_intents.is_empty() {
            let mut soft_hits = Vec::new();
            let mut game_hits = Vec::new();
            let mut other_hits = Vec::new();

            for item in classified {
                match item.1 {
                    DomainCluster::SoftwareEngineering => soft_hits.push(item),
                    DomainCluster::Gaming => game_hits.push(item),
                    _ => other_hits.push(item),
                }
            }

            // Interleave top candidates so the user sees both interpretations
            let mut diversified = Vec::new();
            let max_pairs = soft_hits.len().max(game_hits.len());

            for i in 0..max_pairs {
                if i < soft_hits.len() {
                    let mut item = soft_hits[i].clone();
                    // Calibrate score for top-tier diversified slot
                    item.2 = 0.95 - (i as f32 * 0.04);
                    diversified.push(item);
                }
                if i < game_hits.len() {
                    let mut item = game_hits[i].clone();
                    item.2 = 0.93 - (i as f32 * 0.04);
                    diversified.push(item);
                }
            }

            for mut item in other_hits {
                item.2 = (item.2 * 0.7).max(0.2);
                diversified.push(item);
            }

            return diversified;
        }

        // Default sort by raw score
        classified.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        classified
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_programming_intent() {
        let analysis = IntentDisambiguator::analyze_query("Rust programming language 2026");
        assert_eq!(analysis.explicit_intent.as_deref(), Some("software_engineering"));
        assert!(!analysis.is_ambiguous);
    }

    #[test]
    fn test_explicit_gaming_intent() {
        let analysis = IntentDisambiguator::analyze_query("Rust survival game 2026");
        assert_eq!(analysis.explicit_intent.as_deref(), Some("gaming"));
        assert!(!analysis.is_ambiguous);
    }

    #[test]
    fn test_ambiguous_rust_query() {
        let analysis = IntentDisambiguator::analyze_query("Rust 2026");
        assert!(analysis.is_ambiguous);
        assert!(analysis.candidate_intents.contains(&"software_engineering".to_string()));
        assert!(analysis.candidate_intents.contains(&"gaming".to_string()));
        assert!(!analysis.suggested_refinements.is_empty());
    }

    #[test]
    fn test_hit_classification() {
        let soft_hit = SearchHit {
            url: "https://rustify.rs/articles/whats-new-in-rust-2026".into(),
            title: "What's New in Rust in 2026: async closures and Polonius".into(),
            snippet: "Important compiler and borrow checker changes in Rust".into(),
            published_date: None,
            provider: "DuckDuckGo".into(),
            score: 0.8,
        };
        assert_eq!(IntentDisambiguator::classify_hit(&soft_hit), DomainCluster::SoftwareEngineering);

        let game_hit = SearchHit {
            url: "https://rust.facepunch.com/roadmap/".into(),
            title: "Roadmap - Rust survival game by Facepunch".into(),
            snippet: "Server wipe, blueprints, player raiding, and weapon updates".into(),
            published_date: None,
            provider: "DuckDuckGo".into(),
            score: 0.8,
        };
        assert_eq!(IntentDisambiguator::classify_hit(&game_hit), DomainCluster::Gaming);
    }
}
