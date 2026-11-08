//! Document, chunk, and posting schemas for the positional trigram index.
use serde::{Deserialize, Serialize};

/// Unique document identifier within an index generation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DocId(pub u32);

/// A single document's posting entry in a trigram posting list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionalPosting {
    pub doc_id: DocId,
    pub term_frequency: u16,
    /// Exact byte offsets within the source file where this trigram occurs.
    pub byte_offsets: Vec<u32>,
}

/// Metadata record for an indexed document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub doc_id: DocId,
    pub file_path: String,
    /// Language identifier (e.g., Rust=1, Python=2, TypeScript=3, etc.)
    pub language_id: u16,
    /// SHA-256 hash of file content for deduplication.
    pub content_hash: [u8; 32],
    /// Git commit hash at index time.
    pub commit_hash: [u8; 20],
    /// Offset into the line-start lookup table.
    pub line_index_offset: u32,
    /// Total byte length of the file.
    pub byte_length: u32,
    /// Cryptographically verified tenant identifier.
    pub tenant_id: u64,
}

/// Header for a trigram posting list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostingListHeader {
    /// Total number of distinct documents containing this trigram.
    pub doc_count: u32,
    /// Sum of all occurrences across all documents.
    pub total_occurrences: u32,
}

/// A complete trigram posting list entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrigramEntry {
    /// XXH3-64 hash of the 3-byte trigram.
    pub trigram_hash: u64,
    pub header: PostingListHeader,
    pub postings: Vec<PositionalPosting>,
}

/// Search result with position and scoring information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub doc_id: DocId,
    pub file_path: String,
    pub byte_offset: u32,
    pub line_number: u32,
    pub match_length: u16,
    pub rrf_score: f32,
    pub snippet: String,
}
