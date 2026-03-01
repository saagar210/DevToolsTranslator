#![forbid(unsafe_code)]

use dtt_core::{RolloutStageV1, UpdateChannelV1, UpdateEligibilityV1};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EligibilityDecision {
    pub bucket: u8,
    pub eligibility: UpdateEligibilityV1,
}

pub fn rollout_pct_for_stage(stage: RolloutStageV1) -> u8 {
    match stage {
        RolloutStageV1::Pct5 => 5,
        RolloutStageV1::Pct25 => 25,
        RolloutStageV1::Pct50 => 50,
        RolloutStageV1::Pct100 => 100,
    }
}

pub fn eligibility_for_install(
    install_id: &str,
    channel: UpdateChannelV1,
    version: &str,
    rollout_pct: u8,
    signature_verified: bool,
) -> EligibilityDecision {
    if !signature_verified {
        return EligibilityDecision {
            bucket: 0,
            eligibility: UpdateEligibilityV1::BlockedSignature,
        };
    }
    if install_id.trim().is_empty() || version.trim().is_empty() {
        return EligibilityDecision { bucket: 0, eligibility: UpdateEligibilityV1::BlockedPolicy };
    }
    let bucket = deterministic_bucket(install_id, channel, version);
    let eligibility = if bucket < rollout_pct {
        UpdateEligibilityV1::Eligible
    } else {
        UpdateEligibilityV1::DeferredRollout
    };
    EligibilityDecision { bucket, eligibility }
}

pub fn deterministic_bucket(install_id: &str, channel: UpdateChannelV1, version: &str) -> u8 {
    let channel_raw = match channel {
        UpdateChannelV1::InternalBeta => "internal_beta",
        UpdateChannelV1::StagedPublicPrerelease => "staged_public_prerelease",
        UpdateChannelV1::PublicStable => "public_stable",
    };
    let input = format!("{install_id}:{channel_raw}:{version}");
    let digest = Sha256::digest(input.as_bytes());
    let mut first_eight = [0_u8; 8];
    first_eight.copy_from_slice(&digest[..8]);
    let raw = u64::from_be_bytes(first_eight);
    u8::try_from(raw % 100).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{deterministic_bucket, eligibility_for_install};
    use dtt_core::{UpdateChannelV1, UpdateEligibilityV1};

    #[test]
    fn deterministic_bucket_is_stable() {
        let left =
            deterministic_bucket("install-123", UpdateChannelV1::StagedPublicPrerelease, "1.2.3");
        let right =
            deterministic_bucket("install-123", UpdateChannelV1::StagedPublicPrerelease, "1.2.3");
        assert_eq!(left, right);
    }

    #[test]
    fn eligibility_respects_signature_and_rollout_pct() {
        let blocked = eligibility_for_install(
            "install-123",
            UpdateChannelV1::StagedPublicPrerelease,
            "1.2.3",
            100,
            false,
        );
        assert_eq!(blocked.eligibility, UpdateEligibilityV1::BlockedSignature);

        let eligible = eligibility_for_install(
            "install-123",
            UpdateChannelV1::StagedPublicPrerelease,
            "1.2.3",
            100,
            true,
        );
        assert_eq!(eligible.eligibility, UpdateEligibilityV1::Eligible);
    }
}
