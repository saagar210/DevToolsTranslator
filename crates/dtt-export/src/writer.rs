//! Export bundle writer.

#![forbid(unsafe_code)]

use crate::indexes::{
    build_console_index, build_derived_metrics_index, build_network_index, build_raw_events_index,
};
use crate::model::{ExportWriteRequestV1, ExportWriteResultV1};
use crate::privacy::{validate_profile, PrivacyError};
use crate::report::{build_report_html, build_report_json};
use crate::zip::write_deterministic_zip;
use blake3::Hasher;
use dtt_core::{
    ExportDatasetV1, ExportEvidenceIndexesV1, ExportManifestFileEntryV1,
    ExportManifestIndexEntryV1, ExportManifestV1, ManifestFileKindV1, ManifestIndexModeV1,
};
use dtt_integrity::{bundle_hash, hash_files};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("integrity error: {0}")]
    Integrity(#[from] dtt_integrity::IntegrityError),
    #[error("{0}")]
    Privacy(#[from] PrivacyError),
    #[error("invalid export dataset: {0}")]
    InvalidDataset(String),
}

pub type Result<T> = std::result::Result<T, ExportError>;

pub fn export_session(
    dataset: ExportDatasetV1,
    req: ExportWriteRequestV1,
) -> Result<ExportWriteResultV1> {
    validate_profile(dataset.privacy_mode, dataset.export_profile)?;

    let output_dir = PathBuf::from(&req.output_dir);
    fs::create_dir_all(&output_dir)?;
    let zip_path = output_dir.join(format!("{}.zip", req.export_id));

    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut line_counts: BTreeMap<String, usize> = BTreeMap::new();

    write_json_file(&mut files, "session.json", &dataset.session_json)?;
    line_counts.insert("session.json".to_string(), 1);

    write_ndjson(
        &mut files,
        &mut line_counts,
        "normalized/network_requests.ndjson",
        &dataset.normalized_network_requests,
    )?;
    write_ndjson(
        &mut files,
        &mut line_counts,
        "normalized/network_responses.ndjson",
        &dataset.normalized_network_responses,
    )?;
    write_ndjson(
        &mut files,
        &mut line_counts,
        "normalized/network_completion.ndjson",
        &dataset.normalized_network_completion,
    )?;
    write_ndjson(
        &mut files,
        &mut line_counts,
        "normalized/console_entries.ndjson",
        &dataset.normalized_console_entries,
    )?;
    write_ndjson(
        &mut files,
        &mut line_counts,
        "normalized/page_lifecycle.ndjson",
        &dataset.normalized_page_lifecycle,
    )?;
    write_ndjson(
        &mut files,
        &mut line_counts,
        "normalized/interactions.ndjson",
        &dataset.normalized_interactions,
    )?;
    write_ndjson(
        &mut files,
        &mut line_counts,
        "normalized/interaction_members.ndjson",
        &dataset.normalized_interaction_members,
    )?;

    let network_index = build_network_index(
        &dataset.normalized_network_requests,
        &dataset.normalized_network_responses,
        &dataset.normalized_network_completion,
    );
    let console_index = build_console_index(&dataset.normalized_console_entries);
    write_ndjson(&mut files, &mut line_counts, "normalized/network.index.ndjson", &network_index)?;
    write_ndjson(&mut files, &mut line_counts, "normalized/console.index.ndjson", &console_index)?;

    write_ndjson(
        &mut files,
        &mut line_counts,
        "analysis/findings.ndjson",
        &dataset.analysis_findings,
    )?;
    write_ndjson(&mut files, &mut line_counts, "analysis/claims.ndjson", &dataset.analysis_claims)?;
    write_ndjson(
        &mut files,
        &mut line_counts,
        "analysis/evidence_refs.ndjson",
        &dataset.analysis_evidence_refs,
    )?;
    write_ndjson(
        &mut files,
        &mut line_counts,
        "analysis/derived_metrics.ndjson",
        &dataset.analysis_derived_metrics,
    )?;
    let derived_metrics_index = build_derived_metrics_index(&dataset.analysis_derived_metrics);
    write_ndjson(
        &mut files,
        &mut line_counts,
        "analysis/derived_metrics.index.ndjson",
        &derived_metrics_index,
    )?;

    write_raw_events_file(&mut files, &mut line_counts, &dataset.raw_events)?;
    let raw_events_index = build_raw_events_index(&dataset.raw_events);
    write_ndjson(&mut files, &mut line_counts, "raw/events.index.ndjson", &raw_events_index)?;

    if dataset.export_profile == dtt_core::ExportProfileV1::Full {
        for blob in &dataset.blobs {
            let blob_path = format!("blobs/{}", blob.blake3_hash);
            let blob_bytes = fs::read(&blob.storage_ref).map_err(|error| {
                ExportError::InvalidDataset(format!(
                    "blob {} missing from storage ref {}: {error}",
                    blob.blob_id, blob.storage_ref
                ))
            })?;
            files.insert(blob_path.clone(), blob_bytes);
            line_counts.insert(blob_path, 0);
        }
    }

    let report_json = build_report_json(&dataset, files.len(), true);
    write_json_file(&mut files, "report/report.json", &report_json)?;
    line_counts.insert("report/report.json".to_string(), 1);
    files.insert("report/report.html".to_string(), build_report_html(&report_json).into_bytes());
    line_counts.insert("report/report.html".to_string(), 1);

    let indexes = vec![
        ExportManifestIndexEntryV1 {
            name: "raw/events.index.ndjson".to_string(),
            maps_file: "raw/events.ndjson.zst".to_string(),
            mode: ManifestIndexModeV1::Line,
        },
        ExportManifestIndexEntryV1 {
            name: "normalized/network.index.ndjson".to_string(),
            maps_file: "normalized/network_requests.ndjson".to_string(),
            mode: ManifestIndexModeV1::Line,
        },
        ExportManifestIndexEntryV1 {
            name: "normalized/console.index.ndjson".to_string(),
            maps_file: "normalized/console_entries.ndjson".to_string(),
            mode: ManifestIndexModeV1::Line,
        },
        ExportManifestIndexEntryV1 {
            name: "analysis/derived_metrics.index.ndjson".to_string(),
            maps_file: "analysis/derived_metrics.ndjson".to_string(),
            mode: ManifestIndexModeV1::Line,
        },
    ];
    let evidence_indexes = ExportEvidenceIndexesV1 {
        raw_event: "raw/events.index.ndjson".to_string(),
        net_row: "normalized/network.index.ndjson".to_string(),
        console: "normalized/console.index.ndjson".to_string(),
        derived_metric: "analysis/derived_metrics.index.ndjson".to_string(),
    };

    let mut manifest = ExportManifestV1 {
        v: 1,
        session_id: dataset.session_id.clone(),
        exported_at_ms: dataset.exported_at_ms,
        privacy_mode: dataset.privacy_mode,
        export_profile: dataset.export_profile,
        files: Vec::new(),
        indexes,
        evidence_indexes,
    };
    manifest.files = build_manifest_file_entries(&files, &line_counts);
    write_json_file(&mut files, "manifest.json", &serde_json::to_value(&manifest)?)?;
    line_counts.insert("manifest.json".to_string(), 1);

    let files_manifest = hash_files(files.clone());
    let files_blake3_json = serde_json::to_vec(&files_manifest)?;
    files.insert("integrity/files.blake3.json".to_string(), files_blake3_json);
    line_counts.insert("integrity/files.blake3.json".to_string(), 1);
    let bundle_blake3 = bundle_hash(&files_manifest);
    files.insert(
        "integrity/bundle.blake3.txt".to_string(),
        format!("{bundle_blake3}\n").into_bytes(),
    );
    line_counts.insert("integrity/bundle.blake3.txt".to_string(), 1);

    write_deterministic_zip(&zip_path, &files)?;

    Ok(ExportWriteResultV1 {
        export_id: req.export_id,
        zip_path: zip_path.to_string_lossy().to_string(),
        files_blake3_path: "integrity/files.blake3.json".to_string(),
        bundle_blake3,
        file_count: files.len(),
        manifest,
    })
}

fn write_json_file(files: &mut BTreeMap<String, Vec<u8>>, path: &str, value: &Value) -> Result<()> {
    let bytes = serde_json_canonical_bytes(value)?;
    files.insert(path.to_string(), bytes);
    Ok(())
}

fn write_ndjson(
    files: &mut BTreeMap<String, Vec<u8>>,
    line_counts: &mut BTreeMap<String, usize>,
    path: &str,
    rows: &[Value],
) -> Result<()> {
    let mut bytes: Vec<u8> = Vec::new();
    for row in rows {
        let line = serde_json_canonical_bytes(row)?;
        bytes.extend_from_slice(&line);
        bytes.push(b'\n');
    }
    files.insert(path.to_string(), bytes);
    line_counts.insert(path.to_string(), rows.len());
    Ok(())
}

fn write_raw_events_file(
    files: &mut BTreeMap<String, Vec<u8>>,
    line_counts: &mut BTreeMap<String, usize>,
    rows: &[Value],
) -> Result<()> {
    let mut raw_bytes: Vec<u8> = Vec::new();
    for row in rows {
        let line = serde_json_canonical_bytes(row)?;
        raw_bytes.extend_from_slice(&line);
        raw_bytes.push(b'\n');
    }
    let compressed = zstd::stream::encode_all(std::io::Cursor::new(raw_bytes), 3)?;
    files.insert("raw/events.ndjson.zst".to_string(), compressed);
    line_counts.insert("raw/events.ndjson.zst".to_string(), rows.len());
    Ok(())
}

fn build_manifest_file_entries(
    files: &BTreeMap<String, Vec<u8>>,
    line_counts: &BTreeMap<String, usize>,
) -> Vec<ExportManifestFileEntryV1> {
    let mut output = Vec::new();
    for (path, bytes) in files {
        output.push(ExportManifestFileEntryV1 {
            path: path.clone(),
            kind: classify_file_kind(path),
            line_count: u64::try_from(*line_counts.get(path).unwrap_or(&0)).unwrap_or(u64::MAX),
            sha_blake3: blake3_hex(bytes),
        });
    }
    output
}

fn classify_file_kind(path: &str) -> ManifestFileKindV1 {
    if path.starts_with("normalized/") && path.ends_with(".index.ndjson") {
        return ManifestFileKindV1::Index;
    }
    if path.starts_with("analysis/") && path.ends_with(".index.ndjson") {
        return ManifestFileKindV1::Index;
    }
    if path.starts_with("raw/") && path.ends_with(".index.ndjson") {
        return ManifestFileKindV1::Index;
    }
    if path.starts_with("normalized/") {
        ManifestFileKindV1::Normalized
    } else if path.starts_with("analysis/") {
        ManifestFileKindV1::Analysis
    } else if path.starts_with("raw/") {
        ManifestFileKindV1::Raw
    } else if path.starts_with("blobs/") {
        ManifestFileKindV1::Blob
    } else if path.starts_with("report/") {
        ManifestFileKindV1::Report
    } else if path.starts_with("integrity/") {
        ManifestFileKindV1::Integrity
    } else {
        ManifestFileKindV1::Normalized
    }
}

fn serde_json_canonical_bytes(value: &Value) -> Result<Vec<u8>> {
    Ok(serde_json_canonicalizer::to_vec(value)?)
}

fn blake3_hex(bytes: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

pub(crate) fn read_zip_files(path: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut out = BTreeMap::new();
    for idx in 0..archive.len() {
        let mut entry = archive.by_index(idx)?;
        if entry.is_dir() {
            continue;
        }
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut bytes)?;
        out.insert(entry.name().replace('\\', "/"), bytes);
    }
    Ok(out)
}
