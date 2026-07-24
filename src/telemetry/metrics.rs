// src/telemetry/metrics.rs

use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::book_store::{BookStore, BookStoreStats};
use crate::ptb_store::PtbStore;
use crate::registry::MarketRegistry;
use crate::types::{Asset, EvaluationClassification, EvaluationRow, Timeframe};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TelemetryCounters {
    pub clob_reconnect_count: usize,
    pub rtds_reconnect_count: usize,
}

impl TelemetryCounters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_clob_reconnect(&mut self) {
        self.clob_reconnect_count += 1;
    }

    pub fn increment_rtds_reconnect(&mut self) {
        self.rtds_reconnect_count += 1;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub ts_ms: i64,

    pub markets_total: usize,
    pub markets_live_total: usize,
    pub markets_discovered_by_timeframe: HashMap<Timeframe, usize>,
    pub markets_live_by_timeframe: HashMap<Timeframe, usize>,
    pub markets_discovered_by_asset_timeframe: HashMap<String, usize>,

    pub ptbs_total: usize,
    pub ptbs_captured_exact: usize,
    pub ptbs_pending: usize,
    pub ptbs_missing: usize,
    pub ptbs_invalid: usize,
    pub ptbs_captured_by_timeframe_and_asset: HashMap<String, usize>,
    pub ptbs_missing_by_timeframe_and_asset: HashMap<String, usize>,

    pub priced_pairs_by_timeframe_pair: HashMap<String, usize>,
    pub under_1_candidates_by_pair_type: HashMap<String, usize>,
    pub no_usable_asks_count: usize,

    pub max_gross_edge_by_asset: HashMap<Asset, Decimal>,

    pub book_store_stats: BookStoreStats,

    pub clob_reconnect_count: usize,
    pub rtds_reconnect_count: usize,
}

pub fn collect_telemetry_snapshot(
    registry: &MarketRegistry,
    ptb_store: &PtbStore,
    book_store: &BookStore,
    evaluation_rows: &[EvaluationRow],
    counters: TelemetryCounters,
    now_ms: i64,
    max_book_age_ms: i64,
) -> TelemetrySnapshot {
    let all_markets = registry.all_markets();
    let live_markets = registry.live_markets(now_ms);

    let mut markets_discovered_by_timeframe = HashMap::new();
    let mut markets_live_by_timeframe = HashMap::new();
    let mut markets_discovered_by_asset_timeframe = HashMap::new();

    for market in &all_markets {
        *markets_discovered_by_timeframe
            .entry(market.timeframe)
            .or_insert(0) += 1;

        *markets_discovered_by_asset_timeframe
            .entry(asset_timeframe_key(market.asset, market.timeframe))
            .or_insert(0) += 1;
    }

    for market in &live_markets {
        *markets_live_by_timeframe
            .entry(market.timeframe)
            .or_insert(0) += 1;
    }

    let mut ptbs_captured_by_timeframe_and_asset = HashMap::new();
    let mut ptbs_missing_by_timeframe_and_asset = HashMap::new();

    for ptb in ptb_store.captured_exact() {
        *ptbs_captured_by_timeframe_and_asset
            .entry(asset_timeframe_key(ptb.asset, ptb.timeframe))
            .or_insert(0) += 1;
    }

    for ptb in ptb_store.missing() {
        *ptbs_missing_by_timeframe_and_asset
            .entry(asset_timeframe_key(ptb.asset, ptb.timeframe))
            .or_insert(0) += 1;
    }

    let mut priced_pairs_by_timeframe_pair = HashMap::new();
    let mut under_1_candidates_by_pair_type = HashMap::new();
    let mut max_gross_edge_by_asset = HashMap::new();

    let mut no_usable_asks_count = 0usize;

    for row in evaluation_rows {
        if row.top_cost.is_some() {
            *priced_pairs_by_timeframe_pair
                .entry(row.pair_type())
                .or_insert(0) += 1;
        }

        if row.is_under_one_candidate() {
            *under_1_candidates_by_pair_type
                .entry(row.pair_type())
                .or_insert(0) += 1;
        }

        if row.classification == EvaluationClassification::NoUsableAsks {
            no_usable_asks_count += 1;
        }

        if let Some(edge) = row.edge {
            max_gross_edge_by_asset
                .entry(row.asset)
                .and_modify(|current| {
                    if edge > *current {
                        *current = edge;
                    }
                })
                .or_insert(edge);
        }
    }

    TelemetrySnapshot {
        ts_ms: now_ms,

        markets_total: all_markets.len(),
        markets_live_total: live_markets.len(),
        markets_discovered_by_timeframe,
        markets_live_by_timeframe,
        markets_discovered_by_asset_timeframe,

        ptbs_total: ptb_store.len(),
        ptbs_captured_exact: ptb_store.captured_exact().count(),
        ptbs_pending: ptb_store.pending().count(),
        ptbs_missing: ptb_store.missing().count(),
        ptbs_invalid: ptb_store.invalid().count(),
        ptbs_captured_by_timeframe_and_asset,
        ptbs_missing_by_timeframe_and_asset,

        priced_pairs_by_timeframe_pair,
        under_1_candidates_by_pair_type,
        no_usable_asks_count,

        max_gross_edge_by_asset,

        book_store_stats: book_store.stats(now_ms, max_book_age_ms),

        clob_reconnect_count: counters.clob_reconnect_count,
        rtds_reconnect_count: counters.rtds_reconnect_count,
    }
}

fn asset_timeframe_key(asset: Asset, timeframe: Timeframe) -> String {
    format!("{}-{}", asset, timeframe)
}
