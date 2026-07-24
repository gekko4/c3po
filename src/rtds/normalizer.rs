// src/rtds/normalizer.rs

use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use rust_decimal::Decimal;

use crate::rtds::message::RtdsPayload;
use crate::types::tick::{RtdsSymbol, Tick};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedRtdsTick {
    pub symbol: RtdsSymbol,
    pub timestamp_ms: i64,
    pub raw_value: String,
    pub full_accuracy_value: Option<String>,
    pub normalized_value: Decimal,
}

impl NormalizedRtdsTick {
    pub fn into_tick(self, received_at_ms: i64) -> Tick {
        Tick::from_rtds(
            self.symbol.as_str(),
            self.timestamp_ms,
            self.raw_value,
            self.full_accuracy_value,
            self.normalized_value,
            received_at_ms,
        )
    }
}

pub fn normalize_rtds_payload(payload: &RtdsPayload) -> Result<NormalizedRtdsTick> {
    let symbol = RtdsSymbol::normalized(&payload.symbol);
    let timestamp_ms = normalize_timestamp_ms(payload.timestamp);
    let raw_value = payload.raw_value_string();

    let normalized_value = normalize_value(&raw_value, payload.full_accuracy_value.as_deref())
        .with_context(|| {
            format!(
                "failed to normalize RTDS value for symbol {} at {}",
                symbol, timestamp_ms
            )
        })?;

    Ok(NormalizedRtdsTick {
        symbol,
        timestamp_ms,
        raw_value,
        full_accuracy_value: payload.full_accuracy_value.clone(),
        normalized_value,
    })
}

fn normalize_timestamp_ms(timestamp: i64) -> i64 {
    if timestamp.abs() < 10_000_000_000 {
        timestamp * 1_000
    } else {
        timestamp
    }
}

fn normalize_value(raw_value: &str, full_accuracy_value: Option<&str>) -> Result<Decimal> {
    if let Ok(value) = Decimal::from_str(raw_value.trim()) {
        return Ok(value);
    }

    if let Some(full_accuracy_value) = full_accuracy_value {
        return normalize_full_accuracy_value(full_accuracy_value);
    }

    Err(anyhow!("unable to parse RTDS value: {raw_value}"))
}

fn normalize_full_accuracy_value(value: &str) -> Result<Decimal> {
    let integer = value
        .trim()
        .parse::<i128>()
        .with_context(|| format!("failed to parse full_accuracy_value: {value}"))?;

    Ok(Decimal::from_i128_with_scale(integer, 18))
}
