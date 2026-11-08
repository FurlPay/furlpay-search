//! Cryptographic identifier recognizers (SHA-256, SHA-512, SHA-1, MD5, UUID/GUID).

use std::collections::HashMap;
use super::PatternRecognizer;
use crate::recognition::candidate::{PatternCandidate, PatternCategory, ValidationStatus};

pub struct CryptoRecognizer;

impl CryptoRecognizer {
    pub fn new() -> Self {
        Self
    }

    fn is_hex(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Check if string is a valid canonical UUID format: 8-4-4-4-12 (36 chars with hyphens).
    fn is_uuid_formatted(s: &str) -> bool {
        if s.len() != 36 {
            return false;
        }
        let bytes = s.as_bytes();
        if bytes[8] != b'-' || bytes[13] != b'-' || bytes[18] != b'-' || bytes[23] != b'-' {
            return false;
        }
        bytes[0..8].iter().all(|b| b.is_ascii_hexdigit())
            && bytes[9..13].iter().all(|b| b.is_ascii_hexdigit())
            && bytes[14..18].iter().all(|b| b.is_ascii_hexdigit())
            && bytes[19..23].iter().all(|b| b.is_ascii_hexdigit())
            && bytes[24..36].iter().all(|b| b.is_ascii_hexdigit())
    }
}

impl PatternRecognizer for CryptoRecognizer {
    fn id(&self) -> &str {
        "crypto.universal"
    }

    fn category(&self) -> PatternCategory {
        PatternCategory::Cryptography
    }

    fn recognize(&self, input: &str) -> Vec<PatternCandidate> {
        let trimmed = input.trim();
        let mut candidates = Vec::new();

        // 1. UUID / GUID (36 characters: 8-4-4-4-12)
        if Self::is_uuid_formatted(trimmed) {
            let mut meta = HashMap::new();
            let version_char = trimmed.chars().nth(14).unwrap_or('?');
            let version_desc = match version_char {
                '1' => "v1 (Date-time & MAC)",
                '4' => "v4 (Random)",
                '5' => "v5 (SHA-1 name-based)",
                _ => "General UUID",
            };
            meta.insert("version".into(), version_desc.into());
            meta.insert("standard".into(), "RFC 4122".into());
            meta.insert("algorithm".into(), format!("UUID ({})", version_desc));

            candidates.push(PatternCandidate {
                recognizer_id: "crypto.uuid".into(),
                category: PatternCategory::Cryptography,
                display_name: format!("UUID ({})", version_desc),
                raw_value: trimmed.to_string(),
                normalized_value: trimmed.to_lowercase(),
                validation_status: ValidationStatus::FormatValidated,
                confidence: 0.99,
                metadata: meta,
                evidence: format!("RFC 4122 standard 128-bit identifier (UUID version char: '{}')", version_char),
                suggested_route: "EntityLookup".into(),
            });
            return candidates;
        }

        // Avoid false positives on small hex numbers or prefixed 0x hex
        let hex_body = if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
            &trimmed[2..]
        } else {
            trimmed
        };

        if Self::is_hex(hex_body) {
            match hex_body.len() {
                // SHA-512 (128 hex chars / 512 bits)
                128 => {
                    let mut meta = HashMap::new();
                    meta.insert("algorithm".into(), "SHA-512".into());
                    meta.insert("bit_length".into(), "512".into());

                    candidates.push(PatternCandidate {
                        recognizer_id: "crypto.digest.sha512".into(),
                        category: PatternCategory::Cryptography,
                        display_name: "SHA-512 Cryptographic Digest".into(),
                        raw_value: trimmed.to_string(),
                        normalized_value: hex_body.to_lowercase(),
                        validation_status: ValidationStatus::FormatValidated,
                        confidence: 0.98,
                        metadata: meta,
                        evidence: "128-character hexadecimal string representing 512-bit digest".into(),
                        suggested_route: "CryptographicVerification".into(),
                    });
                    return candidates;
                }
                // SHA-256 (64 hex chars / 256 bits)
                64 => {
                    let mut meta = HashMap::new();
                    meta.insert("algorithm".into(), "SHA-256".into());
                    meta.insert("bit_length".into(), "256".into());

                    candidates.push(PatternCandidate {
                        recognizer_id: "crypto.digest.sha256".into(),
                        category: PatternCategory::Cryptography,
                        display_name: "SHA-256 Cryptographic Digest".into(),
                        raw_value: trimmed.to_string(),
                        normalized_value: hex_body.to_lowercase(),
                        validation_status: ValidationStatus::FormatValidated,
                        confidence: 0.97,
                        metadata: meta,
                        evidence: "64-character hexadecimal string representing 256-bit digest".into(),
                        suggested_route: "CryptographicVerification".into(),
                    });
                    return candidates;
                }
                // SHA-1 / Git Object Hash (40 hex chars / 160 bits)
                40 => {
                    let mut meta = HashMap::new();
                    meta.insert("algorithm".into(), "SHA-1 / Git Object ID".into());
                    meta.insert("bit_length".into(), "160".into());

                    candidates.push(PatternCandidate {
                        recognizer_id: "crypto.digest.sha1".into(),
                        category: PatternCategory::Cryptography,
                        display_name: "SHA-1 / Git Commit Object ID".into(),
                        raw_value: trimmed.to_string(),
                        normalized_value: hex_body.to_lowercase(),
                        validation_status: ValidationStatus::FormatValidated,
                        confidence: 0.95,
                        metadata: meta,
                        evidence: "40-character hexadecimal string matching SHA-1 or Git commit hash".into(),
                        suggested_route: "CryptographicVerification".into(),
                    });
                    return candidates;
                }
                // MD5 (32 hex chars / 128 bits)
                32 => {
                    let mut meta = HashMap::new();
                    meta.insert("algorithm".into(), "MD5".into());
                    meta.insert("bit_length".into(), "128".into());

                    candidates.push(PatternCandidate {
                        recognizer_id: "crypto.digest.md5".into(),
                        category: PatternCategory::Cryptography,
                        display_name: "MD5 Message Digest".into(),
                        raw_value: trimmed.to_string(),
                        normalized_value: hex_body.to_lowercase(),
                        validation_status: ValidationStatus::FormatValidated,
                        confidence: 0.93,
                        metadata: meta,
                        evidence: "32-character hexadecimal string matching MD5 digest".into(),
                        suggested_route: "CryptographicVerification".into(),
                    });
                    return candidates;
                }
                _ => {}
            }
        }

        candidates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid() {
        let rec = CryptoRecognizer::new();
        let res = rec.recognize("123e4567-e89b-12d3-a456-426614174000");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].recognizer_id, "crypto.uuid");
        assert_eq!(res[0].validation_status, ValidationStatus::FormatValidated);
    }

    #[test]
    fn test_sha256() {
        let rec = CryptoRecognizer::new();
        let res = rec.recognize("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].recognizer_id, "crypto.digest.sha256");
        assert_eq!(res[0].metadata.get("algorithm").unwrap(), "SHA-256");
    }

    #[test]
    fn test_sha1() {
        let rec = CryptoRecognizer::new();
        let res = rec.recognize("2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].recognizer_id, "crypto.digest.sha1");
    }

    #[test]
    fn test_md5() {
        let rec = CryptoRecognizer::new();
        let res = rec.recognize("9e107d9d372bb6826bd81d3542a419d6");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].recognizer_id, "crypto.digest.md5");
    }
}
