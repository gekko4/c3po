// src/depth_engine/walker.rs

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::book::{Book, Level};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkResult {
    pub requested_size: Decimal,
    pub filled_size: Decimal,
    pub total_cost: Decimal,
    pub avg_price: Option<Decimal>,
    pub consumed_levels: usize,
    pub fully_filled: bool,
}

impl WalkResult {
    pub fn empty(requested_size: Decimal) -> Self {
        Self {
            requested_size,
            filled_size: Decimal::ZERO,
            total_cost: Decimal::ZERO,
            avg_price: None,
            consumed_levels: 0,
            fully_filled: false,
        }
    }

    pub fn is_usable(&self) -> bool {
        self.fully_filled && self.avg_price.is_some()
    }
}

pub fn walk_ask_book(book: &Book, requested_size: Decimal) -> WalkResult {
    if requested_size <= Decimal::ZERO {
        return WalkResult::empty(requested_size);
    }

    walk_levels(&book.asks, requested_size)
}

fn walk_levels(levels: &[Level], requested_size: Decimal) -> WalkResult {
    let mut remaining = requested_size;
    let mut filled_size = Decimal::ZERO;
    let mut total_cost = Decimal::ZERO;
    let mut consumed_levels = 0usize;

    for level in levels {
        if remaining <= Decimal::ZERO {
            break;
        }

        if level.price <= Decimal::ZERO || level.size <= Decimal::ZERO {
            continue;
        }

        let take_size = remaining.min(level.size);

        filled_size += take_size;
        total_cost += take_size * level.price;
        remaining -= take_size;
        consumed_levels += 1;
    }

    let fully_filled = filled_size >= requested_size;

    let avg_price = if filled_size > Decimal::ZERO {
        Some(total_cost / filled_size)
    } else {
        None
    };

    WalkResult {
        requested_size,
        filled_size,
        total_cost,
        avg_price,
        consumed_levels,
        fully_filled,
    }
}
