CREATE TABLE IF NOT EXISTS extension_rollouts (
  rollout_id TEXT PRIMARY KEY,
  channel TEXT NOT NULL,
  version TEXT NOT NULL,
  stage TEXT NOT NULL,
  status TEXT NOT NULL,
  cws_item_id TEXT,
  notes_md TEXT,
  started_at_ms INTEGER NOT NULL,
  completed_at_ms INTEGER,
  error_code TEXT,
  error_message TEXT
);

CREATE TABLE IF NOT EXISTS extension_compliance_checks (
  check_id TEXT PRIMARY KEY,
  rollout_id TEXT NOT NULL,
  check_key TEXT NOT NULL,
  status TEXT NOT NULL,
  details_json TEXT NOT NULL,
  checked_at_ms INTEGER NOT NULL,
  FOREIGN KEY(rollout_id) REFERENCES extension_rollouts(rollout_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS update_rollouts (
  update_rollout_id TEXT PRIMARY KEY,
  channel TEXT NOT NULL,
  version TEXT NOT NULL,
  stage TEXT NOT NULL,
  rollout_pct INTEGER NOT NULL,
  status TEXT NOT NULL,
  feed_url TEXT NOT NULL,
  signature_verified INTEGER NOT NULL,
  started_at_ms INTEGER NOT NULL,
  completed_at_ms INTEGER,
  error_code TEXT,
  error_message TEXT
);

CREATE TABLE IF NOT EXISTS telemetry_audits (
  audit_id TEXT PRIMARY KEY,
  export_run_id TEXT,
  status TEXT NOT NULL,
  violations_count INTEGER NOT NULL,
  violations_json TEXT NOT NULL,
  payload_sha256 TEXT,
  created_at_ms INTEGER NOT NULL,
  FOREIGN KEY(export_run_id) REFERENCES telemetry_exports(export_run_id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS perf_anomalies (
  anomaly_id TEXT PRIMARY KEY,
  run_kind TEXT NOT NULL,
  bucket_start_ms INTEGER NOT NULL,
  metric_name TEXT NOT NULL,
  severity TEXT NOT NULL,
  score REAL NOT NULL,
  baseline_value REAL NOT NULL,
  observed_value REAL NOT NULL,
  details_json TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_extension_rollouts_started
  ON extension_rollouts(started_at_ms DESC, rollout_id ASC);

CREATE INDEX IF NOT EXISTS idx_extension_compliance_rollout
  ON extension_compliance_checks(rollout_id, checked_at_ms DESC, check_id ASC);

CREATE INDEX IF NOT EXISTS idx_update_rollouts_channel_started
  ON update_rollouts(channel, started_at_ms DESC, update_rollout_id ASC);

CREATE INDEX IF NOT EXISTS idx_telemetry_audits_created
  ON telemetry_audits(created_at_ms DESC, audit_id ASC);

CREATE INDEX IF NOT EXISTS idx_perf_anomalies_run_bucket
  ON perf_anomalies(run_kind, bucket_start_ms DESC, anomaly_id ASC);
