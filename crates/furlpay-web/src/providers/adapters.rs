//! Concrete search provider adapters (DuckDuckGo, Brave, SearXNG, SerpAPI).

use super::search_provider::{BoxFuture, SearchHit, SearchOptions, SearchProvider};
use std::time::Duration;

/// DuckDuckGo provider requiring zero API keys for instant public web search.
pub struct DuckDuckGoProvider {
    client: reqwest::Client,
}

impl Default for DuckDuckGoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DuckDuckGoProvider {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(6))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_default();

        Self { client }
    }

    /// Helper to unwrap DuckDuckGo redirect link `//duckduckgo.com/l/?uddg=<TARGET_URL>&rut=...`
    fn decode_ddg_redirect(raw_url: &str) -> String {
        if let Some(pos) = raw_url.find("uddg=") {
            let encoded = &raw_url[pos + 5..];
            let end = encoded.find('&').unwrap_or(encoded.len());
            let target = &encoded[..end];
            if let Ok(decoded) = urlencoding_decode(target) {
                return decoded;
            }
        }
        if raw_url.starts_with("//") {
            format!("https:{}", raw_url)
        } else {
            raw_url.to_string()
        }
    }
}

fn urlencoding_decode(s: &str) -> Result<String, ()> {
    let mut out = String::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).map_err(|_| ())?, 16) {
                out.push(byte as char);
                i += 3;
                continue;
            }
        } else if bytes[i] == b'+' {
            out.push(' ');
            i += 1;
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    Ok(out)
}

impl SearchProvider for DuckDuckGoProvider {
    fn name(&self) -> &str {
        "DuckDuckGo"
    }

    fn search<'a>(&'a self, query: &'a str, options: &'a SearchOptions) -> BoxFuture<'a, Result<Vec<SearchHit>, String>> {
        Box::pin(async move {
            let mut hits = Vec::new();

            // 1. First attempt: Instant Answer JSON API for quick structured definitions / facts
            let api_url = format!(
                "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
                urlencoding::encode(query)
            );

            if let Ok(resp) = self.client.get(&api_url).send().await {
                if resp.status().is_success() {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        let heading = json["Heading"].as_str().unwrap_or("").trim();
                        let abstract_text = json["AbstractText"].as_str().unwrap_or("").trim();
                        let abstract_url = json["AbstractURL"].as_str().unwrap_or("").trim();

                        if !abstract_text.is_empty() && !abstract_url.is_empty() {
                            hits.push(SearchHit {
                                url: abstract_url.to_string(),
                                title: if heading.is_empty() { query.to_string() } else { heading.to_string() },
                                snippet: abstract_text.to_string(),
                                published_date: None,
                                provider: "DuckDuckGo (InstantAnswer)".into(),
                                score: 0.96,
                            });
                        }

                        // Also check RelatedTopics
                        if let Some(topics) = json["RelatedTopics"].as_array() {
                            for topic in topics.iter().take(options.limit.saturating_sub(hits.len())) {
                                if let (Some(text), Some(url)) = (topic["Text"].as_str(), topic["FirstURL"].as_str()) {
                                    if !text.is_empty() && !url.is_empty() {
                                        hits.push(SearchHit {
                                            url: url.to_string(),
                                            title: text.split(" - ").next().unwrap_or(text).to_string(),
                                            snippet: text.to_string(),
                                            published_date: None,
                                            provider: "DuckDuckGo (Topic)".into(),
                                            score: 0.88,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 2. Second attempt: Live DuckDuckGo HTML Web Search
            if hits.len() < options.limit {
                let html_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
                let html_resp = self.client.get(&html_url).send().await;

                if let Ok(resp) = html_resp {
                    if resp.status().is_success() {
                        if let Ok(body) = resp.text().await {
                            let doc = scraper::Html::parse_document(&body);
                            let result_selector = scraper::Selector::parse(".result").ok();
                            let title_selector = scraper::Selector::parse(".result__title a").ok();
                            let snippet_selector = scraper::Selector::parse(".result__snippet").ok();

                            if let (Some(res_sel), Some(title_sel), Some(snip_sel)) =
                                (result_selector, title_selector, snippet_selector)
                            {
                                for el in doc.select(&res_sel).take(options.limit) {
                                    let mut title = String::new();
                                    let mut url = String::new();
                                    let mut snippet = String::new();

                                    if let Some(title_el) = el.select(&title_sel).next() {
                                        title = title_el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                                        if let Some(href) = title_el.value().attr("href") {
                                            url = Self::decode_ddg_redirect(href);
                                        }
                                    }

                                    if let Some(snip_el) = el.select(&snip_sel).next() {
                                        snippet = snip_el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                                    }

                                    if !url.is_empty() && !title.is_empty() {
                                        if !hits.iter().any(|h| h.url == url) {
                                            hits.push(SearchHit {
                                                url,
                                                title,
                                                snippet,
                                                published_date: None,
                                                provider: "DuckDuckGo".into(),
                                                score: 0.90,
                                            });
                                        }
                                    }

                                    if hits.len() >= options.limit {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 3. Resilient query relaxation fallback for multi-word queries
            // If query returned 0 results and contains auxiliary stop-words, relax and retry once
            if hits.is_empty() {
                let stop_words = ["and", "or", "the", "in", "of", "with", "for", "on", "at", "to", "a", "an"];
                let relaxed_words: Vec<&str> = query
                    .split_whitespace()
                    .filter(|w| !stop_words.contains(&w.to_lowercase().as_str()))
                    .collect();

                if relaxed_words.len() >= 2 && relaxed_words.len() < query.split_whitespace().count() {
                    let relaxed_query = relaxed_words.join(" ");
                    let html_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(&relaxed_query));
                    if let Ok(resp) = self.client.get(&html_url).send().await {
                        if resp.status().is_success() {
                            if let Ok(body) = resp.text().await {
                                let doc = scraper::Html::parse_document(&body);
                                if let (Some(res_sel), Some(title_sel), Some(snip_sel)) = (
                                    scraper::Selector::parse(".result").ok(),
                                    scraper::Selector::parse(".result__title a").ok(),
                                    scraper::Selector::parse(".result__snippet").ok(),
                                ) {
                                    for el in doc.select(&res_sel).take(options.limit) {
                                        let mut title = String::new();
                                        let mut url = String::new();
                                        let mut snippet = String::new();

                                        if let Some(title_el) = el.select(&title_sel).next() {
                                            title = title_el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                                            if let Some(href) = title_el.value().attr("href") {
                                                url = Self::decode_ddg_redirect(href);
                                            }
                                        }

                                        if let Some(snip_el) = el.select(&snip_sel).next() {
                                            snippet = snip_el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                                        }

                                        if !url.is_empty() && !title.is_empty() && !hits.iter().any(|h| h.url == url) {
                                            hits.push(SearchHit {
                                                url,
                                                title,
                                                snippet,
                                                published_date: None,
                                                provider: "DuckDuckGo (Relaxed)".into(),
                                                score: 0.86,
                                            });
                                        }

                                        if hits.len() >= options.limit {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Ok(hits)
        })
    }
}

/// Brave Search provider (configurable via API key).
pub struct BraveSearchProvider {
    client: reqwest::Client,
    api_key: String,
}

impl BraveSearchProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(6))
            .build()
            .unwrap_or_default();
        Self {
            client,
            api_key: api_key.into(),
        }
    }
}

impl SearchProvider for BraveSearchProvider {
    fn name(&self) -> &str {
        "BraveSearch"
    }

    fn search<'a>(&'a self, query: &'a str, options: &'a SearchOptions) -> BoxFuture<'a, Result<Vec<SearchHit>, String>> {
        Box::pin(async move {
            let mut url = format!(
                "https://api.search.brave.com/res/v1/web/search?q={}&count={}",
                urlencoding::encode(query),
                options.limit
            );
            if let Some(ref country) = options.country {
                url.push_str(&format!("&country={}", country));
            }
            if let Some(ref lang) = options.language {
                url.push_str(&format!("&search_lang={}", lang));
            }

            let resp = self
                .client
                .get(&url)
                .header("Accept", "application/json")
                .header("X-Subscription-Token", &self.api_key)
                .send()
                .await
                .map_err(|e| format!("Brave API request failed: {}", e))?;

            if !resp.status().is_success() {
                return Err(format!("Brave API returned status: {}", resp.status()));
            }

            let val: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse Brave API JSON: {}", e))?;

            let mut hits = Vec::new();
            if let Some(results) = val["web"]["results"].as_array() {
                for item in results.iter().take(options.limit) {
                    let title = item["title"].as_str().unwrap_or("").trim().to_string();
                    let url = item["url"].as_str().unwrap_or("").trim().to_string();
                    let snippet = item["description"].as_str().unwrap_or("").trim().to_string();
                    let published = item["page_age"].as_str().map(|s| s.to_string());

                    if !url.is_empty() && !title.is_empty() {
                        hits.push(SearchHit {
                            url,
                            title,
                            snippet,
                            published_date: published,
                            provider: "BraveSearch".into(),
                            score: 0.95,
                        });
                    }
                }
            }

            Ok(hits)
        })
    }
}

/// SearXNG metasearch instance provider.
pub struct SearxngProvider {
    client: reqwest::Client,
    base_url: String,
}

impl SearxngProvider {
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(6))
            .build()
            .unwrap_or_default();
        Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }
}

impl SearchProvider for SearxngProvider {
    fn name(&self) -> &str {
        "SearXNG"
    }

    fn search<'a>(&'a self, query: &'a str, options: &'a SearchOptions) -> BoxFuture<'a, Result<Vec<SearchHit>, String>> {
        Box::pin(async move {
            let mut url = format!(
                "{}/search?q={}&format=json&categories=general",
                self.base_url,
                urlencoding::encode(query)
            );
            if let Some(ref lang) = options.language {
                url.push_str(&format!("&language={}", lang));
            }

            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("SearXNG request failed: {}", e))?;

            if !resp.status().is_success() {
                return Err(format!("SearXNG returned status: {}", resp.status()));
            }

            let val: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse SearXNG JSON: {}", e))?;

            let mut hits = Vec::new();
            if let Some(results) = val["results"].as_array() {
                for item in results.iter().take(options.limit) {
                    let title = item["title"].as_str().unwrap_or("").trim().to_string();
                    let url = item["url"].as_str().unwrap_or("").trim().to_string();
                    let snippet = item["content"].as_str().unwrap_or("").trim().to_string();
                    let published = item["publishedDate"].as_str().map(|s| s.to_string());

                    if !url.is_empty() && !title.is_empty() {
                        hits.push(SearchHit {
                            url,
                            title,
                            snippet,
                            published_date: published,
                            provider: "SearXNG".into(),
                            score: 0.93,
                        });
                    }
                }
            }

            Ok(hits)
        })
    }
}

/// SerpAPI provider (Google, Bing, Yahoo metasearch via API key).
pub struct SerpApiProvider {
    client: reqwest::Client,
    api_key: String,
}

impl SerpApiProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(6))
            .build()
            .unwrap_or_default();
        Self {
            client,
            api_key: api_key.into(),
        }
    }
}

impl SearchProvider for SerpApiProvider {
    fn name(&self) -> &str {
        "SerpAPI"
    }

    fn search<'a>(&'a self, query: &'a str, options: &'a SearchOptions) -> BoxFuture<'a, Result<Vec<SearchHit>, String>> {
        Box::pin(async move {
            let mut url = format!(
                "https://serpapi.com/search.json?q={}&api_key={}&num={}",
                urlencoding::encode(query),
                self.api_key,
                options.limit
            );
            if let Some(ref country) = options.country {
                url.push_str(&format!("&gl={}", country));
            }
            if let Some(ref lang) = options.language {
                url.push_str(&format!("&hl={}", lang));
            }

            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("SerpAPI request failed: {}", e))?;

            if !resp.status().is_success() {
                return Err(format!("SerpAPI returned status: {}", resp.status()));
            }

            let val: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse SerpAPI JSON: {}", e))?;

            let mut hits = Vec::new();
            if let Some(results) = val["organic_results"].as_array() {
                for item in results.iter().take(options.limit) {
                    let title = item["title"].as_str().unwrap_or("").trim().to_string();
                    let url = item["link"].as_str().unwrap_or("").trim().to_string();
                    let snippet = item["snippet"].as_str().unwrap_or("").trim().to_string();

                    if !url.is_empty() && !title.is_empty() {
                        hits.push(SearchHit {
                            url,
                            title,
                            snippet,
                            published_date: None,
                            provider: "SerpAPI".into(),
                            score: 0.94,
                        });
                    }
                }
            }

            Ok(hits)
        })
    }
}

// Module for simple urlencoding
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut encoded = String::new();
        for b in s.bytes() {
            match b {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(b as char);
                }
                b' ' => encoded.push('+'),
                _ => {
                    encoded.push_str(&format!("%{:02X}", b));
                }
            }
        }
        encoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ddg_redirect_decoding() {
        let redirect = "//duckduckgo.com/l/?uddg=https%3A%2F%2Fwww.rust-lang.org%2F&rut=123";
        let target = DuckDuckGoProvider::decode_ddg_redirect(redirect);
        assert_eq!(target, "https://www.rust-lang.org/");
    }

    #[test]
    fn test_urlencode() {
        assert_eq!(urlencoding::encode("Rust 2026 search"), "Rust+2026+search");
    }
}
