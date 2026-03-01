# Data Model v1.0 (SQLite + Canonical JSON Contracts)

## Overview
This document defines SQLite schema contracts, canonical JSON shapes, hashing rules, and redaction contracts for DevTools Translator v1.0.

## Schema Versioning
- `schema_meta.current_version` is `1.0`.
- `schema_migrations` stores ordered forward-only migration metadata.
- Migration files are immutable after merge and include checksums.

## Required Tables And Semantics
### Meta and session lifecycle
- `schema_meta`: current version and compatibility metadata.
- `schema_migrations`: applied migration ledger and checksum.
- `sessions`: one row per capture session.

### Raw and normalized capture
- `events_raw`: append-only CDP/DevTools stream; stores payload bytes (plain or zstd), canonical payload hash.
- `network_requests`: normalized request attributes and headers/timing pointers.
- `network_responses`: normalized response attributes and headers hash.
- `network_completion`: completion/error summary and duration fields.
- `network_stream_chunks`: optional chunk-level stream metadata; metadata-only may store len/hash only.
- `console_entries`: normalized console records with redacted message, hash, and message length.
- `page_lifecycle`: navigation/load milestones and lifecycle markers.

### Analysis outputs
- `interactions`: deterministic interaction containers (`page_load`, `api_burst`, `llm_message`, `llm_regen`, `upload`, `other`).
- `interaction_members`: relation table mapping normalized rows/events into interactions.
- `findings`: detector output root entities with severity/confidence.
- `claims`: finding child assertions (verified/inferred/unknown).
- `evidence_refs`: claim evidence references, including RFC6901 pointers and absence evidence.

### Blob storage
- `blobs`: gated binary/object payload metadata and storage pointer.

## Column-Level Intent (Key Fields)
### `events_raw`
- `event_id`: deterministic unique identifier.
- `session_id`: session foreign key.
- `event_seq`: monotonic sequence from extension.
- `ts_ms`: source timestamp.
- `cdp_method`: event method/type.
- `payload_encoding`: `plain|zstd`.
- `payload_bytes`: opaque payload bytes if allowed by privacy mode.
- `payload_hash`: BLAKE3 over canonical JSON payload.
- `payload_len`: payload size.
- `redaction_level`: metadata_only/redacted/full.

### `network_requests` and `network_responses`
- `request_headers_json` / `response_headers_json`: canonical lowercase header map.
- `headers_hash`: BLAKE3 over canonical headers JSON for container/absence evidence.
- `timing_json`: canonical timing subset.
- `stream_summary_json`: canonical stream summary when applicable.

### `console_entries`
- `message_redacted`: sanitized message text for UI and exports.
- `message_hash`: BLAKE3 digest over canonical source message.
- `message_len`: original message length.

### `findings`, `claims`, `evidence_refs`
- `finding_id`, `claim_id`, `evidence_ref_id`: deterministic IDs.
- `severity_score`: 0..100 via locked rubric.
- `confidence_score`: numeric confidence.
- `fix_steps_json`: array of remediation step objects.
- `ref_json`: canonical EvidenceRef JSON payload.

## Required Index Intent
- Timeline performance:
  - `(session_id, event_seq)`
  - `(session_id, ts_ms)`
- Session network performance:
  - `(session_id, started_at_ms)`
  - `(session_id, status_code)`
  - filtered host/path indexes for request lookup
- Findings performance:
  - `(session_id, severity_score DESC, finding_id)`

## Migration Approach
1. Keep SQL migrations under a versioned migrations directory.
2. Apply in strict order and record checksum in `schema_migrations`.
3. Reject startup if applied migration checksum differs from on-disk migration source.
4. Backward compatibility strategy:
- additive changes only in v1.x
- destructive migrations deferred to v2 with explicit conversion plan

## Canonical JSON Shapes (Locked)
### Headers JSON shape
- JSON object.
- Header keys are lowercase.
- Value is string or array of strings.
- Auth/cookie/token-bearing headers sanitized per redaction rules.
- Evidence pointers target `/header-name` or `/header-name/0`.

### Timing JSON shape
```json
{
  "request_time_s": 0.0,
  "dns_start_ms": null,
  "dns_end_ms": null,
  "connect_start_ms": null,
  "connect_end_ms": null,
  "ssl_start_ms": null,
  "ssl_end_ms": null,
  "send_start_ms": null,
  "send_end_ms": null,
  "receive_headers_end_ms": null,
  "worker_start_ms": null,
  "worker_ready_ms": null,
  "worker_fetch_start_ms": null,
  "worker_respond_with_settled_ms": null
}
```

### Stream summary JSON shape
```json
{
  "is_streaming": false,
  "transport": "sse",
  "content_type": null,
  "chunk_count": 0,
  "bytes_total": 0,
  "first_byte_ms": null,
  "last_byte_ms": null,
  "stream_duration_ms": null,
  "reconstruction": {
    "status": "ok",
    "parse_errors": 0,
    "dropped_chunks": 0
  }
}
```

### Fix steps JSON shape
```json
[
  {
    "step_id": "step-1",
    "title": "Short title",
    "body_md": "Actionable markdown instructions",
    "risk": "low|medium|high",
    "applies_when": ["condition-id"],
    "actions": ["action-id"],
    "evidence_ids": ["evidence_ref_id"]
  }
]
```

## Hashing And Canonicalization
- Hash function: BLAKE3.
- Digest encoding: lowercase hex.
- Canonicalization safe default: RFC8785 JSON Canonicalization Scheme (JCS).
- Hash targets:
  - `events_raw.payload_hash`: canonical payload JSON bytes.
  - `network_responses.headers_hash`: canonical response headers JSON.
  - absence evidence `container_hash`: canonical container JSON bytes.

## Redaction Contracts
### Redaction mode matrix
| Field category | metadata_only | redacted | full |
|---|---|---|---|
| request/response body | omitted | redacted excerpts | full content |
| stream chunk payload | len/hash only | redacted chunk preview | full chunk payload |
| auth/cookie/token headers | redacted | redacted | redacted in share-safe exports |
| blobs table payload | forbidden | allowed (redacted previews) | allowed |
| EvidenceRef preview | omitted or redacted | redacted | full in-app only |

### Share-safe export rules
- Never include blobs.
- Never include auth/cookie/token secrets.
- Redacted preview only, or omission if unsafe.

## Deterministic ID Safe Defaults
- `finding_id = blake3(session_id + detector_id + primary_evidence_signature + rank)`
- `claim_id = blake3(finding_id + claim_type + rank)`
- `evidence_ref_id = blake3(claim_id + kind + target_signature + rank)`

## Gap Register
- GAP: Canonical JSON algorithm not explicitly named in lock.
- SAFE DEFAULT: RFC8785 JCS.
- IMPACT: Cross-language hash mismatches if a different canonicalizer is used.
- VERIFICATION: Cross-language fixture hash conformance tests (Rust + TS).

- GAP: Blob placement DB vs disk policy not fully specified.
- SAFE DEFAULT: Store blob bytes on disk by content hash, store metadata and pointer in `blobs`.
- IMPACT: Migration complexity if storage strategy changes later.
- VERIFICATION: Export/import and integrity checks on blob pointer resolution.
