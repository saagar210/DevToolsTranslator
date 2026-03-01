CREATE TABLE IF NOT EXISTS trusted_devices (
  device_id TEXT PRIMARY KEY,
  browser_label TEXT NOT NULL,
  first_paired_at_ms INTEGER NOT NULL,
  last_seen_at_ms INTEGER NOT NULL,
  revoked INTEGER NOT NULL DEFAULT 0 CHECK (revoked IN (0, 1))
);

CREATE INDEX IF NOT EXISTS idx_trusted_devices_last_seen
  ON trusted_devices(last_seen_at_ms DESC, device_id ASC);
