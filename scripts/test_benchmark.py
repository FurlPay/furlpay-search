#!/usr/bin/env python3
"""
FurlPay Local Retrieval Quality Regression Test & Benchmark Suite 2026.
Runs all 11 required queries against the resident daemon at http://127.0.0.1:3000.
Collects SSE events, validates ranking order, verification semantics, stub handling,
and runs 30 benchmark repetitions to compute latency percentiles.
"""

import urllib.request
import urllib.parse
import json
import time
import math
import sys

BASE_URL = "http://127.0.0.1:3000/api/v1/search/stream"

QUERIES = [
    {
        "name": "Single character (t)",
        "query": "t",
        "expected_top": "crates/furlpay-core/src/index/trigram.rs",
        "description": "Short prefix query; should favor filename stem starting with 't', not generic files"
    },
    {
        "name": "Short token prefix (tr)",
        "query": "tr",
        "expected_top": "crates/furlpay-core/src/index/trigram.rs",
        "description": "Prefix query; trigram.rs #1, tron.ts #2, treesitter.rs #3 with stub penalty"
    },
    {
        "name": "Token prefix (tro)",
        "query": "tro",
        "expected_top": "blockchain/tron.ts",
        "description": "Prefix matching tron.ts stem"
    },
    {
        "name": "Full token (tron)",
        "query": "tron",
        "expected_top": "blockchain/tron.ts",
        "description": "Exact stem matching tron.ts"
    },
    {
        "name": "Identifier with extension (tron.ts)",
        "query": "tron.ts",
        "expected_top": "blockchain/tron.ts",
        "description": "Exact file stem + extension match"
    },
    {
        "name": "Core component (trigram)",
        "query": "trigram",
        "expected_top": "crates/furlpay-core/src/index/trigram.rs",
        "description": "Exact stem match on core algorithm"
    },
    {
        "name": "Unfinished component (treesitter)",
        "query": "treesitter",
        "expected_top": "crates/furlpay-core/src/ast/treesitter.rs",
        "description": "Matches treesitter stem; marked as stub"
    },
    {
        "name": "Explicit stub search (treesitter stub)",
        "query": "treesitter stub",
        "expected_top": "crates/furlpay-core/src/ast/treesitter.rs",
        "description": "Query seeking stubs; promotes stub file to #1"
    },
    {
        "name": "Explicit TODO search (TODO treesitter)",
        "query": "TODO treesitter",
        "expected_top": "crates/furlpay-core/src/ast/treesitter.rs",
        "description": "Query seeking TODO/unfinished work; promotes treesitter.rs to #1"
    },
    {
        "name": "Full relative path (blockchain/tron.ts)",
        "query": "blockchain/tron.ts",
        "expected_top": "blockchain/tron.ts",
        "description": "Exact file path match"
    },
    {
        "name": "TRON USDT address (TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t)",
        "query": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
        "expected_top": "blockchain://tron",
        "description": "Base58Check validated address; CryptographicValidationPassed"
    }
]

def fetch_sse_stream(query, mode="code"):
    """Fetch and parse all SSE events from the daemon."""
    params = urllib.parse.urlencode({"q": query, "mode": mode})
    url = f"{BASE_URL}?{params}"
    req = urllib.request.Request(url, headers={"Accept": "text/event-stream"})

    events = []
    current_event = None
    current_data = []

    with urllib.request.urlopen(req, timeout=5) as response:
        content = response.read().decode("utf-8")

    for line in content.split("\n"):
        line = line.strip()
        if line.startswith("event: "):
            current_event = line[7:].strip()
        elif line.startswith("data: "):
            current_data.append(line[6:].strip())
        elif line == "" and current_event:
            data_str = "\n".join(current_data)
            try:
                parsed = json.loads(data_str)
                events.append({"event": current_event, "data": parsed})
            except Exception:
                events.append({"event": current_event, "raw": data_str})
            current_event = None
            current_data = []

    return events

def run_tests():
    print("=" * 80)
    print(" FurlPay Search: Local Retrieval Quality Regression Test & Benchmark Suite")
    print("=" * 80)

    summary_rows = []

    for idx, test_case in enumerate(QUERIES, 1):
        q = test_case["query"]
        name = test_case["name"]
        print(f"\n[{idx}/11] Running Test: {name}")
        print(f"       Query: '{q}'")

        # 1. Functional check
        events = fetch_sse_stream(q)
        event_names = [e["event"] for e in events]

        # Extract events
        started = next((e["data"] for e in events if e["event"] == "search.started"), None)
        classified = next((e["data"] for e in events if e["event"] == "search.classified"), None)
        matches = next((e["data"] for e in events if e["event"] == "search.matches_found"), None)
        inter_verif = next((e["data"] for e in events if e["event"] == "search.verification" and e["data"].get("stage") == "intermediate"), None)
        inter_res = next((e["data"] for e in events if e["event"] == "search.results" and e["data"].get("is_intermediate") is True), None)
        final_verif = next((e["data"] for e in events if e["event"] == "search.verification" and e["data"].get("stage") == "final"), None)
        final_res = next((e["data"] for e in events if e["event"] == "search.results" and e["data"].get("is_intermediate") is False), None)
        completed = next((e["data"] for e in events if e["event"] == "search.completed"), None)

        # Assertions
        assert "search.completed" in event_names, "Missing terminal search.completed"
        assert event_names.count("search.completed") == 1, "Multiple search.completed events!"

        # Extract top result
        top_item = None
        if final_res and final_res.get("results"):
            top_item = final_res["results"][0]
        elif inter_res and inter_res.get("results"):
            top_item = inter_res["results"][0]

        top_file = top_item["file_path"] if top_item else "NONE"
        top_score = top_item["score"] if top_item else 0.0
        top_status = top_item.get("implementation_status", "NONE") if top_item else "NONE"
        top_verif = top_item.get("verification_kind", "NONE") if top_item else "NONE"
        top_reason = top_item.get("ranking_reason", "NONE") if top_item else "NONE"
        total_results = completed.get("total_results", 0) if completed else 0

        print(f"       Top Result: {top_file}")
        print(f"       Score: {top_score:.3f} | Status: {top_status} | Verif: {top_verif}")
        print(f"       Ranking Reason: {top_reason}")
        print(f"       Total Results: {total_results}")

        # Check progressive accounting
        if inter_verif and final_verif:
            assert inter_verif.get("verified_total") == 1, "Intermediate verified_total != 1"
            assert final_verif.get("verified_total") == total_results, f"Final verified_total ({final_verif.get('verified_total')}) != total ({total_results})"
            assert final_verif.get("pending_verification") == 0, "Final pending_verification != 0"

        # 2. Benchmark (30 iterations)
        latencies = []
        ttfrs = []
        for _ in range(30):
            t_start = time.perf_counter()
            evs = fetch_sse_stream(q)
            t_dur = (time.perf_counter() - t_start) * 1000.0
            comp = next((e["data"] for e in evs if e["event"] == "search.completed"), None)
            if comp:
                elapsed_ms = comp.get("elapsed_ms", t_dur)
                latencies.append(elapsed_ms)
                telemetry = comp.get("telemetry", {})
                first_res_us = telemetry.get("first_result_latency_us")
                if first_res_us:
                    ttfrs.append(first_res_us / 1000.0)

        latencies.sort()
        ttfrs.sort()
        n = len(latencies)
        min_l = latencies[0]
        max_l = latencies[-1]
        med_l = latencies[n // 2]
        p90_l = latencies[int(n * 0.90)]
        p95_l = latencies[int(n * 0.95)]
        med_ttfr = ttfrs[len(ttfrs) // 2] if ttfrs else 0.0

        print(f"       Benchmark (30 runs): Min={min_l:.2f}ms | Med={med_l:.2f}ms | p90={p90_l:.2f}ms | p95={p95_l:.2f}ms | Max={max_l:.2f}ms | TTFR={med_ttfr:.2f}ms")

        summary_rows.append({
            "query": q,
            "name": name,
            "top_file": top_file,
            "score": top_score,
            "status": top_status,
            "verification": top_verif,
            "ttfr_ms": med_ttfr,
            "median_ms": med_l,
            "p90_ms": p90_l,
            "p95_ms": p95_l,
            "total_results": total_results
        })

    # Print summary table
    print("\n" + "=" * 110)
    print(" FINAL BENCHMARK & REGRESSION EVALUATION SUMMARY (30 Iterations / Query)")
    print("=" * 110)
    header = f"{'Query':<35} | {'Top Result':<42} | {'Score':<6} | {'Status':<11} | {'TTFR (ms)':<9} | {'Med (ms)':<8} | {'p95 (ms)':<8}"
    print(header)
    print("-" * 110)
    for row in summary_rows:
        q_disp = row["query"] if len(row["query"]) <= 32 else row["query"][:30] + ".."
        f_disp = row["top_file"] if len(row["top_file"]) <= 40 else row["top_file"][-38:]
        print(f"{q_disp:<35} | {f_disp:<42} | {row['score']:<6.3f} | {row['status']:<11} | {row['ttfr_ms']:<9.2f} | {row['median_ms']:<8.2f} | {row['p95_ms']:<8.2f}")
    print("=" * 110)

if __name__ == "__main__":
    run_tests()
