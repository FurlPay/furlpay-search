//! Browser process sandboxing, flags, and resource limits.

use std::time::Duration;

/// Sandboxing flags and security constraints for headless browser processes.
#[derive(Debug, Clone)]
pub struct BrowserSandboxConfig {
    /// Chromium command-line security arguments.
    pub cli_args: Vec<String>,
    /// Maximum execution time for a single browser task.
    pub task_timeout: Duration,
    /// Maximum allowed memory footprint (in MB).
    pub max_memory_mb: usize,
    /// Whether to run in isolated incognito context with ephemeral profile.
    pub ephemeral_profile: bool,
}

impl Default for BrowserSandboxConfig {
    fn default() -> Self {
        let cli_args = vec![
            "--headless=new".into(),
            "--disable-gpu".into(),
            "--disable-dev-shm-usage".into(),
            "--no-first-run".into(),
            "--no-default-browser-check".into(),
            "--disable-background-networking".into(),
            "--disable-default-apps".into(),
            "--disable-extensions".into(),
            "--disable-sync".into(),
            "--disable-translate".into(),
            "--safebrowsing-disable-auto-update".into(),
            "--mute-audio".into(),
            "--hide-scrollbars".into(),
            "--metrics-recording-only".into(),
        ];

        Self {
            cli_args,
            task_timeout: Duration::from_secs(20),
            max_memory_mb: 512,
            ephemeral_profile: true,
        }
    }
}

impl BrowserSandboxConfig {
    /// Validate that flags contain critical security arguments.
    pub fn is_secure(&self) -> bool {
        self.cli_args.contains(&"--headless=new".to_string())
            && self.cli_args.contains(&"--disable-dev-shm-usage".to_string())
            && self.ephemeral_profile
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_default_security() {
        let config = BrowserSandboxConfig::default();
        assert!(config.is_secure());
    }
}
