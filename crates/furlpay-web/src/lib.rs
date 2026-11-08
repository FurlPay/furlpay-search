//! FurlPay Web — Unified web retrieval engine.
//!
//! Capabilities:
//! 1. **Internet connectivity** — Outbound HTTPS requests with DNS, TLS, redirect handling.
//! 2. **Web retrieval** — Fetch any public URL, extract readable content.
//! 3. **Web search** — Discover relevant URLs via configurable search-source adapters.
//! 4. **Browser automation** — Headless Chromium for JavaScript-heavy pages.

pub mod security;
pub mod fetch;
pub mod extraction;
pub mod providers;
pub mod browser;
pub mod router;
pub mod search_orchestrator;
pub mod sse_stream;
pub mod intent;
pub mod discovery;
pub mod crawler;
pub mod research;

// Re-exports for convenience
pub use fetch::http_client::HttpClient;
pub use extraction::html::HtmlExtractor;
pub use router::{WebResult, WebRouter, WebRouterError, WebRetrievalResponse, WebTelemetry};
pub use search_orchestrator::{SearchOrchestrator, SearchMode, UnifiedSearchResult};
pub use security::ssrf_protection::SsrfGuard;
pub use providers::search_provider::{SearchOptions, SearchHit};
pub use intent::{IntentDisambiguator, IntentAnalysis, DomainCluster};
pub use discovery::{QueryPlanner, SearchPlan, PlanMode, Deduplicator, ProviderRegistry};
pub use crawler::{CrawlScheduler, CrawlPolicy, CrawlResult, CrawledPage, LinkEdge, LinkScorer};
pub use research::{Citation, EvidenceExtractor, EvidenceItem, SourceAnalyzer, ResearchSynthesizer, ResearchReport, CoverageTracker, AdaptiveStoppingController, ResearchRanker};
pub use browser::{BrowserRuntime, RenderedPage, BrowserAction};
pub use security::BrowserPermissions;
