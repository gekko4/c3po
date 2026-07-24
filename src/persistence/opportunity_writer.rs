// src/persistence/opportunity_writer.rs

use std::path::{Path, PathBuf};

use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::depth_engine::DepthResult;
use crate::persistence::jsonl::{append_jsonl, write_jsonl};
use crate::types::evaluation::{EvaluationClassification, EvaluationRow};
use crate::types::{Asset, MarketSlug, PackageName, Timeframe, TokenId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpportunityRecord {
    pub ts_ms: i64,
    pub asset: Asset,
    pub pair_type: String,
    pub short_tf: Timeframe,
    pub long_tf: Timeframe,
    pub short_slug: MarketSlug,
    pub long_slug: MarketSlug,
    pub package_name: Option<PackageName>,
    pub selected_short_token: Option<TokenId>,
    pub selected_long_token: Option<TokenId>,
    pub short_ptb: Option<Decimal>,
    pub long_ptb: Option<Decimal>,
    pub top_cost: Option<Decimal>,
    pub top_edge: Option<Decimal>,
    pub seconds_to_end: i64,
    pub evaluation_classification: EvaluationClassification,
    pub depth_result: Option<DepthResult>,
}

impl OpportunityRecord {
    pub fn from_evaluation(row: &EvaluationRow) -> Self {
        Self {
            ts_ms: row.ts_ms,
            asset: row.asset,
            pair_type: row.pair_type(),
            short_tf: row.short_tf,
            long_tf: row.long_tf,
            short_slug: row.short_slug.clone(),
            long_slug: row.long_slug.clone(),
            package_name: row.package_name,
            selected_short_token: row.selected_short_token.clone(),
            selected_long_token: row.selected_long_token.clone(),
            short_ptb: row.short_ptb,
            long_ptb: row.long_ptb,
            top_cost: row.top_cost,
            top_edge: row.edge,
            seconds_to_end: row.seconds_to_end,
            evaluation_classification: row.classification,
            depth_result: None,
        }
    }

    pub fn from_evaluation_and_depth(row: &EvaluationRow, depth_result: DepthResult) -> Self {
        let mut record = Self::from_evaluation(row);
        record.depth_result = Some(depth_result);
        record
    }

    pub fn is_under_one_candidate(&self) -> bool {
        self.evaluation_classification.is_under_one_candidate()
    }

    pub fn is_depth_confirmed(&self) -> bool {
        self.evaluation_classification.is_depth_confirmed()
            || self
                .depth_result
                .as_ref()
                .map(|depth| depth.is_depth_confirmed())
                .unwrap_or(false)
    }
}

#[derive(Debug, Clone)]
pub struct OpportunityWriter {
    path: PathBuf,
}

impl OpportunityWriter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(&self, record: &OpportunityRecord) -> Result<()> {
        append_jsonl(&self.path, record)
    }

    pub fn append_from_evaluation(&self, row: &EvaluationRow) -> Result<()> {
        let record = OpportunityRecord::from_evaluation(row);
        self.append(&record)
    }

    pub fn append_from_evaluation_and_depth(
        &self,
        row: &EvaluationRow,
        depth_result: DepthResult,
    ) -> Result<()> {
        let record = OpportunityRecord::from_evaluation_and_depth(row, depth_result);
        self.append(&record)
    }

    pub fn overwrite<'a, I>(&self, records: I) -> Result<usize>
    where
        I: IntoIterator<Item = &'a OpportunityRecord>,
    {
        write_jsonl::<OpportunityRecord, _>(&self.path, records)
    }
}
