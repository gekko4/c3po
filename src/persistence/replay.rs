// src/persistence/replay.rs

use std::path::Path;

use anyhow::Result;

use crate::depth_engine::DepthResult;
use crate::persistence::jsonl::read_jsonl;
use crate::persistence::opportunity_writer::OpportunityRecord;
use crate::types::evaluation::EvaluationRow;
use crate::types::signal::Signal;

pub fn load_evaluation_rows(path: impl AsRef<Path>) -> Result<Vec<EvaluationRow>> {
    read_jsonl::<EvaluationRow>(path)
}

pub fn load_opportunity_records(path: impl AsRef<Path>) -> Result<Vec<OpportunityRecord>> {
    read_jsonl::<OpportunityRecord>(path)
}

pub fn load_signals(path: impl AsRef<Path>) -> Result<Vec<Signal>> {
    read_jsonl::<Signal>(path)
}

pub fn load_depth_results(path: impl AsRef<Path>) -> Result<Vec<DepthResult>> {
    read_jsonl::<DepthResult>(path)
}
