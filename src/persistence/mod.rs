// src/persistence/mod.rs

pub mod eval_writer;
pub mod jsonl;
pub mod opportunity_writer;
pub mod replay;
pub mod signal_writer;

pub use eval_writer::EvaluationWriter;

pub use jsonl::{append_jsonl, read_jsonl, write_jsonl};

pub use opportunity_writer::{OpportunityRecord, OpportunityWriter};

pub use replay::{
    load_depth_results, load_evaluation_rows, load_opportunity_records, load_signals,
};

pub use signal_writer::SignalWriter;
