//! Query planning, intent analysis, and search strategy generation.
//!
//! The query planner interprets user intent and produces a concrete
//! execution plan: which providers to query, whether to browse,
//! how deep to crawl, and what content to extract.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A concrete search execution plan produced by the query planner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPlan {
    /// Original user query.
    pub original_query: String,
    /// Normalized / cleaned query.
    pub normalized_query: String,
    /// Detected search mode.
    pub mode: PlanMode,
    /// Discovery queries to execute across search providers.
    pub discovery_queries: Vec<DiscoveryQuery>,
    /// Direct URLs to fetch (if the query contains or implies URLs).
    pub direct_urls: Vec<String>,
    /// Whether to enable browser rendering for JS-heavy pages.
    pub browser_rendering: bool,
    /// Crawl configuration (None = no crawling).
    pub crawl_config: Option<CrawlPlanConfig>,
    /// Maximum total results to return.
    pub max_results: usize,
    /// Time budget for the entire plan execution.
    pub time_budget: Duration,
    /// Recognized entities in the query.
    pub entities: Vec<RecognizedEntity>,
}

/// Search mode the planner decided on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanMode {
    /// Search local indexed code only.
    Local,
    /// Discover public web results.
    Global,
    /// Open and navigate a specific URL or website.
    Browse,
    /// Fetch and inspect a single URL.
    DirectUrl,
    /// Multi-source exploration with evidence synthesis.
    DeepResearch,
    /// Combine local and web search.
    Hybrid,
}

impl std::fmt::Display for PlanMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanMode::Local => write!(f, "local"),
            PlanMode::Global => write!(f, "global"),
            PlanMode::Browse => write!(f, "browse"),
            PlanMode::DirectUrl => write!(f, "direct_url"),
            PlanMode::DeepResearch => write!(f, "deep_research"),
            PlanMode::Hybrid => write!(f, "hybrid"),
        }
    }
}

/// A single discovery query to send to search providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryQuery {
    pub query_text: String,
    pub priority: QueryPriority,
    /// Which provider categories to use (empty = all).
    pub provider_filter: Vec<String>,
    pub max_results: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryPriority {
    Primary,
    Expansion,
    Refinement,
}

/// Crawl configuration embedded in a search plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlPlanConfig {
    pub max_depth: u32,
    pub max_pages: usize,
    pub max_domains: usize,
    pub same_domain_only: bool,
    pub time_budget: Duration,
    pub seed_urls: Vec<String>,
}

/// An entity recognized in the query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognizedEntity {
    pub text: String,
    pub entity_type: EntityType,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Url,
    Domain,
    Organization,
    Technology,
    Person,
    Location,
    Identifier,
    Unknown,
}

pub struct QueryPlanner;

impl QueryPlanner {
    /// Analyze a query string and mode hint to produce a search plan.
    pub fn plan(query: &str, mode_hint: &str) -> SearchPlan {
        let trimmed = query.trim();
        let normalized = Self::normalize_query(trimmed);

        // Detect entities
        let entities = Self::extract_entities(trimmed);

        // Detect URLs in query
        let direct_urls = Self::extract_urls(trimmed);

        // Determine mode
        let mode = Self::resolve_mode(mode_hint, trimmed, &entities, &direct_urls);

        // Build discovery queries
        let discovery_queries = Self::build_discovery_queries(trimmed, &entities, mode);

        // Determine crawl config
        let crawl_config = match mode {
            PlanMode::DeepResearch => Some(CrawlPlanConfig {
                max_depth: 2,
                max_pages: 20,
                max_domains: 5,
                same_domain_only: false,
                time_budget: Duration::from_secs(30),
                seed_urls: direct_urls.clone(),
            }),
            PlanMode::Browse => Some(CrawlPlanConfig {
                max_depth: 1,
                max_pages: 5,
                max_domains: 1,
                same_domain_only: true,
                time_budget: Duration::from_secs(15),
                seed_urls: direct_urls.clone(),
            }),
            _ => None,
        };

        let time_budget = match mode {
            PlanMode::Local => Duration::from_secs(2),
            PlanMode::Global => Duration::from_secs(10),
            PlanMode::Browse => Duration::from_secs(20),
            PlanMode::DirectUrl => Duration::from_secs(10),
            PlanMode::DeepResearch => Duration::from_secs(60),
            PlanMode::Hybrid => Duration::from_secs(10),
        };

        SearchPlan {
            original_query: trimmed.to_string(),
            normalized_query: normalized,
            mode,
            discovery_queries,
            direct_urls,
            browser_rendering: matches!(mode, PlanMode::Browse | PlanMode::DeepResearch),
            crawl_config,
            max_results: if mode == PlanMode::DeepResearch { 30 } else { 15 },
            time_budget,
            entities,
        }
    }

    fn normalize_query(query: &str) -> String {
        // Collapse whitespace, trim, lowercase for matching
        query
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn resolve_mode(
        hint: &str,
        query: &str,
        entities: &[RecognizedEntity],
        urls: &[String],
    ) -> PlanMode {
        // Explicit mode hint takes priority
        match hint.to_lowercase().as_str() {
            "local" | "code" => return PlanMode::Local,
            "browse" => return PlanMode::Browse,
            "deep" | "deep_research" | "research" => return PlanMode::DeepResearch,
            "url" | "direct_url" => return PlanMode::DirectUrl,
            "hybrid" => return PlanMode::Hybrid,
            "global" | "web" => {} // fall through to smart detection
            _ => {}
        }

        // If query is a single URL, use DirectUrl
        if urls.len() == 1 && query.trim() == urls[0] {
            return PlanMode::DirectUrl;
        }

        // If query starts with "research", "explore", "investigate", "compare" → deep
        let lower = query.to_lowercase();
        let research_prefixes = [
            "research ", "explore ", "investigate ", "compare ",
            "analyze ", "find all ", "comprehensive ", "deep dive ",
        ];
        if research_prefixes.iter().any(|p| lower.starts_with(p)) {
            return PlanMode::DeepResearch;
        }

        // If query starts with "open ", "browse ", "navigate to ", "go to " → browse
        let browse_prefixes = ["open ", "browse ", "navigate to ", "go to ", "visit "];
        if browse_prefixes.iter().any(|p| lower.starts_with(p)) {
            return PlanMode::Browse;
        }

        // If entities include a URL → DirectUrl or Browse
        if entities.iter().any(|e| e.entity_type == EntityType::Url) {
            return if !urls.is_empty() { PlanMode::DirectUrl } else { PlanMode::Browse };
        }

        // Default: global web discovery
        PlanMode::Global
    }

    fn extract_urls(query: &str) -> Vec<String> {
        let mut urls = Vec::new();
        for word in query.split_whitespace() {
            let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != ':' && c != '/' && c != '.' && c != '-' && c != '_' && c != '?' && c != '&' && c != '=' && c != '%' && c != '#' && c != '@');
            if (cleaned.starts_with("http://") || cleaned.starts_with("https://"))
                && cleaned.len() > 10
            {
                urls.push(cleaned.to_string());
            } else if cleaned.contains('.') && !cleaned.contains(' ') && cleaned.len() > 4 {
                // Bare domain detection (e.g. "github.com")
                let parts: Vec<&str> = cleaned.split('.').collect();
                if parts.len() >= 2 && parts.last().map_or(false, |tld| tld.len() >= 2 && tld.len() <= 6 && tld.chars().all(|c| c.is_ascii_alphabetic())) {
                    // Looks like a domain
                    if !cleaned.contains('@') { // Not an email
                        urls.push(format!("https://{}", cleaned));
                    }
                }
            }
        }
        urls
    }

    fn extract_entities(query: &str) -> Vec<RecognizedEntity> {
        let mut entities = Vec::new();

        for word in query.split_whitespace() {
            // URL detection
            if word.starts_with("http://") || word.starts_with("https://") {
                entities.push(RecognizedEntity {
                    text: word.to_string(),
                    entity_type: EntityType::Url,
                    confidence: 0.98,
                });
                continue;
            }

            // Domain detection
            if word.contains('.') && !word.contains(' ') && word.len() > 4 {
                let parts: Vec<&str> = word.split('.').collect();
                if parts.len() >= 2 && parts.last().map_or(false, |tld| tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic())) {
                    entities.push(RecognizedEntity {
                        text: word.to_string(),
                        entity_type: EntityType::Domain,
                        confidence: 0.85,
                    });
                    continue;
                }
            }

            // Capitalized multi-char words → potential organization or technology
            if word.len() >= 3 && word.chars().next().map_or(false, |c| c.is_uppercase()) {
                entities.push(RecognizedEntity {
                    text: word.to_string(),
                    entity_type: EntityType::Organization,
                    confidence: 0.45,
                });
            }
        }

        entities
    }

    fn build_discovery_queries(
        query: &str,
        entities: &[RecognizedEntity],
        mode: PlanMode,
    ) -> Vec<DiscoveryQuery> {
        let mut queries = Vec::new();

        // Primary query is always included
        queries.push(DiscoveryQuery {
            query_text: query.to_string(),
            priority: QueryPriority::Primary,
            provider_filter: vec![],
            max_results: 10,
        });

        // For deep research, expand with entity-focused queries
        if mode == PlanMode::DeepResearch {
            let org_entities: Vec<_> = entities.iter()
                .filter(|e| e.entity_type == EntityType::Organization || e.entity_type == EntityType::Technology)
                .collect();

            for entity in org_entities.iter().take(3) {
                queries.push(DiscoveryQuery {
                    query_text: format!("{} official documentation", entity.text),
                    priority: QueryPriority::Expansion,
                    provider_filter: vec![],
                    max_results: 5,
                });
            }

            // Add "explained" variant
            if !query.to_lowercase().starts_with("what") {
                queries.push(DiscoveryQuery {
                    query_text: format!("{} explained", query),
                    priority: QueryPriority::Expansion,
                    provider_filter: vec![],
                    max_results: 5,
                });
            }
        }

        queries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_local_mode() {
        let plan = QueryPlanner::plan("trigram index", "code");
        assert_eq!(plan.mode, PlanMode::Local);
    }

    #[test]
    fn test_plan_global_mode() {
        let plan = QueryPlanner::plan("OpenAI GPT-5", "global");
        assert_eq!(plan.mode, PlanMode::Global);
        assert!(!plan.discovery_queries.is_empty());
    }

    #[test]
    fn test_plan_url_detection() {
        let plan = QueryPlanner::plan("https://docs.rs/tokio/latest/tokio/", "global");
        assert_eq!(plan.mode, PlanMode::DirectUrl);
        assert_eq!(plan.direct_urls.len(), 1);
    }

    #[test]
    fn test_plan_deep_research() {
        let plan = QueryPlanner::plan("research global stablecoin infrastructure", "global");
        assert_eq!(plan.mode, PlanMode::DeepResearch);
        assert!(plan.discovery_queries.len() > 1);
        assert!(plan.crawl_config.is_some());
    }

    #[test]
    fn test_plan_browse_mode() {
        let plan = QueryPlanner::plan("open github.com/rust-lang", "global");
        assert_eq!(plan.mode, PlanMode::Browse);
    }

    #[test]
    fn test_entity_extraction() {
        let plan = QueryPlanner::plan("OpenAI versus Anthropic comparison", "global");
        let org_entities: Vec<_> = plan.entities.iter()
            .filter(|e| e.entity_type == EntityType::Organization)
            .collect();
        assert!(org_entities.len() >= 2);
    }
}
