CREATE TABLE IF NOT EXISTS exports_runs (
  export_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  export_profile TEXT NOT NULL,
  privacy_mode TEXT NOT NULL,
  status TEXT NOT NULL,
  zip_path TEXT,
  output_dir TEXT,
  integrity_ok INTEGER,
  bundle_blake3 TEXT,
  files_blake3_path TEXT,
  manifest_json TEXT,
  file_count INTEGER,
  error_code TEXT,
  error_message TEXT,
  created_at_ms INTEGER NOT NULL,
  completed_at_ms INTEGER,
  FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_exports_runs_session_created
  ON exports_runs(session_id, created_at_ms DESC, export_id ASC);

CREATE INDEX IF NOT EXISTS idx_exports_runs_status_created
  ON exports_runs(status, created_at_ms DESC, export_id ASC);
