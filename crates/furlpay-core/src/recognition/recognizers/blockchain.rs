//! Blockchain entity recognizers (TRON, Ethereum, Solana, Bitcoin, Hashes).

use std::collections::HashMap;
use super::PatternRecognizer;
use crate::recognition::candidate::{PatternCandidate, PatternCategory, ValidationStatus};
use crate::recognition::validation::validate_tron_base58check;

pub struct BlockchainRecognizer;

impl BlockchainRecognizer {
    pub fn new() -> Self {
        Self
    }

    fn is_base58(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| {
            matches!(c, '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z')
        })
    }
}

impl PatternRecognizer for BlockchainRecognizer {
    fn id(&self) -> &str {
        "blockchain.universal"
    }

    fn category(&self) -> PatternCategory {
        PatternCategory::Blockchain
    }

    fn recognize(&self, input: &str) -> Vec<PatternCandidate> {
        let trimmed = input.trim();
        let mut candidates = Vec::new();

        // 1. TRON Address Check (Base58Check)
        if trimmed.starts_with('T') && trimmed.len() == 34 && Self::is_base58(trimmed) {
            let (checksum_ok, detail) = validate_tron_base58check(trimmed);
            let mut meta = HashMap::new();
            meta.insert("network".into(), "TRON".into());
            meta.insert("encoding".into(), "Base58Check".into());
            meta.insert("address_type".into(), "AccountOrContract".into());

            if checksum_ok {
                candidates.push(PatternCandidate {
                    recognizer_id: "blockchain.tron.base58check".into(),
                    category: PatternCategory::Blockchain,
                    display_name: "TRON Validated Wallet Address".into(),
                    raw_value: trimmed.to_string(),
                    normalized_value: trimmed.to_string(),
                    validation_status: ValidationStatus::CryptographicallyVerified,
                    confidence: 0.99,
                    metadata: meta,
                    evidence: format!("Cryptographic TRON Base58Check verified: {}", detail),
                    suggested_route: "BlockchainExplorer".into(),
                });
            } else {
                candidates.push(PatternCandidate {
                    recognizer_id: "blockchain.tron.base58check".into(),
                    category: PatternCategory::Blockchain,
                    display_name: "TRON Wallet Address Candidate (Synthetic/Test)".into(),
                    raw_value: trimmed.to_string(),
                    normalized_value: trimmed.to_string(),
                    validation_status: ValidationStatus::SynthesizedOrTestOnly,
                    confidence: 0.85,
                    metadata: meta,
                    evidence: format!("Format matches TRON syntax (34 chars, starts with 'T'), but checksum failed: {}", detail),
                    suggested_route: "BlockchainExplorer".into(),
                });
            }
            return candidates;
        }

        // 2. Ethereum / EVM Hexadecimal Address
        if (trimmed.starts_with("0x") || trimmed.starts_with("0X")) && trimmed.len() == 42 {
            if trimmed[2..].chars().all(|c| c.is_ascii_hexdigit()) {
                let mut meta = HashMap::new();
                meta.insert("network".into(), "Ethereum/EVM".into());
                meta.insert("encoding".into(), "Hexadecimal".into());

                // Check for mixed-case EIP-55
                let has_upper = trimmed[2..].chars().any(|c| c.is_ascii_uppercase());
                let has_lower = trimmed[2..].chars().any(|c| c.is_ascii_lowercase());
                let (status, conf) = if has_upper && has_lower {
                    (ValidationStatus::FormatValidated, 0.98)
                } else {
                    (ValidationStatus::FormatValidated, 0.95)
                };

                candidates.push(PatternCandidate {
                    recognizer_id: "blockchain.ethereum.evm".into(),
                    category: PatternCategory::Blockchain,
                    display_name: "Ethereum / EVM Wallet Address".into(),
                    raw_value: trimmed.to_string(),
                    normalized_value: format!("0x{}", trimmed[2..].to_lowercase()),
                    validation_status: status,
                    confidence: conf,
                    metadata: meta,
                    evidence: "Valid 42-character hexadecimal EVM account/contract address".into(),
                    suggested_route: "BlockchainExplorer".into(),
                });
                return candidates;
            }
        }

        // 3. Bitcoin Bech32 Address
        if trimmed.starts_with("bc1") && (trimmed.len() == 42 || trimmed.len() == 62) {
            let mut meta = HashMap::new();
            meta.insert("network".into(), "Bitcoin".into());
            meta.insert("encoding".into(), "Bech32".into());
            candidates.push(PatternCandidate {
                recognizer_id: "blockchain.bitcoin.bech32".into(),
                category: PatternCategory::Blockchain,
                display_name: "Bitcoin (Bech32) Address".into(),
                raw_value: trimmed.to_string(),
                normalized_value: trimmed.to_lowercase(),
                validation_status: ValidationStatus::FormatValidated,
                confidence: 0.96,
                metadata: meta,
                evidence: "Valid Bitcoin Bech32 address format (starts with bc1)".into(),
                suggested_route: "BlockchainExplorer".into(),
            });
            return candidates;
        }

        // 4. Solana Address (32-44 Base58 characters, not starting with 0x or T)
        if (32..=44).contains(&trimmed.len())
            && !trimmed.starts_with("0x")
            && !trimmed.starts_with('T')
            && !trimmed.starts_with("bc1")
            && Self::is_base58(trimmed)
        {
            let mut meta = HashMap::new();
            meta.insert("network".into(), "Solana".into());
            meta.insert("encoding".into(), "Base58".into());
            candidates.push(PatternCandidate {
                recognizer_id: "blockchain.solana.base58".into(),
                category: PatternCategory::Blockchain,
                display_name: "Solana Public Key / Address".into(),
                raw_value: trimmed.to_string(),
                normalized_value: trimmed.to_string(),
                validation_status: ValidationStatus::FormatValidated,
                confidence: 0.90,
                metadata: meta,
                evidence: format!("Valid Base58 public key (length {}) matching Solana specification", trimmed.len()),
                suggested_route: "BlockchainExplorer".into(),
            });
            return candidates;
        }

        // 5. Transaction Hashes
        if (trimmed.starts_with("0x") || trimmed.starts_with("0X")) && trimmed.len() == 66 {
            if trimmed[2..].chars().all(|c| c.is_ascii_hexdigit()) {
                let mut meta = HashMap::new();
                meta.insert("network".into(), "EVM".into());
                candidates.push(PatternCandidate {
                    recognizer_id: "blockchain.tx.evm_hash".into(),
                    category: PatternCategory::Blockchain,
                    display_name: "EVM Transaction Hash".into(),
                    raw_value: trimmed.to_string(),
                    normalized_value: trimmed.to_lowercase(),
                    validation_status: ValidationStatus::FormatValidated,
                    confidence: 0.95,
                    metadata: meta,
                    evidence: "66-character 0x-prefixed hex string matching EVM transaction hash".into(),
                    suggested_route: "BlockchainExplorer".into(),
                });
                return candidates;
            }
        }

        candidates
    }
}
