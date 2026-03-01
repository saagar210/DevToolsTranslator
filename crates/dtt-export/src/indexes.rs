//! Deterministic index builders.

#![forbid(unsafe_code)]

use serde_json::{json, Value};

pub fn build_raw_events_index(raw_events: &[Value]) -> Vec<Value> {
    let mut output = Vec::new();
    for (idx, row) in raw_events.iter().enumerate() {
        let Some(event_id) = row.get("event_id").and_then(Value::as_str) else {
            continue;
        };
        output.push(json!({
            "event_id": event_id,
            "line": idx + 1
        }));
    }
    output
}

pub fn build_network_index(
    requests: &[Value],
    responses: &[Value],
    completion: &[Value],
) -> Vec<Value> {
    let mut output = Vec::new();
    for (line, row) in requests.iter().enumerate() {
        if let Some(id) = row.get("net_request_id").and_then(Value::as_str) {
            output.push(json!({
                "table": "network_requests",
                "net_request_id": id,
                "line": line + 1
            }));
        }
    }
    for (line, row) in responses.iter().enumerate() {
        if let Some(id) = row.get("net_request_id").and_then(Value::as_str) {
            output.push(json!({
                "table": "network_responses",
                "net_request_id": id,
                "line": line + 1
            }));
        }
    }
    for (line, row) in completion.iter().enumerate() {
        if let Some(id) = row.get("net_request_id").and_then(Value::as_str) {
            output.push(json!({
                "table": "network_completion",
                "net_request_id": id,
                "line": line + 1
            }));
        }
    }
    output.sort_by(|left, right| {
        left.get("table")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .cmp(right.get("table").and_then(Value::as_str).unwrap_or_default())
            .then(
                left.get("net_request_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .cmp(right.get("net_request_id").and_then(Value::as_str).unwrap_or_default()),
            )
    });
    output
}

pub fn build_console_index(console_entries: &[Value]) -> Vec<Value> {
    let mut output = Vec::new();
    for (idx, row) in console_entries.iter().enumerate() {
        if let Some(console_id) = row.get("console_id").and_then(Value::as_str) {
            output.push(json!({
                "console_id": console_id,
                "line": idx + 1
            }));
        }
    }
    output
}

pub fn build_derived_metrics_index(derived_metrics: &[Value]) -> Vec<Value> {
    let mut output = Vec::new();
    for (idx, row) in derived_metrics.iter().enumerate() {
        if let Some(evidence_ref_id) = row.get("evidence_ref_id").and_then(Value::as_str) {
            output.push(json!({
                "evidence_ref_id": evidence_ref_id,
                "line": idx + 1
            }));
        }
    }
    output
}
