//! JSON content extraction.

/// Extracted content from a JSON document.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractedJson {
    pub parsed: serde_json::Value,
    pub summary: String,
    pub key_count: usize,
}

/// Extract and summarize JSON content.
pub fn extract_json(text: &str) -> Result<ExtractedJson, String> {
    let parsed: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let key_count = count_keys(&parsed);
    let summary = summarize_json(&parsed, 0, 500);

    Ok(ExtractedJson {
        parsed,
        summary,
        key_count,
    })
}

fn count_keys(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(map) => {
            map.len() + map.values().map(count_keys).sum::<usize>()
        }
        serde_json::Value::Array(arr) => arr.iter().map(count_keys).sum(),
        _ => 0,
    }
}

fn summarize_json(value: &serde_json::Value, depth: usize, max_chars: usize) -> String {
    if depth > 3 {
        return "...".to_string();
    }
    match value {
        serde_json::Value::Object(map) => {
            let keys: Vec<&String> = map.keys().take(10).collect();
            let key_list = keys.iter().map(|k| k.as_str()).collect::<Vec<_>>().join(", ");
            let mut s = format!("Object with {} keys: {}", map.len(), key_list);
            if s.len() > max_chars {
                s.truncate(max_chars);
                s.push_str("...");
            }
            s
        }
        serde_json::Value::Array(arr) => {
            format!("Array with {} elements", arr.len())
        }
        serde_json::Value::String(s) => {
            if s.len() > 100 {
                format!("\"{}...\"", &s[..100])
            } else {
                format!("\"{}\"", s)
            }
        }
        other => other.to_string(),
    }
}
