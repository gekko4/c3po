// src/market_scanner/parser.rs

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{Asset, ConditionId, Market, MarketSlug, Timeframe, TokenId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMarket {
    #[serde(default)]
    pub slug: Option<String>,

    #[serde(default, alias = "conditionId", alias = "condition_id")]
    pub condition_id: Option<String>,

    #[serde(default)]
    pub question: Option<String>,

    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub active: Option<bool>,

    #[serde(default)]
    pub closed: Option<bool>,

    #[serde(
        default,
        alias = "eventStartTime",
        alias = "startDate",
        alias = "start_time"
    )]
    pub open_time: Option<Value>,

    #[serde(default, alias = "endDate", alias = "end_time", alias = "endTimestamp")]
    pub end_time: Option<Value>,

    #[serde(default, alias = "upTokenId", alias = "up_token_id")]
    pub up_token_id: Option<String>,

    #[serde(default, alias = "downTokenId", alias = "down_token_id")]
    pub down_token_id: Option<String>,

    #[serde(default, alias = "clobTokenIds", alias = "clob_token_ids")]
    pub clob_token_ids: Option<Value>,

    #[serde(default)]
    pub tokens: Option<Vec<RawOutcomeToken>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawOutcomeToken {
    #[serde(default, alias = "token_id", alias = "tokenId", alias = "id")]
    pub token_id: Option<String>,

    #[serde(default, alias = "outcome")]
    pub outcome: Option<String>,
}

pub fn parse_markets(raw_markets: &[RawMarket], seen_ms: i64) -> Result<Vec<Market>> {
    let mut parsed = Vec::new();

    for raw in raw_markets {
        if let Some(market) = parse_market(raw, seen_ms)? {
            parsed.push(market);
        }
    }

    Ok(parsed)
}

pub fn parse_market(raw: &RawMarket, seen_ms: i64) -> Result<Option<Market>> {
    let Some(slug_text) = raw.slug.as_ref() else {
        return Ok(None);
    };

    let Some(condition_id_text) = raw.condition_id.as_ref() else {
        return Ok(None);
    };

    let Some(asset) = parse_asset(raw) else {
        return Ok(None);
    };

    let Some(timeframe) = parse_timeframe(raw) else {
        return Ok(None);
    };

    let Some(open_ms) = parse_time_ms(raw.open_time.as_ref()) else {
        return Ok(None);
    };

    let Some(end_ms) = parse_time_ms(raw.end_time.as_ref()) else {
        return Ok(None);
    };

    let Some((up_token_id, down_token_id)) = parse_token_ids(raw) else {
        return Ok(None);
    };

    if open_ms >= end_ms {
        return Err(anyhow!("invalid market time window for slug {}", slug_text));
    }

    Ok(Some(Market::new(
        asset,
        timeframe,
        MarketSlug::from(slug_text.clone()),
        ConditionId::from(condition_id_text.clone()),
        TokenId::from(up_token_id),
        TokenId::from(down_token_id),
        open_ms,
        end_ms,
        raw.active.unwrap_or(true),
        raw.closed.unwrap_or(false),
        seen_ms,
    )))
}

fn parse_asset(raw: &RawMarket) -> Option<Asset> {
    if let Some(slug) = &raw.slug {
        let first = slug.split('-').next().unwrap_or_default();
        if let Some(asset) = Asset::from_slug_prefix(first) {
            return Some(asset);
        }
    }

    let text = searchable_text(raw);
    for asset in Asset::ALL {
        if text.contains(&asset.slug_prefix().to_ascii_lowercase())
            || text.contains(&asset.as_str().to_ascii_lowercase())
        {
            return Some(asset);
        }
    }

    None
}

fn parse_timeframe(raw: &RawMarket) -> Option<Timeframe> {
    let text = searchable_text(raw);

    if contains_token(&text, "5m") {
        return Some(Timeframe::FiveMin);
    }

    if contains_token(&text, "15m") {
        return Some(Timeframe::FifteenMin);
    }

    if contains_token(&text, "1h") {
        return Some(Timeframe::OneHour);
    }

    if contains_token(&text, "4h") {
        return Some(Timeframe::FourHour);
    }

    if contains_token(&text, "1d") {
        return Some(Timeframe::OneDay);
    }

    None
}

fn searchable_text(raw: &RawMarket) -> String {
    format!(
        "{} {} {}",
        raw.slug.as_deref().unwrap_or_default(),
        raw.question.as_deref().unwrap_or_default(),
        raw.title.as_deref().unwrap_or_default(),
    )
    .to_ascii_lowercase()
}

fn contains_token(text: &str, token: &str) -> bool {
    text.contains(token)
        || text.contains(&token.replace('m', "-minutes"))
        || text.contains(&token.replace('h', "-hour"))
        || text.contains(&token.replace('d', "-day"))
}

fn parse_time_ms(value: Option<&Value>) -> Option<i64> {
    let value = value?;

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

        if let Ok(dt) = DateTime::parse_from_rfc3339(text) {
            return Some(dt.with_timezone(&Utc).timestamp_millis());
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

fn parse_token_ids(raw: &RawMarket) -> Option<(String, String)> {
    if let (Some(up), Some(down)) = (&raw.up_token_id, &raw.down_token_id) {
        return Some((up.clone(), down.clone()));
    }

    if let Some(tokens) = &raw.tokens {
        let mut up = None;
        let mut down = None;

        for token in tokens {
            let outcome = token
                .outcome
                .as_deref()
                .unwrap_or_default()
                .to_ascii_lowercase();

            if outcome == "up" || outcome.contains("up") {
                up = token.token_id.clone();
            }

            if outcome == "down" || outcome.contains("down") {
                down = token.token_id.clone();
            }
        }

        if let (Some(up), Some(down)) = (up, down) {
            return Some((up, down));
        }
    }

    parse_clob_token_ids(raw.clob_token_ids.as_ref())
}

fn parse_clob_token_ids(value: Option<&Value>) -> Option<(String, String)> {
    let value = value?;

    if let Some(array) = value.as_array() {
        return two_strings_from_array(array);
    }

    if let Some(text) = value.as_str() {
        if let Ok(parsed) = serde_json::from_str::<Value>(text) {
            if let Some(array) = parsed.as_array() {
                return two_strings_from_array(array);
            }
        }
    }

    None
}

fn two_strings_from_array(array: &[Value]) -> Option<(String, String)> {
    let first = array.get(0)?.as_str()?.to_string();
    let second = array.get(1)?.as_str()?.to_string();

    Some((first, second))
}
