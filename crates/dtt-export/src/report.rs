//! Report artifact generation.

#![forbid(unsafe_code)]

use dtt_core::ExportDatasetV1;
use serde_json::{json, Value};

pub fn build_report_json(
    dataset: &ExportDatasetV1,
    file_count: usize,
    integrity_ok: bool,
) -> Value {
    json!({
        "v": 1,
        "session_id": dataset.session_id,
        "export_profile": dataset.export_profile.as_str(),
        "privacy_mode": dataset.privacy_mode.as_str(),
        "exported_at_ms": dataset.exported_at_ms,
        "row_counts": {
            "network_requests": dataset.normalized_network_requests.len(),
            "network_responses": dataset.normalized_network_responses.len(),
            "network_completion": dataset.normalized_network_completion.len(),
            "console_entries": dataset.normalized_console_entries.len(),
            "page_lifecycle": dataset.normalized_page_lifecycle.len(),
            "interactions": dataset.normalized_interactions.len(),
            "interaction_members": dataset.normalized_interaction_members.len(),
            "findings": dataset.analysis_findings.len(),
            "claims": dataset.analysis_claims.len(),
            "evidence_refs": dataset.analysis_evidence_refs.len(),
            "derived_metrics": dataset.analysis_derived_metrics.len(),
            "raw_events": dataset.raw_events.len()
        },
        "file_count": file_count,
        "integrity_ok": integrity_ok
    })
}

pub fn build_report_html(report_json: &Value) -> String {
    let session_id = report_json.get("session_id").and_then(Value::as_str).unwrap_or_default();
    let profile = report_json.get("export_profile").and_then(Value::as_str).unwrap_or_default();
    let privacy = report_json.get("privacy_mode").and_then(Value::as_str).unwrap_or_default();
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>DevTools Translator Export</title></head><body><h1>DevTools Translator Export</h1><p>Session: {session_id}</p><p>Profile: {profile}</p><p>Privacy: {privacy}</p><pre>{}</pre></body></html>",
        serde_json::to_string_pretty(report_json).unwrap_or_else(|_| "{}".to_string())
    )
}
