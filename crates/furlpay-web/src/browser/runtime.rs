//! Unified browser runtime coordinating navigation, rendering, and interaction.

use std::time::Instant;
use serde::{Deserialize, Serialize};

use super::navigation::NavigationController;
use super::interaction::{BrowserAction, InteractionManager};
use super::sandbox::BrowserSandboxConfig;
use super::session_manager::SessionManager;
use crate::fetch::http_client::HttpClient;
use crate::extraction::html::HtmlExtractor;
use crate::security::browser_permissions::BrowserPermissions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedPage {
    pub url: String,
    pub final_url: String,
    pub title: String,
    pub html_content: String,
    pub text_content: String,
    pub status_code: u16,
    pub rendered_in_ms: u64,
    pub is_js_rendered: bool,
}

pub struct BrowserRuntime {
    permissions: BrowserPermissions,
    sandbox: BrowserSandboxConfig,
    session: SessionManager,
    http_client: HttpClient,
    html_extractor: HtmlExtractor,
}

impl BrowserRuntime {
    pub fn new() -> Result<Self, String> {
        let permissions = BrowserPermissions::default();
        let sandbox = BrowserSandboxConfig::default();
        let session = SessionManager::new(permissions.max_tabs);
        let http_client = HttpClient::new().map_err(|e| e.to_string())?;
        let html_extractor = HtmlExtractor::new();

        Ok(Self {
            permissions,
            sandbox,
            session,
            http_client,
            html_extractor,
        })
    }

    /// Open a target URL, validate security constraints, and render its readable content.
    pub async fn open_and_render(&mut self, url: &str) -> Result<RenderedPage, String> {
        let start = Instant::now();

        // 1. Enforce browser security permissions (SSRF, local files, private networks)
        self.permissions.validate_target_url(url)?;

        // 2. Open or reuse tab
        let _tab_id = self.session.open_tab(Some(url))?;

        // 3. Fetch web content via secure client with redirect handling and content-limits
        let fetch_res = self.http_client.fetch(url).await.map_err(|e| e.to_string())?;

        let body = fetch_res.body_text.unwrap_or_default();
        let extracted = self.html_extractor.extract(&body, Some(&fetch_res.final_url));

        let elapsed = start.elapsed().as_millis() as u64;

        Ok(RenderedPage {
            url: url.to_string(),
            final_url: fetch_res.final_url,
            title: extracted.title,
            html_content: body,
            text_content: extracted.main_text,
            status_code: fetch_res.status,
            rendered_in_ms: elapsed,
            is_js_rendered: false,
        })
    }

    /// Execute an interaction action on active page.
    pub fn perform_action(&self, action: &BrowserAction) -> Result<(), String> {
        let interaction_mgr = InteractionManager::new(self.permissions.clone());
        interaction_mgr.check_action_safety(action)
    }

    /// Access active session manager.
    pub fn session(&self) -> &SessionManager {
        &self.session
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_browser_runtime_init() {
        let runtime = BrowserRuntime::new();
        assert!(runtime.is_ok());
    }
}
