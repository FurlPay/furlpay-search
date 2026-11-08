//! Financial identifier recognizers (IBAN, SWIFT/BIC).

use std::collections::HashMap;
use super::PatternRecognizer;
use crate::recognition::candidate::{PatternCandidate, PatternCategory, ValidationStatus};
use crate::recognition::validation::validate_iban;

pub struct FinancialRecognizer;

impl FinancialRecognizer {
    pub fn new() -> Self {
        Self
    }

    /// Check if string matches SWIFT/BIC code pattern (8 or 11 chars).
    /// Format: BBBBCCLL[BBB]
    /// BBBB: 4 letters (Bank code)
    /// CC: 2 letters (ISO 3166-1 country code)
    /// LL: 2 alphanumeric (Location code)
    /// BBB: 3 alphanumeric (Branch code, optional)
    fn is_swift_bic(s: &str) -> bool {
        let clean: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        if clean.len() != 8 && clean.len() != 11 {
            return false;
        }

        // SWIFT/BIC codes are strictly uppercase alphanumeric bank identifiers
        if !clean.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
            return false;
        }

        let upper = clean;

        // Bank code: 4 letters
        if !upper[..4].chars().all(|c| c.is_ascii_alphabetic()) {
            return false;
        }

        // Country code: 2 letters
        if !upper[4..6].chars().all(|c| c.is_ascii_alphabetic()) {
            return false;
        }

        // Location code: 2 alphanumeric
        if !upper[6..8].chars().all(|c| c.is_ascii_alphanumeric()) {
            return false;
        }

        // Branch code: 3 alphanumeric (if 11 chars)
        if upper.len() == 11 && !upper[8..11].chars().all(|c| c.is_ascii_alphanumeric()) {
            return false;
        }

        true
    }
}

impl PatternRecognizer for FinancialRecognizer {
    fn id(&self) -> &str {
        "financial.universal"
    }

    fn category(&self) -> PatternCategory {
        PatternCategory::Financial
    }

    fn recognize(&self, input: &str) -> Vec<PatternCandidate> {
        let trimmed = input.trim();
        let clean: String = trimmed.chars().filter(|c| !c.is_whitespace() && *c != '-').collect();
        let upper = clean.to_uppercase();
        let mut candidates = Vec::new();

        // 1. IBAN recognition
        if upper.len() >= 15 && upper.len() <= 34 {
            let starts_with_country = upper[..2].chars().all(|c| c.is_ascii_alphabetic());
            let has_check_digits = upper[2..4].chars().all(|c| c.is_ascii_digit());
            let remaining_bban = upper[4..].chars().all(|c| c.is_ascii_alphanumeric());

            if starts_with_country && has_check_digits && remaining_bban {
                let (checksum_ok, detail) = validate_iban(&upper);
                let country = &upper[..2];
                let mut meta = HashMap::new();
                meta.insert("country".into(), country.into());
                meta.insert("standard".into(), "ISO 13616 / ISO 7064".into());
                meta.insert("length".into(), upper.len().to_string());

                if checksum_ok {
                    candidates.push(PatternCandidate {
                        recognizer_id: "financial.iban".into(),
                        category: PatternCategory::Financial,
                        display_name: format!("Valid IBAN ({})", country),
                        raw_value: trimmed.to_string(),
                        normalized_value: upper.clone(),
                        validation_status: ValidationStatus::CryptographicallyVerified,
                        confidence: 0.99,
                        metadata: meta,
                        evidence: format!("ISO 7064 Mod 97-10 check passed: {}", detail),
                        suggested_route: "FinancialEngine".into(),
                    });
                    return candidates;
                } else {
                    candidates.push(PatternCandidate {
                        recognizer_id: "financial.iban".into(),
                        category: PatternCategory::Financial,
                        display_name: format!("IBAN Candidate (Checksum Failed - {})", country),
                        raw_value: trimmed.to_string(),
                        normalized_value: upper.clone(),
                        validation_status: ValidationStatus::ChecksumFailed,
                        confidence: 0.85,
                        metadata: meta,
                        evidence: format!("Format matches IBAN structure but checksum failed: {}", detail),
                        suggested_route: "FinancialEngine".into(),
                    });
                    return candidates;
                }
            }
        }

        // 2. SWIFT/BIC Code recognition
        if Self::is_swift_bic(trimmed) {
            let mut meta = HashMap::new();
            let country = &upper[4..6];
            let bank = &upper[..4];
            meta.insert("bank_code".into(), bank.into());
            meta.insert("country_code".into(), country.into());
            meta.insert("code_type".into(), if upper.len() == 8 { "BIC-8 (Primary)" } else { "BIC-11 (Branch)" }.into());

            candidates.push(PatternCandidate {
                recognizer_id: "financial.swift_bic".into(),
                category: PatternCategory::Financial,
                display_name: format!("SWIFT/BIC Code ({})", bank),
                raw_value: trimmed.to_string(),
                normalized_value: upper.clone(),
                validation_status: ValidationStatus::FormatValidated,
                confidence: 0.92,
                metadata: meta,
                evidence: format!("Valid ISO 9362 SWIFT/BIC format (Bank: {}, Country: {})", bank, country),
                suggested_route: "FinancialEngine".into(),
            });
            return candidates;
        }

        candidates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_iban() {
        let recognizer = FinancialRecognizer::new();
        let results = recognizer.recognize("DE89 3704 0044 0532 0130 00");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].validation_status, ValidationStatus::CryptographicallyVerified);
        assert_eq!(results[0].metadata.get("country").unwrap(), "DE");
    }

    #[test]
    fn test_invalid_iban_checksum() {
        let recognizer = FinancialRecognizer::new();
        let results = recognizer.recognize("DE89370400440532013001");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].validation_status, ValidationStatus::ChecksumFailed);
    }

    #[test]
    fn test_swift_bic() {
        let recognizer = FinancialRecognizer::new();
        let results = recognizer.recognize("DEUTDEDD");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].display_name, "SWIFT/BIC Code (DEUT)");
        assert_eq!(results[0].validation_status, ValidationStatus::FormatValidated);
    }
}
