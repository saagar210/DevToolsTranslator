//! Export model types.

#![forbid(unsafe_code)]

use dtt_core::{EvidenceKind, ExportManifestV1};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportWriteRequestV1 {
    pub export_id: String,
    pub output_dir: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportWriteResultV1 {
    pub export_id: String,
    pub zip_path: String,
    pub files_blake3_path: String,
    pub bundle_blake3: String,
    pub file_count: usize,
    pub manifest: ExportManifestV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportEvidenceResolveResultV1 {
    pub evidence_ref_id: String,
    pub session_id: String,
    pub kind: EvidenceKind,
    pub target_id: String,
    pub exact_pointer_found: bool,
    pub fallback_reason: Option<String>,
    pub container_json: Option<Value>,
    pub highlighted_value: Option<Value>,
}
