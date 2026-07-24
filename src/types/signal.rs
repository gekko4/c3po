// src/types/signal.rs

use crate::types::{Asset, EvaluationClassification, MarketSlug, PackageName, Timeframe, TokenId};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Promoted signal emitted only after filters and depth checks pass.
///
/// This is still research/output data. Execution should be handled by a future
/// trading layer with stricter safeguards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signal {
    pub asset: Asset,
    pub pair_type: String,

    pub short_tf: Timeframe,
    pub long_tf: Timeframe,

    pub short_slug: MarketSlug,
    pub long_slug: MarketSlug,

    pub package_name: PackageName,

    pub selected_short_token: TokenId,
    pub selected_long_token: TokenId,

    pub short_ptb: Decimal,
    pub long_ptb: Decimal,

    pub top_cost: Decimal,
    pub depth_avg_cost: Decimal,

    pub executable_size: Decimal,
    pub gross_edge: Decimal,
    pub capital_required: Decimal,

    pub seconds_to_end: i64,

    pub classification: EvaluationClassification,

    pub created_at_ms: i64,
}

impl Signal {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        asset: Asset,
        short_tf: Timeframe,
        long_tf: Timeframe,
        short_slug: MarketSlug,
        long_slug: MarketSlug,
        package_name: PackageName,
        selected_short_token: TokenId,
        selected_long_token: TokenId,
        short_ptb: Decimal,
        long_ptb: Decimal,
        top_cost: Decimal,
        depth_avg_cost: Decimal,
        executable_size: Decimal,
        gross_edge: Decimal,
        capital_required: Decimal,
        seconds_to_end: i64,
        created_at_ms: i64,
    ) -> Self {
        let pair_type = format!("{}-{}", short_tf, long_tf);

        Self {
            asset,
            pair_type,
            short_tf,
            long_tf,
            short_slug,
            long_slug,
            package_name,
            selected_short_token,
            selected_long_token,
            short_ptb,
            long_ptb,
            top_cost,
            depth_avg_cost,
            executable_size,
            gross_edge,
            capital_required,
            seconds_to_end,
            classification: EvaluationClassification::DepthConfirmedUnderOne,
            created_at_ms,
        }
    }

    pub fn is_depth_confirmed(&self) -> bool {
        self.classification == EvaluationClassification::DepthConfirmedUnderOne
    }

    pub fn tokens(&self) -> [&TokenId; 2] {
        [&self.selected_short_token, &self.selected_long_token]
    }

    pub fn edge_per_share(&self) -> Decimal {
        Decimal::ONE - self.depth_avg_cost
    }

    pub fn top_edge_per_share(&self) -> Decimal {
        Decimal::ONE - self.top_cost
    }

    pub fn recompute_pair_type(&mut self) {
        self.pair_type = format!("{}-{}", self.short_tf, self.long_tf);
    }

    pub fn is_expired_or_settled(&self) -> bool {
        self.seconds_to_end <= 0
    }
}
