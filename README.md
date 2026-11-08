# FurlPay Search Engine

> High-performance AI agent retrieval runtime — sub-millisecond code search, tri-modal web retrieval, MCP server.

[![Rust](https://img.shields.io/badge/Rust-1.80+-dea584?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE-MIT)
[![📖 Live Docs](https://img.shields.io/badge/📖_Live_Docs-Docsify-4B8BBE)](https://furlpay.github.io/docs/)

## Architecture

Project FurlPay is a Cargo workspace with 7 crates:

| Crate | Purpose |
|-------|---------|
| `furlpay-core` | Positional trigram index, BM25, AST symbol graph, query routing, RRF ranking |
| `furlpay-simd` | Multi-target C++20 SIMD kernels (AVX-512, AVX2, NEON) with safe Rust FFI |
| `furlpay-ipc` | Zero-copy shared-memory SPSC ring buffer for daemon–client communication |
| `furlpay-web` | Tri-modal web retrieval: Edge Vault cache, QUIC SERP racing, Lexbor parser |
| `furlpay-daemon` | Resident daemon (`furlpayd`) with file watcher and index lifecycle |
| `furlpay-mcp` | Model Context Protocol stdio server for Claude/Cursor/Antigravity |
| `furlpay-eval` | Criterion benchmarks, MRR evaluation, and OpenTelemetry telemetry |

## Quick Start

```bash
# Build workspace
cargo build --release

# Run daemon
cargo run -p furlpay-daemon -- --workspace /path/to/repo

# Run MCP server
cargo run -p furlpay-mcp

# Run benchmarks
cargo bench -p furlpay-eval
```

## Performance Targets

| Metric | Target | Measured |
|--------|--------|----------|
| Code search p50 | < 0.35 ms | — |
| Code search p99 | < 0.50 ms | — |
| MRR (CodeSearchNet) | > 0.435 | — |
| IPC roundtrip | < 16 µs | — |

## Documentation

📖 **[Read the Full Technical Master Plan](https://furlpay.github.io/docs/#/docs/PROJECT-FURLPAY-SEARCH-IMPLEMENTATION-MASTER-PLAN)**

## License

MIT — see [LICENSE-MIT](LICENSE-MIT).
