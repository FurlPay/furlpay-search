//! Document and publishing standards recognizers (ISBN-10, ISBN-13, DOI).

use std::collections::HashMap;
use super::PatternRecognizer;
use crate::recognition::candidate::{PatternCandidate, PatternCategory, ValidationStatus};
use crate::recognition::validation::{validate_isbn10, validate_isbn13};

pub struct StandardsRecognizer;

impl StandardsRecognizer {
    pub fn new() -> Self {
        Self
    }

    /// Check if string matches Digital Object Identifier (DOI) pattern.
    /// Format: 10.NNNN.../SUFFIX
    fn is_doi(s: &str) -> bool {
        let trimmed = s.trim();
        let stripped = if trimmed.starts_with("doi:") || trimmed.starts_with("DOI:") {
            &trimmed[4..]
        } else if trimmed.starts_with("https://doi.org/") {
            &trimmed[16..]
        } else {
            trimmed
        };

        if !stripped.starts_with("10.") {
            return false;
        }

        let parts: Vec<&str> = stripped.splitn(2, '/').collect();
        if parts.len() != 2 {
            return false;
        }

        let registrant = &parts[0][3..]; // after "10."
        let suffix = parts[1];

        // Registrant must be at least 4 digits
        if registrant.len() < 4 || !registrant.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        !suffix.is_empty()
    }
}

impl PatternRecognizer for StandardsRecognizer {
    fn id(&self) -> &str {
        "standards.universal"
    }

    fn category(&self) -> PatternCategory {
        PatternCategory::Documents
    }

    fn recognize(&self, input: &str) -> Vec<PatternCandidate> {
        let trimmed = input.trim();
        let mut candidates = Vec::new();

        // 1. ISBN-13 Check
        let isbn13_clean: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
        if isbn13_clean.len() == 13 && (isbn13_clean.starts_with("978") || isbn13_clean.starts_with("979")) {
            let valid = validate_isbn13(&isbn13_clean);
            let mut meta = HashMap::new();
            meta.insert("standard".into(), "ISBN-13 / EAN-13".into());
            meta.insert("prefix".into(), isbn13_clean[..3].to_string());

            if valid {
                candidates.push(PatternCandidate {
                    recognizer_id: "standards.isbn13".into(),
                    category: PatternCategory::Documents,
                    display_name: "Validated ISBN-13 Publication".into(),
                    raw_value: trimmed.to_string(),
                    normalized_value: isbn13_clean,
                    validation_status: ValidationStatus::CryptographicallyVerified,
                    confidence: 0.99,
                    metadata: meta,
                    evidence: "EAN-13 weighted modulo-10 checksum verified".into(),
                    suggested_route: "DocumentCatalog".into(),
                });
                return candidates;
            } else {
                candidates.push(PatternCandidate {
                    recognizer_id: "standards.isbn13".into(),
                    category: PatternCategory::Documents,
                    display_name: "ISBN-13 Candidate (Checksum Failed)".into(),
                    raw_value: trimmed.to_string(),
                    normalized_value: isbn13_clean,
                    validation_status: ValidationStatus::ChecksumFailed,
                    confidence: 0.85,
                    metadata: meta,
                    evidence: "13-digit sequence starting with 978/979 but failed modulo-10 checksum".into(),
                    suggested_route: "DocumentCatalog".into(),
                });
                return candidates;
            }
        }

        // 2. ISBN-10 Check
        let isbn10_clean: String = trimmed.chars().filter(|c| c.is_ascii_digit() || *c == 'X' || *c == 'x').collect();
        if isbn10_clean.len() == 10 {
            let valid = validate_isbn10(&isbn10_clean);
            let mut meta = HashMap::new();
            meta.insert("standard".into(), "ISBN-10".into());

            if valid {
                candidates.push(PatternCandidate {
                    recognizer_id: "standards.isbn10".into(),
                    category: PatternCategory::Documents,
                    display_name: "Validated ISBN-10 Publication".into(),
                    raw_value: trimmed.to_string(),
                    normalized_value: isbn10_clean.to_uppercase(),
                    validation_status: ValidationStatus::CryptographicallyVerified,
                    confidence: 0.98,
                    metadata: meta,
                    evidence: "ISO 2108 weighted modulo-11 checksum verified".into(),
                    suggested_route: "DocumentCatalog".into(),
                });
                return candidates;
            }
        }

        // 3. Digital Object Identifier (DOI)
        if Self::is_doi(trimmed) {
            let mut meta = HashMap::new();
            meta.insert("standard".into(), "ISO 26324 DOI".into());

            candidates.push(PatternCandidate {
                recognizer_id: "standards.doi".into(),
                category: PatternCategory::Documents,
                display_name: "Digital Object Identifier (DOI)".into(),
                raw_value: trimmed.to_string(),
                normalized_value: trimmed.to_string(),
                validation_status: ValidationStatus::FormatValidated,
                confidence: 0.96,
                metadata: meta,
                evidence: "Matches ISO 26324 syntax (10.NNNN/...)".into(),
                suggested_route: "AcademicIndex".into(),
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
    fn test_valid_isbn13() {
        let rec = StandardsRecognizer::new();
        let res = rec.recognize("978-0-306-40615-7");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].validation_status, ValidationStatus::CryptographicallyVerified);
    }

    #[test]
    fn test_doi() {
        let rec = StandardsRecognizer::new();
        let res = rec.recognize("10.1038/nphys1170");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].recognizer_id, "standards.doi");
    }
}
