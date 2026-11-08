//! Query token and sub-string span extractor.

/// Tokenizer that extracts entity-like spans from compound search queries.
pub struct QueryTokenizer;

impl QueryTokenizer {
    /// Extract potential identifier tokens and spans from input.
    /// Returns the full normalized input as the primary candidate,
    /// followed by any discrete word tokens that have sufficient length (>= 4 chars).
    pub fn extract_candidate_tokens(input: &str) -> Vec<String> {
        let trimmed = input.trim();
        let mut tokens = Vec::new();

        if !trimmed.is_empty() {
            tokens.push(trimmed.to_string());
        }

        // Split by whitespace
        for word in trimmed.split_whitespace() {
            let clean = word.trim_matches(|c: char| c.is_ascii_punctuation() && c != '/' && c != ':' && c != '-');
            if clean.len() >= 4 && clean != trimmed {
                tokens.push(clean.to_string());
            }
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tokens() {
        let tokens = QueryTokenizer::extract_candidate_tokens("verify TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t now");
        assert!(tokens.contains(&"TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string()));
    }
}
