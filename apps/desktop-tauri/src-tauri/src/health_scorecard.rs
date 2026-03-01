#![forbid(unsafe_code)]

use dtt_core::{
    ReleaseHealthMetricV1, ReleaseHealthScorecardV1, RolloutGateReasonV1, RolloutHealthStatusV1,
};
use serde_json::json;

pub fn combine_global(
    channel: &str,
    version: &str,
    created_at_ms: i64,
    extension: Option<&ReleaseHealthScorecardV1>,
    updater: Option<&ReleaseHealthScorecardV1>,
) -> ReleaseHealthScorecardV1 {
    let mut metrics = Vec::<ReleaseHealthMetricV1>::new();
    let mut gate_reasons = Vec::<RolloutGateReasonV1>::new();
    let mut scores = Vec::<f64>::new();

    if let Some(scorecard) = extension {
        scores.push(scorecard.score);
        metrics.extend(scorecard.metrics.clone());
        gate_reasons.extend(scorecard.gate_reasons.clone());
    }
    if let Some(scorecard) = updater {
        scores.push(scorecard.score);
        metrics.extend(scorecard.metrics.clone());
        gate_reasons.extend(scorecard.gate_reasons.clone());
    }

    metrics.sort_by(|left, right| left.metric_key.cmp(&right.metric_key));
    metrics.dedup_by(|left, right| left.metric_key == right.metric_key);
    gate_reasons.sort_by_key(|value| format!("{value:?}"));
    gate_reasons.dedup();

    let score =
        if scores.is_empty() { 0.0 } else { scores.iter().sum::<f64>() / (scores.len() as f64) };

    let overall_status =
        if gate_reasons.iter().any(|reason| *reason != RolloutGateReasonV1::SoakIncomplete) {
            RolloutHealthStatusV1::Fail
        } else if gate_reasons.contains(&RolloutGateReasonV1::SoakIncomplete) {
            RolloutHealthStatusV1::Warn
        } else {
            RolloutHealthStatusV1::Pass
        };

    let mut scorecard = ReleaseHealthScorecardV1 {
        scope: "global".to_string(),
        channel: channel.to_string(),
        version: version.to_string(),
        stage: None,
        overall_status,
        score,
        metrics,
        gate_reasons,
        created_at_ms,
    };

    scorecard.metrics.push(ReleaseHealthMetricV1 {
        metric_key: "components_count".to_string(),
        status: RolloutHealthStatusV1::Pass,
        observed_value: if extension.is_some() && updater.is_some() { 2.0 } else { 1.0 },
        threshold_warn: None,
        threshold_fail: None,
        details_json: json!({
            "has_extension": extension.is_some(),
            "has_updater": updater.is_some(),
        }),
    });

    scorecard
}

#[cfg(test)]
mod tests {
    use super::combine_global;
    use dtt_core::{
        ReleaseHealthMetricV1, ReleaseHealthScorecardV1, RolloutGateReasonV1, RolloutHealthStatusV1,
    };
    use serde_json::json;

    #[test]
    fn combine_global_merges_scores_and_reasons() {
        let extension = ReleaseHealthScorecardV1 {
            scope: "extension".to_string(),
            channel: "chrome_store_public".to_string(),
            version: "1.0.0".to_string(),
            stage: None,
            overall_status: RolloutHealthStatusV1::Warn,
            score: 90.0,
            metrics: vec![ReleaseHealthMetricV1 {
                metric_key: "a".to_string(),
                status: RolloutHealthStatusV1::Pass,
                observed_value: 1.0,
                threshold_warn: None,
                threshold_fail: None,
                details_json: json!({}),
            }],
            gate_reasons: vec![RolloutGateReasonV1::SoakIncomplete],
            created_at_ms: 1,
        };
        let updater = ReleaseHealthScorecardV1 {
            scope: "updater".to_string(),
            channel: "public_stable".to_string(),
            version: "1.0.0".to_string(),
            stage: None,
            overall_status: RolloutHealthStatusV1::Fail,
            score: 40.0,
            metrics: vec![ReleaseHealthMetricV1 {
                metric_key: "b".to_string(),
                status: RolloutHealthStatusV1::Fail,
                observed_value: 1.0,
                threshold_warn: None,
                threshold_fail: None,
                details_json: json!({}),
            }],
            gate_reasons: vec![RolloutGateReasonV1::SignatureInvalid],
            created_at_ms: 1,
        };
        let combined =
            combine_global("public_stable", "1.0.0", 10, Some(&extension), Some(&updater));
        assert_eq!(combined.overall_status, RolloutHealthStatusV1::Fail);
        assert!((combined.score - 65.0).abs() < 0.01);
        assert!(combined.gate_reasons.contains(&RolloutGateReasonV1::SignatureInvalid));
    }
}
