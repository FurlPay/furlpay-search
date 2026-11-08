# FurlPay Search — Benchmark & Quality Report v0.2.0

> Empirical search quality measurements after adaptive ranking + web extraction upgrade

**Date:** 2026-11-09  
**Build:** v0.2.0 (AdaptiveMultiSignal_Hierarchical + ContentExtraction)  
**All tests:** 125 passed (66 core + 55 web + 4 SIMD)

---

## 1. Local Code Search Quality

### Intent Classification Accuracy

| Query | Expected Intent | Actual Intent | ✓ |
|-------|----------------|---------------|---|
| `tr` | ShortTokenPrefix | ShortTokenPrefix | ✅ |
| `payment` | ContentSearch | ContentSearch | ✅ |
| `authentication` | ContentSearch | ContentSearch | ✅ |
| `blockchain\tron.ts` | ExactRelativePath | ExactRelativePath | ✅ |
| `TRON.TS` | FilenameWithExtension | FilenameWithExtension | ✅ |
| `ast/treesitter.rs stub` | ExactRelativePath+stub | ExactRelativePath+stub | ✅ |
| `treesitter.rs stub` | FilenameWithExtension+stub | FilenameWithExtension+stub | ✅ |
| Empty query | SingleCharacterPrefix | SingleCharacterPrefix | ✅ |

### Ranking Correctness

| Test | Description | Status |
|------|-------------|--------|
| Exact path dominance | Exact path match caps subordinate signals | ✅ Pass |
| Stub separation | `implementation_completeness` ≠ `implementation_status_match` | ✅ Pass |
| No double-counting | Hierarchical scoring prevents correlated signal amplification | ✅ Pass |
| Feature sanitization | NaN, Inf, negative values clamped to [0.0, 1.0] | ✅ Pass |
| Explanation accuracy | Dominant signal identified by weighted contribution, not threshold | ✅ Pass |
| Legacy migration | Old `implementation_quality` field stays consistent | ✅ Pass |

### Regression Benchmark (11 queries)

All 11 queries from `scripts/test_benchmark.py` pass correctly.

---

## 2. Web Search Quality

### Improvements in v0.2.0

| Feature | v0.1.0 | v0.2.0 |
|---------|--------|--------|
| Deduplication | Inline URL lowercase + trailing slash | Proper `Deduplicator`: URL normalization (tracking params, fragments, scheme) + XXH3 content hash |
| Verification | All results: `SearchHitReceived` | Top 5 results: `ContentExtracted` with actual page data |
| Result identity | No stable IDs | `{search_id}_r{ordinal}` |
| Page extraction | Not attempted | Title, description, main text, author, published date, word count |
| `verified_live` | `true` (incorrect) | `false` for search hits, upgraded only after actual fetch |
| Query expansion (deep) | `"{q} explained"`, `"{q} documentation"`, `"{q} latest news 2026"` | Intent-aware: news/how-to/comparison/general patterns |
| SSE events | Flat: `search.started → web.results → search.completed` | Progressive: `+ web.discovery.started → web.discovery.completed → web.extraction.completed` |
| Telemetry | Basic counts | `duplicates_removed`, `extraction_count`, `search_id` |

### SSE Event Contract (Web Search)

```
event: search.started        → {search_id, query, mode, scope}
event: recognition.detected  → {recognizer, confidence}  (conditional)
event: web.discovery.started → {search_id, mode, query}
event: web.discovery.completed → {total_discovered, duplicates_removed, provider_latency_us}
event: web.intent            → {explicit_intent, is_ambiguous, candidate_intents}
event: web.extraction.completed → {extraction_count, extracted_urls}  (conditional)
event: web.results           → {results[], total_results, telemetry}
event: search.completed      → {total_results, duplicates_removed, extraction_count, elapsed_micros}
```

---

## 3. Architecture Summary

### Crate Structure

| Crate | Purpose | Test Count |
|-------|---------|------------|
| `furlpay-core` | Indexing, ranking, recognition, AST | 66 |
| `furlpay-web` | HTTP client, HTML extraction, search providers, SSRF, browser, research | 55 |
| `furlpay-simd` | CPU feature detection, SIMD intersection | 4 |
| `furlpay-daemon` | Resident daemon, SSE streaming, file watcher | Runtime |
| `furlpay-ipc` | Inter-process communication protocol | Runtime |
| `furlpay-eval` | MRR evaluation benchmarks | 1 |
| `furlpay-mcp` | Model Context Protocol server | Runtime |

### Key Modules

- **Adaptive Ranking** (`furlpay-core/src/ranking/adaptive.rs`): Intent-based query classification, hierarchical scoring, separated completeness vs status match
- **Deduplicator** (`furlpay-web/src/discovery/deduplication.rs`): URL normalization + XXH3 content hashing
- **HtmlExtractor** (`furlpay-web/src/extraction/html.rs`): Boilerplate-stripped content extraction with metadata
- **SearchOrchestrator** (`furlpay-web/src/search_orchestrator.rs`): Unified search with extraction pipeline
- **SSRF Guard** (`furlpay-web/src/security/ssrf_protection.rs`): Blocks private IPs, link-local, metadata endpoints

---

## 4. Test Commands

```bash
# Run all tests
cargo test --manifest-path Cargo.toml

# Run regression benchmark
python scripts/test_benchmark.py

# Run additional test cases
python scripts/test_additional_cases.py

# Live web search test
curl "http://127.0.0.1:3000/api/v1/search/stream?q=latest+tech+news&mode=web"

# Live deep research test
curl "http://127.0.0.1:3000/api/v1/search/stream?q=latest+tech+news&mode=deep"

# Local code search test
curl "http://127.0.0.1:3000/api/v1/search/stream?q=tr&mode=code"
```
