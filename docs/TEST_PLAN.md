# Test Plan v1.0

## Objectives
- Guarantee deterministic outputs for identical fixtures.
- Validate evidence resolution in app and exports.
- Prevent detector regressions and schema drift.
- Enforce release gates for Rust and TypeScript workspaces.

## Golden Fixture Catalog (Minimum Locked Set)
1. `fx_cors_preflight_fail`
2. `fx_cors_missing_acao`
3. `fx_cors_credentials_wildcard`
4. `fx_csp_console_violation`
5. `fx_auth_401_primary`
6. `fx_429_with_retry_after`
7. `fx_5xx_burst`
8. `fx_blocked_by_client`
9. `fx_mixed_content_block`
10. `fx_dns_failure`
11. `fx_tls_failure`
12. `fx_stale_sw_suspected`
13. `fx_cache_control_conflict`
14. `fx_long_request_duration`
15. `fx_large_js_response`
16. `fx_llm_sse_stream`
17. `fx_llm_model_identity_mix`
18. `fx_llm_refusal`
19. `fx_llm_tool_call_schema`
20. `fx_llm_retry_backoff`

## Determinism Test Suite
### Test D1: Replay parity
- Run same fixture twice in isolated DBs.
- Assert byte-identical outputs for:
  - findings NDJSON
  - claims NDJSON
  - evidence_refs NDJSON
  - export indexes
  - integrity hash files

### Test D2: Ordering stability
- Assert stable ordering by deterministic IDs and tie-break keys.
- Verify no wall-clock-derived sort differences.

### Test D3: ID reproducibility
- Assert deterministic IDs for finding/claim/evidence ref graph across runs.

## Evidence Resolution Tests
### Test E1: In-app DB evidence resolution
- For every claim evidence reference, resolve target row and pointer in SQLite.
- Validate absence evidence via `container_hash` when `absence` is set.

### Test E2: Export evidence resolution
- For every exported evidence reference, resolve through manifest + index + NDJSON line.
- Assert pointer-level extraction parity with in-app resolution.

### Test E3: Highlight readiness
- Ensure resolved targets include enough metadata for UI deep-link and highlight behavior.

## Detector Regression Suite
### Test R1: Snapshot findings by fixture
- Snapshot findings/claims/evidence for each fixture.
- Block changes unless explicitly approved.

### Test R2: Crash isolation
- Simulate detector crash and assert best-effort skip behavior for crashed detector only.

### Test R3: Invalid evidence rejection
- Inject invalid EvidenceRef and assert finding rejection for affected detector output.

## Schema And Migration Compatibility
### Test S1: Fresh bootstrap
- Apply all migrations to empty DB and verify schema/table/index presence.

### Test S2: Upgrade path
- Apply forward migrations from historical versions and verify checksums.

### Test S3: Canonical JSON/hash parity
- Cross-language conformance for JCS canonicalization and BLAKE3 digest outputs.

## Privacy/Redaction Matrix Tests
- `metadata_only`: no body/chunk/blob payload persistence.
- `redacted`: redacted previews available, secrets removed.
- `full`: full payload permitted in-app, share-safe still redacts.

## Export Integrity Tests
- Verify `integrity/files.blake3.json` for each exported file.
- Verify canonical bundle hash generation in `bundle.blake3.txt`.
- Fail export if required index or integrity file missing.

## Phase 13 Rollout / Update / Audit Tests
### Test P13-1: Extension public rollout stages
- Validate `pct_5`, `pct_25`, `pct_50`, `pct_100` progression and deterministic stage state transitions.
- Validate compliance snapshot counters and blocking reasons.

### Test P13-2: Desktop updater eligibility
- Validate deterministic bucketing (`install_id + channel + version`) and staged eligibility.
- Validate blocked signature path and blocked policy path.

### Test P13-3: Telemetry audit persistence
- Validate audit insertion/list ordering.
- Validate critical violations block telemetry export completion.

### Test P13-4: Multi-day anomaly persistence
- Validate MAD-based severity mapping (`low/medium/high/critical`).
- Validate deterministic listing by bucket time and anomaly id.

## Phase 14 Rollout Controller / Scorecard / Evidence Pack Tests
### Test P14-1: Controller decision determinism
- Validate fixed gate precedence and deterministic `advance|pause|block|noop` output.
- Validate 24h soak enforcement across `pct_5/25/50/100`.

### Test P14-2: Stage transition persistence
- Validate deterministic insert/list order in `rollout_stage_transitions`.
- Validate extension and updater transitions persist decision JSON and action.

### Test P14-3: Health scorecard persistence
- Validate `release_health_snapshots` insert/list ordering and score consistency.
- Validate global scorecard combines extension/updater signals with stable metric ordering.

### Test P14-4: Compliance evidence packs
- Validate generated pack files and manifest hashes are stable for identical inputs.
- Validate `compliance_evidence_packs` generated/failed paths and UI query behavior.

### Test P14-5: Script gate regression
- `evaluate_extension_stage.mjs` dry-run decisions are deterministic for fixed inputs.
- `advance_extension_stage.mjs` blocks non-dry-run when evaluation action is not `advance`.
- `evaluate_updater_rollout.mjs` and `advance_updater_rollout.mjs` enforce signature/soak/manual smoke gates.

## Verification Gates (Blocking)
### TypeScript
- `pnpm -r lint`
- `pnpm -r typecheck`
- `pnpm -r test`
- `pnpm -r build`

### Rust
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo build --workspace`

### Domain-specific
- Fixture parity gate.
- Evidence resolution gate.
- Export integrity gate.
- Extension rollout compliance gate.
- Updater feed/signature gate.
- Telemetry privacy audit gate.
- Rollout controller decision gate.
- Health scorecard persistence gate.
- Compliance evidence pack integrity gate.

`fail` or `not-run` on any required gate blocks completion.
