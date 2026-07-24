// src/registry/lifecycle.rs

use crate::types::market::Market;

pub fn is_live_market(market: &Market, now_ms: i64) -> bool {
    market.active && !market.closed && market.open_ms <= now_ms && now_ms < market.end_ms
}

pub fn is_upcoming_market(market: &Market, now_ms: i64) -> bool {
    market.active && !market.closed && now_ms < market.open_ms
}

pub fn is_expired_market(market: &Market, now_ms: i64) -> bool {
    market.closed || now_ms >= market.end_ms
}

pub fn seconds_to_start(market: &Market, now_ms: i64) -> i64 {
    market.open_ms.saturating_sub(now_ms) / 1_000
}

pub fn seconds_to_end(market: &Market, now_ms: i64) -> i64 {
    market.end_ms.saturating_sub(now_ms) / 1_000
}
