CREATE TABLE IF NOT EXISTS reliability_metrics (
  metric_id TEXT PRIMARY KEY,
  session_id TEXT,
  source TEXT NOT NULL,
  metric_key TEXT NOT NULL,
  metric_value REAL NOT NULL,
  labels_json TEXT NOT NULL,
  ts_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS perf_runs (
  perf_run_id TEXT PRIMARY KEY,
  run_kind TEXT NOT NULL,
  status TEXT NOT NULL,
  input_ref TEXT NOT NULL,
  summary_json TEXT NOT NULL,
  started_at_ms INTEGER NOT NULL,
  completed_at_ms INTEGER,
  error_code TEXT,
  error_message TEXT
);

ALTER TABLE release_runs ADD COLUMN platform_matrix_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE release_runs ADD COLUMN artifact_count INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_reliability_metrics_key_ts
  ON reliability_metrics(metric_key, ts_ms DESC, metric_id ASC);

CREATE INDEX IF NOT EXISTS idx_reliability_metrics_session_ts
  ON reliability_metrics(session_id, ts_ms DESC, metric_id ASC);

CREATE INDEX IF NOT EXISTS idx_perf_runs_kind_started
  ON perf_runs(run_kind, started_at_ms DESC, perf_run_id ASC);
