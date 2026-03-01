//! Deterministic interaction correlation v1.0.

#![forbid(unsafe_code)]

use blake3::Hasher;
use dtt_core::{
    CorrelationConstantsV1, HeaderMap, HeaderValue, InteractionKindV1, InteractionMemberTypeV1,
    NormalizedInteractionMemberRecordV1, NormalizedInteractionRecordV1, StreamTransport,
};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LlmWeightsV1 {
    pub host_match: i64,
    pub streaming_signal: i64,
    pub content_type: i64,
    pub payload_markers: i64,
}

impl Default for LlmWeightsV1 {
    fn default() -> Self {
        Self { host_match: 30, streaming_signal: 25, content_type: 20, payload_markers: 25 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrelationConfig {
    pub constants: CorrelationConstantsV1,
    pub telemetry_host_substrings: Vec<String>,
    pub telemetry_path_substrings: Vec<String>,
    pub llm_provider_hosts: Vec<String>,
    pub llm_weights: LlmWeightsV1,
    pub llm_primary_threshold: i64,
}

impl Default for CorrelationConfig {
    fn default() -> Self {
        Self {
            constants: CorrelationConstantsV1::default(),
            telemetry_host_substrings: Vec::new(),
            telemetry_path_substrings: Vec::new(),
            llm_provider_hosts: Vec::new(),
            llm_weights: LlmWeightsV1::default(),
            llm_primary_threshold: 70,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestCandidateInput {
    pub net_request_id: String,
    pub ts_ms: i64,
    pub started_at_ms: i64,
    pub method: Option<String>,
    pub host: Option<String>,
    pub path: Option<String>,
    pub scheme: Option<String>,
    pub request_headers: HeaderMap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseCandidateInput {
    pub net_request_id: String,
    pub ts_ms: i64,
    pub status_code: Option<i64>,
    pub mime_type: Option<String>,
    pub stream_transport: Option<StreamTransport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionCandidateInput {
    pub net_request_id: String,
    pub ts_ms: i64,
    pub duration_ms: Option<i64>,
    pub success: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleCandidateInput {
    pub console_id: String,
    pub ts_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleCandidateInput {
    pub lifecycle_id: String,
    pub ts_ms: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawRequestHintInput {
    pub net_request_id: String,
    pub request_type: Option<String>,
    pub has_websocket_activity: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrelationInput {
    pub session_id: String,
    pub requests: Vec<RequestCandidateInput>,
    pub responses: Vec<ResponseCandidateInput>,
    pub completions: Vec<CompletionCandidateInput>,
    pub console_entries: Vec<ConsoleCandidateInput>,
    pub lifecycle_entries: Vec<LifecycleCandidateInput>,
    pub raw_request_hints: Vec<RawRequestHintInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrelationOutput {
    pub interactions: Vec<NormalizedInteractionRecordV1>,
    pub members: Vec<NormalizedInteractionMemberRecordV1>,
    pub request_candidates_seen: usize,
    pub unassigned_candidates: usize,
    pub skipped_candidates: usize,
}

#[derive(Debug, Clone)]
struct RequestFeatures {
    request: RequestCandidateInput,
    status_code: Option<i64>,
    mime_type: Option<String>,
    stream_transport: Option<StreamTransport>,
    duration_ms: Option<i64>,
    response_ts_ms: Option<i64>,
    completion_ts_ms: Option<i64>,
    request_type: Option<String>,
}

#[derive(Debug, Clone)]
struct InteractionDraft {
    kind: InteractionKindV1,
    request_ids: Vec<String>,
    opened_at_ms: i64,
    closed_at_ms: i64,
}

#[derive(Debug, Clone)]
struct DraftMember {
    ts_ms: i64,
    member_type: InteractionMemberTypeV1,
    member_id: String,
    is_primary: bool,
}

#[must_use]
pub fn correlate(input: CorrelationInput, config: CorrelationConfig) -> CorrelationOutput {
    let request_index = build_request_index(&input);
    if request_index.is_empty() {
        return CorrelationOutput {
            interactions: Vec::new(),
            members: Vec::new(),
            request_candidates_seen: 0,
            unassigned_candidates: 0,
            skipped_candidates: 0,
        };
    }

    let mut ordered_request_ids: Vec<String> = request_index.keys().cloned().collect();
    ordered_request_ids.sort_by(|left, right| {
        let l = request_index.get(left).expect("left request");
        let r = request_index.get(right).expect("right request");
        cmp_request_features(l, r)
    });

    let mut assigned: HashMap<String, bool> =
        ordered_request_ids.iter().map(|id| (id.clone(), false)).collect();
    let mut drafts: Vec<InteractionDraft> = Vec::new();

    for request_id in &ordered_request_ids {
        if assigned.get(request_id).copied().unwrap_or(false) {
            continue;
        }
        let Some(seed) = request_index.get(request_id) else {
            continue;
        };

        let seed_kind = classify_kind(seed, &config);
        let clustered_ids = match seed_kind {
            InteractionKindV1::PageLoad => {
                collect_page_load(seed, &ordered_request_ids, &request_index, &assigned, &config)
            }
            InteractionKindV1::ApiBurst => {
                collect_api_burst(seed, &ordered_request_ids, &request_index, &assigned, &config)
            }
            InteractionKindV1::Upload => {
                collect_upload(seed, &ordered_request_ids, &request_index, &assigned, &config)
            }
            InteractionKindV1::LlmMessage | InteractionKindV1::LlmRegen => {
                collect_llm(seed, &ordered_request_ids, &request_index, &assigned, &config)
            }
            InteractionKindV1::Other => vec![seed.request.net_request_id.clone()],
        };

        let request_ids = if clustered_ids.is_empty() {
            vec![seed.request.net_request_id.clone()]
        } else {
            clustered_ids
        };

        for clustered_id in &request_ids {
            assigned.insert(clustered_id.clone(), true);
        }

        let opened_at_ms = request_ids
            .iter()
            .filter_map(|id| request_index.get(id).map(|request| request.request.started_at_ms))
            .min()
            .unwrap_or(seed.request.started_at_ms);
        let closed_at_ms = request_ids
            .iter()
            .filter_map(|id| request_index.get(id).map(request_closed_at))
            .max()
            .unwrap_or(seed.request.started_at_ms);

        drafts.push(InteractionDraft { kind: seed_kind, request_ids, opened_at_ms, closed_at_ms });
    }

    drafts.sort_by(|left, right| {
        left.opened_at_ms
            .cmp(&right.opened_at_ms)
            .then_with(|| kind_as_str(left.kind).cmp(kind_as_str(right.kind)))
            .then_with(|| left.request_ids.cmp(&right.request_ids))
    });

    apply_llm_regen_pass(&mut drafts, &request_index, &config);

    let mut interactions: Vec<NormalizedInteractionRecordV1> = Vec::new();
    let mut members: Vec<NormalizedInteractionMemberRecordV1> = Vec::new();
    let mut draft_members_by_interaction: HashMap<String, Vec<DraftMember>> = HashMap::new();

    for (idx, draft) in drafts.iter().enumerate() {
        let rank = u32::try_from(idx + 1).unwrap_or(u32::MAX);
        let primary_request_id = pick_primary_request_id(draft, &request_index, &config)
            .unwrap_or_else(|| {
                draft.request_ids.first().cloned().unwrap_or_else(|| "unknown_request".to_string())
            });
        let interaction_id = derive_interaction_id(
            &input.session_id,
            draft.kind,
            draft.opened_at_ms,
            draft.closed_at_ms,
            rank,
            &primary_request_id,
        );
        let primary_member_id = format!("network_request:{primary_request_id}");

        let interaction = NormalizedInteractionRecordV1 {
            interaction_id: interaction_id.clone(),
            session_id: input.session_id.clone(),
            interaction_kind: draft.kind,
            opened_at_ms: draft.opened_at_ms,
            closed_at_ms: Some(draft.closed_at_ms),
            primary_member_id: Some(primary_member_id),
            rank,
        };
        interactions.push(interaction);

        let mut draft_members: Vec<DraftMember> = Vec::new();
        for request_id in &draft.request_ids {
            if let Some(request) = request_index.get(request_id) {
                draft_members.push(DraftMember {
                    ts_ms: request.request.started_at_ms,
                    member_type: InteractionMemberTypeV1::NetworkRequest,
                    member_id: request_id.clone(),
                    is_primary: request_id == &primary_request_id,
                });
                if request.response_ts_ms.is_some() {
                    draft_members.push(DraftMember {
                        ts_ms: request.response_ts_ms.unwrap_or(request.request.started_at_ms),
                        member_type: InteractionMemberTypeV1::NetworkResponse,
                        member_id: request_id.clone(),
                        is_primary: false,
                    });
                }
                if request.completion_ts_ms.is_some() {
                    draft_members.push(DraftMember {
                        ts_ms: request.completion_ts_ms.unwrap_or(request.request.started_at_ms),
                        member_type: InteractionMemberTypeV1::NetworkCompletion,
                        member_id: request_id.clone(),
                        is_primary: false,
                    });
                }
            }
        }
        draft_members_by_interaction.insert(interaction_id, draft_members);
    }

    attach_console_and_lifecycle_members(
        &interactions,
        &input.console_entries,
        &input.lifecycle_entries,
        &config,
        &mut draft_members_by_interaction,
    );

    let mut sorted_interactions = interactions;
    sorted_interactions.sort_by(|left, right| {
        left.opened_at_ms
            .cmp(&right.opened_at_ms)
            .then_with(|| {
                kind_as_str(left.interaction_kind).cmp(kind_as_str(right.interaction_kind))
            })
            .then_with(|| left.interaction_id.cmp(&right.interaction_id))
    });

    for interaction in &sorted_interactions {
        if let Some(draft_members) =
            draft_members_by_interaction.get_mut(&interaction.interaction_id)
        {
            draft_members.sort_by(|left, right| {
                left.ts_ms
                    .cmp(&right.ts_ms)
                    .then_with(|| {
                        member_type_rank(left.member_type).cmp(&member_type_rank(right.member_type))
                    })
                    .then_with(|| left.member_id.cmp(&right.member_id))
            });

            for (member_idx, draft_member) in draft_members.iter().enumerate() {
                let member_rank = u32::try_from(member_idx + 1).unwrap_or(u32::MAX);
                members.push(NormalizedInteractionMemberRecordV1 {
                    interaction_id: interaction.interaction_id.clone(),
                    member_type: draft_member.member_type,
                    member_id: draft_member.member_id.clone(),
                    member_rank,
                    is_primary: draft_member.is_primary,
                });
            }
        }
    }

    members.sort_by(|left, right| {
        left.interaction_id
            .cmp(&right.interaction_id)
            .then_with(|| left.member_rank.cmp(&right.member_rank))
            .then_with(|| {
                member_type_rank(left.member_type).cmp(&member_type_rank(right.member_type))
            })
            .then_with(|| left.member_id.cmp(&right.member_id))
    });

    CorrelationOutput {
        interactions: sorted_interactions,
        members,
        request_candidates_seen: request_index.len(),
        unassigned_candidates: assigned.values().filter(|assigned_value| !**assigned_value).count(),
        skipped_candidates: 0,
    }
}

fn build_request_index(input: &CorrelationInput) -> HashMap<String, RequestFeatures> {
    let mut request_index: HashMap<String, RequestFeatures> = HashMap::new();

    for request in &input.requests {
        request_index.insert(
            request.net_request_id.clone(),
            RequestFeatures {
                request: request.clone(),
                status_code: None,
                mime_type: None,
                stream_transport: None,
                duration_ms: None,
                response_ts_ms: None,
                completion_ts_ms: None,
                request_type: None,
            },
        );
    }

    for response in &input.responses {
        if let Some(request) = request_index.get_mut(&response.net_request_id) {
            request.status_code = response.status_code;
            request.mime_type = response.mime_type.clone();
            request.stream_transport = response.stream_transport;
            request.response_ts_ms = Some(response.ts_ms);
        }
    }

    for completion in &input.completions {
        if let Some(request) = request_index.get_mut(&completion.net_request_id) {
            request.duration_ms = completion.duration_ms;
            request.completion_ts_ms = Some(completion.ts_ms);
        }
    }

    for hint in &input.raw_request_hints {
        if let Some(request) = request_index.get_mut(&hint.net_request_id) {
            request.request_type = hint.request_type.clone();
        }
    }

    request_index
}

fn cmp_request_features(left: &RequestFeatures, right: &RequestFeatures) -> Ordering {
    left.request
        .started_at_ms
        .cmp(&right.request.started_at_ms)
        .then_with(|| left.request.ts_ms.cmp(&right.request.ts_ms))
        .then_with(|| left.request.net_request_id.cmp(&right.request.net_request_id))
}

fn request_closed_at(request: &RequestFeatures) -> i64 {
    request.completion_ts_ms.or(request.response_ts_ms).unwrap_or(request.request.started_at_ms)
}

fn classify_kind(request: &RequestFeatures, config: &CorrelationConfig) -> InteractionKindV1 {
    let llm_score = llm_score(request, config);
    if llm_score >= config.llm_primary_threshold {
        return InteractionKindV1::LlmMessage;
    }

    if is_upload_request(request) {
        return InteractionKindV1::Upload;
    }

    if is_page_load_request(request) {
        return InteractionKindV1::PageLoad;
    }

    if request.request.method.is_some() {
        return InteractionKindV1::ApiBurst;
    }

    InteractionKindV1::Other
}

fn is_page_load_request(request: &RequestFeatures) -> bool {
    if request
        .request_type
        .as_deref()
        .map(|value| value.eq_ignore_ascii_case("Document"))
        .unwrap_or(false)
    {
        return true;
    }

    matches!(
        request.request.method.as_deref(),
        Some(method) if method.eq_ignore_ascii_case("GET")
    ) && request.request.path.as_deref() == Some("/")
}

fn is_upload_request(request: &RequestFeatures) -> bool {
    if let Some(content_type) = header_first_value(&request.request.request_headers, "content-type")
    {
        if content_type.to_ascii_lowercase().contains("multipart/form-data") {
            return true;
        }
    }

    request
        .request
        .path
        .as_deref()
        .map(|path| path.to_ascii_lowercase().contains("upload"))
        .unwrap_or(false)
}

fn is_telemetry_request(request: &RequestFeatures, config: &CorrelationConfig) -> bool {
    let host = request.request.host.as_deref().unwrap_or_default().to_ascii_lowercase();
    let path = request.request.path.as_deref().unwrap_or_default().to_ascii_lowercase();

    config.telemetry_host_substrings.iter().any(|needle| host.contains(needle))
        || config.telemetry_path_substrings.iter().any(|needle| path.contains(needle))
}

fn llm_score(request: &RequestFeatures, config: &CorrelationConfig) -> i64 {
    let mut score = 0_i64;
    let host = request.request.host.as_deref().unwrap_or_default().to_ascii_lowercase();
    let path = request.request.path.as_deref().unwrap_or_default().to_ascii_lowercase();
    let mime = request.mime_type.as_deref().unwrap_or_default().to_ascii_lowercase();

    if config.llm_provider_hosts.iter().any(|provider| host.contains(provider)) {
        score += config.llm_weights.host_match;
    }

    if matches!(request.stream_transport, Some(StreamTransport::Sse | StreamTransport::Websocket)) {
        score += config.llm_weights.streaming_signal;
    }

    if mime.contains("event-stream") || mime.contains("json") {
        score += config.llm_weights.content_type;
    }

    if path.contains("chat")
        || path.contains("completion")
        || path.contains("response")
        || path.contains("message")
    {
        score += config.llm_weights.payload_markers;
    }

    score
}

fn collect_page_load(
    seed: &RequestFeatures,
    ordered_ids: &[String],
    request_index: &HashMap<String, RequestFeatures>,
    assigned: &HashMap<String, bool>,
    config: &CorrelationConfig,
) -> Vec<String> {
    let opened = seed.request.started_at_ms;
    let hard_limit = opened + config.constants.pageload_hard_timeout_ms;
    let mut ids: Vec<String> = Vec::new();

    for request_id in ordered_ids {
        if assigned.get(request_id).copied().unwrap_or(false) {
            continue;
        }
        let Some(candidate) = request_index.get(request_id) else {
            continue;
        };
        if candidate.request.started_at_ms < opened || candidate.request.started_at_ms > hard_limit
        {
            continue;
        }

        // Keep document/static requests with page load and leave API calls for burst grouping.
        let request_type = candidate.request_type.as_deref().unwrap_or_default();
        if request_type.eq_ignore_ascii_case("Fetch")
            || request_type.eq_ignore_ascii_case("XHR")
            || request_type.eq_ignore_ascii_case("Preflight")
        {
            continue;
        }
        ids.push(request_id.clone());
    }

    if ids.is_empty() {
        ids.push(seed.request.net_request_id.clone());
    }
    ids
}

fn collect_api_burst(
    seed: &RequestFeatures,
    ordered_ids: &[String],
    request_index: &HashMap<String, RequestFeatures>,
    assigned: &HashMap<String, bool>,
    config: &CorrelationConfig,
) -> Vec<String> {
    let mut ids: Vec<String> = vec![seed.request.net_request_id.clone()];
    let mut last_ts = seed.request.started_at_ms;
    let max_end = seed.request.started_at_ms + config.constants.burst_max_window_ms;
    let mut saw_seed = false;

    for request_id in ordered_ids {
        let Some(candidate) = request_index.get(request_id) else {
            continue;
        };
        if request_id == &seed.request.net_request_id {
            saw_seed = true;
            continue;
        }
        if !saw_seed || assigned.get(request_id).copied().unwrap_or(false) {
            continue;
        }
        if candidate.request.started_at_ms > max_end {
            break;
        }

        let gap = candidate.request.started_at_ms - last_ts;
        if gap > config.constants.burst_gap_ms {
            break;
        }

        ids.push(request_id.clone());
        last_ts = candidate.request.started_at_ms;
    }

    attach_preflight_pair(ids, seed, ordered_ids, request_index, assigned, config)
}

fn collect_upload(
    seed: &RequestFeatures,
    ordered_ids: &[String],
    request_index: &HashMap<String, RequestFeatures>,
    assigned: &HashMap<String, bool>,
    config: &CorrelationConfig,
) -> Vec<String> {
    let mut ids = vec![seed.request.net_request_id.clone()];
    let endpoint = endpoint_key(seed);
    let start = seed.request.started_at_ms;

    for request_id in ordered_ids {
        if request_id == &seed.request.net_request_id
            || assigned.get(request_id).copied().unwrap_or(false)
        {
            continue;
        }
        let Some(candidate) = request_index.get(request_id) else {
            continue;
        };
        if endpoint_key(candidate) != endpoint {
            continue;
        }
        let delta = candidate.request.started_at_ms - start;
        if delta < 0 || delta > config.constants.interaction_close_idle_ms {
            continue;
        }
        ids.push(request_id.clone());
    }

    ids.sort();
    ids
}

fn collect_llm(
    seed: &RequestFeatures,
    ordered_ids: &[String],
    request_index: &HashMap<String, RequestFeatures>,
    assigned: &HashMap<String, bool>,
    config: &CorrelationConfig,
) -> Vec<String> {
    // Keep one request per llm interaction by default, while still pairing preflight.
    attach_preflight_pair(
        vec![seed.request.net_request_id.clone()],
        seed,
        ordered_ids,
        request_index,
        assigned,
        config,
    )
}

fn attach_preflight_pair(
    mut ids: Vec<String>,
    seed: &RequestFeatures,
    ordered_ids: &[String],
    request_index: &HashMap<String, RequestFeatures>,
    assigned: &HashMap<String, bool>,
    config: &CorrelationConfig,
) -> Vec<String> {
    let seed_is_options = seed
        .request
        .method
        .as_deref()
        .map(|method| method.eq_ignore_ascii_case("OPTIONS"))
        .unwrap_or(false);
    let seed_endpoint = endpoint_key(seed);
    let seed_ts = seed.request.started_at_ms;

    if seed_is_options {
        for request_id in ordered_ids {
            if ids.contains(request_id) || assigned.get(request_id).copied().unwrap_or(false) {
                continue;
            }
            let Some(candidate) = request_index.get(request_id) else {
                continue;
            };
            if candidate.request.started_at_ms < seed_ts {
                continue;
            }
            if candidate.request.started_at_ms - seed_ts
                > config.constants.preflight_followup_window_ms
            {
                break;
            }
            let is_followup = candidate
                .request
                .method
                .as_deref()
                .map(|method| !method.eq_ignore_ascii_case("OPTIONS"))
                .unwrap_or(false);
            if is_followup && endpoint_key(candidate) == seed_endpoint {
                ids.push(request_id.clone());
                break;
            }
        }
    }

    ids.sort();
    ids.dedup();
    ids
}

fn endpoint_key(request: &RequestFeatures) -> String {
    let scheme = request.request.scheme.as_deref().unwrap_or_default().to_ascii_lowercase();
    let host = request.request.host.as_deref().unwrap_or_default().to_ascii_lowercase();
    let path = request.request.path.as_deref().unwrap_or_default().to_ascii_lowercase();
    format!("{scheme}|{host}|{path}")
}

fn apply_llm_regen_pass(
    drafts: &mut [InteractionDraft],
    request_index: &HashMap<String, RequestFeatures>,
    config: &CorrelationConfig,
) {
    let mut last_by_endpoint: HashMap<String, i64> = HashMap::new();
    for draft in drafts.iter_mut() {
        if draft.kind != InteractionKindV1::LlmMessage {
            continue;
        }
        let Some(first_request_id) = draft.request_ids.first() else {
            continue;
        };
        let Some(first_request) = request_index.get(first_request_id) else {
            continue;
        };
        let endpoint = endpoint_key(first_request);
        if let Some(previous_closed_at) = last_by_endpoint.get(&endpoint).copied() {
            let delta = draft.opened_at_ms - previous_closed_at;
            if (0..=config.constants.interaction_close_idle_ms).contains(&delta) {
                draft.kind = InteractionKindV1::LlmRegen;
            }
        }
        last_by_endpoint.insert(endpoint, draft.closed_at_ms);
    }
}

fn pick_primary_request_id(
    draft: &InteractionDraft,
    request_index: &HashMap<String, RequestFeatures>,
    config: &CorrelationConfig,
) -> Option<String> {
    let mut scored: Vec<(String, i64, i64)> = Vec::new();
    for request_id in &draft.request_ids {
        let request = request_index.get(request_id)?;
        let started = request.request.started_at_ms;
        let score = match draft.kind {
            InteractionKindV1::PageLoad => {
                let mut score = 0_i64;
                if request
                    .request_type
                    .as_deref()
                    .map(|value| value.eq_ignore_ascii_case("Document"))
                    .unwrap_or(false)
                {
                    score += 100;
                }
                if request
                    .request
                    .method
                    .as_deref()
                    .map(|method| method.eq_ignore_ascii_case("GET"))
                    .unwrap_or(false)
                {
                    score += 10;
                }
                score
            }
            InteractionKindV1::ApiBurst => {
                let mut score = 0_i64;
                let request_type = request.request_type.as_deref().unwrap_or_default();
                if request_type.eq_ignore_ascii_case("Fetch")
                    || request_type.eq_ignore_ascii_case("XHR")
                {
                    score += 25;
                }
                if request.status_code.map(|status| status >= 400).unwrap_or(false) {
                    score += 20;
                }
                if request.duration_ms.map(|duration| duration >= 2_000).unwrap_or(false) {
                    score += 10;
                }
                if request
                    .request
                    .method
                    .as_deref()
                    .map(|method| !method.eq_ignore_ascii_case("OPTIONS"))
                    .unwrap_or(false)
                {
                    score += 5;
                }
                if is_telemetry_request(request, config) {
                    score -= 20;
                }
                score
            }
            InteractionKindV1::LlmMessage | InteractionKindV1::LlmRegen => {
                llm_score(request, config)
            }
            InteractionKindV1::Upload => {
                if is_upload_request(request) {
                    100
                } else {
                    0
                }
            }
            InteractionKindV1::Other => 0,
        };
        scored.push((request_id.clone(), score, started));
    }

    scored.sort_by(|left, right| {
        right.1.cmp(&left.1).then_with(|| left.2.cmp(&right.2)).then_with(|| left.0.cmp(&right.0))
    });
    scored.first().map(|entry| entry.0.clone())
}

fn attach_console_and_lifecycle_members(
    interactions: &[NormalizedInteractionRecordV1],
    console_entries: &[ConsoleCandidateInput],
    lifecycle_entries: &[LifecycleCandidateInput],
    config: &CorrelationConfig,
    members_by_interaction: &mut HashMap<String, Vec<DraftMember>>,
) {
    for console in console_entries {
        if let Some(interaction_id) = pick_interaction_for_ts(interactions, console.ts_ms, config) {
            members_by_interaction.entry(interaction_id).or_default().push(DraftMember {
                ts_ms: console.ts_ms,
                member_type: InteractionMemberTypeV1::ConsoleEntry,
                member_id: console.console_id.clone(),
                is_primary: false,
            });
        }
    }

    for lifecycle in lifecycle_entries {
        if let Some(interaction_id) = pick_interaction_for_ts(interactions, lifecycle.ts_ms, config)
        {
            members_by_interaction.entry(interaction_id).or_default().push(DraftMember {
                ts_ms: lifecycle.ts_ms,
                member_type: InteractionMemberTypeV1::PageLifecycle,
                member_id: lifecycle.lifecycle_id.clone(),
                is_primary: false,
            });
        }
    }
}

fn pick_interaction_for_ts(
    interactions: &[NormalizedInteractionRecordV1],
    ts_ms: i64,
    config: &CorrelationConfig,
) -> Option<String> {
    let mut candidates: Vec<(String, i64)> = Vec::new();
    for interaction in interactions {
        let opened = interaction.opened_at_ms - config.constants.stream_end_grace_ms;
        let closed = interaction.closed_at_ms.unwrap_or(interaction.opened_at_ms)
            + config.constants.interaction_close_idle_ms;
        if ts_ms < opened || ts_ms > closed {
            continue;
        }
        let distance = if ts_ms < interaction.opened_at_ms {
            interaction.opened_at_ms - ts_ms
        } else if let Some(closed_at) = interaction.closed_at_ms {
            if ts_ms > closed_at {
                ts_ms - closed_at
            } else {
                0
            }
        } else {
            0
        };
        candidates.push((interaction.interaction_id.clone(), distance));
    }

    candidates.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)));
    candidates.first().map(|candidate| candidate.0.clone())
}

fn derive_interaction_id(
    session_id: &str,
    kind: InteractionKindV1,
    opened_at_ms: i64,
    closed_at_ms: i64,
    rank: u32,
    primary_request_id: &str,
) -> String {
    let signature = format!(
        "{session_id}:{}:{opened_at_ms}:{closed_at_ms}:{rank}:{primary_request_id}",
        kind_as_str(kind)
    );
    format!("int_{}", blake3_hex(signature.as_bytes()))
}

fn kind_as_str(kind: InteractionKindV1) -> &'static str {
    match kind {
        InteractionKindV1::PageLoad => "page_load",
        InteractionKindV1::ApiBurst => "api_burst",
        InteractionKindV1::LlmMessage => "llm_message",
        InteractionKindV1::LlmRegen => "llm_regen",
        InteractionKindV1::Upload => "upload",
        InteractionKindV1::Other => "other",
    }
}

fn member_type_rank(member_type: InteractionMemberTypeV1) -> u8 {
    match member_type {
        InteractionMemberTypeV1::NetworkRequest => 1,
        InteractionMemberTypeV1::NetworkResponse => 2,
        InteractionMemberTypeV1::NetworkCompletion => 3,
        InteractionMemberTypeV1::ConsoleEntry => 4,
        InteractionMemberTypeV1::PageLifecycle => 5,
        InteractionMemberTypeV1::RawEvent => 6,
    }
}

fn header_first_value(headers: &HeaderMap, header_name: &str) -> Option<String> {
    let value = headers.get(&header_name.to_ascii_lowercase())?;
    match value {
        HeaderValue::Single(single) => Some(single.clone()),
        HeaderValue::Multi(values) => values.first().cloned(),
    }
}

fn blake3_hex(bytes: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        correlate, CompletionCandidateInput, ConsoleCandidateInput, CorrelationConfig,
        CorrelationInput, LifecycleCandidateInput, RawRequestHintInput, RequestCandidateInput,
        ResponseCandidateInput,
    };
    use dtt_core::{HeaderMap, HeaderValue, InteractionKindV1, StreamTransport};

    fn empty_headers() -> HeaderMap {
        HeaderMap::new()
    }

    #[test]
    fn preflight_and_followup_are_grouped_in_single_interaction() {
        let input = CorrelationInput {
            session_id: "sess_preflight".to_string(),
            requests: vec![
                RequestCandidateInput {
                    net_request_id: "req_options".to_string(),
                    ts_ms: 1_000,
                    started_at_ms: 1_000,
                    method: Some("OPTIONS".to_string()),
                    host: Some("api.example.com".to_string()),
                    path: Some("/v1/chat".to_string()),
                    scheme: Some("https".to_string()),
                    request_headers: empty_headers(),
                },
                RequestCandidateInput {
                    net_request_id: "req_post".to_string(),
                    ts_ms: 1_500,
                    started_at_ms: 1_500,
                    method: Some("POST".to_string()),
                    host: Some("api.example.com".to_string()),
                    path: Some("/v1/chat".to_string()),
                    scheme: Some("https".to_string()),
                    request_headers: empty_headers(),
                },
            ],
            responses: vec![],
            completions: vec![],
            console_entries: vec![],
            lifecycle_entries: vec![],
            raw_request_hints: vec![],
        };

        let output = correlate(input, CorrelationConfig::default());
        assert_eq!(output.interactions.len(), 1);
        let request_member_count = output
            .members
            .iter()
            .filter(|member| {
                member.member_type == dtt_core::InteractionMemberTypeV1::NetworkRequest
            })
            .count();
        assert_eq!(request_member_count, 2);
    }

    #[test]
    fn llm_candidate_uses_priority_over_api_burst() {
        let input = CorrelationInput {
            session_id: "sess_llm".to_string(),
            requests: vec![RequestCandidateInput {
                net_request_id: "req_llm".to_string(),
                ts_ms: 1_000,
                started_at_ms: 1_000,
                method: Some("POST".to_string()),
                host: Some("api.openai.com".to_string()),
                path: Some("/v1/responses".to_string()),
                scheme: Some("https".to_string()),
                request_headers: empty_headers(),
            }],
            responses: vec![ResponseCandidateInput {
                net_request_id: "req_llm".to_string(),
                ts_ms: 1_100,
                status_code: Some(200),
                mime_type: Some("text/event-stream".to_string()),
                stream_transport: Some(StreamTransport::Sse),
            }],
            completions: vec![],
            console_entries: vec![],
            lifecycle_entries: vec![],
            raw_request_hints: vec![],
        };

        let mut config = CorrelationConfig::default();
        config.llm_provider_hosts.push("openai.com".to_string());
        let output = correlate(input, config);
        assert_eq!(output.interactions.len(), 1);
        assert_eq!(output.interactions[0].interaction_kind, InteractionKindV1::LlmMessage);
    }

    #[test]
    fn llm_regen_is_marked_when_followup_is_within_idle_window() {
        let input = CorrelationInput {
            session_id: "sess_regen".to_string(),
            requests: vec![
                RequestCandidateInput {
                    net_request_id: "req_1".to_string(),
                    ts_ms: 1_000,
                    started_at_ms: 1_000,
                    method: Some("POST".to_string()),
                    host: Some("api.openai.com".to_string()),
                    path: Some("/v1/responses".to_string()),
                    scheme: Some("https".to_string()),
                    request_headers: empty_headers(),
                },
                RequestCandidateInput {
                    net_request_id: "req_2".to_string(),
                    ts_ms: 3_200,
                    started_at_ms: 3_200,
                    method: Some("POST".to_string()),
                    host: Some("api.openai.com".to_string()),
                    path: Some("/v1/responses".to_string()),
                    scheme: Some("https".to_string()),
                    request_headers: empty_headers(),
                },
            ],
            responses: vec![
                ResponseCandidateInput {
                    net_request_id: "req_1".to_string(),
                    ts_ms: 1_300,
                    status_code: Some(200),
                    mime_type: Some("text/event-stream".to_string()),
                    stream_transport: Some(StreamTransport::Sse),
                },
                ResponseCandidateInput {
                    net_request_id: "req_2".to_string(),
                    ts_ms: 3_300,
                    status_code: Some(200),
                    mime_type: Some("text/event-stream".to_string()),
                    stream_transport: Some(StreamTransport::Sse),
                },
            ],
            completions: vec![
                CompletionCandidateInput {
                    net_request_id: "req_1".to_string(),
                    ts_ms: 1_500,
                    duration_ms: Some(500),
                    success: Some(true),
                },
                CompletionCandidateInput {
                    net_request_id: "req_2".to_string(),
                    ts_ms: 3_700,
                    duration_ms: Some(500),
                    success: Some(true),
                },
            ],
            console_entries: vec![],
            lifecycle_entries: vec![],
            raw_request_hints: vec![],
        };

        let mut config = CorrelationConfig::default();
        config.llm_provider_hosts.push("openai.com".to_string());

        let output = correlate(input, config);
        assert_eq!(output.interactions.len(), 2);
        assert_eq!(output.interactions[0].interaction_kind, InteractionKindV1::LlmMessage);
        assert_eq!(output.interactions[1].interaction_kind, InteractionKindV1::LlmRegen);
    }

    #[test]
    fn api_burst_primary_deprioritizes_telemetry() {
        let input = CorrelationInput {
            session_id: "sess_api".to_string(),
            requests: vec![
                RequestCandidateInput {
                    net_request_id: "req_telemetry".to_string(),
                    ts_ms: 1_000,
                    started_at_ms: 1_000,
                    method: Some("POST".to_string()),
                    host: Some("telemetry.example.com".to_string()),
                    path: Some("/collect".to_string()),
                    scheme: Some("https".to_string()),
                    request_headers: empty_headers(),
                },
                RequestCandidateInput {
                    net_request_id: "req_api".to_string(),
                    ts_ms: 1_200,
                    started_at_ms: 1_200,
                    method: Some("GET".to_string()),
                    host: Some("api.example.com".to_string()),
                    path: Some("/v1/data".to_string()),
                    scheme: Some("https".to_string()),
                    request_headers: empty_headers(),
                },
            ],
            responses: vec![ResponseCandidateInput {
                net_request_id: "req_api".to_string(),
                ts_ms: 1_300,
                status_code: Some(500),
                mime_type: Some("application/json".to_string()),
                stream_transport: Some(StreamTransport::Unknown),
            }],
            completions: vec![CompletionCandidateInput {
                net_request_id: "req_api".to_string(),
                ts_ms: 3_900,
                duration_ms: Some(2_700),
                success: Some(false),
            }],
            console_entries: vec![ConsoleCandidateInput {
                console_id: "console_1".to_string(),
                ts_ms: 1_500,
            }],
            lifecycle_entries: vec![LifecycleCandidateInput {
                lifecycle_id: "life_1".to_string(),
                ts_ms: 1_550,
                name: "networkIdle".to_string(),
            }],
            raw_request_hints: vec![
                RawRequestHintInput {
                    net_request_id: "req_telemetry".to_string(),
                    request_type: Some("Fetch".to_string()),
                    has_websocket_activity: false,
                },
                RawRequestHintInput {
                    net_request_id: "req_api".to_string(),
                    request_type: Some("XHR".to_string()),
                    has_websocket_activity: false,
                },
            ],
        };

        let config = CorrelationConfig {
            telemetry_host_substrings: vec!["telemetry".to_string()],
            telemetry_path_substrings: vec!["/collect".to_string()],
            ..CorrelationConfig::default()
        };

        let output = correlate(input, config);
        assert!(!output.interactions.is_empty());
        let primary = output.interactions[0].primary_member_id.clone().expect("primary member id");
        assert_eq!(primary, "network_request:req_api");
    }

    #[test]
    fn output_is_stable_for_shuffled_request_inputs() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "content-type".to_string(),
            HeaderValue::Single("application/json".to_string()),
        );

        let input_left = CorrelationInput {
            session_id: "sess_det".to_string(),
            requests: vec![
                RequestCandidateInput {
                    net_request_id: "req_b".to_string(),
                    ts_ms: 2_000,
                    started_at_ms: 2_000,
                    method: Some("GET".to_string()),
                    host: Some("api.example.com".to_string()),
                    path: Some("/b".to_string()),
                    scheme: Some("https".to_string()),
                    request_headers: headers.clone(),
                },
                RequestCandidateInput {
                    net_request_id: "req_a".to_string(),
                    ts_ms: 1_000,
                    started_at_ms: 1_000,
                    method: Some("GET".to_string()),
                    host: Some("api.example.com".to_string()),
                    path: Some("/a".to_string()),
                    scheme: Some("https".to_string()),
                    request_headers: headers,
                },
            ],
            responses: vec![],
            completions: vec![],
            console_entries: vec![],
            lifecycle_entries: vec![],
            raw_request_hints: vec![],
        };

        let mut input_right = input_left.clone();
        input_right.requests.reverse();

        let left = correlate(input_left, CorrelationConfig::default());
        let right = correlate(input_right, CorrelationConfig::default());

        assert_eq!(left.interactions, right.interactions);
        assert_eq!(left.members, right.members);
    }
}
