//! JSON Schema definitions for MCP tool contracts.

use serde_json::{json, Value};

pub fn code_search_schema() -> Value {
    json!({
        "name": "furlpay_search_code",
        "description": "High-performance sub-millisecond code retrieval across workspace index",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search term, regex pattern, or symbol name"
                },
                "query_type": {
                    "type": "string",
                    "enum": ["exact", "regex", "symbol", "hybrid"],
                    "default": "hybrid"
                },
                "path_glob": {
                    "type": "string",
                    "description": "Optional file filter (e.g. src/**/*.rs)"
                },
                "max_results": { "type": "integer", "default": 10 },
                "timeout_ms": { "type": "integer", "default": 500 }
            },
            "required": ["query"]
        }
    })
}

pub fn web_search_schema() -> Value {
    json!({
        "name": "furlpay_search_web",
        "description": "Tri-modal web retrieval across edge documentation vault and live SERP",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Technical query or API documentation request"
                },
                "require_crawl": { "type": "boolean", "default": false },
                "freshness": {
                    "type": "string",
                    "enum": ["day", "week", "month", "any"],
                    "default": "any"
                }
            },
            "required": ["query"]
        }
    })
}
