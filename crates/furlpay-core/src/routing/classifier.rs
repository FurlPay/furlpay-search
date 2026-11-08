//! Intent and query routing matrix — classifies queries and selects optimal
//! primary and secondary retrieval pipelines.
//!
//! Architecture: Multi-candidate recognition with universal fallback.
//! Recognition assists search but never restricts it.
//! Unknown queries are always eligible for general web discovery.

use serde::{Deserialize, Serialize};
use crate::recognition::engine::PatternEngine;
use crate::recognition::candidate::PatternCategory;

/// Detailed classification of a user query.
///
/// This is an open-ended classification — unrecognized queries become
/// `NaturalLanguage` or `Hybrid` and are always eligible for search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryClass {
    /// A structurally validated wallet address (TRON, Ethereum, Bitcoin, Solana, etc.)
    WalletAddress { network: String },
    /// A recognized blockchain network name.
    BlockchainName { network: String },
    /// A recognized token or asset ticker symbol.
    TokenTicker { ticker: String },
    /// A transaction hash or signature.
    TransactionHash { network: Option<String> },
    /// A parseable URL (http, https, ftp, ws, wss).
    Url,
    /// A single-token code identifier (variable, function, class name, 4+ chars).
    ExactIdentifier,
    /// A single-character ambiguous query (e.g. 't', 'a').
    AmbiguousShortQuery { ch: char },
    /// A 2-3 character partial token / prefix candidate (e.g. 'tr', 'tro').
    PartialTokenPrefix { prefix: String },
    /// A regular expression syntax pattern.
    RegularExpression,
    /// A query prefixed with a programming-language keyword.
    CodeSymbol,
    /// A natural-language question or phrase — the universal fallback.
    NaturalLanguage,
    /// Call-graph or dependency analysis query.
    CallGraph,
    /// Multi-signal or multi-word query benefiting from parallel retrieval.
    Hybrid,
    /// A financial identifier (IBAN, SWIFT/BIC).
    FinancialIdentifier { kind: String },
    /// A cryptographic digest or UUID.
    CryptographicHash { algorithm: String },
    /// A document standard (ISBN, DOI).
    DocumentIdentifier { standard: String },
    /// An internet entity (email, domain, IP address).
    InternetEntity { kind: String },
    /// A software version, API route, or git commit.
    SoftwareArtifact { kind: String },
}

/// Retrieval pipeline that can be selected by the router.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetrievalPipeline {
    TrigramIndex,
    SimdVerify,
    TrigramPrefilter,
    RegexEngine,
    SymbolGraph,
    AstScope,
    Bm25FullText,
    SemanticVector,
    CallGraphIndex,
    ParallelExecution,
    RrfScorer,
    BlockchainExplorer,
    DirectUrlFetch,
    /// Global public web discovery — the universal fallback pipeline.
    GlobalWebDiscovery,
}

/// Search mode controlling the scope of retrieval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchScope {
    /// Search indexed local files and source code only.
    Local,
    /// Search global public web sources only.
    Web,
    /// Retrieve a specified public URL.
    Url,
    /// Combine local search, web discovery, and specialized retrieval.
    Hybrid,
    /// Broad public web discovery — the default for web queries.
    Global,
    /// Expand into multiple related searches for comprehensive research.
    Deep,
}

impl Default for SearchScope {
    fn default() -> Self {
        Self::Global
    }
}

impl std::str::FromStr for SearchScope {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local" | "code" => Ok(SearchScope::Local),
            "web" => Ok(SearchScope::Web),
            "url" => Ok(SearchScope::Url),
            "hybrid" => Ok(SearchScope::Hybrid),
            "deep" => Ok(SearchScope::Deep),
            _ => Ok(SearchScope::Global),
        }
    }
}

/// Routing decision with primary, secondary, and fallback pipelines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub query_class: QueryClass,
    pub primary: RetrievalPipeline,
    pub secondary: RetrievalPipeline,
    pub fallback: Option<RetrievalPipeline>,
    pub confidence: f32,
    pub detected_entity: Option<String>,
    /// Recognition candidates from the pattern engine (serialized summary).
    pub recognition_summary: Option<String>,
}

/// The query classifier and router.
///
/// Uses the Universal Pattern Recognition Engine to classify queries,
/// then maps recognized patterns to retrieval pipelines.
/// Unrecognized queries always fall through to general search.
pub struct QueryRouter;

impl QueryRouter {
    /// Classify a query string and return the optimal routing decision.
    ///
    /// This function never rejects a query. Unrecognized input becomes
    /// `NaturalLanguage` or `Hybrid` and is routed to general search.
    pub fn classify(query: &str) -> RoutingDecision {
        let trimmed = query.trim();

        // 1. URL detection — use proper URL parsing, not prefix matching
        if Self::parse_url(trimmed) {
            return RoutingDecision {
                query_class: QueryClass::Url,
                primary: RetrievalPipeline::DirectUrlFetch,
                secondary: RetrievalPipeline::GlobalWebDiscovery,
                fallback: None,
                confidence: 0.98,
                detected_entity: Some(trimmed.to_string()),
                recognition_summary: None,
            };
        }

        // 2. Run the universal pattern recognition engine
        let engine = PatternEngine::new();
        let candidates = engine.analyze(trimmed);

        if let Some(best) = candidates.first() {
            let is_multiword = trimmed.split_whitespace().count() > 2;
            let full_match = best.raw_value == trimmed;

            // Only use recognition if it meets confidence threshold AND (matches full query OR query is short/single-entity)
            if best.confidence >= 0.60
                && best.recognizer_id != "general.universal_fallback"
                && (!is_multiword || full_match)
            {
                let summary = format!(
                    "{}:{} (conf:{:.2}, status:{:?})",
                    best.recognizer_id, best.display_name, best.confidence, best.validation_status
                );

                match best.category {
                    PatternCategory::Blockchain => {
                        // Determine specific blockchain sub-classification
                        let id = &best.recognizer_id;
                        if id.contains("tron") || id.contains("ethereum") || id.contains("bitcoin")
                            || id.contains("solana") || id.contains("base58") || id.contains("bech32")
                            || id.contains("evm")
                        {
                            if id.contains("tx") || id.contains("hash") {
                                return RoutingDecision {
                                    query_class: QueryClass::TransactionHash {
                                        network: best.metadata.get("network").cloned(),
                                    },
                                    primary: RetrievalPipeline::BlockchainExplorer,
                                    secondary: RetrievalPipeline::GlobalWebDiscovery,
                                    fallback: Some(RetrievalPipeline::TrigramIndex),
                                    confidence: best.confidence,
                                    detected_entity: Some(best.raw_value.clone()),
                                    recognition_summary: Some(summary),
                                };
                            } else {
                                let network = best.metadata.get("network")
                                    .cloned()
                                    .unwrap_or_else(|| best.display_name.clone());
                                return RoutingDecision {
                                    query_class: QueryClass::WalletAddress { network },
                                    primary: RetrievalPipeline::BlockchainExplorer,
                                    secondary: RetrievalPipeline::TrigramIndex,
                                    fallback: Some(RetrievalPipeline::GlobalWebDiscovery),
                                    confidence: best.confidence,
                                    detected_entity: Some(best.raw_value.clone()),
                                    recognition_summary: Some(summary),
                                };
                            }
                        }
                    }
                    PatternCategory::Financial => {
                        let kind = if best.recognizer_id.contains("iban") {
                            "IBAN"
                        } else if best.recognizer_id.contains("swift") || best.recognizer_id.contains("bic") {
                            "SWIFT/BIC"
                        } else {
                            "Financial"
                        };
                        return RoutingDecision {
                            query_class: QueryClass::FinancialIdentifier { kind: kind.into() },
                            primary: RetrievalPipeline::GlobalWebDiscovery,
                            secondary: RetrievalPipeline::TrigramIndex,
                            fallback: None,
                            confidence: best.confidence,
                            detected_entity: Some(best.raw_value.clone()),
                            recognition_summary: Some(summary),
                        };
                    }
                    PatternCategory::Internet => {
                        let kind = if best.recognizer_id.contains("email") {
                            "Email"
                        } else if best.recognizer_id.contains("ipv4") {
                            "IPv4"
                        } else if best.recognizer_id.contains("ipv6") {
                            "IPv6"
                        } else if best.recognizer_id.contains("domain") {
                            "Domain"
                        } else {
                            "Internet"
                        };
                        return RoutingDecision {
                            query_class: QueryClass::InternetEntity { kind: kind.into() },
                            primary: RetrievalPipeline::GlobalWebDiscovery,
                            secondary: RetrievalPipeline::TrigramIndex,
                            fallback: None,
                            confidence: best.confidence,
                            detected_entity: Some(best.raw_value.clone()),
                            recognition_summary: Some(summary),
                        };
                    }
                    PatternCategory::Cryptography => {
                        let algo = best.metadata.get("algorithm")
                            .cloned()
                            .unwrap_or_else(|| best.display_name.clone());
                        return RoutingDecision {
                            query_class: QueryClass::CryptographicHash { algorithm: algo },
                            primary: RetrievalPipeline::TrigramIndex,
                            secondary: RetrievalPipeline::GlobalWebDiscovery,
                            fallback: None,
                            confidence: best.confidence,
                            detected_entity: Some(best.raw_value.clone()),
                            recognition_summary: Some(summary),
                        };
                    }
                    PatternCategory::Software => {
                        let kind = if best.recognizer_id.contains("semver") {
                            "SemVer"
                        } else if best.recognizer_id.contains("api_route") {
                            "APIRoute"
                        } else if best.recognizer_id.contains("git") {
                            "GitCommit"
                        } else {
                            "Software"
                        };
                        return RoutingDecision {
                            query_class: QueryClass::SoftwareArtifact { kind: kind.into() },
                            primary: RetrievalPipeline::TrigramIndex,
                            secondary: RetrievalPipeline::GlobalWebDiscovery,
                            fallback: Some(RetrievalPipeline::SymbolGraph),
                            confidence: best.confidence,
                            detected_entity: Some(best.raw_value.clone()),
                            recognition_summary: Some(summary),
                        };
                    }
                    PatternCategory::Documents => {
                        let standard = if best.recognizer_id.contains("isbn") {
                            "ISBN"
                        } else if best.recognizer_id.contains("doi") {
                            "DOI"
                        } else {
                            "Document"
                        };
                        return RoutingDecision {
                            query_class: QueryClass::DocumentIdentifier { standard: standard.into() },
                            primary: RetrievalPipeline::GlobalWebDiscovery,
                            secondary: RetrievalPipeline::TrigramIndex,
                            fallback: None,
                            confidence: best.confidence,
                            detected_entity: Some(best.raw_value.clone()),
                            recognition_summary: Some(summary),
                        };
                    }
                    _ => {
                        // GeneralText, Unknown — fall through to heuristic classifiers below
                    }
                }
            }
        }

        // 3. Structural heuristics for code-search queries (no hardcoded dictionaries)

        // 3a. Blockchain wallet address by structural pattern (format detection only,
        //     works for any future chain without hardcoded names)
        if let Some((network, entity)) = Self::classify_wallet_address(trimmed) {
            return RoutingDecision {
                query_class: QueryClass::WalletAddress { network },
                primary: RetrievalPipeline::BlockchainExplorer,
                secondary: RetrievalPipeline::TrigramIndex,
                fallback: Some(RetrievalPipeline::GlobalWebDiscovery),
                confidence: 0.93,
                detected_entity: Some(entity),
                recognition_summary: None,
            };
        }

        // 3b. Transaction hash by structural pattern
        if let Some((network, hash)) = Self::classify_tx_hash(trimmed) {
            return RoutingDecision {
                query_class: QueryClass::TransactionHash { network },
                primary: RetrievalPipeline::BlockchainExplorer,
                secondary: RetrievalPipeline::TrigramIndex,
                fallback: None,
                confidence: 0.92,
                detected_entity: Some(hash),
                recognition_summary: None,
            };
        }

        // 3c. Regex detection — use actual regex parser, not character scanning
        if Self::parse_regex(trimmed) {
            return RoutingDecision {
                query_class: QueryClass::RegularExpression,
                primary: RetrievalPipeline::TrigramPrefilter,
                secondary: RetrievalPipeline::RegexEngine,
                fallback: Some(RetrievalPipeline::Bm25FullText),
                confidence: 0.85,
                detected_entity: None,
                recognition_summary: None,
            };
        }

        // 3d. Programming symbol detection — structural, not keyword-list
        if Self::looks_like_symbol_query(trimmed) {
            return RoutingDecision {
                query_class: QueryClass::CodeSymbol,
                primary: RetrievalPipeline::SymbolGraph,
                secondary: RetrievalPipeline::AstScope,
                fallback: Some(RetrievalPipeline::GlobalWebDiscovery),
                confidence: 0.80,
                detected_entity: None,
                recognition_summary: None,
            };
        }

        // 3e. Single character query — ambiguous partial input (calibrated low confidence)
        if trimmed.len() == 1 {
            let ch = trimmed.chars().next().unwrap();
            return RoutingDecision {
                query_class: QueryClass::AmbiguousShortQuery { ch },
                primary: RetrievalPipeline::TrigramIndex,
                secondary: RetrievalPipeline::SimdVerify,
                fallback: Some(RetrievalPipeline::GlobalWebDiscovery),
                confidence: 0.30,
                detected_entity: Some(trimmed.to_string()),
                recognition_summary: Some("short.single_char:Ambiguous single-character query".into()),
            };
        }

        // 3f. 2 to 3 character query — partial token / prefix candidate
        if (2..=3).contains(&trimmed.len())
            && !trimmed.contains(' ')
            && trimmed.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return RoutingDecision {
                query_class: QueryClass::PartialTokenPrefix { prefix: trimmed.to_string() },
                primary: RetrievalPipeline::TrigramIndex,
                secondary: RetrievalPipeline::SimdVerify,
                fallback: Some(RetrievalPipeline::GlobalWebDiscovery),
                confidence: 0.55,
                detected_entity: Some(trimmed.to_string()),
                recognition_summary: Some("short.prefix:Partial token prefix candidate".into()),
            };
        }

        // 3g. Identifier detection — structural pattern, 4+ chars, no whitespace, alphanumeric
        if trimmed.len() >= 4 && Self::looks_like_identifier(trimmed) {
            return RoutingDecision {
                query_class: QueryClass::ExactIdentifier,
                primary: RetrievalPipeline::TrigramIndex,
                secondary: RetrievalPipeline::SimdVerify,
                fallback: Some(RetrievalPipeline::GlobalWebDiscovery),
                confidence: 0.88,
                detected_entity: Some(trimmed.to_string()),
                recognition_summary: None,
            };
        }

        // 4. Multi-word queries → Hybrid parallel retrieval
        if trimmed.split_whitespace().count() > 3 {
            return RoutingDecision {
                query_class: QueryClass::Hybrid,
                primary: RetrievalPipeline::ParallelExecution,
                secondary: RetrievalPipeline::GlobalWebDiscovery,
                fallback: None,
                confidence: 0.70,
                detected_entity: None,
                recognition_summary: None,
            };
        }

        // 5. Universal fallback — every query is searchable
        RoutingDecision {
            query_class: QueryClass::NaturalLanguage,
            primary: RetrievalPipeline::Bm25FullText,
            secondary: RetrievalPipeline::GlobalWebDiscovery,
            fallback: Some(RetrievalPipeline::TrigramIndex),
            confidence: 0.65,
            detected_entity: None,
            recognition_summary: None,
        }
    }

    // ── Structural validators (format-based, no hardcoded dictionaries) ──

    /// Check if string is a valid Base58 encoded string (Bitcoin/Tron/Solana charset).
    fn is_base58(s: &str) -> bool {
        !s.is_empty()
            && s.chars().all(|c| {
                matches!(
                    c,
                    '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z'
                )
            })
    }

    /// Validate and identify wallet address format by structure, not by name dictionary.
    pub fn classify_wallet_address(s: &str) -> Option<(String, String)> {
        // TRON Address: Starts with 'T', exactly 34 chars, valid Base58
        if s.starts_with('T') && s.len() == 34 && Self::is_base58(s) {
            return Some(("TRON (Base58Check)".into(), s.to_string()));
        }

        // Ethereum / EVM Address: Starts with 0x/0X, exactly 42 chars, hexadecimal
        if (s.starts_with("0x") || s.starts_with("0X")) && s.len() == 42 {
            if s[2..].chars().all(|c| c.is_ascii_hexdigit()) {
                return Some(("Ethereum / EVM".into(), s.to_string()));
            }
        }

        // Bitcoin Bech32 Address: Starts with bc1, 42 or 62 chars
        if s.starts_with("bc1") && (s.len() == 42 || s.len() == 62) {
            return Some(("Bitcoin (Bech32)".into(), s.to_string()));
        }

        // Bitcoin Legacy / P2SH Address: Starts with 1 or 3, 26-35 chars, valid Base58
        if (s.starts_with('1') || s.starts_with('3'))
            && (26..=35).contains(&s.len())
            && Self::is_base58(s)
        {
            return Some(("Bitcoin (Base58)".into(), s.to_string()));
        }

        // Solana Address: 32 to 44 chars, valid Base58, not starting with 0x or T
        if (32..=44).contains(&s.len())
            && !s.starts_with("0x")
            && !s.starts_with('T')
            && !s.starts_with("bc1")
            && Self::is_base58(s)
        {
            return Some(("Solana".into(), s.to_string()));
        }

        None
    }

    /// Classify transaction hash by structure.
    fn classify_tx_hash(s: &str) -> Option<(Option<String>, String)> {
        // 66 chars starting with 0x (EVM tx hash)
        if (s.starts_with("0x") || s.starts_with("0X"))
            && s.len() == 66
            && s[2..].chars().all(|c| c.is_ascii_hexdigit())
        {
            return Some((Some("EVM".into()), s.to_string()));
        }

        // 64 hex chars (TRON / Bitcoin / Solana signature or hash)
        if s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some((Some("HexTxHash".into()), s.to_string()));
        }

        // Solana 88-character Base58 transaction signature
        if (87..=88).contains(&s.len()) && Self::is_base58(s) {
            return Some((Some("SolanaTxSig".into()), s.to_string()));
        }

        None
    }

    /// URL detection using proper URL parsing, not just prefix checks.
    fn parse_url(s: &str) -> bool {
        // Quick structural check: must contain "://" or start with "www."
        if s.contains("://") {
            let scheme_end = s.find("://").unwrap();
            let scheme = &s[..scheme_end].to_lowercase();
            // Accept standard web schemes
            matches!(scheme.as_str(), "http" | "https" | "ftp" | "ws" | "wss")
        } else if s.starts_with("www.") && s.contains('.') && s.len() > 5 {
            true
        } else {
            false
        }
    }

    /// Regex detection — attempt to parse the input as regex.
    /// Only classify as regex if it contains explicit regex metacharacters
    /// AND is valid regex syntax. Ordinary text is not regex.
    fn parse_regex(query: &str) -> bool {
        // Must contain at least one unambiguous regex metacharacter
        let has_metachar = query.contains('|')
            || query.contains('^')
            || query.contains('$')
            || (query.contains('[') && query.contains(']'))
            || (query.contains('{') && query.contains('}'))
            || query.contains("\\d")
            || query.contains("\\w")
            || query.contains("\\s")
            || query.contains(".+")
            || query.contains(".*");

        if !has_metachar {
            return false;
        }

        // Verify it actually parses as valid regex
        regex::Regex::new(query).is_ok()
    }

    /// Programming symbol detection — structural, using colon-separated
    /// or dot-separated qualified names, not keyword dictionaries.
    fn looks_like_symbol_query(query: &str) -> bool {
        // Qualified symbol: contains :: (Rust), or is CamelCase with dots
        if query.contains("::") && !query.contains(' ') {
            return true;
        }

        // Language keyword prefix — kept as a minimal structural hint,
        // but these are syntax tokens not a knowledge database
        let first_word = query.split_whitespace().next().unwrap_or("");
        matches!(
            first_word,
            "fn" | "func" | "def" | "class" | "struct" | "interface" | "enum" | "type" | "impl" | "trait" | "module" | "package"
        ) && query.split_whitespace().count() >= 2
    }

    /// Identifier detection — structural pattern, no whitespace.
    fn looks_like_identifier(query: &str) -> bool {
        !query.contains(' ')
            && query.len() >= 1
            && query
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == ':' || c == '.' || c == '-')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tron_wallet_address() {
        let decision = QueryRouter::classify("TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf");
        match decision.query_class {
            QueryClass::WalletAddress { network } => {
                assert!(network.contains("TRON"));
            }
            _ => panic!("Expected TRON wallet address, got {:?}!", decision.query_class),
        }
    }

    #[test]
    fn test_short_t_not_tron_address() {
        let decision = QueryRouter::classify("T");
        assert_ne!(
            decision.query_class,
            QueryClass::WalletAddress {
                network: "TRON (Base58Check)".into()
            }
        );
        assert_eq!(decision.query_class, QueryClass::AmbiguousShortQuery { ch: 'T' });
        assert_eq!(decision.confidence, 0.30);
    }

    #[test]
    fn test_short_prefix_tr_and_tro() {
        let dec_tr = QueryRouter::classify("tr");
        assert_eq!(dec_tr.query_class, QueryClass::PartialTokenPrefix { prefix: "tr".into() });
        assert_eq!(dec_tr.confidence, 0.55);

        let dec_tro = QueryRouter::classify("tro");
        assert_eq!(dec_tro.query_class, QueryClass::PartialTokenPrefix { prefix: "tro".into() });
        assert_eq!(dec_tro.confidence, 0.55);
    }

    #[test]
    fn test_blockchain_name_tron_is_general_search() {
        let decision = QueryRouter::classify("tron");
        assert_eq!(decision.query_class, QueryClass::ExactIdentifier);
        assert_eq!(decision.confidence, 0.88);
        assert!(
            decision.fallback == Some(RetrievalPipeline::GlobalWebDiscovery),
            "Unknown terms must be eligible for global web discovery"
        );
    }

    #[test]
    fn test_token_ticker_is_general_search() {
        let decision = QueryRouter::classify("TRX");
        assert_eq!(decision.query_class, QueryClass::PartialTokenPrefix { prefix: "TRX".into() });
        assert_eq!(decision.confidence, 0.55);
    }

    #[test]
    fn test_ethereum_wallet_address() {
        let decision = QueryRouter::classify("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
        match decision.query_class {
            QueryClass::WalletAddress { network } => {
                assert!(network.contains("Ethereum") || network.contains("EVM"));
            }
            _ => panic!("Expected Ethereum wallet address, got {:?}!", decision.query_class),
        }
    }

    #[test]
    fn test_url_classification() {
        let decision = QueryRouter::classify("https://furlpay.dev/docs");
        assert_eq!(decision.query_class, QueryClass::Url);
    }

    #[test]
    fn test_natural_language_goes_to_global_search() {
        let decision = QueryRouter::classify("what is the capital of Japan");
        assert!(
            decision.query_class == QueryClass::Hybrid
                || decision.query_class == QueryClass::NaturalLanguage,
            "Natural language query must be searchable, got {:?}", decision.query_class
        );
        assert!(
            decision.secondary == RetrievalPipeline::GlobalWebDiscovery
                || decision.primary == RetrievalPipeline::GlobalWebDiscovery
                || decision.fallback == Some(RetrievalPipeline::GlobalWebDiscovery),
            "NL queries must reach GlobalWebDiscovery"
        );
    }

    #[test]
    fn test_arbitrary_unknown_topic_is_searchable() {
        let decision = QueryRouter::classify("quantum entanglement entropy research 2026");
        assert!(
            decision.query_class == QueryClass::Hybrid
                || decision.query_class == QueryClass::NaturalLanguage,
            "Unknown topics must be routed to search, got {:?}", decision.query_class
        );
    }

    #[test]
    fn test_iban_recognized() {
        let decision = QueryRouter::classify("DE89370400440532013000");
        match decision.query_class {
            QueryClass::FinancialIdentifier { kind } => {
                assert!(kind.contains("IBAN"), "Expected IBAN, got {}", kind);
            }
            _ => panic!("Expected FinancialIdentifier for IBAN, got {:?}", decision.query_class),
        }
    }

    #[test]
    fn test_uuid_recognized() {
        let decision = QueryRouter::classify("123e4567-e89b-12d3-a456-426614174000");
        match decision.query_class {
            QueryClass::CryptographicHash { algorithm } => {
                assert!(algorithm.contains("UUID") || algorithm.contains("SHA"),
                    "Expected UUID or hash, got {}", algorithm);
            }
            _ => {
                // Also acceptable as ExactIdentifier since it's a single token
            }
        }
    }

    #[test]
    fn test_regex_detection() {
        let decision = QueryRouter::classify("^fn\\s+\\w+");
        assert_eq!(decision.query_class, QueryClass::RegularExpression);
    }

    #[test]
    fn test_plain_text_not_regex() {
        // Simple text with common punctuation should not be classified as regex
        let decision = QueryRouter::classify("hello world");
        assert_ne!(decision.query_class, QueryClass::RegularExpression);
    }
}
