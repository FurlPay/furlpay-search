//! User and autonomous browser interactions (clicking, scrolling, typing).

use serde::{Deserialize, Serialize};
use crate::security::browser_permissions::BrowserPermissions;

/// An interaction command to perform on an active page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserAction {
    Click { selector: String },
    ScrollDown { pixels: u32 },
    TypeText { selector: String, text: String },
    WaitForSelector { selector: String, timeout_ms: u64 },
    CaptureScreenshot,
}

/// Result of executing a browser action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action_type: String,
    pub success: bool,
    pub details: String,
    pub elapsed_ms: u64,
}

pub struct InteractionManager {
    permissions: BrowserPermissions,
}

impl InteractionManager {
    pub fn new(permissions: BrowserPermissions) -> Self {
        Self { permissions }
    }

    /// Check whether an action is safe to execute under permissions.
    pub fn check_action_safety(&self, action: &BrowserAction) -> Result<(), String> {
        match action {
            BrowserAction::Click { selector } => {
                let lower = selector.to_lowercase();
                if lower.contains("submit") || lower.contains("buy") || lower.contains("pay") || lower.contains("transfer") {
                    self.permissions.check_form_submission("POST")?;
                }
            }
            BrowserAction::TypeText { selector, .. } => {
                let lower = selector.to_lowercase();
                if lower.contains("password") || lower.contains("secret") || lower.contains("credit") || lower.contains("card") {
                    return Err("Typing credentials or payment details requires explicit interactive session".into());
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocks_sensitive_submit_click() {
        let mgr = InteractionManager::new(BrowserPermissions::default());
        let action = BrowserAction::Click { selector: "button.submit-order".into() };
        assert!(mgr.check_action_safety(&action).is_err());
    }

    #[test]
    fn test_allows_safe_scroll() {
        let mgr = InteractionManager::new(BrowserPermissions::default());
        let action = BrowserAction::ScrollDown { pixels: 500 };
        assert!(mgr.check_action_safety(&action).is_ok());
    }
}
