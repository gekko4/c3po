// src/signal_engine/signal_builder.rs

use anyhow::{bail, Result};
use rust_decimal::Decimal;

use crate::config::Config;
use crate::depth_engine::DepthResult;
use crate::signal_engine::filters::filter_signal_candidate;
use crate::types::package::PackageCandidate;
use crate::types::signal::Signal;

pub fn build_signal(
    config: &Config,
    candidate: &PackageCandidate,
    top_cost: Decimal,
    depth_result: &DepthResult,
    created_at_ms: i64,
) -> Result<Signal> {
    let outcome = filter_signal_candidate(config, candidate, top_cost, depth_result);

    if !outcome.accepted {
        bail!("signal rejected: {}", outcome.message);
    }

    Ok(build_signal_unchecked(
        candidate,
        top_cost,
        depth_result,
        created_at_ms,
    ))
}

pub fn build_signal_unchecked(
    candidate: &PackageCandidate,
    top_cost: Decimal,
    depth_result: &DepthResult,
    created_at_ms: i64,
) -> Signal {
    let depth_avg_cost = depth_result.avg_package_cost.unwrap_or(top_cost);

    Signal::new(
        candidate.asset,
        candidate.short_tf,
        candidate.long_tf,
        candidate.short_market_slug.clone(),
        candidate.long_market_slug.clone(),
        candidate.package_name,
        candidate.selected_short_token.clone(),
        candidate.selected_long_token.clone(),
        candidate.short_ptb,
        candidate.long_ptb,
        top_cost,
        depth_avg_cost,
        depth_result.max_size_under_threshold,
        depth_result.gross_edge,
        depth_result.capital_required,
        candidate.seconds_to_end,
        created_at_ms,
    )
}
