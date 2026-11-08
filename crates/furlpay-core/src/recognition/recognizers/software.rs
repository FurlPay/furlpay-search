//! Software and developer artifact recognizers (SemVer, API routes, Git commits).

use std::collections::HashMap;
use super::PatternRecognizer;
use crate::recognition::candidate::{PatternCandidate, PatternCategory, ValidationStatus};

pub struct SoftwareRecognizer;

impl SoftwareRecognizer {
    pub fn new() -> Self {
        Self
    }

    /// Check if string matches Semantic Versioning (SemVer 2.0.0).
    /// Format: [v|V]MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]
    fn parse_semver(s: &str) -> Option<(u64, u64, u64, Option<String>, Option<String>)> {
        let v_stripped = if s.starts_with('v') || s.starts_with('V') {
            &s[1..]
        } else {
            s
        };

        let (core, build) = match v_stripped.split_once('+') {
            Some((c, b)) => (c, Some(b.to_string())),
            None => (v_stripped, None),
        };

        let (version_parts, prerelease) = match core.split_once('-') {
            Some((c, p)) => (c, Some(p.to_string())),
            None => (core, None),
        };

        let nums: Vec<&str> = version_parts.split('.').collect();
        if nums.len() != 3 {
            return None;
        }

        let major = nums[0].parse::<u64>().ok()?;
        let minor = nums[1].parse::<u64>().ok()?;
        let patch = nums[2].parse::<u64>().ok()?;

        Some((major, minor, patch, prerelease, build))
    }

    /// Check if string matches HTTP API endpoint / route pattern.
    fn is_api_route(s: &str) -> bool {
        let trimmed = s.trim();
        // Method prefixed (e.g. "GET /api/v1/search")
        for method in &["GET ", "POST ", "PUT ", "DELETE ", "PATCH ", "HEAD ", "OPTIONS "] {
            if trimmed.starts_with(method) {
                let path = &trimmed[method.len()..];
                return path.starts_with('/');
            }
        }
        // Direct root path starting with /api/, /v1/, /v2/, /graphql, or /rest/
        if trimmed.starts_with("/api/")
            || trimmed.starts_with("/v1/")
            || trimmed.starts_with("/v2/")
            || trimmed.starts_with("/v3/")
            || trimmed.starts_with("/graphql")
            || trimmed.starts_with("/health")
            || trimmed.starts_with("/metrics")
        {
            return true;
        }
        false
    }
}

impl PatternRecognizer for SoftwareRecognizer {
    fn id(&self) -> &str {
        "software.universal"
    }

    fn category(&self) -> PatternCategory {
        PatternCategory::Software
    }

    fn recognize(&self, input: &str) -> Vec<PatternCandidate> {
        let trimmed = input.trim();
        let mut candidates = Vec::new();

        // 1. Semantic Versioning (SemVer)
        if let Some((major, minor, patch, prerelease, build)) = Self::parse_semver(trimmed) {
            let mut meta = HashMap::new();
            meta.insert("major".into(), major.to_string());
            meta.insert("minor".into(), minor.to_string());
            meta.insert("patch".into(), patch.to_string());
            if let Some(ref p) = prerelease {
                meta.insert("prerelease".into(), p.clone());
            }
            if let Some(ref b) = build {
                meta.insert("build".into(), b.clone());
            }

            candidates.push(PatternCandidate {
                recognizer_id: "software.semver".into(),
                category: PatternCategory::Software,
                display_name: format!("Semantic Version (v{}.{}.{})", major, minor, patch),
                raw_value: trimmed.to_string(),
                normalized_value: format!("{}.{}.{}{}{}", major, minor, patch,
                    prerelease.as_ref().map(|p| format!("-{}", p)).unwrap_or_default(),
                    build.as_ref().map(|b| format!("+{}", b)).unwrap_or_default()
                ),
                validation_status: ValidationStatus::FormatValidated,
                confidence: 0.95,
                metadata: meta,
                evidence: format!("Valid SemVer 2.0.0 specification ({}.{}.{})", major, minor, patch),
                suggested_route: "PackageRegistry".into(),
            });
            return candidates;
        }

        // 2. API Route / Endpoint
        if Self::is_api_route(trimmed) {
            let mut meta = HashMap::new();
            let (method, path) = if let Some((m, p)) = trimmed.split_once(' ') {
                (m.trim().to_uppercase(), p.trim())
            } else {
                ("ANY".to_string(), trimmed)
            };
            meta.insert("http_method".into(), method.clone());
            meta.insert("path".into(), path.to_string());

            candidates.push(PatternCandidate {
                recognizer_id: "software.api_route".into(),
                category: PatternCategory::Software,
                display_name: format!("API Route [{} {}]", method, path),
                raw_value: trimmed.to_string(),
                normalized_value: format!("{} {}", method, path),
                validation_status: ValidationStatus::FormatValidated,
                confidence: 0.94,
                metadata: meta,
                evidence: format!("REST / HTTP route endpoint pattern with method '{}'", method),
                suggested_route: "CodebaseSymbolSearch".into(),
            });
            return candidates;
        }

        // 3. Short Git Commit Hash (7 to 12 hex characters)
        if (7..=12).contains(&trimmed.len()) && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            let mut meta = HashMap::new();
            meta.insert("type".into(), "AbbreviatedCommitHash".into());
            meta.insert("length".into(), trimmed.len().to_string());

            candidates.push(PatternCandidate {
                recognizer_id: "software.git.short_commit".into(),
                category: PatternCategory::Software,
                display_name: format!("Git Short Commit ({})", trimmed),
                raw_value: trimmed.to_string(),
                normalized_value: trimmed.to_lowercase(),
                validation_status: ValidationStatus::FormatValidated,
                confidence: 0.88,
                metadata: meta,
                evidence: format!("{}-character hexadecimal string matching Git abbreviated object ID", trimmed.len()),
                suggested_route: "GitRepository".into(),
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
    fn test_semver() {
        let rec = SoftwareRecognizer::new();
        let res = rec.recognize("v1.2.3-beta.1+20261108");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].recognizer_id, "software.semver");
        assert_eq!(res[0].metadata.get("major").unwrap(), "1");
        assert_eq!(res[0].metadata.get("minor").unwrap(), "2");
        assert_eq!(res[0].metadata.get("patch").unwrap(), "3");
    }

    #[test]
    fn test_api_route() {
        let rec = SoftwareRecognizer::new();
        let res = rec.recognize("GET /api/v1/search/stream");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].recognizer_id, "software.api_route");
        assert_eq!(res[0].metadata.get("http_method").unwrap(), "GET");
    }

    #[test]
    fn test_git_short_commit() {
        let rec = SoftwareRecognizer::new();
        let res = rec.recognize("ea2a0d4");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].recognizer_id, "software.git.short_commit");
    }
}
