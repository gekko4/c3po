// src/tick_store/replay.rs

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use anyhow::{Context, Result};

use crate::tick_store::store::TickStore;
use crate::types::tick::Tick;

pub fn load_ticks_jsonl(path: impl AsRef<Path>) -> Result<Vec<Tick>> {
    let path = path.as_ref();

    let file = File::open(path)
        .with_context(|| format!("failed to open tick replay file: {}", path.display()))?;

    let reader = BufReader::new(file);
    let mut ticks = Vec::new();

    for (line_no, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!(
                "failed to read line {} from tick replay file: {}",
                line_no + 1,
                path.display()
            )
        })?;

        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let tick: Tick = serde_json::from_str(trimmed).with_context(|| {
            format!(
                "failed to parse tick JSON on line {} from file: {}",
                line_no + 1,
                path.display()
            )
        })?;

        ticks.push(tick);
    }

    Ok(ticks)
}

pub fn replay_ticks_into_store(path: impl AsRef<Path>, store: &mut TickStore) -> Result<usize> {
    let ticks = load_ticks_jsonl(path)?;

    let mut inserted = 0usize;

    for tick in ticks {
        store.insert_tick(tick);
        inserted += 1;
    }

    Ok(inserted)
}

pub fn write_ticks_jsonl<'a, I>(path: impl AsRef<Path>, ticks: I) -> Result<usize>
where
    I: IntoIterator<Item = &'a Tick>,
{
    let path = path.as_ref();

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .with_context(|| format!("failed to open tick output file: {}", path.display()))?;

    let mut written = 0usize;

    for tick in ticks {
        let json = serde_json::to_string(tick).context("failed to serialize tick as JSON")?;

        writeln!(file, "{json}")
            .with_context(|| format!("failed to write tick to file: {}", path.display()))?;

        written += 1;
    }

    Ok(written)
}
