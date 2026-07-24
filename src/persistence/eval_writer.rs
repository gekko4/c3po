// src/persistence/eval_writer.rs

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::persistence::jsonl::{append_jsonl, write_jsonl};
use crate::types::evaluation::EvaluationRow;

#[derive(Debug, Clone)]
pub struct EvaluationWriter {
    path: PathBuf,
}

impl EvaluationWriter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(&self, row: &EvaluationRow) -> Result<()> {
        append_jsonl(&self.path, row)
    }

    pub fn append_many<'a, I>(&self, rows: I) -> Result<usize>
    where
        I: IntoIterator<Item = &'a EvaluationRow>,
    {
        let mut count = 0usize;

        for row in rows {
            self.append(row)?;
            count += 1;
        }

        Ok(count)
    }

    pub fn overwrite<'a, I>(&self, rows: I) -> Result<usize>
    where
        I: IntoIterator<Item = &'a EvaluationRow>,
    {
        write_jsonl::<EvaluationRow, _>(&self.path, rows)
    }
}
