#![cfg(feature = "desktop_shell")]

use crate::DesktopUiFacade;
use dtt_core::{
    EvtHelloPayload, EvtSessionEndedPayload, EvtSessionStartedPayload, ExportProfileV1,
    RedactionLevel, ReleaseArtifactV1, ReleaseChannelV1, ReleaseHealthScorecardV1,
    ReleasePlatformV1, ReliabilityMetricKeyV1, RetentionPolicyV1, RetentionRunModeV1,
    RolloutStageV1, TabDescriptorV1, TelemetryAuditRunV1, TelemetryExportRunV1,
    UiAdvanceExtensionRolloutStageResultV1, UiAdvanceUpdateRolloutResultV1, UiApplyUpdateResultV1,
    UiBundleInspectEvidenceResolveResultV1, UiBundleInspectFindingV1, UiBundleInspectOpenResultV1,
    UiBundleInspectOverviewV1, UiCheckForUpdateResultV1, UiConsoleRowV1, UiDeleteSessionResultV1,
    UiDiagnosticEntryV1, UiDiagnosticsSnapshotV1, UiEvaluateExtensionRolloutStageResultV1,
    UiEvaluateUpdateRolloutResultV1, UiEvidenceResolveResultV1, UiExportCapabilityV1,
    UiExportListItemV1, UiExtensionComplianceSnapshotV1, UiFindingCardV1,
    UiGetComplianceEvidencePackResultV1, UiLaunchDesktopResultV1,
    UiListComplianceEvidencePacksItemV1, UiListExtensionRolloutsItemV1, UiListPerfAnomaliesItemV1,
    UiNetworkRowV1, UiOpenExportFolderResultV1, UiPairingStateV1, UiPerfRunListItemV1,
    UiPerfTrendPointV1, UiReleaseListItemV1, UiReleasePromotionResultV1,
    UiReliabilitySeriesPointV1, UiReliabilitySnapshotV1, UiRetentionRunResultV1,
    UiRetentionSettingsV1, UiRunTelemetryAuditResultV1, UiSessionListItemV1, UiSessionOverviewV1,
    UiSigningSnapshotV1, UiStartExportResultV1, UiStartExtensionPublicRolloutResultV1,
    UiStartPerfRunResultV1, UiStartReleaseResultV1, UiTelemetryExportResultV1,
    UiTelemetrySettingsV1, UiTimelineBundleV1, UiUpdateRolloutSnapshotV1, UiValidateExportResultV1,
    UpdateChannelV1,
};
use std::sync::Mutex;

pub type SharedUiFacade = Mutex<DesktopUiFacade>;

fn with_facade<T>(
    state: tauri::State<'_, SharedUiFacade>,
    f: impl FnOnce(&DesktopUiFacade) -> Result<T, crate::UiCommandError>,
) -> Result<T, String> {
    let guard = state.lock().map_err(|_| "ui state lock poisoned".to_string())?;
    f(&guard).map_err(|error| format!("{}: {}", error.code(), error))
}

#[tauri::command]
pub fn ui_list_tabs(
    state: tauri::State<'_, SharedUiFacade>,
) -> Result<Vec<TabDescriptorV1>, String> {
    with_facade(state, |facade| facade.ui_list_tabs())
}

#[tauri::command]
pub fn ui_start_capture(
    state: tauri::State<'_, SharedUiFacade>,
    tab_id: i64,
    privacy_mode: RedactionLevel,
    session_id: String,
) -> Result<EvtSessionStartedPayload, String> {
    with_facade(state, |facade| facade.ui_start_capture(tab_id, privacy_mode, &session_id))
}

#[tauri::command]
pub fn ui_stop_capture(
    state: tauri::State<'_, SharedUiFacade>,
    session_id: String,
) -> Result<EvtSessionEndedPayload, String> {
    with_facade(state, |facade| facade.ui_stop_capture(&session_id))
}

#[tauri::command]
pub fn ui_set_ui_capture(
    state: tauri::State<'_, SharedUiFacade>,
    enabled: bool,
) -> Result<EvtHelloPayload, String> {
    with_facade(state, |facade| facade.ui_set_ui_capture(enabled))
}

#[tauri::command]
pub fn ui_get_pairing_state(
    state: tauri::State<'_, SharedUiFacade>,
) -> Result<UiPairingStateV1, String> {
    with_facade(state, DesktopUiFacade::ui_get_pairing_state)
}

#[tauri::command]
pub fn ui_pairing_discover(
    state: tauri::State<'_, SharedUiFacade>,
    device_id: String,
    browser_label: String,
) -> Result<UiPairingStateV1, String> {
    with_facade(state, |facade| facade.ui_pairing_discover(&device_id, &browser_label))
}

#[tauri::command]
pub fn ui_pairing_approve(
    state: tauri::State<'_, SharedUiFacade>,
    device_id: String,
    browser_label: String,
) -> Result<UiPairingStateV1, String> {
    with_facade(state, |facade| facade.ui_pairing_approve(&device_id, &browser_label))
}

#[tauri::command]
pub fn ui_pairing_revoke(
    state: tauri::State<'_, SharedUiFacade>,
    device_id: String,
) -> Result<UiPairingStateV1, String> {
    with_facade(state, |facade| facade.ui_pairing_revoke(&device_id))
}

#[tauri::command]
pub fn ui_launch_or_focus_desktop(
    state: tauri::State<'_, SharedUiFacade>,
) -> Result<UiLaunchDesktopResultV1, String> {
    with_facade(state, DesktopUiFacade::ui_launch_or_focus_desktop)
}

#[tauri::command]
pub fn ui_get_sessions(
    state: tauri::State<'_, SharedUiFacade>,
    limit: Option<usize>,
) -> Result<Vec<UiSessionListItemV1>, String> {
    let limit = limit.unwrap_or(200);
    with_facade(state, |facade| facade.ui_get_sessions(limit))
}

#[tauri::command]
pub fn ui_get_session_overview(
    state: tauri::State<'_, SharedUiFacade>,
    session_id: String,
) -> Result<Option<UiSessionOverviewV1>, String> {
    with_facade(state, |facade| facade.ui_get_session_overview(&session_id))
}

#[tauri::command]
pub fn ui_get_timeline(
    state: tauri::State<'_, SharedUiFacade>,
    session_id: String,
) -> Result<UiTimelineBundleV1, String> {
    with_facade(state, |facade| facade.ui_get_timeline(&session_id))
}

#[tauri::command]
pub fn ui_get_network(
    state: tauri::State<'_, SharedUiFacade>,
    session_id: String,
) -> Result<Vec<UiNetworkRowV1>, String> {
    with_facade(state, |facade| facade.ui_get_network(&session_id))
}

#[tauri::command]
pub fn ui_get_console(
    state: tauri::State<'_, SharedUiFacade>,
    session_id: String,
) -> Result<Vec<UiConsoleRowV1>, String> {
    with_facade(state, |facade| facade.ui_get_console(&session_id))
}

#[tauri::command]
pub fn ui_get_findings(
    state: tauri::State<'_, SharedUiFacade>,
    session_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<UiFindingCardV1>, String> {
    with_facade(state, |facade| facade.ui_get_findings(session_id.as_deref(), limit.unwrap_or(100)))
}

#[tauri::command]
pub fn ui_get_exports(
    state: tauri::State<'_, SharedUiFacade>,
    session_id: String,
) -> Result<UiExportCapabilityV1, String> {
    with_facade(state, |facade| facade.ui_get_exports(&session_id))
}

#[tauri::command]
pub fn ui_start_export(
    state: tauri::State<'_, SharedUiFacade>,
    session_id: String,
    profile: ExportProfileV1,
    output_dir: Option<String>,
) -> Result<UiStartExportResultV1, String> {
    with_facade(state, |facade| facade.ui_start_export(&session_id, profile, output_dir.as_deref()))
}

#[tauri::command]
pub fn ui_list_exports(
    state: tauri::State<'_, SharedUiFacade>,
    session_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<UiExportListItemV1>, String> {
    with_facade(state, |facade| facade.ui_list_exports(session_id.as_deref(), limit.unwrap_or(200)))
}

#[tauri::command]
pub fn ui_validate_export(
    state: tauri::State<'_, SharedUiFacade>,
    export_id: String,
) -> Result<UiValidateExportResultV1, String> {
    with_facade(state, |facade| facade.ui_validate_export(&export_id))
}

#[tauri::command]
pub fn ui_open_export_folder(
    state: tauri::State<'_, SharedUiFacade>,
    export_id: Option<String>,
) -> Result<UiOpenExportFolderResultV1, String> {
    with_facade(state, |facade| facade.ui_open_export_folder(export_id.as_deref()))
}

#[tauri::command]
pub fn ui_start_release(
    state: tauri::State<'_, SharedUiFacade>,
    channel: ReleaseChannelV1,
    version: String,
    notes_md: String,
    dry_run: bool,
) -> Result<UiStartReleaseResultV1, String> {
    with_facade(state, |facade| facade.ui_start_release(channel, &version, &notes_md, dry_run))
}

#[tauri::command]
pub fn ui_list_releases(
    state: tauri::State<'_, SharedUiFacade>,
    limit: Option<usize>,
) -> Result<Vec<UiReleaseListItemV1>, String> {
    with_facade(state, |facade| facade.ui_list_releases(limit.unwrap_or(100)))
}

#[tauri::command]
pub fn ui_get_release_artifacts_by_platform(
    state: tauri::State<'_, SharedUiFacade>,
    platform: ReleasePlatformV1,
    limit_runs: Option<usize>,
) -> Result<Vec<ReleaseArtifactV1>, String> {
    with_facade(state, |facade| {
        facade.ui_get_release_artifacts_by_platform(platform, limit_runs.unwrap_or(100))
    })
}

#[tauri::command]
pub fn ui_start_release_matrix(
    state: tauri::State<'_, SharedUiFacade>,
    channel: ReleaseChannelV1,
    version: String,
    notes_md: String,
    dry_run: bool,
) -> Result<UiStartReleaseResultV1, String> {
    with_facade(state, |facade| {
        facade.ui_start_release_matrix(channel, &version, &notes_md, dry_run)
    })
}

#[tauri::command]
pub fn ui_start_release_promotion(
    state: tauri::State<'_, SharedUiFacade>,
    channel: ReleaseChannelV1,
    promote_from_internal_run_id: String,
    notes_md: String,
    dry_run: bool,
) -> Result<UiReleasePromotionResultV1, String> {
    with_facade(state, |facade| {
        facade.ui_start_release_promotion(
            channel,
            &promote_from_internal_run_id,
            &notes_md,
            dry_run,
        )
    })
}

#[tauri::command]
pub fn ui_get_signing_snapshot(
    state: tauri::State<'_, SharedUiFacade>,
    run_id: String,
) -> Result<UiSigningSnapshotV1, String> {
    with_facade(state, |facade| facade.ui_get_signing_snapshot(&run_id))
}

#[tauri::command]
pub fn ui_start_extension_public_rollout(
    state: tauri::State<'_, SharedUiFacade>,
    version: String,
    stage: RolloutStageV1,
    notes_md: String,
    dry_run: bool,
) -> Result<UiStartExtensionPublicRolloutResultV1, String> {
    with_facade(state, |facade| {
        facade.ui_start_extension_public_rollout(&version, stage, &notes_md, dry_run)
    })
}

#[tauri::command]
pub fn ui_list_extension_rollouts(
    state: tauri::State<'_, SharedUiFacade>,
    limit: Option<usize>,
) -> Result<Vec<UiListExtensionRolloutsItemV1>, String> {
    with_facade(state, |facade| facade.ui_list_extension_rollouts(limit.unwrap_or(100)))
}

#[tauri::command]
pub fn ui_get_extension_compliance_snapshot(
    state: tauri::State<'_, SharedUiFacade>,
    rollout_id: Option<String>,
) -> Result<UiExtensionComplianceSnapshotV1, String> {
    with_facade(state, |facade| facade.ui_get_extension_compliance_snapshot(rollout_id.as_deref()))
}

#[tauri::command]
pub fn ui_check_for_updates(
    state: tauri::State<'_, SharedUiFacade>,
    channel: UpdateChannelV1,
    install_id: String,
    current_version: String,
) -> Result<UiCheckForUpdateResultV1, String> {
    with_facade(state, |facade| facade.ui_check_for_updates(channel, &install_id, &current_version))
}

#[tauri::command]
pub fn ui_apply_update(
    state: tauri::State<'_, SharedUiFacade>,
    channel: UpdateChannelV1,
    install_id: String,
    current_version: String,
) -> Result<UiApplyUpdateResultV1, String> {
    with_facade(state, |facade| facade.ui_apply_update(channel, &install_id, &current_version))
}

#[tauri::command]
pub fn ui_get_update_rollout_snapshot(
    state: tauri::State<'_, SharedUiFacade>,
    channel: UpdateChannelV1,
) -> Result<UiUpdateRolloutSnapshotV1, String> {
    with_facade(state, |facade| facade.ui_get_update_rollout_snapshot(channel))
}

#[tauri::command]
pub fn ui_evaluate_extension_rollout_stage(
    state: tauri::State<'_, SharedUiFacade>,
    version: String,
    stage: RolloutStageV1,
) -> Result<UiEvaluateExtensionRolloutStageResultV1, String> {
    with_facade(state, |facade| facade.ui_evaluate_extension_rollout_stage(&version, stage))
}

#[tauri::command]
pub fn ui_advance_extension_rollout_stage(
    state: tauri::State<'_, SharedUiFacade>,
    version: String,
    from_stage: RolloutStageV1,
    to_stage: RolloutStageV1,
    dry_run: bool,
) -> Result<UiAdvanceExtensionRolloutStageResultV1, String> {
    with_facade(state, |facade| {
        facade.ui_advance_extension_rollout_stage(&version, from_stage, to_stage, dry_run)
    })
}

#[tauri::command]
pub fn ui_evaluate_update_rollout(
    state: tauri::State<'_, SharedUiFacade>,
    channel: UpdateChannelV1,
    version: String,
    stage: RolloutStageV1,
) -> Result<UiEvaluateUpdateRolloutResultV1, String> {
    with_facade(state, |facade| facade.ui_evaluate_update_rollout(channel, &version, stage))
}

#[tauri::command]
pub fn ui_advance_update_rollout(
    state: tauri::State<'_, SharedUiFacade>,
    channel: UpdateChannelV1,
    version: String,
    from_stage: RolloutStageV1,
    to_stage: RolloutStageV1,
    dry_run: bool,
) -> Result<UiAdvanceUpdateRolloutResultV1, String> {
    with_facade(state, |facade| {
        facade.ui_advance_update_rollout(channel, &version, from_stage, to_stage, dry_run)
    })
}

#[tauri::command]
pub fn ui_get_release_health_scorecard(
    state: tauri::State<'_, SharedUiFacade>,
    version: String,
    updater_channel: UpdateChannelV1,
) -> Result<ReleaseHealthScorecardV1, String> {
    with_facade(state, |facade| facade.ui_get_release_health_scorecard(&version, updater_channel))
}

#[tauri::command]
pub fn ui_get_compliance_evidence_pack(
    state: tauri::State<'_, SharedUiFacade>,
    kind: String,
    channel: String,
    version: String,
    stage: Option<RolloutStageV1>,
) -> Result<UiGetComplianceEvidencePackResultV1, String> {
    with_facade(state, |facade| {
        facade.ui_get_compliance_evidence_pack(&kind, &channel, &version, stage)
    })
}

#[tauri::command]
pub fn ui_list_compliance_evidence_packs(
    state: tauri::State<'_, SharedUiFacade>,
    kind: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<UiListComplianceEvidencePacksItemV1>, String> {
    with_facade(state, |facade| {
        facade.ui_list_compliance_evidence_packs(kind.as_deref(), limit.unwrap_or(100))
    })
}

#[tauri::command]
pub fn ui_run_rollout_controller_tick(
    state: tauri::State<'_, SharedUiFacade>,
    version: String,
    stage: RolloutStageV1,
    updater_channel: UpdateChannelV1,
) -> Result<ReleaseHealthScorecardV1, String> {
    with_facade(state, |facade| {
        facade.ui_run_rollout_controller_tick(&version, stage, updater_channel)
    })
}

#[tauri::command]
pub fn ui_open_bundle_inspect(
    state: tauri::State<'_, SharedUiFacade>,
    bundle_path: String,
) -> Result<UiBundleInspectOpenResultV1, String> {
    with_facade(state, |facade| facade.ui_open_bundle_inspect(&bundle_path))
}

#[tauri::command]
pub fn ui_get_bundle_inspect_overview(
    state: tauri::State<'_, SharedUiFacade>,
    inspect_id: String,
) -> Result<UiBundleInspectOverviewV1, String> {
    with_facade(state, |facade| facade.ui_get_bundle_inspect_overview(&inspect_id))
}

#[tauri::command]
pub fn ui_list_bundle_inspect_findings(
    state: tauri::State<'_, SharedUiFacade>,
    inspect_id: String,
    limit: Option<usize>,
) -> Result<Vec<UiBundleInspectFindingV1>, String> {
    with_facade(state, |facade| {
        facade.ui_list_bundle_inspect_findings(&inspect_id, limit.unwrap_or(200))
    })
}

#[tauri::command]
pub fn ui_resolve_bundle_inspect_evidence(
    state: tauri::State<'_, SharedUiFacade>,
    inspect_id: String,
    evidence_ref_id: String,
) -> Result<Option<UiBundleInspectEvidenceResolveResultV1>, String> {
    with_facade(state, |facade| {
        facade.ui_resolve_bundle_inspect_evidence(&inspect_id, &evidence_ref_id)
    })
}

#[tauri::command]
pub fn ui_close_bundle_inspect(
    state: tauri::State<'_, SharedUiFacade>,
    inspect_id: String,
) -> Result<(), String> {
    with_facade(state, |facade| facade.ui_close_bundle_inspect(&inspect_id))
}

#[tauri::command]
pub fn ui_get_diagnostics(
    state: tauri::State<'_, SharedUiFacade>,
    session_id: Option<String>,
) -> Result<UiDiagnosticsSnapshotV1, String> {
    with_facade(state, |facade| facade.ui_get_diagnostics(session_id.as_deref()))
}

#[tauri::command]
pub fn ui_get_bridge_diagnostics(
    state: tauri::State<'_, SharedUiFacade>,
    session_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<UiDiagnosticEntryV1>, String> {
    with_facade(state, |facade| {
        facade.ui_get_bridge_diagnostics(session_id.as_deref(), limit.unwrap_or(200))
    })
}

#[tauri::command]
pub fn ui_get_reliability_snapshot(
    state: tauri::State<'_, SharedUiFacade>,
    window_ms: Option<i64>,
) -> Result<UiReliabilitySnapshotV1, String> {
    with_facade(state, |facade| facade.ui_get_reliability_snapshot(window_ms.unwrap_or(86_400_000)))
}

#[tauri::command]
pub fn ui_list_reliability_series(
    state: tauri::State<'_, SharedUiFacade>,
    metric_key: ReliabilityMetricKeyV1,
    from_ms: i64,
    to_ms: i64,
    bucket_ms: Option<i64>,
) -> Result<Vec<UiReliabilitySeriesPointV1>, String> {
    with_facade(state, |facade| {
        facade.ui_list_reliability_series(
            metric_key,
            from_ms,
            to_ms,
            bucket_ms.unwrap_or(3_600_000),
        )
    })
}

#[tauri::command]
pub fn ui_start_perf_run(
    state: tauri::State<'_, SharedUiFacade>,
    run_kind: String,
    input_ref: String,
) -> Result<UiStartPerfRunResultV1, String> {
    with_facade(state, |facade| facade.ui_start_perf_run(&run_kind, &input_ref))
}

#[tauri::command]
pub fn ui_list_perf_runs(
    state: tauri::State<'_, SharedUiFacade>,
    limit: Option<usize>,
) -> Result<Vec<UiPerfRunListItemV1>, String> {
    with_facade(state, |facade| facade.ui_list_perf_runs(limit.unwrap_or(50)))
}

#[tauri::command]
pub fn ui_start_endurance_run(
    state: tauri::State<'_, SharedUiFacade>,
    run_kind: String,
) -> Result<UiStartPerfRunResultV1, String> {
    with_facade(state, |facade| facade.ui_start_endurance_run(&run_kind))
}

#[tauri::command]
pub fn ui_list_perf_trends(
    state: tauri::State<'_, SharedUiFacade>,
    run_kind: String,
    limit: Option<usize>,
) -> Result<Vec<UiPerfTrendPointV1>, String> {
    with_facade(state, |facade| facade.ui_list_perf_trends(&run_kind, limit.unwrap_or(100)))
}

#[tauri::command]
pub fn ui_list_perf_anomalies(
    state: tauri::State<'_, SharedUiFacade>,
    run_kind: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<UiListPerfAnomaliesItemV1>, String> {
    with_facade(state, |facade| {
        facade.ui_list_perf_anomalies(run_kind.as_deref(), limit.unwrap_or(100))
    })
}

#[tauri::command]
pub fn ui_get_telemetry_settings(
    state: tauri::State<'_, SharedUiFacade>,
) -> Result<UiTelemetrySettingsV1, String> {
    with_facade(state, |facade| facade.ui_get_telemetry_settings())
}

#[tauri::command]
pub fn ui_set_telemetry_settings(
    state: tauri::State<'_, SharedUiFacade>,
    settings: UiTelemetrySettingsV1,
) -> Result<UiTelemetrySettingsV1, String> {
    with_facade(state, |facade| facade.ui_set_telemetry_settings(settings))
}

#[tauri::command]
pub fn ui_run_telemetry_export(
    state: tauri::State<'_, SharedUiFacade>,
    from_ms: Option<i64>,
    to_ms: Option<i64>,
) -> Result<UiTelemetryExportResultV1, String> {
    with_facade(state, |facade| facade.ui_run_telemetry_export(from_ms, to_ms))
}

#[tauri::command]
pub fn ui_list_telemetry_exports(
    state: tauri::State<'_, SharedUiFacade>,
    limit: Option<usize>,
) -> Result<Vec<TelemetryExportRunV1>, String> {
    with_facade(state, |facade| facade.ui_list_telemetry_exports(limit.unwrap_or(100)))
}

#[tauri::command]
pub fn ui_run_telemetry_audit(
    state: tauri::State<'_, SharedUiFacade>,
    export_run_id: Option<String>,
) -> Result<UiRunTelemetryAuditResultV1, String> {
    with_facade(state, |facade| facade.ui_run_telemetry_audit(export_run_id.as_deref()))
}

#[tauri::command]
pub fn ui_list_telemetry_audits(
    state: tauri::State<'_, SharedUiFacade>,
    limit: Option<usize>,
) -> Result<Vec<TelemetryAuditRunV1>, String> {
    with_facade(state, |facade| facade.ui_list_telemetry_audits(limit.unwrap_or(100)))
}

#[tauri::command]
pub fn ui_get_retention_settings(
    state: tauri::State<'_, SharedUiFacade>,
) -> Result<UiRetentionSettingsV1, String> {
    with_facade(state, |facade| facade.ui_get_retention_settings())
}

#[tauri::command]
pub fn ui_set_retention_settings(
    state: tauri::State<'_, SharedUiFacade>,
    policy: RetentionPolicyV1,
) -> Result<UiRetentionSettingsV1, String> {
    with_facade(state, |facade| facade.ui_set_retention_settings(policy))
}

#[tauri::command]
pub fn ui_run_retention(
    state: tauri::State<'_, SharedUiFacade>,
    mode: RetentionRunModeV1,
) -> Result<UiRetentionRunResultV1, String> {
    with_facade(state, |facade| facade.ui_run_retention(mode))
}

#[tauri::command]
pub fn ui_delete_session(
    state: tauri::State<'_, SharedUiFacade>,
    session_id: String,
) -> Result<UiDeleteSessionResultV1, String> {
    with_facade(state, |facade| facade.ui_delete_session(&session_id))
}

#[tauri::command]
pub fn ui_resolve_evidence(
    state: tauri::State<'_, SharedUiFacade>,
    evidence_ref_id: String,
) -> Result<Option<UiEvidenceResolveResultV1>, String> {
    with_facade(state, |facade| facade.ui_resolve_evidence(&evidence_ref_id))
}

pub fn build_invoke_handler(
) -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        ui_list_tabs,
        ui_start_capture,
        ui_stop_capture,
        ui_set_ui_capture,
        ui_get_pairing_state,
        ui_pairing_discover,
        ui_pairing_approve,
        ui_pairing_revoke,
        ui_launch_or_focus_desktop,
        ui_get_sessions,
        ui_get_session_overview,
        ui_get_timeline,
        ui_get_network,
        ui_get_console,
        ui_get_findings,
        ui_get_exports,
        ui_start_export,
        ui_list_exports,
        ui_validate_export,
        ui_open_export_folder,
        ui_start_release,
        ui_list_releases,
        ui_get_release_artifacts_by_platform,
        ui_start_release_matrix,
        ui_start_release_promotion,
        ui_get_signing_snapshot,
        ui_start_extension_public_rollout,
        ui_list_extension_rollouts,
        ui_get_extension_compliance_snapshot,
        ui_check_for_updates,
        ui_apply_update,
        ui_get_update_rollout_snapshot,
        ui_evaluate_extension_rollout_stage,
        ui_advance_extension_rollout_stage,
        ui_evaluate_update_rollout,
        ui_advance_update_rollout,
        ui_get_release_health_scorecard,
        ui_get_compliance_evidence_pack,
        ui_list_compliance_evidence_packs,
        ui_run_rollout_controller_tick,
        ui_open_bundle_inspect,
        ui_get_bundle_inspect_overview,
        ui_list_bundle_inspect_findings,
        ui_resolve_bundle_inspect_evidence,
        ui_close_bundle_inspect,
        ui_get_diagnostics,
        ui_get_bridge_diagnostics,
        ui_get_reliability_snapshot,
        ui_list_reliability_series,
        ui_start_perf_run,
        ui_list_perf_runs,
        ui_start_endurance_run,
        ui_list_perf_trends,
        ui_list_perf_anomalies,
        ui_get_telemetry_settings,
        ui_set_telemetry_settings,
        ui_run_telemetry_export,
        ui_list_telemetry_exports,
        ui_run_telemetry_audit,
        ui_list_telemetry_audits,
        ui_get_retention_settings,
        ui_set_retention_settings,
        ui_run_retention,
        ui_delete_session,
        ui_resolve_evidence
    ]
}
