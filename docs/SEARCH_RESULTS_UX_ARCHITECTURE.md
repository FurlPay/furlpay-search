# FurlPay Search — Results UX Architecture

> SSE Event Contract, Result Model, Ranking Formula, Verification Levels

**Version:** 0.2.0  
**Date:** 2026-11-09  
**Status:** Production

---

## 1. SSE Event Lifecycle

### Local Code Search

```
search.started → search.classified → search.index_lookup → search.candidates →
search.ranking → search.verification_batch → search.verified → search.completed
```

### Web / Global Search

```
search.started → recognition.detected? → web.discovery.started →
web.discovery.completed → web.intent → web.extraction.completed? →
web.results → search.completed
```

### Deep Research Search

```
search.started → recognition.detected? → web.discovery.started →
web.discovery.completed → web.intent → web.extraction.completed? →
web.results → search.completed
```

Every search session emits exactly one `search.started` and one `search.completed`.
Events marked `?` are conditional.

---

## 2. Result Model — `WebResult`

| Field | Type | Description |
|-------|------|-------------|
| `result_id` | `String` | Stable ID: `{search_id}_r{ordinal}` |
| `url` | `String` | Discovered URL |
| `title` | `String` | Title from search provider snippet |
| `snippet` | `String` | Provider-supplied description |
| `pathway_used` | `String` | Which provider returned this hit |
| `http_status` | `u16` | Assumed HTTP status (200 from search hit) |
| `rank_score` | `f32` | Final ranking score (0.0–1.0) |
| `topic_cluster` | `Option<String>` | Domain cluster assignment |
| `verification_status` | `String` | Current verification level |
| `verification_detail` | `String` | Human-readable verification explanation |
| `published_at` | `Option<String>` | Publication date (from search or extraction) |
| `retrieved_at` | `String` | ISO 8601 timestamp of retrieval |
| `canonical_url` | `Option<String>` | Canonical URL from `<link rel="canonical">` |
| `extraction_status` | `String` | `not_attempted`, `success`, `failed`, `timeout`, `no_html` |
| `content_summary` | `Option<String>` | First ~300 chars of extracted main text |
| `page_title` | `Option<String>` | Title from actual HTML `<title>` tag |
| `author` | `Option<String>` | Author from page metadata |
| `word_count` | `Option<usize>` | Word count of extracted main text |
| `http_status_actual` | `Option<u16>` | Actual HTTP status from page fetch |

---

## 3. Verification Levels

| Level | Meaning | Evidence |
|-------|---------|----------|
| `SearchHitReceived` | Raw result from a search provider index | URL, title, and snippet from HTTPS search API |
| `UrlResolved` | DNS + TCP + TLS handshake succeeded | Connection established to target host |
| `PageFetched` | HTTP response received | Actual HTTP status code recorded |
| `ContentExtracted` | HTML parsed, content extracted | Title, main text, metadata, word count available |
| `CrossVerified` | Content corroborated across sources | Multiple independent sources agree on claims |

**Contract:** A result's `verification_status` is never upgraded without the corresponding evidence. `SearchHitReceived` means we have a search index record — nothing more.

---

## 4. Deduplication

Uses `Deduplicator` from `crates/furlpay-web/src/discovery/deduplication.rs`:

1. **URL normalization:** lowercase scheme/host, strip tracking params (`utm_*`, `fbclid`, `gclid`, `ref`), remove fragments, remove trailing slashes
2. **Content hashing:** XXH3 64-bit hash of whitespace-normalized, lowercased snippet text
3. **Telemetry:** `duplicates_removed` count reported in `web.discovery.completed` event

---

## 5. Page-Level Content Extraction

For the top N results (default: 5), the `SearchOrchestrator` calls `fetch_url()`:

1. SSRF guard validates the URL is safe to fetch
2. HTTP GET with 5-second timeout
3. `HtmlExtractor` parses the response:
   - `<title>` or `og:title`
   - `<meta name="description">` or `og:description`
   - Main text (stripped of nav, footer, script, style boilerplate)
   - `article:published_time`, `meta[name="date"]`
   - `<link rel="canonical">`
   - Author, word count
4. Result upgraded to `ContentExtracted` with extracted fields populated

---

## 6. Ranking Formula

### Web Results

Multi-signal scoring:

```
score = (provider_score × 0.5) + min(term_relevance, 0.5)

term_relevance = (title_matches × 0.4) + (snippet_matches × 0.2)
```

Then intent-aware reranking via `IntentDisambiguator::rerank_and_diversify()` which applies domain cluster diversification.

### Local Code Results

Hierarchical 3-stage scoring (see `adaptive.rs`):

```
S = S_identity + S_semantic + S_status − P_generic

S_identity = w_path × exact_path + w_filename × exact_filename + ...
S_semantic = w_relevance × contextual_relevance + w_trigram × trigram_similarity + ...
S_status = w_status × implementation_status_match
P_generic = w_penalty × generic_match_penalty
```

With exact-path match capping subordinate signal residuals to prevent double-counting.

---

## 7. Deep Research Mode

1. Primary query with extraction (top 5 pages)
2. Up to 3 expanded queries based on query intent:
   - **News queries:** `"{core terms} highlights this week"`, `"... developments 2026"`, `"... industry analysis"`
   - **How-to queries:** `"... best practices"`, `"... examples"`, `"... troubleshooting"`
   - **Comparison queries:** `"... benchmark"`, `"... review"`
   - **General:** `"... explained"`, `"... documentation"`, `"... latest news 2026"`
3. Cross-query deduplication via `Deduplicator`
4. Combined results ranked and returned

---

## 8. Testing via curl

```bash
# Global web search
curl "http://127.0.0.1:3000/api/v1/search/stream?q=latest+tech+news&mode=web"

# Deep research
curl "http://127.0.0.1:3000/api/v1/search/stream?q=latest+tech+news&mode=deep"

# Local code search
curl "http://127.0.0.1:3000/api/v1/search/stream?q=tr&mode=code"
```
