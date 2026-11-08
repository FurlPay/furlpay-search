//! Parallel hedged SERP query racer (Milestone 2 stub).

#[derive(Debug, Clone)]
pub struct SerpResult {
    pub provider: String,
    pub url: String,
    pub title: String,
    pub snippet: String,
}

pub struct SerpRacer;
impl SerpRacer {
    pub async fn race(_query: &str) -> Vec<SerpResult> {
        Vec::new()
    }
}
