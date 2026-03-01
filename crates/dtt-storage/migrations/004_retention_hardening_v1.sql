CREATE TABLE IF NOT EXISTS app_settings (
  setting_key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS retention_runs (
  run_id TEXT PRIMARY KEY,
  mode TEXT NOT NULL,
  started_at_ms INTEGER NOT NULL,
  finished_at_ms INTEGER,
  report_json TEXT NOT NULL,
  evaluated_sessions INTEGER NOT NULL,
  deleted_sessions INTEGER NOT NULL,
  failed_sessions INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS bridge_diagnostics (
  diag_id TEXT PRIMARY KEY,
  session_id TEXT,
  ts_ms INTEGER NOT NULL,
  kind TEXT NOT NULL,
  message TEXT NOT NULL,
  source TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_bridge_diagnostics_ts
  ON bridge_diagnostics(ts_ms DESC, diag_id ASC);

CREATE INDEX IF NOT EXISTS idx_bridge_diagnostics_session_ts
  ON bridge_diagnostics(session_id, ts_ms DESC, diag_id ASC);

CREATE INDEX IF NOT EXISTS idx_retention_runs_started
  ON retention_runs(started_at_ms DESC, run_id ASC);
