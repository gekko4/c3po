// src/types/asset.rs

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported Polymarket crypto Up/Down assets.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "UPPERCASE")]
pub enum Asset {
    BTC,
    ETH,
    SOL,
    XRP,
}

impl Asset {
    pub const ALL: [Asset; 4] = [Asset::BTC, Asset::ETH, Asset::SOL, Asset::XRP];

    pub fn as_str(self) -> &'static str {
        match self {
            Asset::BTC => "BTC",
            Asset::ETH => "ETH",
            Asset::SOL => "SOL",
            Asset::XRP => "XRP",
        }
    }

    pub fn slug_prefix(self) -> &'static str {
        match self {
            Asset::BTC => "btc",
            Asset::ETH => "eth",
            Asset::SOL => "sol",
            Asset::XRP => "xrp",
        }
    }

    pub fn rtds_symbol(self) -> &'static str {
        match self {
            Asset::BTC => "btc/usd",
            Asset::ETH => "eth/usd",
            Asset::SOL => "sol/usd",
            Asset::XRP => "xrp/usd",
        }
    }

    pub fn from_slug_prefix(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "btc" => Some(Asset::BTC),
            "eth" => Some(Asset::ETH),
            "sol" => Some(Asset::SOL),
            "xrp" => Some(Asset::XRP),
            _ => None,
        }
    }

    pub fn from_rtds_symbol(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "btc/usd" => Some(Asset::BTC),
            "eth/usd" => Some(Asset::ETH),
            "sol/usd" => Some(Asset::SOL),
            "xrp/usd" => Some(Asset::XRP),
            _ => None,
        }
    }
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).as_str())
    }
}

impl FromStr for Asset {
    type Err = ParseAssetError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let normalized = input
            .trim()
            .to_ascii_uppercase()
            .replace('_', "")
            .replace('-', "")
            .replace(' ', "");

        match normalized.as_str() {
            "BTC" | "BITCOIN" | "XBT" => Ok(Asset::BTC),
            "ETH" | "ETHEREUM" => Ok(Asset::ETH),
            "SOL" | "SOLANA" => Ok(Asset::SOL),
            "XRP" | "RIPPLE" => Ok(Asset::XRP),
            _ => Err(ParseAssetError {
                value: input.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseAssetError {
    pub value: String,
}

impl fmt::Display for ParseAssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unsupported asset: {}", self.value)
    }
}

impl std::error::Error for ParseAssetError {}