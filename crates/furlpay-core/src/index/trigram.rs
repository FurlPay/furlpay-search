//! Positional trigram sparse directory with XXH3 hash lookup.

use std::collections::HashMap;
use xxhash_rust::xxh3::xxh3_64;
use crate::index::schema::{DocId, PositionalPosting, PostingListHeader, TrigramEntry};

/// Extracts all 3-byte trigrams from a byte slice.
pub fn extract_trigrams(data: &[u8]) -> Vec<(u64, u32)> {
    if data.len() < 3 {
        return Vec::new();
    }
    let mut trigrams = Vec::with_capacity(data.len() - 2);
    for i in 0..data.len() - 2 {
        let tri = &data[i..i + 3];
        let hash = xxh3_64(tri);
        trigrams.push((hash, i as u32));
    }
    trigrams
}

/// Flat-array trigram directory for O(1) hash-based lookup.
pub struct TrigramDirectory {
    /// Map from trigram hash -> posting list.
    index: HashMap<u64, TrigramEntry>,
    /// Total documents indexed.
    total_docs: u32,
}

impl TrigramDirectory {
    /// Create a new empty trigram directory.
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            total_docs: 0,
        }
    }

    /// Index a single document's content.
    pub fn index_document(&mut self, doc_id: DocId, content: &[u8]) {
        let trigrams = extract_trigrams(content);

        // Group trigrams by hash.
        let mut grouped: HashMap<u64, Vec<u32>> = HashMap::new();
        for (hash, offset) in trigrams {
            grouped.entry(hash).or_default().push(offset);
        }

        for (hash, offsets) in grouped {
            let entry = self.index.entry(hash).or_insert_with(|| TrigramEntry {
                trigram_hash: hash,
                header: PostingListHeader {
                    doc_count: 0,
                    total_occurrences: 0,
                },
                postings: Vec::new(),
            });
            let tf = offsets.len() as u16;
            entry.header.doc_count += 1;
            entry.header.total_occurrences += offsets.len() as u32;
            entry.postings.push(PositionalPosting {
                doc_id,
                term_frequency: tf,
                byte_offsets: offsets,
            });
        }
        self.total_docs += 1;
    }

    /// Look up a trigram by its XXH3 hash.
    pub fn lookup(&self, trigram_hash: u64) -> Option<&TrigramEntry> {
        self.index.get(&trigram_hash)
    }

    /// Search for an exact substring by intersecting trigram posting lists.
    pub fn search_exact(&self, query: &[u8]) -> Vec<DocId> {
        if query.len() < 3 {
            return Vec::new();
        }

        let query_trigrams = extract_trigrams(query);
        let mut query_hashes: Vec<u64> = query_trigrams.iter().map(|(h, _)| *h).collect();
        query_hashes.sort_unstable();
        query_hashes.dedup();

        // Get posting lists for all query trigrams, sorted by selectivity.
        let mut posting_lists: Vec<&TrigramEntry> = Vec::new();
        for hash in &query_hashes {
            match self.index.get(hash) {
                Some(entry) => posting_lists.push(entry),
                None => return Vec::new(), // Trigram not in index => no matches.
            }
        }
        posting_lists.sort_by_key(|e| e.header.doc_count);

        if posting_lists.is_empty() {
            return Vec::new();
        }

        // Intersect posting lists: start with the most selective.
        let mut candidates: Vec<DocId> = posting_lists[0]
            .postings
            .iter()
            .map(|p| p.doc_id)
            .collect();
        candidates.sort_unstable();
        candidates.dedup();

        for list in &posting_lists[1..] {
            let mut list_docs: Vec<DocId> = list.postings.iter().map(|p| p.doc_id).collect();
            list_docs.sort_unstable();
            list_docs.dedup();
            candidates = intersect_sorted(&candidates, &list_docs);
            if candidates.is_empty() {
                return Vec::new();
            }
        }

        candidates
    }

    /// Returns the document frequency of a trigram.
    pub fn document_frequency(&self, trigram_hash: u64) -> u32 {
        self.index
            .get(&trigram_hash)
            .map(|e| e.header.doc_count)
            .unwrap_or(0)
    }

    /// Returns the total number of indexed documents.
    pub fn total_documents(&self) -> u32 {
        self.total_docs
    }

    /// Returns the number of unique trigrams in the index.
    pub fn trigram_count(&self) -> usize {
        self.index.len()
    }
}

impl Default for TrigramDirectory {
    fn default() -> Self {
        Self::new()
    }
}

/// Sorted intersection of two DocId slices.
fn intersect_sorted(a: &[DocId], b: &[DocId]) -> Vec<DocId> {
    let mut result = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                result.push(a[i]);
                i += 1;
                j += 1;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_trigrams() {
        let data = b"hello";
        let trigrams = extract_trigrams(data);
        assert_eq!(trigrams.len(), 3); // "hel", "ell", "llo"
    }

    #[test]
    fn test_index_and_search() {
        let mut dir = TrigramDirectory::new();
        dir.index_document(DocId(0), b"fn handle_payment(amount: u64) -> Result<(), Error>");
        dir.index_document(DocId(1), b"fn verify_token(token: &str) -> bool");
        dir.index_document(DocId(2), b"fn handle_refund(payment_id: u64)");

        let results = dir.search_exact(b"handle");
        assert!(results.contains(&DocId(0)));
        assert!(results.contains(&DocId(2)));
        assert!(!results.contains(&DocId(1)));
    }

    #[test]
    fn test_empty_query() {
        let dir = TrigramDirectory::new();
        assert!(dir.search_exact(b"ab").is_empty());
    }

    #[test]
    fn test_no_match() {
        let mut dir = TrigramDirectory::new();
        dir.index_document(DocId(0), b"hello world");
        assert!(dir.search_exact(b"xyz_nonexistent").is_empty());
    }
}
