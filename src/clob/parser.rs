// src/clob/parser.rs

use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use rust_decimal::Decimal;
use serde_json::Value;

use crate::clob::message::{ClobBookLevel, ClobRawMessage};
use crate::types::{Book, Level, TokenId};

pub fn parse_clob_text(text: &str, received_at_ms: i64) -> Result<Vec<Book>> {
    let value: Value = serde_json::from_str(text).context("failed to parse CLOB text as JSON")?;
    parse_clob_value(&value, received_at_ms)
}

pub fn parse_clob_value(value: &Value, received_at_ms: i64) -> Result<Vec<Book>> {
    if let Some(array) = value.as_array() {
        let mut books = Vec::new();

        for item in array {
            books.extend(parse_clob_value(item, received_at_ms)?);
        }

        return Ok(books);
    }

    if let Some(data) = value.get("data") {
        return parse_clob_value(data, received_at_ms);
    }

    if let Some(payload) = value.get("payload") {
        return parse_clob_value(payload, received_at_ms);
    }

    parse_single_book(value, received_at_ms).map(|book| book.into_iter().collect())
}

fn parse_single_book(value: &Value, received_at_ms: i64) -> Result<Option<Book>> {
    let raw: ClobRawMessage =
        serde_json::from_value(value.clone()).context("failed to decode CLOB raw message")?;

    let Some(token_id) = extract_token_id(&raw) else {
        return Ok(None);
    };

    let bids = parse_levels_from_sources(&[raw.bids.as_ref(), raw.buys.as_ref()])?;

    let asks = parse_levels_from_sources(&[raw.asks.as_ref(), raw.sells.as_ref()])?;

    if bids.is_empty() && asks.is_empty() {
        return Ok(None);
    }

    let exchange_ts_ms = raw.timestamp.as_ref().and_then(parse_timestamp_ms);

    Ok(Some(Book::new(
        TokenId::from(token_id),
        bids,
        asks,
        exchange_ts_ms,
        received_at_ms,
    )))
}

fn extract_token_id(raw: &ClobRawMessage) -> Option<String> {
    raw.token_id
        .clone()
        .or_else(|| raw.market.clone())
        .filter(|value| !value.trim().is_empty())
}

fn parse_levels_from_sources(sources: &[Option<&Vec<ClobBookLevel>>]) -> Result<Vec<Level>> {
    let mut levels = Vec::new();

    for source in sources {
        let Some(source_levels) = source else {
            continue;
        };

        for level in *source_levels {
            if let Some(parsed) = parse_level(level)? {
                levels.push(parsed);
            }
        }
    }

    Ok(levels)
}

fn parse_level(level: &ClobBookLevel) -> Result<Option<Level>> {
    let Some(price_value) = &level.price else {
        return Ok(None);
    };

    let Some(size_value) = &level.size else {
        return Ok(None);
    };

    let price = parse_decimal_value(price_value)?;
    let size = parse_decimal_value(size_value)?;

    if price <= Decimal::ZERO || size <= Decimal::ZERO {
        return Ok(None);
    }

    Ok(Some(Level::new(price, size)))
}

fn parse_decimal_value(value: &Value) -> Result<Decimal> {
    match value {
        Value::String(text) => Decimal::from_str(text.trim())
            .with_context(|| format!("failed to parse decimal string: {text}")),

        Value::Number(number) => Decimal::from_str(&number.to_string())
            .with_context(|| format!("failed to parse decimal number: {number}")),

        other => Err(anyhow!("unsupported decimal JSON value: {other}")),
    }
}

fn parse_timestamp_ms(value: &Value) -> Option<i64> {
    if let Some(number) = value.as_i64() {
        return Some(normalize_timestamp_ms(number));
    }

    if let Some(number) = value.as_u64() {
        return i64::try_from(number).ok().map(normalize_timestamp_ms);
    }

    if let Some(text) = value.as_str() {
        if let Ok(number) = text.parse::<i64>() {
            return Some(normalize_timestamp_ms(number));
        }
    }

    None
}

fn normalize_timestamp_ms(value: i64) -> i64 {
    if value.abs() < 10_000_000_000 {
        value * 1_000
    } else {
        value
    }
}
