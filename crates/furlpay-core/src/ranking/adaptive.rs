//! Query-adaptive ranking features, weights, and explainable scoring for FurlPay Search.
//!
//! # Architecture & Scoring Formula
//! The scoring architecture separates retrieval into three hierarchical stages:
//!
//! Stage 1 — Identity (Hierarchical, non-double-counting):
//!   S_identity = if exact_relative_path_match > 0.8:
//!                  w_exact_path * path_match + residual(filename, stem, dir)
//!                else if exact_filename_match > 0.8:
//!                  w_exact_filename * filename_match + residual(stem, dir)
//!                else if exact_filename_stem_match > 0.8:
//!                  w_exact_stem * stem_match + residual(symbol, dir)
//!                else:
//!                  w_filename_prefix * prefix + w_symbol * symbol + ...
//!
//! Stage 2 — Semantic & Content:
//!   S_semantic = w_content_token * content_token_prefix_match
//!              + w_context * contextual_relevance
//!              + w_trigram * trigram_similarity
//!
//! Stage 3 — Implementation Status & Quality:
//!   S_status = w_status * implementation_status_match
//!            + w_completeness * implementation_completeness
//!
//! Penalty:
//!   P_generic = w_penalty * generic_match_penalty
//!
//! Final Calibrated Score:
//!   S(d, q) = max(0.0, S_identity + S_semantic + S_status - P_generic)

use serde::{Deserialize, Serialize};

/// High-level query intent classification reflecting structure and matching goal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryIntent {
    /// 1-character prefix query (e.g. "t")
    SingleCharacterPrefix,
    /// 2-3 character short token prefix (e.g. "tr", "tro")
    ShortTokenPrefix,
    /// Exact filename stem matching a known component (e.g. "tron", "trigram")
    FilenameStem,
    /// Exact filename with extension (e.g. "tron.ts", "schema.json")
    FilenameWithExtension,
    /// Relative or absolute path query (e.g. "blockchain/tron.ts", "src/main.rs")
    ExactRelativePath,
    /// Exact programmatic identifier or camelCase/PascalCase/snake_case symbol
    ExactIdentifier,
    /// Code symbol lookup (function, struct, method, trait)
    SymbolLookup,
    /// Multi-term or natural language content query (e.g. "payment", "authentication architecture")
    ContentSearch,
    /// Query explicitly requesting stub, WIP, or unfinished implementation
    ExplicitImplementationStatus,
}

/// Helper functions for robust input normalization, token boundary matching, and intent classification.
pub mod intent_classifier {
    use super::QueryIntent;

    /// Normalize paths across Windows and Unix conventions.
    pub fn normalize_path(path: &str) -> String {
        let mut norm = path.replace('\\', "/");
        while norm.starts_with("./") {
            norm = norm[2..].to_string();
        }
        while norm.starts_with('/') {
            norm = norm[1..].to_string();
        }
        norm
    }

    /// Check if a query token matches stub / unfinished intent using strict token boundaries.
    /// Prevents false positives like "student" or "methodology".
    pub fn is_stub_token(token: &str) -> bool {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '!' && c != '(' && c != ')');
        matches!(
            clean,
            "stub"
                | "stubs"
                | "todo"
                | "todos"
                | "placeholder"
                | "placeholders"
                | "unimplemented"
                | "wip"
                | "todo!()"
                | "unimplemented!()"
        )
    }

    /// Detect whether the query contains explicit stub intent at word/token boundaries.
    pub fn detect_stub_intent(query: &str) -> bool {
        let lower = query.to_lowercase();
        // Split on whitespace or common punctuation
        for part in lower.split(|c: char| c.is_whitespace() || c == ',' || c == ':' || c == ';') {
            let part_trimmed = part.trim();
            if part_trimmed.is_empty() {
                continue;
            }
            if is_stub_token(part_trimmed) {
                return true;
            }
            // Also check inner tokens if delimited by parentheses
            for sub in part_trimmed.split(|c: char| c == '(' || c == ')' || c == '[' || c == ']') {
                let sub_trimmed = sub.trim();
                if is_stub_token(sub_trimmed) {
                    return true;
                }
            }
        }
        false
    }

    /// Strip stub tokens from the query to extract the underlying target item.
    pub fn strip_stub_terms(query: &str) -> String {
        let mut target_words = Vec::new();
        for word in query.split_whitespace() {
            let lower = word.to_lowercase();
            if !is_stub_token(&lower) {
                target_words.push(word);
            }
        }
        target_words.join(" ")
    }

    /// Check if a string has a valid file extension (1 to 8 alphanumeric characters after the last dot).
    pub fn has_file_extension(s: &str) -> bool {
        let path = std::path::Path::new(s);
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                let count = ext_str.chars().count();
                return count >= 1 && count <= 8 && ext_str.chars().all(|c| c.is_alphanumeric());
            }
        }
        false
    }

    /// Classify query intent based on structural and lexical characteristics.
    ///
    /// # Arguments
    /// * `query` - Raw user search query.
    /// * `has_indexed_stem` - Optional closure indicating if a term exists as an indexed file stem.
    pub fn classify_query(
        query: &str,
        has_indexed_stem: impl Fn(&str) -> bool,
    ) -> (QueryIntent, bool, String) {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return (QueryIntent::ContentSearch, false, String::new());
        }

        let is_stub_search = detect_stub_intent(trimmed);
        let clean_query = if is_stub_search {
            strip_stub_terms(trimmed)
        } else {
            trimmed.to_string()
        };

        if is_stub_search && clean_query.is_empty() {
            return (QueryIntent::ExplicitImplementationStatus, true, clean_query);
        }

        let eval_target = if clean_query.is_empty() { trimmed } else { &clean_query };
        let _norm_eval = normalize_path(eval_target);
        let char_count = eval_target.chars().count();

        // 1. Path check: contains '/' or '\\'
        if eval_target.contains('/') || eval_target.contains('\\') {
            return (QueryIntent::ExactRelativePath, is_stub_search, clean_query);
        }

        // 2. Filename with extension check: e.g. "tron.ts", "schema.json"
        if has_file_extension(eval_target) && !eval_target.contains(char::is_whitespace) {
            return (QueryIntent::FilenameWithExtension, is_stub_search, clean_query);
        }

        // 3. Length-based prefix classification (Unicode character-aware)
        if char_count == 1 {
            return (QueryIntent::SingleCharacterPrefix, is_stub_search, clean_query);
        }

        if char_count <= 3 && !eval_target.contains(char::is_whitespace) {
            return (QueryIntent::ShortTokenPrefix, is_stub_search, clean_query);
        }

        // 4. Check for known indexed stem (e.g. "tron", "trigram")
        let lower_target = eval_target.to_lowercase();
        if has_indexed_stem(&lower_target) {
            return (QueryIntent::FilenameStem, is_stub_search, clean_query);
        }

        // 5. Code identifier check (camelCase, PascalCase, snake_case)
        let is_code_identifier = (eval_target.contains('_') && !eval_target.contains(char::is_whitespace))
            || (eval_target.chars().any(|c| c.is_uppercase())
                && eval_target.chars().any(|c| c.is_lowercase())
                && !eval_target.contains(char::is_whitespace));

        if is_code_identifier {
            return (QueryIntent::ExactIdentifier, is_stub_search, clean_query);
        }

        // 6. Natural language or general content query (e.g. "payment", "authentication", "blockchain architecture")
        (QueryIntent::ContentSearch, is_stub_search, clean_query)
    }
}

/// Helper function to safely clamp feature values into [0.0, 1.0], treating NaNs and infinities as 0.0.
fn sanitize_feature(v: f32) -> f32 {
    if !v.is_finite() || v < 0.0 {
        0.0
    } else if v > 1.0 {
        1.0
    } else {
        v
    }
}

/// Explicit relevance features extracted for a document-query pair.
/// All features are guaranteed to be in [0.0, 1.0] and finite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RankingFeatures {
    // ── Stage 1: Identity Signals [0.0..1.0] ──
    /// Exact relative path match (e.g. "blockchain/tron.ts")
    pub exact_relative_path_match: f32,
    /// Exact filename with extension match (e.g. "tron.ts")
    pub exact_filename_match: f32,
    /// Exact filename stem match (e.g. "tron", "trigram")
    pub exact_filename_stem_match: f32,
    /// Filename stem starts with query prefix
    pub filename_prefix_match: f32,
    /// Directory component match (e.g. "blockchain" in "blockchain/tron.ts")
    pub directory_component_match: f32,
    /// Symbol name match in SymbolGraph
    pub symbol_prefix_match: f32,
    /// Identifier prefix match in code
    pub identifier_prefix_match: f32,

    // ── Stage 2: Semantic & Content Signals [0.0..1.0] ──
    /// Content token prefix match at word boundaries
    pub content_token_prefix_match: f32,
    /// Contextual relevance across surrounding lines
    pub contextual_relevance: f32,
    /// Trigram similarity coverage
    pub trigram_similarity: f32,

    // ── Stage 3: Implementation Signals [0.0..1.0] ──
    /// Actual intrinsic file implementation completeness (1.0 = complete, 0.05 = stub).
    /// This property describes the document itself regardless of query.
    pub implementation_completeness: f32,

    /// How well the document's completion status matches user search intent.
    /// If user is searching for stubs: 1.0 if doc is stub, 0.05 if complete.
    /// If user is searching normally: 1.0 if doc is complete, 0.05 if stub.
    pub implementation_status_match: f32,

    // ── Penalty [0.0..1.0] ──
    /// Generic non-boundary substring penalty
    pub generic_match_penalty: f32,

    // ── Legacy Compatibility Fields ──
    /// Legacy quality field (mirrors implementation_status_match)
    pub implementation_quality: f32,
    /// Legacy token_prefix field
    pub token_prefix: f32,
    /// Legacy exact_identifier field
    pub exact_identifier: f32,
    /// Legacy filename_prefix field
    pub filename_prefix: f32,
    /// Legacy symbol_match field
    pub symbol_match: f32,
}

impl Default for RankingFeatures {
    fn default() -> Self {
        Self {
            exact_relative_path_match: 0.0,
            exact_filename_match: 0.0,
            exact_filename_stem_match: 0.0,
            filename_prefix_match: 0.0,
            directory_component_match: 0.0,
            symbol_prefix_match: 0.0,
            identifier_prefix_match: 0.0,
            content_token_prefix_match: 0.0,
            contextual_relevance: 0.0,
            trigram_similarity: 0.0,
            implementation_completeness: 1.0,
            implementation_status_match: 1.0,
            generic_match_penalty: 0.0,
            implementation_quality: 1.0,
            token_prefix: 0.0,
            exact_identifier: 0.0,
            filename_prefix: 0.0,
            symbol_match: 0.0,
        }
    }
}

impl RankingFeatures {
    /// Sanitize all features into valid [0.0, 1.0] ranges, eliminating non-finite numbers (NaN/Inf).
    pub fn sanitize(&mut self) {
        self.exact_relative_path_match = sanitize_feature(self.exact_relative_path_match);
        self.exact_filename_match = sanitize_feature(self.exact_filename_match);
        self.exact_filename_stem_match = sanitize_feature(self.exact_filename_stem_match);
        self.filename_prefix_match = sanitize_feature(self.filename_prefix_match);
        self.directory_component_match = sanitize_feature(self.directory_component_match);
        self.symbol_prefix_match = sanitize_feature(self.symbol_prefix_match);
        self.identifier_prefix_match = sanitize_feature(self.identifier_prefix_match);
        self.content_token_prefix_match = sanitize_feature(self.content_token_prefix_match);
        self.contextual_relevance = sanitize_feature(self.contextual_relevance);
        self.trigram_similarity = sanitize_feature(self.trigram_similarity);
        self.implementation_completeness = sanitize_feature(self.implementation_completeness);
        self.implementation_status_match = sanitize_feature(self.implementation_status_match);
        self.generic_match_penalty = sanitize_feature(self.generic_match_penalty);

        // Keep legacy fields synchronized
        self.implementation_quality = self.implementation_status_match;
        self.token_prefix = sanitize_feature(self.token_prefix);
        self.exact_identifier = sanitize_feature(self.exact_identifier);
        self.filename_prefix = sanitize_feature(self.filename_prefix);
        self.symbol_match = sanitize_feature(self.symbol_match);
    }

    /// Compute hierarchical, non-double-counting relevance score:
    ///
    /// S(d, q) = S_identity + S_semantic + S_status - P_generic
    pub fn compute_score(&self, weights: &RankingWeights) -> f32 {
        // Stage 1: Hierarchical identity scoring to prevent correlated signal accumulation
        let s_identity = if self.exact_relative_path_match > 0.8 {
            let base = self.exact_relative_path_match * weights.w_exact_path;
            let residual = (self.exact_filename_match * weights.w_exact_filename * 0.05)
                + (self.exact_filename_stem_match * weights.w_exact_stem * 0.05)
                + (self.directory_component_match * weights.w_directory * 0.05);
            base + residual.min(8.0)
        } else if self.exact_filename_match > 0.8 {
            let base = self.exact_filename_match * weights.w_exact_filename;
            let residual = (self.exact_filename_stem_match * weights.w_exact_stem * 0.10)
                + (self.directory_component_match * weights.w_directory * 0.05);
            base + residual.min(6.0)
        } else if self.exact_filename_stem_match > 0.8 {
            let base = self.exact_filename_stem_match * weights.w_exact_stem;
            let residual = (self.symbol_prefix_match * weights.w_symbol * 0.10)
                + (self.directory_component_match * weights.w_directory * 0.05);
            base + residual.min(5.0)
        } else {
            let stem_prefix = self.filename_prefix_match * weights.w_filename_prefix;
            let sym_prefix = self.symbol_prefix_match * weights.w_symbol;
            let id_prefix = self.identifier_prefix_match * weights.w_identifier;
            let dir_match = self.directory_component_match * weights.w_directory;
            // Cap composite prefix matching to not exceed an exact stem match
            (stem_prefix + sym_prefix * 0.5 + id_prefix * 0.5 + dir_match * 0.3)
                .min(weights.w_exact_stem)
        };

        // Stage 2: Semantic & Content scoring
        let s_semantic = (self.content_token_prefix_match * weights.w_content_token)
            + (self.contextual_relevance * weights.w_context)
            + (self.trigram_similarity * weights.w_trigram);

        // Stage 3: Implementation status & completeness
        let s_status = (self.implementation_status_match * weights.w_status)
            + (self.implementation_completeness * weights.w_completeness);

        // Penalty: Generic non-boundary substring matches
        let p_generic = self.generic_match_penalty * weights.w_penalty;

        let raw = (s_identity + s_semantic + s_status - p_generic).max(0.0);
        if raw.is_finite() { raw } else { 0.0 }
    }

    /// Compute calibrated normalized score in [0.05, 0.99] range.
    pub fn compute_normalized_score(&self, weights: &RankingWeights) -> f32 {
        let raw = self.compute_score(weights);
        let normalized = (raw / 125.0).clamp(0.05, 0.99);
        (normalized * 1000.0).round() / 1000.0
    }

    /// Generate structured ranking explanation based on actual weighted signal contributions.
    pub fn explain(
        &self,
        weights: &RankingWeights,
        _intent: &QueryIntent,
        is_stub_query: bool,
    ) -> RankingExplanation {
        let mut signals: Vec<(&'static str, f32, String)> = Vec::new();

        if self.exact_relative_path_match > 0.0 {
            let c = self.exact_relative_path_match * weights.w_exact_path;
            signals.push(("exact_relative_path", c, "Exact relative path match".to_string()));
        }
        if self.exact_filename_match > 0.0 {
            let c = self.exact_filename_match * weights.w_exact_filename;
            signals.push(("exact_filename", c, "Exact filename with extension match".to_string()));
        }
        if is_stub_query && self.implementation_status_match > 0.5 {
            let c = self.implementation_status_match * weights.w_status;
            signals.push(("stub_status", c, "Requested stub / unfinished implementation found".to_string()));
        }
        if self.exact_filename_stem_match > 0.0 {
            let c = self.exact_filename_stem_match * weights.w_exact_stem;
            signals.push(("exact_stem", c, "Exact filename stem match".to_string()));
        }
        if self.filename_prefix_match > 0.0 {
            let c = self.filename_prefix_match * weights.w_filename_prefix;
            signals.push(("filename_prefix", c, "Filename stem starts with query prefix".to_string()));
        }
        if self.symbol_prefix_match > 0.0 {
            let c = self.symbol_prefix_match * weights.w_symbol;
            signals.push(("symbol_match", c, "Symbol in code graph matches query".to_string()));
        }
        if self.identifier_prefix_match > 0.0 {
            let c = self.identifier_prefix_match * weights.w_identifier;
            signals.push(("identifier_match", c, "Identifier in code matches prefix".to_string()));
        }
        if self.directory_component_match > 0.0 {
            let c = self.directory_component_match * weights.w_directory;
            signals.push(("directory_match", c, "Directory component matches query".to_string()));
        }
        if self.content_token_prefix_match > 0.0 {
            let c = self.content_token_prefix_match * weights.w_content_token;
            signals.push(("content_token", c, "Token at word boundary matches prefix".to_string()));
        }
        if self.contextual_relevance > 0.0 {
            let c = self.contextual_relevance * weights.w_context;
            signals.push(("contextual_relevance", c, "Strong contextual match in content".to_string()));
        }
        if self.trigram_similarity > 0.0 {
            let c = self.trigram_similarity * weights.w_trigram;
            signals.push(("trigram_similarity", c, "Content trigram similarity match".to_string()));
        }

        signals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let penalty = self.generic_match_penalty * weights.w_penalty;

        if let Some((name, val, summary)) = signals.first() {
            let summary_text = if penalty > 10.0 {
                format!("{} (penalty: -{:.1})", summary, penalty)
            } else {
                summary.clone()
            };
            RankingExplanation {
                dominant_signal: name.to_string(),
                dominant_contribution: *val,
                total_positive_score: signals.iter().map(|s| s.1).sum(),
                penalty_applied: penalty,
                summary: summary_text,
            }
        } else if penalty > 0.0 {
            RankingExplanation {
                dominant_signal: "generic_penalty".to_string(),
                dominant_contribution: 0.0,
                total_positive_score: 0.0,
                penalty_applied: penalty,
                summary: "Generic match penalty applied".to_string(),
            }
        } else {
            RankingExplanation {
                dominant_signal: "fallback".to_string(),
                dominant_contribution: 0.0,
                total_positive_score: 0.0,
                penalty_applied: 0.0,
                summary: "Content similarity match".to_string(),
            }
        }
    }

    /// Backwards compatible string explanation.
    pub fn explanation(&self, is_stub_query: bool) -> String {
        let weights = if is_stub_query {
            RankingWeights::stub_search()
        } else {
            RankingWeights::short_prefix()
        };
        let intent = if is_stub_query {
            QueryIntent::ExplicitImplementationStatus
        } else {
            QueryIntent::ShortTokenPrefix
        };
        self.explain(&weights, &intent, is_stub_query).summary
    }
}

/// Structured ranking explanation output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RankingExplanation {
    pub dominant_signal: String,
    pub dominant_contribution: f32,
    pub total_positive_score: f32,
    pub penalty_applied: f32,
    pub summary: String,
}

/// Query-dependent weights controlling signal influence based on query classification.
#[derive(Debug, Clone, PartialEq)]
pub struct RankingWeights {
    pub w_exact_path: f32,
    pub w_exact_filename: f32,
    pub w_exact_stem: f32,
    pub w_filename_prefix: f32,
    pub w_directory: f32,
    pub w_symbol: f32,
    pub w_identifier: f32,
    pub w_content_token: f32,
    pub w_context: f32,
    pub w_trigram: f32,
    pub w_status: f32,
    pub w_completeness: f32,
    pub w_penalty: f32,
}

impl RankingWeights {
    /// Weights for single-character prefixes (e.g. "t").
    /// Heavy penalty on generic substrings; prioritizes filename prefix.
    pub fn single_character_prefix() -> Self {
        Self {
            w_exact_path: 60.0,
            w_exact_filename: 55.0,
            w_exact_stem: 50.0,
            w_filename_prefix: 45.0,
            w_directory: 15.0,
            w_symbol: 25.0,
            w_identifier: 20.0,
            w_content_token: 5.0,
            w_context: 5.0,
            w_trigram: 5.0,
            w_status: 20.0,
            w_completeness: 10.0,
            w_penalty: 50.0,
        }
    }

    /// Weights for 2-3 char prefixes (e.g. "tr", "tro").
    pub fn short_prefix() -> Self {
        Self {
            w_exact_path: 65.0,
            w_exact_filename: 60.0,
            w_exact_stem: 55.0,
            w_filename_prefix: 42.0,
            w_directory: 18.0,
            w_symbol: 30.0,
            w_identifier: 25.0,
            w_content_token: 12.0,
            w_context: 8.0,
            w_trigram: 10.0,
            w_status: 20.0,
            w_completeness: 10.0,
            w_penalty: 45.0,
        }
    }

    /// Weights for exact relative path queries (e.g. "blockchain/tron.ts").
    pub fn exact_relative_path() -> Self {
        Self {
            w_exact_path: 85.0,
            w_exact_filename: 50.0,
            w_exact_stem: 40.0,
            w_filename_prefix: 20.0,
            w_directory: 25.0,
            w_symbol: 20.0,
            w_identifier: 15.0,
            w_content_token: 10.0,
            w_context: 8.0,
            w_trigram: 10.0,
            w_status: 15.0,
            w_completeness: 10.0,
            w_penalty: 25.0,
        }
    }

    /// Weights for filename with extension queries (e.g. "tron.ts").
    pub fn filename_with_extension() -> Self {
        Self {
            w_exact_path: 60.0,
            w_exact_filename: 80.0,
            w_exact_stem: 60.0,
            w_filename_prefix: 30.0,
            w_directory: 15.0,
            w_symbol: 25.0,
            w_identifier: 20.0,
            w_content_token: 10.0,
            w_context: 8.0,
            w_trigram: 10.0,
            w_status: 15.0,
            w_completeness: 10.0,
            w_penalty: 30.0,
        }
    }

    /// Weights for exact filename stem queries (e.g. "tron", "trigram").
    pub fn filename_stem() -> Self {
        Self {
            w_exact_path: 50.0,
            w_exact_filename: 55.0,
            w_exact_stem: 75.0,
            w_filename_prefix: 35.0,
            w_directory: 15.0,
            w_symbol: 35.0,
            w_identifier: 30.0,
            w_content_token: 15.0,
            w_context: 10.0,
            w_trigram: 12.0,
            w_status: 15.0,
            w_completeness: 10.0,
            w_penalty: 30.0,
        }
    }

    /// Weights for exact programmatic identifiers (e.g. "processTronPayment").
    pub fn exact_identifier() -> Self {
        Self {
            w_exact_path: 45.0,
            w_exact_filename: 50.0,
            w_exact_stem: 55.0,
            w_filename_prefix: 25.0,
            w_directory: 10.0,
            w_symbol: 70.0,
            w_identifier: 65.0,
            w_content_token: 20.0,
            w_context: 12.0,
            w_trigram: 15.0,
            w_status: 15.0,
            w_completeness: 10.0,
            w_penalty: 25.0,
        }
    }

    /// Weights for full-text / multi-term natural language queries (e.g. "payment", "authentication architecture").
    pub fn content_search() -> Self {
        Self {
            w_exact_path: 35.0,
            w_exact_filename: 35.0,
            w_exact_stem: 30.0,
            w_filename_prefix: 15.0,
            w_directory: 10.0,
            w_symbol: 25.0,
            w_identifier: 20.0,
            w_content_token: 35.0,
            w_context: 35.0,
            w_trigram: 30.0,
            w_status: 15.0,
            w_completeness: 10.0,
            w_penalty: 15.0,
        }
    }

    /// Weights for explicit stub / WIP search queries (e.g. "treesitter stub", "TODO treesitter").
    pub fn stub_search() -> Self {
        Self {
            w_exact_path: 50.0,
            w_exact_filename: 45.0,
            w_exact_stem: 45.0,
            w_filename_prefix: 25.0,
            w_directory: 15.0,
            w_symbol: 25.0,
            w_identifier: 20.0,
            w_content_token: 15.0,
            w_context: 20.0,
            w_trigram: 15.0,
            w_status: 60.0, // Substantial bonus for files matching stub status
            w_completeness: 0.0, // Completeness penalty disabled for stub searches
            w_penalty: 20.0,
        }
    }

    /// Select optimal weights based on explicit query intent and stub intent.
    pub fn for_intent(intent: &QueryIntent, is_stub_search: bool) -> Self {
        let mut weights = match intent {
            QueryIntent::SingleCharacterPrefix => Self::single_character_prefix(),
            QueryIntent::ShortTokenPrefix => Self::short_prefix(),
            QueryIntent::FilenameStem => Self::filename_stem(),
            QueryIntent::FilenameWithExtension => Self::filename_with_extension(),
            QueryIntent::ExactRelativePath => Self::exact_relative_path(),
            QueryIntent::ExactIdentifier | QueryIntent::SymbolLookup => Self::exact_identifier(),
            QueryIntent::ContentSearch => Self::content_search(),
            QueryIntent::ExplicitImplementationStatus => Self::stub_search(),
        };

        // If the query combines intent (e.g. exact path + stub search "blockchain/treesitter.rs stub"),
        // ensure stub status bonus is enabled while retaining the path weights!
        if is_stub_search {
            weights.w_status = 60.0;
            weights.w_completeness = 0.0;
        }

        weights
    }

    /// Classify query and return optimal weights.
    pub fn for_query(query: &str) -> Self {
        let (intent, is_stub_search, _) = intent_classifier::classify_query(query, |_| false);
        Self::for_intent(&intent, is_stub_search)
    }

    /// Classify query with candidate index context (e.g. knowing if a term matches an indexed stem).
    pub fn for_query_with_stem_lookup(query: &str, has_stem: impl Fn(&str) -> bool) -> Self {
        let (intent, is_stub_search, _) = intent_classifier::classify_query(query, has_stem);
        Self::for_intent(&intent, is_stub_search)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_language_query_not_classified_as_path() {
        let (intent_auth, is_stub_auth, _) = intent_classifier::classify_query("authentication", |_| false);
        assert_eq!(intent_auth, QueryIntent::ContentSearch);
        assert!(!is_stub_auth);

        let (intent_pay, is_stub_pay, _) = intent_classifier::classify_query("payment", |_| false);
        assert_eq!(intent_pay, QueryIntent::ContentSearch);
        assert!(!is_stub_pay);

        let weights = RankingWeights::for_query("authentication");
        assert_eq!(weights, RankingWeights::content_search());
    }

    #[test]
    fn test_exact_path_match_dominates_component_matches() {
        let weights = RankingWeights::exact_relative_path();
        let exact_path_doc = RankingFeatures {
            exact_relative_path_match: 1.0,
            exact_filename_match: 1.0,
            exact_filename_stem_match: 1.0,
            directory_component_match: 1.0,
            implementation_completeness: 1.0,
            implementation_status_match: 1.0,
            ..Default::default()
        };

        let component_only_doc = RankingFeatures {
            directory_component_match: 1.0,
            content_token_prefix_match: 1.0,
            contextual_relevance: 1.0,
            implementation_completeness: 1.0,
            implementation_status_match: 1.0,
            ..Default::default()
        };

        let exact_score = exact_path_doc.compute_score(&weights);
        let comp_score = component_only_doc.compute_score(&weights);
        assert!(exact_score > comp_score * 1.5, "Exact path ({}) must strongly dominate component-only ({})", exact_score, comp_score);
    }

    #[test]
    fn test_filename_extension_match() {
        let (intent, is_stub, clean) = intent_classifier::classify_query("tron.ts", |_| false);
        assert_eq!(intent, QueryIntent::FilenameWithExtension);
        assert!(!is_stub);
        assert_eq!(clean, "tron.ts");

        let weights = RankingWeights::for_intent(&intent, is_stub);
        let doc_with_ext = RankingFeatures {
            exact_filename_match: 1.0,
            exact_filename_stem_match: 1.0,
            implementation_completeness: 1.0,
            implementation_status_match: 1.0,
            ..Default::default()
        };
        let doc_diff_ext = RankingFeatures {
            exact_filename_match: 0.0,
            exact_filename_stem_match: 1.0,
            implementation_completeness: 1.0,
            implementation_status_match: 1.0,
            ..Default::default()
        };

        assert!(doc_with_ext.compute_score(&weights) > doc_diff_ext.compute_score(&weights));
    }

    #[test]
    fn test_windows_path_normalization() {
        let (intent1, _, _) = intent_classifier::classify_query("blockchain\\tron.ts", |_| false);
        assert_eq!(intent1, QueryIntent::ExactRelativePath);

        let (intent2, _, _) = intent_classifier::classify_query("./blockchain/tron.ts", |_| false);
        assert_eq!(intent2, QueryIntent::ExactRelativePath);

        assert_eq!(
            intent_classifier::normalize_path("blockchain\\tron.ts"),
            "blockchain/tron.ts"
        );
        assert_eq!(
            intent_classifier::normalize_path("./blockchain/tron.ts"),
            "blockchain/tron.ts"
        );
    }

    #[test]
    fn test_unicode_query_classification() {
        // Non-ASCII 1-character query
        let (intent_1, _, _) = intent_classifier::classify_query("⚡", |_| false);
        assert_eq!(intent_1, QueryIntent::SingleCharacterPrefix);

        // Non-ASCII 2-character query
        let (intent_2, _, _) = intent_classifier::classify_query("über", |_| false);
        assert_eq!(intent_2, QueryIntent::ContentSearch);
    }

    #[test]
    fn test_case_insensitive_extension_detection() {
        let (intent, _, _) = intent_classifier::classify_query("TRON.TS", |_| false);
        assert_eq!(intent, QueryIntent::FilenameWithExtension);

        let (intent_json, _, _) = intent_classifier::classify_query("Schema.JSON", |_| false);
        assert_eq!(intent_json, QueryIntent::FilenameWithExtension);
    }

    #[test]
    fn test_stub_status_is_separate_from_completeness() {
        let mut stub_doc = RankingFeatures {
            exact_filename_stem_match: 1.0,
            implementation_completeness: 0.05, // Intrinsic: file is a stub
            implementation_status_match: 0.05, // Normal search: stub does NOT match intent
            ..Default::default()
        };

        let normal_weights = RankingWeights::short_prefix();
        let normal_score = stub_doc.compute_score(&normal_weights);

        // When user asks for stubs:
        let stub_weights = RankingWeights::stub_search();
        stub_doc.implementation_status_match = 1.0; // Stub query: stub matches requested intent!
        // Notice completeness stays truthfully at 0.05:
        assert_eq!(stub_doc.implementation_completeness, 0.05);

        let stub_score = stub_doc.compute_score(&stub_weights);
        assert!(
            stub_score > normal_score,
            "Stub doc score during stub search ({}) must be higher than during normal search ({})",
            stub_score, normal_score
        );
    }

    #[test]
    fn test_stub_intent_token_boundaries() {
        // Words with 'stub' or 'todo' inside them should NOT trigger stub search
        assert!(!intent_classifier::detect_stub_intent("student"));
        assert!(!intent_classifier::detect_stub_intent("methodology"));
        assert!(!intent_classifier::detect_stub_intent("stubborn"));

        // Valid stub tokens must trigger stub search
        assert!(intent_classifier::detect_stub_intent("treesitter stub"));
        assert!(intent_classifier::detect_stub_intent("TODO treesitter"));
        assert!(intent_classifier::detect_stub_intent("treesitter todo!()"));
        assert!(intent_classifier::detect_stub_intent("wip feature"));
    }

    #[test]
    fn test_explanation_uses_weighted_contribution() {
        let weights = RankingWeights::exact_relative_path();
        let doc = RankingFeatures {
            exact_relative_path_match: 1.0,
            exact_filename_match: 1.0,
            exact_filename_stem_match: 1.0,
            filename_prefix_match: 1.0,
            ..Default::default()
        };

        let explanation = doc.explain(&weights, &QueryIntent::ExactRelativePath, false);
        assert_eq!(explanation.dominant_signal, "exact_relative_path");
        assert_eq!(explanation.dominant_contribution, 85.0);
        assert_eq!(explanation.summary, "Exact relative path match");
    }

    #[test]
    fn test_scores_are_finite_and_bounded() {
        let mut malformed_doc = RankingFeatures {
            exact_relative_path_match: f32::NAN,
            exact_filename_match: f32::INFINITY,
            exact_filename_stem_match: -5.0,
            filename_prefix_match: 25.0, // Out of [0..1] range
            ..Default::default()
        };

        malformed_doc.sanitize();
        assert!(malformed_doc.exact_relative_path_match.is_finite());
        assert_eq!(malformed_doc.exact_relative_path_match, 0.0);
        assert_eq!(malformed_doc.exact_filename_match, 0.0);
        assert_eq!(malformed_doc.exact_filename_stem_match, 0.0);
        assert_eq!(malformed_doc.filename_prefix_match, 1.0);

        let weights = RankingWeights::short_prefix();
        let score = malformed_doc.compute_score(&weights);
        assert!(score.is_finite());
        let norm = malformed_doc.compute_normalized_score(&weights);
        assert!(norm >= 0.05 && norm <= 0.99);
    }

    #[test]
    fn test_exact_path_does_not_double_count_all_signals() {
        let weights = RankingWeights::exact_relative_path();
        let exact_doc = RankingFeatures {
            exact_relative_path_match: 1.0,
            exact_filename_match: 1.0,
            exact_filename_stem_match: 1.0,
            filename_prefix_match: 1.0,
            directory_component_match: 1.0,
            ..Default::default()
        };

        let score = exact_doc.compute_score(&weights);
        // Pure additive sum would be 85 + 50 + 40 + 20 + 25 = 220
        // Hierarchical score caps subordinate signals to small residual (< 100)
        assert!(score < 130.0, "Hierarchical scoring should avoid runaway accumulation: score is {}", score);
        assert!(score >= 85.0, "Exact path base weight must be preserved");
    }

    #[test]
    fn test_empty_query_classification() {
        let (intent, is_stub, clean) = intent_classifier::classify_query("", |_| false);
        assert_eq!(intent, QueryIntent::ContentSearch);
        assert!(!is_stub);
        assert_eq!(clean, "");
    }

    #[test]
    fn test_legacy_feature_migration() {
        let mut features = RankingFeatures {
            implementation_status_match: 0.85,
            token_prefix: 0.9,
            exact_identifier: 1.0,
            filename_prefix: 0.95,
            symbol_match: 0.8,
            ..Default::default()
        };
        features.sanitize();
        assert_eq!(features.implementation_quality, 0.85);
        assert_eq!(features.token_prefix, 0.9);
    }

    #[test]
    fn test_combined_path_and_stub_query() {
        let (intent, is_stub, clean) = intent_classifier::classify_query("blockchain/treesitter.rs stub", |_| false);
        assert_eq!(intent, QueryIntent::ExactRelativePath);
        assert!(is_stub);
        assert_eq!(clean, "blockchain/treesitter.rs");

        let weights = RankingWeights::for_intent(&intent, is_stub);
        // Preserves exact path weight AND stub bonus
        assert_eq!(weights.w_exact_path, 85.0);
        assert_eq!(weights.w_status, 60.0);
    }
}
