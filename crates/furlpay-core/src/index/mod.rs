//! Index subsystem — positional trigram, BM25, generation management, memory tiers.

pub mod schema;
pub mod trigram;
pub mod tantivy_wrapper;
pub mod generation;
pub mod memory_tier;
