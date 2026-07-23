// src/ptb_store/sanity.rs

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::Asset;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PtbSanityResult {
    pub is_plausible: bool,
    pub status: String,
}

impl PtbSanityResult {
    pub fn ok() -> Self {
        Self {
            is_plausible: true,
            status: "plausible".to_string(),
        }
    }

    pub fn failed(reason: impl Into<String>) -> Self {
        Self {
            is_plausible: false,
            status: reason.into(),
        }
    }
}

pub fn check_ptb_plausibility(asset: Asset, value: Decimal) -> PtbSanityResult {
    if value <= Decimal::ZERO {
        return PtbSanityResult::failed("non_positive_ptb");
    }

    let (min, max) = plausible_range(asset);

    if value < min {
        return PtbSanityResult::failed(format!(
            "below_plausible_range:{}<{}",
            value, min
        ));
    }

    if value > max {
        return PtbSanityResult::failed(format!(
            "above_plausible_range:{}>{}",
            value, max
        ));
    }

    PtbSanityResult::ok()
}

fn plausible_range(asset: Asset) -> (Decimal, Decimal) {
    match asset {
        Asset::BTC => (
            Decimal::from(1_000),
            Decimal::from(1_000_000),
        ),
        Asset::ETH => (
            Decimal::from(100),
            Decimal::from(100_000),
        ),
        Asset::SOL => (
            Decimal::from(1),
            Decimal::from(10_000),
        ),
        Asset::XRP => (
            Decimal::new(1, 2),
            Decimal::from(100),
        ),
    }
}