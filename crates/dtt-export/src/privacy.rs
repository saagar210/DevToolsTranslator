//! Privacy checks and guardrails.

#![forbid(unsafe_code)]

use dtt_core::{ExportProfileV1, RedactionLevel};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PrivacyError {
    #[error("full export is blocked when session privacy_mode is metadata_only")]
    FullBlockedForMetadataOnly,
}

pub type Result<T> = std::result::Result<T, PrivacyError>;

pub fn validate_profile(privacy_mode: RedactionLevel, profile: ExportProfileV1) -> Result<()> {
    if profile == ExportProfileV1::Full && privacy_mode == RedactionLevel::MetadataOnly {
        return Err(PrivacyError::FullBlockedForMetadataOnly);
    }
    Ok(())
}
