// src/types/timeframe.rs

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported Up/Down market timeframes.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Timeframe {
    FiveMin,
    FifteenMin,
    OneHour,
    FourHour,
    OneDay,
}

impl Timeframe {
    pub const ALL: [Timeframe; 5] = [
        Timeframe::FiveMin,
        Timeframe::FifteenMin,
        Timeframe::OneHour,
        Timeframe::FourHour,
        Timeframe::OneDay,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Timeframe::FiveMin => "5m",
            Timeframe::FifteenMin => "15m",
            Timeframe::OneHour => "1h",
            Timeframe::FourHour => "4h",
            Timeframe::OneDay => "1d",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Timeframe::FiveMin => "FiveMin",
            Timeframe::FifteenMin => "FifteenMin",
            Timeframe::OneHour => "OneHour",
            Timeframe::FourHour => "FourHour",
            Timeframe::OneDay => "OneDay",
        }
    }

    pub fn seconds(self) -> i64 {
        match self {
            Timeframe::FiveMin => 5 * 60,
            Timeframe::FifteenMin => 15 * 60,
            Timeframe::OneHour => 60 * 60,
            Timeframe::FourHour => 4 * 60 * 60,
            Timeframe::OneDay => 24 * 60 * 60,
        }
    }

    pub fn milliseconds(self) -> i64 {
        self.seconds() * 1_000
    }

    pub fn is_shorter_than(self, other: Timeframe) -> bool {
        self.seconds() < other.seconds()
    }

    pub fn is_longer_than(self, other: Timeframe) -> bool {
        self.seconds() > other.seconds()
    }

    pub fn ordered_pair(a: Timeframe, b: Timeframe) -> Option<(Timeframe, Timeframe)> {
        if a == b {
            None
        } else if a.seconds() < b.seconds() {
            Some((a, b))
        } else {
            Some((b, a))
        }
    }
}

impl fmt::Display for Timeframe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).as_str())
    }
}

impl FromStr for Timeframe {
    type Err = ParseTimeframeError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let normalized = input
            .trim()
            .to_ascii_lowercase()
            .replace('_', "")
            .replace('-', "")
            .replace(' ', "");

        match normalized.as_str() {
            "5m" | "5min" | "5mins" | "fivemin" | "fivemins" | "fiveminute" | "fiveminutes" => {
                Ok(Timeframe::FiveMin)
            }
            "15m"
            | "15min"
            | "15mins"
            | "fifteenmin"
            | "fifteenmins"
            | "fifteenminute"
            | "fifteenminutes" => Ok(Timeframe::FifteenMin),
            "1h" | "1hr" | "1hour" | "onehour" => Ok(Timeframe::OneHour),
            "4h" | "4hr" | "4hour" | "fourhour" => Ok(Timeframe::FourHour),
            "1d" | "1day" | "oneday" | "daily" => Ok(Timeframe::OneDay),
            _ => Err(ParseTimeframeError {
                value: input.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseTimeframeError {
    pub value: String,
}

impl fmt::Display for ParseTimeframeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unsupported timeframe: {}", self.value)
    }
}

impl std::error::Error for ParseTimeframeError {}