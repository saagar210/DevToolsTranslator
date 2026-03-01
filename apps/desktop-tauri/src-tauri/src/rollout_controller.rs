#![forbid(unsafe_code)]

use dtt_core::{
    ReleaseHealthMetricV1, ReleaseHealthScorecardV1, RolloutControllerActionV1,
    RolloutGateReasonV1, RolloutHealthStatusV1, RolloutStageV1,
};
use serde_json::json;

pub const MIN_STAGE_SOAK_MS: i64 = 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutControllerInput {
    pub scope: String,
    pub channel: String,
    pub version: String,
    pub stage: RolloutStageV1,
    pub stage_started_at_ms: i64,
    pub now_ms: i64,
    pub manual_smoke_ready: bool,
    pub compliance_failed: bool,
    pub telemetry_audit_failed: bool,
    pub anomaly_budget_failed: bool,
    pub incident_budget_failed: bool,
    pub signature_verified: bool,
    pub require_signature: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutControllerDecision {
    pub action: RolloutControllerActionV1,
    pub status: RolloutHealthStatusV1,
    pub gate_reasons: Vec<RolloutGateReasonV1>,
    pub soak_remaining_ms: i64,
    pub scorecard: ReleaseHealthScorecardV1,
}

pub fn next_stage(stage: RolloutStageV1) -> Option<RolloutStageV1> {
    match stage {
        RolloutStageV1::Pct5 => Some(RolloutStageV1::Pct25),
        RolloutStageV1::Pct25 => Some(RolloutStageV1::Pct50),
        RolloutStageV1::Pct50 => Some(RolloutStageV1::Pct100),
        RolloutStageV1::Pct100 => None,
    }
}

pub fn evaluate(input: &RolloutControllerInput) -> RolloutControllerDecision {
    let stage_age_ms = input.now_ms.saturating_sub(input.stage_started_at_ms);
    let soak_remaining_ms = (MIN_STAGE_SOAK_MS - stage_age_ms).max(0);

    let mut gate_reasons = Vec::new();
    if !input.manual_smoke_ready {
        gate_reasons.push(RolloutGateReasonV1::ManualSmokeMissing);
    }
    if input.compliance_failed {
        gate_reasons.push(RolloutGateReasonV1::ComplianceFailed);
    }
    if input.telemetry_audit_failed {
        gate_reasons.push(RolloutGateReasonV1::TelemetryAuditFailed);
    }
    if input.anomaly_budget_failed {
        gate_reasons.push(RolloutGateReasonV1::AnomalyBudgetFailed);
    }
    if input.incident_budget_failed {
        gate_reasons.push(RolloutGateReasonV1::IncidentBudgetFailed);
    }
    if input.require_signature && !input.signature_verified {
        gate_reasons.push(RolloutGateReasonV1::SignatureInvalid);
    }
    if soak_remaining_ms > 0 {
        gate_reasons.push(RolloutGateReasonV1::SoakIncomplete);
    }

    let mut score = 100.0;
    if gate_reasons.contains(&RolloutGateReasonV1::ManualSmokeMissing) {
        score -= 10.0;
    }
    if gate_reasons.contains(&RolloutGateReasonV1::ComplianceFailed) {
        score -= 40.0;
    }
    if gate_reasons.contains(&RolloutGateReasonV1::TelemetryAuditFailed) {
        score -= 25.0;
    }
    if gate_reasons.contains(&RolloutGateReasonV1::AnomalyBudgetFailed) {
        score -= 20.0;
    }
    if gate_reasons.contains(&RolloutGateReasonV1::IncidentBudgetFailed) {
        score -= 15.0;
    }
    if gate_reasons.contains(&RolloutGateReasonV1::SignatureInvalid) {
        score -= 20.0;
    }
    if gate_reasons.contains(&RolloutGateReasonV1::SoakIncomplete) {
        score -= 5.0;
    }
    if score < 0.0 {
        score = 0.0;
    }

    let blocking_reason_exists =
        gate_reasons.iter().any(|reason| *reason != RolloutGateReasonV1::SoakIncomplete);
    let status = if blocking_reason_exists {
        RolloutHealthStatusV1::Fail
    } else if gate_reasons.contains(&RolloutGateReasonV1::SoakIncomplete) {
        RolloutHealthStatusV1::Warn
    } else {
        RolloutHealthStatusV1::Pass
    };

    let action = if blocking_reason_exists {
        RolloutControllerActionV1::Block
    } else if gate_reasons.contains(&RolloutGateReasonV1::SoakIncomplete) {
        RolloutControllerActionV1::Pause
    } else if next_stage(input.stage).is_some() {
        RolloutControllerActionV1::Advance
    } else {
        RolloutControllerActionV1::Noop
    };

    let metrics = vec![
        metric(
            "manual_smoke",
            if input.manual_smoke_ready {
                RolloutHealthStatusV1::Pass
            } else {
                RolloutHealthStatusV1::Fail
            },
            if input.manual_smoke_ready { 1.0 } else { 0.0 },
        ),
        metric(
            "compliance_failed",
            if input.compliance_failed {
                RolloutHealthStatusV1::Fail
            } else {
                RolloutHealthStatusV1::Pass
            },
            if input.compliance_failed { 1.0 } else { 0.0 },
        ),
        metric(
            "telemetry_audit_failed",
            if input.telemetry_audit_failed {
                RolloutHealthStatusV1::Fail
            } else {
                RolloutHealthStatusV1::Pass
            },
            if input.telemetry_audit_failed { 1.0 } else { 0.0 },
        ),
        metric(
            "anomaly_budget_failed",
            if input.anomaly_budget_failed {
                RolloutHealthStatusV1::Fail
            } else {
                RolloutHealthStatusV1::Pass
            },
            if input.anomaly_budget_failed { 1.0 } else { 0.0 },
        ),
        metric(
            "incident_budget_failed",
            if input.incident_budget_failed {
                RolloutHealthStatusV1::Fail
            } else {
                RolloutHealthStatusV1::Pass
            },
            if input.incident_budget_failed { 1.0 } else { 0.0 },
        ),
        metric(
            "soak_remaining_ms",
            if soak_remaining_ms > 0 {
                RolloutHealthStatusV1::Warn
            } else {
                RolloutHealthStatusV1::Pass
            },
            soak_remaining_ms as f64,
        ),
    ];

    let scorecard = ReleaseHealthScorecardV1 {
        scope: input.scope.clone(),
        channel: input.channel.clone(),
        version: input.version.clone(),
        stage: Some(input.stage),
        overall_status: status,
        score,
        metrics,
        gate_reasons: gate_reasons.clone(),
        created_at_ms: input.now_ms,
    };

    RolloutControllerDecision { action, status, gate_reasons, soak_remaining_ms, scorecard }
}

fn metric(key: &str, status: RolloutHealthStatusV1, observed_value: f64) -> ReleaseHealthMetricV1 {
    ReleaseHealthMetricV1 {
        metric_key: key.to_string(),
        status,
        observed_value,
        threshold_warn: Some(0.0),
        threshold_fail: Some(1.0),
        details_json: json!({"deterministic": true}),
    }
}

#[cfg(test)]
mod tests {
    use super::{evaluate, next_stage, RolloutControllerInput, MIN_STAGE_SOAK_MS};
    use dtt_core::{
        RolloutControllerActionV1, RolloutGateReasonV1, RolloutHealthStatusV1, RolloutStageV1,
    };

    #[test]
    fn stage_ladder_is_fixed() {
        assert_eq!(next_stage(RolloutStageV1::Pct5), Some(RolloutStageV1::Pct25));
        assert_eq!(next_stage(RolloutStageV1::Pct25), Some(RolloutStageV1::Pct50));
        assert_eq!(next_stage(RolloutStageV1::Pct50), Some(RolloutStageV1::Pct100));
        assert_eq!(next_stage(RolloutStageV1::Pct100), None);
    }

    #[test]
    fn evaluate_blocks_for_failing_gate() {
        let decision = evaluate(&RolloutControllerInput {
            scope: "extension".to_string(),
            channel: "chrome_store_public".to_string(),
            version: "1.0.0".to_string(),
            stage: RolloutStageV1::Pct5,
            stage_started_at_ms: 1_000,
            now_ms: 1_000 + MIN_STAGE_SOAK_MS,
            manual_smoke_ready: false,
            compliance_failed: true,
            telemetry_audit_failed: false,
            anomaly_budget_failed: false,
            incident_budget_failed: false,
            signature_verified: true,
            require_signature: false,
        });
        assert_eq!(decision.action, RolloutControllerActionV1::Block);
        assert_eq!(decision.status, RolloutHealthStatusV1::Fail);
        assert!(decision.gate_reasons.contains(&RolloutGateReasonV1::ManualSmokeMissing));
        assert!(decision.gate_reasons.contains(&RolloutGateReasonV1::ComplianceFailed));
    }

    #[test]
    fn evaluate_advances_when_gates_pass_and_soak_complete() {
        let decision = evaluate(&RolloutControllerInput {
            scope: "updater".to_string(),
            channel: "public_stable".to_string(),
            version: "1.0.0".to_string(),
            stage: RolloutStageV1::Pct25,
            stage_started_at_ms: 1_000,
            now_ms: 1_000 + MIN_STAGE_SOAK_MS,
            manual_smoke_ready: true,
            compliance_failed: false,
            telemetry_audit_failed: false,
            anomaly_budget_failed: false,
            incident_budget_failed: false,
            signature_verified: true,
            require_signature: true,
        });
        assert_eq!(decision.action, RolloutControllerActionV1::Advance);
        assert_eq!(decision.status, RolloutHealthStatusV1::Pass);
        assert_eq!(decision.soak_remaining_ms, 0);
    }
}
