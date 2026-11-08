//! ArcSwap-based immutable index generation management.
//!
//! Enables lock-free concurrent read access with zero-downtime index swaps.

use arc_swap::ArcSwap;
use std::sync::Arc;
use crate::index::schema::DocumentMetadata;
use crate::ast::symbol_graph::SymbolGraph;
use crate::index::trigram::TrigramDirectory;

/// An immutable snapshot of the entire search index at a point in time.
pub struct IndexGeneration {
    /// Monotonically increasing generation identifier.
    pub generation_id: u64,
    /// Git commit hash this generation was built from.
    pub commit_hash: [u8; 20],
    /// Unix epoch milliseconds when this generation was created.
    pub timestamp_epoch_ms: u64,
    /// The positional trigram index.
    pub trigram_index: Arc<TrigramDirectory>,
    /// Tantivy BM25 full-text index handle.
    pub bm25_index: Arc<tantivy::Index>,
    /// The AST symbol graph for structural queries.
    pub ast_graph: Arc<SymbolGraph>,
    /// Document metadata for all indexed files.
    pub doc_metadata: Arc<Vec<DocumentMetadata>>,
}

/// Manages the lifecycle of index generations using lock-free ArcSwap.
pub struct GenerationManager {
    current: ArcSwap<IndexGeneration>,
}

impl GenerationManager {
    /// Create a new manager with an initial generation.
    pub fn new(initial: Arc<IndexGeneration>) -> Self {
        Self {
            current: ArcSwap::from(initial),
        }
    }

    /// Take a consistent read snapshot of the current generation.
    pub fn snapshot(&self) -> Arc<IndexGeneration> {
        self.current.load_full()
    }

    /// Atomically publish a new generation without blocking active readers.
    pub fn publish(&self, next: Arc<IndexGeneration>) {
        self.current.store(next);
    }

    /// Returns the current generation ID without taking a full snapshot.
    pub fn current_generation_id(&self) -> u64 {
        self.current.load().generation_id
    }
}
