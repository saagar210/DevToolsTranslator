CREATE TABLE IF NOT EXISTS release_promotions (
  promotion_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  channel TEXT NOT NULL,
  visibility TEXT NOT NULL,
  status TEXT NOT NULL,
  provenance_json TEXT NOT NULL,
  started_at_ms INTEGER NOT NULL,
  completed_at_ms INTEGER,
  error_code TEXT,
  error_message TEXT,
  FOREIGN KEY(run_id) REFERENCES release_runs(run_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS telemetry_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  mode TEXT NOT NULL,
  otlp_config_json TEXT NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS telemetry_exports (
  export_run_id TEXT PRIMARY KEY,
  status TEXT NOT NULL,
  from_ms INTEGER NOT NULL,
  to_ms INTEGER NOT NULL,
  sample_count INTEGER NOT NULL,
  redacted_count INTEGER NOT NULL,
  payload_sha256 TEXT,
  created_at_ms INTEGER NOT NULL,
  completed_at_ms INTEGER,
  error_code TEXT,
  error_message TEXT
);

ALTER TABLE perf_runs ADD COLUMN run_duration_target_ms INTEGER NOT NULL DEFAULT 0;
ALTER TABLE perf_runs ADD COLUMN actual_duration_ms INTEGER;
ALTER TABLE perf_runs ADD COLUMN budget_result TEXT;
ALTER TABLE perf_runs ADD COLUMN trend_delta_pct REAL;

CREATE INDEX IF NOT EXISTS idx_release_promotions_started
  ON release_promotions(started_at_ms DESC, promotion_id ASC);

CREATE INDEX IF NOT EXISTS idx_telemetry_exports_created
  ON telemetry_exports(created_at_ms DESC, export_run_id ASC);

CREATE INDEX IF NOT EXISTS idx_perf_runs_started
  ON perf_runs(started_at_ms DESC, perf_run_id ASC);
