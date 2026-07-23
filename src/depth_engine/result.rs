// src/depth_engine/result.rs

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DepthLeg {
    Short,
    Long,
    Both,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepthStatus {
    DepthConfirmedUnderOne,
    TopBookOnlyUnderOne,
    InsufficientDepth,
    NoUsableAsks,
    InvalidTargetSize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepthResult {
    pub status: DepthStatus,

    pub requested_size: Decimal,
    pub max_size_under_threshold: Decimal,

    pub short_avg_cost: Option<Decimal>,
    pub long_avg_cost: Option<Decimal>,
    pub avg_package_cost: Option<Decimal>,

    pub gross_edge: Decimal,
    pub capital_required: Decimal,

    pub limiting_leg: DepthLeg,

    pub consumed_levels_short: usize,
    pub consumed_levels_long: usize,
}

impl DepthResult {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        status: DepthStatus,
        requested_size: Decimal,
        max_size_under_threshold: Decimal,
        short_avg_cost: Option<Decimal>,
        long_avg_cost: Option<Decimal>,
        avg_package_cost: Option<Decimal>,
        gross_edge: Decimal,
        capital_required: Decimal,
        limiting_leg: DepthLeg,
        consumed_levels_short: usize,
        consumed_levels_long: usize,
    ) -> Self {
        Self {
            status,
            requested_size,
            max_size_under_threshold,
            short_avg_cost,
            long_avg_cost,
            avg_package_cost,
            gross_edge,
            capital_required,
            limiting_leg,
            consumed_levels_short,
            consumed_levels_long,
        }
    }

    pub fn no_usable_asks(requested_size: Decimal) -> Self {
        Self::new(
            DepthStatus::NoUsableAsks,
            requested_size,
            Decimal::ZERO,
            None,
            None,
            None,
            Decimal::ZERO,
            Decimal::ZERO,
            DepthLeg::None,
            0,
            0,
        )
    }

    pub fn invalid_target_size(requested_size: Decimal) -> Self {
        Self::new(
            DepthStatus::InvalidTargetSize,
            requested_size,
            Decimal::ZERO,
            None,
            None,
            None,
            Decimal::ZERO,
            Decimal::ZERO,
            DepthLeg::None,
            0,
            0,
        )
    }

    pub fn is_depth_confirmed(&self) -> bool {
        self.status == DepthStatus::DepthConfirmedUnderOne
    }
}