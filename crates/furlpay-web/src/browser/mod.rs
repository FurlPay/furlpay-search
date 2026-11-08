//! Isolated browser runtime, session management, navigation, and rendering.

pub mod sandbox;
pub mod navigation;
pub mod interaction;
pub mod session_manager;
pub mod runtime;

pub use sandbox::BrowserSandboxConfig;
pub use navigation::{NavigationController, NavigationRecord, NavigationState};
pub use interaction::{BrowserAction, ActionResult, InteractionManager};
pub use session_manager::{SessionManager, BrowserTab};
pub use runtime::{BrowserRuntime, RenderedPage};

/// Browser worker pool for backward compatibility.
pub struct BrowserWorkerPool;

impl BrowserWorkerPool {
    /// Render page using the unified BrowserRuntime.
    pub async fn render_page(url: &str) -> Result<RenderedPage, String> {
        let mut runtime = BrowserRuntime::new()?;
        runtime.open_and_render(url).await
    }
}
