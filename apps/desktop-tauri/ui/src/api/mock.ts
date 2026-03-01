import type {
  EvidenceRefV1,
  UiExportListItemV1,
  UiConsoleRowV1,
  UiDiagnosticsSnapshotV1,
  UiEvidenceResolveResultV1,
  UiExportCapabilityV1,
  UiFindingCardV1,
  UiNetworkRowV1,
  UiSessionListItemV1,
  UiSessionOverviewV1,
  UiTimelineBundleV1,
} from '@dtt/shared-types';

const now = 1_729_001_000_000;

const sampleEvidence: EvidenceRefV1 = {
  v: 1,
  kind: 'net_row',
  session_id: 'sess_mock_001',
  label: 'Response status 403',
  ts_ms: now,
  redaction_level: 'metadata_only',
  target: {
    net_request_id: 'net_mock_1',
    table: 'network_responses',
    column: 'status_code',
    json_pointer: '/status_code',
  },
};

export const mockSessions: UiSessionListItemV1[] = [
  {
    session_id: 'sess_mock_001',
    privacy_mode: 'metadata_only',
    capture_source: 'extension_mv3',
    started_at_ms: now - 15_000,
    ended_at_ms: now,
    duration_ms: 15_000,
    findings_count: 2,
    status: 'completed',
  },
];

export const mockFindings: UiFindingCardV1[] = [
  {
    finding_id: 'finding_mock_1',
    session_id: 'sess_mock_001',
    detector_id: 'general.auth.401_403_primary.v1',
    detector_version: '1.0.0',
    title: 'Primary request returned 403',
    summary: 'Main API request failed with authorization-related status.',
    category: 'auth',
    severity_score: 72,
    confidence_score: 0.91,
    created_at_ms: now,
    interaction_id: 'int_mock_1',
    fix_steps_json: [
      {
        step_id: 'step-1',
        title: 'Verify token scope',
        body_md: 'Check token audience and permissions.',
        risk: 'low',
        applies_when: ['status=403'],
        actions: ['Inspect auth middleware config'],
        evidence_ids: ['evr_mock_1'],
      },
    ],
    claims: [
      {
        claim_id: 'claim_mock_1',
        rank: 1,
        truth: 'verified',
        title: '403 from primary API call',
        summary: 'Primary call failed with 403 in interaction window.',
        confidence_score: 0.91,
        evidence_refs: [sampleEvidence],
      },
    ],
  },
];

export const mockOverview: UiSessionOverviewV1 = {
  session: mockSessions[0],
  interactions_count: 1,
  network_requests_count: 1,
  network_responses_count: 1,
  network_completion_count: 1,
  console_entries_count: 1,
  findings_count: 2,
  top_findings: mockFindings,
};

export const mockTimeline: UiTimelineBundleV1 = {
  interactions: [
    {
      interaction_id: 'int_mock_1',
      interaction_kind: 'api_burst',
      opened_at_ms: now - 12_000,
      closed_at_ms: now - 10_000,
      primary_member_id: 'network_response:net_mock_1',
      members_count: 3,
    },
  ],
  events: [
    {
      stable_id: 'raw:evt_mock_1',
      ts_ms: now - 12_000,
      kind: 'raw_event',
      label: 'Network.requestWillBeSent',
      source_id: 'evt_mock_1',
    },
  ],
};

export const mockNetwork: UiNetworkRowV1[] = [
  {
    net_request_id: 'net_mock_1',
    started_at_ms: now - 12_000,
    method: 'GET',
    host: 'api.example.com',
    path: '/v1/profile',
    status_code: 403,
    duration_ms: 220,
    mime_type: 'application/json',
    is_streaming: false,
    redaction_level: 'metadata_only',
  },
];

export const mockConsole: UiConsoleRowV1[] = [
  {
    console_id: 'console_mock_1',
    ts_ms: now - 11_900,
    level: 'error',
    source: 'network',
    message_redacted: 'Request failed with status 403',
    message_len: 30,
  },
];

export const mockExports: UiExportCapabilityV1 = {
  session_id: 'sess_mock_001',
  default_mode: 'share_safe',
  full_export_allowed: false,
  full_export_block_reason: 'Full export is blocked for metadata_only sessions.',
  phase8_ready: true,
};

export const mockExportRuns: UiExportListItemV1[] = [
  {
    export_id: 'exp_mock_0',
    session_id: 'sess_mock_001',
    profile: 'share_safe',
    status: 'completed',
    zip_path: '/tmp/dtt-exports/exp_mock_0.zip',
    created_at_ms: now - 1_000,
    completed_at_ms: now - 900,
    integrity_ok: true,
    bundle_blake3: 'mock_bundle_hash_0',
    error_code: null,
    error_message: null,
  },
];

export const mockDiagnostics: UiDiagnosticsSnapshotV1 = {
  pairing_port: 32124,
  pairing_token: '0123456789abcdef0123456789abcdef',
  connection_status: 'connected',
  diagnostics: [
    {
      ts_ms: now,
      kind: 'connected',
      message: 'Extension websocket connected',
    },
  ],
  capture_drop_markers: 0,
  capture_limit_markers: 0,
};

export const mockEvidenceResolution: UiEvidenceResolveResultV1 = {
  evidence_ref_id: 'evr_mock_1',
  session_id: 'sess_mock_001',
  kind: 'net_row',
  route_subview: 'network',
  target_id: 'net_mock_1',
  column: 'status_code',
  json_pointer: '/status_code',
  exact_pointer_found: true,
  fallback_reason: null,
  container_json: {
    net_request_id: 'net_mock_1',
    status_code: 403,
  },
  highlighted_value: 403,
};
