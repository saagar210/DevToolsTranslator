//! Shared core contracts for DevTools Translator v1.0.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub const ENVELOPE_VERSION: u8 = 1;
pub const EVT_RAW_EVENT: &str = "evt.raw_event";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedactionLevel {
    MetadataOnly,
    Redacted,
    Full,
}

impl RedactionLevel {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MetadataOnly => "metadata_only",
            Self::Redacted => "redacted",
            Self::Full => "full",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlEnvelope<TPayload> {
    pub v: u8,
    #[serde(rename = "type")]
    pub envelope_type: String,
    pub ts_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_seq: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_mode: Option<RedactionLevel>,
    pub payload: TPayload,
}

pub type JsonEnvelope = ControlEnvelope<Value>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandTypeV1 {
    #[serde(rename = "cmd.list_tabs")]
    ListTabs,
    #[serde(rename = "cmd.start_capture")]
    StartCapture,
    #[serde(rename = "cmd.stop_capture")]
    StopCapture,
    #[serde(rename = "cmd.set_ui_capture")]
    SetUiCapture,
    #[serde(rename = "cmd.pairing_discover")]
    PairingDiscover,
    #[serde(rename = "cmd.pairing_approve")]
    PairingApprove,
    #[serde(rename = "cmd.pairing_revoke")]
    PairingRevoke,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventTypeV1 {
    #[serde(rename = "evt.hello")]
    Hello,
    #[serde(rename = "evt.tabs_list")]
    TabsList,
    #[serde(rename = "evt.session_started")]
    SessionStarted,
    #[serde(rename = "evt.raw_event")]
    RawEvent,
    #[serde(rename = "evt.session_ended")]
    SessionEnded,
    #[serde(rename = "evt.error")]
    Error,
    #[serde(rename = "evt.pairing_discovered")]
    PairingDiscovered,
    #[serde(rename = "evt.pairing_approval_needed")]
    PairingApprovalNeeded,
    #[serde(rename = "evt.pairing_established")]
    PairingEstablished,
    #[serde(rename = "evt.pairing_revoked")]
    PairingRevoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CmdListTabsPayload {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CmdStartCapturePayload {
    pub tab_id: i64,
    pub privacy_mode: RedactionLevel,
    pub session_id: String,
    #[serde(default)]
    pub enable_security_domain: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CmdStopCapturePayload {
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CmdSetUiCapturePayload {
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairingUxStateV1 {
    NotPaired,
    Discovering,
    AwaitingApproval,
    Paired,
    Reconnecting,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustedDeviceRecordV1 {
    pub device_id: String,
    pub browser_label: String,
    pub first_paired_at_ms: i64,
    pub last_seen_at_ms: i64,
    pub revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CmdPairingDiscoverPayload {
    pub device_id: String,
    pub browser_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CmdPairingApprovePayload {
    pub device_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CmdPairingRevokePayload {
    pub device_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvtPairingDiscoveredPayload {
    pub state: PairingUxStateV1,
    pub device_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvtPairingApprovalNeededPayload {
    pub state: PairingUxStateV1,
    pub device_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvtPairingEstablishedPayload {
    pub state: PairingUxStateV1,
    pub device_id: String,
    pub trusted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvtPairingRevokedPayload {
    pub state: PairingUxStateV1,
    pub device_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabDescriptorV1 {
    pub tab_id: i64,
    pub window_id: i64,
    pub url: String,
    pub title: String,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvtHelloPayload {
    pub extension_version: String,
    pub protocol_version: u8,
    pub connected: bool,
    pub consent_enabled: bool,
    pub ui_capture_enabled: bool,
    pub active_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pairing_state: Option<PairingUxStateV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted_device_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvtTabsListPayload {
    pub tabs: Vec<TabDescriptorV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvtSessionStartedPayload {
    pub session_id: String,
    pub tab_id: i64,
    pub privacy_mode: RedactionLevel,
    pub started_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvtSessionEndedPayload {
    pub session_id: String,
    pub ended_at_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventErrorCodeV1 {
    AlreadyAttached,
    PermissionDenied,
    PairingNotEstablished,
    TokenInvalid,
    WsDisconnected,
    UnsupportedCommand,
    InternalError,
    DeleteBlockedRunningSession,
    DeleteArtifactPathBlocked,
    DeleteArtifactIoError,
    RetentionPolicyInvalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvtErrorPayload {
    pub code: EventErrorCodeV1,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawEventPayload {
    pub event_id: String,
    pub cdp_method: String,
    pub raw_event: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    RawEvent,
    NetRow,
    Console,
    DerivedMetric,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbsenceEvidence {
    pub reason: String,
    pub container_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawEventEvidenceTarget {
    pub event_id: String,
    pub cdp_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_pointer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub absence: Option<AbsenceEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetRowEvidenceTarget {
    pub net_request_id: String,
    pub table: NetTable,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_pointer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub absence: Option<AbsenceEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetTable {
    NetworkRequests,
    NetworkResponses,
    NetworkCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsoleEvidenceTarget {
    pub console_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_pointer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DerivedMetricEvidenceInput {
    pub kind: EvidenceKind,
    pub label: String,
    pub ts_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DerivedMetricEvidenceTarget {
    pub metric_name: String,
    pub value: f64,
    pub unit: String,
    pub inputs: Vec<DerivedMetricEvidenceInput>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EvidenceTarget {
    RawEvent(RawEventEvidenceTarget),
    NetRow(NetRowEvidenceTarget),
    Console(ConsoleEvidenceTarget),
    DerivedMetric(DerivedMetricEvidenceTarget),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRefV1 {
    pub v: u8,
    pub kind: EvidenceKind,
    pub session_id: String,
    pub label: String,
    pub ts_ms: i64,
    pub redaction_level: RedactionLevel,
    pub target: EvidenceTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HeaderValue {
    Single(String),
    Multi(Vec<String>),
}

pub type HeaderMap = BTreeMap<String, HeaderValue>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimingJsonV1 {
    pub request_time_s: f64,
    pub dns_start_ms: Option<f64>,
    pub dns_end_ms: Option<f64>,
    pub connect_start_ms: Option<f64>,
    pub connect_end_ms: Option<f64>,
    pub ssl_start_ms: Option<f64>,
    pub ssl_end_ms: Option<f64>,
    pub send_start_ms: Option<f64>,
    pub send_end_ms: Option<f64>,
    pub receive_headers_end_ms: Option<f64>,
    pub worker_start_ms: Option<f64>,
    pub worker_ready_ms: Option<f64>,
    pub worker_fetch_start_ms: Option<f64>,
    pub worker_respond_with_settled_ms: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamTransport {
    Sse,
    Websocket,
    ChunkedFetch,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconstructionStatus {
    Ok,
    Partial,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamReconstructionV1 {
    pub status: ReconstructionStatus,
    pub parse_errors: u32,
    pub dropped_chunks: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamSummaryV1 {
    pub is_streaming: bool,
    pub transport: StreamTransport,
    pub content_type: Option<String>,
    pub chunk_count: u32,
    pub bytes_total: u64,
    pub first_byte_ms: Option<i64>,
    pub last_byte_ms: Option<i64>,
    pub stream_duration_ms: Option<i64>,
    pub reconstruction: StreamReconstructionV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedNetworkRequestRecord {
    pub net_request_id: String,
    pub session_id: String,
    pub event_seq: i64,
    pub ts_ms: i64,
    pub started_at_ms: i64,
    pub method: String,
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query: Option<String>,
    pub request_headers_json: HeaderMap,
    pub timing_json: TimingJsonV1,
    pub redaction_level: RedactionLevel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedNetworkResponseRecord {
    pub net_request_id: String,
    pub session_id: String,
    pub ts_ms: i64,
    pub status_code: u16,
    pub protocol: Option<String>,
    pub mime_type: Option<String>,
    pub encoded_data_length: Option<u64>,
    pub response_headers_json: HeaderMap,
    pub headers_hash: String,
    pub stream_summary_json: Option<StreamSummaryV1>,
    pub redaction_level: RedactionLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedNetworkCompletionRecord {
    pub net_request_id: String,
    pub session_id: String,
    pub ts_ms: i64,
    pub finished_at_ms: i64,
    pub duration_ms: i64,
    pub success: bool,
    pub error_text: Option<String>,
    pub canceled: bool,
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionKindV1 {
    PageLoad,
    ApiBurst,
    LlmMessage,
    LlmRegen,
    Upload,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionMemberTypeV1 {
    NetworkRequest,
    NetworkResponse,
    NetworkCompletion,
    ConsoleEntry,
    PageLifecycle,
    RawEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrelationConstantsV1 {
    pub burst_gap_ms: i64,
    pub burst_max_window_ms: i64,
    pub pageload_soft_timeout_ms: i64,
    pub pageload_hard_timeout_ms: i64,
    pub stream_end_grace_ms: i64,
    pub interaction_close_idle_ms: i64,
    pub preflight_followup_window_ms: i64,
}

impl Default for CorrelationConstantsV1 {
    fn default() -> Self {
        Self {
            burst_gap_ms: 900,
            burst_max_window_ms: 20_000,
            pageload_soft_timeout_ms: 25_000,
            pageload_hard_timeout_ms: 60_000,
            stream_end_grace_ms: 2_000,
            interaction_close_idle_ms: 2_500,
            preflight_followup_window_ms: 2_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedInteractionRecordV1 {
    pub interaction_id: String,
    pub session_id: String,
    pub interaction_kind: InteractionKindV1,
    pub opened_at_ms: i64,
    pub closed_at_ms: Option<i64>,
    pub primary_member_id: Option<String>,
    pub rank: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedInteractionMemberRecordV1 {
    pub interaction_id: String,
    pub member_type: InteractionMemberTypeV1,
    pub member_id: String,
    pub member_rank: u32,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimTruth {
    Verified,
    Inferred,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixStepRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixStepV1 {
    pub step_id: String,
    pub title: String,
    pub body_md: String,
    pub risk: FixStepRisk,
    pub applies_when: Vec<String>,
    pub actions: Vec<String>,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaimV1 {
    pub claim_id: String,
    pub finding_id: String,
    pub rank: u32,
    pub truth: ClaimTruth,
    pub title: String,
    pub summary: String,
    pub confidence_score: f64,
    pub evidence_refs: Vec<EvidenceRefV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindingV1 {
    pub finding_id: String,
    pub session_id: String,
    pub detector_id: String,
    pub detector_version: String,
    pub title: String,
    pub summary: String,
    pub category: String,
    pub severity_score: u8,
    pub confidence_score: f64,
    pub created_at_ms: i64,
    pub interaction_id: Option<String>,
    pub fix_steps_json: Vec<FixStepV1>,
    pub claims: Vec<ClaimV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiSessionStatusV1 {
    Running,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiSessionListItemV1 {
    pub session_id: String,
    pub privacy_mode: RedactionLevel,
    pub capture_source: String,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub duration_ms: Option<i64>,
    pub findings_count: u32,
    pub status: UiSessionStatusV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiSessionOverviewV1 {
    pub session: UiSessionListItemV1,
    pub interactions_count: u32,
    pub network_requests_count: u32,
    pub network_responses_count: u32,
    pub network_completion_count: u32,
    pub console_entries_count: u32,
    pub findings_count: u32,
    pub top_findings: Vec<UiFindingCardV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTimelineKindV1 {
    RawEvent,
    ConsoleEntry,
    PageLifecycle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiTimelineEventV1 {
    pub stable_id: String,
    pub ts_ms: i64,
    pub kind: UiTimelineKindV1,
    pub label: String,
    pub source_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiTimelineInteractionV1 {
    pub interaction_id: String,
    pub interaction_kind: InteractionKindV1,
    pub opened_at_ms: i64,
    pub closed_at_ms: Option<i64>,
    pub primary_member_id: Option<String>,
    pub members_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiTimelineBundleV1 {
    pub interactions: Vec<UiTimelineInteractionV1>,
    pub events: Vec<UiTimelineEventV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiNetworkRowV1 {
    pub net_request_id: String,
    pub started_at_ms: i64,
    pub method: Option<String>,
    pub host: Option<String>,
    pub path: Option<String>,
    pub status_code: Option<i64>,
    pub duration_ms: Option<i64>,
    pub mime_type: Option<String>,
    pub is_streaming: bool,
    pub redaction_level: RedactionLevel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiConsoleRowV1 {
    pub console_id: String,
    pub ts_ms: i64,
    pub level: Option<String>,
    pub source: Option<String>,
    pub message_redacted: Option<String>,
    pub message_len: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiClaimV1 {
    pub claim_id: String,
    pub rank: u32,
    pub truth: ClaimTruth,
    pub title: String,
    pub summary: String,
    pub confidence_score: f64,
    pub evidence_refs: Vec<EvidenceRefV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiFindingCardV1 {
    pub finding_id: String,
    pub session_id: String,
    pub detector_id: String,
    pub detector_version: String,
    pub title: String,
    pub summary: String,
    pub category: String,
    pub severity_score: u8,
    pub confidence_score: f64,
    pub created_at_ms: i64,
    pub interaction_id: Option<String>,
    pub fix_steps_json: Vec<FixStepV1>,
    pub claims: Vec<UiClaimV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiExportModeV1 {
    ShareSafe,
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiExportCapabilityV1 {
    pub session_id: String,
    pub default_mode: UiExportModeV1,
    pub full_export_allowed: bool,
    pub full_export_block_reason: Option<String>,
    pub phase8_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportProfileV1 {
    ShareSafe,
    Full,
}

impl ExportProfileV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ShareSafe => "share_safe",
            Self::Full => "full",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportRunStatusV1 {
    Queued,
    Running,
    Completed,
    Failed,
    Invalid,
}

impl ExportRunStatusV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Invalid => "invalid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestFileKindV1 {
    Normalized,
    Analysis,
    Raw,
    Blob,
    Report,
    Integrity,
    Index,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestIndexModeV1 {
    Line,
    #[serde(rename = "line+byte")]
    LineAndByte,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportManifestFileEntryV1 {
    pub path: String,
    pub kind: ManifestFileKindV1,
    pub line_count: u64,
    pub sha_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportManifestIndexEntryV1 {
    pub name: String,
    pub maps_file: String,
    pub mode: ManifestIndexModeV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportEvidenceIndexesV1 {
    pub raw_event: String,
    pub net_row: String,
    pub console: String,
    pub derived_metric: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportManifestV1 {
    pub v: u8,
    pub session_id: String,
    pub exported_at_ms: i64,
    pub privacy_mode: RedactionLevel,
    pub export_profile: ExportProfileV1,
    pub files: Vec<ExportManifestFileEntryV1>,
    pub indexes: Vec<ExportManifestIndexEntryV1>,
    pub evidence_indexes: ExportEvidenceIndexesV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportRunRecordV1 {
    pub export_id: String,
    pub session_id: String,
    pub status: ExportRunStatusV1,
    pub profile: ExportProfileV1,
    pub zip_path: Option<String>,
    pub created_at_ms: i64,
    pub completed_at_ms: Option<i64>,
    pub integrity_ok: Option<bool>,
    pub bundle_blake3: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiExportListItemV1 {
    pub export_id: String,
    pub session_id: String,
    pub profile: ExportProfileV1,
    pub status: ExportRunStatusV1,
    pub zip_path: Option<String>,
    pub created_at_ms: i64,
    pub completed_at_ms: Option<i64>,
    pub integrity_ok: Option<bool>,
    pub bundle_blake3: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStartExportRequestV1 {
    pub session_id: String,
    pub profile: ExportProfileV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_dir: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStartExportResultV1 {
    pub export_id: String,
    pub status: ExportRunStatusV1,
    pub zip_path: Option<String>,
    pub integrity_ok: Option<bool>,
    pub bundle_blake3: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiValidateExportResultV1 {
    pub export_id: String,
    pub valid: bool,
    pub bundle_hash_matches: bool,
    pub mismatched_files: Vec<String>,
    pub missing_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiOpenExportFolderResultV1 {
    pub supported: bool,
    pub opened: bool,
    pub path: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseChannelV1 {
    InternalBeta,
    StagedPublicPrerelease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionChannelV1 {
    ChromeStorePublic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RolloutStageV1 {
    Pct5,
    Pct25,
    Pct50,
    Pct100,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RolloutStatusV1 {
    Planned,
    Active,
    Promoted,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateChannelV1 {
    InternalBeta,
    StagedPublicPrerelease,
    PublicStable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateEligibilityV1 {
    Eligible,
    DeferredRollout,
    BlockedSignature,
    BlockedPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseVisibilityV1 {
    Internal,
    StagedPublic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SigningStatusV1 {
    NotApplicable,
    Pending,
    Verified,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactProvenanceV1 {
    pub build_id: String,
    pub workflow_run_id: String,
    pub source_commit: String,
    pub signing_status: SigningStatusV1,
    pub notarization_status: SigningStatusV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopArtifactKindV1 {
    MacAppBundle,
    MacDmg,
    MacZip,
    Checksums,
    ReleaseManifest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionArtifactKindV1 {
    ExtensionZip,
    Checksums,
    ReleaseManifest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseArtifactKindV1 {
    MacAppBundle,
    MacDmg,
    MacZip,
    WindowsMsi,
    WindowsZip,
    LinuxAppImage,
    LinuxDeb,
    LinuxTarGz,
    ExtensionZip,
    Checksums,
    ReleaseManifest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleasePlatformV1 {
    Macos,
    Windows,
    Linux,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseArchV1 {
    X64,
    Arm64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseArtifactV1 {
    pub kind: ReleaseArtifactKindV1,
    #[serde(default = "default_release_platform")]
    pub platform: ReleasePlatformV1,
    #[serde(default = "default_release_arch")]
    pub arch: ReleaseArchV1,
    #[serde(default = "default_release_target_triple")]
    pub target_triple: String,
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

fn default_release_platform() -> ReleasePlatformV1 {
    ReleasePlatformV1::Macos
}

fn default_release_arch() -> ReleaseArchV1 {
    ReleaseArchV1::X64
}

fn default_release_target_triple() -> String {
    "x86_64-apple-darwin".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseRunStatusV1 {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseRunRecordV1 {
    pub run_id: String,
    pub channel: ReleaseChannelV1,
    pub version: String,
    pub commit_sha: String,
    pub status: ReleaseRunStatusV1,
    pub artifacts: Vec<ReleaseArtifactV1>,
    pub started_at_ms: i64,
    pub completed_at_ms: Option<i64>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStartReleaseRequestV1 {
    pub channel: ReleaseChannelV1,
    pub version: String,
    pub notes_md: String,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStartReleaseResultV1 {
    pub run_id: String,
    pub status: ReleaseRunStatusV1,
    pub artifacts: Vec<ReleaseArtifactV1>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiReleaseListItemV1 {
    pub run_id: String,
    pub channel: ReleaseChannelV1,
    pub version: String,
    pub commit_sha: String,
    pub status: ReleaseRunStatusV1,
    pub started_at_ms: i64,
    pub completed_at_ms: Option<i64>,
    pub artifacts: Vec<ReleaseArtifactV1>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStartExtensionPublicRolloutRequestV1 {
    pub channel: ExtensionChannelV1,
    pub version: String,
    pub stage: RolloutStageV1,
    pub notes_md: String,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStartExtensionPublicRolloutResultV1 {
    pub rollout_id: String,
    pub channel: ExtensionChannelV1,
    pub version: String,
    pub stage: RolloutStageV1,
    pub status: RolloutStatusV1,
    pub cws_item_id: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiListExtensionRolloutsItemV1 {
    pub rollout_id: String,
    pub channel: ExtensionChannelV1,
    pub version: String,
    pub stage: RolloutStageV1,
    pub status: RolloutStatusV1,
    pub cws_item_id: Option<String>,
    pub started_at_ms: i64,
    pub completed_at_ms: Option<i64>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiExtensionComplianceSnapshotV1 {
    pub rollout_id: Option<String>,
    pub checks_total: u32,
    pub checks_passed: u32,
    pub checks_failed: u32,
    pub checks_warn: u32,
    pub checks: Vec<Value>,
    pub blocking_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiCheckForUpdateRequestV1 {
    pub channel: UpdateChannelV1,
    pub install_id: String,
    pub current_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiCheckForUpdateResultV1 {
    pub channel: UpdateChannelV1,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub eligibility: UpdateEligibilityV1,
    pub stage: Option<RolloutStageV1>,
    pub rollout_pct: Option<u8>,
    pub signature_verified: bool,
    pub update_rollout_id: Option<String>,
    pub artifact: Option<ReleaseArtifactV1>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiApplyUpdateResultV1 {
    pub update_rollout_id: String,
    pub applied: bool,
    pub eligibility: UpdateEligibilityV1,
    pub signature_verified: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiUpdateRolloutSnapshotV1 {
    pub update_rollout_id: Option<String>,
    pub channel: UpdateChannelV1,
    pub version: Option<String>,
    pub stage: Option<RolloutStageV1>,
    pub rollout_pct: Option<u8>,
    pub status: Option<RolloutStatusV1>,
    pub feed_url: Option<String>,
    pub signature_verified: bool,
    pub started_at_ms: Option<i64>,
    pub completed_at_ms: Option<i64>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBundleInspectOpenRequestV1 {
    pub bundle_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiBundleInspectOpenResultV1 {
    pub inspect_id: String,
    pub bundle_path: String,
    pub integrity_valid: bool,
    pub session_id: Option<String>,
    pub exported_at_ms: Option<i64>,
    pub privacy_mode: Option<RedactionLevel>,
    pub profile: Option<ExportProfileV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiBundleInspectOverviewV1 {
    pub inspect_id: String,
    pub bundle_path: String,
    pub integrity_valid: bool,
    pub session_id: Option<String>,
    pub exported_at_ms: Option<i64>,
    pub privacy_mode: Option<RedactionLevel>,
    pub profile: Option<ExportProfileV1>,
    pub findings_count: u32,
    pub evidence_refs_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiBundleInspectFindingV1 {
    pub finding_id: String,
    pub detector_id: String,
    pub title: String,
    pub summary: String,
    pub category: String,
    pub severity_score: u8,
    pub confidence_score: f64,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiBundleInspectEvidenceResolveResultV1 {
    pub inspect_id: String,
    pub evidence_ref_id: String,
    pub kind: EvidenceKind,
    pub target_id: String,
    pub exact_pointer_found: bool,
    pub fallback_reason: Option<String>,
    pub container_json: Option<Value>,
    pub highlighted_value: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReliabilityMetricKeyV1 {
    WsDisconnectCount,
    WsReconnectCount,
    CaptureDropCount,
    CaptureLimitCount,
    CommandTimeoutCount,
    SessionPipelineFailCount,
    PermissionDeniedCount,
    AlreadyAttachedCount,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReliabilityMetricSampleV1 {
    pub metric_id: String,
    pub session_id: Option<String>,
    pub source: String,
    pub metric_key: ReliabilityMetricKeyV1,
    pub metric_value: f64,
    pub labels_json: Value,
    pub ts_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReliabilityWindowSummaryV1 {
    pub window_ms: i64,
    pub from_ms: i64,
    pub to_ms: i64,
    pub totals_by_key: std::collections::BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerfRunStatusV1 {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerfRunRecordV1 {
    pub perf_run_id: String,
    pub run_kind: String,
    pub status: PerfRunStatusV1,
    pub input_ref: String,
    pub summary_json: Value,
    pub started_at_ms: i64,
    pub completed_at_ms: Option<i64>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub run_duration_target_ms: i64,
    pub actual_duration_ms: Option<i64>,
    pub budget_result: Option<PerfBudgetResultV1>,
    pub trend_delta_pct: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerfBudgetResultV1 {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiReliabilitySeriesPointV1 {
    pub metric_key: ReliabilityMetricKeyV1,
    pub bucket_start_ms: i64,
    pub metric_value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiReliabilitySnapshotV1 {
    pub window: ReliabilityWindowSummaryV1,
    pub recent_samples: Vec<ReliabilityMetricSampleV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiPerfRunListItemV1 {
    pub perf_run_id: String,
    pub run_kind: String,
    pub status: PerfRunStatusV1,
    pub input_ref: String,
    pub started_at_ms: i64,
    pub completed_at_ms: Option<i64>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiStartPerfRunRequestV1 {
    pub run_kind: String,
    pub input_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiStartPerfRunResultV1 {
    pub perf_run_id: String,
    pub status: PerfRunStatusV1,
    pub summary_json: Value,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiPerfTrendPointV1 {
    pub run_kind: String,
    pub bucket_start_ms: i64,
    pub metric_name: String,
    pub metric_value: f64,
    pub baseline_value: f64,
    pub trend_delta_pct: f64,
    pub budget_result: PerfBudgetResultV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryModeV1 {
    LocalOnly,
    LocalPlusOtlp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryAuditStatusV1 {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtlpSinkConfigV1 {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub protocol: String,
    pub timeout_ms: i64,
    pub batch_size: u32,
    pub redaction_profile: String,
}

impl Default for OtlpSinkConfigV1 {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: None,
            protocol: "http".to_string(),
            timeout_ms: 5_000,
            batch_size: 250,
            redaction_profile: "counters_only".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryExportRunV1 {
    pub export_run_id: String,
    pub status: PerfRunStatusV1,
    pub from_ms: i64,
    pub to_ms: i64,
    pub sample_count: u32,
    pub redacted_count: u32,
    pub payload_sha256: Option<String>,
    pub created_at_ms: i64,
    pub completed_at_ms: Option<i64>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiReleasePromotionRequestV1 {
    pub channel: ReleaseChannelV1,
    pub promote_from_internal_run_id: String,
    pub notes_md: String,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiReleasePromotionResultV1 {
    pub promotion_id: String,
    pub channel: ReleaseChannelV1,
    pub visibility: ReleaseVisibilityV1,
    pub status: ReleaseRunStatusV1,
    pub provenance: ArtifactProvenanceV1,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiSigningSnapshotV1 {
    pub run_id: String,
    pub channel: ReleaseChannelV1,
    pub visibility: ReleaseVisibilityV1,
    pub artifact_count: u32,
    pub signing_status: SigningStatusV1,
    pub notarization_status: SigningStatusV1,
    pub manual_smoke_ready: bool,
    pub blocking_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTelemetrySettingsV1 {
    pub mode: TelemetryModeV1,
    pub otlp: OtlpSinkConfigV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiTelemetryExportResultV1 {
    pub run: TelemetryExportRunV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryAuditRunV1 {
    pub audit_id: String,
    pub export_run_id: Option<String>,
    pub status: TelemetryAuditStatusV1,
    pub violations_count: u32,
    pub violations_json: Value,
    pub payload_sha256: Option<String>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiRunTelemetryAuditResultV1 {
    pub run: TelemetryAuditRunV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerfAnomalySeverityV1 {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerfAnomalyRecordV1 {
    pub anomaly_id: String,
    pub run_kind: String,
    pub bucket_start_ms: i64,
    pub metric_name: String,
    pub severity: PerfAnomalySeverityV1,
    pub score: f64,
    pub baseline_value: f64,
    pub observed_value: f64,
    pub details_json: Value,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiListPerfAnomaliesItemV1 {
    pub anomaly_id: String,
    pub run_kind: String,
    pub bucket_start_ms: i64,
    pub metric_name: String,
    pub severity: PerfAnomalySeverityV1,
    pub score: f64,
    pub baseline_value: f64,
    pub observed_value: f64,
    pub details_json: Value,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RolloutHealthStatusV1 {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RolloutGateReasonV1 {
    ManualSmokeMissing,
    ComplianceFailed,
    TelemetryAuditFailed,
    AnomalyBudgetFailed,
    IncidentBudgetFailed,
    SignatureInvalid,
    SoakIncomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RolloutControllerActionV1 {
    Advance,
    Pause,
    Block,
    Noop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseHealthMetricV1 {
    pub metric_key: String,
    pub status: RolloutHealthStatusV1,
    pub observed_value: f64,
    pub threshold_warn: Option<f64>,
    pub threshold_fail: Option<f64>,
    pub details_json: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseHealthScorecardV1 {
    pub scope: String,
    pub channel: String,
    pub version: String,
    pub stage: Option<RolloutStageV1>,
    pub overall_status: RolloutHealthStatusV1,
    pub score: f64,
    pub metrics: Vec<ReleaseHealthMetricV1>,
    pub gate_reasons: Vec<RolloutGateReasonV1>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseHealthSnapshotV1 {
    pub snapshot_id: String,
    pub scorecard: ReleaseHealthScorecardV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiEvaluateExtensionRolloutStageRequestV1 {
    pub version: String,
    pub stage: RolloutStageV1,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiEvaluateExtensionRolloutStageResultV1 {
    pub action: RolloutControllerActionV1,
    pub status: RolloutHealthStatusV1,
    pub scorecard: ReleaseHealthScorecardV1,
    pub soak_remaining_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAdvanceExtensionRolloutStageRequestV1 {
    pub version: String,
    pub from_stage: RolloutStageV1,
    pub to_stage: RolloutStageV1,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiAdvanceExtensionRolloutStageResultV1 {
    pub rollout_id: Option<String>,
    pub action: RolloutControllerActionV1,
    pub status: RolloutStatusV1,
    pub from_stage: RolloutStageV1,
    pub to_stage: RolloutStageV1,
    pub gate_reasons: Vec<RolloutGateReasonV1>,
    pub scorecard: ReleaseHealthScorecardV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiEvaluateUpdateRolloutRequestV1 {
    pub channel: UpdateChannelV1,
    pub version: String,
    pub stage: RolloutStageV1,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiEvaluateUpdateRolloutResultV1 {
    pub action: RolloutControllerActionV1,
    pub status: RolloutHealthStatusV1,
    pub scorecard: ReleaseHealthScorecardV1,
    pub soak_remaining_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAdvanceUpdateRolloutRequestV1 {
    pub channel: UpdateChannelV1,
    pub version: String,
    pub from_stage: RolloutStageV1,
    pub to_stage: RolloutStageV1,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiAdvanceUpdateRolloutResultV1 {
    pub update_rollout_id: Option<String>,
    pub action: RolloutControllerActionV1,
    pub status: RolloutStatusV1,
    pub channel: UpdateChannelV1,
    pub from_stage: RolloutStageV1,
    pub to_stage: RolloutStageV1,
    pub gate_reasons: Vec<RolloutGateReasonV1>,
    pub scorecard: ReleaseHealthScorecardV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplianceEvidenceItemV1 {
    pub item_key: String,
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplianceEvidencePackV1 {
    pub pack_id: String,
    pub kind: String,
    pub channel: String,
    pub version: String,
    pub stage: Option<RolloutStageV1>,
    pub pack_path: String,
    pub manifest_sha256: String,
    pub items: Vec<ComplianceEvidenceItemV1>,
    pub created_at_ms: i64,
    pub status: String,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiGetComplianceEvidencePackResultV1 {
    pub pack: Option<ComplianceEvidencePackV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiListComplianceEvidencePacksItemV1 {
    pub pack_id: String,
    pub kind: String,
    pub channel: String,
    pub version: String,
    pub stage: Option<RolloutStageV1>,
    pub status: String,
    pub created_at_ms: i64,
    pub pack_path: String,
    pub manifest_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionPolicyV1 {
    pub enabled: bool,
    pub retain_days: u32,
    pub max_sessions: u32,
    pub delete_exports: bool,
    pub delete_blobs: bool,
}

impl Default for RetentionPolicyV1 {
    fn default() -> Self {
        Self {
            enabled: true,
            retain_days: 30,
            max_sessions: 1000,
            delete_exports: true,
            delete_blobs: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionRunModeV1 {
    DryRun,
    Apply,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionDeleteResultV1 {
    pub session_id: String,
    pub db_deleted: bool,
    pub files_deleted: u32,
    pub missing_files: Vec<String>,
    pub blocked_paths: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionRunReportV1 {
    pub run_id: String,
    pub mode: RetentionRunModeV1,
    pub evaluated_sessions: u32,
    pub candidate_sessions: u32,
    pub deleted_sessions: u32,
    pub skipped_running_sessions: u32,
    pub failed_sessions: u32,
    pub started_at_ms: i64,
    pub finished_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRetentionSettingsV1 {
    pub policy: RetentionPolicyV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRetentionRunResultV1 {
    pub report: RetentionRunReportV1,
    pub deleted: Vec<SessionDeleteResultV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDeleteSessionResultV1 {
    pub result: SessionDeleteResultV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportBlobDescriptorV1 {
    pub blob_id: String,
    pub media_type: Option<String>,
    pub len_bytes: i64,
    pub blake3_hash: String,
    pub storage_kind: String,
    pub storage_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportDatasetV1 {
    pub session_id: String,
    pub privacy_mode: RedactionLevel,
    pub export_profile: ExportProfileV1,
    pub exported_at_ms: i64,
    pub session_json: Value,
    pub normalized_network_requests: Vec<Value>,
    pub normalized_network_responses: Vec<Value>,
    pub normalized_network_completion: Vec<Value>,
    pub normalized_console_entries: Vec<Value>,
    pub normalized_page_lifecycle: Vec<Value>,
    pub normalized_interactions: Vec<Value>,
    pub normalized_interaction_members: Vec<Value>,
    pub analysis_findings: Vec<Value>,
    pub analysis_claims: Vec<Value>,
    pub analysis_evidence_refs: Vec<Value>,
    pub analysis_derived_metrics: Vec<Value>,
    pub raw_events: Vec<Value>,
    pub blobs: Vec<ExportBlobDescriptorV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiConnectionStatusV1 {
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDiagnosticEntryV1 {
    pub ts_ms: i64,
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDiagnosticsSnapshotV1 {
    pub pairing_port: Option<u16>,
    pub pairing_token: Option<String>,
    pub connection_status: UiConnectionStatusV1,
    pub diagnostics: Vec<UiDiagnosticEntryV1>,
    pub capture_drop_markers: u64,
    pub capture_limit_markers: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPairingStateV1 {
    pub state: PairingUxStateV1,
    pub pairing_port: Option<u16>,
    pub trusted_device_id: Option<String>,
    pub connected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiLaunchDesktopResultV1 {
    pub launched: bool,
    pub method: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiEvidenceResolveRequestV1 {
    pub evidence_ref_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiEvidenceResolveResultV1 {
    pub evidence_ref_id: String,
    pub session_id: String,
    pub kind: EvidenceKind,
    pub route_subview: String,
    pub target_id: String,
    pub column: Option<String>,
    pub json_pointer: Option<String>,
    pub exact_pointer_found: bool,
    pub fallback_reason: Option<String>,
    pub container_json: Option<Value>,
    pub highlighted_value: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::{
        ClaimTruth, CommandTypeV1, ControlEnvelope, CorrelationConstantsV1, EventErrorCodeV1,
        EventTypeV1, EvidenceKind, EvidenceRefV1, EvidenceTarget, InteractionKindV1,
        InteractionMemberTypeV1, NetRowEvidenceTarget, NetTable, RawEventPayload, RedactionLevel,
        ENVELOPE_VERSION, EVT_RAW_EVENT,
    };
    use serde_json::json;

    #[test]
    fn evidence_ref_round_trip_json() {
        let ref_value = EvidenceRefV1 {
            v: 1,
            kind: EvidenceKind::NetRow,
            session_id: "sess_01".to_string(),
            label: "HTTP 429 response".to_string(),
            ts_ms: 1_729_000_123_456,
            redaction_level: RedactionLevel::MetadataOnly,
            target: EvidenceTarget::NetRow(NetRowEvidenceTarget {
                net_request_id: "net_01".to_string(),
                table: NetTable::NetworkResponses,
                column: Some("status_code".to_string()),
                json_pointer: None,
                absence: None,
            }),
            preview: None,
            integrity: None,
        };

        let encoded = serde_json::to_string(&ref_value).expect("encode evidence");
        let decoded: EvidenceRefV1 = serde_json::from_str(&encoded).expect("decode evidence");

        assert_eq!(decoded, ref_value);
    }

    #[test]
    fn claim_truth_serializes_as_snake_case() {
        let encoded = serde_json::to_string(&ClaimTruth::Verified).expect("serialize truth");
        assert_eq!(encoded, "\"verified\"");
    }

    #[test]
    fn control_envelope_round_trips_with_type_and_ids() {
        let envelope = ControlEnvelope {
            v: ENVELOPE_VERSION,
            envelope_type: EVT_RAW_EVENT.to_string(),
            ts_ms: 1_729_000_000_000,
            token: Some("token_123".to_string()),
            request_id: Some("req_1".to_string()),
            correlation_id: Some("corr_1".to_string()),
            session_id: Some("sess_1".to_string()),
            event_seq: Some(1),
            privacy_mode: Some(RedactionLevel::MetadataOnly),
            payload: RawEventPayload {
                event_id: "evt_1".to_string(),
                cdp_method: "Network.requestWillBeSent".to_string(),
                raw_event: json!({"method": "Network.requestWillBeSent"}),
            },
        };

        let encoded = serde_json::to_string(&envelope).expect("encode envelope");
        let decoded: ControlEnvelope<RawEventPayload> =
            serde_json::from_str(&encoded).expect("decode envelope");

        assert_eq!(decoded, envelope);
    }

    #[test]
    fn command_and_event_types_use_expected_wire_names() {
        let command = serde_json::to_string(&CommandTypeV1::StartCapture).expect("command json");
        let event = serde_json::to_string(&EventTypeV1::SessionStarted).expect("event json");
        let error_code =
            serde_json::to_string(&EventErrorCodeV1::AlreadyAttached).expect("error json");

        assert_eq!(command, "\"cmd.start_capture\"");
        assert_eq!(event, "\"evt.session_started\"");
        assert_eq!(error_code, "\"already_attached\"");
    }

    #[test]
    fn correlation_kinds_and_members_use_snake_case() {
        let kind_json = serde_json::to_string(&InteractionKindV1::PageLoad).expect("kind json");
        let member_json =
            serde_json::to_string(&InteractionMemberTypeV1::NetworkRequest).expect("member json");

        assert_eq!(kind_json, "\"page_load\"");
        assert_eq!(member_json, "\"network_request\"");
    }

    #[test]
    fn correlation_constants_match_spec_defaults() {
        let constants = CorrelationConstantsV1::default();
        assert_eq!(constants.burst_gap_ms, 900);
        assert_eq!(constants.pageload_soft_timeout_ms, 25_000);
        assert_eq!(constants.pageload_hard_timeout_ms, 60_000);
        assert_eq!(constants.interaction_close_idle_ms, 2_500);
        assert_eq!(constants.preflight_followup_window_ms, 2_000);
    }
}
