// src/types/ptb.rs

use crate::types::{Asset, MarketSlug, Timeframe};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Lifecycle status for price-to-beat capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PtbStatus {
    PendingPtb,
    CapturedExact,
    MissingPtb,
    InvalidPtb,
}

impl PtbStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            PtbStatus::PendingPtb => "pending_ptb",
            PtbStatus::CapturedExact => "captured_exact",
            PtbStatus::MissingPtb => "missing_ptb",
            PtbStatus::InvalidPtb => "invalid_ptb",
        }
    }

    pub fn is_usable(self) -> bool {
        matches!(self, PtbStatus::CapturedExact)
    }
}

impl fmt::Display for PtbStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).as_str())
    }
}

/// Source of PTB value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PtbSource {
    RtdsExactOpenTick,
    Replay,
    ManualFixture,
}

impl PtbSource {
    pub fn as_str(self) -> &'static str {
        match self {
            PtbSource::RtdsExactOpenTick => "rtds_exact_open_tick",
            PtbSource::Replay => "replay",
            PtbSource::ManualFixture => "manual_fixture",
        }
    }
}

impl fmt::Display for PtbSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).as_str())
    }
}

/// Captured price-to-beat for one market.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceToBeat {
    pub market_slug: MarketSlug,

    pub asset: Asset,
    pub timeframe: Timeframe,

    pub open_ms: i64,

    pub normalized_value: Option<Decimal>,
    pub raw_value: Option<String>,
    pub full_accuracy_value: Option<String>,

    pub source_tick_timestamp_ms: Option<i64>,
    pub source: Option<PtbSource>,

    pub status: PtbStatus,

    pub plausibility_status: Option<String>,

    pub captured_at_ms: Option<i64>,
}

impl PriceToBeat {
    pub fn pending(
        market_slug: MarketSlug,
        asset: Asset,
        timeframe: Timeframe,
        open_ms: i64,
    ) -> Self {
        Self {
            market_slug,
            asset,
            timeframe,
            open_ms,
            normalized_value: None,
            raw_value: None,
            full_accuracy_value: None,
            source_tick_timestamp_ms: None,
            source: None,
            status: PtbStatus::PendingPtb,
            plausibility_status: None,
            captured_at_ms: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn captured_exact(
        market_slug: MarketSlug,
        asset: Asset,
        timeframe: Timeframe,
        open_ms: i64,
        normalized_value: Decimal,
        raw_value: impl Into<String>,
        full_accuracy_value: Option<String>,
        source_tick_timestamp_ms: i64,
        plausibility_status: Option<String>,
        captured_at_ms: i64,
    ) -> Self {
        Self {
            market_slug,
            asset,
            timeframe,
            open_ms,
            normalized_value: Some(normalized_value),
            raw_value: Some(raw_value.into()),
            full_accuracy_value,
            source_tick_timestamp_ms: Some(source_tick_timestamp_ms),
            source: Some(PtbSource::RtdsExactOpenTick),
            status: PtbStatus::CapturedExact,
            plausibility_status,
            captured_at_ms: Some(captured_at_ms),
        }
    }

    pub fn missing(
        market_slug: MarketSlug,
        asset: Asset,
        timeframe: Timeframe,
        open_ms: i64,
        checked_at_ms: i64,
    ) -> Self {
        Self {
            market_slug,
            asset,
            timeframe,
            open_ms,
            normalized_value: None,
            raw_value: None,
            full_accuracy_value: None,
            source_tick_timestamp_ms: None,
            source: None,
            status: PtbStatus::MissingPtb,
            plausibility_status: None,
            captured_at_ms: Some(checked_at_ms),
        }
    }

    pub fn invalid(
        market_slug: MarketSlug,
        asset: Asset,
        timeframe: Timeframe,
        open_ms: i64,
        raw_value: Option<String>,
        reason: impl Into<String>,
        checked_at_ms: i64,
    ) -> Self {
        Self {
            market_slug,
            asset,
            timeframe,
            open_ms,
            normalized_value: None,
            raw_value,
            full_accuracy_value: None,
            source_tick_timestamp_ms: None,
            source: None,
            status: PtbStatus::InvalidPtb,
            plausibility_status: Some(reason.into()),
            captured_at_ms: Some(checked_at_ms),
        }
    }

    pub fn is_usable(&self) -> bool {
        self.status.is_usable() && self.normalized_value.is_some()
    }

    pub fn value(&self) -> Option<Decimal> {
        self.normalized_value
    }
}