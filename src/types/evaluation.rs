// src/types/evaluation.rs

use crate::types::{Asset, MarketSlug, PackageName, Timeframe, TokenId};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Classification emitted by the evaluator for every checked pair/package.
///
/// Keep these values stable because they are written to JSONL and used in replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvaluationClassification {
    MissingShortPtb,
    MissingLongPtb,
    EqualPtbNoPackage,
    NoUsableAsks,
    StaleBook,
    ExpiredOrTooClose,
    ExpectedAboveOne,
    TopOfBookOnlyUnderOne,
    DepthConfirmedUnderOne,
}

impl EvaluationClassification {
    pub fn as_str(self) -> &'static str {
        match self {
            EvaluationClassification::MissingShortPtb => "MISSING_SHORT_PTB",
            EvaluationClassification::MissingLongPtb => "MISSING_LONG_PTB",
            EvaluationClassification::EqualPtbNoPackage => "EQUAL_PTB_NO_PACKAGE",
            EvaluationClassification::NoUsableAsks => "NO_USABLE_ASKS",
            EvaluationClassification::StaleBook => "STALE_BOOK",
            EvaluationClassification::ExpiredOrTooClose => "EXPIRED_OR_TOO_CLOSE",
            EvaluationClassification::ExpectedAboveOne => "EXPECTED_ABOVE_1",
            EvaluationClassification::TopOfBookOnlyUnderOne => "TOP_OF_BOOK_ONLY_UNDER_1",
            EvaluationClassification::DepthConfirmedUnderOne => "DEPTH_CONFIRMED_UNDER_1",
        }
    }

    pub fn is_under_one_candidate(self) -> bool {
        matches!(
            self,
            EvaluationClassification::TopOfBookOnlyUnderOne
                | EvaluationClassification::DepthConfirmedUnderOne
        )
    }

    pub fn is_depth_confirmed(self) -> bool {
        matches!(self, EvaluationClassification::DepthConfirmedUnderOne)
    }

    pub fn is_terminal_failure(self) -> bool {
        matches!(
            self,
            EvaluationClassification::MissingShortPtb
                | EvaluationClassification::MissingLongPtb
                | EvaluationClassification::EqualPtbNoPackage
                | EvaluationClassification::NoUsableAsks
                | EvaluationClassification::StaleBook
                | EvaluationClassification::ExpiredOrTooClose
                | EvaluationClassification::ExpectedAboveOne
        )
    }
}

/// One persisted evaluator row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationRow {
    pub ts_ms: i64,

    pub asset: Asset,

    pub short_tf: Timeframe,
    pub long_tf: Timeframe,

    pub short_slug: MarketSlug,
    pub long_slug: MarketSlug,

    pub short_ptb: Option<Decimal>,
    pub long_ptb: Option<Decimal>,

    pub package_name: Option<PackageName>,

    pub selected_short_token: Option<TokenId>,
    pub selected_long_token: Option<TokenId>,

    pub short_ask: Option<Decimal>,
    pub long_ask: Option<Decimal>,

    pub short_ask_size: Option<Decimal>,
    pub long_ask_size: Option<Decimal>,

    pub top_cost: Option<Decimal>,
    pub edge: Option<Decimal>,

    pub seconds_to_end: i64,

    pub classification: EvaluationClassification,
}

impl EvaluationRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ts_ms: i64,
        asset: Asset,
        short_tf: Timeframe,
        long_tf: Timeframe,
        short_slug: MarketSlug,
        long_slug: MarketSlug,
        short_ptb: Option<Decimal>,
        long_ptb: Option<Decimal>,
        package_name: Option<PackageName>,
        selected_short_token: Option<TokenId>,
        selected_long_token: Option<TokenId>,
        short_ask: Option<Decimal>,
        long_ask: Option<Decimal>,
        short_ask_size: Option<Decimal>,
        long_ask_size: Option<Decimal>,
        top_cost: Option<Decimal>,
        edge: Option<Decimal>,
        seconds_to_end: i64,
        classification: EvaluationClassification,
    ) -> Self {
        Self {
            ts_ms,
            asset,
            short_tf,
            long_tf,
            short_slug,
            long_slug,
            short_ptb,
            long_ptb,
            package_name,
            selected_short_token,
            selected_long_token,
            short_ask,
            long_ask,
            short_ask_size,
            long_ask_size,
            top_cost,
            edge,
            seconds_to_end,
            classification,
        }
    }

    pub fn pair_type(&self) -> String {
        format!("{}-{}", self.short_tf, self.long_tf)
    }

    pub fn is_under_one_candidate(&self) -> bool {
        self.classification.is_under_one_candidate()
    }

    pub fn is_depth_confirmed(&self) -> bool {
        self.classification.is_depth_confirmed()
    }

    pub fn is_terminal_failure(&self) -> bool {
        self.classification.is_terminal_failure()
    }
}
