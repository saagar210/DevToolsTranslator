CREATE TABLE IF NOT EXISTS release_runs (
  run_id TEXT PRIMARY KEY,
  channel TEXT NOT NULL,
  version TEXT NOT NULL,
  commit_sha TEXT NOT NULL,
  status TEXT NOT NULL,
  artifacts_json TEXT NOT NULL,
  notes_md TEXT,
  started_at_ms INTEGER NOT NULL,
  completed_at_ms INTEGER,
  error_code TEXT,
  error_message TEXT
);

CREATE TABLE IF NOT EXISTS bundle_inspections (
  inspect_id TEXT PRIMARY KEY,
  bundle_path TEXT NOT NULL,
  session_id TEXT,
  integrity_valid INTEGER NOT NULL,
  summary_json TEXT NOT NULL,
  opened_at_ms INTEGER NOT NULL,
  closed_at_ms INTEGER,
  error_code TEXT,
  error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_release_runs_started
  ON release_runs(started_at_ms DESC, run_id ASC);

CREATE INDEX IF NOT EXISTS idx_release_runs_status_started
  ON release_runs(status, started_at_ms DESC, run_id ASC);

CREATE INDEX IF NOT EXISTS idx_bundle_inspections_opened
  ON bundle_inspections(opened_at_ms DESC, inspect_id ASC);
