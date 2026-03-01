import type {
  EvtHelloPayload,
  EvtSessionEndedPayload,
  EvtSessionStartedPayload,
  ExportProfileV1,
  RolloutStageV1,
  UpdateChannelV1,
  UiApplyUpdateResultV1,
  UiAdvanceExtensionRolloutStageResultV1,
  UiAdvanceUpdateRolloutResultV1,
  UiCheckForUpdateResultV1,
  UiEvaluateExtensionRolloutStageResultV1,
  UiEvaluateUpdateRolloutResultV1,
  UiGetComplianceEvidencePackResultV1,
  UiExtensionComplianceSnapshotV1,
  UiListComplianceEvidencePacksItemV1,
  UiListExtensionRolloutsItemV1,
  UiListPerfAnomaliesItemV1,
  ReleaseHealthScorecardV1,
  UiRunTelemetryAuditResultV1,
  UiStartExtensionPublicRolloutResultV1,
  UiUpdateRolloutSnapshotV1,
  RedactionLevel,
  TelemetryAuditRunV1,
  TelemetryExportRunV1,
  ReliabilityMetricKeyV1,
  ReleaseChannelV1,
  ReleasePlatformV1,
  ReleaseArtifactV1,
  RetentionPolicyV1,
  RetentionRunModeV1,
  TabDescriptorV1,
  UiBundleInspectEvidenceResolveResultV1,
  UiBundleInspectFindingV1,
  UiBundleInspectOpenResultV1,
  UiBundleInspectOverviewV1,
  UiConsoleRowV1,
  UiDeleteSessionResultV1,
  UiDiagnosticEntryV1,
  UiDiagnosticsSnapshotV1,
  UiEvidenceResolveResultV1,
  UiExportCapabilityV1,
  UiExportListItemV1,
  UiFindingCardV1,
  UiNetworkRowV1,
  UiOpenExportFolderResultV1,
  UiPerfRunListItemV1,
  UiPerfTrendPointV1,
  UiReleasePromotionResultV1,
  UiSigningSnapshotV1,
  UiTelemetryExportResultV1,
  UiTelemetrySettingsV1,
  UiRetentionRunResultV1,
  UiRetentionSettingsV1,
  UiSessionListItemV1,
  UiSessionOverviewV1,
  UiPairingStateV1,
  UiLaunchDesktopResultV1,
  UiStartExportResultV1,
  UiStartPerfRunResultV1,
  UiStartReleaseResultV1,
  UiTimelineBundleV1,
  UiValidateExportResultV1,
  UiReleaseListItemV1,
  UiReliabilitySeriesPointV1,
  UiReliabilitySnapshotV1,
} from '@dtt/shared-types';
import {
  mockConsole,
  mockDiagnostics,
  mockEvidenceResolution,
  mockExports,
  mockExportRuns,
  mockFindings,
  mockNetwork,
  mockOverview,
  mockSessions,
  mockTimeline,
} from './mock.js';

export interface DesktopClient {
  uiGetSessions(limit: number): Promise<UiSessionListItemV1[]>;
  uiGetSessionOverview(sessionId: string): Promise<UiSessionOverviewV1 | null>;
  uiGetTimeline(sessionId: string): Promise<UiTimelineBundleV1>;
  uiGetNetwork(sessionId: string): Promise<UiNetworkRowV1[]>;
  uiGetConsole(sessionId: string): Promise<UiConsoleRowV1[]>;
  uiGetFindings(sessionId: string | null, limit: number): Promise<UiFindingCardV1[]>;
  uiGetExports(sessionId: string): Promise<UiExportCapabilityV1>;
  uiStartExport(
    sessionId: string,
    profile: ExportProfileV1,
    outputDir: string | null,
  ): Promise<UiStartExportResultV1>;
  uiListExports(sessionId: string | null, limit: number): Promise<UiExportListItemV1[]>;
  uiValidateExport(exportId: string): Promise<UiValidateExportResultV1>;
  uiOpenExportFolder(exportId: string | null): Promise<UiOpenExportFolderResultV1>;
  uiStartRelease(
    channel: ReleaseChannelV1,
    version: string,
    notesMd: string,
    dryRun: boolean,
  ): Promise<UiStartReleaseResultV1>;
  uiListReleases(limit: number): Promise<UiReleaseListItemV1[]>;
  uiGetReleaseArtifactsByPlatform(
    platform: ReleasePlatformV1,
    limitRuns: number,
  ): Promise<ReleaseArtifactV1[]>;
  uiStartReleaseMatrix(
    channel: ReleaseChannelV1,
    version: string,
    notesMd: string,
    dryRun: boolean,
  ): Promise<UiStartReleaseResultV1>;
  uiStartReleasePromotion(
    channel: ReleaseChannelV1,
    promoteFromInternalRunId: string,
    notesMd: string,
    dryRun: boolean,
  ): Promise<UiReleasePromotionResultV1>;
  uiGetSigningSnapshot(runId: string): Promise<UiSigningSnapshotV1>;
  uiStartExtensionPublicRollout(
    version: string,
    stage: RolloutStageV1,
    notesMd: string,
    dryRun: boolean,
  ): Promise<UiStartExtensionPublicRolloutResultV1>;
  uiListExtensionRollouts(limit: number): Promise<UiListExtensionRolloutsItemV1[]>;
  uiGetExtensionComplianceSnapshot(
    rolloutId: string | null,
  ): Promise<UiExtensionComplianceSnapshotV1>;
  uiCheckForUpdates(
    channel: UpdateChannelV1,
    installId: string,
    currentVersion: string,
  ): Promise<UiCheckForUpdateResultV1>;
  uiApplyUpdate(
    channel: UpdateChannelV1,
    installId: string,
    currentVersion: string,
  ): Promise<UiApplyUpdateResultV1>;
  uiGetUpdateRolloutSnapshot(channel: UpdateChannelV1): Promise<UiUpdateRolloutSnapshotV1>;
  uiEvaluateExtensionRolloutStage(
    version: string,
    stage: RolloutStageV1,
  ): Promise<UiEvaluateExtensionRolloutStageResultV1>;
  uiAdvanceExtensionRolloutStage(
    version: string,
    fromStage: RolloutStageV1,
    toStage: RolloutStageV1,
    dryRun: boolean,
  ): Promise<UiAdvanceExtensionRolloutStageResultV1>;
  uiEvaluateUpdateRollout(
    channel: UpdateChannelV1,
    version: string,
    stage: RolloutStageV1,
  ): Promise<UiEvaluateUpdateRolloutResultV1>;
  uiAdvanceUpdateRollout(
    channel: UpdateChannelV1,
    version: string,
    fromStage: RolloutStageV1,
    toStage: RolloutStageV1,
    dryRun: boolean,
  ): Promise<UiAdvanceUpdateRolloutResultV1>;
  uiGetReleaseHealthScorecard(
    version: string,
    updaterChannel: UpdateChannelV1,
  ): Promise<ReleaseHealthScorecardV1>;
  uiGetComplianceEvidencePack(
    kind: string,
    channel: string,
    version: string,
    stage: RolloutStageV1 | null,
  ): Promise<UiGetComplianceEvidencePackResultV1>;
  uiListComplianceEvidencePacks(
    kind: string | null,
    limit: number,
  ): Promise<UiListComplianceEvidencePacksItemV1[]>;
  uiRunRolloutControllerTick(
    version: string,
    stage: RolloutStageV1,
    updaterChannel: UpdateChannelV1,
  ): Promise<ReleaseHealthScorecardV1>;
  uiOpenBundleInspect(bundlePath: string): Promise<UiBundleInspectOpenResultV1>;
  uiGetBundleInspectOverview(inspectId: string): Promise<UiBundleInspectOverviewV1>;
  uiListBundleInspectFindings(
    inspectId: string,
    limit: number,
  ): Promise<UiBundleInspectFindingV1[]>;
  uiResolveBundleInspectEvidence(
    inspectId: string,
    evidenceRefId: string,
  ): Promise<UiBundleInspectEvidenceResolveResultV1 | null>;
  uiCloseBundleInspect(inspectId: string): Promise<void>;
  uiGetDiagnostics(sessionId: string | null): Promise<UiDiagnosticsSnapshotV1>;
  uiGetBridgeDiagnostics(sessionId: string | null, limit: number): Promise<UiDiagnosticEntryV1[]>;
  uiGetReliabilitySnapshot(windowMs: number): Promise<UiReliabilitySnapshotV1>;
  uiListReliabilitySeries(
    metricKey: ReliabilityMetricKeyV1,
    fromMs: number,
    toMs: number,
    bucketMs: number,
  ): Promise<UiReliabilitySeriesPointV1[]>;
  uiStartPerfRun(runKind: string, inputRef: string): Promise<UiStartPerfRunResultV1>;
  uiListPerfRuns(limit: number): Promise<UiPerfRunListItemV1[]>;
  uiStartEnduranceRun(runKind: string): Promise<UiStartPerfRunResultV1>;
  uiListPerfTrends(runKind: string, limit: number): Promise<UiPerfTrendPointV1[]>;
  uiGetTelemetrySettings(): Promise<UiTelemetrySettingsV1>;
  uiSetTelemetrySettings(settings: UiTelemetrySettingsV1): Promise<UiTelemetrySettingsV1>;
  uiRunTelemetryExport(
    fromMs: number | null,
    toMs: number | null,
  ): Promise<UiTelemetryExportResultV1>;
  uiListTelemetryExports(limit: number): Promise<TelemetryExportRunV1[]>;
  uiRunTelemetryAudit(exportRunId: string | null): Promise<UiRunTelemetryAuditResultV1>;
  uiListTelemetryAudits(limit: number): Promise<TelemetryAuditRunV1[]>;
  uiListPerfAnomalies(runKind: string | null, limit: number): Promise<UiListPerfAnomaliesItemV1[]>;
  uiResolveEvidence(evidenceRefId: string): Promise<UiEvidenceResolveResultV1 | null>;
  uiListTabs(): Promise<TabDescriptorV1[]>;
  uiStartCapture(
    tabId: number,
    privacyMode: RedactionLevel,
    sessionId: string,
  ): Promise<EvtSessionStartedPayload>;
  uiStopCapture(sessionId: string): Promise<EvtSessionEndedPayload>;
  uiSetUiCapture(enabled: boolean): Promise<EvtHelloPayload>;
  uiGetPairingState(): Promise<UiPairingStateV1>;
  uiPairingDiscover(deviceId: string, browserLabel: string): Promise<UiPairingStateV1>;
  uiPairingApprove(deviceId: string, browserLabel: string): Promise<UiPairingStateV1>;
  uiPairingRevoke(deviceId: string): Promise<UiPairingStateV1>;
  uiLaunchOrFocusDesktop(): Promise<UiLaunchDesktopResultV1>;
  uiGetRetentionSettings(): Promise<UiRetentionSettingsV1>;
  uiSetRetentionSettings(policy: RetentionPolicyV1): Promise<UiRetentionSettingsV1>;
  uiRunRetention(mode: RetentionRunModeV1): Promise<UiRetentionRunResultV1>;
  uiDeleteSession(sessionId: string): Promise<UiDeleteSessionResultV1>;
}

function isLikelyTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const module = await import('@tauri-apps/api/core');
  return module.invoke<T>(command, args);
}

export class TauriDesktopClient implements DesktopClient {
  uiGetSessions(limit: number): Promise<UiSessionListItemV1[]> {
    return tauriInvoke('ui_get_sessions', { limit });
  }

  uiGetSessionOverview(sessionId: string): Promise<UiSessionOverviewV1 | null> {
    return tauriInvoke('ui_get_session_overview', { sessionId });
  }

  uiGetTimeline(sessionId: string): Promise<UiTimelineBundleV1> {
    return tauriInvoke('ui_get_timeline', { sessionId });
  }

  uiGetNetwork(sessionId: string): Promise<UiNetworkRowV1[]> {
    return tauriInvoke('ui_get_network', { sessionId });
  }

  uiGetConsole(sessionId: string): Promise<UiConsoleRowV1[]> {
    return tauriInvoke('ui_get_console', { sessionId });
  }

  uiGetFindings(sessionId: string | null, limit: number): Promise<UiFindingCardV1[]> {
    return tauriInvoke('ui_get_findings', { sessionId, limit });
  }

  uiGetExports(sessionId: string): Promise<UiExportCapabilityV1> {
    return tauriInvoke('ui_get_exports', { sessionId });
  }

  uiStartExport(
    sessionId: string,
    profile: ExportProfileV1,
    outputDir: string | null,
  ): Promise<UiStartExportResultV1> {
    return tauriInvoke('ui_start_export', { sessionId, profile, outputDir });
  }

  uiListExports(sessionId: string | null, limit: number): Promise<UiExportListItemV1[]> {
    return tauriInvoke('ui_list_exports', { sessionId, limit });
  }

  uiValidateExport(exportId: string): Promise<UiValidateExportResultV1> {
    return tauriInvoke('ui_validate_export', { exportId });
  }

  uiOpenExportFolder(exportId: string | null): Promise<UiOpenExportFolderResultV1> {
    return tauriInvoke('ui_open_export_folder', { exportId });
  }

  uiStartRelease(
    channel: ReleaseChannelV1,
    version: string,
    notesMd: string,
    dryRun: boolean,
  ): Promise<UiStartReleaseResultV1> {
    return tauriInvoke('ui_start_release', {
      channel,
      version,
      notes_md: notesMd,
      dry_run: dryRun,
    });
  }

  uiStartReleasePromotion(
    channel: ReleaseChannelV1,
    promoteFromInternalRunId: string,
    notesMd: string,
    dryRun: boolean,
  ): Promise<UiReleasePromotionResultV1> {
    return tauriInvoke('ui_start_release_promotion', {
      channel,
      promote_from_internal_run_id: promoteFromInternalRunId,
      notes_md: notesMd,
      dry_run: dryRun,
    });
  }

  uiGetSigningSnapshot(runId: string): Promise<UiSigningSnapshotV1> {
    return tauriInvoke('ui_get_signing_snapshot', { run_id: runId });
  }

  uiStartExtensionPublicRollout(
    version: string,
    stage: RolloutStageV1,
    notesMd: string,
    dryRun: boolean,
  ): Promise<UiStartExtensionPublicRolloutResultV1> {
    return tauriInvoke('ui_start_extension_public_rollout', {
      version,
      stage,
      notes_md: notesMd,
      dry_run: dryRun,
    });
  }

  uiListExtensionRollouts(limit: number): Promise<UiListExtensionRolloutsItemV1[]> {
    return tauriInvoke('ui_list_extension_rollouts', { limit });
  }

  uiGetExtensionComplianceSnapshot(
    rolloutId: string | null,
  ): Promise<UiExtensionComplianceSnapshotV1> {
    return tauriInvoke('ui_get_extension_compliance_snapshot', {
      rollout_id: rolloutId,
    });
  }

  uiCheckForUpdates(
    channel: UpdateChannelV1,
    installId: string,
    currentVersion: string,
  ): Promise<UiCheckForUpdateResultV1> {
    return tauriInvoke('ui_check_for_updates', {
      channel,
      install_id: installId,
      current_version: currentVersion,
    });
  }

  uiApplyUpdate(
    channel: UpdateChannelV1,
    installId: string,
    currentVersion: string,
  ): Promise<UiApplyUpdateResultV1> {
    return tauriInvoke('ui_apply_update', {
      channel,
      install_id: installId,
      current_version: currentVersion,
    });
  }

  uiGetUpdateRolloutSnapshot(channel: UpdateChannelV1): Promise<UiUpdateRolloutSnapshotV1> {
    return tauriInvoke('ui_get_update_rollout_snapshot', { channel });
  }

  uiEvaluateExtensionRolloutStage(
    version: string,
    stage: RolloutStageV1,
  ): Promise<UiEvaluateExtensionRolloutStageResultV1> {
    return tauriInvoke('ui_evaluate_extension_rollout_stage', { version, stage });
  }

  uiAdvanceExtensionRolloutStage(
    version: string,
    fromStage: RolloutStageV1,
    toStage: RolloutStageV1,
    dryRun: boolean,
  ): Promise<UiAdvanceExtensionRolloutStageResultV1> {
    return tauriInvoke('ui_advance_extension_rollout_stage', {
      version,
      from_stage: fromStage,
      to_stage: toStage,
      dry_run: dryRun,
    });
  }

  uiEvaluateUpdateRollout(
    channel: UpdateChannelV1,
    version: string,
    stage: RolloutStageV1,
  ): Promise<UiEvaluateUpdateRolloutResultV1> {
    return tauriInvoke('ui_evaluate_update_rollout', { channel, version, stage });
  }

  uiAdvanceUpdateRollout(
    channel: UpdateChannelV1,
    version: string,
    fromStage: RolloutStageV1,
    toStage: RolloutStageV1,
    dryRun: boolean,
  ): Promise<UiAdvanceUpdateRolloutResultV1> {
    return tauriInvoke('ui_advance_update_rollout', {
      channel,
      version,
      from_stage: fromStage,
      to_stage: toStage,
      dry_run: dryRun,
    });
  }

  uiGetReleaseHealthScorecard(
    version: string,
    updaterChannel: UpdateChannelV1,
  ): Promise<ReleaseHealthScorecardV1> {
    return tauriInvoke('ui_get_release_health_scorecard', {
      version,
      updater_channel: updaterChannel,
    });
  }

  uiGetComplianceEvidencePack(
    kind: string,
    channel: string,
    version: string,
    stage: RolloutStageV1 | null,
  ): Promise<UiGetComplianceEvidencePackResultV1> {
    return tauriInvoke('ui_get_compliance_evidence_pack', {
      kind,
      channel,
      version,
      stage,
    });
  }

  uiListComplianceEvidencePacks(
    kind: string | null,
    limit: number,
  ): Promise<UiListComplianceEvidencePacksItemV1[]> {
    return tauriInvoke('ui_list_compliance_evidence_packs', { kind, limit });
  }

  uiRunRolloutControllerTick(
    version: string,
    stage: RolloutStageV1,
    updaterChannel: UpdateChannelV1,
  ): Promise<ReleaseHealthScorecardV1> {
    return tauriInvoke('ui_run_rollout_controller_tick', {
      version,
      stage,
      updater_channel: updaterChannel,
    });
  }

  uiListReleases(limit: number): Promise<UiReleaseListItemV1[]> {
    return tauriInvoke('ui_list_releases', { limit });
  }

  uiGetReleaseArtifactsByPlatform(
    platform: ReleasePlatformV1,
    limitRuns: number,
  ): Promise<ReleaseArtifactV1[]> {
    return tauriInvoke('ui_get_release_artifacts_by_platform', {
      platform,
      limit_runs: limitRuns,
    });
  }

  uiStartReleaseMatrix(
    channel: ReleaseChannelV1,
    version: string,
    notesMd: string,
    dryRun: boolean,
  ): Promise<UiStartReleaseResultV1> {
    return tauriInvoke('ui_start_release_matrix', {
      channel,
      version,
      notes_md: notesMd,
      dry_run: dryRun,
    });
  }

  uiOpenBundleInspect(bundlePath: string): Promise<UiBundleInspectOpenResultV1> {
    return tauriInvoke('ui_open_bundle_inspect', { bundle_path: bundlePath });
  }

  uiGetBundleInspectOverview(inspectId: string): Promise<UiBundleInspectOverviewV1> {
    return tauriInvoke('ui_get_bundle_inspect_overview', { inspect_id: inspectId });
  }

  uiListBundleInspectFindings(
    inspectId: string,
    limit: number,
  ): Promise<UiBundleInspectFindingV1[]> {
    return tauriInvoke('ui_list_bundle_inspect_findings', { inspect_id: inspectId, limit });
  }

  uiResolveBundleInspectEvidence(
    inspectId: string,
    evidenceRefId: string,
  ): Promise<UiBundleInspectEvidenceResolveResultV1 | null> {
    return tauriInvoke('ui_resolve_bundle_inspect_evidence', {
      inspect_id: inspectId,
      evidence_ref_id: evidenceRefId,
    });
  }

  uiCloseBundleInspect(inspectId: string): Promise<void> {
    return tauriInvoke('ui_close_bundle_inspect', { inspect_id: inspectId });
  }

  uiGetDiagnostics(sessionId: string | null): Promise<UiDiagnosticsSnapshotV1> {
    return tauriInvoke('ui_get_diagnostics', { sessionId });
  }

  uiGetBridgeDiagnostics(sessionId: string | null, limit: number): Promise<UiDiagnosticEntryV1[]> {
    return tauriInvoke('ui_get_bridge_diagnostics', { sessionId, limit });
  }

  uiGetReliabilitySnapshot(windowMs: number): Promise<UiReliabilitySnapshotV1> {
    return tauriInvoke('ui_get_reliability_snapshot', { window_ms: windowMs });
  }

  uiListReliabilitySeries(
    metricKey: ReliabilityMetricKeyV1,
    fromMs: number,
    toMs: number,
    bucketMs: number,
  ): Promise<UiReliabilitySeriesPointV1[]> {
    return tauriInvoke('ui_list_reliability_series', {
      metric_key: metricKey,
      from_ms: fromMs,
      to_ms: toMs,
      bucket_ms: bucketMs,
    });
  }

  uiStartPerfRun(runKind: string, inputRef: string): Promise<UiStartPerfRunResultV1> {
    return tauriInvoke('ui_start_perf_run', { run_kind: runKind, input_ref: inputRef });
  }

  uiListPerfRuns(limit: number): Promise<UiPerfRunListItemV1[]> {
    return tauriInvoke('ui_list_perf_runs', { limit });
  }

  uiStartEnduranceRun(runKind: string): Promise<UiStartPerfRunResultV1> {
    return tauriInvoke('ui_start_endurance_run', { run_kind: runKind });
  }

  uiListPerfTrends(runKind: string, limit: number): Promise<UiPerfTrendPointV1[]> {
    return tauriInvoke('ui_list_perf_trends', { run_kind: runKind, limit });
  }

  uiGetTelemetrySettings(): Promise<UiTelemetrySettingsV1> {
    return tauriInvoke('ui_get_telemetry_settings');
  }

  uiSetTelemetrySettings(settings: UiTelemetrySettingsV1): Promise<UiTelemetrySettingsV1> {
    return tauriInvoke('ui_set_telemetry_settings', { settings });
  }

  uiRunTelemetryExport(
    fromMs: number | null,
    toMs: number | null,
  ): Promise<UiTelemetryExportResultV1> {
    return tauriInvoke('ui_run_telemetry_export', { from_ms: fromMs, to_ms: toMs });
  }

  uiListTelemetryExports(limit: number): Promise<TelemetryExportRunV1[]> {
    return tauriInvoke('ui_list_telemetry_exports', { limit });
  }

  uiRunTelemetryAudit(exportRunId: string | null): Promise<UiRunTelemetryAuditResultV1> {
    return tauriInvoke('ui_run_telemetry_audit', { export_run_id: exportRunId });
  }

  uiListTelemetryAudits(limit: number): Promise<TelemetryAuditRunV1[]> {
    return tauriInvoke('ui_list_telemetry_audits', { limit });
  }

  uiListPerfAnomalies(runKind: string | null, limit: number): Promise<UiListPerfAnomaliesItemV1[]> {
    return tauriInvoke('ui_list_perf_anomalies', { run_kind: runKind, limit });
  }

  uiResolveEvidence(evidenceRefId: string): Promise<UiEvidenceResolveResultV1 | null> {
    return tauriInvoke('ui_resolve_evidence', { evidenceRefId });
  }

  uiListTabs(): Promise<TabDescriptorV1[]> {
    return tauriInvoke('ui_list_tabs');
  }

  uiStartCapture(
    tabId: number,
    privacyMode: RedactionLevel,
    sessionId: string,
  ): Promise<EvtSessionStartedPayload> {
    return tauriInvoke('ui_start_capture', { tabId, privacyMode, sessionId });
  }

  uiStopCapture(sessionId: string): Promise<EvtSessionEndedPayload> {
    return tauriInvoke('ui_stop_capture', { sessionId });
  }

  uiSetUiCapture(enabled: boolean): Promise<EvtHelloPayload> {
    return tauriInvoke('ui_set_ui_capture', { enabled });
  }

  uiGetPairingState(): Promise<UiPairingStateV1> {
    return tauriInvoke('ui_get_pairing_state');
  }

  uiPairingDiscover(deviceId: string, browserLabel: string): Promise<UiPairingStateV1> {
    return tauriInvoke('ui_pairing_discover', { deviceId, browserLabel });
  }

  uiPairingApprove(deviceId: string, browserLabel: string): Promise<UiPairingStateV1> {
    return tauriInvoke('ui_pairing_approve', { deviceId, browserLabel });
  }

  uiPairingRevoke(deviceId: string): Promise<UiPairingStateV1> {
    return tauriInvoke('ui_pairing_revoke', { deviceId });
  }

  uiLaunchOrFocusDesktop(): Promise<UiLaunchDesktopResultV1> {
    return tauriInvoke('ui_launch_or_focus_desktop');
  }

  uiGetRetentionSettings(): Promise<UiRetentionSettingsV1> {
    return tauriInvoke('ui_get_retention_settings');
  }

  uiSetRetentionSettings(policy: RetentionPolicyV1): Promise<UiRetentionSettingsV1> {
    return tauriInvoke('ui_set_retention_settings', { policy });
  }

  uiRunRetention(mode: RetentionRunModeV1): Promise<UiRetentionRunResultV1> {
    return tauriInvoke('ui_run_retention', { mode });
  }

  uiDeleteSession(sessionId: string): Promise<UiDeleteSessionResultV1> {
    return tauriInvoke('ui_delete_session', { sessionId });
  }
}

const defaultRetentionPolicy: RetentionPolicyV1 = {
  enabled: true,
  retain_days: 30,
  max_sessions: 1000,
  delete_exports: true,
  delete_blobs: true,
};

const defaultTelemetrySettings: UiTelemetrySettingsV1 = {
  mode: 'local_only',
  otlp: {
    enabled: false,
    endpoint: null,
    protocol: 'http',
    timeout_ms: 5000,
    batch_size: 250,
    redaction_profile: 'counters_only',
  },
};

export class MockDesktopClient implements DesktopClient {
  private readonly exportRuns: UiExportListItemV1[] = [...mockExportRuns];
  private readonly sessions: UiSessionListItemV1[] = [...mockSessions];
  private readonly releaseRuns: UiReleaseListItemV1[] = [];
  private readonly extensionRollouts: UiListExtensionRolloutsItemV1[] = [];
  private readonly complianceEvidencePacks: UiListComplianceEvidencePacksItemV1[] = [];
  private readonly perfRuns: UiPerfRunListItemV1[] = [];
  private readonly perfAnomalies: UiListPerfAnomaliesItemV1[] = [];
  private readonly telemetryExports: TelemetryExportRunV1[] = [];
  private readonly telemetryAudits: TelemetryAuditRunV1[] = [];
  private readonly bundleInspects = new Map<string, UiBundleInspectOpenResultV1>();
  private retentionPolicy: RetentionPolicyV1 = { ...defaultRetentionPolicy };
  private telemetrySettings: UiTelemetrySettingsV1 = { ...defaultTelemetrySettings };
  private pairingState: UiPairingStateV1 = {
    state: 'not_paired',
    pairing_port: 32123,
    trusted_device_id: null,
    connected: false,
  };

  async uiGetSessions(limit: number): Promise<UiSessionListItemV1[]> {
    return this.sessions.slice(0, limit);
  }

  async uiGetSessionOverview(sessionId: string): Promise<UiSessionOverviewV1 | null> {
    return sessionId === mockOverview.session.session_id ? mockOverview : null;
  }

  async uiGetTimeline(_sessionId: string): Promise<UiTimelineBundleV1> {
    return mockTimeline;
  }

  async uiGetNetwork(_sessionId: string): Promise<UiNetworkRowV1[]> {
    return mockNetwork;
  }

  async uiGetConsole(_sessionId: string): Promise<UiConsoleRowV1[]> {
    return mockConsole;
  }

  async uiGetFindings(sessionId: string | null, limit: number): Promise<UiFindingCardV1[]> {
    const rows = sessionId
      ? mockFindings.filter((row) => row.session_id === sessionId)
      : mockFindings;
    return rows.slice(0, limit);
  }

  async uiGetExports(_sessionId: string): Promise<UiExportCapabilityV1> {
    return mockExports;
  }

  async uiStartExport(
    sessionId: string,
    profile: ExportProfileV1,
    _outputDir: string | null,
  ): Promise<UiStartExportResultV1> {
    const exportId = `exp_mock_${this.exportRuns.length + 1}`;
    const now = 1_729_001_100_000 + this.exportRuns.length;
    this.exportRuns.unshift({
      export_id: exportId,
      session_id: sessionId,
      profile,
      status: 'completed',
      zip_path: `/tmp/dtt-exports/${exportId}.zip`,
      created_at_ms: now,
      completed_at_ms: now,
      integrity_ok: true,
      bundle_blake3: 'mock_bundle_hash',
      error_code: null,
      error_message: null,
    });
    return {
      export_id: exportId,
      status: 'completed',
      zip_path: `/tmp/dtt-exports/${exportId}.zip`,
      integrity_ok: true,
      bundle_blake3: 'mock_bundle_hash',
      error_message: null,
    };
  }

  async uiListExports(sessionId: string | null, limit: number): Promise<UiExportListItemV1[]> {
    const rows = sessionId
      ? this.exportRuns.filter((row) => row.session_id === sessionId)
      : this.exportRuns;
    return rows.slice(0, limit);
  }

  async uiValidateExport(exportId: string): Promise<UiValidateExportResultV1> {
    const found = this.exportRuns.find((row) => row.export_id === exportId);
    return {
      export_id: exportId,
      valid: Boolean(found),
      bundle_hash_matches: Boolean(found),
      mismatched_files: [],
      missing_paths: [],
    };
  }

  async uiOpenExportFolder(exportId: string | null): Promise<UiOpenExportFolderResultV1> {
    const found = exportId
      ? this.exportRuns.find((row) => row.export_id === exportId)
      : this.exportRuns[0];
    return {
      supported: false,
      opened: false,
      path: found?.zip_path ?? '/tmp/dtt-exports',
      message: 'Open-folder is unavailable in this build',
    };
  }

  async uiStartRelease(
    channel: ReleaseChannelV1,
    version: string,
    _notesMd: string,
    dryRun: boolean,
  ): Promise<UiStartReleaseResultV1> {
    const runId = `rel_mock_${this.releaseRuns.length + 1}`;
    const startedAt = 1_729_100_000_000 + this.releaseRuns.length;
    const artifacts = [
      {
        kind: 'mac_zip' as const,
        platform: 'macos' as const,
        arch: 'x64' as const,
        target_triple: 'x86_64-apple-darwin',
        path: `/tmp/dtt-releases/${version}/${runId}/dtt-desktop-macos-v${version}.zip`,
        sha256: dryRun ? 'dry_run' : 'mock_sha256',
        size_bytes: dryRun ? 0 : 1_234,
      },
      {
        kind: 'release_manifest' as const,
        platform: 'macos' as const,
        arch: 'x64' as const,
        target_triple: 'x86_64-apple-darwin',
        path: `/tmp/dtt-releases/${version}/${runId}/release-manifest.v1.json`,
        sha256: dryRun ? 'dry_run' : 'mock_sha256_manifest',
        size_bytes: dryRun ? 0 : 456,
      },
    ];
    this.releaseRuns.unshift({
      run_id: runId,
      channel,
      version,
      commit_sha: 'mock_commit',
      status: 'completed',
      artifacts,
      started_at_ms: startedAt,
      completed_at_ms: startedAt + 100,
      error_code: null,
      error_message: null,
    });
    return {
      run_id: runId,
      status: 'completed',
      artifacts,
      error_message: null,
    };
  }

  async uiListReleases(limit: number): Promise<UiReleaseListItemV1[]> {
    return this.releaseRuns.slice(0, limit);
  }

  async uiGetReleaseArtifactsByPlatform(
    platform: ReleasePlatformV1,
    limitRuns: number,
  ): Promise<ReleaseArtifactV1[]> {
    return this.releaseRuns
      .slice(0, limitRuns)
      .flatMap((run) => run.artifacts)
      .filter((artifact) => artifact.platform === platform);
  }

  async uiStartReleaseMatrix(
    channel: ReleaseChannelV1,
    version: string,
    notesMd: string,
    dryRun: boolean,
  ): Promise<UiStartReleaseResultV1> {
    const base = await this.uiStartRelease(channel, version, notesMd, dryRun);
    const run = this.releaseRuns[0];
    if (!run) {
      return base;
    }
    const nextArtifacts: UiStartReleaseResultV1['artifacts'] = [
      ...run.artifacts,
      {
        kind: 'windows_zip',
        platform: 'windows',
        arch: 'x64',
        target_triple: 'x86_64-pc-windows-msvc',
        path: `/tmp/dtt-releases/${version}/${run.run_id}/dtt-desktop-windows-v${version}.zip`,
        sha256: dryRun ? 'dry_run' : 'mock_sha256_win_zip',
        size_bytes: dryRun ? 0 : 2_048,
      },
      {
        kind: 'linux_app_image',
        platform: 'linux',
        arch: 'x64',
        target_triple: 'x86_64-unknown-linux-gnu',
        path: `/tmp/dtt-releases/${version}/${run.run_id}/dtt-desktop-linux-v${version}.AppImage`,
        sha256: dryRun ? 'dry_run' : 'mock_sha256_linux_appimage',
        size_bytes: dryRun ? 0 : 3_072,
      },
    ];
    this.releaseRuns[0] = { ...run, artifacts: nextArtifacts };
    return { ...base, artifacts: nextArtifacts };
  }

  async uiStartReleasePromotion(
    channel: ReleaseChannelV1,
    promoteFromInternalRunId: string,
    _notesMd: string,
    _dryRun: boolean,
  ): Promise<UiReleasePromotionResultV1> {
    return {
      promotion_id: `prm_mock_${promoteFromInternalRunId}`,
      channel,
      visibility: channel === 'staged_public_prerelease' ? 'staged_public' : 'internal',
      status: 'completed',
      provenance: {
        build_id: promoteFromInternalRunId,
        workflow_run_id: 'mock_workflow',
        source_commit: 'mock_commit',
        signing_status: 'verified',
        notarization_status: 'verified',
      },
      error_message: null,
    };
  }

  async uiGetSigningSnapshot(runId: string): Promise<UiSigningSnapshotV1> {
    return {
      run_id: runId,
      channel: 'internal_beta',
      visibility: 'internal',
      artifact_count: 3,
      signing_status: 'verified',
      notarization_status: 'verified',
      manual_smoke_ready: false,
      blocking_reasons: ['manual_smoke_missing'],
    };
  }

  async uiStartExtensionPublicRollout(
    version: string,
    stage: RolloutStageV1,
    _notesMd: string,
    _dryRun: boolean,
  ): Promise<UiStartExtensionPublicRolloutResultV1> {
    const rolloutId = `ext_mock_${this.extensionRollouts.length + 1}`;
    const startedAt = 1_729_510_000_000 + this.extensionRollouts.length;
    const row: UiListExtensionRolloutsItemV1 = {
      rollout_id: rolloutId,
      channel: 'chrome_store_public',
      version,
      stage,
      status: 'completed',
      cws_item_id: 'mock_cws_item_id',
      started_at_ms: startedAt,
      completed_at_ms: startedAt + 50,
      error_code: null,
      error_message: null,
    };
    this.extensionRollouts.unshift(row);
    return {
      rollout_id: rolloutId,
      channel: 'chrome_store_public',
      version,
      stage,
      status: 'completed',
      cws_item_id: 'mock_cws_item_id',
      error_message: null,
    };
  }

  async uiListExtensionRollouts(limit: number): Promise<UiListExtensionRolloutsItemV1[]> {
    return this.extensionRollouts.slice(0, limit);
  }

  async uiGetExtensionComplianceSnapshot(
    rolloutId: string | null,
  ): Promise<UiExtensionComplianceSnapshotV1> {
    return {
      rollout_id: rolloutId,
      checks_total: 5,
      checks_passed: 4,
      checks_failed: 0,
      checks_warn: 1,
      checks: [
        {
          check_key: 'privacy_policy_url_present',
          status: 'pass',
          details_json: { https: true },
          checked_at_ms: 1_729_510_000_000,
        },
      ],
      blocking_reasons: [],
    };
  }

  async uiCheckForUpdates(
    channel: UpdateChannelV1,
    _installId: string,
    currentVersion: string,
  ): Promise<UiCheckForUpdateResultV1> {
    return {
      channel,
      current_version: currentVersion,
      latest_version: '0.2.0',
      eligibility: 'eligible',
      stage: 'pct_25',
      rollout_pct: 25,
      signature_verified: true,
      update_rollout_id: 'upd_mock_1',
      artifact: null,
      reason: null,
    };
  }

  async uiApplyUpdate(
    _channel: UpdateChannelV1,
    _installId: string,
    _currentVersion: string,
  ): Promise<UiApplyUpdateResultV1> {
    return {
      update_rollout_id: 'upd_mock_1',
      applied: true,
      eligibility: 'eligible',
      signature_verified: true,
      message: 'update eligible and ready for installer handoff',
    };
  }

  async uiGetUpdateRolloutSnapshot(channel: UpdateChannelV1): Promise<UiUpdateRolloutSnapshotV1> {
    return {
      update_rollout_id: 'upd_mock_1',
      channel,
      version: '0.2.0',
      stage: 'pct_25',
      rollout_pct: 25,
      status: 'active',
      feed_url: 'https://example.invalid/feed/latest.json',
      signature_verified: true,
      started_at_ms: 1_729_510_000_000,
      completed_at_ms: null,
      error_code: null,
      error_message: null,
    };
  }

  async uiEvaluateExtensionRolloutStage(
    version: string,
    stage: RolloutStageV1,
  ): Promise<UiEvaluateExtensionRolloutStageResultV1> {
    return {
      action: 'advance',
      status: 'pass',
      soak_remaining_ms: 0,
      scorecard: {
        scope: 'extension',
        channel: 'chrome_store_public',
        version,
        stage,
        overall_status: 'pass',
        score: 100,
        metrics: [],
        gate_reasons: [],
        created_at_ms: 1_729_530_000_000,
      },
    };
  }

  async uiAdvanceExtensionRolloutStage(
    version: string,
    fromStage: RolloutStageV1,
    toStage: RolloutStageV1,
    dryRun: boolean,
  ): Promise<UiAdvanceExtensionRolloutStageResultV1> {
    const evaluated = await this.uiEvaluateExtensionRolloutStage(version, fromStage);
    const rolloutId = dryRun ? null : `ext_mock_adv_${this.extensionRollouts.length + 1}`;
    this.complianceEvidencePacks.unshift({
      pack_id: `cep_mock_${this.complianceEvidencePacks.length + 1}`,
      kind: 'extension',
      channel: 'chrome_store_public',
      version,
      stage: toStage,
      status: 'generated',
      created_at_ms: 1_729_530_000_001 + this.complianceEvidencePacks.length,
      pack_path: `/tmp/dtt/evidence/extension/${version}/${toStage}`,
      manifest_sha256: 'mock_manifest_sha256',
    });
    return {
      rollout_id: rolloutId,
      action: evaluated.action,
      status: dryRun ? 'planned' : 'active',
      from_stage: fromStage,
      to_stage: toStage,
      gate_reasons: [],
      scorecard: evaluated.scorecard,
    };
  }

  async uiEvaluateUpdateRollout(
    channel: UpdateChannelV1,
    version: string,
    stage: RolloutStageV1,
  ): Promise<UiEvaluateUpdateRolloutResultV1> {
    return {
      action: 'advance',
      status: 'pass',
      soak_remaining_ms: 0,
      scorecard: {
        scope: 'updater',
        channel,
        version,
        stage,
        overall_status: 'pass',
        score: 95,
        metrics: [],
        gate_reasons: [],
        created_at_ms: 1_729_530_100_000,
      },
    };
  }

  async uiAdvanceUpdateRollout(
    channel: UpdateChannelV1,
    version: string,
    fromStage: RolloutStageV1,
    toStage: RolloutStageV1,
    dryRun: boolean,
  ): Promise<UiAdvanceUpdateRolloutResultV1> {
    const evaluated = await this.uiEvaluateUpdateRollout(channel, version, fromStage);
    return {
      update_rollout_id: dryRun ? null : 'upd_mock_advanced',
      action: evaluated.action,
      status: dryRun ? 'planned' : 'active',
      channel,
      from_stage: fromStage,
      to_stage: toStage,
      gate_reasons: [],
      scorecard: evaluated.scorecard,
    };
  }

  async uiGetReleaseHealthScorecard(
    version: string,
    updaterChannel: UpdateChannelV1,
  ): Promise<ReleaseHealthScorecardV1> {
    return {
      scope: 'global',
      channel: updaterChannel,
      version,
      stage: null,
      overall_status: 'pass',
      score: 97.5,
      metrics: [],
      gate_reasons: [],
      created_at_ms: 1_729_530_200_000,
    };
  }

  async uiGetComplianceEvidencePack(
    kind: string,
    channel: string,
    version: string,
    stage: RolloutStageV1 | null,
  ): Promise<UiGetComplianceEvidencePackResultV1> {
    const listed = this.complianceEvidencePacks.find(
      (row) =>
        row.kind === kind &&
        row.channel === channel &&
        row.version === version &&
        row.stage === stage,
    );
    if (!listed) {
      return { pack: null };
    }
    return {
      pack: {
        pack_id: listed.pack_id,
        kind: listed.kind,
        channel: listed.channel,
        version: listed.version,
        stage: listed.stage,
        pack_path: listed.pack_path,
        manifest_sha256: listed.manifest_sha256,
        items: [
          {
            item_key: 'permission_allowlist_diff',
            path: `${listed.pack_path}/permission_allowlist_diff.json`,
            sha256: 'mock_sha256',
            size_bytes: 120,
          },
        ],
        created_at_ms: listed.created_at_ms,
        status: listed.status,
        error_code: null,
        error_message: null,
      },
    };
  }

  async uiListComplianceEvidencePacks(
    kind: string | null,
    limit: number,
  ): Promise<UiListComplianceEvidencePacksItemV1[]> {
    const rows = kind
      ? this.complianceEvidencePacks.filter((row) => row.kind === kind)
      : this.complianceEvidencePacks;
    return rows.slice(0, limit);
  }

  async uiRunRolloutControllerTick(
    version: string,
    _stage: RolloutStageV1,
    updaterChannel: UpdateChannelV1,
  ): Promise<ReleaseHealthScorecardV1> {
    return this.uiGetReleaseHealthScorecard(version, updaterChannel);
  }

  async uiOpenBundleInspect(bundlePath: string): Promise<UiBundleInspectOpenResultV1> {
    const inspectId = `insp_mock_${this.bundleInspects.size + 1}`;
    const result: UiBundleInspectOpenResultV1 = {
      inspect_id: inspectId,
      bundle_path: bundlePath,
      integrity_valid: true,
      session_id: mockOverview.session.session_id,
      exported_at_ms: mockOverview.session.ended_at_ms ?? mockOverview.session.started_at_ms,
      privacy_mode: mockOverview.session.privacy_mode,
      profile: 'share_safe',
    };
    this.bundleInspects.set(inspectId, result);
    return result;
  }

  async uiGetBundleInspectOverview(inspectId: string): Promise<UiBundleInspectOverviewV1> {
    const opened = this.bundleInspects.get(inspectId);
    if (!opened) {
      throw new Error(`inspect ${inspectId} not found`);
    }
    return {
      inspect_id: opened.inspect_id,
      bundle_path: opened.bundle_path,
      integrity_valid: opened.integrity_valid,
      session_id: opened.session_id,
      exported_at_ms: opened.exported_at_ms,
      privacy_mode: opened.privacy_mode,
      profile: opened.profile,
      findings_count: mockFindings.length,
      evidence_refs_count: mockFindings.length,
    };
  }

  async uiListBundleInspectFindings(
    inspectId: string,
    limit: number,
  ): Promise<UiBundleInspectFindingV1[]> {
    if (!this.bundleInspects.has(inspectId)) {
      throw new Error(`inspect ${inspectId} not found`);
    }
    return mockFindings.slice(0, limit).map((finding) => ({
      finding_id: finding.finding_id,
      detector_id: finding.detector_id,
      title: finding.title,
      summary: finding.summary,
      category: finding.category,
      severity_score: finding.severity_score,
      confidence_score: finding.confidence_score,
      created_at_ms: finding.created_at_ms,
    }));
  }

  async uiResolveBundleInspectEvidence(
    inspectId: string,
    evidenceRefId: string,
  ): Promise<UiBundleInspectEvidenceResolveResultV1 | null> {
    if (!this.bundleInspects.has(inspectId)) {
      throw new Error(`inspect ${inspectId} not found`);
    }
    return {
      inspect_id: inspectId,
      evidence_ref_id: evidenceRefId,
      kind: mockEvidenceResolution.kind,
      target_id: mockEvidenceResolution.target_id,
      exact_pointer_found: mockEvidenceResolution.exact_pointer_found,
      fallback_reason: mockEvidenceResolution.fallback_reason,
      container_json: mockEvidenceResolution.container_json,
      highlighted_value: mockEvidenceResolution.highlighted_value,
    };
  }

  async uiCloseBundleInspect(inspectId: string): Promise<void> {
    this.bundleInspects.delete(inspectId);
  }

  async uiGetDiagnostics(_sessionId: string | null): Promise<UiDiagnosticsSnapshotV1> {
    return mockDiagnostics;
  }

  async uiGetBridgeDiagnostics(
    _sessionId: string | null,
    limit: number,
  ): Promise<UiDiagnosticEntryV1[]> {
    return mockDiagnostics.diagnostics.slice(0, limit);
  }

  async uiGetReliabilitySnapshot(_windowMs: number): Promise<UiReliabilitySnapshotV1> {
    return {
      window: {
        window_ms: 86_400_000,
        from_ms: 1_729_000_000_000,
        to_ms: 1_729_086_400_000,
        totals_by_key: {
          ws_disconnect_count: 2,
          ws_reconnect_count: 2,
          capture_drop_count: 1,
          capture_limit_count: 0,
          command_timeout_count: 0,
          session_pipeline_fail_count: 0,
          permission_denied_count: 0,
          already_attached_count: 0,
        },
      },
      recent_samples: [
        {
          metric_id: 'met_mock_1',
          session_id: 'sess_mock_001',
          source: 'ws_bridge',
          metric_key: 'ws_disconnect_count',
          metric_value: 1,
          labels_json: { reason: 'closed' },
          ts_ms: 1_729_050_000_000,
        },
      ],
    };
  }

  async uiListReliabilitySeries(
    metricKey: ReliabilityMetricKeyV1,
    fromMs: number,
    toMs: number,
    bucketMs: number,
  ): Promise<UiReliabilitySeriesPointV1[]> {
    const points: UiReliabilitySeriesPointV1[] = [];
    const safeBucket = Math.max(1, bucketMs);
    for (let bucketStart = fromMs; bucketStart <= toMs; bucketStart += safeBucket) {
      points.push({
        metric_key: metricKey,
        bucket_start_ms: bucketStart,
        metric_value: bucketStart === fromMs ? 1 : 0,
      });
    }
    return points;
  }

  async uiStartPerfRun(runKind: string, inputRef: string): Promise<UiStartPerfRunResultV1> {
    const perfRunId = `prf_mock_${this.perfRuns.length + 1}`;
    const startedAt = 1_729_210_000_000 + this.perfRuns.length;
    this.perfRuns.unshift({
      perf_run_id: perfRunId,
      run_kind: runKind,
      status: 'completed',
      input_ref: inputRef,
      started_at_ms: startedAt,
      completed_at_ms: startedAt + 1_000,
      error_code: null,
      error_message: null,
    });
    if (runKind.includes('24h')) {
      this.perfAnomalies.unshift({
        anomaly_id: `anm_mock_${this.perfAnomalies.length + 1}`,
        run_kind: runKind,
        bucket_start_ms: startedAt,
        metric_name: 'drift_pct',
        severity: 'medium',
        score: 3.6,
        baseline_value: 10,
        observed_value: 14,
        details_json: { source: 'mad_zscore', window: 20 },
        created_at_ms: startedAt + 1,
      });
    }
    return {
      perf_run_id: perfRunId,
      status: 'completed',
      summary_json: {
        run_kind: runKind,
        throughput_events_per_s: 2500,
        drift_pct: 0,
      },
      error_message: null,
    };
  }

  async uiListPerfRuns(limit: number): Promise<UiPerfRunListItemV1[]> {
    return this.perfRuns.slice(0, limit);
  }

  async uiStartEnduranceRun(runKind: string): Promise<UiStartPerfRunResultV1> {
    return this.uiStartPerfRun(runKind, runKind);
  }

  async uiListPerfTrends(runKind: string, limit: number): Promise<UiPerfTrendPointV1[]> {
    const points: UiPerfTrendPointV1[] = [];
    const now = 1_729_400_000_000;
    for (let index = 0; index < Math.min(limit, 4); index += 1) {
      points.push({
        run_kind: runKind,
        bucket_start_ms: now - (3 - index) * 3_600_000,
        metric_name: 'drift_pct',
        metric_value: index * 4,
        baseline_value: 0,
        trend_delta_pct: index * 4,
        budget_result: index >= 3 ? 'warn' : 'pass',
      });
    }
    return points;
  }

  async uiGetTelemetrySettings(): Promise<UiTelemetrySettingsV1> {
    return this.telemetrySettings;
  }

  async uiSetTelemetrySettings(settings: UiTelemetrySettingsV1): Promise<UiTelemetrySettingsV1> {
    this.telemetrySettings = settings;
    return this.telemetrySettings;
  }

  async uiRunTelemetryExport(
    fromMs: number | null,
    toMs: number | null,
  ): Promise<UiTelemetryExportResultV1> {
    const created = 1_729_410_000_000 + this.telemetryExports.length;
    const run: TelemetryExportRunV1 = {
      export_run_id: `tex_mock_${this.telemetryExports.length + 1}`,
      status: 'completed',
      from_ms: fromMs ?? created - 86_400_000,
      to_ms: toMs ?? created,
      sample_count: 10,
      redacted_count: 2,
      payload_sha256: 'mock_payload_hash',
      created_at_ms: created,
      completed_at_ms: created + 1,
      error_code: null,
      error_message: null,
    };
    this.telemetryExports.unshift(run);
    return { run };
  }

  async uiListTelemetryExports(limit: number): Promise<TelemetryExportRunV1[]> {
    return this.telemetryExports.slice(0, limit);
  }

  async uiRunTelemetryAudit(exportRunId: string | null): Promise<UiRunTelemetryAuditResultV1> {
    const run =
      this.telemetryExports.find((item) => item.export_run_id === exportRunId) ??
      this.telemetryExports[0];
    const audit: TelemetryAuditRunV1 = {
      audit_id: `aud_mock_${this.telemetryAudits.length + 1}`,
      export_run_id: run?.export_run_id ?? null,
      status: run?.redacted_count ? 'warn' : 'pass',
      violations_count: 0,
      violations_json: [],
      payload_sha256: run?.payload_sha256 ?? null,
      created_at_ms: 1_729_520_000_000 + this.telemetryAudits.length,
    };
    this.telemetryAudits.unshift(audit);
    return { run: audit };
  }

  async uiListTelemetryAudits(limit: number): Promise<TelemetryAuditRunV1[]> {
    return this.telemetryAudits.slice(0, limit);
  }

  async uiListPerfAnomalies(
    runKind: string | null,
    limit: number,
  ): Promise<UiListPerfAnomaliesItemV1[]> {
    const rows = runKind
      ? this.perfAnomalies.filter((row) => row.run_kind === runKind)
      : this.perfAnomalies;
    return rows.slice(0, limit);
  }

  async uiResolveEvidence(evidenceRefId: string): Promise<UiEvidenceResolveResultV1 | null> {
    return evidenceRefId === 'evr_mock_1' ? mockEvidenceResolution : null;
  }

  async uiListTabs(): Promise<TabDescriptorV1[]> {
    return [
      {
        tab_id: 42,
        window_id: 7,
        url: 'https://example.com/',
        title: 'Example',
        active: true,
      },
    ];
  }

  async uiStartCapture(
    tabId: number,
    privacyMode: RedactionLevel,
    sessionId: string,
  ): Promise<EvtSessionStartedPayload> {
    return {
      session_id: sessionId,
      tab_id: tabId,
      privacy_mode: privacyMode,
      started_at_ms: Date.now(),
    };
  }

  async uiStopCapture(sessionId: string): Promise<EvtSessionEndedPayload> {
    return { session_id: sessionId, ended_at_ms: Date.now() };
  }

  async uiSetUiCapture(enabled: boolean): Promise<EvtHelloPayload> {
    return {
      extension_version: '0.1.0',
      protocol_version: 1,
      connected: this.pairingState.connected,
      consent_enabled: true,
      ui_capture_enabled: enabled,
      active_session_id: null,
      pairing_state: this.pairingState.state,
      trusted_device_id: this.pairingState.trusted_device_id ?? undefined,
    };
  }

  async uiGetPairingState(): Promise<UiPairingStateV1> {
    return this.pairingState;
  }

  async uiPairingDiscover(deviceId: string, _browserLabel: string): Promise<UiPairingStateV1> {
    this.pairingState = {
      ...this.pairingState,
      state: 'discovering',
      trusted_device_id: deviceId,
    };
    return this.pairingState;
  }

  async uiPairingApprove(deviceId: string, _browserLabel: string): Promise<UiPairingStateV1> {
    this.pairingState = {
      ...this.pairingState,
      state: 'paired',
      trusted_device_id: deviceId,
      connected: true,
    };
    return this.pairingState;
  }

  async uiPairingRevoke(_deviceId: string): Promise<UiPairingStateV1> {
    this.pairingState = {
      ...this.pairingState,
      state: 'not_paired',
      trusted_device_id: null,
      connected: false,
    };
    return this.pairingState;
  }

  async uiLaunchOrFocusDesktop(): Promise<UiLaunchDesktopResultV1> {
    return {
      launched: true,
      method: 'mock',
      message: 'Desktop launch simulated in mock mode.',
    };
  }

  async uiGetRetentionSettings(): Promise<UiRetentionSettingsV1> {
    return { policy: this.retentionPolicy };
  }

  async uiSetRetentionSettings(policy: RetentionPolicyV1): Promise<UiRetentionSettingsV1> {
    this.retentionPolicy = { ...policy };
    return { policy: this.retentionPolicy };
  }

  async uiRunRetention(mode: RetentionRunModeV1): Promise<UiRetentionRunResultV1> {
    return {
      report: {
        run_id: `rrn_mock_${mode}`,
        mode,
        evaluated_sessions: this.sessions.length,
        candidate_sessions: 0,
        deleted_sessions: 0,
        skipped_running_sessions: 0,
        failed_sessions: 0,
        started_at_ms: 1_729_001_200_000,
        finished_at_ms: 1_729_001_200_000,
      },
      deleted: [],
    };
  }

  async uiDeleteSession(sessionId: string): Promise<UiDeleteSessionResultV1> {
    return {
      result: {
        session_id: sessionId,
        db_deleted: false,
        files_deleted: 0,
        missing_files: [],
        blocked_paths: [],
        errors: ['delete_blocked_running_session'],
      },
    };
  }
}

export function createDesktopClient(): DesktopClient {
  if (isLikelyTauriRuntime()) {
    return new TauriDesktopClient();
  }
  return new MockDesktopClient();
}
