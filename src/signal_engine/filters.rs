// src/signal_engine/filters.rs

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::depth_engine::DepthResult;
use crate::types::PackageCandidate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SignalFilterRejection {
    ExpiredOrTooClose,
    TopCostNotUnderTriggerThreshold,
    DepthNotConfirmed,
    MissingDepthAverageCost,
    DepthAverageCostNotUnderExecutionThreshold,
    ExecutableSizeTooSmall,
    GrossEdgeTooSmall,
    ZeroOrNegativeCapitalRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalFilterOutcome {
    pub accepted: bool,
    pub rejection: Option<SignalFilterRejection>,
    pub message: String,
}

impl SignalFilterOutcome {
    pub fn accepted() -> Self {
        Self {
            accepted: true,
            rejection: None,
            message: "accepted".to_string(),
        }
    }

    pub fn rejected(rejection: SignalFilterRejection, message: impl Into<String>) -> Self {
        Self {
            accepted: false,
            rejection: Some(rejection),
            message: message.into(),
        }
    }
}

pub fn filter_signal_candidate(
    config: &Config,
    candidate: &PackageCandidate,
    top_cost: Decimal,
    depth_result: &DepthResult,
) -> SignalFilterOutcome {
    if candidate.seconds_to_end < config.eval.min_seconds_to_end as i64 {
        return SignalFilterOutcome::rejected(
            SignalFilterRejection::ExpiredOrTooClose,
            format!(
                "seconds_to_end {} is below minimum {}",
                candidate.seconds_to_end, config.eval.min_seconds_to_end
            ),
        );
    }

    let trigger_threshold = Decimal::ONE - config.eval.trigger_buffer;

    if top_cost >= trigger_threshold {
        return SignalFilterOutcome::rejected(
            SignalFilterRejection::TopCostNotUnderTriggerThreshold,
            format!(
                "top_cost {} is not below trigger threshold {}",
                top_cost, trigger_threshold
            ),
        );
    }

    if !depth_result.is_depth_confirmed() {
        return SignalFilterOutcome::rejected(
            SignalFilterRejection::DepthNotConfirmed,
            "depth result is not DEPTH_CONFIRMED_UNDER_1",
        );
    }

    let Some(depth_avg_cost) = depth_result.avg_package_cost else {
        return SignalFilterOutcome::rejected(
            SignalFilterRejection::MissingDepthAverageCost,
            "depth result has no average package cost",
        );
    };

    let execution_threshold = Decimal::ONE - config.eval.execution_buffer;

    if depth_avg_cost >= execution_threshold {
        return SignalFilterOutcome::rejected(
            SignalFilterRejection::DepthAverageCostNotUnderExecutionThreshold,
            format!(
                "depth average cost {} is not below execution threshold {}",
                depth_avg_cost, execution_threshold
            ),
        );
    }

    if depth_result.max_size_under_threshold < config.eval.min_depth_confirmed_size {
        return SignalFilterOutcome::rejected(
            SignalFilterRejection::ExecutableSizeTooSmall,
            format!(
                "executable size {} is below minimum {}",
                depth_result.max_size_under_threshold, config.eval.min_depth_confirmed_size
            ),
        );
    }

    if depth_result.gross_edge < config.eval.min_gross_edge {
        return SignalFilterOutcome::rejected(
            SignalFilterRejection::GrossEdgeTooSmall,
            format!(
                "gross edge {} is below minimum {}",
                depth_result.gross_edge, config.eval.min_gross_edge
            ),
        );
    }

    if depth_result.capital_required <= Decimal::ZERO {
        return SignalFilterOutcome::rejected(
            SignalFilterRejection::ZeroOrNegativeCapitalRequired,
            "capital required is zero or negative",
        );
    }

    SignalFilterOutcome::accepted()
}
