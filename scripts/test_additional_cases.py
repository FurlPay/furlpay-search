#!/usr/bin/env python3
"""
Test additional query cases:
- Natural language query (authentication, payment) -> content search
- Windows path separators (blockchain\\tron.ts) -> exact relative path
- Case-insensitive extension (TRON.TS) -> exact filename match
- Combined path and stub intent (blockchain/treesitter.rs stub) -> exact relative path + stub promotion
"""

import urllib.request
import urllib.parse
import json

BASE_URL = "http://127.0.0.1:3000/api/v1/search/stream"

TEST_CASES = [
    {
        "name": "Natural language content (payment)",
        "query": "payment",
        "expected_intent": "ContentSearch",
    },
    {
        "name": "Natural language content (authentication)",
        "query": "authentication",
        "expected_intent": "ContentSearch",
    },
    {
        "name": "Windows backslash path (blockchain\\tron.ts)",
        "query": "blockchain\\tron.ts",
        "expected_intent": "ExactRelativePath",
        "expected_top": "blockchain/tron.ts",
    },
    {
        "name": "Case-insensitive extension (TRON.TS)",
        "query": "TRON.TS",
        "expected_intent": "FilenameWithExtension",
        "expected_top": "blockchain/tron.ts",
    },
    {
        "name": "Combined path and stub query (ast/treesitter.rs stub)",
        "query": "ast/treesitter.rs stub",
        "expected_intent": "ExactRelativePath",
        "expected_top": "crates/furlpay-core/src/ast/treesitter.rs",
    },
    {
        "name": "Filename with extension and stub (treesitter.rs stub)",
        "query": "treesitter.rs stub",
        "expected_intent": "FilenameWithExtension",
        "expected_top": "crates/furlpay-core/src/ast/treesitter.rs",
    },
]

def fetch_events(query):
    params = urllib.parse.urlencode({"q": query, "mode": "code"})
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

for tc in TEST_CASES:
    q = tc["query"]
    events = fetch_events(q)
    classified = next((e["data"] for e in events if e["event"] == "search.classified"), None)
    inter_res = next((e["data"] for e in events if e["event"] == "search.results" and e["data"].get("is_intermediate")), None)
    final_res = next((e["data"] for e in events if e["event"] == "search.results" and not e["data"].get("is_intermediate")), None)
    completed = next((e["data"] for e in events if e["event"] == "search.completed"), None)

    intent = classified.get("query_intent") if classified else None
    results = []
    if final_res and final_res.get("results"):
        results = final_res["results"]
    elif inter_res and inter_res.get("results"):
        results = inter_res["results"]

    top_result = results[0]["file_path"] if results else "NONE"
    top_score = results[0]["score"] if results else 0.0
    top_reason = results[0].get("ranking_reason") if results else "NONE"

    print(f"Test: {tc['name']}")
    print(f"  Query: '{q}'")
    print(f"  Classified Intent: {intent} (Expected: {tc['expected_intent']})")
    print(f"  Top Result: {top_result}")
    print(f"  Score: {top_score:.3f} | Reason: {top_reason}")
    print(f"  Total Results: {completed.get('total_results') if completed else 0}")
    print("-" * 70)
