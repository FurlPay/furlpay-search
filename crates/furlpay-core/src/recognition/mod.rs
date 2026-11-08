//! Universal Pattern Intelligence Subsystem (Milestone 2).
//!
//! Provides extensible pattern recognition, structural validation, cryptographic checksums,
//! and contextual entity classification across multi-domain inputs.

pub mod candidate;
pub mod confidence;
pub mod context;
pub mod engine;
pub mod fallback;
pub mod normalizer;
pub mod registry;
pub mod tokenizer;
pub mod validation;
pub mod recognizers;

// Re-exports
pub use candidate::{PatternCandidate, PatternCategory, ValidationStatus};
pub use engine::PatternEngine;
pub use registry::PatternRegistry;
