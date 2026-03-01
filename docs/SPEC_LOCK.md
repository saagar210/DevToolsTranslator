========================
SPEC LOCK APPENDIX (AUTHORITATIVE)
========================

A) EvidenceRef Spec v1.0 (canonical)
- Stored in evidence_refs.ref_json (TEXT JSON).
- Shape:
{
  "v":1,
  "kind":"raw_event|net_row|console|derived_metric",
  "session_id":"…",
  "label":"Short UI label",
  "ts_ms":<epoch_ms>,
  "redaction_level":"metadata_only|redacted|full",
  "target":{...},
  "preview":{... optional, redacted ...},
  "integrity":{... optional hashes ...}
}
- Uses RFC6901 json_pointer.
- Must support “absence evidence” with container_hash (BLAKE3 over canonical container JSON).
- Kinds:
  1) raw_event target: {event_id, cdp_method, json_pointer?, selection?, absence?}
  2) net_row target: {net_request_id, table, column?, json_pointer?, absence?}
  3) console target: {console_id, column?, json_pointer?}
  4) derived_metric target: {metric_name, value, unit, inputs:[EvidenceRef-lite]}
- Evidence resolution must work in-app (DB) and in exports (manifest + index).
- Share-safe: previews must be redacted or omitted; never include auth/cookies/tokens.

B) SQLite Schema v1.0 (tables must exist with these semantics)
Core tables (required):
- schema_meta, schema_migrations
- sessions
- events_raw (append-only; stores payload_bytes possibly zstd; stores payload_hash = BLAKE3 of canonical JSON)
- network_requests, network_responses, network_completion
- network_stream_chunks (optional; allow hash/len only in metadata_only)
- console_entries (stores message_redacted + message_hash + message_len)
- page_lifecycle
- interactions, interaction_members
- findings, claims, evidence_refs
- blobs (gated content; may store in DB or on disk)
Indexes (required intent):
- fast timeline: session_id+event_seq, session_id+ts_ms
- fast session network: session_id+started_at_ms, status, host/path filters
- fast findings: session_id severity desc

C) Normalized JSON shapes v1.0
1) request_headers_json/response_headers_json:
- JSON object, lowercase keys, values string or array of strings; sanitize auth/cookie headers.
- Evidence pointers reference headers via /header-name or /header-name/0.
- network_responses.headers_hash = BLAKE3 over canonical headers JSON for absence evidence.

2) timing_json:
- Canonical subset:
{
  "request_time_s": <number>,
  "*_start_ms": <number|null>,
  "*_end_ms": <number|null>,
  "receive_headers_end_ms": <number|null>,
  "worker_*": <number|null>
}
- request_time_s is baseline seconds; *_ms are relative ms.

3) stream_summary_json:
{
  "is_streaming": <bool>,
  "transport": "sse|websocket|chunked_fetch|unknown",
  "content_type": <string|null>,
  "chunk_count": <int>,
  "bytes_total": <int>,
  "first_byte_ms": <int|null>,
  "last_byte_ms": <int|null>,
  "stream_duration_ms": <int|null>,
  "reconstruction": {"status":"ok|partial|failed","parse_errors":<int>,"dropped_chunks":<int>}
}

4) fix_steps_json:
- Array of step objects with {step_id,title,body_md,risk,applies_when[],actions[],evidence_ids[]}

D) Interaction Correlation Rules v1.0 (deterministic)
- Interaction kinds: page_load, api_burst, llm_message, llm_regen, upload, other
- Priority assignment: llm_message > upload > page_load > api_burst
- Default constants:
  BURST_GAP_MS=900
  BURST_MAX_WINDOW_MS=20000
  PAGELOAD_SOFT_TIMEOUT_MS=25000
  PAGELOAD_HARD_TIMEOUT_MS=60000
  STREAM_END_GRACE_MS=2000
  INTERACTION_CLOSE_IDLE_MS=2500
- Preflight OPTIONS within 2000ms gets grouped with follow-up request.
- Primary selection rules:
  - page_load: Document navigation preferred
  - api_burst: Fetch/XHR weighted; errors and slow requests weighted; telemetry de-prioritized
  - llm_message: highest scoring LLM primary candidate
  - upload: triggering request

E) Detector Interface Spec v1.0
- Batch-only engine for v1.0 (analyze_session)
- Detector identity:
  detector_id: <pack>.<category>.<name>.v<major>
  detector_version: semver string
- Output: Findings -> Claims -> EvidenceRefs
- Determinism rules: stable ordering for findings/claims/evidence
- Severity rubric: Impact(0–5)*Scope(0–5)*4 capped at 100
- Confidence rubric: verified=1.0, strong pattern 0.7–0.9, weak 0.4–0.6
- Engine validates evidence refs; invalid evidence rejects finding/detector result (configurable, default best-effort skip detector on crash).

F) Detector Registry + Config v1.0
- registry.v1.json entries with {detector_id,detector_version,pack,category,run_scope,inputs,privacy_min,default_enabled,threshold_keys,tags,description}
- config/detectors.v1.json supports enabling packs and detectors and params.
- Locked initial Top-20 detectors (15 general, 5 llm):
  General: CORS preflight fail, missing ACAO, credentials+widlcard, CSP console, 401/403 primary, 429, 5xx burst, blocked_by_client, mixed content, DNS failure, TLS failure, stale SW suspected, cache-control conflict, long request duration, large JS response
  LLM: streaming SSE detected, model identity verified/inferred/unknown, safety block/refusal, tool-call schema detected, retry/backoff pattern

G) Console patterns / telemetry filters / LLM fingerprints v1.0
- patterns.console.v1.json: regex list for cors/csp/mixed_content/blocked_by_client/dns/tls/chunk_load_failed/network.failed_fetch
- telemetry filter defaults: host_substrings + path_substrings list
- llm_web fingerprints: provider host list + scoring weights; primary = highest score >= 70 based on streaming signals and host/type.

H) Chrome MV3 Extension Spec v1.0
- permissions: debugger, storage, tabs, scripting
- host_permissions: http://127.0.0.1/* and http://localhost/*
- optional host perms for UI label capture
- CDP domains enabled: Network, Runtime, Log, Page, optional Security
- Privacy modes:
  metadata_only default (no bodies)
  redacted (redacted bodies allowed)
  full (bodies allowed)
- Body cap MAX_BODY_BYTES=2,000,000; if exceeded store len/hash only and emit capture_limit finding
- Attach/detach: explicit user action; handle “already attached” gracefully
- Transport: WebSocket ws://127.0.0.1:<port>/ws with pairing token
- Pairing: desktop chooses port 32123–32133, generates 128-bit token; extension stores in chrome.storage.local
- Command/event types:
  cmd.list_tabs, cmd.start_capture, cmd.stop_capture, cmd.set_ui_capture
  evt.hello, evt.tabs_list, evt.session_started, evt.raw_event, evt.session_ended, evt.error
- Buffering cap: MAX_BUFFER_EVENTS=5000 or MAX_BUFFER_BYTES=10MB; drop oldest and emit capture_drop marker

I) Export Bundle Spec v1.0
- Output: zip
- Root layout:
  manifest.json, session.json,
  normalized/*.ndjson + indexes,
  analysis/*.ndjson,
  raw/events.ndjson.zst + index,
  blobs/ only in full export,
  report/report.html + report.json,
  integrity/files.blake3.json + bundle.blake3.txt
- Share-safe default excludes blobs and secret headers; full export blocked if privacy_mode=metadata_only
- Evidence resolves via manifest + index (line-based)
- Integrity: BLAKE3 hashes over stored file bytes + canonical bundle hash string

J) UX Spec v1.0
- Nav: Sessions, Live Capture, Findings, Exports, Settings, About/Diagnostics
- Session views: Overview, Timeline, Network, Console, Findings, Export
- Evidence clicks deep-link and highlight referenced field (column/json_pointer)
- Export flow includes share-safe default; full export gated by privacy_mode

END SPEC LOCK APPENDIX
