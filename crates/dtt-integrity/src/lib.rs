//! Integrity hashing and verification for export bundles.

#![forbid(unsafe_code)]

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use thiserror::Error;
use zip::ZipArchive;

const BUNDLE_HASH_PATH: &str = "integrity/bundle.blake3.txt";
const FILE_HASHES_PATH: &str = "integrity/files.blake3.json";

const REQUIRED_PATHS: &[&str] = &[
    "manifest.json",
    "session.json",
    "normalized/network_requests.ndjson",
    "normalized/network_responses.ndjson",
    "normalized/network_completion.ndjson",
    "normalized/console_entries.ndjson",
    "normalized/page_lifecycle.ndjson",
    "normalized/interactions.ndjson",
    "normalized/interaction_members.ndjson",
    "normalized/network.index.ndjson",
    "normalized/console.index.ndjson",
    "analysis/findings.ndjson",
    "analysis/claims.ndjson",
    "analysis/evidence_refs.ndjson",
    "analysis/derived_metrics.ndjson",
    "analysis/derived_metrics.index.ndjson",
    "raw/events.ndjson.zst",
    "raw/events.index.ndjson",
    "report/report.json",
    "report/report.html",
    FILE_HASHES_PATH,
    BUNDLE_HASH_PATH,
];

#[derive(Debug, Error)]
pub enum IntegrityError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("missing required path: {0}")]
    MissingRequiredPath(String),
}

pub type Result<T> = std::result::Result<T, IntegrityError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilesHashManifestV1 {
    pub files: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityValidationReportV1 {
    pub valid: bool,
    pub bundle_hash_matches: bool,
    pub mismatched_files: Vec<String>,
    pub missing_paths: Vec<String>,
}

pub fn hash_files(files: BTreeMap<String, Vec<u8>>) -> FilesHashManifestV1 {
    let mut output = BTreeMap::new();
    for (path, bytes) in files {
        if path == FILE_HASHES_PATH || path == BUNDLE_HASH_PATH {
            continue;
        }
        output.insert(path, blake3_hex(&bytes));
    }
    FilesHashManifestV1 { files: output }
}

pub fn bundle_hash(files_hash_manifest: &FilesHashManifestV1) -> String {
    let mut lines = String::new();
    for (path, hash) in &files_hash_manifest.files {
        if path == BUNDLE_HASH_PATH {
            continue;
        }
        lines.push_str(hash);
        lines.push_str("  ");
        lines.push_str(path);
        lines.push('\n');
    }
    blake3_hex(lines.as_bytes())
}

pub fn verify_bundle_contents(
    bundle_path: impl AsRef<Path>,
) -> Result<IntegrityValidationReportV1> {
    let bundle_path = bundle_path.as_ref();
    let archive_file = File::open(bundle_path)?;
    let mut archive = ZipArchive::new(archive_file)?;
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        if entry.is_dir() {
            continue;
        }
        let path = entry.name().replace('\\', "/");
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        files.insert(path, bytes);
    }

    let mut missing_paths: Vec<String> = Vec::new();
    for required in REQUIRED_PATHS {
        if !files.contains_key(*required) {
            missing_paths.push((*required).to_string());
        }
    }

    let Some(files_manifest_bytes) = files.get(FILE_HASHES_PATH) else {
        return Ok(IntegrityValidationReportV1 {
            valid: false,
            bundle_hash_matches: false,
            mismatched_files: Vec::new(),
            missing_paths: vec![FILE_HASHES_PATH.to_string()],
        });
    };
    let files_manifest: FilesHashManifestV1 = serde_json::from_slice(files_manifest_bytes)?;
    let expected_bundle_hash = files
        .get(BUNDLE_HASH_PATH)
        .map(|bytes| String::from_utf8_lossy(bytes).trim().to_string())
        .unwrap_or_default();

    let mut mismatched_files: Vec<String> = Vec::new();
    for (path, expected_hash) in &files_manifest.files {
        match files.get(path) {
            Some(bytes) => {
                let actual_hash = blake3_hex(bytes);
                if actual_hash != *expected_hash {
                    mismatched_files.push(path.clone());
                }
            }
            None => {
                missing_paths.push(path.clone());
            }
        }
    }

    missing_paths.sort();
    missing_paths.dedup();
    mismatched_files.sort();

    let computed_bundle_hash = bundle_hash(&files_manifest);
    let bundle_hash_matches =
        !expected_bundle_hash.is_empty() && computed_bundle_hash == expected_bundle_hash;
    let valid = missing_paths.is_empty() && mismatched_files.is_empty() && bundle_hash_matches;

    Ok(IntegrityValidationReportV1 { valid, bundle_hash_matches, mismatched_files, missing_paths })
}

fn blake3_hex(bytes: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        bundle_hash, hash_files, verify_bundle_contents, FilesHashManifestV1, BUNDLE_HASH_PATH,
        FILE_HASHES_PATH,
    };
    use std::collections::BTreeMap;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    #[test]
    fn files_hash_manifest_is_stable() {
        let mut left = BTreeMap::new();
        left.insert("b.txt".to_string(), b"bbb".to_vec());
        left.insert("a.txt".to_string(), b"aaa".to_vec());
        let mut right = BTreeMap::new();
        right.insert("a.txt".to_string(), b"aaa".to_vec());
        right.insert("b.txt".to_string(), b"bbb".to_vec());

        let left_manifest = hash_files(left);
        let right_manifest = hash_files(right);
        assert_eq!(left_manifest, right_manifest);
    }

    #[test]
    fn bundle_hash_is_stable_and_path_sorted() {
        let mut files = BTreeMap::new();
        files.insert("z.txt".to_string(), "1".to_string());
        files.insert("a.txt".to_string(), "2".to_string());
        let one = bundle_hash(&FilesHashManifestV1 { files: files.clone() });
        let two = bundle_hash(&FilesHashManifestV1 { files });
        assert_eq!(one, two);
    }

    #[test]
    fn verify_bundle_detects_hash_mismatch() {
        let dir = tempdir().expect("tempdir");
        let zip_path = dir.path().join("bad.zip");
        let file = File::create(&zip_path).expect("create zip");
        let mut writer = ZipWriter::new(file);
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        writer.start_file("manifest.json", opts).expect("manifest");
        writer.write_all(b"{}").expect("manifest bytes");
        writer.start_file("session.json", opts).expect("session");
        writer.write_all(b"{}").expect("session bytes");

        for path in [
            "normalized/network_requests.ndjson",
            "normalized/network_responses.ndjson",
            "normalized/network_completion.ndjson",
            "normalized/console_entries.ndjson",
            "normalized/page_lifecycle.ndjson",
            "normalized/interactions.ndjson",
            "normalized/interaction_members.ndjson",
            "normalized/network.index.ndjson",
            "normalized/console.index.ndjson",
            "analysis/findings.ndjson",
            "analysis/claims.ndjson",
            "analysis/evidence_refs.ndjson",
            "analysis/derived_metrics.ndjson",
            "analysis/derived_metrics.index.ndjson",
            "raw/events.ndjson.zst",
            "raw/events.index.ndjson",
            "report/report.json",
            "report/report.html",
        ] {
            writer.start_file(path, opts).expect("start required");
            writer.write_all(b"{}").expect("required bytes");
        }

        let mut hashes = BTreeMap::new();
        hashes.insert("manifest.json".to_string(), "deadbeef".to_string());
        let files_manifest = FilesHashManifestV1 { files: hashes };
        writer.start_file(FILE_HASHES_PATH, opts).expect("files hash");
        writer
            .write_all(serde_json::to_string(&files_manifest).expect("serialize").as_bytes())
            .expect("files hash bytes");
        writer.start_file(BUNDLE_HASH_PATH, opts).expect("bundle hash");
        writer.write_all(b"mismatch").expect("bundle hash bytes");
        writer.finish().expect("finish zip");

        let report = verify_bundle_contents(&zip_path).expect("verify report");
        assert!(!report.valid);
        assert!(!report.bundle_hash_matches);
        assert!(report.mismatched_files.contains(&"manifest.json".to_string()));
    }

    #[test]
    fn verify_bundle_detects_missing_required_index() {
        let dir = tempdir().expect("tempdir");
        let zip_path = dir.path().join("missing.zip");
        let file = File::create(&zip_path).expect("create zip");
        let mut writer = ZipWriter::new(file);
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        writer.start_file("manifest.json", opts).expect("manifest");
        writer.write_all(b"{}").expect("manifest bytes");
        writer.start_file("session.json", opts).expect("session");
        writer.write_all(b"{}").expect("session bytes");
        writer.start_file(FILE_HASHES_PATH, opts).expect("files hash");
        writer.write_all(br#"{"files":{}}"#).expect("files hash bytes");
        writer.start_file(BUNDLE_HASH_PATH, opts).expect("bundle hash");
        writer.write_all(b"").expect("bundle hash bytes");
        writer.finish().expect("finish zip");

        let report = verify_bundle_contents(&zip_path).expect("verify report");
        assert!(!report.valid);
        assert!(report.missing_paths.iter().any(|path| path == "raw/events.index.ndjson"));
    }
}
