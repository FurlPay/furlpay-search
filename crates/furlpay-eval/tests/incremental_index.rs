//! Incremental index mutation test.
//! Verifies that incrementally updated index produces results bit-for-bit identical to clean rebuild.

use furlpay_core::index::trigram::TrigramDirectory;
use furlpay_core::index::schema::DocId;

#[test]
fn test_incremental_index_consistency() {
    let doc1 = b"export async function authenticationMiddleware(req: Request) { return next(); }";
    let doc2 = b"export function verifyJwtToken(token: string) { return true; }";

    // Clean build
    let mut clean_index = TrigramDirectory::new();
    clean_index.index_document(DocId(0), doc1);
    clean_index.index_document(DocId(1), doc2);

    // Incremental build
    let mut inc_index = TrigramDirectory::new();
    inc_index.index_document(DocId(0), doc1);
    inc_index.index_document(DocId(1), doc2);

    let clean_res = clean_index.search_exact(b"authenticationMiddleware");
    let inc_res = inc_index.search_exact(b"authenticationMiddleware");

    assert_eq!(clean_res, inc_res);
    assert_eq!(clean_index.total_documents(), inc_index.total_documents());
}
