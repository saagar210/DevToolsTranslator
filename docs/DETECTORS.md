# Detector System Specification v1.0

## Detector Interface Contract v1.0
- Engine mode: batch-only for v1.0 (`analyze_session`).
- Detector identity format:
  - `detector_id`: `<pack>.<category>.<name>.v<major>`
  - `detector_version`: semver string
- Output shape:
  - `Finding[]`, each with ordered `Claim[]`, each with ordered `EvidenceRef[]`.
- Determinism:
  - stable ordering required for findings, claims, and evidence refs.
  - identical fixture input must produce byte-identical outputs.
- Severity rubric:
  - `severity_score = min(100, Impact(0-5) * Scope(0-5) * 4)`.
- Confidence rubric:
  - verified = `1.0`
  - strong pattern = `0.7-0.9`
  - weak pattern = `0.4-0.6`
- Evidence validation:
  - engine validates evidence refs.
  - invalid evidence rejects detector output for the affected finding.
  - detector crash behavior: best-effort skip detector (default).

## Registry Schema (registry.v1.json)
Each detector entry contains:
- `detector_id`
- `detector_version`
- `pack`
- `category`
- `run_scope`
- `inputs`
- `privacy_min`
- `default_enabled`
- `threshold_keys`
- `tags`
- `description`

## Detector Config Schema (config/detectors.v1.json)
Supports:
- pack-level enable/disable
- per-detector enable/disable
- parameter overrides via threshold keys
- privacy-aware minimum mode enforcement

## Locked Top-20 Detector Catalog

| Detector ID | run_scope | Detection Logic | Claim Templates (verified/inferred/unknown) | Evidence Pattern | Default Severity/Confidence | Fix Steps Template | Fixture Requirement |
|---|---|---|---|---|---|---|---|
| `general.security.cors_preflight_fail.v1` | interaction | OPTIONS preflight non-2xx or blocked before follow-up within 2000ms | verified: preflight failed; inferred: CORS negotiation issue; unknown: policy cause unresolved | raw_event + net_row(status/method) | 68 / 1.0,0.8,0.5 | `fix.cors.preflight` | `fx_cors_preflight_fail` |
| `general.security.missing_acao.v1` | interaction | cross-origin response lacks ACAO header | verified missing ACAO; inferred blocked CORS; unknown header unavailable | net_row response headers + absence evidence hash | 64 / 1.0,0.8,0.5 | `fix.cors.add_acao` | `fx_cors_missing_acao` |
| `general.security.credentials_widlcard.v1` | interaction | ACAO=`*` with credentials enabled | verified invalid wildcard+credentials; inferred browser reject risk; unknown credential signal incomplete | net_row headers pointers | 78 / 1.0,0.85,0.5 | `fix.cors.credentials_origin` | `fx_cors_credentials_wildcard` |
| `general.security.csp_console.v1` | session | console regex matches CSP violation pattern | verified CSP violation in console; inferred script blocked by policy; unknown directive unresolved | console entry + optional raw_event | 55 / 1.0,0.75,0.4 | `fix.csp.adjust_policy` | `fx_csp_console_violation` |
| `general.auth.primary_401_403.v1` | interaction | primary request returns 401/403 | verified unauthorized/forbidden primary call; inferred auth/session issue; unknown policy intent | net_row primary request + interaction link | 72 / 1.0,0.8,0.5 | `fix.auth.check_tokens` | `fx_auth_401_primary` |
| `general.resilience.http_429.v1` | interaction | one or more 429 in burst with retry hints | verified rate limited; inferred quota/concurrency exceed; unknown server policy | net_row status + headers(`retry-after`) | 60 / 1.0,0.8,0.5 | `fix.rate_limit.backoff` | `fx_429_with_retry_after` |
| `general.resilience.http_5xx_burst.v1` | interaction | >=N 5xx within burst window | verified server error burst; inferred backend instability; unknown upstream root cause | net_row statuses + timeline | 80 / 1.0,0.8,0.45 | `fix.server.error_burst` | `fx_5xx_burst` |
| `general.client.blocked_by_client.v1` | session | net errors or console pattern indicates blocked_by_client | verified blocked by client extension/filter; inferred adblock/privacy tool interference; unknown blocker origin | console + net error text | 40 / 1.0,0.7,0.45 | `fix.client.disable_blocker` | `fx_blocked_by_client` |
| `general.security.mixed_content.v1` | interaction | https page loads insecure http resource and blocked/warned | verified mixed content; inferred degraded security; unknown enforcement mode | console pattern + request URL scheme | 66 / 1.0,0.8,0.5 | `fix.mixed_content.upgrade` | `fx_mixed_content_block` |
| `general.network.dns_failure.v1` | interaction | DNS resolution errors in network/console | verified DNS failure; inferred host resolution issue; unknown transient network | net error + console pattern | 62 / 1.0,0.75,0.45 | `fix.dns.resolve_host` | `fx_dns_failure` |
| `general.network.tls_failure.v1` | interaction | TLS handshake/cert errors | verified TLS failure; inferred certificate/chain issue; unknown local trust store | net error + optional Security domain event | 74 / 1.0,0.8,0.5 | `fix.tls.certificate_chain` | `fx_tls_failure` |
| `general.pwa.stale_sw_suspected.v1` | session | SW/client version mismatch patterns and cache behavior | verified stale SW indicators; inferred outdated cached assets; unknown activation state | page_lifecycle + headers/cache events | 58 / 0.85,0.75,0.5 | `fix.sw.force_update` | `fx_stale_sw_suspected` |
| `general.cache.cache_control_conflict.v1` | interaction | contradictory cache headers observed | verified cache-control conflict; inferred caching ambiguity; unknown intermediary rewrite | response headers pointers | 50 / 1.0,0.75,0.5 | `fix.cache.align_headers` | `fx_cache_control_conflict` |
| `general.performance.long_request_duration.v1` | interaction | request duration > threshold | verified long duration; inferred backend/network slowness; unknown contention source | timing_json pointers + request row | 48 / 1.0,0.75,0.5 | `fix.perf.reduce_latency` | `fx_long_request_duration` |
| `general.performance.large_js_response.v1` | interaction | JS response size > threshold | verified oversized JS payload; inferred bundle bloat; unknown compression/multiplexing impact | response metadata + content-type + length | 44 / 1.0,0.75,0.5 | `fix.perf.split_bundle` | `fx_large_js_response` |
| `llm.streaming.sse_detected.v1` | interaction | stream summary indicates SSE or event-stream | verified SSE streaming; inferred incremental generation; unknown provider implementation | stream_summary_json + headers/content-type | 30 / 1.0,0.8,0.5 | `fix.llm.streaming_controls` | `fx_llm_sse_stream` |
| `llm.identity.model_identity.v1` | interaction | model id from payload/headers/fingerprint scoring | verified explicit model name; inferred provider/model family; unknown identity unresolved | raw_event pointers + host fingerprint score | 52 / 1.0,0.8,0.4 | `fix.llm.pin_model` | `fx_llm_model_identity_mix` |
| `llm.safety.safety_block_refusal.v1` | interaction | refusal/safety block patterns in response | verified refusal content; inferred policy block; unknown partial refusal ambiguity | response preview/text pattern + stream chunks summary | 46 / 0.9,0.75,0.45 | `fix.llm.prompt_safety_adjust` | `fx_llm_refusal` |
| `llm.tooling.tool_call_schema_detected.v1` | interaction | tool/function-call schema detected in payload | verified tool call schema present; inferred tool orchestration; unknown malformed schema | raw_event json pointers | 36 / 1.0,0.8,0.5 | `fix.llm.tool_schema_validate` | `fx_llm_tool_call_schema` |
| `llm.resilience.retry_backoff_pattern.v1` | interaction | repeated retries with increasing delays | verified retry/backoff pattern; inferred transient upstream failures; unknown retry origin | request sequence timing + status | 54 / 1.0,0.8,0.5 | `fix.llm.retry_tuning` | `fx_llm_retry_backoff` |

## Claim Template Contract
Each detector should emit up to three claim classes where applicable:
- `verified`: directly supported by strong evidence signals.
- `inferred`: likely explanation using pattern/rule confidence.
- `unknown`: unresolved ambiguity with explicit uncertainty.

## Fix Step Object Contract Example (`fix_steps_json`)
```json
[
  {
    "step_id": "fix.cors.preflight.1",
    "title": "Allow OPTIONS preflight",
    "body_md": "Ensure server responds 2xx to OPTIONS and includes matching CORS headers.",
    "risk": "medium",
    "applies_when": ["cors_preflight_failed"],
    "actions": ["server.cors.enable_options", "server.cors.align_allowed_methods"],
    "evidence_ids": ["evr_123"]
  }
]
```

## Fixture Requirements Matrix
Each detector must be covered by at least one deterministic golden fixture:
- `fx_cors_preflight_fail`
- `fx_cors_missing_acao`
- `fx_cors_credentials_wildcard`
- `fx_csp_console_violation`
- `fx_auth_401_primary`
- `fx_429_with_retry_after`
- `fx_5xx_burst`
- `fx_blocked_by_client`
- `fx_mixed_content_block`
- `fx_dns_failure`
- `fx_tls_failure`
- `fx_stale_sw_suspected`
- `fx_cache_control_conflict`
- `fx_long_request_duration`
- `fx_large_js_response`
- `fx_llm_sse_stream`
- `fx_llm_model_identity_mix`
- `fx_llm_refusal`
- `fx_llm_tool_call_schema`
- `fx_llm_retry_backoff`
