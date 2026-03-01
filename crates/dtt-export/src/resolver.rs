//! Offline evidence resolution for export bundles.

#![forbid(unsafe_code)]

use crate::model::ExportEvidenceResolveResultV1;
use crate::writer::read_zip_files;
use crate::{ExportError, Result};
use dtt_core::{EvidenceKind, EvidenceRefV1, EvidenceTarget, NetTable};
use serde_json::Value;
use std::path::Path;

pub fn resolve_evidence_from_bundle(
    bundle_path: impl AsRef<Path>,
    evidence_ref_id: &str,
) -> Result<Option<ExportEvidenceResolveResultV1>> {
    let files = read_zip_files(bundle_path.as_ref())?;
    let evidence_rows =
        parse_ndjson(files.get("analysis/evidence_refs.ndjson").ok_or_else(|| {
            ExportError::InvalidDataset("missing analysis/evidence_refs.ndjson".to_string())
        })?)?;
    let mut matched_ref_json: Option<&str> = None;
    for row in &evidence_rows {
        if row.get("evidence_ref_id").and_then(Value::as_str) == Some(evidence_ref_id) {
            matched_ref_json = row.get("ref_json").and_then(Value::as_str);
            break;
        }
    }
    let Some(ref_json_raw) = matched_ref_json else {
        return Ok(None);
    };
    let evidence_ref: EvidenceRefV1 = serde_json::from_str(ref_json_raw)?;

    let result = match &evidence_ref.target {
        EvidenceTarget::RawEvent(target) => {
            let container = resolve_raw_event(
                &files,
                target.event_id.as_str(),
                target.json_pointer.as_deref(),
            )?;
            ExportEvidenceResolveResultV1 {
                evidence_ref_id: evidence_ref_id.to_string(),
                session_id: evidence_ref.session_id,
                kind: EvidenceKind::RawEvent,
                target_id: target.event_id.clone(),
                exact_pointer_found: container.1,
                fallback_reason: if container.1 {
                    None
                } else {
                    Some("Exact pointer unavailable".to_string())
                },
                container_json: Some(container.0.clone()),
                highlighted_value: container.2,
            }
        }
        EvidenceTarget::NetRow(target) => {
            let container = resolve_network_row(
                &files,
                target.table,
                target.net_request_id.as_str(),
                target.json_pointer.as_deref(),
            )?;
            ExportEvidenceResolveResultV1 {
                evidence_ref_id: evidence_ref_id.to_string(),
                session_id: evidence_ref.session_id,
                kind: EvidenceKind::NetRow,
                target_id: target.net_request_id.clone(),
                exact_pointer_found: container.1,
                fallback_reason: if container.1 {
                    None
                } else {
                    Some("Exact pointer unavailable".to_string())
                },
                container_json: Some(container.0.clone()),
                highlighted_value: container.2,
            }
        }
        EvidenceTarget::Console(target) => {
            let container = resolve_console_row(
                &files,
                target.console_id.as_str(),
                target.json_pointer.as_deref(),
            )?;
            ExportEvidenceResolveResultV1 {
                evidence_ref_id: evidence_ref_id.to_string(),
                session_id: evidence_ref.session_id,
                kind: EvidenceKind::Console,
                target_id: target.console_id.clone(),
                exact_pointer_found: container.1,
                fallback_reason: if container.1 {
                    None
                } else {
                    Some("Exact pointer unavailable".to_string())
                },
                container_json: Some(container.0.clone()),
                highlighted_value: container.2,
            }
        }
        EvidenceTarget::DerivedMetric(target) => {
            let container = resolve_derived_metric(&files, evidence_ref_id, Some("/value"))?;
            ExportEvidenceResolveResultV1 {
                evidence_ref_id: evidence_ref_id.to_string(),
                session_id: evidence_ref.session_id,
                kind: EvidenceKind::DerivedMetric,
                target_id: target.metric_name.clone(),
                exact_pointer_found: container.1,
                fallback_reason: if container.1 {
                    None
                } else {
                    Some("Exact pointer unavailable".to_string())
                },
                container_json: Some(container.0.clone()),
                highlighted_value: container.2,
            }
        }
    };
    Ok(Some(result))
}

fn resolve_raw_event(
    files: &std::collections::BTreeMap<String, Vec<u8>>,
    event_id: &str,
    json_pointer: Option<&str>,
) -> Result<(Value, bool, Option<Value>)> {
    let index_rows = parse_ndjson(files.get("raw/events.index.ndjson").ok_or_else(|| {
        ExportError::InvalidDataset("missing raw/events.index.ndjson".to_string())
    })?)?;
    let line = index_rows
        .iter()
        .find(|row| row.get("event_id").and_then(Value::as_str) == Some(event_id))
        .and_then(|row| row.get("line").and_then(Value::as_u64))
        .ok_or_else(|| {
            ExportError::InvalidDataset("raw event id missing from index".to_string())
        })?;
    let events_bytes = files
        .get("raw/events.ndjson.zst")
        .ok_or_else(|| ExportError::InvalidDataset("missing raw/events.ndjson.zst".to_string()))?;
    let decompressed = zstd::stream::decode_all(std::io::Cursor::new(events_bytes))?;
    let rows = parse_ndjson(&decompressed)?;
    let line_idx = usize::try_from(line.saturating_sub(1)).unwrap_or(0);
    let container = rows.get(line_idx).cloned().ok_or_else(|| {
        ExportError::InvalidDataset("raw event index line out of bounds".to_string())
    })?;
    Ok(pointer_extract(container, json_pointer))
}

fn resolve_network_row(
    files: &std::collections::BTreeMap<String, Vec<u8>>,
    table: NetTable,
    net_request_id: &str,
    json_pointer: Option<&str>,
) -> Result<(Value, bool, Option<Value>)> {
    let index_rows =
        parse_ndjson(files.get("normalized/network.index.ndjson").ok_or_else(|| {
            ExportError::InvalidDataset("missing normalized/network.index.ndjson".to_string())
        })?)?;
    let table_name = match table {
        NetTable::NetworkRequests => "network_requests",
        NetTable::NetworkResponses => "network_responses",
        NetTable::NetworkCompletion => "network_completion",
    };
    let line = index_rows
        .iter()
        .find(|row| {
            row.get("table").and_then(Value::as_str) == Some(table_name)
                && row.get("net_request_id").and_then(Value::as_str) == Some(net_request_id)
        })
        .and_then(|row| row.get("line").and_then(Value::as_u64))
        .ok_or_else(|| ExportError::InvalidDataset("network id missing from index".to_string()))?;

    let file_path = match table {
        NetTable::NetworkRequests => "normalized/network_requests.ndjson",
        NetTable::NetworkResponses => "normalized/network_responses.ndjson",
        NetTable::NetworkCompletion => "normalized/network_completion.ndjson",
    };
    let rows = parse_ndjson(
        files
            .get(file_path)
            .ok_or_else(|| ExportError::InvalidDataset(format!("missing {file_path}")))?,
    )?;
    let line_idx = usize::try_from(line.saturating_sub(1)).unwrap_or(0);
    let container = rows.get(line_idx).cloned().ok_or_else(|| {
        ExportError::InvalidDataset("network index line out of bounds".to_string())
    })?;
    Ok(pointer_extract(container, json_pointer))
}

fn resolve_console_row(
    files: &std::collections::BTreeMap<String, Vec<u8>>,
    console_id: &str,
    json_pointer: Option<&str>,
) -> Result<(Value, bool, Option<Value>)> {
    let index_rows =
        parse_ndjson(files.get("normalized/console.index.ndjson").ok_or_else(|| {
            ExportError::InvalidDataset("missing normalized/console.index.ndjson".to_string())
        })?)?;
    let line = index_rows
        .iter()
        .find(|row| row.get("console_id").and_then(Value::as_str) == Some(console_id))
        .and_then(|row| row.get("line").and_then(Value::as_u64))
        .ok_or_else(|| ExportError::InvalidDataset("console id missing from index".to_string()))?;
    let rows = parse_ndjson(files.get("normalized/console_entries.ndjson").ok_or_else(|| {
        ExportError::InvalidDataset("missing normalized/console_entries.ndjson".to_string())
    })?)?;
    let line_idx = usize::try_from(line.saturating_sub(1)).unwrap_or(0);
    let container = rows.get(line_idx).cloned().ok_or_else(|| {
        ExportError::InvalidDataset("console index line out of bounds".to_string())
    })?;
    Ok(pointer_extract(container, json_pointer))
}

fn resolve_derived_metric(
    files: &std::collections::BTreeMap<String, Vec<u8>>,
    evidence_ref_id: &str,
    json_pointer: Option<&str>,
) -> Result<(Value, bool, Option<Value>)> {
    let index_rows =
        parse_ndjson(files.get("analysis/derived_metrics.index.ndjson").ok_or_else(|| {
            ExportError::InvalidDataset("missing analysis/derived_metrics.index.ndjson".to_string())
        })?)?;
    let line = index_rows
        .iter()
        .find(|row| row.get("evidence_ref_id").and_then(Value::as_str) == Some(evidence_ref_id))
        .and_then(|row| row.get("line").and_then(Value::as_u64))
        .ok_or_else(|| {
            ExportError::InvalidDataset("derived metric evidence id missing from index".to_string())
        })?;
    let rows = parse_ndjson(files.get("analysis/derived_metrics.ndjson").ok_or_else(|| {
        ExportError::InvalidDataset("missing analysis/derived_metrics.ndjson".to_string())
    })?)?;
    let line_idx = usize::try_from(line.saturating_sub(1)).unwrap_or(0);
    let container = rows.get(line_idx).cloned().ok_or_else(|| {
        ExportError::InvalidDataset("derived metrics index line out of bounds".to_string())
    })?;
    Ok(pointer_extract(container, json_pointer))
}

fn pointer_extract(container: Value, json_pointer: Option<&str>) -> (Value, bool, Option<Value>) {
    let Some(pointer) = json_pointer else {
        return (container, true, None);
    };
    let value = container.pointer(pointer).cloned();
    match value {
        Some(value) => (container, true, Some(value)),
        None => (container, false, None),
    }
}

fn parse_ndjson(bytes: &[u8]) -> Result<Vec<Value>> {
    let mut output = Vec::new();
    for line in String::from_utf8_lossy(bytes).lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        output.push(serde_json::from_str::<Value>(trimmed)?);
    }
    Ok(output)
}
