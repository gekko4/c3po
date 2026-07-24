// src/types/market.rs

use crate::types::{Asset, ConditionId, Timeframe, TokenId};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Polymarket market slug wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MarketSlug(String);

impl MarketSlug {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl fmt::Display for MarketSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for MarketSlug {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for MarketSlug {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Market metadata plus PTB-related fields needed by registry/replay.
///
/// The PTB store can also persist richer PTB records, but keeping the basic
/// PTB fields here makes registry snapshots self-describing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Market {
    pub asset: Asset,
    pub timeframe: Timeframe,

    pub slug: MarketSlug,
    pub condition_id: ConditionId,

    pub up_token_id: TokenId,
    pub down_token_id: TokenId,

    pub open_ms: i64,
    pub end_ms: i64,

    pub active: bool,
    pub closed: bool,

    pub price_to_beat: Option<Decimal>,
    pub price_to_beat_status: Option<String>,
    pub price_to_beat_source: Option<String>,
    pub price_to_beat_timestamp_ms: Option<i64>,
    pub price_to_beat_raw_value: Option<String>,
    pub price_to_beat_full_accuracy_value: Option<String>,

    pub first_seen_ms: i64,
    pub last_seen_ms: i64,
}

impl Market {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        asset: Asset,
        timeframe: Timeframe,
        slug: MarketSlug,
        condition_id: ConditionId,
        up_token_id: TokenId,
        down_token_id: TokenId,
        open_ms: i64,
        end_ms: i64,
        active: bool,
        closed: bool,
        seen_ms: i64,
    ) -> Self {
        Self {
            asset,
            timeframe,
            slug,
            condition_id,
            up_token_id,
            down_token_id,
            open_ms,
            end_ms,
            active,
            closed,

            price_to_beat: None,
            price_to_beat_status: None,
            price_to_beat_source: None,
            price_to_beat_timestamp_ms: None,
            price_to_beat_raw_value: None,
            price_to_beat_full_accuracy_value: None,

            first_seen_ms: seen_ms,
            last_seen_ms: seen_ms,
        }
    }

    pub fn is_live(&self, now_ms: i64) -> bool {
        self.active && !self.closed && self.open_ms <= now_ms && now_ms < self.end_ms
    }

    pub fn is_upcoming(&self, now_ms: i64) -> bool {
        self.active && !self.closed && now_ms < self.open_ms
    }

    pub fn is_expired(&self, now_ms: i64) -> bool {
        self.closed || now_ms >= self.end_ms
    }

    pub fn seconds_to_start(&self, now_ms: i64) -> i64 {
        self.open_ms.saturating_sub(now_ms) / 1_000
    }

    pub fn seconds_to_end(&self, now_ms: i64) -> i64 {
        self.end_ms.saturating_sub(now_ms) / 1_000
    }

    pub fn duration_ms(&self) -> i64 {
        self.end_ms.saturating_sub(self.open_ms)
    }

    pub fn has_tokens(&self) -> bool {
        !self.up_token_id.as_str().trim().is_empty()
            && !self.down_token_id.as_str().trim().is_empty()
    }

    pub fn has_valid_time_window(&self) -> bool {
        self.open_ms < self.end_ms
    }

    pub fn has_price_to_beat(&self) -> bool {
        self.price_to_beat.is_some()
    }

    pub fn has_captured_exact_ptb(&self) -> bool {
        matches!(
            self.price_to_beat_status.as_deref(),
            Some("captured_exact") | Some("CAPTURED_EXACT")
        ) && self.price_to_beat.is_some()
    }

    pub fn mark_pending_ptb(&mut self) {
        self.price_to_beat = None;
        self.price_to_beat_status = Some("pending_ptb".to_string());
        self.price_to_beat_source = None;
        self.price_to_beat_timestamp_ms = None;
        self.price_to_beat_raw_value = None;
        self.price_to_beat_full_accuracy_value = None;
    }

    pub fn mark_missing_ptb(&mut self) {
        self.price_to_beat = None;
        self.price_to_beat_status = Some("missing_ptb".to_string());
        self.price_to_beat_source = None;
        self.price_to_beat_timestamp_ms = None;
        self.price_to_beat_raw_value = None;
        self.price_to_beat_full_accuracy_value = None;
    }

    pub fn mark_invalid_ptb(&mut self, raw_value: Option<String>) {
        self.price_to_beat = None;
        self.price_to_beat_status = Some("invalid_ptb".to_string());
        self.price_to_beat_source = None;
        self.price_to_beat_timestamp_ms = None;
        self.price_to_beat_raw_value = raw_value;
        self.price_to_beat_full_accuracy_value = None;
    }

    pub fn capture_exact_ptb(
        &mut self,
        price_to_beat: Decimal,
        source: impl Into<String>,
        source_tick_timestamp_ms: i64,
        raw_value: impl Into<String>,
        full_accuracy_value: Option<String>,
    ) {
        self.price_to_beat = Some(price_to_beat);
        self.price_to_beat_status = Some("captured_exact".to_string());
        self.price_to_beat_source = Some(source.into());
        self.price_to_beat_timestamp_ms = Some(source_tick_timestamp_ms);
        self.price_to_beat_raw_value = Some(raw_value.into());
        self.price_to_beat_full_accuracy_value = full_accuracy_value;
    }

    pub fn touch(&mut self, seen_ms: i64) {
        self.last_seen_ms = seen_ms;
    }

    pub fn update_lifecycle(&mut self, active: bool, closed: bool, seen_ms: i64) {
        self.active = active;
        self.closed = closed;
        self.touch(seen_ms);
    }
}
