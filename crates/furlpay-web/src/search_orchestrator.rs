//! Unified search orchestrator — coordinates Local Code, Public Web, Direct URL,
//! Global Discovery, Deep Research, and Hybrid search.
//!
//! GLOBAL is the default web discovery mode. It imposes no subject-matter,
//! geographic, or topic restrictions on what users can search.

use serde::{Deserialize, Serialize};

use crate::discovery::deduplication::Deduplicator;
use crate::fetch::http_client::{FetchResponse, HttpClient};
use crate::extraction::html::{ExtractedHtml, HtmlExtractor};
use crate::extraction::metadata::{detect_format, DocumentFormat};
use crate::providers::search_provider::SearchOptions;
use crate::router::{WebResult, WebRouter, WebRetrievalResponse};

/// Search mode controlling the scope of retrieval.
/// GLOBAL is the default for web discovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchMode {
    /// Search indexed local files and source code only.
    Code,
    /// Search global public web sources (default web mode).
    Web,
    /// Retrieve a specified public URL.
    Url,
    /// Combine local search, web discovery, and specialized retrieval.
    Hybrid,
    /// Broad public web discovery — the default search mode.
    Global,
    /// Expand into multiple related searches for comprehensive research.
    Deep,
}

impl Default for SearchMode {
    fn default() -> Self {
        Self::Global
    }
}

impl std::str::FromStr for SearchMode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "code" | "local" => Ok(SearchMode::Code),
            "url" => Ok(SearchMode::Url),
            "hybrid" => Ok(SearchMode::Hybrid),
            "deep" => Ok(SearchMode::Deep),
            "global" => Ok(SearchMode::Global),
            "web" => Ok(SearchMode::Web),
            _ => Ok(SearchMode::Global),
        }
    }
}

impl std::fmt::Display for SearchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchMode::Code => write!(f, "code"),
            SearchMode::Web => write!(f, "web"),
            SearchMode::Url => write!(f, "url"),
            SearchMode::Hybrid => write!(f, "hybrid"),
            SearchMode::Global => write!(f, "global"),
            SearchMode::Deep => write!(f, "deep"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSearchResult {
    pub id: String,
    pub source: String,
    pub title: String,
    pub url_or_path: String,
    pub snippet: String,
    pub score: f32,
    pub verified: bool,
    pub mode: String,
    pub latency_us: u64,
    /// Data provenance: which provider or index produced this result.
    pub provider: Option<String>,
    /// Original URL for web results; file path for local results.
    pub canonical_url: Option<String>,
    /// Retrieval timestamp (ISO 8601).
    pub retrieved_at: Option<String>,
}

/// Callback type for progressive extraction events.
/// Called when each page extraction completes (or fails).
pub type ExtractionCallback = Box<dyn Fn(&WebResult, ExtractionEvent) + Send + Sync>;

/// Event emitted during progressive page extraction.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractionEvent {
    pub result_id: String,
    pub url: String,
    pub status: String,          // "started", "success", "failed", "timeout"
    pub http_status: Option<u16>,
    pub word_count: Option<usize>,
    pub page_title: Option<String>,
    pub canonical_url: Option<String>,
    pub published_at: Option<String>,
    pub latency_us: u64,
}

pub struct SearchOrchestrator {
    http_client: HttpClient,
    html_extractor: HtmlExtractor,
}

impl SearchOrchestrator {
    pub fn new() -> Result<Self, crate::fetch::http_client::FetchError> {
        Ok(Self {
            http_client: HttpClient::new()?,
            html_extractor: HtmlExtractor::new(),
        })
    }

    /// Fetch an arbitrary public URL safely with SSRF protection and extract content.
    pub async fn fetch_url(&self, url: &str) -> Result<(FetchResponse, Option<ExtractedHtml>), String> {
        let resp = self.http_client.fetch(url).await
            .map_err(|e| e.to_string())?;

        let doc_format = detect_format(&resp.content_type, &resp.final_url);
        let extracted = match doc_format {
            DocumentFormat::Html => {
                resp.body_text.as_ref().map(|html| self.html_extractor.extract(html, Some(&resp.final_url)))
            }
            _ => None,
        };

        Ok((resp, extracted))
    }

    /// Execute web search with page-level content extraction for top N results.
    ///
    /// This is the primary upgrade over `search_global()`:
    /// 1. Runs discovery via `retrieve_with_options()`
    /// 2. For the top `extract_count` results, fetches the actual page via `fetch_url()`
    /// 3. Extracts title, description, published_date, main_text using `HtmlExtractor`
    /// 4. Updates result with extracted metadata
    /// 5. Upgrades `verification_status` from `SearchHitReceived` to `ContentExtracted`
    pub async fn search_with_extraction(
        &self,
        query: &str,
        options: &SearchOptions,
        extract_count: usize,
    ) -> Result<WebRetrievalResponse, String> {
        let mut response = WebRouter::retrieve_with_options(query, options).await
            .map_err(|e| e.to_string())?;

        // Extract content for top N results concurrently (bounded)
        let extract_n = extract_count.min(response.results.len());
        let mut extraction_count = 0;

        for i in 0..extract_n {
            let url = response.results[i].url.clone();
            let fetch_start = std::time::Instant::now();

            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                self.fetch_url(&url),
            ).await {
                Ok(Ok((fetch_resp, extracted))) => {
                    let result = &mut response.results[i];
                    result.http_status_actual = Some(fetch_resp.status);

                    if let Some(html) = extracted {
                        result.extraction_status = "success".into();
                        result.verification_status = "ContentExtracted".into();
                        result.verification_detail = format!(
                            "Page fetched (HTTP {}) and content extracted: {} words",
                            fetch_resp.status,
                            html.word_count
                        );

                        // Update with extracted metadata
                        if !html.title.is_empty() {
                            result.page_title = Some(html.title);
                        }
                        if let Some(ref canonical) = html.canonical_url {
                            result.canonical_url = Some(canonical.clone());
                        }
                        if let Some(ref pub_date) = html.metadata.published_date {
                            result.published_at = Some(pub_date.clone());
                        }
                        if let Some(ref author) = html.metadata.author {
                            result.author = Some(author.clone());
                        }
                        result.word_count = Some(html.word_count);

                        // Content summary: first ~300 chars of main text
                        if !html.main_text.is_empty() {
                            let summary: String = html.main_text.chars().take(300).collect();
                            let summary = if html.main_text.chars().count() > 300 {
                                format!("{}…", summary.trim())
                            } else {
                                summary.trim().to_string()
                            };
                            result.content_summary = Some(summary);
                        }

                        extraction_count += 1;
                    } else {
                        result.extraction_status = "no_html".into();
                        result.verification_status = "PageFetched".into();
                        result.verification_detail = format!(
                            "Page fetched (HTTP {}) but content type is not HTML",
                            fetch_resp.status
                        );
                    }
                }
                Ok(Err(e)) => {
                    let result = &mut response.results[i];
                    result.extraction_status = "failed".into();
                    result.verification_detail = format!(
                        "Page fetch failed: {}",
                        e
                    );
                    tracing::warn!(url = %url, error = %e, "Page extraction failed");
                }
                Err(_) => {
                    let result = &mut response.results[i];
                    result.extraction_status = "timeout".into();
                    result.verification_detail = format!(
                        "Page fetch timed out after 5s for {}",
                        url
                    );
                    tracing::warn!(url = %url, "Page extraction timed out");
                }
            }

            let _latency = fetch_start.elapsed().as_micros() as u64;
        }

        response.telemetry.extraction_count = extraction_count;
        Ok(response)
    }

    /// Execute global web search — the default, unrestricted discovery mode.
    pub async fn search_global(&self, query: &str, options: &SearchOptions) -> Result<WebRetrievalResponse, String> {
        WebRouter::retrieve_with_options(query, options).await
            .map_err(|e| e.to_string())
    }

    /// Execute web search via WebRouter (backwards-compatible).
    pub async fn search_web(&self, query: &str) -> Result<Vec<WebResult>, String> {
        WebRouter::retrieve(query, false).await
            .map_err(|e| e.to_string())
    }

    /// Execute deep search — expand query into multiple related searches with extraction.
    pub async fn search_deep(&self, query: &str, options: &SearchOptions) -> Result<WebRetrievalResponse, String> {
        let pipeline_start = std::time::Instant::now();
        let mut all_results = Vec::new();

        // Primary query with extraction
        let primary = self.search_with_extraction(query, options, 5).await?;
        let primary_telemetry = primary.telemetry.clone();
        let search_id = primary.search_id.clone();
        all_results.extend(primary.results);

        // Generate expanded queries from the primary query
        let expansions = Self::generate_expansions(query);

        // Run expansions concurrently with limited extraction
        let expansion_options = SearchOptions {
            limit: 5,
            ..options.clone()
        };
        for expansion in expansions.iter().take(3) {
            match WebRouter::retrieve_with_options(expansion, &expansion_options).await {
                Ok(resp) => all_results.extend(resp.results),
                Err(e) => {
                    tracing::warn!(expansion, error = %e, "Deep search expansion failed");
                }
            }
        }

        // Deduplicate all results using proper Deduplicator on URLs
        let mut deduplicator = Deduplicator::new();
        let mut unique = Vec::new();
        for r in all_results {
            if !deduplicator.is_url_seen(&r.url) {
                unique.push(r);
            }
        }

        let mut telemetry = primary_telemetry;
        telemetry.result_count = unique.len();
        telemetry.total_pipeline_latency_us = pipeline_start.elapsed().as_micros() as u64;

        Ok(WebRetrievalResponse {
            results: unique,
            telemetry,
            search_id,
        })
    }

    /// Generate intelligent query expansions for deep research.
    ///
    /// Strategy:
    /// - Extract key entities/concepts from the query
    /// - Generate complementary searches, not just suffix appends
    /// - News queries get date-specific and source-specific expansions
    /// - Technical queries get documentation and tutorial expansions
    fn generate_expansions(query: &str) -> Vec<String> {
        let q_lower = query.to_lowercase();
        let words: Vec<&str> = q_lower.split_whitespace().collect();
        let mut expansions = Vec::new();

        // Detect query intent for smarter expansion
        let is_news_query = words.iter().any(|&w|
            matches!(w, "news" | "latest" | "today" | "yesterday" | "recent" | "update" | "updates" | "breaking")
        );
        let is_how_to = words.iter().any(|&w|
            matches!(w, "how" | "tutorial" | "guide" | "setup" | "install" | "configure")
        );
        let is_comparison = words.iter().any(|&w|
            matches!(w, "vs" | "versus" | "compare" | "comparison" | "alternative" | "alternatives")
        );

        if is_news_query {
            // For news: try different source types and timeframes
            let core_terms: Vec<&str> = words.iter()
                .filter(|&&w| !matches!(w, "news" | "latest" | "today" | "yesterday" | "recent" | "today's"))
                .copied()
                .collect();
            let core = core_terms.join(" ");

            if !core.is_empty() {
                expansions.push(format!("{} highlights this week", core));
                expansions.push(format!("{} developments 2026", core));
                expansions.push(format!("{} industry analysis", core));
            }
        } else if is_how_to {
            // For tutorials: add "best practices" and "examples"
            if !q_lower.starts_with("what") {
                expansions.push(format!("{} best practices", query));
            }
            expansions.push(format!("{} examples", query));
            expansions.push(format!("{} troubleshooting", query));
        } else if is_comparison {
            // For comparisons: look for benchmarks and reviews
            expansions.push(format!("{} benchmark", query));
            expansions.push(format!("{} review", query));
        } else {
            // General: definitional + documentation + recent
            if !q_lower.starts_with("what") {
                expansions.push(format!("{} explained", query));
            }
            expansions.push(format!("{} documentation", query));
            expansions.push(format!("{} latest news 2026", query));
        }

        expansions
    }
}
