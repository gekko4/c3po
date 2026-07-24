// src/ptb_store/matcher.rs

use crate::ptb_store::sanity::check_ptb_plausibility;
use crate::types::market::Market;
use crate::types::ptb::{PriceToBeat, PtbStatus};
use crate::types::tick::{RtdsSymbol, Tick};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtbMatchResult {
    CapturedExact(PriceToBeat),
    Pending(PriceToBeat),
    Invalid(PriceToBeat),
}

impl PtbMatchResult {
    pub fn into_ptb(self) -> PriceToBeat {
        match self {
            PtbMatchResult::CapturedExact(ptb)
            | PtbMatchResult::Pending(ptb)
            | PtbMatchResult::Invalid(ptb) => ptb,
        }
    }

    pub fn status(&self) -> PtbStatus {
        match self {
            PtbMatchResult::CapturedExact(_) => PtbStatus::CapturedExact,
            PtbMatchResult::Pending(_) => PtbStatus::PendingPtb,
            PtbMatchResult::Invalid(_) => PtbStatus::InvalidPtb,
        }
    }
}

pub fn expected_rtds_symbol_for_market(market: &Market) -> RtdsSymbol {
    RtdsSymbol::normalized(market.asset.rtds_symbol())
}

pub fn match_market_to_tick(
    market: &Market,
    tick: Option<&Tick>,
    checked_at_ms: i64,
) -> PtbMatchResult {
    let Some(tick) = tick else {
        return PtbMatchResult::Pending(PriceToBeat::pending(
            market.slug.clone(),
            market.asset,
            market.timeframe,
            market.open_ms,
        ));
    };

    let expected_symbol = expected_rtds_symbol_for_market(market);

    if !tick.matches_symbol_and_timestamp(&expected_symbol, market.open_ms) {
        return PtbMatchResult::Pending(PriceToBeat::pending(
            market.slug.clone(),
            market.asset,
            market.timeframe,
            market.open_ms,
        ));
    }

    if !tick.has_positive_value() {
        return PtbMatchResult::Invalid(PriceToBeat::invalid(
            market.slug.clone(),
            market.asset,
            market.timeframe,
            market.open_ms,
            Some(tick.raw_value.clone()),
            "non_positive_tick_value",
            checked_at_ms,
        ));
    }

    let sanity = check_ptb_plausibility(market.asset, tick.normalized_value);

    if !sanity.is_plausible {
        return PtbMatchResult::Invalid(PriceToBeat::invalid(
            market.slug.clone(),
            market.asset,
            market.timeframe,
            market.open_ms,
            Some(tick.raw_value.clone()),
            sanity.status,
            checked_at_ms,
        ));
    }

    PtbMatchResult::CapturedExact(PriceToBeat::captured_exact(
        market.slug.clone(),
        market.asset,
        market.timeframe,
        market.open_ms,
        tick.normalized_value,
        tick.raw_value.clone(),
        tick.full_accuracy_value.clone(),
        tick.timestamp_ms,
        Some(sanity.status),
        checked_at_ms,
    ))
}

pub fn apply_ptb_to_market(market: &mut Market, ptb: &PriceToBeat) {
    match ptb.status {
        PtbStatus::PendingPtb => {
            market.mark_pending_ptb();
        }
        PtbStatus::MissingPtb => {
            market.mark_missing_ptb();
        }
        PtbStatus::InvalidPtb => {
            market.mark_invalid_ptb(ptb.raw_value.clone());
        }
        PtbStatus::CapturedExact => {
            if let Some(value) = ptb.normalized_value {
                market.capture_exact_ptb(
                    value,
                    ptb.source
                        .map(|source| source.as_str().to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    ptb.source_tick_timestamp_ms.unwrap_or(ptb.open_ms),
                    ptb.raw_value.clone().unwrap_or_default(),
                    ptb.full_accuracy_value.clone(),
                );
            } else {
                market.mark_invalid_ptb(ptb.raw_value.clone());
            }
        }
    }
}
