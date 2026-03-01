use dtt_core::JsonEnvelope;
use dtt_storage::Storage;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: emit_detector_snapshot <fixture_file> <session_id>");
        std::process::exit(2);
    }

    let fixture_name = &args[1];
    let session_id = &args[2];

    let mut storage = Storage::open_in_memory().expect("open db");
    storage.apply_migrations().expect("apply migrations");

    let fixture_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/raw").join(fixture_name);
    let fixture_data = fs::read_to_string(&fixture_path).expect("read fixture");
    for line in fixture_data.lines().filter(|line| !line.trim().is_empty()) {
        let mut envelope: Value = serde_json::from_str(line).expect("parse fixture line");
        envelope["session_id"] = Value::String(session_id.to_string());
        let parsed: JsonEnvelope = serde_json::from_value(envelope).expect("parse envelope type");
        storage.ingest_raw_event_envelope(&parsed).expect("ingest event");
    }

    storage.normalize_session(session_id).expect("normalize");
    storage.correlate_session(session_id).expect("correlate");
    storage.analyze_session(session_id).expect("analyze");
    for row in storage.debug_dump_analysis_rows(session_id).expect("dump analysis rows") {
        println!("{row}");
    }
}
