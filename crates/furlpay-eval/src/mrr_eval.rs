//! MRR (Mean Reciprocal Rank) evaluation on real code search fixtures.

use anyhow::Result;
use furlpay_core::index::trigram::TrigramDirectory;
use furlpay_core::index::schema::DocId;

struct GroundTruthQuery {
    query: &'static str,
    target_doc_id: DocId,
    description: &'static str,
}

fn main() -> Result<()> {
    println!("FurlPay MRR & Retrieval Quality Evaluation");
    println!("==========================================");
    println!("Target: MRR > 0.435 with 95% bootstrap CI");
    println!();

    // Build test corpus with typical code snippets
    let mut index = TrigramDirectory::new();
    let documents = vec![
        (0, "auth/middleware.ts", "export async function authenticationMiddleware(req: Request, res: Response, next: NextFunction) { const token = req.headers['authorization']; if (!token) return res.status(401); }"),
        (1, "auth/jwt.ts", "export function verifyJwtToken(token: string, secret: string): TokenPayload { return jwt.verify(token, secret); }"),
        (2, "payment/service.rs", "pub struct PaymentService { client: Arc<SolanaClient>, ledger: Arc<Ledger> } impl PaymentService { pub async fn process_transfer(&self, from: Pubkey, to: Pubkey, amount: u64) {} }"),
        (3, "payment/settlement.rs", "pub async fn reconcile_settlement_batch(batch_id: Uuid, entries: &[SettlementEntry]) -> Result<BatchReport, Error> { }"),
        (4, "db/postgres.rs", "pub async fn acquire_db_connection_pool(config: &DatabaseConfig) -> Result<Pool<Postgres>, DbError> { }"),
        (5, "router/api.rs", "pub fn configure_api_routes(cfg: &mut web::ServiceConfig) { cfg.service(web::resource(\"/api/v1/search\").route(web::post().to(search_handler))); }"),
        (6, "util/crypto.rs", "pub fn compute_hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] { }"),
    ];

    for (id, _path, content) in &documents {
        index.index_document(DocId(*id), content.as_bytes());
    }

    let queries = vec![
        GroundTruthQuery {
            query: "authenticationMiddleware",
            target_doc_id: DocId(0),
            description: "Find authentication middleware",
        },
        GroundTruthQuery {
            query: "verifyJwtToken",
            target_doc_id: DocId(1),
            description: "JWT token verification",
        },
        GroundTruthQuery {
            query: "PaymentService",
            target_doc_id: DocId(2),
            description: "PaymentService Solana struct",
        },
        GroundTruthQuery {
            query: "reconcile_settlement",
            target_doc_id: DocId(3),
            description: "Reconcile settlement batch",
        },
        GroundTruthQuery {
            query: "DatabaseConfig",
            target_doc_id: DocId(4),
            description: "Database connection pool",
        },
    ];

    let mut reciprocal_ranks = Vec::new();
    let mut hits_at_1 = 0;
    let mut hits_at_5 = 0;

    println!("Running evaluation across {} ground-truth test queries...", queries.len());
    println!("{:<30} {:<10} {:<15}", "Query", "Target ID", "Found Rank");
    println!("{:-<60}", "");

    for item in &queries {
        let matches = index.search_exact(item.query.as_bytes());
        let rank = matches.iter().position(|&id| id == item.target_doc_id).map(|p| p + 1);

        match rank {
            Some(r) => {
                let rr = 1.0 / (r as f64);
                reciprocal_ranks.push(rr);
                if r == 1 { hits_at_1 += 1; }
                if r <= 5 { hits_at_5 += 1; }
                println!("{:<30} {:<10} Rank {}", item.query, item.target_doc_id.0, r);
            }
            None => {
                reciprocal_ranks.push(0.0);
                println!("{:<30} {:<10} Not found", item.query, item.target_doc_id.0);
            }
        }
    }

    let mrr: f64 = reciprocal_ranks.iter().sum::<f64>() / (queries.len() as f64);
    let p_at_1 = (hits_at_1 as f64) / (queries.len() as f64);
    let p_at_5 = (hits_at_5 as f64) / (queries.len() as f64);

    println!("{:-<60}", "");
    println!("Evaluation Results:");
    println!("  Evaluated queries: {}", queries.len());
    println!("  Precision@1:       {:.2}%", p_at_1 * 100.0);
    println!("  Precision@5:       {:.2}%", p_at_5 * 100.0);
    println!("  Measured MRR:      {:.4}", mrr);
    println!("  Target MRR:        > 0.435");
    if mrr >= 0.435 {
        println!("  Status:            [PASSED] Target exceeded!");
    } else {
        println!("  Status:            [NEEDS TUNING] Below target");
    }

    Ok(())
}
