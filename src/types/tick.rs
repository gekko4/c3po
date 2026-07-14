// src/types/tick.rs

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// RTDS symbol wrapper, for example "btc/usd".
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RtdsSymbol(String);

impl RtdsSymbol {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn normalized(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().trim().to_ascii_lowercase())
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl fmt::Display for RtdsSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for RtdsSymbol {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for RtdsSymbol {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Raw/normalized RTDS Chainlink tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tick {
    pub symbol: RtdsSymbol,

    /// RTDS tick timestamp.
    pub timestamp_ms: i64,

    /// Original value field as a string for auditability.
    pub raw_value: String,

    /// Optional full-accuracy RTDS value.
    pub full_accuracy_value: Option<String>,

    /// Decimal value after normalization.
    pub normalized_value: Decimal,

    /// Local receive timestamp.
    pub received_at_ms: i64,

    /// Source label, usually "rtds".
    pub source: String,
}

impl Tick {
    pub fn new(
        symbol: RtdsSymbol,
        timestamp_ms: i64,
        raw_value: impl Into<String>,
        full_accuracy_value: Option<String>,
        normalized_value: Decimal,
        received_at_ms: i64,
        source: impl Into<String>,
    ) -> Self {
        Self {
            symbol,
            timestamp_ms,
            raw_value: raw_value.into(),
            full_accuracy_value,
            normalized_value,
            received_at_ms,
            source: source.into(),
        }
    }

    pub fn exact_key(&self) -> (&RtdsSymbol, i64) {
        (&self.symbol, self.timestamp_ms)
    }

    pub fn is_exact_open_tick(&self, open_ms: i64) -> bool {
        self.timestamp_ms == open_ms
    }
}