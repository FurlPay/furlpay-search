//! File system watcher for incremental index updates.

use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Renamed { from: PathBuf, to: PathBuf },
}

pub struct WorkspaceWatcher {
    workspace_root: PathBuf,
}

impl WorkspaceWatcher {
    pub fn new(workspace_root: &Path) -> Self {
        Self {
            workspace_root: workspace_root.to_path_buf(),
        }
    }

    /// Stub: actual implementation in Phase 2 (Sprint 2-3).
    pub async fn watch(&self, _tx: mpsc::Sender<FileEvent>) -> anyhow::Result<()> {
        tracing::info!(path = %self.workspace_root.display(), "File watcher started (stub)");
        Ok(())
    }
}
