// src/depth_engine/sizing.rs

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::book::Book;

use super::result::{DepthLeg, DepthResult, DepthStatus};
use super::walker::{walk_ask_book, WalkResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepthSizingConfig {
    pub execution_buffer: Decimal,
    pub min_depth_confirmed_size: Decimal,
    pub min_gross_edge: Decimal,
}

impl DepthSizingConfig {
    pub fn execution_threshold(&self) -> Decimal {
        Decimal::ONE - self.execution_buffer
    }
}

pub fn calculate_depth_for_size(
    short_book: &Book,
    long_book: &Book,
    requested_size: Decimal,
    config: &DepthSizingConfig,
) -> DepthResult {
    if requested_size <= Decimal::ZERO {
        return DepthResult::invalid_target_size(requested_size);
    }

    let short_walk = walk_ask_book(short_book, requested_size);
    let long_walk = walk_ask_book(long_book, requested_size);

    if !short_walk.is_usable() || !long_walk.is_usable() {
        return insufficient_depth_result(requested_size, short_walk, long_walk);
    }

    let short_avg = short_walk.avg_price.expect("checked by is_usable");
    let long_avg = long_walk.avg_price.expect("checked by is_usable");

    let avg_package_cost = short_avg + long_avg;
    let capital_required = requested_size * avg_package_cost;
    let gross_edge = requested_size * (Decimal::ONE - avg_package_cost);

    let passes_cost = avg_package_cost < config.execution_threshold();
    let passes_size = requested_size >= config.min_depth_confirmed_size;
    let passes_edge = gross_edge >= config.min_gross_edge;

    let status = if passes_cost && passes_size && passes_edge {
        DepthStatus::DepthConfirmedUnderOne
    } else if avg_package_cost < Decimal::ONE {
        DepthStatus::TopBookOnlyUnderOne
    } else {
        DepthStatus::InsufficientDepth
    };

    DepthResult::new(
        status,
        requested_size,
        if passes_cost {
            requested_size
        } else {
            Decimal::ZERO
        },
        Some(short_avg),
        Some(long_avg),
        Some(avg_package_cost),
        gross_edge.max(Decimal::ZERO),
        capital_required,
        limiting_leg_for_walks(&short_walk, &long_walk),
        short_walk.consumed_levels,
        long_walk.consumed_levels,
    )
}

pub fn find_max_size_under_threshold(
    short_book: &Book,
    long_book: &Book,
    max_requested_size: Decimal,
    config: &DepthSizingConfig,
) -> DepthResult {
    if max_requested_size <= Decimal::ZERO {
        return DepthResult::invalid_target_size(max_requested_size);
    }

    let candidate_sizes = merged_ask_depth_sizes(short_book, long_book, max_requested_size);

    if candidate_sizes.is_empty() {
        return DepthResult::no_usable_asks(max_requested_size);
    }

    let mut best: Option<DepthResult> = None;

    for size in candidate_sizes {
        let result = calculate_depth_for_size(short_book, long_book, size, config);

        if let Some(avg_package_cost) = result.avg_package_cost {
            if avg_package_cost < config.execution_threshold() {
                match &best {
                    Some(current_best)
                        if result.max_size_under_threshold
                            <= current_best.max_size_under_threshold => {}
                    _ => best = Some(result),
                }
            }
        }
    }

    match best {
        Some(mut result) => {
            let passes_size = result.max_size_under_threshold >= config.min_depth_confirmed_size;
            let passes_edge = result.gross_edge >= config.min_gross_edge;

            result.status = if passes_size && passes_edge {
                DepthStatus::DepthConfirmedUnderOne
            } else {
                DepthStatus::TopBookOnlyUnderOne
            };

            result
        }
        None => calculate_depth_for_size(short_book, long_book, max_requested_size, config),
    }
}

fn insufficient_depth_result(
    requested_size: Decimal,
    short_walk: WalkResult,
    long_walk: WalkResult,
) -> DepthResult {
    let limiting_leg = limiting_leg_for_walks(&short_walk, &long_walk);

    DepthResult::new(
        DepthStatus::InsufficientDepth,
        requested_size,
        Decimal::ZERO,
        short_walk.avg_price,
        long_walk.avg_price,
        None,
        Decimal::ZERO,
        Decimal::ZERO,
        limiting_leg,
        short_walk.consumed_levels,
        long_walk.consumed_levels,
    )
}

fn limiting_leg_for_walks(short_walk: &WalkResult, long_walk: &WalkResult) -> DepthLeg {
    match (short_walk.fully_filled, long_walk.fully_filled) {
        (true, true) => DepthLeg::None,
        (false, true) => DepthLeg::Short,
        (true, false) => DepthLeg::Long,
        (false, false) => {
            if short_walk.filled_size < long_walk.filled_size {
                DepthLeg::Short
            } else if long_walk.filled_size < short_walk.filled_size {
                DepthLeg::Long
            } else {
                DepthLeg::Both
            }
        }
    }
}

fn merged_ask_depth_sizes(
    short_book: &Book,
    long_book: &Book,
    max_requested_size: Decimal,
) -> Vec<Decimal> {
    let mut sizes = Vec::new();

    collect_cumulative_ask_sizes(short_book, max_requested_size, &mut sizes);
    collect_cumulative_ask_sizes(long_book, max_requested_size, &mut sizes);

    sizes.push(max_requested_size);

    sizes.retain(|size| *size > Decimal::ZERO && *size <= max_requested_size);
    sizes.sort();
    sizes.dedup();

    sizes
}

fn collect_cumulative_ask_sizes(
    book: &Book,
    max_requested_size: Decimal,
    output: &mut Vec<Decimal>,
) {
    let mut cumulative = Decimal::ZERO;

    for level in &book.asks {
        if level.price <= Decimal::ZERO || level.size <= Decimal::ZERO {
            continue;
        }

        cumulative += level.size;

        if cumulative > max_requested_size {
            output.push(max_requested_size);
            break;
        }

        output.push(cumulative);
    }
}
