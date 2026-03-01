# DevTools Translator Architecture v1.0

## System Context
DevTools Translator runs entirely on the local machine:
- Chrome MV3 extension captures CDP-domain events.
- Desktop app (Tauri + Rust backend + React/TS UI) receives, stores, analyzes, and presents findings.
- SQLite is the system-of-record for captured and derived data.
- Exports are generated as deterministic zip bundles with integrity files.

No cloud endpoints are required for core functionality.

## Process Boundaries
### Extension process (Chrome)
- Service worker manages debugger attach/detach.
- CDP domain subscriptions: `Network`, `Runtime`, `Log`, `Page`, optional `Security`.
- Applies privacy mode rules before transport.
- Buffers events with hard caps and drop markers.

### Desktop process (Tauri/Rust)
- WebSocket control plane endpoint at `ws://127.0.0.1:<port>/ws`.
- Pairing/token validation.
- Ingestion and append-only raw event persistence.
- Normalization pipeline to network/console/page tables.
- Interaction correlation engine.
- Detector engine (batch session analysis).
- Export pipeline (manifest/index/report/integrity).

### Desktop UI process (React/TypeScript)
- Queries session/findings data.
- Drives evidence deep-link navigation and highlighting.
- Controls export modes and warnings.
- Exposes diagnostics and capture controls.

### Shared package boundary
- JSON schemas and type contracts for envelopes, findings graph, EvidenceRef, and export schemas.

## Module Boundaries
1. `capture-extension`
- debugger lifecycle
- event filtering/redaction
- ws transport + backpressure

2. `ingress-ws`
- message validation
- token auth
- per-session sequencing

3. `normalizer`
- converts raw events into canonical normalized rows
- computes canonical hashes for headers/payload JSON

4. `storage`
- SQLite migration and DAO layer
- append-only and derived table persistence

5. `correlation`
- deterministic interaction assignment
- primary request selection

6. `detector-engine`
- loads registry/config
- runs Top-20 detectors
- validates output evidence refs

7. `exporter`
- materializes bundle layout and indexes
- emits report and integrity chain

8. `ui-query`
- high-level read models for sessions, findings, and evidence targets

## End-to-End Data Flow
1. User starts capture.
2. Extension emits `evt.session_started`, then `evt.raw_event` stream.
3. Desktop persists `events_raw` append-only records.
4. Normalizer derives `network_*`, `console_entries`, and `page_lifecycle` rows.
5. Correlator groups events into deterministic interactions.
6. Detector engine produces findings/claims/evidence refs.
7. UI renders findings and evidence deep-links.
8. Exporter writes zip bundle with manifest/index/report/integrity.

## Interface Contracts
### WebSocket command envelope (desktop -> extension)
```json
{
  "v": 1,
  "type": "cmd.list_tabs|cmd.start_capture|cmd.stop_capture|cmd.set_ui_capture",
  "request_id": "deterministic-string",
  "session_id": "optional",
  "ts_ms": 0,
  "token": "pairing-token",
  "payload": {}
}
```

### WebSocket event envelope (extension -> desktop)
```json
{
  "v": 1,
  "type": "evt.hello|evt.tabs_list|evt.session_started|evt.raw_event|evt.session_ended|evt.error",
  "event_seq": 0,
  "session_id": "required except hello/tabs",
  "ts_ms": 0,
  "privacy_mode": "metadata_only|redacted|full",
  "payload": {}
}
```

### Detector interface
- Engine API: `analyze_session(session_id) -> Finding[]`
- Output graph: `Finding -> Claim[] -> EvidenceRef[]`
- Deterministic ordering: stable by canonical IDs and tie-break fields.

## Storage + Analysis Pipeline
- Ingestion is append-only in `events_raw`.
- Derivations populate normalized network/console/page tables.
- Correlation layer assigns interaction membership and primary request.
- Detector results are persisted in `findings`, `claims`, and `evidence_refs`.
- Blobs are gated by privacy mode and export profile.

## Determinism Controls
- Canonical JSON serialization before hashing (RFC8785 JCS safe default).
- BLAKE3 hashes for payloads, headers containers, blobs, and export files.
- Stable ordering strategy:
  - primary sort by deterministic ID
  - secondary sort by `ts_ms`
  - tertiary sort by natural keys (`event_seq`, row IDs)
- No wall-clock time or random seed in analysis outputs.

## Security And Privacy Boundaries
- Pairing token required for all control-plane commands.
- Share-safe exports omit blobs and redact secret-bearing fields.
- Metadata-only mode never stores body content.
- Evidence previews in share-safe mode must be redacted or absent.
