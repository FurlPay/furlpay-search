//! Input normalization and query canonicalization.

/// Query normalizer.
pub struct QueryNormalizer;

impl QueryNormalizer {
    /// Normalize raw user input by trimming, stripping wrapping quotes or brackets,
    /// and extracting potential embedded identifiers.
    pub fn normalize(input: &str) -> String {
        let mut s = input.trim();

        // Strip enclosing quotes: "...", '...', `...`
        if (s.starts_with('"') && s.ends_with('"'))
            || (s.starts_with('\'') && s.ends_with('\''))
            || (s.starts_with('`') && s.ends_with('`'))
        {
            if s.len() >= 2 {
                s = &s[1..s.len() - 1].trim();
            }
        }

        // Strip enclosing brackets: <...>, (...), [...]
        if (s.starts_with('<') && s.ends_with('>'))
            || (s.starts_with('(') && s.ends_with(')'))
            || (s.starts_with('[') && s.ends_with(']'))
        {
            if s.len() >= 2 {
                s = &s[1..s.len() - 1].trim();
            }
        }

        // Strip known scheme prefixes (case-insensitive)
        let lower = s.to_lowercase();
        const KNOWN_SCHEME_PREFIXES: &[&str] = &[
            "tron:", "ethereum:", "solana:", "bitcoin:", "iban:", "isbn:", "doi:", "git:"
        ];

        for &prefix in KNOWN_SCHEME_PREFIXES {
            if lower.starts_with(prefix) {
                return s[prefix.len()..].trim().to_string();
            }
        }

        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quotes_stripping() {
        assert_eq!(QueryNormalizer::normalize("\"TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t\""), "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t");
        assert_eq!(QueryNormalizer::normalize("<https://furlpay.io>"), "https://furlpay.io");
        assert_eq!(QueryNormalizer::normalize("tron:TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"), "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t");
    }
}
