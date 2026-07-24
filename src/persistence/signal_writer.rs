// src/persistence/signal_writer.rs

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::persistence::jsonl::{append_jsonl, write_jsonl};
use crate::types::signal::Signal;

#[derive(Debug, Clone)]
pub struct SignalWriter {
    path: PathBuf,
}

impl SignalWriter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(&self, signal: &Signal) -> Result<()> {
        append_jsonl(&self.path, signal)
    }

    pub fn append_many<'a, I>(&self, signals: I) -> Result<usize>
    where
        I: IntoIterator<Item = &'a Signal>,
    {
        let mut count = 0usize;

        for signal in signals {
            self.append(signal)?;
            count += 1;
        }

        Ok(count)
    }

    pub fn overwrite<'a, I>(&self, signals: I) -> Result<usize>
    where
        I: IntoIterator<Item = &'a Signal>,
    {
        write_jsonl::<Signal, _>(&self.path, signals)
    }
}
