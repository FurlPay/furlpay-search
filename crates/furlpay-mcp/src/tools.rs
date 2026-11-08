//! MCP tool implementations.

use serde_json::Value;

pub async fn call_tool(name: &str, arguments: &Value) -> String {
    match name {
        "furlpay_search_code" => search_code(arguments).await,
        "furlpay_search_web" => search_web(arguments).await,
        _ => format!("Unknown tool: {}", name),
    }
}

async fn search_code(args: &Value) -> String {
    let query = args["query"].as_str().unwrap_or("");
    let query_type = args["query_type"].as_str().unwrap_or("hybrid");
    let max_results = args["max_results"].as_u64().unwrap_or(10);

    tracing::info!(%query, %query_type, max_results, "furlpay_search_code called");

    let decision = furlpay_core::routing::classifier::QueryRouter::classify(query);

    format!(
        "Query: {}\nClassification: {:?}\nPrimary pipeline: {:?}\nSecondary pipeline: {:?}\n(Full search pipeline pending — Milestone 1 Phase 2-4)",
        query, decision.query_class, decision.primary, decision.secondary
    )
}

async fn search_web(args: &Value) -> String {
    let query = args["query"].as_str().unwrap_or("");
    tracing::info!(%query, "furlpay_search_web called");
    format!(
        "Web search for: {}\n(Milestone 2 — Web retrieval not yet implemented)",
        query
    )
}
