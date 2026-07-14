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
    pub fn pair_type(&self) -> String {
        format!("{}-{}", self.short_tf, self.long_tf)
    }

    pub fn is_under_one_candidate(&self) -> bool {
        self.classification.is_under_one_candidate()
    }

    pub fn is_depth_confirmed(&self) -> bool {
        self.classification.is_depth_confirmed()
    }
}