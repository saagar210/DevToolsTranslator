export type RedactionLevel = 'metadata_only' | 'redacted' | 'full';

export const ENVELOPE_VERSION = 1 as const;
export type EnvelopeVersion = typeof ENVELOPE_VERSION;

export type CommandEnvelopeType =
  | 'cmd.list_tabs'
  | 'cmd.start_capture'
  | 'cmd.stop_capture'
  | 'cmd.set_ui_capture'
  | 'cmd.pairing_discover'
  | 'cmd.pairing_approve'
  | 'cmd.pairing_revoke';

export type EventEnvelopeType =
  | 'evt.hello'
  | 'evt.tabs_list'
  | 'evt.session_started'
  | 'evt.raw_event'
  | 'evt.session_ended'
  | 'evt.error'
  | 'evt.pairing_discovered'
  | 'evt.pairing_approval_needed'
  | 'evt.pairing_established'
  | 'evt.pairing_revoked';

export type EnvelopeType = CommandEnvelopeType | EventEnvelopeType;

export interface ControlEnvelopeV1<TPayload = Record<string, unknown>> {
  readonly v: EnvelopeVersion;
  readonly type: EnvelopeType;
  readonly ts_ms: number;
  readonly token?: string;
  readonly request_id?: string;
  readonly correlation_id?: string;
  readonly session_id?: string;
  readonly event_seq?: number;
  readonly privacy_mode?: RedactionLevel;
  readonly payload: TPayload;
}

export interface CmdListTabsPayload {}

export interface CmdStartCapturePayload {
  readonly tab_id: number;
  readonly privacy_mode: RedactionLevel;
  readonly session_id: string;
  readonly enable_security_domain: boolean;
}

export interface CmdStopCapturePayload {
  readonly session_id: string;
}

export interface CmdSetUiCapturePayload {
  readonly enabled: boolean;
}

export type PairingUxStateV1 =
  | 'not_paired'
  | 'discovering'
  | 'awaiting_approval'
  | 'paired'
  | 'reconnecting'
  | 'error';

export interface TrustedDeviceRecordV1 {
  readonly device_id: string;
  readonly browser_label: string;
  readonly first_paired_at_ms: number;
  readonly last_seen_at_ms: number;
  readonly revoked: boolean;
}

export interface CmdPairingDiscoverPayload {
  readonly device_id: string;
  readonly browser_label: string;
}

export interface CmdPairingApprovePayload {
  readonly device_id: string;
}

export interface CmdPairingRevokePayload {
  readonly device_id: string;
}

export interface EvtPairingDiscoveredPayload {
  readonly state: PairingUxStateV1;
  readonly device_id: string;
}

export interface EvtPairingApprovalNeededPayload {
  readonly state: PairingUxStateV1;
  readonly device_id: string;
}

export interface EvtPairingEstablishedPayload {
  readonly state: PairingUxStateV1;
  readonly device_id: string;
  readonly trusted: boolean;
}

export interface EvtPairingRevokedPayload {
  readonly state: PairingUxStateV1;
  readonly device_id: string;
}

export interface TabDescriptorV1 {
  readonly tab_id: number;
  readonly window_id: number;
  readonly url: string;
  readonly title: string;
  readonly active: boolean;
}

export interface EvtHelloPayload {
  readonly extension_version: string;
  readonly protocol_version: number;
  readonly connected: boolean;
  readonly consent_enabled: boolean;
  readonly ui_capture_enabled: boolean;
  readonly active_session_id: string | null;
  readonly pairing_state?: PairingUxStateV1;
  readonly trusted_device_id?: string;
}

export interface EvtTabsListPayload {
  readonly tabs: ReadonlyArray<TabDescriptorV1>;
}

export interface EvtSessionStartedPayload {
  readonly session_id: string;
  readonly tab_id: number;
  readonly privacy_mode: RedactionLevel;
  readonly started_at_ms: number;
}

export interface EvtSessionEndedPayload {
  readonly session_id: string;
  readonly ended_at_ms: number;
}

export type EventErrorCodeV1 =
  | 'already_attached'
  | 'permission_denied'
  | 'pairing_not_established'
  | 'token_invalid'
  | 'ws_disconnected'
  | 'unsupported_command'
  | 'internal_error'
  | 'delete_blocked_running_session'
  | 'delete_artifact_path_blocked'
  | 'delete_artifact_io_error'
  | 'retention_policy_invalid';

export interface EvtErrorPayload {
  readonly code: EventErrorCodeV1;
  readonly message: string;
  readonly details?: string;
  readonly session_id?: string;
}

export interface RawEventEnvelopePayload {
  readonly event_id?: string;
  readonly cdp_method: string;
  readonly raw_event: Record<string, unknown>;
}

export type EvidenceKind = 'raw_event' | 'net_row' | 'console' | 'derived_metric';

export interface AbsenceEvidence {
  readonly reason: string;
  readonly container_hash: string;
}

export interface RawEventEvidenceTarget {
  readonly event_id: string;
  readonly cdp_method: string;
  readonly json_pointer?: string;
  readonly selection?: string;
  readonly absence?: AbsenceEvidence;
}

export interface NetRowEvidenceTarget {
  readonly net_request_id: string;
  readonly table: 'network_requests' | 'network_responses' | 'network_completion';
  readonly column?: string;
  readonly json_pointer?: string;
  readonly absence?: AbsenceEvidence;
}

export interface ConsoleEvidenceTarget {
  readonly console_id: string;
  readonly column?: string;
  readonly json_pointer?: string;
}

export interface DerivedMetricEvidenceInput {
  readonly kind: EvidenceKind;
  readonly label: string;
  readonly ts_ms: number;
}

export interface DerivedMetricEvidenceTarget {
  readonly metric_name: string;
  readonly value: number;
  readonly unit: string;
  readonly inputs: ReadonlyArray<DerivedMetricEvidenceInput>;
}

export type EvidenceTarget =
  | RawEventEvidenceTarget
  | NetRowEvidenceTarget
  | ConsoleEvidenceTarget
  | DerivedMetricEvidenceTarget;

export interface EvidenceRefV1 {
  readonly v: 1;
  readonly kind: EvidenceKind;
  readonly session_id: string;
  readonly label: string;
  readonly ts_ms: number;
  readonly redaction_level: RedactionLevel;
  readonly target: EvidenceTarget;
  readonly preview?: Record<string, unknown>;
  readonly integrity?: Record<string, string>;
}

export type HeaderValue = string | ReadonlyArray<string>;

export type HeaderMap = Readonly<Record<string, HeaderValue>>;

export interface TimingJsonV1 {
  readonly request_time_s: number;
  readonly dns_start_ms: number | null;
  readonly dns_end_ms: number | null;
  readonly connect_start_ms: number | null;
  readonly connect_end_ms: number | null;
  readonly ssl_start_ms: number | null;
  readonly ssl_end_ms: number | null;
  readonly send_start_ms: number | null;
  readonly send_end_ms: number | null;
  readonly receive_headers_end_ms: number | null;
  readonly worker_start_ms: number | null;
  readonly worker_ready_ms: number | null;
  readonly worker_fetch_start_ms: number | null;
  readonly worker_respond_with_settled_ms: number | null;
}

export interface StreamReconstructionV1 {
  readonly status: 'ok' | 'partial' | 'failed';
  readonly parse_errors: number;
  readonly dropped_chunks: number;
}

export interface StreamSummaryV1 {
  readonly is_streaming: boolean;
  readonly transport: 'sse' | 'websocket' | 'chunked_fetch' | 'unknown';
  readonly content_type: string | null;
  readonly chunk_count: number;
  readonly bytes_total: number;
  readonly first_byte_ms: number | null;
  readonly last_byte_ms: number | null;
  readonly stream_duration_ms: number | null;
  readonly reconstruction: StreamReconstructionV1;
}

export interface NormalizedNetworkRequestRecord {
  readonly net_request_id: string;
  readonly session_id: string;
  readonly event_seq: number;
  readonly ts_ms: number;
  readonly started_at_ms: number;
  readonly method: string;
  readonly scheme: string;
  readonly host: string;
  readonly port: number | null;
  readonly path: string;
  readonly query: string | null;
  readonly request_headers_json: HeaderMap;
  readonly timing_json: TimingJsonV1;
  readonly redaction_level: RedactionLevel;
}

export interface NormalizedNetworkResponseRecord {
  readonly net_request_id: string;
  readonly session_id: string;
  readonly ts_ms: number;
  readonly status_code: number;
  readonly protocol: string | null;
  readonly mime_type: string | null;
  readonly encoded_data_length: number | null;
  readonly response_headers_json: HeaderMap;
  readonly headers_hash: string;
  readonly stream_summary_json: StreamSummaryV1 | null;
  readonly redaction_level: RedactionLevel;
}

export interface NormalizedNetworkCompletionRecord {
  readonly net_request_id: string;
  readonly session_id: string;
  readonly ts_ms: number;
  readonly finished_at_ms: number;
  readonly duration_ms: number;
  readonly success: boolean;
  readonly error_text: string | null;
  readonly canceled: boolean;
  readonly blocked_reason: string | null;
}

export type InteractionKindV1 =
  | 'page_load'
  | 'api_burst'
  | 'llm_message'
  | 'llm_regen'
  | 'upload'
  | 'other';

export type InteractionMemberTypeV1 =
  | 'network_request'
  | 'network_response'
  | 'network_completion'
  | 'console_entry'
  | 'page_lifecycle'
  | 'raw_event';

export interface CorrelationConstantsV1 {
  readonly burst_gap_ms: number;
  readonly burst_max_window_ms: number;
  readonly pageload_soft_timeout_ms: number;
  readonly pageload_hard_timeout_ms: number;
  readonly stream_end_grace_ms: number;
  readonly interaction_close_idle_ms: number;
  readonly preflight_followup_window_ms: number;
}

export const CORRELATION_CONSTANTS_V1_DEFAULT: CorrelationConstantsV1 = {
  burst_gap_ms: 900,
  burst_max_window_ms: 20_000,
  pageload_soft_timeout_ms: 25_000,
  pageload_hard_timeout_ms: 60_000,
  stream_end_grace_ms: 2_000,
  interaction_close_idle_ms: 2_500,
  preflight_followup_window_ms: 2_000,
};

export interface NormalizedInteractionRecordV1 {
  readonly interaction_id: string;
  readonly session_id: string;
  readonly interaction_kind: InteractionKindV1;
  readonly opened_at_ms: number;
  readonly closed_at_ms: number | null;
  readonly primary_member_id: string | null;
  readonly rank: number;
}

export interface NormalizedInteractionMemberRecordV1 {
  readonly interaction_id: string;
  readonly member_type: InteractionMemberTypeV1;
  readonly member_id: string;
  readonly member_rank: number;
  readonly is_primary: boolean;
}

export type ClaimTruth = 'verified' | 'inferred' | 'unknown';

export interface FixStepV1 {
  readonly step_id: string;
  readonly title: string;
  readonly body_md: string;
  readonly risk: 'low' | 'medium' | 'high';
  readonly applies_when: ReadonlyArray<string>;
  readonly actions: ReadonlyArray<string>;
  readonly evidence_ids: ReadonlyArray<string>;
}

export interface ClaimV1 {
  readonly claim_id: string;
  readonly finding_id: string;
  readonly rank: number;
  readonly truth: ClaimTruth;
  readonly title: string;
  readonly summary: string;
  readonly confidence_score: number;
  readonly evidence_refs: ReadonlyArray<EvidenceRefV1>;
}

export interface FindingV1 {
  readonly finding_id: string;
  readonly session_id: string;
  readonly detector_id: string;
  readonly detector_version: string;
  readonly title: string;
  readonly summary: string;
  readonly category: string;
  readonly severity_score: number;
  readonly confidence_score: number;
  readonly created_at_ms: number;
  readonly interaction_id: string | null;
  readonly fix_steps_json: ReadonlyArray<FixStepV1>;
  readonly claims: ReadonlyArray<ClaimV1>;
}

export type UiSessionStatusV1 = 'running' | 'completed';

export interface UiSessionListItemV1 {
  readonly session_id: string;
  readonly privacy_mode: RedactionLevel;
  readonly capture_source: string;
  readonly started_at_ms: number;
  readonly ended_at_ms: number | null;
  readonly duration_ms: number | null;
  readonly findings_count: number;
  readonly status: UiSessionStatusV1;
}

export interface UiClaimV1 {
  readonly claim_id: string;
  readonly rank: number;
  readonly truth: ClaimTruth;
  readonly title: string;
  readonly summary: string;
  readonly confidence_score: number;
  readonly evidence_refs: ReadonlyArray<EvidenceRefV1>;
}

export interface UiFindingCardV1 {
  readonly finding_id: string;
  readonly session_id: string;
  readonly detector_id: string;
  readonly detector_version: string;
  readonly title: string;
  readonly summary: string;
  readonly category: string;
  readonly severity_score: number;
  readonly confidence_score: number;
  readonly created_at_ms: number;
  readonly interaction_id: string | null;
  readonly fix_steps_json: ReadonlyArray<FixStepV1>;
  readonly claims: ReadonlyArray<UiClaimV1>;
}

export interface UiSessionOverviewV1 {
  readonly session: UiSessionListItemV1;
  readonly interactions_count: number;
  readonly network_requests_count: number;
  readonly network_responses_count: number;
  readonly network_completion_count: number;
  readonly console_entries_count: number;
  readonly findings_count: number;
  readonly top_findings: ReadonlyArray<UiFindingCardV1>;
}

export type UiTimelineKindV1 = 'raw_event' | 'console_entry' | 'page_lifecycle';

export interface UiTimelineEventV1 {
  readonly stable_id: string;
  readonly ts_ms: number;
  readonly kind: UiTimelineKindV1;
  readonly label: string;
  readonly source_id: string;
}

export interface UiTimelineInteractionV1 {
  readonly interaction_id: string;
  readonly interaction_kind: InteractionKindV1;
  readonly opened_at_ms: number;
  readonly closed_at_ms: number | null;
  readonly primary_member_id: string | null;
  readonly members_count: number;
}

export interface UiTimelineBundleV1 {
  readonly interactions: ReadonlyArray<UiTimelineInteractionV1>;
  readonly events: ReadonlyArray<UiTimelineEventV1>;
}

export interface UiNetworkRowV1 {
  readonly net_request_id: string;
  readonly started_at_ms: number;
  readonly method: string | null;
  readonly host: string | null;
  readonly path: string | null;
  readonly status_code: number | null;
  readonly duration_ms: number | null;
  readonly mime_type: string | null;
  readonly is_streaming: boolean;
  readonly redaction_level: RedactionLevel;
}

export interface UiConsoleRowV1 {
  readonly console_id: string;
  readonly ts_ms: number;
  readonly level: string | null;
  readonly source: string | null;
  readonly message_redacted: string | null;
  readonly message_len: number | null;
}

export type UiExportModeV1 = 'share_safe' | 'full';

export interface UiExportCapabilityV1 {
  readonly session_id: string;
  readonly default_mode: UiExportModeV1;
  readonly full_export_allowed: boolean;
  readonly full_export_block_reason: string | null;
  readonly phase8_ready: boolean;
}

export type ExportProfileV1 = 'share_safe' | 'full';

export type ExportRunStatusV1 = 'queued' | 'running' | 'completed' | 'failed' | 'invalid';

export type ManifestFileKindV1 =
  | 'normalized'
  | 'analysis'
  | 'raw'
  | 'blob'
  | 'report'
  | 'integrity'
  | 'index';

export type ManifestIndexModeV1 = 'line' | 'line+byte';

export interface ExportManifestFileEntryV1 {
  readonly path: string;
  readonly kind: ManifestFileKindV1;
  readonly line_count: number;
  readonly sha_blake3: string;
}

export interface ExportManifestIndexEntryV1 {
  readonly name: string;
  readonly maps_file: string;
  readonly mode: ManifestIndexModeV1;
}

export interface ExportEvidenceIndexesV1 {
  readonly raw_event: string;
  readonly net_row: string;
  readonly console: string;
  readonly derived_metric: string;
}

export interface ExportManifestV1 {
  readonly v: number;
  readonly session_id: string;
  readonly exported_at_ms: number;
  readonly privacy_mode: RedactionLevel;
  readonly export_profile: ExportProfileV1;
  readonly files: ReadonlyArray<ExportManifestFileEntryV1>;
  readonly indexes: ReadonlyArray<ExportManifestIndexEntryV1>;
  readonly evidence_indexes: ExportEvidenceIndexesV1;
}

export interface ExportRunRecordV1 {
  readonly export_id: string;
  readonly session_id: string;
  readonly status: ExportRunStatusV1;
  readonly profile: ExportProfileV1;
  readonly zip_path: string | null;
  readonly created_at_ms: number;
  readonly completed_at_ms: number | null;
  readonly integrity_ok: boolean | null;
  readonly bundle_blake3: string | null;
  readonly error_code: string | null;
  readonly error_message: string | null;
}

export interface UiExportListItemV1 {
  readonly export_id: string;
  readonly session_id: string;
  readonly profile: ExportProfileV1;
  readonly status: ExportRunStatusV1;
  readonly zip_path: string | null;
  readonly created_at_ms: number;
  readonly completed_at_ms: number | null;
  readonly integrity_ok: boolean | null;
  readonly bundle_blake3: string | null;
  readonly error_code: string | null;
  readonly error_message: string | null;
}

export interface UiStartExportRequestV1 {
  readonly session_id: string;
  readonly profile: ExportProfileV1;
  readonly output_dir: string | null;
}

export interface UiStartExportResultV1 {
  readonly export_id: string;
  readonly status: ExportRunStatusV1;
  readonly zip_path: string | null;
  readonly integrity_ok: boolean | null;
  readonly bundle_blake3: string | null;
  readonly error_message: string | null;
}

export interface UiValidateExportResultV1 {
  readonly export_id: string;
  readonly valid: boolean;
  readonly bundle_hash_matches: boolean;
  readonly mismatched_files: ReadonlyArray<string>;
  readonly missing_paths: ReadonlyArray<string>;
}

export interface UiOpenExportFolderResultV1 {
  readonly supported: boolean;
  readonly opened: boolean;
  readonly path: string | null;
  readonly message: string | null;
}

export type ReleaseChannelV1 = 'internal_beta' | 'staged_public_prerelease';

export type ExtensionChannelV1 = 'chrome_store_public';

export type RolloutStageV1 = 'pct_5' | 'pct_25' | 'pct_50' | 'pct_100';

export type RolloutStatusV1 = 'planned' | 'active' | 'promoted' | 'paused' | 'completed' | 'failed';

export type UpdateChannelV1 = 'internal_beta' | 'staged_public_prerelease' | 'public_stable';

export type UpdateEligibilityV1 =
  | 'eligible'
  | 'deferred_rollout'
  | 'blocked_signature'
  | 'blocked_policy';

export type ReleaseVisibilityV1 = 'internal' | 'staged_public';

export type SigningStatusV1 = 'not_applicable' | 'pending' | 'verified' | 'failed';

export interface ArtifactProvenanceV1 {
  readonly build_id: string;
  readonly workflow_run_id: string;
  readonly source_commit: string;
  readonly signing_status: SigningStatusV1;
  readonly notarization_status: SigningStatusV1;
}

export type DesktopArtifactKindV1 =
  | 'mac_app_bundle'
  | 'mac_dmg'
  | 'mac_zip'
  | 'checksums'
  | 'release_manifest';

export type ExtensionArtifactKindV1 = 'extension_zip' | 'checksums' | 'release_manifest';

export type ReleaseArtifactKindV1 =
  | 'mac_app_bundle'
  | 'mac_dmg'
  | 'mac_zip'
  | 'windows_msi'
  | 'windows_zip'
  | 'linux_app_image'
  | 'linux_deb'
  | 'linux_tar_gz'
  | 'extension_zip'
  | 'checksums'
  | 'release_manifest';

export type ReleasePlatformV1 = 'macos' | 'windows' | 'linux';

export type ReleaseArchV1 = 'x64' | 'arm64';

export interface ReleaseArtifactV1 {
  readonly kind: ReleaseArtifactKindV1;
  readonly platform: ReleasePlatformV1;
  readonly arch: ReleaseArchV1;
  readonly target_triple: string;
  readonly path: string;
  readonly sha256: string;
  readonly size_bytes: number;
}

export type ReleaseRunStatusV1 = 'queued' | 'running' | 'completed' | 'failed';

export interface ReleaseRunRecordV1 {
  readonly run_id: string;
  readonly channel: ReleaseChannelV1;
  readonly version: string;
  readonly commit_sha: string;
  readonly status: ReleaseRunStatusV1;
  readonly artifacts: ReadonlyArray<ReleaseArtifactV1>;
  readonly started_at_ms: number;
  readonly completed_at_ms: number | null;
  readonly error_code: string | null;
  readonly error_message: string | null;
}

export interface UiStartReleaseRequestV1 {
  readonly channel: ReleaseChannelV1;
  readonly version: string;
  readonly notes_md: string;
  readonly dry_run: boolean;
}

export interface UiStartReleaseResultV1 {
  readonly run_id: string;
  readonly status: ReleaseRunStatusV1;
  readonly artifacts: ReadonlyArray<ReleaseArtifactV1>;
  readonly error_message: string | null;
}

export interface UiReleaseListItemV1 {
  readonly run_id: string;
  readonly channel: ReleaseChannelV1;
  readonly version: string;
  readonly commit_sha: string;
  readonly status: ReleaseRunStatusV1;
  readonly started_at_ms: number;
  readonly completed_at_ms: number | null;
  readonly artifacts: ReadonlyArray<ReleaseArtifactV1>;
  readonly error_code: string | null;
  readonly error_message: string | null;
}

export interface UiStartExtensionPublicRolloutRequestV1 {
  readonly channel: ExtensionChannelV1;
  readonly version: string;
  readonly stage: RolloutStageV1;
  readonly notes_md: string;
  readonly dry_run: boolean;
}

export interface UiStartExtensionPublicRolloutResultV1 {
  readonly rollout_id: string;
  readonly channel: ExtensionChannelV1;
  readonly version: string;
  readonly stage: RolloutStageV1;
  readonly status: RolloutStatusV1;
  readonly cws_item_id: string | null;
  readonly error_message: string | null;
}

export interface UiListExtensionRolloutsItemV1 {
  readonly rollout_id: string;
  readonly channel: ExtensionChannelV1;
  readonly version: string;
  readonly stage: RolloutStageV1;
  readonly status: RolloutStatusV1;
  readonly cws_item_id: string | null;
  readonly started_at_ms: number;
  readonly completed_at_ms: number | null;
  readonly error_code: string | null;
  readonly error_message: string | null;
}

export interface UiExtensionComplianceSnapshotV1 {
  readonly rollout_id: string | null;
  readonly checks_total: number;
  readonly checks_passed: number;
  readonly checks_failed: number;
  readonly checks_warn: number;
  readonly checks: ReadonlyArray<Record<string, unknown>>;
  readonly blocking_reasons: ReadonlyArray<string>;
}

export interface UiCheckForUpdateRequestV1 {
  readonly channel: UpdateChannelV1;
  readonly install_id: string;
  readonly current_version: string;
}

export interface UiCheckForUpdateResultV1 {
  readonly channel: UpdateChannelV1;
  readonly current_version: string;
  readonly latest_version: string | null;
  readonly eligibility: UpdateEligibilityV1;
  readonly stage: RolloutStageV1 | null;
  readonly rollout_pct: number | null;
  readonly signature_verified: boolean;
  readonly update_rollout_id: string | null;
  readonly artifact: ReleaseArtifactV1 | null;
  readonly reason: string | null;
}

export interface UiApplyUpdateResultV1 {
  readonly update_rollout_id: string;
  readonly applied: boolean;
  readonly eligibility: UpdateEligibilityV1;
  readonly signature_verified: boolean;
  readonly message: string | null;
}

export interface UiUpdateRolloutSnapshotV1 {
  readonly update_rollout_id: string | null;
  readonly channel: UpdateChannelV1;
  readonly version: string | null;
  readonly stage: RolloutStageV1 | null;
  readonly rollout_pct: number | null;
  readonly status: RolloutStatusV1 | null;
  readonly feed_url: string | null;
  readonly signature_verified: boolean;
  readonly started_at_ms: number | null;
  readonly completed_at_ms: number | null;
  readonly error_code: string | null;
  readonly error_message: string | null;
}

export interface UiBundleInspectOpenRequestV1 {
  readonly bundle_path: string;
}

export interface UiBundleInspectOpenResultV1 {
  readonly inspect_id: string;
  readonly bundle_path: string;
  readonly integrity_valid: boolean;
  readonly session_id: string | null;
  readonly exported_at_ms: number | null;
  readonly privacy_mode: RedactionLevel | null;
  readonly profile: ExportProfileV1 | null;
}

export interface UiBundleInspectOverviewV1 {
  readonly inspect_id: string;
  readonly bundle_path: string;
  readonly integrity_valid: boolean;
  readonly session_id: string | null;
  readonly exported_at_ms: number | null;
  readonly privacy_mode: RedactionLevel | null;
  readonly profile: ExportProfileV1 | null;
  readonly findings_count: number;
  readonly evidence_refs_count: number;
}

export interface UiBundleInspectFindingV1 {
  readonly finding_id: string;
  readonly detector_id: string;
  readonly title: string;
  readonly summary: string;
  readonly category: string;
  readonly severity_score: number;
  readonly confidence_score: number;
  readonly created_at_ms: number;
}

export interface UiBundleInspectEvidenceResolveResultV1 {
  readonly inspect_id: string;
  readonly evidence_ref_id: string;
  readonly kind: EvidenceKind;
  readonly target_id: string;
  readonly exact_pointer_found: boolean;
  readonly fallback_reason: string | null;
  readonly container_json: Record<string, unknown> | null;
  readonly highlighted_value: unknown | null;
}

export type ReliabilityMetricKeyV1 =
  | 'ws_disconnect_count'
  | 'ws_reconnect_count'
  | 'capture_drop_count'
  | 'capture_limit_count'
  | 'command_timeout_count'
  | 'session_pipeline_fail_count'
  | 'permission_denied_count'
  | 'already_attached_count';

export interface ReliabilityMetricSampleV1 {
  readonly metric_id: string;
  readonly session_id: string | null;
  readonly source: string;
  readonly metric_key: ReliabilityMetricKeyV1;
  readonly metric_value: number;
  readonly labels_json: Record<string, unknown>;
  readonly ts_ms: number;
}

export interface ReliabilityWindowSummaryV1 {
  readonly window_ms: number;
  readonly from_ms: number;
  readonly to_ms: number;
  readonly totals_by_key: Record<string, number>;
}

export type PerfRunStatusV1 = 'queued' | 'running' | 'completed' | 'failed';

export interface PerfRunRecordV1 {
  readonly perf_run_id: string;
  readonly run_kind: string;
  readonly status: PerfRunStatusV1;
  readonly input_ref: string;
  readonly summary_json: Record<string, unknown>;
  readonly started_at_ms: number;
  readonly completed_at_ms: number | null;
  readonly error_code: string | null;
  readonly error_message: string | null;
  readonly run_duration_target_ms: number;
  readonly actual_duration_ms: number | null;
  readonly budget_result: PerfBudgetResultV1 | null;
  readonly trend_delta_pct: number | null;
}

export type PerfBudgetResultV1 = 'pass' | 'warn' | 'fail';

export interface UiReliabilitySeriesPointV1 {
  readonly metric_key: ReliabilityMetricKeyV1;
  readonly bucket_start_ms: number;
  readonly metric_value: number;
}

export interface UiReliabilitySnapshotV1 {
  readonly window: ReliabilityWindowSummaryV1;
  readonly recent_samples: ReadonlyArray<ReliabilityMetricSampleV1>;
}

export interface UiPerfRunListItemV1 {
  readonly perf_run_id: string;
  readonly run_kind: string;
  readonly status: PerfRunStatusV1;
  readonly input_ref: string;
  readonly started_at_ms: number;
  readonly completed_at_ms: number | null;
  readonly error_code: string | null;
  readonly error_message: string | null;
}

export interface UiStartPerfRunRequestV1 {
  readonly run_kind: string;
  readonly input_ref: string;
}

export interface UiStartPerfRunResultV1 {
  readonly perf_run_id: string;
  readonly status: PerfRunStatusV1;
  readonly summary_json: Record<string, unknown>;
  readonly error_message: string | null;
}

export interface UiPerfTrendPointV1 {
  readonly run_kind: string;
  readonly bucket_start_ms: number;
  readonly metric_name: string;
  readonly metric_value: number;
  readonly baseline_value: number;
  readonly trend_delta_pct: number;
  readonly budget_result: PerfBudgetResultV1;
}

export type TelemetryModeV1 = 'local_only' | 'local_plus_otlp';

export type TelemetryAuditStatusV1 = 'pass' | 'warn' | 'fail';

export interface OtlpSinkConfigV1 {
  readonly enabled: boolean;
  readonly endpoint: string | null;
  readonly protocol: string;
  readonly timeout_ms: number;
  readonly batch_size: number;
  readonly redaction_profile: string;
}

export interface TelemetryExportRunV1 {
  readonly export_run_id: string;
  readonly status: PerfRunStatusV1;
  readonly from_ms: number;
  readonly to_ms: number;
  readonly sample_count: number;
  readonly redacted_count: number;
  readonly payload_sha256: string | null;
  readonly created_at_ms: number;
  readonly completed_at_ms: number | null;
  readonly error_code: string | null;
  readonly error_message: string | null;
}

export interface UiReleasePromotionRequestV1 {
  readonly channel: ReleaseChannelV1;
  readonly promote_from_internal_run_id: string;
  readonly notes_md: string;
  readonly dry_run: boolean;
}

export interface UiReleasePromotionResultV1 {
  readonly promotion_id: string;
  readonly channel: ReleaseChannelV1;
  readonly visibility: ReleaseVisibilityV1;
  readonly status: ReleaseRunStatusV1;
  readonly provenance: ArtifactProvenanceV1;
  readonly error_message: string | null;
}

export interface UiSigningSnapshotV1 {
  readonly run_id: string;
  readonly channel: ReleaseChannelV1;
  readonly visibility: ReleaseVisibilityV1;
  readonly artifact_count: number;
  readonly signing_status: SigningStatusV1;
  readonly notarization_status: SigningStatusV1;
  readonly manual_smoke_ready: boolean;
  readonly blocking_reasons: ReadonlyArray<string>;
}

export interface UiTelemetrySettingsV1 {
  readonly mode: TelemetryModeV1;
  readonly otlp: OtlpSinkConfigV1;
}

export interface UiTelemetryExportResultV1 {
  readonly run: TelemetryExportRunV1;
}

export interface TelemetryAuditRunV1 {
  readonly audit_id: string;
  readonly export_run_id: string | null;
  readonly status: TelemetryAuditStatusV1;
  readonly violations_count: number;
  readonly violations_json: unknown;
  readonly payload_sha256: string | null;
  readonly created_at_ms: number;
}

export interface UiRunTelemetryAuditResultV1 {
  readonly run: TelemetryAuditRunV1;
}

export type PerfAnomalySeverityV1 = 'low' | 'medium' | 'high' | 'critical';

export interface PerfAnomalyRecordV1 {
  readonly anomaly_id: string;
  readonly run_kind: string;
  readonly bucket_start_ms: number;
  readonly metric_name: string;
  readonly severity: PerfAnomalySeverityV1;
  readonly score: number;
  readonly baseline_value: number;
  readonly observed_value: number;
  readonly details_json: Record<string, unknown>;
  readonly created_at_ms: number;
}

export interface UiListPerfAnomaliesItemV1 {
  readonly anomaly_id: string;
  readonly run_kind: string;
  readonly bucket_start_ms: number;
  readonly metric_name: string;
  readonly severity: PerfAnomalySeverityV1;
  readonly score: number;
  readonly baseline_value: number;
  readonly observed_value: number;
  readonly details_json: Record<string, unknown>;
  readonly created_at_ms: number;
}

export type RolloutHealthStatusV1 = 'pass' | 'warn' | 'fail';

export type RolloutGateReasonV1 =
  | 'manual_smoke_missing'
  | 'compliance_failed'
  | 'telemetry_audit_failed'
  | 'anomaly_budget_failed'
  | 'incident_budget_failed'
  | 'signature_invalid'
  | 'soak_incomplete';

export type RolloutControllerActionV1 = 'advance' | 'pause' | 'block' | 'noop';

export interface ReleaseHealthMetricV1 {
  readonly metric_key: string;
  readonly status: RolloutHealthStatusV1;
  readonly observed_value: number;
  readonly threshold_warn: number | null;
  readonly threshold_fail: number | null;
  readonly details_json: Record<string, unknown>;
}

export interface ReleaseHealthScorecardV1 {
  readonly scope: string;
  readonly channel: string;
  readonly version: string;
  readonly stage: RolloutStageV1 | null;
  readonly overall_status: RolloutHealthStatusV1;
  readonly score: number;
  readonly metrics: ReadonlyArray<ReleaseHealthMetricV1>;
  readonly gate_reasons: ReadonlyArray<RolloutGateReasonV1>;
  readonly created_at_ms: number;
}

export interface ReleaseHealthSnapshotV1 {
  readonly snapshot_id: string;
  readonly scorecard: ReleaseHealthScorecardV1;
}

export interface UiEvaluateExtensionRolloutStageRequestV1 {
  readonly version: string;
  readonly stage: RolloutStageV1;
  readonly dry_run: boolean;
}

export interface UiEvaluateExtensionRolloutStageResultV1 {
  readonly action: RolloutControllerActionV1;
  readonly status: RolloutHealthStatusV1;
  readonly scorecard: ReleaseHealthScorecardV1;
  readonly soak_remaining_ms: number;
}

export interface UiAdvanceExtensionRolloutStageRequestV1 {
  readonly version: string;
  readonly from_stage: RolloutStageV1;
  readonly to_stage: RolloutStageV1;
  readonly dry_run: boolean;
}

export interface UiAdvanceExtensionRolloutStageResultV1 {
  readonly rollout_id: string | null;
  readonly action: RolloutControllerActionV1;
  readonly status: RolloutStatusV1;
  readonly from_stage: RolloutStageV1;
  readonly to_stage: RolloutStageV1;
  readonly gate_reasons: ReadonlyArray<RolloutGateReasonV1>;
  readonly scorecard: ReleaseHealthScorecardV1;
}

export interface UiEvaluateUpdateRolloutRequestV1 {
  readonly channel: UpdateChannelV1;
  readonly version: string;
  readonly stage: RolloutStageV1;
  readonly dry_run: boolean;
}

export interface UiEvaluateUpdateRolloutResultV1 {
  readonly action: RolloutControllerActionV1;
  readonly status: RolloutHealthStatusV1;
  readonly scorecard: ReleaseHealthScorecardV1;
  readonly soak_remaining_ms: number;
}

export interface UiAdvanceUpdateRolloutRequestV1 {
  readonly channel: UpdateChannelV1;
  readonly version: string;
  readonly from_stage: RolloutStageV1;
  readonly to_stage: RolloutStageV1;
  readonly dry_run: boolean;
}

export interface UiAdvanceUpdateRolloutResultV1 {
  readonly update_rollout_id: string | null;
  readonly action: RolloutControllerActionV1;
  readonly status: RolloutStatusV1;
  readonly channel: UpdateChannelV1;
  readonly from_stage: RolloutStageV1;
  readonly to_stage: RolloutStageV1;
  readonly gate_reasons: ReadonlyArray<RolloutGateReasonV1>;
  readonly scorecard: ReleaseHealthScorecardV1;
}

export interface ComplianceEvidenceItemV1 {
  readonly item_key: string;
  readonly path: string;
  readonly sha256: string;
  readonly size_bytes: number;
}

export interface ComplianceEvidencePackV1 {
  readonly pack_id: string;
  readonly kind: string;
  readonly channel: string;
  readonly version: string;
  readonly stage: RolloutStageV1 | null;
  readonly pack_path: string;
  readonly manifest_sha256: string;
  readonly items: ReadonlyArray<ComplianceEvidenceItemV1>;
  readonly created_at_ms: number;
  readonly status: string;
  readonly error_code: string | null;
  readonly error_message: string | null;
}

export interface UiGetComplianceEvidencePackResultV1 {
  readonly pack: ComplianceEvidencePackV1 | null;
}

export interface UiListComplianceEvidencePacksItemV1 {
  readonly pack_id: string;
  readonly kind: string;
  readonly channel: string;
  readonly version: string;
  readonly stage: RolloutStageV1 | null;
  readonly status: string;
  readonly created_at_ms: number;
  readonly pack_path: string;
  readonly manifest_sha256: string;
}

export interface RetentionPolicyV1 {
  readonly enabled: boolean;
  readonly retain_days: number;
  readonly max_sessions: number;
  readonly delete_exports: boolean;
  readonly delete_blobs: boolean;
}

export type RetentionRunModeV1 = 'dry_run' | 'apply';

export interface SessionDeleteResultV1 {
  readonly session_id: string;
  readonly db_deleted: boolean;
  readonly files_deleted: number;
  readonly missing_files: ReadonlyArray<string>;
  readonly blocked_paths: ReadonlyArray<string>;
  readonly errors: ReadonlyArray<string>;
}

export interface RetentionRunReportV1 {
  readonly run_id: string;
  readonly mode: RetentionRunModeV1;
  readonly evaluated_sessions: number;
  readonly candidate_sessions: number;
  readonly deleted_sessions: number;
  readonly skipped_running_sessions: number;
  readonly failed_sessions: number;
  readonly started_at_ms: number;
  readonly finished_at_ms: number;
}

export interface UiRetentionSettingsV1 {
  readonly policy: RetentionPolicyV1;
}

export interface UiRetentionRunResultV1 {
  readonly report: RetentionRunReportV1;
  readonly deleted: ReadonlyArray<SessionDeleteResultV1>;
}

export interface UiDeleteSessionResultV1 {
  readonly result: SessionDeleteResultV1;
}

export type UiConnectionStatusV1 = 'disconnected' | 'connecting' | 'connected';

export interface UiDiagnosticEntryV1 {
  readonly ts_ms: number;
  readonly kind: string;
  readonly message: string;
}

export interface UiDiagnosticsSnapshotV1 {
  readonly pairing_port: number | null;
  readonly pairing_token: string | null;
  readonly connection_status: UiConnectionStatusV1;
  readonly diagnostics: ReadonlyArray<UiDiagnosticEntryV1>;
  readonly capture_drop_markers: number;
  readonly capture_limit_markers: number;
}

export interface UiPairingStateV1 {
  readonly state: PairingUxStateV1;
  readonly pairing_port: number | null;
  readonly trusted_device_id: string | null;
  readonly connected: boolean;
}

export interface UiLaunchDesktopResultV1 {
  readonly launched: boolean;
  readonly method: string;
  readonly message: string;
}

export interface UiEvidenceResolveRequestV1 {
  readonly evidence_ref_id: string;
}

export interface UiEvidenceResolveResultV1 {
  readonly evidence_ref_id: string;
  readonly session_id: string;
  readonly kind: EvidenceKind;
  readonly route_subview: 'timeline' | 'network' | 'console' | 'findings' | 'overview';
  readonly target_id: string;
  readonly column: string | null;
  readonly json_pointer: string | null;
  readonly exact_pointer_found: boolean;
  readonly fallback_reason: string | null;
  readonly container_json: Record<string, unknown> | null;
  readonly highlighted_value: unknown | null;
}
