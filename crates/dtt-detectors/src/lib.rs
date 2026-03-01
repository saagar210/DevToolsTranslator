//! Batch detector engine v1.0.

#![forbid(unsafe_code)]

use blake3::Hasher;
use dtt_core::{
    AbsenceEvidence, ClaimTruth, ConsoleEvidenceTarget, EvidenceKind, EvidenceRefV1,
    EvidenceTarget, FindingV1, FixStepRisk, FixStepV1, HeaderMap, HeaderValue,
    NetRowEvidenceTarget, NetTable, RawEventEvidenceTarget, RedactionLevel,
};
use regex::Regex;
use rusqlite::{params, Connection};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use thiserror::Error;

const DETECTOR_VERSION: &str = "1.0.0";

#[derive(Debug, Error)]
pub enum DetectorError {
    #[error("db error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("compression error: {0}")]
    Compression(#[from] std::io::Error),
    #[error("invalid detector data: {0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, DetectorError>;

#[derive(Debug, Clone, PartialEq)]
pub struct DetectorRunReport {
    pub session_id: String,
    pub detectors_considered: usize,
    pub detectors_ran: usize,
    pub findings: Vec<FindingV1>,
    pub skipped_detectors: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RegistryFile {
    v: u8,
    detectors: Vec<RegistryEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct RegistryEntry {
    detector_id: String,
    pack: String,
    default_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct DetectorConfigFile {
    v: u8,
    packs: HashMap<String, PackConfig>,
    detectors: HashMap<String, DetectorToggleConfig>,
    params: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct PackConfig {
    enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct DetectorToggleConfig {
    enabled: Option<bool>,
}

#[derive(Debug, Clone)]
struct SessionContext {
    session_id: String,
    redaction_level: RedactionLevel,
    requests: HashMap<String, RequestRow>,
    responses: HashMap<String, ResponseRow>,
    completions: HashMap<String, CompletionRow>,
    console_entries: HashMap<String, ConsoleRow>,
    lifecycle_entries: HashMap<String, LifecycleRow>,
    interactions: Vec<InteractionRow>,
    interaction_members: HashMap<String, Vec<InteractionMemberRow>>,
    raw_events: HashMap<String, RawEventRow>,
    request_to_raw_events: HashMap<String, Vec<String>>,
    console_patterns: Vec<PatternConfig>,
}

#[derive(Debug, Clone)]
struct RequestRow {
    net_request_id: String,
    ts_ms: i64,
    started_at_ms: i64,
    method: Option<String>,
    host: Option<String>,
    path: Option<String>,
    scheme: Option<String>,
    request_headers: HeaderMap,
    timing_json: Value,
}

#[derive(Debug, Clone)]
struct ResponseRow {
    net_request_id: String,
    ts_ms: i64,
    status_code: Option<i64>,
    mime_type: Option<String>,
    encoded_data_length: Option<i64>,
    response_headers: HeaderMap,
    headers_hash: Option<String>,
    stream_summary: Option<Value>,
}

#[derive(Debug, Clone)]
struct CompletionRow {
    net_request_id: String,
    ts_ms: i64,
    duration_ms: Option<i64>,
    success: Option<bool>,
    error_text: Option<String>,
    blocked_reason: Option<String>,
}

#[derive(Debug, Clone)]
struct ConsoleRow {
    console_id: String,
    ts_ms: i64,
    level: Option<String>,
    source: Option<String>,
    message_redacted: String,
}

#[derive(Debug, Clone)]
struct LifecycleRow {
    lifecycle_id: String,
    ts_ms: i64,
    name: String,
    value_json: Value,
}

#[derive(Debug, Clone)]
struct InteractionRow {
    interaction_id: String,
    interaction_kind: String,
    opened_at_ms: i64,
    closed_at_ms: Option<i64>,
    primary_member_id: Option<String>,
}

#[derive(Debug, Clone)]
struct InteractionMemberRow {
    member_type: String,
    member_id: String,
}

#[derive(Debug, Clone)]
struct RawEventRow {
    event_id: String,
    ts_ms: i64,
    cdp_method: String,
    payload: Value,
}

type RawEventMap = HashMap<String, RawEventRow>;
type RequestRawEventMap = HashMap<String, Vec<String>>;
type RawEventLoad = (RawEventMap, RequestRawEventMap);

#[derive(Debug, Clone, Deserialize)]
struct ConsolePatternFile {
    patterns: Vec<PatternConfig>,
}

#[derive(Debug, Clone, Deserialize)]
struct PatternConfig {
    id: String,
    regex: String,
}

#[derive(Debug, Clone)]
struct FindingSeed {
    detector_id: String,
    title: String,
    summary: String,
    category: String,
    severity_score: u8,
    interaction_id: Option<String>,
    created_at_ms: i64,
    claims: Vec<ClaimSeed>,
    fix_step_id: String,
}

#[derive(Debug, Clone)]
struct ClaimSeed {
    truth: ClaimTruth,
    title: String,
    summary: String,
    confidence_score: f64,
    evidence_refs: Vec<EvidenceRefV1>,
}

pub fn analyze_session(conn: &Connection, session_id: &str) -> Result<DetectorRunReport> {
    let ctx = load_session_context(conn, session_id)?;
    let registry = load_registry()?;
    let config = load_detector_config()?;
    let params = config.params.clone();

    let mut considered = 0_usize;
    let mut ran = 0_usize;
    let mut skipped_detectors: Vec<String> = Vec::new();
    let mut findings: Vec<FindingV1> = Vec::new();

    for entry in registry.detectors {
        if !is_detector_enabled(&entry, &config) {
            continue;
        }
        considered += 1;

        let detector_fn = detector_fn_for_id(&entry.detector_id);
        let Some(detector_fn) = detector_fn else {
            skipped_detectors.push(format!("{}:not_implemented", entry.detector_id));
            continue;
        };

        let detector_result =
            std::panic::catch_unwind(AssertUnwindSafe(|| detector_fn(&ctx, &params)));
        let seeds = match detector_result {
            Ok(result) => result,
            Err(_) => {
                skipped_detectors.push(format!("{}:panic", entry.detector_id));
                continue;
            }
        };
        ran += 1;

        let mut entry_rank = 0_u32;
        for seed in seeds {
            if seed.claims.is_empty() {
                skipped_detectors.push(format!("{}:empty_claims", entry.detector_id));
                continue;
            }
            let first_evidence = seed
                .claims
                .first()
                .and_then(|claim| claim.evidence_refs.first())
                .ok_or_else(|| DetectorError::Invalid("missing evidence refs".to_string()))?;
            let signature = canonical_signature(
                &json!({"detector": entry.detector_id, "target": first_evidence.target}),
            )?;
            entry_rank += 1;
            let finding_id = format!(
                "fnd_{}",
                blake3_hex(
                    format!("{session_id}:{}:{signature}:{entry_rank}", entry.detector_id)
                        .as_bytes()
                )
            );

            let mut claims = Vec::new();
            let mut claim_rank = 0_u32;
            for claim_seed in seed.claims {
                if claim_seed.evidence_refs.is_empty() {
                    continue;
                }
                claim_rank += 1;
                let claim_id = format!(
                    "clm_{}",
                    blake3_hex(
                        format!(
                            "{finding_id}:{}:{claim_rank}",
                            claim_truth_as_str(claim_seed.truth)
                        )
                        .as_bytes()
                    )
                );

                let mut valid_refs = Vec::new();
                for evidence in claim_seed.evidence_refs {
                    if validate_evidence_ref(&ctx, &evidence).is_ok() {
                        valid_refs.push(evidence);
                    }
                }
                if valid_refs.is_empty() {
                    continue;
                }
                claims.push(dtt_core::ClaimV1 {
                    claim_id,
                    finding_id: finding_id.clone(),
                    rank: claim_rank,
                    truth: claim_seed.truth,
                    title: claim_seed.title,
                    summary: claim_seed.summary,
                    confidence_score: claim_seed.confidence_score,
                    evidence_refs: valid_refs,
                });
            }
            if claims.is_empty() {
                skipped_detectors.push(format!("{}:invalid_evidence", entry.detector_id));
                continue;
            }

            let fix_steps = vec![FixStepV1 {
                step_id: seed.fix_step_id.clone(),
                title: format!("Apply {}", seed.fix_step_id),
                body_md: format!("Follow remediation guidance for `{}`.", seed.detector_id),
                risk: FixStepRisk::Medium,
                applies_when: vec![seed.detector_id.clone()],
                actions: vec![seed.fix_step_id],
                evidence_ids: Vec::new(),
            }];

            findings.push(FindingV1 {
                finding_id,
                session_id: session_id.to_string(),
                detector_id: seed.detector_id,
                detector_version: DETECTOR_VERSION.to_string(),
                title: seed.title,
                summary: seed.summary,
                category: seed.category,
                severity_score: seed.severity_score,
                confidence_score: claims
                    .iter()
                    .map(|claim| claim.confidence_score)
                    .fold(0.0, f64::max),
                created_at_ms: seed.created_at_ms,
                interaction_id: seed.interaction_id,
                fix_steps_json: fix_steps,
                claims,
            });
        }
    }

    findings.sort_by(|left, right| {
        right
            .severity_score
            .cmp(&left.severity_score)
            .then_with(|| left.detector_id.cmp(&right.detector_id))
            .then_with(|| left.created_at_ms.cmp(&right.created_at_ms))
            .then_with(|| left.finding_id.cmp(&right.finding_id))
    });

    Ok(DetectorRunReport {
        session_id: session_id.to_string(),
        detectors_considered: considered,
        detectors_ran: ran,
        findings,
        skipped_detectors,
    })
}

type DetectorFn = fn(&SessionContext, &HashMap<String, Value>) -> Vec<FindingSeed>;

fn detector_fn_for_id(detector_id: &str) -> Option<DetectorFn> {
    match detector_id {
        "general.security.cors_preflight_fail.v1" => Some(detector_cors_preflight_fail),
        "general.security.missing_acao.v1" => Some(detector_missing_acao),
        "general.security.credentials_widlcard.v1" => Some(detector_credentials_wildcard),
        "general.security.csp_console.v1" => Some(detector_csp_console),
        "general.auth.primary_401_403.v1" => Some(detector_primary_401_403),
        "general.resilience.http_429.v1" => Some(detector_http_429),
        "general.resilience.http_5xx_burst.v1" => Some(detector_http_5xx_burst),
        "general.client.blocked_by_client.v1" => Some(detector_blocked_by_client),
        "general.security.mixed_content.v1" => Some(detector_mixed_content),
        "general.network.dns_failure.v1" => Some(detector_dns_failure),
        "general.network.tls_failure.v1" => Some(detector_tls_failure),
        "general.pwa.stale_sw_suspected.v1" => Some(detector_stale_sw),
        "general.cache.cache_control_conflict.v1" => Some(detector_cache_control_conflict),
        "general.performance.long_request_duration.v1" => Some(detector_long_request_duration),
        "general.performance.large_js_response.v1" => Some(detector_large_js_response),
        "llm.streaming.sse_detected.v1" => Some(detector_llm_sse_detected),
        "llm.identity.model_identity.v1" => Some(detector_model_identity),
        "llm.safety.safety_block_refusal.v1" => Some(detector_refusal),
        "llm.tooling.tool_call_schema_detected.v1" => Some(detector_tool_call_schema),
        "llm.resilience.retry_backoff_pattern.v1" => Some(detector_retry_backoff),
        _ => None,
    }
}

fn detector_cors_preflight_fail(
    ctx: &SessionContext,
    _: &HashMap<String, Value>,
) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        for request_id in interaction_request_ids(ctx, &interaction.interaction_id) {
            let Some(request) = ctx.requests.get(&request_id) else {
                continue;
            };
            if request
                .method
                .as_deref()
                .map(|method| !method.eq_ignore_ascii_case("OPTIONS"))
                .unwrap_or(true)
            {
                continue;
            }
            let response = ctx.responses.get(&request_id);
            let completion = ctx.completions.get(&request_id);
            let response_failed = response
                .and_then(|row| row.status_code)
                .map(|status| !(200..300).contains(&status))
                .unwrap_or(false);
            let completion_failed = completion
                .map(|row| {
                    row.blocked_reason
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains("cors")
                        || row
                            .error_text
                            .as_deref()
                            .unwrap_or_default()
                            .to_ascii_lowercase()
                            .contains("failed to fetch")
                })
                .unwrap_or(false);
            if !(response_failed || completion_failed) {
                continue;
            }
            let has_followup = interaction_request_ids(ctx, &interaction.interaction_id)
                .into_iter()
                .any(|other_request_id| {
                    if other_request_id == request_id {
                        return false;
                    }
                    let Some(other_request) = ctx.requests.get(&other_request_id) else {
                        return false;
                    };
                    let method = other_request.method.as_deref().unwrap_or_default();
                    !method.eq_ignore_ascii_case("OPTIONS")
                        && endpoint_key(request) == endpoint_key(other_request)
                        && (other_request.started_at_ms - request.started_at_ms).abs() <= 2_000
                });
            if !has_followup {
                continue;
            }
            let mut evidences = vec![evidence_net_row(
                ctx,
                &request_id,
                NetTable::NetworkRequests,
                Some("method"),
                None,
                request.started_at_ms,
                "CORS preflight request",
            )];
            if let Some(response_row) = response {
                evidences.push(evidence_net_row(
                    ctx,
                    &request_id,
                    NetTable::NetworkResponses,
                    Some("status_code"),
                    None,
                    response_row.ts_ms,
                    "Preflight response status",
                ));
            }
            out.push(build_default_finding(
                "general.security.cors_preflight_fail.v1",
                "CORS preflight failed",
                "Preflight OPTIONS failed before follow-up request.",
                "security",
                68,
                interaction,
                evidences,
                "fix.cors.preflight",
            ));
        }
    }
    out
}

fn detector_missing_acao(ctx: &SessionContext, _: &HashMap<String, Value>) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        for request_id in interaction_request_ids(ctx, &interaction.interaction_id) {
            let Some(request) = ctx.requests.get(&request_id) else {
                continue;
            };
            let Some(response) = ctx.responses.get(&request_id) else {
                continue;
            };
            let has_origin = request.request_headers.contains_key("origin");
            if !has_origin {
                continue;
            }
            if response.response_headers.contains_key("access-control-allow-origin") {
                continue;
            }
            let absence = response.headers_hash.as_ref().map(|hash| AbsenceEvidence {
                reason: "missing_access_control_allow_origin".to_string(),
                container_hash: hash.clone(),
            });
            let evidence = EvidenceRefV1 {
                v: 1,
                kind: EvidenceKind::NetRow,
                session_id: ctx.session_id.clone(),
                label: "ACAO header missing".to_string(),
                ts_ms: response.ts_ms,
                redaction_level: ctx.redaction_level,
                target: EvidenceTarget::NetRow(NetRowEvidenceTarget {
                    net_request_id: request_id.clone(),
                    table: NetTable::NetworkResponses,
                    column: Some("response_headers_json".to_string()),
                    json_pointer: Some("/access-control-allow-origin".to_string()),
                    absence,
                }),
                preview: None,
                integrity: None,
            };
            out.push(build_default_finding(
                "general.security.missing_acao.v1",
                "Missing ACAO header",
                "Cross-origin response is missing Access-Control-Allow-Origin.",
                "security",
                64,
                interaction,
                vec![evidence],
                "fix.cors.add_acao",
            ));
        }
    }
    out
}

fn detector_credentials_wildcard(
    ctx: &SessionContext,
    _: &HashMap<String, Value>,
) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        for request_id in interaction_request_ids(ctx, &interaction.interaction_id) {
            let Some(response) = ctx.responses.get(&request_id) else {
                continue;
            };
            let acao =
                header_first_value(&response.response_headers, "access-control-allow-origin");
            let acac =
                header_first_value(&response.response_headers, "access-control-allow-credentials");
            if acao.as_deref() != Some("*")
                || acac.as_deref().map(|value| !value.eq_ignore_ascii_case("true")).unwrap_or(true)
            {
                continue;
            }
            out.push(build_default_finding(
                "general.security.credentials_widlcard.v1",
                "Wildcard CORS with credentials",
                "ACAO=* with credentials=true can be rejected by browsers.",
                "security",
                78,
                interaction,
                vec![
                    evidence_net_row(
                        ctx,
                        &request_id,
                        NetTable::NetworkResponses,
                        Some("response_headers_json"),
                        Some("/access-control-allow-origin"),
                        response.ts_ms,
                        "ACAO wildcard header",
                    ),
                    evidence_net_row(
                        ctx,
                        &request_id,
                        NetTable::NetworkResponses,
                        Some("response_headers_json"),
                        Some("/access-control-allow-credentials"),
                        response.ts_ms,
                        "Credentials CORS header",
                    ),
                ],
                "fix.cors.credentials_origin",
            ));
        }
    }
    out
}

fn detector_csp_console(ctx: &SessionContext, _: &HashMap<String, Value>) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    for console in ctx.console_entries.values() {
        if !console_matches(ctx, &console.message_redacted, "csp") {
            continue;
        }
        out.push(build_session_finding(
            "general.security.csp_console.v1",
            "CSP violation detected",
            "Console output indicates a Content-Security-Policy violation.",
            "security",
            55,
            vec![evidence_console(ctx, &console.console_id, console.ts_ms, "CSP console message")],
            "fix.csp.adjust_policy",
        ));
    }
    out
}

fn detector_primary_401_403(ctx: &SessionContext, _: &HashMap<String, Value>) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        let Some(primary_request_id) = interaction_primary_request_id(interaction) else {
            continue;
        };
        let Some(response) = ctx.responses.get(&primary_request_id) else {
            continue;
        };
        let status = response.status_code.unwrap_or(0);
        if status != 401 && status != 403 {
            continue;
        }
        out.push(build_default_finding(
            "general.auth.primary_401_403.v1",
            "Primary request unauthorized",
            "The primary interaction request returned 401/403.",
            "auth",
            72,
            interaction,
            vec![evidence_net_row(
                ctx,
                &primary_request_id,
                NetTable::NetworkResponses,
                Some("status_code"),
                None,
                response.ts_ms,
                "Primary request status",
            )],
            "fix.auth.check_tokens",
        ));
    }
    out
}

fn detector_http_429(ctx: &SessionContext, _: &HashMap<String, Value>) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        for request_id in interaction_request_ids(ctx, &interaction.interaction_id) {
            let Some(response) = ctx.responses.get(&request_id) else {
                continue;
            };
            if response.status_code != Some(429) {
                continue;
            }
            let mut evidence = vec![evidence_net_row(
                ctx,
                &request_id,
                NetTable::NetworkResponses,
                Some("status_code"),
                None,
                response.ts_ms,
                "429 status code",
            )];
            if response.response_headers.contains_key("retry-after") {
                evidence.push(evidence_net_row(
                    ctx,
                    &request_id,
                    NetTable::NetworkResponses,
                    Some("response_headers_json"),
                    Some("/retry-after"),
                    response.ts_ms,
                    "Retry-After header",
                ));
            }
            out.push(build_default_finding(
                "general.resilience.http_429.v1",
                "HTTP 429 rate limit",
                "Interaction contains rate-limited response(s).",
                "resilience",
                60,
                interaction,
                evidence,
                "fix.rate_limit.backoff",
            ));
        }
    }
    out
}

fn detector_http_5xx_burst(
    ctx: &SessionContext,
    params: &HashMap<String, Value>,
) -> Vec<FindingSeed> {
    let threshold = param_i64(params, "threshold.http_5xx_burst.count", 2);
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        let failed_requests: Vec<String> =
            interaction_request_ids(ctx, &interaction.interaction_id)
                .into_iter()
                .filter(|request_id| {
                    ctx.responses
                        .get(request_id)
                        .and_then(|response| response.status_code)
                        .map(|status| status >= 500)
                        .unwrap_or(false)
                })
                .collect();
        if i64::try_from(failed_requests.len()).unwrap_or(0) < threshold {
            continue;
        }
        let evidence = failed_requests
            .iter()
            .filter_map(|request_id| {
                ctx.responses.get(request_id).map(|response| {
                    evidence_net_row(
                        ctx,
                        request_id,
                        NetTable::NetworkResponses,
                        Some("status_code"),
                        None,
                        response.ts_ms,
                        "5xx response in interaction burst",
                    )
                })
            })
            .collect::<Vec<EvidenceRefV1>>();
        out.push(build_default_finding(
            "general.resilience.http_5xx_burst.v1",
            "HTTP 5xx burst detected",
            "Multiple server errors occurred in one interaction.",
            "resilience",
            80,
            interaction,
            evidence,
            "fix.server.error_burst",
        ));
    }
    out
}

fn detector_blocked_by_client(
    ctx: &SessionContext,
    _: &HashMap<String, Value>,
) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    for completion in ctx.completions.values() {
        let error = completion.error_text.as_deref().unwrap_or_default().to_ascii_lowercase();
        if !error.contains("blocked_by_client") {
            continue;
        }
        out.push(build_session_finding(
            "general.client.blocked_by_client.v1",
            "Request blocked by client",
            "Client extension/filter appears to have blocked a request.",
            "client",
            40,
            vec![evidence_net_row(
                ctx,
                &completion.net_request_id,
                NetTable::NetworkCompletion,
                Some("error_text"),
                None,
                completion.ts_ms,
                "Network completion error",
            )],
            "fix.client.disable_blocker",
        ));
    }
    for console in ctx.console_entries.values() {
        if !console_matches(ctx, &console.message_redacted, "blocked_by_client") {
            continue;
        }
        out.push(build_session_finding(
            "general.client.blocked_by_client.v1",
            "Client-side blocking signal",
            "Console indicates blocked_by_client behavior.",
            "client",
            40,
            vec![evidence_console(
                ctx,
                &console.console_id,
                console.ts_ms,
                "blocked_by_client console signal",
            )],
            "fix.client.disable_blocker",
        ));
    }
    out
}

fn detector_mixed_content(ctx: &SessionContext, _: &HashMap<String, Value>) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        let mut has_https = false;
        let mut has_http = false;
        let mut ts = interaction.opened_at_ms;
        for request_id in interaction_request_ids(ctx, &interaction.interaction_id) {
            if let Some(request) = ctx.requests.get(&request_id) {
                ts = ts.min(request.started_at_ms);
                match request.scheme.as_deref() {
                    Some("https") => has_https = true,
                    Some("http") => has_http = true,
                    _ => {}
                }
            }
        }
        let mixed_console = ctx
            .console_entries
            .values()
            .any(|console| console_matches(ctx, &console.message_redacted, "mixed_content"));
        if !(mixed_console || (has_https && has_http)) {
            continue;
        }
        let mut evidence = Vec::new();
        if let Some(console) = ctx
            .console_entries
            .values()
            .find(|console| console_matches(ctx, &console.message_redacted, "mixed_content"))
        {
            evidence.push(evidence_console(
                ctx,
                &console.console_id,
                console.ts_ms,
                "Mixed content console warning",
            ));
        } else if let Some(http_request_id) =
            interaction_request_ids(ctx, &interaction.interaction_id).into_iter().find(
                |request_id| {
                    ctx.requests.get(request_id).and_then(|request| request.scheme.as_deref())
                        == Some("http")
                },
            )
        {
            evidence.push(evidence_net_row(
                ctx,
                &http_request_id,
                NetTable::NetworkRequests,
                Some("scheme"),
                None,
                ts,
                "HTTP resource under HTTPS flow",
            ));
        }
        out.push(build_default_finding(
            "general.security.mixed_content.v1",
            "Mixed content detected",
            "HTTPS flow references insecure HTTP content.",
            "security",
            66,
            interaction,
            evidence,
            "fix.mixed_content.upgrade",
        ));
    }
    out
}

fn detector_dns_failure(ctx: &SessionContext, _: &HashMap<String, Value>) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        let mut evidence = Vec::new();
        for request_id in interaction_request_ids(ctx, &interaction.interaction_id) {
            if let Some(completion) = ctx.completions.get(&request_id) {
                let error =
                    completion.error_text.as_deref().unwrap_or_default().to_ascii_lowercase();
                if error.contains("dns") || error.contains("name_not_resolved") {
                    evidence.push(evidence_net_row(
                        ctx,
                        &request_id,
                        NetTable::NetworkCompletion,
                        Some("error_text"),
                        None,
                        completion.ts_ms,
                        "DNS-related network error",
                    ));
                }
            }
        }
        if evidence.is_empty() {
            for console in ctx.console_entries.values() {
                if console_matches(ctx, &console.message_redacted, "dns") {
                    evidence.push(evidence_console(
                        ctx,
                        &console.console_id,
                        console.ts_ms,
                        "DNS console signal",
                    ));
                    break;
                }
            }
        }
        if evidence.is_empty() {
            continue;
        }
        out.push(build_default_finding(
            "general.network.dns_failure.v1",
            "DNS failure signal",
            "DNS resolution appears to have failed during the interaction.",
            "network",
            62,
            interaction,
            evidence,
            "fix.dns.resolve_host",
        ));
    }
    out
}

fn detector_tls_failure(ctx: &SessionContext, _: &HashMap<String, Value>) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        let mut evidence = Vec::new();
        for request_id in interaction_request_ids(ctx, &interaction.interaction_id) {
            if let Some(completion) = ctx.completions.get(&request_id) {
                let error =
                    completion.error_text.as_deref().unwrap_or_default().to_ascii_lowercase();
                if error.contains("tls") || error.contains("ssl") || error.contains("certificate") {
                    evidence.push(evidence_net_row(
                        ctx,
                        &request_id,
                        NetTable::NetworkCompletion,
                        Some("error_text"),
                        None,
                        completion.ts_ms,
                        "TLS network error",
                    ));
                }
            }
        }
        if evidence.is_empty() {
            for console in ctx.console_entries.values() {
                if console_matches(ctx, &console.message_redacted, "tls") {
                    evidence.push(evidence_console(
                        ctx,
                        &console.console_id,
                        console.ts_ms,
                        "TLS console signal",
                    ));
                    break;
                }
            }
        }
        if evidence.is_empty() {
            continue;
        }
        out.push(build_default_finding(
            "general.network.tls_failure.v1",
            "TLS failure signal",
            "TLS/certificate failure appears in interaction data.",
            "network",
            74,
            interaction,
            evidence,
            "fix.tls.certificate_chain",
        ));
    }
    out
}

fn detector_stale_sw(ctx: &SessionContext, _: &HashMap<String, Value>) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    let service_worker_console = ctx.console_entries.values().find(|console| {
        let message = console.message_redacted.to_ascii_lowercase();
        message.contains("service worker")
            && (message.contains("stale") || message.contains("update") || message.contains("old"))
    });
    if let Some(console) = service_worker_console {
        out.push(build_session_finding(
            "general.pwa.stale_sw_suspected.v1",
            "Stale service worker suspected",
            "Console indicates potential stale service-worker assets.",
            "pwa",
            58,
            vec![evidence_console(
                ctx,
                &console.console_id,
                console.ts_ms,
                "Service-worker stale signal",
            )],
            "fix.sw.force_update",
        ));
        return out;
    }

    let lifecycle_signal = ctx.lifecycle_entries.values().find(|lifecycle| {
        let lifecycle_name = lifecycle.name.to_ascii_lowercase();
        let lifecycle_json =
            serde_json::to_string(&lifecycle.value_json).unwrap_or_default().to_ascii_lowercase();
        lifecycle_name.contains("service") || lifecycle_json.contains("serviceworker")
    });
    if let Some(lifecycle) = lifecycle_signal {
        out.push(build_session_finding(
            "general.pwa.stale_sw_suspected.v1",
            "Stale service worker suspected",
            "Lifecycle data indicates potential stale service-worker behavior.",
            "pwa",
            58,
            vec![EvidenceRefV1 {
                v: 1,
                kind: EvidenceKind::DerivedMetric,
                session_id: ctx.session_id.clone(),
                label: "Lifecycle service-worker signal".to_string(),
                ts_ms: lifecycle.ts_ms,
                redaction_level: ctx.redaction_level,
                target: EvidenceTarget::DerivedMetric(dtt_core::DerivedMetricEvidenceTarget {
                    metric_name: "lifecycle_service_worker_signal".to_string(),
                    value: 1.0,
                    unit: "count".to_string(),
                    inputs: vec![dtt_core::DerivedMetricEvidenceInput {
                        kind: EvidenceKind::Console,
                        label: lifecycle.lifecycle_id.clone(),
                        ts_ms: lifecycle.ts_ms,
                    }],
                }),
                preview: None,
                integrity: None,
            }],
            "fix.sw.force_update",
        ));
    }
    out
}

fn detector_cache_control_conflict(
    ctx: &SessionContext,
    _: &HashMap<String, Value>,
) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        for request_id in interaction_request_ids(ctx, &interaction.interaction_id) {
            let Some(response) = ctx.responses.get(&request_id) else {
                continue;
            };
            let cache_control =
                header_first_value(&response.response_headers, "cache-control").unwrap_or_default();
            let cache_lower = cache_control.to_ascii_lowercase();
            let pragma =
                header_first_value(&response.response_headers, "pragma").unwrap_or_default();
            let conflict = (cache_lower.contains("no-store") && cache_lower.contains("max-age"))
                || (pragma.to_ascii_lowercase().contains("no-cache")
                    && cache_lower.contains("immutable"));
            if !conflict {
                continue;
            }
            out.push(build_default_finding(
                "general.cache.cache_control_conflict.v1",
                "Cache-control conflict",
                "Response headers contain contradictory cache directives.",
                "cache",
                50,
                interaction,
                vec![evidence_net_row(
                    ctx,
                    &request_id,
                    NetTable::NetworkResponses,
                    Some("response_headers_json"),
                    Some("/cache-control"),
                    response.ts_ms,
                    "Conflicting cache-control header",
                )],
                "fix.cache.align_headers",
            ));
        }
    }
    out
}

fn detector_long_request_duration(
    ctx: &SessionContext,
    params: &HashMap<String, Value>,
) -> Vec<FindingSeed> {
    let threshold = param_i64(params, "threshold.long_request_duration_ms", 2_000);
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        for request_id in interaction_request_ids(ctx, &interaction.interaction_id) {
            let Some(completion) = ctx.completions.get(&request_id) else {
                continue;
            };
            let Some(duration_ms) = completion.duration_ms else {
                continue;
            };
            if duration_ms <= threshold {
                continue;
            }
            out.push(build_default_finding(
                "general.performance.long_request_duration.v1",
                "Long request duration",
                "Request duration exceeded configured threshold.",
                "performance",
                48,
                interaction,
                vec![evidence_net_row(
                    ctx,
                    &request_id,
                    NetTable::NetworkCompletion,
                    Some("duration_ms"),
                    None,
                    completion.ts_ms,
                    "Request duration metric",
                )],
                "fix.perf.reduce_latency",
            ));
        }
    }
    out
}

fn detector_large_js_response(
    ctx: &SessionContext,
    params: &HashMap<String, Value>,
) -> Vec<FindingSeed> {
    let threshold = param_i64(params, "threshold.large_js_response_bytes", 500_000);
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        for request_id in interaction_request_ids(ctx, &interaction.interaction_id) {
            let Some(response) = ctx.responses.get(&request_id) else {
                continue;
            };
            let is_js = response
                .mime_type
                .as_deref()
                .map(|mime| mime.to_ascii_lowercase().contains("javascript"))
                .unwrap_or(false);
            if !is_js {
                continue;
            }
            if response.encoded_data_length.unwrap_or(0) <= threshold {
                continue;
            }
            out.push(build_default_finding(
                "general.performance.large_js_response.v1",
                "Large JavaScript response",
                "JavaScript payload size exceeds configured threshold.",
                "performance",
                44,
                interaction,
                vec![evidence_net_row(
                    ctx,
                    &request_id,
                    NetTable::NetworkResponses,
                    Some("encoded_data_length"),
                    None,
                    response.ts_ms,
                    "Encoded JS payload length",
                )],
                "fix.perf.split_bundle",
            ));
        }
    }
    out
}

fn detector_llm_sse_detected(ctx: &SessionContext, _: &HashMap<String, Value>) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        if !is_llm_interaction(interaction) {
            continue;
        }
        for request_id in interaction_request_ids(ctx, &interaction.interaction_id) {
            let Some(response) = ctx.responses.get(&request_id) else {
                continue;
            };
            let stream = response
                .stream_summary
                .as_ref()
                .and_then(|value| value.get("transport"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            let is_event_stream = response
                .mime_type
                .as_deref()
                .unwrap_or_default()
                .to_ascii_lowercase()
                .contains("event-stream");
            if stream != "sse" && !is_event_stream {
                continue;
            }
            out.push(build_default_finding(
                "llm.streaming.sse_detected.v1",
                "LLM SSE streaming detected",
                "Interaction uses server-sent events for incremental tokens.",
                "llm",
                30,
                interaction,
                vec![evidence_net_row(
                    ctx,
                    &request_id,
                    NetTable::NetworkResponses,
                    Some("stream_summary_json"),
                    Some("/transport"),
                    response.ts_ms,
                    "Stream transport marker",
                )],
                "fix.llm.streaming_controls",
            ));
        }
    }
    out
}

fn detector_model_identity(ctx: &SessionContext, _: &HashMap<String, Value>) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    let model_regex = Regex::new("\"model\"\\s*:\\s*\"([^\"]+)\"").expect("valid model regex");
    for interaction in &ctx.interactions {
        if !is_llm_interaction(interaction) {
            continue;
        }
        let mut model_value: Option<String> = None;
        let mut evidence: Vec<EvidenceRefV1> = Vec::new();
        for request_id in interaction_request_ids(ctx, &interaction.interaction_id) {
            if let Some(raw_event_ids) = ctx.request_to_raw_events.get(&request_id) {
                for event_id in raw_event_ids {
                    let Some(event) = ctx.raw_events.get(event_id) else {
                        continue;
                    };
                    let payload_string = serde_json::to_string(&event.payload).unwrap_or_default();
                    if let Some(capture) = model_regex.captures(&payload_string) {
                        model_value = capture.get(1).map(|m| m.as_str().to_string());
                        evidence.push(evidence_raw_event(
                            ctx,
                            event,
                            Some("/params".to_string()),
                            "Raw model marker",
                        ));
                        break;
                    }
                }
            }
            if model_value.is_some() {
                break;
            }
        }

        if model_value.is_none() {
            for request_id in interaction_request_ids(ctx, &interaction.interaction_id) {
                if let Some(request) = ctx.requests.get(&request_id) {
                    let host = request.host.as_deref().unwrap_or_default().to_ascii_lowercase();
                    if host.contains("openai")
                        || host.contains("anthropic")
                        || host.contains("google")
                    {
                        model_value = Some(host);
                        evidence.push(evidence_net_row(
                            ctx,
                            &request_id,
                            NetTable::NetworkRequests,
                            Some("host"),
                            None,
                            request.ts_ms,
                            "Provider host fingerprint",
                        ));
                        break;
                    }
                }
            }
        }

        if let Some(model_hint) = model_value {
            out.push(build_default_finding(
                "llm.identity.model_identity.v1",
                "LLM model identity signal",
                &format!("Model/provider hint detected: {model_hint}"),
                "llm",
                52,
                interaction,
                evidence,
                "fix.llm.pin_model",
            ));
        }
    }
    out
}

fn detector_refusal(ctx: &SessionContext, _: &HashMap<String, Value>) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        let mut evidence = Vec::new();
        for request_id in interaction_request_ids(ctx, &interaction.interaction_id) {
            if let Some(raw_event_ids) = ctx.request_to_raw_events.get(&request_id) {
                for event_id in raw_event_ids {
                    let Some(event) = ctx.raw_events.get(event_id) else {
                        continue;
                    };
                    let payload = serde_json::to_string(&event.payload)
                        .unwrap_or_default()
                        .to_ascii_lowercase();
                    if payload.contains("refusal") || payload.contains("safety") {
                        evidence.push(evidence_raw_event(
                            ctx,
                            event,
                            Some("/params".to_string()),
                            "Refusal/safety marker",
                        ));
                        break;
                    }
                }
            }
        }
        if evidence.is_empty() {
            continue;
        }
        out.push(build_default_finding(
            "llm.safety.safety_block_refusal.v1",
            "LLM refusal/safety block",
            "Response contains refusal or safety-policy indicators.",
            "llm",
            46,
            interaction,
            evidence,
            "fix.llm.prompt_safety_adjust",
        ));
    }
    out
}

fn detector_tool_call_schema(ctx: &SessionContext, _: &HashMap<String, Value>) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        let mut evidence = Vec::new();
        for request_id in interaction_request_ids(ctx, &interaction.interaction_id) {
            if let Some(raw_event_ids) = ctx.request_to_raw_events.get(&request_id) {
                for event_id in raw_event_ids {
                    let Some(event) = ctx.raw_events.get(event_id) else {
                        continue;
                    };
                    let payload = serde_json::to_string(&event.payload)
                        .unwrap_or_default()
                        .to_ascii_lowercase();
                    if payload.contains("tool_calls")
                        || payload.contains("function_call")
                        || payload.contains("\"tools\"")
                    {
                        evidence.push(evidence_raw_event(
                            ctx,
                            event,
                            Some("/params".to_string()),
                            "Tool-call schema marker",
                        ));
                        break;
                    }
                }
            }
        }
        if evidence.is_empty() {
            continue;
        }
        out.push(build_default_finding(
            "llm.tooling.tool_call_schema_detected.v1",
            "LLM tool-call schema detected",
            "Payload includes tool/function call schema.",
            "llm",
            36,
            interaction,
            evidence,
            "fix.llm.tool_schema_validate",
        ));
    }
    out
}

fn detector_retry_backoff(ctx: &SessionContext, _: &HashMap<String, Value>) -> Vec<FindingSeed> {
    let mut out = Vec::new();
    for interaction in &ctx.interactions {
        let mut requests: Vec<&RequestRow> =
            interaction_request_ids(ctx, &interaction.interaction_id)
                .iter()
                .filter_map(|request_id| ctx.requests.get(request_id))
                .collect();
        requests.sort_by(|left, right| {
            left.started_at_ms
                .cmp(&right.started_at_ms)
                .then_with(|| left.net_request_id.cmp(&right.net_request_id))
        });
        if requests.len() < 3 {
            continue;
        }
        let mut gaps = Vec::new();
        for pair in requests.windows(2) {
            let gap = pair[1].started_at_ms - pair[0].started_at_ms;
            if gap > 0 {
                gaps.push(gap);
            }
        }
        if gaps.len() < 2 {
            continue;
        }
        let non_decreasing = gaps.windows(2).all(|pair| pair[1] >= pair[0]);
        if !non_decreasing {
            continue;
        }
        let failure_count = requests
            .iter()
            .filter(|request| {
                ctx.responses
                    .get(&request.net_request_id)
                    .and_then(|response| response.status_code)
                    .map(|status| status == 429 || status >= 500)
                    .unwrap_or(false)
            })
            .count();
        if failure_count == 0 {
            continue;
        }
        let first = requests.first().expect("first request");
        out.push(build_default_finding(
            "llm.resilience.retry_backoff_pattern.v1",
            "Retry backoff pattern detected",
            "Request timings show non-decreasing retry intervals with failure statuses.",
            "llm",
            54,
            interaction,
            vec![evidence_net_row(
                ctx,
                &first.net_request_id,
                NetTable::NetworkRequests,
                Some("started_at_ms"),
                None,
                first.started_at_ms,
                "Retry sequence start",
            )],
            "fix.llm.retry_tuning",
        ));
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn build_default_finding(
    detector_id: &str,
    title: &str,
    summary: &str,
    category: &str,
    severity_score: u8,
    interaction: &InteractionRow,
    evidence_refs: Vec<EvidenceRefV1>,
    fix_step_id: &str,
) -> FindingSeed {
    let created_at_ms = evidence_refs
        .iter()
        .map(|evidence| evidence.ts_ms)
        .min()
        .unwrap_or(interaction.closed_at_ms.unwrap_or(interaction.opened_at_ms));
    FindingSeed {
        detector_id: detector_id.to_string(),
        title: title.to_string(),
        summary: summary.to_string(),
        category: category.to_string(),
        severity_score,
        interaction_id: Some(interaction.interaction_id.clone()),
        created_at_ms,
        claims: default_claims(summary, evidence_refs),
        fix_step_id: fix_step_id.to_string(),
    }
}

fn build_session_finding(
    detector_id: &str,
    title: &str,
    summary: &str,
    category: &str,
    severity_score: u8,
    evidence_refs: Vec<EvidenceRefV1>,
    fix_step_id: &str,
) -> FindingSeed {
    let created_at_ms = evidence_refs.iter().map(|evidence| evidence.ts_ms).min().unwrap_or(0);
    FindingSeed {
        detector_id: detector_id.to_string(),
        title: title.to_string(),
        summary: summary.to_string(),
        category: category.to_string(),
        severity_score,
        interaction_id: None,
        created_at_ms,
        claims: default_claims(summary, evidence_refs),
        fix_step_id: fix_step_id.to_string(),
    }
}

fn default_claims(summary: &str, evidence_refs: Vec<EvidenceRefV1>) -> Vec<ClaimSeed> {
    vec![
        ClaimSeed {
            truth: ClaimTruth::Verified,
            title: "Verified signal".to_string(),
            summary: summary.to_string(),
            confidence_score: 1.0,
            evidence_refs: evidence_refs.clone(),
        },
        ClaimSeed {
            truth: ClaimTruth::Inferred,
            title: "Likely impact".to_string(),
            summary: "Pattern suggests a related root-cause or policy issue.".to_string(),
            confidence_score: 0.8,
            evidence_refs: evidence_refs.clone(),
        },
        ClaimSeed {
            truth: ClaimTruth::Unknown,
            title: "Unknown exact cause".to_string(),
            summary: "Additional context is needed to determine the exact cause.".to_string(),
            confidence_score: 0.5,
            evidence_refs,
        },
    ]
}

fn evidence_net_row(
    ctx: &SessionContext,
    net_request_id: &str,
    table: NetTable,
    column: Option<&str>,
    json_pointer: Option<&str>,
    ts_ms: i64,
    label: &str,
) -> EvidenceRefV1 {
    EvidenceRefV1 {
        v: 1,
        kind: EvidenceKind::NetRow,
        session_id: ctx.session_id.clone(),
        label: label.to_string(),
        ts_ms,
        redaction_level: ctx.redaction_level,
        target: EvidenceTarget::NetRow(NetRowEvidenceTarget {
            net_request_id: net_request_id.to_string(),
            table,
            column: column.map(ToOwned::to_owned),
            json_pointer: json_pointer.map(ToOwned::to_owned),
            absence: None,
        }),
        preview: None,
        integrity: None,
    }
}

fn evidence_console(
    ctx: &SessionContext,
    console_id: &str,
    ts_ms: i64,
    label: &str,
) -> EvidenceRefV1 {
    EvidenceRefV1 {
        v: 1,
        kind: EvidenceKind::Console,
        session_id: ctx.session_id.clone(),
        label: label.to_string(),
        ts_ms,
        redaction_level: ctx.redaction_level,
        target: EvidenceTarget::Console(ConsoleEvidenceTarget {
            console_id: console_id.to_string(),
            column: Some("message_redacted".to_string()),
            json_pointer: None,
        }),
        preview: None,
        integrity: None,
    }
}

fn evidence_raw_event(
    ctx: &SessionContext,
    event: &RawEventRow,
    json_pointer: Option<String>,
    label: &str,
) -> EvidenceRefV1 {
    EvidenceRefV1 {
        v: 1,
        kind: EvidenceKind::RawEvent,
        session_id: ctx.session_id.clone(),
        label: label.to_string(),
        ts_ms: event.ts_ms,
        redaction_level: ctx.redaction_level,
        target: EvidenceTarget::RawEvent(RawEventEvidenceTarget {
            event_id: event.event_id.clone(),
            cdp_method: event.cdp_method.clone(),
            json_pointer,
            selection: None,
            absence: None,
        }),
        preview: None,
        integrity: None,
    }
}

fn validate_evidence_ref(ctx: &SessionContext, evidence: &EvidenceRefV1) -> Result<()> {
    if evidence.session_id != ctx.session_id {
        return Err(DetectorError::Invalid("evidence session mismatch".to_string()));
    }
    match &evidence.target {
        EvidenceTarget::RawEvent(target) => {
            let event = ctx
                .raw_events
                .get(&target.event_id)
                .ok_or_else(|| DetectorError::Invalid("raw event not found".to_string()))?;
            if event.cdp_method != target.cdp_method {
                return Err(DetectorError::Invalid("raw event method mismatch".to_string()));
            }
            if let Some(pointer) = &target.json_pointer {
                if event.payload.pointer(pointer).is_none() {
                    return Err(DetectorError::Invalid(
                        "raw event json pointer unresolved".to_string(),
                    ));
                }
            }
        }
        EvidenceTarget::NetRow(target) => match target.table {
            NetTable::NetworkRequests => {
                let row = ctx.requests.get(&target.net_request_id).ok_or_else(|| {
                    DetectorError::Invalid("network request row missing".to_string())
                })?;
                if let Some(pointer) = &target.json_pointer {
                    let value = json!({
                        "method": row.method,
                        "host": row.host,
                        "path": row.path,
                        "scheme": row.scheme,
                        "request_headers_json": row.request_headers,
                        "timing_json": row.timing_json
                    });
                    if value.pointer(pointer).is_none() {
                        return Err(DetectorError::Invalid(
                            "network request json pointer unresolved".to_string(),
                        ));
                    }
                }
            }
            NetTable::NetworkResponses => {
                let row = ctx.responses.get(&target.net_request_id).ok_or_else(|| {
                    DetectorError::Invalid("network response row missing".to_string())
                })?;
                if let Some(pointer) = &target.json_pointer {
                    let value = json!({
                        "status_code": row.status_code,
                        "mime_type": row.mime_type,
                        "encoded_data_length": row.encoded_data_length,
                        "response_headers_json": row.response_headers,
                        "stream_summary_json": row.stream_summary
                    });
                    if value.pointer(pointer).is_none() && target.absence.is_none() {
                        return Err(DetectorError::Invalid(
                            "network response json pointer unresolved".to_string(),
                        ));
                    }
                }
                if let Some(absence) = &target.absence {
                    let container_hash = row.headers_hash.as_deref().unwrap_or_default();
                    if container_hash != absence.container_hash {
                        return Err(DetectorError::Invalid(
                            "absence evidence container hash mismatch".to_string(),
                        ));
                    }
                }
            }
            NetTable::NetworkCompletion => {
                let row = ctx.completions.get(&target.net_request_id).ok_or_else(|| {
                    DetectorError::Invalid("network completion row missing".to_string())
                })?;
                if let Some(pointer) = &target.json_pointer {
                    let value = json!({
                        "duration_ms": row.duration_ms,
                        "success": row.success,
                        "error_text": row.error_text,
                        "blocked_reason": row.blocked_reason
                    });
                    if value.pointer(pointer).is_none() {
                        return Err(DetectorError::Invalid(
                            "network completion json pointer unresolved".to_string(),
                        ));
                    }
                }
            }
        },
        EvidenceTarget::Console(target) => {
            let row = ctx
                .console_entries
                .get(&target.console_id)
                .ok_or_else(|| DetectorError::Invalid("console row missing".to_string()))?;
            if let Some(pointer) = &target.json_pointer {
                let value = json!({
                    "level": row.level,
                    "source": row.source,
                    "message_redacted": row.message_redacted
                });
                if value.pointer(pointer).is_none() {
                    return Err(DetectorError::Invalid(
                        "console json pointer unresolved".to_string(),
                    ));
                }
            }
        }
        EvidenceTarget::DerivedMetric(target) => {
            if target.inputs.is_empty() {
                return Err(DetectorError::Invalid("derived metric has no inputs".to_string()));
            }
        }
    }
    Ok(())
}

fn interaction_request_ids(ctx: &SessionContext, interaction_id: &str) -> Vec<String> {
    let mut ids = ctx
        .interaction_members
        .get(interaction_id)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|member| member.member_type == "network_request")
        .map(|member| member.member_id)
        .collect::<Vec<String>>();
    ids.sort();
    ids
}

fn interaction_primary_request_id(interaction: &InteractionRow) -> Option<String> {
    interaction.primary_member_id.as_ref().and_then(|member| {
        let mut parts = member.splitn(2, ':');
        let member_type = parts.next().unwrap_or_default();
        let member_id = parts.next().unwrap_or_default();
        if member_type == "network_request" && !member_id.is_empty() {
            Some(member_id.to_string())
        } else {
            None
        }
    })
}

fn is_llm_interaction(interaction: &InteractionRow) -> bool {
    interaction.interaction_kind == "llm_message" || interaction.interaction_kind == "llm_regen"
}

fn endpoint_key(request: &RequestRow) -> String {
    format!(
        "{}|{}|{}",
        request.scheme.as_deref().unwrap_or_default(),
        request.host.as_deref().unwrap_or_default(),
        request.path.as_deref().unwrap_or_default(),
    )
}

fn load_session_context(conn: &Connection, session_id: &str) -> Result<SessionContext> {
    let redaction_raw: String = conn.query_row(
        "SELECT privacy_mode FROM sessions WHERE session_id = ?1",
        params![session_id],
        |row| row.get(0),
    )?;
    let redaction_level = parse_redaction_level(&redaction_raw)?;

    let requests = load_requests(conn, session_id)?;
    let responses = load_responses(conn, session_id)?;
    let completions = load_completions(conn, session_id)?;
    let console_entries = load_console_entries(conn, session_id)?;
    let lifecycle_entries = load_lifecycle_entries(conn, session_id)?;
    let interactions = load_interactions(conn, session_id)?;
    let interaction_members = load_interaction_members(conn, session_id)?;
    let (raw_events, request_to_raw_events) = load_raw_events(conn, session_id)?;
    let patterns = load_console_patterns()?;

    Ok(SessionContext {
        session_id: session_id.to_string(),
        redaction_level,
        requests,
        responses,
        completions,
        console_entries,
        lifecycle_entries,
        interactions,
        interaction_members,
        raw_events,
        request_to_raw_events,
        console_patterns: patterns,
    })
}

fn load_requests(conn: &Connection, session_id: &str) -> Result<HashMap<String, RequestRow>> {
    let mut out = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT net_request_id, ts_ms, started_at_ms, method, host, path, scheme, request_headers_json, timing_json
         FROM network_requests
         WHERE session_id = ?1",
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, Option<String>>(8)?,
        ))
    })?;
    for row in rows {
        let (
            net_request_id,
            ts_ms,
            started_at_ms,
            method,
            host,
            path,
            scheme,
            headers_json,
            timing_json,
        ) = row?;
        let request = RequestRow {
            net_request_id,
            ts_ms,
            started_at_ms,
            method,
            host,
            path,
            scheme,
            request_headers: parse_header_map(headers_json.as_deref())?,
            timing_json: parse_json_or_default(timing_json.as_deref(), json!({})),
        };
        out.insert(request.net_request_id.clone(), request);
    }
    Ok(out)
}

fn load_responses(conn: &Connection, session_id: &str) -> Result<HashMap<String, ResponseRow>> {
    let mut out = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT net_request_id, ts_ms, status_code, mime_type, encoded_data_length, response_headers_json, headers_hash, stream_summary_json
         FROM network_responses
         WHERE session_id = ?1",
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, Option<i64>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<i64>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, Option<String>>(7)?,
        ))
    })?;
    for row in rows {
        let (
            net_request_id,
            ts_ms,
            status_code,
            mime_type,
            encoded_data_length,
            headers_json,
            headers_hash,
            summary_json,
        ) = row?;
        let response = ResponseRow {
            net_request_id,
            ts_ms,
            status_code,
            mime_type,
            encoded_data_length,
            response_headers: parse_header_map(headers_json.as_deref())?,
            headers_hash,
            stream_summary: summary_json.as_deref().map(serde_json::from_str).transpose()?,
        };
        out.insert(response.net_request_id.clone(), response);
    }
    Ok(out)
}

fn load_completions(conn: &Connection, session_id: &str) -> Result<HashMap<String, CompletionRow>> {
    let mut out = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT net_request_id, ts_ms, duration_ms, success, error_text, blocked_reason
         FROM network_completion
         WHERE session_id = ?1",
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        let success_raw: Option<i64> = row.get(3)?;
        Ok(CompletionRow {
            net_request_id: row.get(0)?,
            ts_ms: row.get(1)?,
            duration_ms: row.get(2)?,
            success: success_raw.map(|value| value != 0),
            error_text: row.get(4)?,
            blocked_reason: row.get(5)?,
        })
    })?;
    for row in rows {
        let completion = row?;
        out.insert(completion.net_request_id.clone(), completion);
    }
    Ok(out)
}

fn load_console_entries(
    conn: &Connection,
    session_id: &str,
) -> Result<HashMap<String, ConsoleRow>> {
    let mut out = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT console_id, ts_ms, level, source, message_redacted
         FROM console_entries
         WHERE session_id = ?1",
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        Ok(ConsoleRow {
            console_id: row.get(0)?,
            ts_ms: row.get(1)?,
            level: row.get(2)?,
            source: row.get(3)?,
            message_redacted: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
        })
    })?;
    for row in rows {
        let console = row?;
        out.insert(console.console_id.clone(), console);
    }
    Ok(out)
}

fn load_lifecycle_entries(
    conn: &Connection,
    session_id: &str,
) -> Result<HashMap<String, LifecycleRow>> {
    let mut out = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT lifecycle_id, ts_ms, name, value_json
         FROM page_lifecycle
         WHERE session_id = ?1",
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        let value_json: Option<String> = row.get(3)?;
        Ok(LifecycleRow {
            lifecycle_id: row.get(0)?,
            ts_ms: row.get(1)?,
            name: row.get(2)?,
            value_json: parse_json_or_default(value_json.as_deref(), json!({})),
        })
    })?;
    for row in rows {
        let lifecycle = row?;
        out.insert(lifecycle.lifecycle_id.clone(), lifecycle);
    }
    Ok(out)
}

fn load_interactions(conn: &Connection, session_id: &str) -> Result<Vec<InteractionRow>> {
    let mut out = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT interaction_id, interaction_kind, opened_at_ms, closed_at_ms, primary_member_id
         FROM interactions
         WHERE session_id = ?1
         ORDER BY opened_at_ms, interaction_kind, interaction_id",
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        Ok(InteractionRow {
            interaction_id: row.get(0)?,
            interaction_kind: row.get(1)?,
            opened_at_ms: row.get(2)?,
            closed_at_ms: row.get(3)?,
            primary_member_id: row.get(4)?,
        })
    })?;
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn load_interaction_members(
    conn: &Connection,
    session_id: &str,
) -> Result<HashMap<String, Vec<InteractionMemberRow>>> {
    let mut out: HashMap<String, Vec<InteractionMemberRow>> = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT im.interaction_id, im.member_type, im.member_id
         FROM interaction_members im
         JOIN interactions i ON i.interaction_id = im.interaction_id
         WHERE i.session_id = ?1
         ORDER BY im.interaction_id, im.member_rank, im.member_type, im.member_id",
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            InteractionMemberRow { member_type: row.get(1)?, member_id: row.get(2)? },
        ))
    })?;
    for row in rows {
        let (interaction_id, member) = row?;
        out.entry(interaction_id).or_default().push(member);
    }
    Ok(out)
}

fn load_raw_events(conn: &Connection, session_id: &str) -> Result<RawEventLoad> {
    let mut events = HashMap::new();
    let mut request_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT event_id, ts_ms, cdp_method, payload_encoding, payload_bytes
         FROM events_raw
         WHERE session_id = ?1
         ORDER BY event_seq ASC",
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, Vec<u8>>(4)?,
        ))
    })?;
    for row in rows {
        let (event_id, ts_ms, cdp_method, payload_encoding, payload_bytes) = row?;
        let payload = decode_payload(&payload_encoding, &payload_bytes)?;
        if let Some(request_id) = payload
            .get("params")
            .and_then(|params| params.get("requestId").or_else(|| params.get("identifier")))
            .and_then(Value::as_str)
        {
            request_map.entry(request_id.to_string()).or_default().push(event_id.clone());
        }
        events.insert(event_id.clone(), RawEventRow { event_id, ts_ms, cdp_method, payload });
    }
    Ok((events, request_map))
}

fn load_console_patterns() -> Result<Vec<PatternConfig>> {
    let parsed: ConsolePatternFile =
        serde_json::from_str(include_str!("../../../config/patterns.console.v1.json"))?;
    Ok(parsed.patterns)
}

fn load_registry() -> Result<RegistryFile> {
    let parsed: RegistryFile = serde_json::from_str(include_str!("../../../registry.v1.json"))?;
    if parsed.v != 1 {
        return Err(DetectorError::Invalid("registry version mismatch".to_string()));
    }
    Ok(parsed)
}

fn load_detector_config() -> Result<DetectorConfigFile> {
    let parsed: DetectorConfigFile =
        serde_json::from_str(include_str!("../../../config/detectors.v1.json"))?;
    if parsed.v != 1 {
        return Err(DetectorError::Invalid("detector config version mismatch".to_string()));
    }
    Ok(parsed)
}

fn is_detector_enabled(entry: &RegistryEntry, config: &DetectorConfigFile) -> bool {
    let pack_enabled = config
        .packs
        .get(&entry.pack)
        .map(|pack_cfg| pack_cfg.enabled)
        .unwrap_or(entry.default_enabled);
    if !pack_enabled {
        return false;
    }
    config
        .detectors
        .get(&entry.detector_id)
        .and_then(|toggle| toggle.enabled)
        .unwrap_or(entry.default_enabled)
}

fn parse_redaction_level(value: &str) -> Result<RedactionLevel> {
    match value {
        "metadata_only" => Ok(RedactionLevel::MetadataOnly),
        "redacted" => Ok(RedactionLevel::Redacted),
        "full" => Ok(RedactionLevel::Full),
        other => Err(DetectorError::Invalid(format!("unknown redaction level: {other}"))),
    }
}

fn parse_header_map(value: Option<&str>) -> Result<HeaderMap> {
    let Some(value) = value else {
        return Ok(HeaderMap::new());
    };
    if value.trim().is_empty() {
        return Ok(HeaderMap::new());
    }
    Ok(serde_json::from_str(value)?)
}

fn parse_json_or_default(value: Option<&str>, default: Value) -> Value {
    value
        .filter(|raw| !raw.trim().is_empty())
        .and_then(|raw| serde_json::from_str(raw).ok())
        .unwrap_or(default)
}

fn decode_payload(encoding: &str, bytes: &[u8]) -> Result<Value> {
    let decoded = match encoding {
        "zstd" => zstd::stream::decode_all(bytes)?,
        "plain" => bytes.to_vec(),
        other => {
            return Err(DetectorError::Invalid(format!("unsupported payload encoding: {other}")))
        }
    };
    Ok(serde_json::from_slice(&decoded)?)
}

fn header_first_value(headers: &HeaderMap, key: &str) -> Option<String> {
    let value = headers.get(&key.to_ascii_lowercase())?;
    match value {
        HeaderValue::Single(single) => Some(single.clone()),
        HeaderValue::Multi(values) => values.first().cloned(),
    }
}

fn console_matches(ctx: &SessionContext, message: &str, pattern_id: &str) -> bool {
    let Some(pattern) = ctx.console_patterns.iter().find(|pattern| pattern.id == pattern_id) else {
        return false;
    };
    let regex = Regex::new(&format!("(?i){}", pattern.regex));
    match regex {
        Ok(regex) => regex.is_match(message),
        Err(_) => false,
    }
}

fn param_i64(params: &HashMap<String, Value>, key: &str, default: i64) -> i64 {
    params.get(key).and_then(Value::as_i64).unwrap_or(default)
}

fn canonical_signature(value: &Value) -> Result<String> {
    Ok(serde_json_canonicalizer::to_string(value)?)
}

fn claim_truth_as_str(truth: ClaimTruth) -> &'static str {
    match truth {
        ClaimTruth::Verified => "verified",
        ClaimTruth::Inferred => "inferred",
        ClaimTruth::Unknown => "unknown",
    }
}

fn blake3_hex(bytes: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::analyze_session;
    use rusqlite::{params, Connection};
    use serde_json::json;

    #[test]
    fn analyzer_returns_findings_for_cors_preflight_case() {
        let conn = Connection::open_in_memory().expect("open db");
        bootstrap_schema(&conn);
        seed_preflight_case(&conn);

        let report = analyze_session(&conn, "sess_preflight").expect("analyze session");
        assert!(!report.findings.is_empty());
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.detector_id == "general.security.cors_preflight_fail.v1"));
    }

    #[test]
    fn analyzer_returns_llm_findings_for_llm_case() {
        let conn = Connection::open_in_memory().expect("open db");
        bootstrap_schema(&conn);
        seed_llm_case(&conn);

        let report = analyze_session(&conn, "sess_llm").expect("analyze session");
        assert!(report.findings.iter().any(|finding| finding.detector_id.starts_with("llm.")));
    }

    fn bootstrap_schema(conn: &Connection) {
        conn.execute_batch(include_str!(
            "../../../crates/dtt-storage/migrations/001_schema_v1.sql"
        ))
        .expect("apply migration 001");
        conn.execute_batch(include_str!(
            "../../../crates/dtt-storage/migrations/002_normalization_columns_v1.sql"
        ))
        .expect("apply migration 002");
    }

    fn seed_preflight_case(conn: &Connection) {
        conn.execute(
            "INSERT INTO sessions (session_id, privacy_mode, capture_source, started_at_ms, created_at_ms, updated_at_ms)
             VALUES (?1, 'metadata_only', 'fixture', 1000, 1000, 1000)",
            params!["sess_preflight"],
        )
        .expect("insert session");

        conn.execute(
            "INSERT INTO network_requests (net_request_id, session_id, started_at_ms, ts_ms, method, host, path, request_headers_json, timing_json, event_seq, scheme, redaction_level)
             VALUES (?1, ?2, 1000, 1000, 'OPTIONS', 'api.example.com', '/v1/chat', ?3, '{}', 1, 'https', 'metadata_only')",
            params!["req_options", "sess_preflight", json!({"origin":"https://app.example.com"}).to_string()],
        )
        .expect("insert options request");
        conn.execute(
            "INSERT INTO network_requests (net_request_id, session_id, started_at_ms, ts_ms, method, host, path, request_headers_json, timing_json, event_seq, scheme, redaction_level)
             VALUES (?1, ?2, 1300, 1300, 'POST', 'api.example.com', '/v1/chat', '{}', '{}', 2, 'https', 'metadata_only')",
            params!["req_post", "sess_preflight"],
        )
        .expect("insert post request");

        conn.execute(
            "INSERT INTO network_responses (net_request_id, session_id, ts_ms, status_code, response_headers_json, headers_hash, stream_summary_json, redaction_level)
             VALUES (?1, ?2, 1100, 403, '{}', 'hash1', '{}', 'metadata_only')",
            params!["req_options", "sess_preflight"],
        )
        .expect("insert response");
        conn.execute(
            "INSERT INTO network_completion (net_request_id, session_id, ts_ms, duration_ms, success, error_text, finished_at_ms, canceled, blocked_reason)
             VALUES (?1, ?2, 1400, 100, 0, 'Failed to fetch', 1400, 1, 'cors')",
            params!["req_post", "sess_preflight"],
        )
        .expect("insert completion");

        conn.execute(
            "INSERT INTO interactions (interaction_id, session_id, interaction_kind, opened_at_ms, closed_at_ms, primary_member_id, rank)
             VALUES (?1, ?2, 'api_burst', 1000, 1400, 'network_request:req_post', 1)",
            params!["int_preflight", "sess_preflight"],
        )
        .expect("insert interaction");
        conn.execute(
            "INSERT INTO interaction_members (interaction_id, member_type, member_id, member_rank, is_primary)
             VALUES (?1, 'network_request', 'req_options', 1, 0)",
            params!["int_preflight"],
        )
        .expect("insert member");
        conn.execute(
            "INSERT INTO interaction_members (interaction_id, member_type, member_id, member_rank, is_primary)
             VALUES (?1, 'network_request', 'req_post', 2, 1)",
            params!["int_preflight"],
        )
        .expect("insert primary member");
    }

    fn seed_llm_case(conn: &Connection) {
        conn.execute(
            "INSERT INTO sessions (session_id, privacy_mode, capture_source, started_at_ms, created_at_ms, updated_at_ms)
             VALUES (?1, 'metadata_only', 'fixture', 2000, 2000, 2000)",
            params!["sess_llm"],
        )
        .expect("insert llm session");

        conn.execute(
            "INSERT INTO network_requests (net_request_id, session_id, started_at_ms, ts_ms, method, host, path, request_headers_json, timing_json, event_seq, scheme, redaction_level)
             VALUES (?1, ?2, 2000, 2000, 'POST', 'api.openai.com', '/v1/responses', '{}', '{}', 1, 'https', 'metadata_only')",
            params!["req_llm", "sess_llm"],
        )
        .expect("insert llm request");
        conn.execute(
            "INSERT INTO network_responses (net_request_id, session_id, ts_ms, status_code, mime_type, encoded_data_length, response_headers_json, headers_hash, stream_summary_json, redaction_level)
             VALUES (?1, ?2, 2100, 200, 'text/event-stream', 900, ?3, 'hashllm', ?4, 'metadata_only')",
            params![
                "req_llm",
                "sess_llm",
                json!({"content-type":"text/event-stream"}).to_string(),
                json!({"transport":"sse","content_type":"text/event-stream","is_streaming":true}).to_string()
            ],
        )
        .expect("insert llm response");
        conn.execute(
            "INSERT INTO network_completion (net_request_id, session_id, ts_ms, duration_ms, success, error_text, finished_at_ms, canceled, blocked_reason)
             VALUES (?1, ?2, 2600, 600, 1, NULL, 2600, 0, NULL)",
            params!["req_llm", "sess_llm"],
        )
        .expect("insert llm completion");
        conn.execute(
            "INSERT INTO interactions (interaction_id, session_id, interaction_kind, opened_at_ms, closed_at_ms, primary_member_id, rank)
             VALUES (?1, ?2, 'llm_message', 2000, 2600, 'network_request:req_llm', 1)",
            params!["int_llm", "sess_llm"],
        )
        .expect("insert llm interaction");
        conn.execute(
            "INSERT INTO interaction_members (interaction_id, member_type, member_id, member_rank, is_primary)
             VALUES (?1, 'network_request', 'req_llm', 1, 1)",
            params!["int_llm"],
        )
        .expect("insert llm member");

        let raw_payload = json!({
            "method":"Network.requestWillBeSent",
            "params":{"requestId":"req_llm","payload":{"model":"gpt-5","tools":[{"type":"function"}]}}
        });
        conn.execute(
            "INSERT INTO events_raw (event_id, session_id, event_seq, ts_ms, cdp_method, payload_encoding, payload_bytes, payload_hash, payload_len, redaction_level, created_at_ms)
             VALUES (?1, ?2, 1, 2000, 'Network.requestWillBeSent', 'plain', ?3, 'rawhash', ?4, 'metadata_only', 2000)",
            params![
                "evt_llm_1",
                "sess_llm",
                serde_json::to_vec(&raw_payload).expect("serialize raw payload"),
                i64::try_from(serde_json::to_vec(&raw_payload).expect("serialize len").len()).expect("len i64")
            ],
        )
        .expect("insert raw event");
    }
}
