//! Isolated multi-tab browser session management.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserTab {
    pub tab_id: String,
    pub current_url: Option<String>,
    pub title: String,
    pub is_loading: bool,
    pub created_at: String,
}

pub struct SessionManager {
    tabs: HashMap<String, BrowserTab>,
    active_tab_id: Option<String>,
    max_tabs: usize,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(5)
    }
}

impl SessionManager {
    pub fn new(max_tabs: usize) -> Self {
        Self {
            tabs: HashMap::new(),
            active_tab_id: None,
            max_tabs,
        }
    }

    /// Open a new tab, returning its tab ID.
    pub fn open_tab(&mut self, url: Option<&str>) -> Result<String, String> {
        if self.tabs.len() >= self.max_tabs {
            return Err(format!("Max concurrent tabs limit reached ({})", self.max_tabs));
        }

        let tab_id = format!("tab-{}", self.tabs.len() + 1);
        let tab = BrowserTab {
            tab_id: tab_id.clone(),
            current_url: url.map(|u| u.to_string()),
            title: "New Tab".into(),
            is_loading: false,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        self.tabs.insert(tab_id.clone(), tab);
        self.active_tab_id = Some(tab_id.clone());
        Ok(tab_id)
    }

    /// Close an existing tab.
    pub fn close_tab(&mut self, tab_id: &str) -> bool {
        let removed = self.tabs.remove(tab_id).is_some();
        if self.active_tab_id.as_deref() == Some(tab_id) {
            self.active_tab_id = self.tabs.keys().next().cloned();
        }
        removed
    }

    /// Get reference to active tab.
    pub fn active_tab(&self) -> Option<&BrowserTab> {
        self.active_tab_id.as_ref().and_then(|id| self.tabs.get(id))
    }

    /// Total count of open tabs.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_lifecycle() {
        let mut session = SessionManager::new(2);
        let t1 = session.open_tab(Some("https://example.com")).unwrap();
        let t2 = session.open_tab(Some("https://rust-lang.org")).unwrap();

        assert_eq!(session.tab_count(), 2);
        // Exceeding max tabs returns Err
        assert!(session.open_tab(None).is_err());

        session.close_tab(&t1);
        assert_eq!(session.tab_count(), 1);
        assert_eq!(session.active_tab().map(|t| t.tab_id.as_str()), Some(t2.as_str()));
    }
}
