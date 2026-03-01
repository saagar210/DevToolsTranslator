//! Export engine v1.0.

#![forbid(unsafe_code)]

mod indexes;
mod model;
mod privacy;
mod report;
mod resolver;
mod writer;
mod zip;

pub use model::{ExportEvidenceResolveResultV1, ExportWriteRequestV1, ExportWriteResultV1};
pub use resolver::resolve_evidence_from_bundle;
pub use writer::{export_session, ExportError, Result};

#[cfg(test)]
mod tests {
    use super::{export_session, resolve_evidence_from_bundle, ExportWriteRequestV1};
    use dtt_core::{EvidenceKind, ExportDatasetV1, ExportProfileV1, RedactionLevel};
    use dtt_integrity::verify_bundle_contents;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn sample_dataset(privacy_mode: RedactionLevel, profile: ExportProfileV1) -> ExportDatasetV1 {
        let evidence_ref_id = "evr_1";
        ExportDatasetV1 {
            session_id: "sess_export_1".to_string(),
            privacy_mode,
            export_profile: profile,
            exported_at_ms: 1_729_123_000_000,
            session_json: json!({
                "session_id":"sess_export_1",
                "privacy_mode": privacy_mode.as_str()
            }),
            normalized_network_requests: vec![json!({
                "net_request_id":"net_1",
                "status_code":200
            })],
            normalized_network_responses: vec![json!({
                "net_request_id":"net_1",
                "status_code":200
            })],
            normalized_network_completion: vec![json!({
                "net_request_id":"net_1",
                "success":true
            })],
            normalized_console_entries: vec![json!({
                "console_id":"con_1",
                "message_redacted":"ok"
            })],
            normalized_page_lifecycle: vec![json!({
                "lifecycle_id":"life_1",
                "name":"load"
            })],
            normalized_interactions: vec![json!({
                "interaction_id":"int_1",
                "session_id":"sess_export_1"
            })],
            normalized_interaction_members: vec![json!({
                "interaction_id":"int_1",
                "member_type":"network_response",
                "member_id":"net_1"
            })],
            analysis_findings: vec![json!({
                "finding_id":"fnd_1",
                "session_id":"sess_export_1"
            })],
            analysis_claims: vec![json!({
                "claim_id":"clm_1",
                "finding_id":"fnd_1"
            })],
            analysis_evidence_refs: vec![
                json!({
                    "evidence_ref_id":"evr_raw",
                    "claim_id":"clm_1",
                    "evidence_rank":1,
                    "ref_json": serde_json::to_string(&json!({
                        "v":1,
                        "kind":"raw_event",
                        "session_id":"sess_export_1",
                        "label":"raw",
                        "ts_ms":1,
                        "redaction_level":"metadata_only",
                        "target":{"event_id":"evt_1","cdp_method":"Network.requestWillBeSent","json_pointer":"/event_id"}
                    })).expect("raw ref json")
                }),
                json!({
                    "evidence_ref_id":"evr_net",
                    "claim_id":"clm_1",
                    "evidence_rank":2,
                    "ref_json": serde_json::to_string(&json!({
                        "v":1,
                        "kind":"net_row",
                        "session_id":"sess_export_1",
                        "label":"net",
                        "ts_ms":1,
                        "redaction_level":"metadata_only",
                        "target":{"net_request_id":"net_1","table":"network_responses","json_pointer":"/status_code"}
                    })).expect("net ref json")
                }),
                json!({
                    "evidence_ref_id":"evr_console",
                    "claim_id":"clm_1",
                    "evidence_rank":3,
                    "ref_json": serde_json::to_string(&json!({
                        "v":1,
                        "kind":"console",
                        "session_id":"sess_export_1",
                        "label":"console",
                        "ts_ms":1,
                        "redaction_level":"metadata_only",
                        "target":{"console_id":"con_1","json_pointer":"/console_id"}
                    })).expect("console ref json")
                }),
                json!({
                    "evidence_ref_id":evidence_ref_id,
                    "claim_id":"clm_1",
                    "evidence_rank":4,
                    "ref_json": serde_json::to_string(&json!({
                        "v":1,
                        "kind":"derived_metric",
                        "session_id":"sess_export_1",
                        "label":"metric",
                        "ts_ms":1,
                        "redaction_level":"metadata_only",
                        "target":{"metric_name":"error_rate","value":0.5,"unit":"ratio","inputs":[]}
                    })).expect("metric ref json")
                }),
            ],
            analysis_derived_metrics: vec![json!({
                "evidence_ref_id": evidence_ref_id,
                "metric_name":"error_rate",
                "value":0.5,
                "unit":"ratio",
                "inputs":[]
            })],
            raw_events: vec![json!({
                "event_id":"evt_1",
                "event_seq":1,
                "ts_ms":1,
                "cdp_method":"Network.requestWillBeSent"
            })],
            blobs: Vec::new(),
        }
    }

    #[test]
    fn manifest_has_required_layout_and_indexes() {
        let dir = tempdir().expect("tempdir");
        let dataset = sample_dataset(RedactionLevel::Redacted, ExportProfileV1::ShareSafe);
        let result = export_session(
            dataset,
            ExportWriteRequestV1 {
                export_id: "exp_1".to_string(),
                output_dir: dir.path().to_string_lossy().to_string(),
            },
        )
        .expect("export");
        assert!(PathBuf::from(&result.zip_path).exists());
        assert_eq!(result.manifest.v, 1);
        assert_eq!(result.manifest.evidence_indexes.raw_event, "raw/events.index.ndjson");
        assert!(result
            .manifest
            .files
            .iter()
            .any(|entry| entry.path == "normalized/network.index.ndjson"));
    }

    #[test]
    fn file_order_is_lexicographically_stable() {
        let dir = tempdir().expect("tempdir");
        let dataset = sample_dataset(RedactionLevel::Redacted, ExportProfileV1::ShareSafe);
        let left = export_session(
            dataset.clone(),
            ExportWriteRequestV1 {
                export_id: "exp_left".to_string(),
                output_dir: dir.path().to_string_lossy().to_string(),
            },
        )
        .expect("left export");
        let right = export_session(
            dataset,
            ExportWriteRequestV1 {
                export_id: "exp_right".to_string(),
                output_dir: dir.path().to_string_lossy().to_string(),
            },
        )
        .expect("right export");
        assert_eq!(left.manifest.files, right.manifest.files);
    }

    #[test]
    fn share_safe_excludes_blobs() {
        let dir = tempdir().expect("tempdir");
        let dataset = sample_dataset(RedactionLevel::Redacted, ExportProfileV1::ShareSafe);
        let result = export_session(
            dataset,
            ExportWriteRequestV1 {
                export_id: "exp_share".to_string(),
                output_dir: dir.path().to_string_lossy().to_string(),
            },
        )
        .expect("share export");
        assert!(!result.manifest.files.iter().any(|entry| entry.path.starts_with("blobs/")));
    }

    #[test]
    fn full_export_blocks_for_metadata_only() {
        let dir = tempdir().expect("tempdir");
        let dataset = sample_dataset(RedactionLevel::MetadataOnly, ExportProfileV1::Full);
        let result = export_session(
            dataset,
            ExportWriteRequestV1 {
                export_id: "exp_full_blocked".to_string(),
                output_dir: dir.path().to_string_lossy().to_string(),
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn raw_events_output_is_zstd_ndjson() {
        let dir = tempdir().expect("tempdir");
        let dataset = sample_dataset(RedactionLevel::Redacted, ExportProfileV1::ShareSafe);
        let result = export_session(
            dataset,
            ExportWriteRequestV1 {
                export_id: "exp_raw".to_string(),
                output_dir: dir.path().to_string_lossy().to_string(),
            },
        )
        .expect("export");
        let report = verify_bundle_contents(&result.zip_path).expect("verify");
        assert!(report.valid);
    }

    #[test]
    fn report_files_present_and_parseable() {
        let dir = tempdir().expect("tempdir");
        let dataset = sample_dataset(RedactionLevel::Redacted, ExportProfileV1::ShareSafe);
        let result = export_session(
            dataset,
            ExportWriteRequestV1 {
                export_id: "exp_report".to_string(),
                output_dir: dir.path().to_string_lossy().to_string(),
            },
        )
        .expect("export");
        assert!(result.manifest.files.iter().any(|entry| entry.path == "report/report.json"));
        assert!(result.manifest.files.iter().any(|entry| entry.path == "report/report.html"));
    }

    #[test]
    fn full_export_includes_blobs_when_allowed() {
        let dir = tempdir().expect("tempdir");
        let blob_file_path = dir.path().join("blob.bin");
        fs::write(&blob_file_path, b"blob-bytes").expect("write blob file");

        let mut dataset = sample_dataset(RedactionLevel::Full, ExportProfileV1::Full);
        dataset.blobs.push(dtt_core::ExportBlobDescriptorV1 {
            blob_id: "blob_1".to_string(),
            media_type: Some("application/octet-stream".to_string()),
            len_bytes: 10,
            blake3_hash: "blobhash1".to_string(),
            storage_kind: "file".to_string(),
            storage_ref: blob_file_path.to_string_lossy().to_string(),
        });

        let result = export_session(
            dataset,
            ExportWriteRequestV1 {
                export_id: "exp_full_blob".to_string(),
                output_dir: dir.path().to_string_lossy().to_string(),
            },
        )
        .expect("full export");
        assert!(result.manifest.files.iter().any(|entry| entry.path.starts_with("blobs/")));
    }

    #[test]
    fn deterministic_replay_produces_identical_zip_bytes() {
        let dir = tempdir().expect("tempdir");
        let dataset = sample_dataset(RedactionLevel::Redacted, ExportProfileV1::ShareSafe);
        let left = export_session(
            dataset.clone(),
            ExportWriteRequestV1 {
                export_id: "exp_det_left".to_string(),
                output_dir: dir.path().to_string_lossy().to_string(),
            },
        )
        .expect("left export");
        let right = export_session(
            dataset,
            ExportWriteRequestV1 {
                export_id: "exp_det_right".to_string(),
                output_dir: dir.path().to_string_lossy().to_string(),
            },
        )
        .expect("right export");
        let left_bytes = fs::read(left.zip_path).expect("read left zip");
        let right_bytes = fs::read(right.zip_path).expect("read right zip");
        assert_eq!(left_bytes, right_bytes);
    }

    #[test]
    fn evidence_resolution_works_for_all_kinds() {
        let dir = tempdir().expect("tempdir");
        let dataset = sample_dataset(RedactionLevel::Redacted, ExportProfileV1::ShareSafe);
        let result = export_session(
            dataset,
            ExportWriteRequestV1 {
                export_id: "exp_resolve".to_string(),
                output_dir: dir.path().to_string_lossy().to_string(),
            },
        )
        .expect("export");
        let raw = resolve_evidence_from_bundle(&result.zip_path, "evr_raw")
            .expect("resolve raw")
            .expect("raw present");
        assert_eq!(raw.kind, EvidenceKind::RawEvent);
        let net = resolve_evidence_from_bundle(&result.zip_path, "evr_net")
            .expect("resolve net")
            .expect("net present");
        assert_eq!(net.kind, EvidenceKind::NetRow);
        let console = resolve_evidence_from_bundle(&result.zip_path, "evr_console")
            .expect("resolve console")
            .expect("console present");
        assert_eq!(console.kind, EvidenceKind::Console);
        let metric = resolve_evidence_from_bundle(&result.zip_path, "evr_1")
            .expect("resolve metric")
            .expect("metric present");
        assert_eq!(metric.kind, EvidenceKind::DerivedMetric);
    }
}
