//! Browser navigation controller and history management.

use std::time::Instant;
use serde::{Deserialize, Serialize};
use crate::security::browser_permissions::BrowserPermissions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationState {
    Idle,
    Navigating,
    DomLoaded,
    NetworkIdle,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationRecord {
    pub url: String,
    pub http_status: u16,
    pub title: String,
    pub state: NavigationState,
    pub duration_ms: u64,
}

pub struct NavigationController {
    history: Vec<String>,
    current_index: usize,
    permissions: BrowserPermissions,
}

impl NavigationController {
    pub fn new(permissions: BrowserPermissions) -> Self {
        Self {
            history: Vec::new(),
            current_index: 0,
            permissions,
        }
    }

    /// Check if target URL is allowed and record navigation intention.
    pub fn navigate_to(&mut self, target_url: &str) -> Result<String, String> {
        self.permissions.validate_target_url(target_url)?;

        // If navigated from a back position, truncate forward history
        if self.current_index < self.history.len() {
            self.history.truncate(self.current_index + 1);
        }

        self.history.push(target_url.to_string());
        self.current_index = self.history.len() - 1;

        Ok(target_url.to_string())
    }

    /// Navigate back in browser history.
    pub fn go_back(&mut self) -> Option<&str> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(&self.history[self.current_index])
        } else {
            None
        }
    }

    /// Navigate forward in browser history.
    pub fn go_forward(&mut self) -> Option<&str> {
        if self.current_index + 1 < self.history.len() {
            self.current_index += 1;
            Some(&self.history[self.current_index])
        } else {
            None
        }
    }

    /// Current active URL.
    pub fn current_url(&self) -> Option<&str> {
        self.history.get(self.current_index).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_history() {
        let mut controller = NavigationController::new(BrowserPermissions::default());
        assert!(controller.navigate_to("https://example.com/page1").is_ok());
        assert!(controller.navigate_to("https://example.com/page2").is_ok());

        assert_eq!(controller.current_url(), Some("https://example.com/page2"));
        assert_eq!(controller.go_back(), Some("https://example.com/page1"));
        assert_eq!(controller.go_forward(), Some("https://example.com/page2"));
    }
}
