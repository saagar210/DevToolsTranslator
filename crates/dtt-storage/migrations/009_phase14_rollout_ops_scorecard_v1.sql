CREATE TABLE IF NOT EXISTS release_health_snapshots (
  snapshot_id TEXT PRIMARY KEY,
  scope TEXT NOT NULL,
  channel TEXT NOT NULL,
  version TEXT NOT NULL,
  stage TEXT,
  health_status TEXT NOT NULL,
  score REAL NOT NULL,
  metrics_json TEXT NOT NULL,
  gate_reasons_json TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS rollout_stage_transitions (
  transition_id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  channel TEXT NOT NULL,
  version TEXT NOT NULL,
  from_stage TEXT,
  to_stage TEXT,
  action TEXT NOT NULL,
  decision_json TEXT NOT NULL,
  decided_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS compliance_evidence_packs (
  pack_id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  channel TEXT NOT NULL,
  version TEXT NOT NULL,
  stage TEXT,
  pack_path TEXT NOT NULL,
  manifest_sha256 TEXT NOT NULL,
  items_json TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  status TEXT NOT NULL,
  error_code TEXT,
  error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_release_health_snapshots_channel_version
  ON release_health_snapshots (channel, version, created_at_ms DESC, snapshot_id ASC);

CREATE INDEX IF NOT EXISTS idx_rollout_stage_transitions_kind_channel_version
  ON rollout_stage_transitions (kind, channel, version, decided_at_ms DESC, transition_id ASC);

CREATE INDEX IF NOT EXISTS idx_compliance_evidence_packs_kind_channel_version
  ON compliance_evidence_packs (kind, channel, version, created_at_ms DESC, pack_id ASC);
