//! Deterministic zip writer.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipWriter};

pub fn write_deterministic_zip(
    output_path: &Path,
    files: &BTreeMap<String, Vec<u8>>,
) -> std::result::Result<(), zip::result::ZipError> {
    let zip_file = File::create(output_path)?;
    let mut writer = ZipWriter::new(zip_file);
    let fixed_time = DateTime::default();
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(fixed_time);

    for (path, bytes) in files {
        writer.start_file(path, options)?;
        writer.write_all(bytes)?;
    }
    writer.finish()?;
    Ok(())
}
