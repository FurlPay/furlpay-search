//! HTTP connection pool statistics and management.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Connection pool statistics for monitoring.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PoolStats {
    pub total_requests: u64,
    pub active_connections: u64,
    pub reused_connections: u64,
    pub failed_connections: u64,
}

/// Tracks connection pool metrics. The actual connection pooling is handled
/// by reqwest's internal hyper connection pool.
#[derive(Debug, Clone)]
pub struct ConnectionPoolTracker {
    total_requests: Arc<AtomicU64>,
    active_connections: Arc<AtomicU64>,
    reused_connections: Arc<AtomicU64>,
    failed_connections: Arc<AtomicU64>,
}

impl ConnectionPoolTracker {
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            active_connections: Arc::new(AtomicU64::new(0)),
            reused_connections: Arc::new(AtomicU64::new(0)),
            failed_connections: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_request(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_complete(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.failed_connections.fetch_add(1, Ordering::Relaxed);
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn stats(&self) -> PoolStats {
        PoolStats {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            reused_connections: self.reused_connections.load(Ordering::Relaxed),
            failed_connections: self.failed_connections.load(Ordering::Relaxed),
        }
    }
}

impl Default for ConnectionPoolTracker {
    fn default() -> Self {
        Self::new()
    }
}
