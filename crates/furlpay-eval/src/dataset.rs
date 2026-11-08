//! Dataset loader and test fixtures for FurlPay evaluation.

use serde::{Deserialize, Serialize};
use furlpay_core::index::schema::DocId;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFixture {
    pub id: String,
    pub category: String,
    pub query: String,
    pub expected_file: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct EvaluationDocument {
    pub id: DocId,
    pub path: String,
    pub content: String,
}

pub struct Dataset {
    pub documents: Vec<EvaluationDocument>,
    pub queries: Vec<QueryFixture>,
}

impl Dataset {
    pub fn default_fixture() -> Self {
        let documents = vec![
            EvaluationDocument {
                id: DocId(0),
                path: "auth/middleware.ts".into(),
                content: "export async function authenticationMiddleware(req: Request, res: Response, next: NextFunction) { const token = req.headers['authorization']; if (!token) return res.status(401); }".into(),
            },
            EvaluationDocument {
                id: DocId(1),
                path: "auth/jwt.ts".into(),
                content: "export function verifyJwtToken(token: string, secret: string): TokenPayload { return jwt.verify(token, secret); }".into(),
            },
            EvaluationDocument {
                id: DocId(2),
                path: "crates/furlpay-daemon/src/auth.rs".into(),
                content: "pub fn validate(&self, token_hash: &[u8; 32]) -> AuthResult { AuthResult::Authorized { tenant_id: 1 } }".into(),
            },
            EvaluationDocument {
                id: DocId(3),
                path: "crates/furlpay-simd/src/lib.rs".into(),
                content: "pub fn intersect_posting_lists(a: &[u32], b: &[u32], output: &mut Vec<u32>) { furlpay_intersect_simd(small.as_ptr(), small.len(), large.as_ptr(), large.len(), output.as_mut_ptr(), capacity); }".into(),
            },
            EvaluationDocument {
                id: DocId(4),
                path: "crates/furlpay-ipc/src/protocol.rs".into(),
                content: "pub struct SearchRequestMessage { pub request_id: u64, pub tenant_token_hash: [u8; 32], pub query_type: u8 }".into(),
            },
        ];

        let queries = vec![
            QueryFixture {
                id: "q1".into(),
                category: "exact".into(),
                query: "authenticationMiddleware".into(),
                expected_file: "auth/middleware.ts".into(),
                description: "Find exact authentication middleware".into(),
            },
            QueryFixture {
                id: "q2".into(),
                category: "exact".into(),
                query: "verifyJwtToken".into(),
                expected_file: "auth/jwt.ts".into(),
                description: "Find JWT token verification".into(),
            },
            QueryFixture {
                id: "q3".into(),
                category: "symbol".into(),
                query: "intersect_posting_lists".into(),
                expected_file: "crates/furlpay-simd/src/lib.rs".into(),
                description: "Find SIMD intersection function".into(),
            },
            QueryFixture {
                id: "q4".into(),
                category: "symbol".into(),
                query: "SearchRequestMessage".into(),
                expected_file: "crates/furlpay-ipc/src/protocol.rs".into(),
                description: "Find IPC search request message struct".into(),
            },
            QueryFixture {
                id: "q5".into(),
                category: "exact".into(),
                query: "tenant_token_hash".into(),
                expected_file: "crates/furlpay-ipc/src/protocol.rs".into(),
                description: "Find tenant token HMAC hash field".into(),
            },
        ];

        Self { documents, queries }
    }

    pub fn load_from_dir(_dir: &Path) -> anyhow::Result<Self> {
        Ok(Self::default_fixture())
    }
}
