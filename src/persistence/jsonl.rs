// src/persistence/jsonl.rs

use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn append_jsonl<T>(path: impl AsRef<Path>, item: &T) -> Result<()>
where
    T: Serialize,
{
    let path = path.as_ref();
    ensure_parent_dir(path)?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open JSONL append file: {}", path.display()))?;

    let json = serde_json::to_string(item).context("failed to serialize JSONL item")?;

    writeln!(file, "{json}")
        .with_context(|| format!("failed to append JSONL item to {}", path.display()))?;

    Ok(())
}

pub fn write_jsonl<'a, T, I>(path: impl AsRef<Path>, items: I) -> Result<usize>
where
    T: Serialize + 'a,
    I: IntoIterator<Item = &'a T>,
{
    let path = path.as_ref();
    ensure_parent_dir(path)?;

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .with_context(|| format!("failed to open JSONL write file: {}", path.display()))?;

    let mut written = 0usize;

    for item in items {
        let json = serde_json::to_string(item).context("failed to serialize JSONL item")?;

        writeln!(file, "{json}")
            .with_context(|| format!("failed to write JSONL item to {}", path.display()))?;

        written += 1;
    }

    Ok(written)
}

pub fn read_jsonl<T>(path: impl AsRef<Path>) -> Result<Vec<T>>
where
    T: DeserializeOwned,
{
    let path = path.as_ref();

    let file = File::open(path)
        .with_context(|| format!("failed to open JSONL file: {}", path.display()))?;

    let reader = BufReader::new(file);
    let mut rows = Vec::new();

    for (line_no, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!(
                "failed to read line {} from JSONL file {}",
                line_no + 1,
                path.display()
            )
        })?;

        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let row = serde_json::from_str::<T>(trimmed).with_context(|| {
            format!(
                "failed to deserialize JSONL line {} from {}",
                line_no + 1,
                path.display()
            )
        })?;

        rows.push(row);
    }

    Ok(rows)
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            create_dir_all(parent)
                .with_context(|| format!("failed to create directory {}", parent.display()))?;
        }
    }

    Ok(())
}
