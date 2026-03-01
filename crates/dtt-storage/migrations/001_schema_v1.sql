CREATE TABLE IF NOT EXISTS sessions (
  session_id TEXT PRIMARY KEY,
  privacy_mode TEXT NOT NULL,
  capture_source TEXT NOT NULL,
  started_at_ms INTEGER NOT NULL,
  ended_at_ms INTEGER,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS events_raw (
  event_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  event_seq INTEGER NOT NULL,
  ts_ms INTEGER NOT NULL,
  cdp_method TEXT NOT NULL,
  payload_encoding TEXT NOT NULL,
  payload_bytes BLOB NOT NULL,
  payload_hash TEXT NOT NULL,
  payload_len INTEGER NOT NULL,
  redaction_level TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE,
  UNIQUE(session_id, event_seq)
);

CREATE TABLE IF NOT EXISTS network_requests (
  net_request_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  started_at_ms INTEGER NOT NULL,
  ts_ms INTEGER NOT NULL,
  method TEXT,
  host TEXT,
  path TEXT,
  request_headers_json TEXT,
  timing_json TEXT,
  FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS network_responses (
  net_request_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  ts_ms INTEGER NOT NULL,
  status_code INTEGER,
  response_headers_json TEXT,
  headers_hash TEXT,
  stream_summary_json TEXT,
  FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS network_completion (
  net_request_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  ts_ms INTEGER NOT NULL,
  duration_ms INTEGER,
  success INTEGER,
  error_text TEXT,
  FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS network_stream_chunks (
  chunk_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  net_request_id TEXT NOT NULL,
  ts_ms INTEGER NOT NULL,
  chunk_seq INTEGER NOT NULL,
  chunk_len INTEGER,
  chunk_hash TEXT,
  payload_bytes BLOB,
  FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS console_entries (
  console_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  ts_ms INTEGER NOT NULL,
  level TEXT,
  source TEXT,
  message_redacted TEXT,
  message_hash TEXT,
  message_len INTEGER,
  FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS page_lifecycle (
  lifecycle_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  ts_ms INTEGER NOT NULL,
  frame_id TEXT,
  loader_id TEXT,
  name TEXT,
  value_json TEXT,
  FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS interactions (
  interaction_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  interaction_kind TEXT NOT NULL,
  opened_at_ms INTEGER NOT NULL,
  closed_at_ms INTEGER,
  primary_member_id TEXT,
  rank INTEGER,
  FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS interaction_members (
  interaction_id TEXT NOT NULL,
  member_type TEXT NOT NULL,
  member_id TEXT NOT NULL,
  member_rank INTEGER NOT NULL,
  is_primary INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY(interaction_id, member_type, member_id),
  FOREIGN KEY(interaction_id) REFERENCES interactions(interaction_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS findings (
  finding_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  detector_id TEXT NOT NULL,
  detector_version TEXT NOT NULL,
  title TEXT NOT NULL,
  summary TEXT NOT NULL,
  category TEXT NOT NULL,
  severity_score INTEGER NOT NULL,
  confidence_score REAL NOT NULL,
  created_at_ms INTEGER NOT NULL,
  interaction_id TEXT,
  fix_steps_json TEXT,
  FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS claims (
  claim_id TEXT PRIMARY KEY,
  finding_id TEXT NOT NULL,
  claim_rank INTEGER NOT NULL,
  truth TEXT NOT NULL,
  title TEXT NOT NULL,
  summary TEXT NOT NULL,
  confidence_score REAL NOT NULL,
  FOREIGN KEY(finding_id) REFERENCES findings(finding_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS evidence_refs (
  evidence_ref_id TEXT PRIMARY KEY,
  claim_id TEXT NOT NULL,
  evidence_rank INTEGER NOT NULL,
  ref_json TEXT NOT NULL,
  FOREIGN KEY(claim_id) REFERENCES claims(claim_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS blobs (
  blob_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  privacy_mode TEXT NOT NULL,
  media_type TEXT,
  len_bytes INTEGER NOT NULL,
  blake3_hash TEXT NOT NULL,
  storage_kind TEXT NOT NULL,
  storage_ref TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_events_raw_session_event_seq ON events_raw(session_id, event_seq);
CREATE INDEX IF NOT EXISTS idx_events_raw_session_ts ON events_raw(session_id, ts_ms);
CREATE INDEX IF NOT EXISTS idx_network_requests_session_started ON network_requests(session_id, started_at_ms);
CREATE INDEX IF NOT EXISTS idx_network_responses_session_status ON network_responses(session_id, status_code);
CREATE INDEX IF NOT EXISTS idx_network_requests_session_host_path ON network_requests(session_id, host, path);
CREATE INDEX IF NOT EXISTS idx_findings_session_severity ON findings(session_id, severity_score DESC, finding_id);
