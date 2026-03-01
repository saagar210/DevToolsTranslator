ALTER TABLE network_requests ADD COLUMN event_seq INTEGER;
ALTER TABLE network_requests ADD COLUMN scheme TEXT;
ALTER TABLE network_requests ADD COLUMN port INTEGER;
ALTER TABLE network_requests ADD COLUMN query TEXT;
ALTER TABLE network_requests ADD COLUMN redaction_level TEXT;

ALTER TABLE network_responses ADD COLUMN protocol TEXT;
ALTER TABLE network_responses ADD COLUMN mime_type TEXT;
ALTER TABLE network_responses ADD COLUMN encoded_data_length INTEGER;
ALTER TABLE network_responses ADD COLUMN redaction_level TEXT;

ALTER TABLE network_completion ADD COLUMN finished_at_ms INTEGER;
ALTER TABLE network_completion ADD COLUMN canceled INTEGER DEFAULT 0;
ALTER TABLE network_completion ADD COLUMN blocked_reason TEXT;

CREATE INDEX IF NOT EXISTS idx_network_requests_session_ts ON network_requests(session_id, ts_ms);
CREATE INDEX IF NOT EXISTS idx_network_responses_session_ts ON network_responses(session_id, ts_ms);
CREATE INDEX IF NOT EXISTS idx_network_completion_session_ts ON network_completion(session_id, ts_ms);
