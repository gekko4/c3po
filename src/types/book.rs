// src/types/book.rs

use crate::types::TokenId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// One orderbook price level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Level {
    pub price: Decimal,
    pub size: Decimal,
}

impl Level {
    pub fn new(price: Decimal, size: Decimal) -> Self {
        Self { price, size }
    }

    pub fn is_positive(&self) -> bool {
        self.price > Decimal::ZERO && self.size > Decimal::ZERO
    }
}

/// Book side used when parsing or applying CLOB updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BookSide {
    Bid,
    Ask,
}

/// Current orderbook snapshot for one outcome token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Book {
    pub token_id: TokenId,

    pub bids: Vec<Level>,
    pub asks: Vec<Level>,

    pub best_bid: Option<Decimal>,
    pub best_bid_size: Option<Decimal>,

    pub best_ask: Option<Decimal>,
    pub best_ask_size: Option<Decimal>,

    /// Timestamp from the exchange message, if provided.
    pub exchange_ts_ms: Option<i64>,

    /// Local receive timestamp.
    pub received_at_ms: i64,
}

impl Book {
    pub fn new(
        token_id: TokenId,
        bids: Vec<Level>,
        asks: Vec<Level>,
        exchange_ts_ms: Option<i64>,
        received_at_ms: i64,
    ) -> Self {
        let mut book = Self {
            token_id,
            bids,
            asks,
            best_bid: None,
            best_bid_size: None,
            best_ask: None,
            best_ask_size: None,
            exchange_ts_ms,
            received_at_ms,
        };

        book.normalize();
        book
    }

    pub fn empty(token_id: TokenId, received_at_ms: i64) -> Self {
        Self {
            token_id,
            bids: Vec::new(),
            asks: Vec::new(),
            best_bid: None,
            best_bid_size: None,
            best_ask: None,
            best_ask_size: None,
            exchange_ts_ms: None,
            received_at_ms,
        }
    }

    /// Sorts bids descending and asks ascending, then refreshes best levels.
    pub fn normalize(&mut self) {
        self.bids.sort_by(|a, b| b.price.cmp(&a.price));
        self.asks.sort_by(|a, b| a.price.cmp(&b.price));
        self.recompute_best_levels();
    }

    /// Refreshes cached best bid/ask fields from already-normalized levels.
    pub fn recompute_best_levels(&mut self) {
        self.best_bid = self.bids.first().map(|level| level.price);
        self.best_bid_size = self.bids.first().map(|level| level.size);

        self.best_ask = self.asks.first().map(|level| level.price);
        self.best_ask_size = self.asks.first().map(|level| level.size);
    }

    pub fn best_bid_level(&self) -> Option<&Level> {
        self.bids.first()
    }

    pub fn best_ask_level(&self) -> Option<&Level> {
        self.asks.first()
    }

    pub fn usable_ask_or_none(&self) -> Option<&Level> {
        let level = self.best_ask_level()?;

        if level.is_positive() {
            Some(level)
        } else {
            None
        }
    }

    pub fn has_usable_asks(&self) -> bool {
        self.usable_ask_or_none().is_some()
    }

    pub fn age_ms(&self, now_ms: i64) -> i64 {
        now_ms.saturating_sub(self.received_at_ms)
    }

    pub fn is_fresh(&self, now_ms: i64, max_book_age_ms: i64) -> bool {
        self.age_ms(now_ms) <= max_book_age_ms
    }

    pub fn is_stale(&self, now_ms: i64, max_book_age_ms: i64) -> bool {
        !self.is_fresh(now_ms, max_book_age_ms)
    }

    pub fn is_usable(&self, now_ms: i64, max_book_age_ms: i64) -> bool {
        self.has_usable_asks() && self.is_fresh(now_ms, max_book_age_ms)
    }
}
