//! Pattern candidate models and classification results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// High-level domain category of a recognized pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternCategory {
    Blockchain,
    Financial,
    Internet,
    Cryptography,
    Software,
    Documents,
    GeneralText,
    Unknown,
}

/// Depth of validation applied to a candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStatus {
    /// Full cryptographic checksum verified (e.g. Base58Check double-SHA256, Mod-97 IBAN).
    CryptographicallyVerified,
    /// Structural and character-set format valid, but checksum was optional or unkeyed.
    FormatValidated,
    /// Matches format syntax, but cryptographic or arithmetic checksum check failed.
    ChecksumFailed,
    /// Known synthesized or testing identifier format.
    SynthesizedOrTestOnly,
    /// Plausible pattern but cannot be verified without external live network query.
    Unvalidated,
}

/// A recognized entity or structure candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCandidate {
    /// Unique recognizer identifier (e.g. "blockchain.tron.base58check").
    pub recognizer_id: String,
    /// Domain category.
    pub category: PatternCategory,
    /// Display name (e.g. "TRON (Base58Check) Wallet Address").
    pub display_name: String,
    /// Raw query substring.
    pub raw_value: String,
    /// Canonical normalized representation.
    pub normalized_value: String,
    /// Rigorous validation verdict.
    pub validation_status: ValidationStatus,
    /// Calibrated confidence score between 0.0 and 1.0.
    pub confidence: f32,
    /// Detailed metadata (e.g. network, currency, country, version).
    pub metadata: HashMap<String, String>,
    /// Concrete evidence detailing why this candidate was recognized.
    pub evidence: String,
    /// Recommended downstream retrieval route.
    pub suggested_route: String,
}
