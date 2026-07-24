// src/evaluator/top_book.rs

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::config::EvaluationConfig;
use crate::types::book::Book;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopBookQuote {
    pub short_ask: Decimal,
    pub long_ask: Decimal,
    pub short_ask_size: Decimal,
    pub long_ask_size: Decimal,
    pub top_cost: Decimal,
    pub edge: Decimal,
    pub top_book_size: Decimal,
    pub trigger_threshold: Decimal,
    pub passes_trigger: bool,
    pub passes_min_top_book_size: bool,
}

impl TopBookQuote {
    pub fn is_under_trigger(&self) -> bool {
        self.passes_trigger
    }

    pub fn has_min_top_book_size(&self) -> bool {
        self.passes_min_top_book_size
    }
}

pub fn calculate_top_book_quote(
    short_book: &Book,
    long_book: &Book,
    eval_config: &EvaluationConfig,
) -> Option<TopBookQuote> {
    let short_ask_level = short_book.usable_ask_or_none()?;
    let long_ask_level = long_book.usable_ask_or_none()?;

    let short_ask = short_ask_level.price;
    let long_ask = long_ask_level.price;

    let short_ask_size = short_ask_level.size;
    let long_ask_size = long_ask_level.size;

    let top_cost = short_ask + long_ask;
    let edge = Decimal::ONE - top_cost;
    let top_book_size = short_ask_size.min(long_ask_size);

    let trigger_threshold = Decimal::ONE - eval_config.trigger_buffer;

    Some(TopBookQuote {
        short_ask,
        long_ask,
        short_ask_size,
        long_ask_size,
        top_cost,
        edge,
        top_book_size,
        trigger_threshold,
        passes_trigger: top_cost < trigger_threshold,
        passes_min_top_book_size: top_book_size >= eval_config.min_top_book_size,
    })
}
