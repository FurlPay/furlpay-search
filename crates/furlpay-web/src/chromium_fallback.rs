//! Async headless Chromium rendering worker pool (Milestone 2 stub).
pub struct ChromiumPool;
impl ChromiumPool {
    pub async fn render(_url: &str) -> Result<String, std::io::Error> {
        Ok(String::new())
    }
}
