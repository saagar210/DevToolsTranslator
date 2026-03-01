# DevTools Translator Product Specification v1.0

## Product Summary
DevTools Translator is a local-first diagnostic product that captures Chrome DevTools/CDP traffic and translates low-level technical signals into human-readable findings with auditable evidence. The product ships as:
- A Chrome MV3 extension for capture and pairing.
- A Tauri desktop app for ingestion, analysis, explanation, and export.

The core user promise is: every explanation is traceable to concrete evidence, and every export can be independently verified.

## Problem Statement
Frontend and API debugging data is high volume, noisy, and often difficult to share safely. Engineers and product teams need deterministic, privacy-aware analysis that turns DevTools events into understandable claims without leaking secrets.

## Product Scope
### In scope
- Capture CDP/DevTools events into local desktop storage.
- Normalize events into queryable structures.
- Correlate interactions deterministically.
- Run detector packs to produce findings, claims, and EvidenceRefs.
- Export signed bundles (share-safe by default; full gated).

### Out of scope
- Cloud sync.
- Telemetry.
- Remote inference or hosted analysis.
- Any non-local capture transport.

## Detector Pack Model
### General Pack (15 detectors)
1. CORS preflight fail
2. Missing ACAO
3. Credentials + wildcard
4. CSP console
5. 401/403 primary
6. 429
7. 5xx burst
8. blocked_by_client
9. mixed content
10. DNS failure
11. TLS failure
12. stale SW suspected
13. cache-control conflict
14. long request duration
15. large JS response

### LLM Pack (5 detectors)
1. streaming SSE detected
2. model identity verified/inferred/unknown
3. safety block/refusal
4. tool-call schema detected
5. retry/backoff pattern

## Primary User Journeys
### Journey 1: Live capture to findings
1. User opens desktop app and pairs extension.
2. User starts capture from Live Capture.
3. Extension streams events to desktop via localhost WebSocket.
4. Session stops; normalization and analysis run.
5. Findings view shows severity-ranked issues with claim/evidence chains.

### Journey 2: Session inspection and evidence drilldown
1. User opens a saved session.
2. User navigates Timeline, Network, Console, and Findings.
3. User clicks claim evidence.
4. App deep-links to the referenced row/field and highlights the exact pointer.
5. User can inspect claim confidence and remediation steps.

### Journey 3: Share-safe export workflow
1. User opens Export tab.
2. Share-safe export is preselected by default.
3. App generates zip with manifest/index/integrity files.
4. User shares bundle without blobs or secrets.

## Privacy Modes And Defaults
- `metadata_only` (default): no request/response body capture; chunk/blob data represented by metadata only.
- `redacted` (opt-in): body content may be captured but is redacted per contract.
- `full` (opt-in): full body/blob capture allowed; only permitted outside metadata-only sessions.

### Export defaults
- Default export profile: share-safe.
- Full export is gated and blocked when session privacy mode is `metadata_only`.

## Product Constraints
- Local-first operation only.
- No telemetry, analytics beacons, or cloud dependencies.
- Deterministic outputs for identical fixtures.
- Evidence must resolve in-app and in exports.
- Secret-bearing fields must never appear in share-safe exports.

## Success Metrics
### Determinism
- 100% fixture replay parity: identical findings/claims/evidence IDs and ordering across repeated runs.

### Detection quality
- Per-fixture precision/recall threshold documented and gated in regression tests.

### Evidence quality
- >=99.9% evidence resolution success across in-app and export paths.

### Export integrity
- 100% integrity validation pass rate for generated bundles during CI regression suite.

## Risks And Mitigations
- Risk: over-redaction may reduce usefulness.
  - Mitigation: preserve redacted previews and deterministic absence evidence.
- Risk: under-redaction may leak secrets.
  - Mitigation: denylist + allowlist redaction contracts with fixture tests.
- Risk: detector drift over time.
  - Mitigation: locked registry versions and snapshot-based regression gates.
